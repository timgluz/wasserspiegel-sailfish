//! Serde models mirroring the JSON shapes actually served by the
//! wasserspiegel API (raw PegelOnline flavoured payloads).
//!
//! Verified against live responses on 2026-08-15; see tests/fixtures.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaterRef {
    #[serde(default)]
    pub longname: String,
    #[serde(default)]
    pub shortname: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawStation {
    pub uuid: String,
    #[serde(default)]
    pub longname: String,
    #[serde(default)]
    pub shortname: String,
    #[serde(default)]
    pub km: f64,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    #[serde(default)]
    pub water: WaterRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationListResponse {
    #[serde(default)]
    pub stations: Vec<RawStation>,
}

/// Single measurement point as served in water level collections.
/// `unit` is usually empty on series entries; the collection carries it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawMeasurement {
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub unit: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawTrend {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p1d: Option<RawMeasurement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p3d: Option<RawMeasurement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p7d: Option<RawMeasurement>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawWaterLevel {
    #[serde(default)]
    pub station_id: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
    #[serde(default)]
    pub measurements: Vec<RawMeasurement>,
    #[serde(default)]
    pub latest: Option<RawMeasurement>,
    #[serde(default)]
    pub trend: RawTrend,
    #[serde(default)]
    pub unit: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StationDetailResponse {
    #[serde(default)]
    pub station: RawStation,
    #[serde(default)]
    pub water_level: RawWaterLevel,
}

/// Error envelope: `{"error": "..."}` - served on all failures
/// (including "resource not found" which arrives with HTTP 500).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    #[serde(default)]
    pub error: String,
}
