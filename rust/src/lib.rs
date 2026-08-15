//! wasserspiegel-core: Rust core for the Sailfish OS Wasserspiegel app.
//!
//! All networking, parsing, caching and graph math lives here; the C++
//! layer is thin glue that calls into the cxx bridge from a worker
//! thread and forwards results to QML.

pub mod cache;
pub mod client;
pub mod domain;
pub mod error;
pub mod graph;
pub mod models;

use cache::Cache;
use client::Client;
use domain::{MeasurementPoint, StationMetrics, StationSummary};
use error::{CoreError, CoreResult};
use std::sync::{Arc, RwLock};

/// Bridge types and entry points exposed to C++ under namespace
/// `wasserspiegel`. All functions are blocking; call off the UI thread.
#[cxx::bridge(namespace = "wasserspiegel")]
mod ffi {
    struct FfiMeasurementPoint {
        pub timestamp_ms: i64,
        pub value: f64,
    }

    struct FfiStationSummary {
        pub id: String,
        pub name: String,
        pub water: String,
        pub km: f64,
        pub latitude: f64,
        pub longitude: f64,
    }

    struct FfiStationMetrics {
        pub station_id: String,
        pub station_name: String,
        pub water: String,
        pub km: f64,
        pub latitude: f64,
        pub longitude: f64,
        pub current_level: f64,
        pub current_timestamp_ms: i64,
        pub change_1day: f64,
        pub change_3day: f64,
        pub change_7day: f64,
        pub unit: String,
        pub history: Vec<FfiMeasurementPoint>,
        pub fetched_at_ms: i64,
        /// true when this payload came from the local cache
        pub from_cache: bool,
    }

    struct FfiSeriesRange {
        pub min: f64,
        pub max: f64,
    }

    extern "Rust" {
        type CoreClient;

        /// Creates a client. `api_base` e.g. "https://host", `token` the
        /// bearer token, `cache_dir` a writable directory for offline data.
        fn new_client(
            api_base: String,
            token: String,
            cache_dir: String,
        ) -> Result<Box<CoreClient>>;

        /// Apply changed settings (takes effect immediately).
        fn set_config(self: &CoreClient, api_base: String, token: String) -> Result<()>;

        /// GET /stations/{id}; refreshes the cache on success.
        fn fetch_station(self: &CoreClient, station_id: &str) -> Result<FfiStationMetrics>;

        /// Last cached dashboard payload for a station (offline path).
        fn load_cached_station(self: &CoreClient, station_id: &str) -> Result<FfiStationMetrics>;

        /// GET /stations - full list; refreshes the cache on success.
        fn fetch_stations(self: &CoreClient) -> Result<Vec<FfiStationSummary>>;

        /// Cached full station list (offline path for the picker).
        fn load_cached_stations(self: &CoreClient) -> Result<Vec<FfiStationSummary>>;

        /// Case-insensitive substring filter over name/water/id,
        /// capped at `limit` results. Runs locally.
        fn filter_stations(
            list: &[FfiStationSummary],
            query: &str,
            limit: usize,
        ) -> Vec<FfiStationSummary>;

        /// Slice history to the last `hours_back` hours and downsample
        /// to at most `max_points` (endpoints always kept).
        fn slice_series(
            history: &[FfiMeasurementPoint],
            hours_back: i64,
            max_points: usize,
        ) -> Vec<FfiMeasurementPoint>;

        /// Min/max value of a series (guarded against flat lines).
        fn series_range(history: &[FfiMeasurementPoint]) -> FfiSeriesRange;
    }
}

pub struct CoreClient {
    /// Arc so network calls snapshot the client and never hold the lock
    /// (nor block `set_config`) for the duration of blocking I/O.
    client: RwLock<Arc<Client>>,
    cache: Cache,
}

/// Recover from lock poisoning instead of panicking on every later call -
/// one panicked worker must not take down the whole app.
fn read_client(lock: &RwLock<Arc<Client>>) -> Arc<Client> {
    lock.read().unwrap_or_else(|e| e.into_inner()).clone()
}

impl From<MeasurementPoint> for ffi::FfiMeasurementPoint {
    fn from(p: MeasurementPoint) -> Self {
        ffi::FfiMeasurementPoint {
            timestamp_ms: p.timestamp_ms,
            value: p.value,
        }
    }
}

