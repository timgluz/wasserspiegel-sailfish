//! File cache for the last station list and per-station dashboard payloads.
//! Written atomically (unique tmp file + rename) so concurrent writers or
//! a killed app never leave half-written JSON behind.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::domain::{StationMetrics, StationSummary};
use crate::error::{CoreError, CoreResult};

/// Distinguishes simultaneous tmp files from concurrent workers.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
struct CachedEnvelope<T> {
    fetched_at_ms: i64,
    payload: T,
}

#[derive(Serialize, Deserialize)]
struct SerializablePoint {
    timestamp_ms: i64,
    value: f64,
}

#[derive(Serialize, Deserialize)]
struct SerializableMetrics {
    station_id: String,
    station_name: String,
    water: String,
    km: f64,
    latitude: f64,
    longitude: f64,
    current_level: f64,
    current_timestamp_ms: i64,
    change_1day: f64,
    change_3day: f64,
    change_7day: f64,
    unit: String,
    history: Vec<SerializablePoint>,
    fetched_at_ms: i64,
}

impl From<&StationMetrics> for SerializableMetrics {
    fn from(m: &StationMetrics) -> Self {
        SerializableMetrics {
            station_id: m.station_id.clone(),
            station_name: m.station_name.clone(),
            water: m.water.clone(),
            km: m.km,
            latitude: m.latitude,
            longitude: m.longitude,
            current_level: m.current_level,
            current_timestamp_ms: m.current_timestamp_ms,
            change_1day: m.change_1day,
            change_3day: m.change_3day,
            change_7day: m.change_7day,
            unit: m.unit.clone(),
            history: m
                .history
                .iter()
                .map(|p| SerializablePoint {
                    timestamp_ms: p.timestamp_ms,
                    value: p.value,
                })
                .collect(),
            fetched_at_ms: m.fetched_at_ms,
        }
    }
}

impl From<SerializableMetrics> for StationMetrics {
    fn from(s: SerializableMetrics) -> Self {
        StationMetrics {
            station_id: s.station_id,
            station_name: s.station_name,
            water: s.water,
            km: s.km,
            latitude: s.latitude,
            longitude: s.longitude,
            current_level: s.current_level,
            current_timestamp_ms: s.current_timestamp_ms,
            change_1day: s.change_1day,
            change_3day: s.change_3day,
            change_7day: s.change_7day,
            unit: s.unit,
            history: s
                .history
                .into_iter()
                .map(|p| crate::domain::MeasurementPoint {
                    timestamp_ms: p.timestamp_ms,
                    value: p.value,
                })
                .collect(),
            fetched_at_ms: s.fetched_at_ms,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerializableSummary {
    id: String,
    name: String,
    water: String,
    km: f64,
    latitude: f64,
    longitude: f64,
}

#[derive(Default)]
pub struct Cache {
    dir: Option<PathBuf>,
}

impl Cache {
    pub fn set_dir(&mut self, dir: String) {
        self.dir = Some(PathBuf::from(dir));
    }

    // ---- stations list ----

    pub fn store_stations(&self, stations: &[StationSummary]) -> CoreResult<i64> {
        let now = chrono::Utc::now().timestamp_millis();
        let env = CachedEnvelope {
            fetched_at_ms: now,
            payload: stations
                .iter()
                .map(|s| SerializableSummary {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    water: s.water.clone(),
                    km: s.km,
                    latitude: s.latitude,
                    longitude: s.longitude,
                })
                .collect::<Vec<_>>(),
        };
        self.write_json("stations.json", &env)?;
        Ok(now)
    }

    pub fn load_stations(&self) -> CoreResult<(Vec<StationSummary>, i64)> {
        let env: CachedEnvelope<Vec<SerializableSummary>> = self.read_json("stations.json")?;
        Ok((
            env.payload
                .into_iter()
                .map(|s| StationSummary {
                    id: s.id,
                    name: s.name,
                    water: s.water,
                    km: s.km,
                    latitude: s.latitude,
                    longitude: s.longitude,
                })
                .collect(),
            env.fetched_at_ms,
        ))
    }

    // ---- station metrics ----

    pub fn store_station_metrics(&self, metrics: &StationMetrics) -> CoreResult<()> {
        let env = CachedEnvelope {
            fetched_at_ms: metrics.fetched_at_ms,
            payload: SerializableMetrics::from(metrics),
        };
        let fname = format!("station_{}.json", sanitize_id(&metrics.station_id));
        self.write_json(&fname, &env)
    }

    pub fn load_station_metrics(&self, station_id: &str) -> CoreResult<(StationMetrics, i64)> {
        let fname = format!("station_{}.json", sanitize_id(station_id));
        let env: CachedEnvelope<SerializableMetrics> = self.read_json(&fname)?;
        let metrics = StationMetrics::from(env.payload);
        Ok((metrics.clone(), metrics.fetched_at_ms))
    }

    // ---- helpers ----

    fn path_for(&self, name: &str) -> CoreResult<PathBuf> {
        let dir = self
            .dir
            .as_deref()
            .ok_or_else(|| CoreError::Cache("cache directory not configured".into()))?;
        Ok(dir.join(name))
    }

    fn write_json<T: Serialize>(&self, name: &str, value: &T) -> CoreResult<()> {
        let path = self.path_for(name)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CoreError::Cache(format!("create dir for {name}: {e}")))?;
        }
        let tmp = path.with_extension(format!(
            "tmp.{}.{}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        if let Err(e) = (|| {
            let data = serde_json::to_vec(value)?;
            fs::write(&tmp, data).map_err(|e| CoreError::Cache(e.to_string()))?;
            fs::rename(&tmp, &path).map_err(|e| CoreError::Cache(e.to_string()))
        })() {
            let _ = fs::remove_file(&tmp);
            return Err(CoreError::Cache(format!("failed to store {name}: {e}")));
        }
        Ok(())
    }

    fn read_json<T: for<'de> Deserialize<'de>>(&self, name: &str) -> CoreResult<T> {
        let path = self.path_for(name)?;
        let data = fs::read(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => CoreError::NotFound(format!("no cached {name}")),
            _ => CoreError::Cache(format!("failed to read {name}: {e}")),
        })?;
        Ok(serde_json::from_slice(&data)?)
    }
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::MeasurementPoint;

