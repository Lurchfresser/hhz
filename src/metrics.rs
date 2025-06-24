#![allow(static_mut_refs)]

#[cfg(feature = "metrics")]
use serde::Serialize;
#[cfg(feature = "metrics")]
use std::sync::Once;
#[cfg(feature = "metrics")]
use std::time::{Duration, Instant};

#[cfg(feature = "metrics")]
#[derive(Copy, Clone, Debug, Serialize)]
pub struct SearchMetricsData {
    feature_name: &'static str,
    depth: u32,
    positions_generated: u64,
    normal_search_entries: u64,
    q_search_entries: u64,
    num_alpha_beta_cutoffs: u64, // New field
    #[serde(serialize_with = "serialize_duration_as_ms")]
    search_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    q_search_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    evaluation_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    move_gen_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    move_ordering_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    total_time: Duration, // New field
}

#[cfg(feature = "metrics")]
impl SearchMetricsData {
    pub fn new(feature_name: &'static str, depth: u32) -> Self {
        Self {
            feature_name,
            depth,
            positions_generated: 0,
            normal_search_entries: 0,
            q_search_entries: 0,
            num_alpha_beta_cutoffs: 0,
            search_time: Duration::default(),
            evaluation_time: Duration::default(),
            move_gen_time: Duration::default(),
            move_ordering_time: Duration::default(),
            total_time: Duration::default(),
            q_search_time: Duration::default(),
        }
    }
}

#[cfg(not(feature = "metrics"))]
pub struct SearchMetricsData {}

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
static mut LAST_TIMING_CHANGE: Option<Instant> = None;
#[cfg(feature = "metrics")]
static mut CURRENT_TIMING_KIND: Option<TimingKind> = None;
#[cfg(feature = "metrics")]
static mut START_TIME: Option<Instant> = None;

pub static mut METRICS: Option<SearchMetricsData> = None;
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
    pub fn increment_alpha_beta_cutoffs() {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                metrics.num_alpha_beta_cutoffs += 1;
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn increment_alpha_beta_cutoffs() {}

    #[cfg(feature = "metrics")]
    pub fn new_measurement(feature_name: &'static str, depth: u32) {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                *metrics = SearchMetricsData::new(feature_name, depth);
            }
            // Reset timing variables
            LAST_TIMING_CHANGE = None;
            CURRENT_TIMING_KIND = None;
            // Set start time for total timing
            START_TIME = Some(Instant::now());
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
    pub fn change_timing_kind(new_kind: TimingKind) {
        unsafe {
            // Stop timing for the current kind if active
            if let Some(start_time) = LAST_TIMING_CHANGE.take() {
                let elapsed = start_time.elapsed();

                // Add elapsed time to the appropriate counter if we have a current timing kind
                if let Some(metrics) = &mut METRICS {
                    if let Some(current_kind) = &CURRENT_TIMING_KIND {
                        match current_kind {
                            TimingKind::Search => metrics.search_time += elapsed,
                            TimingKind::QSearch => metrics.q_search_time += elapsed,
                            TimingKind::Evaluation => metrics.evaluation_time += elapsed,
                            TimingKind::MoveGen => metrics.move_gen_time += elapsed,
                            TimingKind::MoveOrdering => metrics.move_ordering_time += elapsed,
                        }
                    }
                }
            }

            // Start timing for the new kind
            LAST_TIMING_CHANGE = Some(Instant::now());
            CURRENT_TIMING_KIND = Some(new_kind);
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn change_timing_kind(_new_kind: TimingKind) {
        // No-op if metrics feature is not enabled
    }

    #[cfg(feature = "metrics")]
    pub fn report() -> String {
        let mut report = String::new();
        unsafe {
            // Update total time from start to now
            if let (Some(start_time), Some(metrics)) = (START_TIME, &mut METRICS) {
                metrics.total_time = start_time.elapsed();
            }

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
                    "Alpha-beta cutoffs: {}\n",
                    metrics.num_alpha_beta_cutoffs
                ));
                report.push_str(&format!(
                    "Total time: {:.3} ms\n",
                    metrics.total_time.as_secs_f64() * 1000.0
                ));
                report.push_str(&format!(
                    "Search time: {:.3} ms\n",
                    metrics.search_time.as_secs_f64() * 1000.0
                ));
                report.push_str(&format!(
                    "Q-Search time: {:.3} ms\n",
                    metrics.q_search_time.as_secs_f64() * 1000.0
                ));
                report.push_str(&format!(
                    "Evaluation time: {:.3} ms\n",
                    metrics.evaluation_time.as_secs_f64() * 1000.0
                ));
                report.push_str(&format!(
                    "Move gen time: {:.3} ms\n",
                    metrics.move_gen_time.as_secs_f64() * 1000.0
                ));
                report.push_str(&format!(
                    "Move ordering time: {:.3} ms\n",
                    metrics.move_ordering_time.as_secs_f64() * 1000.0
                ));

                if metrics.normal_search_entries > 0 {
                    let positions_per_second =
                        metrics.positions_generated as f64 / metrics.total_time.as_secs_f64();
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
        SearchMetricsData {}
    }

    #[cfg(not(feature = "metrics"))]
    pub fn report() -> String {
        String::new()
    }
}

pub enum TimingKind {
    Search,
    QSearch,
    Evaluation,
    MoveGen,
    MoveOrdering,
}
