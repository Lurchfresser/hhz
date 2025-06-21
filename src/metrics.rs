#![allow(static_mut_refs)]

#[cfg(feature = "metrics")]
use serde::Serialize;
#[cfg(feature = "metrics")]
use std::sync::Once;
#[cfg(feature = "metrics")]
use std::time::{Duration, Instant};


//TODO: total time
#[cfg(feature = "metrics")]
#[derive(Copy, Clone, Debug, Serialize)]
pub struct SearchMetricsData {
    //TODO: Position based metrics
    feature_name: &'static str,
    depth: u32,
    positions_generated: u64,
    normal_search_entries: u64,
    q_search_entries: u64,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    search_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    evaluation_time: Duration,
}

#[cfg(not(feature = "metrics"))]
struct SearchMetricsData{}

#[cfg(feature = "metrics")]
fn serialize_duration_as_ms<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let ms = duration.as_secs_f64() * 1000.0;
    serializer.serialize_f64(ms)
}

// Add these static variables to track timing state separately
#[cfg(feature = "metrics")]
static mut SEARCH_START_TIME: Option<Instant> = None;
#[cfg(feature = "metrics")]
static mut EVALUATION_START_TIME: Option<Instant> = None;

#[cfg(feature = "metrics")]
impl SearchMetricsData {
    pub fn new(feature_name: &'static str, depth: u32) -> Self {
        Self {
            feature_name,
            depth,
            positions_generated: 0,
            normal_search_entries: 0,
            q_search_entries: 0,
            search_time: Duration::default(),
            evaluation_time: Duration::default(),
        }
    }
}

static mut METRICS: Option<SearchMetricsData> = None;
#[cfg(feature = "metrics")]
static INIT: Once = Once::new();

pub struct SearchMetrics;

impl SearchMetrics {
    #[cfg(feature = "metrics")]
    pub fn initialize() {
        INIT.call_once(|| unsafe {
            METRICS = Some(SearchMetricsData::new("Default Feature", 0));
        });
    }

    #[cfg(not(feature = "metrics"))]
    pub fn initialize() {
        // No-op if metrics feature is not enabled
    }

    #[cfg(feature = "metrics")]
    pub fn new_measurement(feature_name: &'static str, depth: u32) {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                *metrics = SearchMetricsData {
                    feature_name,
                    depth,
                    positions_generated: 0,
                    normal_search_entries: 0,
                    q_search_entries: 0,
                    search_time: Duration::default(),
                    evaluation_time: Duration::default(),
                };
            }
            // Reset timing variables too
            SEARCH_START_TIME = None;
            EVALUATION_START_TIME = None;
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn reset() {}

    #[cfg(feature = "metrics")]
    pub fn increment_positions_generated(count: u64) {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                metrics.positions_generated += count;
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn increment_positions_generated(_count: u64) {}

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_entries() {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                metrics.normal_search_entries += 1;
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_entries() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_entries() {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                metrics.q_search_entries += 1;
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_entries() {}

    #[cfg(feature = "metrics")]
    pub fn start_timing() {
        unsafe {
            SEARCH_START_TIME = Some(Instant::now());
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn start_timing() {}

    #[cfg(feature = "metrics")]
    pub fn stop_timing() {
        unsafe {
            if let Some(start_time) = SEARCH_START_TIME.take() {
                if let Some(metrics) = &mut METRICS {
                    metrics.search_time += start_time.elapsed();
                }
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn stop_timing() {}

    #[cfg(feature = "metrics")]
    pub fn start_evaluation_timing() {
        unsafe {
            EVALUATION_START_TIME = Some(Instant::now());
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn start_evaluation_timing() {}

    #[cfg(feature = "metrics")]
    pub fn stop_evaluation_timing() {
        unsafe {
            if let Some(start_time) = EVALUATION_START_TIME.take() {
                if let Some(metrics) = &mut METRICS {
                    metrics.evaluation_time += start_time.elapsed();
                }
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn stop_evaluation_timing() {}

    #[cfg(feature = "metrics")]
    pub fn report() -> String {
        let mut report = String::new();
        unsafe {
            if let Some(metrics) = &METRICS {
                report.push_str("=== Search Metrics ===\n");
                report.push_str(&format!(
                    "Positions generated: {}\n",
                    metrics.positions_generated
                ));
                report.push_str(&format!(
                    "Normal search entries: {}\n",
                    metrics.normal_search_entries
                ));
                report.push_str(&format!(
                    "Quiescence search entries: {}\n",
                    metrics.q_search_entries
                ));
                report.push_str(&format!(
                    "Search time: {:.3} ms\n",
                    metrics.search_time.as_secs_f64() * 1000.0
                ));
                report.push_str(&format!(
                    "Evaluation time: {:.3} ms\n",
                    metrics.evaluation_time.as_secs_f64() * 1000.0
                ));

                if metrics.normal_search_entries > 0 {
                    let positions_per_second =
                        metrics.positions_generated as f64 / metrics.search_time.as_secs_f64();
                    report.push_str(&format!(
                        "Positions per second: {:.0}\n",
                        positions_per_second
                    ));
                }
            }
        }
        report
    }
    #[cfg(feature = "metrics")]
    pub unsafe fn get_metrics() -> SearchMetricsData {
        unsafe { METRICS.expect("Metrics not initialized").clone() }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn get_metrics() -> SearchMetricsData {
        SearchMetricsData{}
    }

    #[cfg(not(feature = "metrics"))]
    pub fn report() -> String {
        String::new()
    }
}

#[cfg(feature = "metrics")]
pub struct TimingGuard {
    kind: TimingKind,
}

#[cfg(feature = "metrics")]
pub enum TimingKind {
    Search,
    Evaluation,
}

#[cfg(feature = "metrics")]
impl TimingGuard {
    pub fn new_search() -> Self {
        SearchMetrics::start_timing();
        Self {
            kind: TimingKind::Search,
        }
    }

    pub fn new_evaluation() -> Self {
        SearchMetrics::start_evaluation_timing();
        Self {
            kind: TimingKind::Evaluation,
        }
    }
}

#[cfg(feature = "metrics")]
impl Drop for TimingGuard {
    fn drop(&mut self) {
        match self.kind {
            TimingKind::Search => SearchMetrics::stop_timing(),
            TimingKind::Evaluation => SearchMetrics::stop_evaluation_timing(),
        }
    }
}

#[cfg(not(feature = "metrics"))]
pub struct TimingGuard;

#[cfg(not(feature = "metrics"))]
impl TimingGuard {
    pub fn new_search() -> Self {
        Self
    }
    pub fn new_evaluation() -> Self {
        Self
    }
}
