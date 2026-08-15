//! HTTPS client for the wasserspiegel API (ureq + rustls, blocking calls -
//! the C++ layer is expected to invoke them off the UI thread).

use std::io::Read;
use std::time::Duration;

use crate::domain::{StationMetrics, StationSummary};
use crate::error::{CoreError, CoreResult};
use crate::models::{ApiError, StationDetailResponse, StationListResponse};

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
pub const GLOBAL_TIMEOUT: Duration = Duration::from_secs(30);

/// Hard cap on response body size - guards against runaway allocations
/// on the memory-constrained mobile target.
const MAX_BODY_BYTES: u64 = 16 * 1024 * 1024;

pub struct Client {
    agent: ureq::Agent,
    api_base: String,
    token: String,
}

impl Client {
    pub fn new(api_base: impl Into<String>, token: impl Into<String>) -> CoreResult<Self> {
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(GLOBAL_TIMEOUT))
            // keep error responses so we can read their JSON envelope
            .http_status_as_error(false)
            // the API base is fixed; unexpected redirects (e.g. a
            // compromised or misconfigured server) must not receive our
            // bearer token
            .max_redirects(0)
            .build();
        Ok(Client {
            agent: config.new_agent(),
            api_base: normalize_base(api_base.into())?,
            token: token.into(),
        })
    }

    pub fn set_config(
        &mut self,
        api_base: impl Into<String>,
        token: impl Into<String>,
    ) -> CoreResult<()> {
        self.api_base = normalize_base(api_base.into())?;
        self.token = token.into();
        Ok(())
    }

    /// GET /stations - full station list (server currently ignores
    /// offset/limit and returns everything at once).
    pub fn list_stations(&self) -> CoreResult<Vec<StationSummary>> {
        let url = format!("{}/stations", self.api_base);
        let resp: StationListResponse = self.get_json(&url)?;
        Ok(resp.stations.iter().map(|s| s.into()).collect())
    }

    /// GET /stations/{id} - station detail with water level, trends and
    /// ~P10D measurement history. The id crosses the FFI boundary from
    /// QML/QSettings, so it is validated before it reaches the URL.
    pub fn fetch_station(&self, station_id: &str) -> CoreResult<StationMetrics> {
        let station_id = validate_station_id(station_id)?;
        let url = format!("{}/stations/{}", self.api_base, station_id);
        let detail: StationDetailResponse = self.get_json(&url)?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        Ok(StationMetrics::from_detail(&detail, now_ms))
    }

    fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> CoreResult<T> {
        let resp = self
            .agent
            .get(url)
            .header("Authorization", &format!("Bearer {}", self.token))
            .call()
            .map_err(|e| CoreError::Network(format!("{e} for GET {url}")))?;

        let status = resp.status().as_u16();
        let mut body = String::new();
        resp.into_body()
            .into_reader()
            .take(MAX_BODY_BYTES)
            .read_to_string(&mut body)
            .map_err(|e| CoreError::Network(format!("{e} for GET {url}")))?;

        if status >= 400 {
            return Err(Self::map_error_body(status, &body));
        }

        serde_json::from_str(&body).map_err(|e| CoreError::Parse(format!("{e} for GET {url}")))
    }

    fn map_error_body(status: u16, body: &str) -> CoreError {
        let api_msg = serde_json::from_str::<ApiError>(body)
            .ok()
            .filter(|e| !e.error.is_empty())
            .map(|e| e.error);
        match api_msg {
            Some(msg) if status == 401 || status == 403 => CoreError::Auth(msg),
            Some(msg) if msg.to_lowercase().contains("not found") => CoreError::NotFound(msg),
            Some(msg) => CoreError::Server(status, msg),
            None if status == 401 || status == 403 => CoreError::Auth(format!("HTTP {status}")),
            None => CoreError::Server(status, format!("HTTP {status}")),
        }
    }
}

/// Station ids are PegelOnline UUIDs; anything that could alter the URL
/// path (separators, query or fragment characters) is rejected outright.
fn validate_station_id(id: &str) -> CoreResult<&str> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid {
        Ok(id)
    } else {
        Err(CoreError::Config(format!("invalid station id: {id:?}")))
    }
}

fn normalize_base(base: String) -> CoreResult<String> {
    let trimmed = base.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        return Err(CoreError::Config("API base URL is empty".into()));
    }
    // https only: the bearer token must never travel in cleartext
    if !trimmed.starts_with("https://") {
        return Err(CoreError::Config(format!(
            "API base URL must start with https://: {trimmed}"
        )));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_normalization() {
        assert_eq!(
            normalize_base("https://x.example/".into()).unwrap(),
            "https://x.example"
        );
        assert!(normalize_base("".into()).is_err());
        assert!(normalize_base("x.example".into()).is_err());
        // plain http is rejected - the token must not travel in cleartext
        assert!(normalize_base("http://x.example".into()).is_err());
    }

    #[test]
    fn station_id_validation() {
        assert_eq!(
            validate_station_id("57090802-c51a-4d09-8340-b4453cd0e1f5").unwrap(),
            "57090802-c51a-4d09-8340-b4453cd0e1f5"
        );
        assert!(validate_station_id("").is_err());
        assert!(validate_station_id("../admin").is_err());
        assert!(validate_station_id("a?b").is_err());
        assert!(validate_station_id("a#b").is_err());
        assert!(validate_station_id("a b").is_err());
        let long = "a".repeat(65);
        assert!(validate_station_id(&long).is_err());
    }
}