impl From<&StationSummary> for ffi::FfiStationSummary {
    fn from(s: &StationSummary) -> Self {
        ffi::FfiStationSummary {
            id: s.id.clone(),
            name: s.name.clone(),
            water: s.water.clone(),
            km: s.km,
            latitude: s.latitude,
            longitude: s.longitude,
        }
    }
}

impl StationMetrics {
    fn into_ffi(self, from_cache: bool) -> ffi::FfiStationMetrics {
        ffi::FfiStationMetrics {
            station_id: self.station_id,
            station_name: self.station_name,
            water: self.water,
            km: self.km,
            latitude: self.latitude,
            longitude: self.longitude,
            current_level: self.current_level,
            current_timestamp_ms: self.current_timestamp_ms,
            change_1day: self.change_1day,
            change_3day: self.change_3day,
            change_7day: self.change_7day,
            unit: self.unit,
            history: self.history.into_iter().map(Into::into).collect(),
            fetched_at_ms: self.fetched_at_ms,
            from_cache,
        }
    }
}

fn new_client(api_base: String, token: String, cache_dir: String) -> CoreResult<Box<CoreClient>> {
    let client = Client::new(api_base, token)?;
    let mut cache = Cache::default();
    cache.set_dir(cache_dir);
    Ok(Box::new(CoreClient {
        client: RwLock::new(Arc::new(client)),
        cache,
    }))
}

impl CoreClient {
    fn set_config(&self, api_base: String, token: String) -> CoreResult<()> {
        // validate first; on error the working client stays in place
        let next = Client::new(api_base, token)?;
        *self.client.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(next);
        Ok(())
    }

    fn fetch_station(&self, station_id: &str) -> CoreResult<ffi::FfiStationMetrics> {
        let client = read_client(&self.client);
        let metrics = client.fetch_station(station_id)?;
        if let Err(e) = self.cache.store_station_metrics(&metrics) {
            log_cache_failure("station metrics", &e);
        }
        Ok(metrics.into_ffi(false))
    }

    fn load_cached_station(&self, station_id: &str) -> CoreResult<ffi::FfiStationMetrics> {
        let (metrics, _) = self.cache.load_station_metrics(station_id)?;
        Ok(metrics.into_ffi(true))
    }

    fn fetch_stations(&self) -> CoreResult<Vec<ffi::FfiStationSummary>> {
        let client = read_client(&self.client);
        let stations = client.list_stations()?;
        if let Err(e) = self.cache.store_stations(&stations) {
            log_cache_failure("station list", &e);
        }
        Ok(stations.iter().map(Into::into).collect())
    }

    fn load_cached_stations(&self) -> CoreResult<Vec<ffi::FfiStationSummary>> {
        let (stations, _) = self.cache.load_stations()?;
        Ok(stations.iter().map(Into::into).collect())
    }
}

fn filter_stations(
    list: &[ffi::FfiStationSummary],
    query: &str,
    limit: usize,
) -> Vec<ffi::FfiStationSummary> {
    let q = query.trim().to_lowercase();
    let mut out = Vec::new();
    if q.is_empty() {
        return out;
    }
    for s in list {
        let hay = format!("{} {} {}", s.name, s.water, s.id).to_lowercase();
        if hay.contains(&q) {
            out.push(ffi::FfiStationSummary {
                id: s.id.clone(),
                name: s.name.clone(),
                water: s.water.clone(),
                km: s.km,
                latitude: s.latitude,
                longitude: s.longitude,
            });
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

fn slice_series(
    history: &[ffi::FfiMeasurementPoint],
    hours_back: i64,
    max_points: usize,
) -> Vec<ffi::FfiMeasurementPoint> {
    let owned: Vec<MeasurementPoint> = history
        .iter()
        .map(|p| MeasurementPoint {
            timestamp_ms: p.timestamp_ms,
            value: p.value,
        })
        .collect();
    graph::slice_series(&owned, hours_back, max_points)
        .into_iter()
        .map(Into::into)
        .collect()
}

fn series_range(history: &[ffi::FfiMeasurementPoint]) -> ffi::FfiSeriesRange {
    let owned: Vec<MeasurementPoint> = history
        .iter()
        .map(|p| MeasurementPoint {
            timestamp_ms: p.timestamp_ms,
            value: p.value,
        })
        .collect();
    let (min, max) = graph::value_range(&owned);
    ffi::FfiSeriesRange { min, max }
}

fn log_cache_failure(what: &str, e: &CoreError) {
    eprintln!("wasserspiegel-core: failed to store {what} cache: {e}");
}