    fn sample_metrics() -> StationMetrics {
        StationMetrics {
            station_id: "57090802-c51a-4d09-8340-b4453cd0e1f5".into(),
            station_name: "MANNHEIM".into(),
            water: "RHEIN".into(),
            km: 424.7,
            latitude: 49.4,
            longitude: 8.4,
            current_level: 71.0,
            current_timestamp_ms: 1_755_000_000_000,
            change_1day: -3.0,
            change_3day: 3.5,
            change_7day: 16.0,
            unit: "cm".into(),
            history: vec![
                MeasurementPoint {
                    timestamp_ms: 1,
                    value: 80.0,
                },
                MeasurementPoint {
                    timestamp_ms: 2,
                    value: 75.5,
                },
            ],
            fetched_at_ms: 1_755_000_600_000,
        }
    }

    #[test]
    fn roundtrip_metrics_and_stations() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = {
            let mut c = Cache::default();
            c.set_dir(tmp.path().to_string_lossy().into());
            c
        };

        let m = sample_metrics();
        cache.store_station_metrics(&m).unwrap();
        let (loaded, fetched_at) = cache.load_station_metrics(&m.station_id).unwrap();
        assert_eq!(loaded.station_name, "MANNHEIM");
        assert_eq!(loaded.history.len(), 2);
        assert_eq!(loaded.history[1].value, 75.5);
        assert_eq!(fetched_at, m.fetched_at_ms);

        let stations = vec![StationSummary {
            id: "x".into(),
            name: "BONN".into(),
            water: "RHEIN".into(),
            km: 654.3,
            latitude: 1.0,
            longitude: 2.0,
        }];
        cache.store_stations(&stations).unwrap();
        let (loaded, _) = cache.load_stations().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "BONN");
    }

    #[test]
    fn missing_cache_is_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = {
            let mut c = Cache::default();
            c.set_dir(tmp.path().to_string_lossy().into());
            c
        };
        assert!(matches!(
            cache.load_station_metrics("nope"),
            Err(CoreError::NotFound(_))
        ));
    }
}
