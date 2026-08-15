//! Live API smoke test. Ignored by default; run with:
//!   cargo test --test live_smoke -- --ignored --nocapture
//! Requires WASSERSPIEGEL_API and WASSERSPIEGEL_TOKEN in the environment.

use wasserspiegel_core::client::Client;

fn api() -> (String, String) {
    (
        std::env::var("WASSERSPIEGEL_API").expect("WASSERSPIEGEL_API set"),
        std::env::var("WASSERSPIEGEL_TOKEN").expect("WASSERSPIEGEL_TOKEN set"),
    )
}

#[test]
#[ignore]
fn fetch_real_station() {
    let (base, token) = api();
    let client = Client::new(&base, &token).unwrap();

    let stations = client.list_stations().unwrap();
    println!("stations: {}", stations.len());
    assert!(stations.len() > 500);

    let mannheim = stations
        .iter()
        .find(|s| s.name.contains("MANNHEIM") && s.water == "RHEIN")
        .expect("MANNHEIM/RHEIN present");
    let metrics = client.fetch_station(&mannheim.id).unwrap();
    println!(
        "{} / {}: {} {} (trend 1d {})",
        metrics.station_name,
        metrics.water,
        metrics.current_level,
        metrics.unit,
        metrics.change_1day
    );
    assert_eq!(metrics.station_name, "MANNHEIM");
    assert!(metrics.current_level > 0.0);
    assert!(metrics.history.len() > 100);
}

#[test]
#[ignore]
fn auth_error_mapping() {
    let (base, _) = api();
    let client = Client::new(&base, "wrong-token").unwrap();
    let err = client.list_stations().unwrap_err();
    println!("error: {err}");
    assert!(matches!(err, wasserspiegel_core::error::CoreError::Auth(_)));
}

#[test]
#[ignore]
fn notfound_error_mapping() {
    let (base, token) = api();
    let client = Client::new(&base, &token).unwrap();
    let err = client.fetch_station("does-not-exist").unwrap_err();
    println!("error: {err}");
    assert!(
        matches!(&err, wasserspiegel_core::error::CoreError::NotFound(m) if m.contains("not found"))
    );
}
