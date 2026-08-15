//! Processed, FFI-friendly domain types derived from raw API payloads.

use chrono::DateTime;

use crate::models::{RawMeasurement, RawStation, RawTrend, StationDetailResponse};

/// One point of the level history, timestamp in epoch milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasurementPoint {
    pub timestamp_ms: i64,
    pub value: f64,
}

/// Fully processed dashboard payload for a single station.
#[derive(Debug, Clone)]
pub struct StationMetrics {
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
    pub history: Vec<MeasurementPoint>,
    pub fetched_at_ms: i64,
}

/// Compact station descriptor for pickers and lists.
#[derive(Debug, Clone)]
pub struct StationSummary {
    pub id: String,
    pub name: String,
    pub water: String,
    pub km: f64,
    pub latitude: f64,
    pub longitude: f64,
}

pub fn parse_timestamp_ms(ts: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

impl From<&RawStation> for StationSummary {
    fn from(s: &RawStation) -> Self {
        StationSummary {
            id: s.uuid.clone(),
            name: s.longname.clone(),
            water: s.water.longname.clone(),
            km: s.km,
            latitude: s.latitude,
            longitude: s.longitude,
        }
    }
}

fn trend_value(trend: &RawTrend, key: &str) -> f64 {
    let m = match key {
        "p1d" => trend.p1d.as_ref(),
        "p3d" => trend.p3d.as_ref(),
        _ => trend.p7d.as_ref(),
    };
    m.map(|RawMeasurement { value, .. }| *value)
        .unwrap_or(f64::NAN)
}

impl StationMetrics {
    pub fn from_detail(detail: &StationDetailResponse, fetched_at_ms: i64) -> Self {
        let station = &detail.station;
        let wl = &detail.water_level;

        let history: Vec<MeasurementPoint> = wl
            .measurements
            .iter()
            .filter_map(|m| {
                parse_timestamp_ms(&m.timestamp).map(|ts| MeasurementPoint {
                    timestamp_ms: ts,
                    value: m.value,
                })
            })
            .collect();

        let latest = wl.latest.as_ref();
        let current_level = latest
            .map(|m| m.value)
            .unwrap_or_else(|| history.last().map(|p| p.value).unwrap_or(f64::NAN));
        let current_timestamp_ms = latest
            .and_then(|m| parse_timestamp_ms(&m.timestamp))
            .or_else(|| history.last().map(|p| p.timestamp_ms))
            .unwrap_or(0);

        StationMetrics {
            station_id: station.uuid.clone(),
            station_name: station.longname.clone(),
            water: station.water.longname.clone(),
            km: station.km,
            latitude: station.latitude,
            longitude: station.longitude,
            current_level,
            current_timestamp_ms,
            change_1day: trend_value(&wl.trend, "p1d"),
            change_3day: trend_value(&wl.trend, "p3d"),
            change_7day: trend_value(&wl.trend, "p7d"),
            unit: if wl.unit.is_empty() {
                String::from("cm")
            } else {
                wl.unit.clone()
            },
            history,
            fetched_at_ms,
        }
    }
}
