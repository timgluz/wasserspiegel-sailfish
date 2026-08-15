//! Time-series slicing and downsampling for the QML trend graph.

use crate::domain::MeasurementPoint;

/// Return the points within the last `hours_back` before the newest
/// sample (not before "now" - so stale caches still render correctly),
/// downsampled to at most `max_points` via stride sampling that always
/// keeps the first and last sample.
pub fn slice_series(
    history: &[MeasurementPoint],
    hours_back: i64,
    max_points: usize,
) -> Vec<MeasurementPoint> {
    if history.is_empty() || max_points == 0 {
        return Vec::new();
    }
    if max_points == 1 {
        return vec![history[history.len() - 1]];
    }

    let hours_back = hours_back.max(1);
    let cutoff = history[history.len() - 1].timestamp_ms - hours_back * 3_600_000;
    let start = history.partition_point(|p| p.timestamp_ms < cutoff);
    let window = &history[start..];
    if window.len() <= max_points {
        return window.to_vec();
    }

    let step = (window.len() - 1) as f64 / (max_points - 1) as f64;
    let mut out: Vec<MeasurementPoint> = Vec::with_capacity(max_points);
    for i in 0..max_points {
        let idx = ((i as f64) * step).round() as usize;
        let idx = idx.min(window.len() - 1);
        let point = window[idx];
        if out.last().map(|p| p.timestamp_ms) != Some(point.timestamp_ms) {
            out.push(point);
        }
    }
    out
}

/// Min/max of a series, with a guard against degenerate flat lines.
pub fn value_range(series: &[MeasurementPoint]) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for p in series {
        if p.value.is_finite() {
            min = min.min(p.value);
            max = max.max(p.value);
        }
    }
    if !min.is_finite() || !max.is_finite() {
        return (0.0, 1.0);
    }
    if (max - min).abs() < f64::EPSILON {
        return (min - 1.0, max + 1.0);
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(n: usize, start_ms: i64, step_ms: i64) -> Vec<MeasurementPoint> {
        (0..n)
            .map(|i| MeasurementPoint {
                timestamp_ms: start_ms + (i as i64) * step_ms,
                value: i as f64,
            })
            .collect()
    }

    #[test]
    fn slicing_by_hours() {
        // hourly points, 48 of them; window is [latest-24h, latest] -> 25 points
        let s = series(48, 0, 3_600_000);
        let out = slice_series(&s, 24, 1000);
        assert_eq!(out.len(), 25);
        assert_eq!(out[0].timestamp_ms, 23 * 3_600_000);
        assert_eq!(out.last().unwrap().timestamp_ms, 47 * 3_600_000);
    }

    #[test]
    fn downsampling_keeps_endpoints() {
        let s = series(960, 0, 900_000); // 15-min interval, 10 days
        let out = slice_series(&s, 24 * 10, 200);
        assert!(out.len() <= 200);
        assert_eq!(out[0].timestamp_ms, 0);
        assert_eq!(out.last().unwrap().timestamp_ms, 959 * 900_000);
        // strictly increasing timestamps
        assert!(out
            .windows(2)
            .all(|w| w[0].timestamp_ms < w[1].timestamp_ms));
    }

    #[test]
    fn no_resample_when_small() {
        let s = series(10, 0, 1);
        let out = slice_series(&s, 24, 200);
        assert_eq!(out.len(), 10);
    }

    #[test]
    fn degenerate_max_points() {
        let s = series(10, 0, 1);
        assert!(slice_series(&s, 24, 0).is_empty());
        let one = slice_series(&s, 24, 1);
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].timestamp_ms, s[9].timestamp_ms);
    }

    #[test]
    fn non_positive_hours_clamped() {
        let s = series(48, 0, 3_600_000);
        // 0 or negative hours must not swallow the whole series
        let out = slice_series(&s, 0, 1000);
        assert_eq!(out.len(), 2); // [latest-1h, latest]
        assert!(!slice_series(&s, -5, 1000).is_empty());
    }

    #[test]
    fn flat_range_guard() {
        let s = vec![
            MeasurementPoint {
                timestamp_ms: 0,
                value: 5.0,
            },
            MeasurementPoint {
                timestamp_ms: 1,
                value: 5.0,
            },
        ];
        let (min, max) = value_range(&s);
        assert!(min < 5.0 && max > 5.0);
    }
}
