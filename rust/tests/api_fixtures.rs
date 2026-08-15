//! Integration tests against JSON fixtures captured from the live
//! wasserspiegel API on 2026-08-15.

use std::fs;
use std::path::PathBuf;

use wasserspiegel_core::domain::{StationMetrics, StationSummary};
use wasserspiegel_core::models::{ApiError, StationDetailResponse, StationListResponse};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[test]
fn parses_station_list() {
    let list: StationListResponse = serde_json::from_str(&fixture("stations.json")).unwrap();
    assert!(!list.stations.is_empty());

    let mannheim: Vec<_> = list
        .stations
        .iter()
        .filter(|s| s.longname.contains("MANNHEIM"))
        .collect();
    assert!(!mannheim.is_empty());
    let rhein = mannheim
        .iter()
        .find(|s| s.water.longname == "RHEIN")
        .expect("MANNHEIM / RHEIN in fixture");
    assert!(!rhein.uuid.is_empty());
    assert!(rhein.km > 0.0);
    assert!(rhein.latitude != 0.0);

    let summaries: Vec<StationSummary> = list.stations.iter().map(Into::into).collect();
    assert_eq!(summaries.len(), list.stations.len());
    assert!(summaries.iter().all(|s| !s.id.is_empty()));
}

#[test]
fn parses_station_detail_into_metrics() {
    let detail: StationDetailResponse =
        serde_json::from_str(&fixture("station_detail.json")).unwrap();
    let metrics = StationMetrics::from_detail(&detail, 1_755_000_600_000);

    assert_eq!(metrics.station_name, "MANNHEIM");
    assert_eq!(metrics.water, "RHEIN");
    assert_eq!(metrics.unit, "cm");
    assert_eq!(metrics.current_level, 71.0);
    assert!(metrics.current_timestamp_ms > 0);

    assert_eq!(metrics.change_1day, -3.0);
    assert!((metrics.change_3day - 3.5416666666666714).abs() < 1e-9);
    assert!((metrics.change_7day - 16.020833333337535).abs() < 1e-6);

    // 120 fixture points (every 8th of 960), all with valid timestamps
    assert_eq!(metrics.history.len(), 120);
    assert!(metrics.history.iter().all(|p| p.timestamp_ms > 0));
    // history is chronological
    assert!(metrics
        .history
        .windows(2)
        .all(|w| w[0].timestamp_ms <= w[1].timestamp_ms));
}

#[test]
fn tolerates_missing_trend_fields() {
    let json = r#"{
        "station": {"uuid": "u1", "longname": "X", "shortname": "X", "km": 1.0,
                     "latitude": 1.0, "longitude": 2.0,
                     "water": {"longname": "Y", "shortname": "Y"}},
        "water_level": {
            "station_id": "u1",
            "measurements": [{"timestamp": "2026-08-15T10:00:00+02:00", "value": 42.5}],
            "trend": {}
        }
    }"#;
    let detail: StationDetailResponse = serde_json::from_str(json).unwrap();
    let metrics = StationMetrics::from_detail(&detail, 0);
    assert_eq!(metrics.current_level, 42.5); // falls back to last history point
    assert!(metrics.change_1day.is_nan());
    assert_eq!(metrics.unit, "cm"); // defaults when empty
    assert_eq!(metrics.history.len(), 1);
}

#[test]
fn parses_error_envelope() {
    let err: ApiError = serde_json::from_str(&fixture("error_notfound.json")).unwrap();
    assert!(err.error.contains("resource not found"));
}

#[test]
fn client_rejects_bad_base_urls() {
    use wasserspiegel_core::client::Client;
    use wasserspiegel_core::error::CoreError;
    assert!(matches!(
        Client::new("not-a-url", "tok"),
        Err(CoreError::Config(_))
    ));
    assert!(Client::new(" https://x.example/ ", "tok").is_ok());
}
