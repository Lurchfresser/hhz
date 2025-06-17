#![allow(static_mut_refs)]

// src/metrics.rs
#[cfg(feature = "metrics")]
use std::sync::Once;
#[cfg(feature = "metrics")]
use std::time::{Duration, Instant};

#[cfg(feature = "metrics")]
pub struct SearchMetricsData {
    positions_generated: u64,
    normal_search_entries: u64,
    q_search_entries: u64,
    search_time: Duration,
    search_start_time: Option<Instant>,
    evaluation_time: Duration,
    evaluation_start_time: Option<Instant>,
}

#[cfg(feature = "metrics")]
static mut METRICS: Option<SearchMetricsData> = None;
#[cfg(feature = "metrics")]
static INIT: Once = Once::new();

pub struct SearchMetrics;

impl SearchMetrics {
    #[cfg(feature = "metrics")]
    pub fn init() {
        INIT.call_once(|| unsafe {
            METRICS = Some(SearchMetricsData {
                positions_generated: 0,
                normal_search_entries: 0,
                q_search_entries: 0,
                search_time: Duration::default(),
                search_start_time: None,
                evaluation_time: Duration::default(),
                evaluation_start_time: None,
            });
        });
    }

    #[cfg(not(feature = "metrics"))]
    pub fn init() {}

    #[cfg(feature = "metrics")]
    pub fn reset() {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                *metrics = SearchMetricsData {
                    positions_generated: 0,
                    normal_search_entries: 0,
                    q_search_entries: 0,
                    search_time: Duration::default(),
                    search_start_time: None,
                    evaluation_time: Duration::default(),
                    evaluation_start_time: None,
                };
            }
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
            if let Some(metrics) = &mut METRICS {
                metrics.search_start_time = Some(Instant::now());
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn start_timing() {}

    #[cfg(feature = "metrics")]
    pub fn stop_timing() {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                if let Some(start_time) = metrics.search_start_time.take() {
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
            if let Some(metrics) = &mut METRICS {
                metrics.evaluation_start_time = Some(Instant::now());
            }
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn start_evaluation_timing() {}

    #[cfg(feature = "metrics")]
    pub fn stop_evaluation_timing() {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                if let Some(start_time) = metrics.evaluation_start_time.take() {
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
                report.push_str(&format!("Positions generated: {}\n", metrics.positions_generated));
                report.push_str(&format!("Normal search entries: {}\n", metrics.normal_search_entries));
                report.push_str(&format!("Quiescence search entries: {}\n", metrics.q_search_entries));
                report.push_str(&format!("Search time: {:.3} ms\n", metrics.search_time.as_secs_f64() * 1000.0));
                report.push_str(&format!("Evaluation time: {:.3} ms\n", metrics.evaluation_time.as_secs_f64() * 1000.0));

                if metrics.normal_search_entries > 0 {
                    let positions_per_second = metrics.positions_generated as f64 / metrics.search_time.as_secs_f64();
                    report.push_str(&format!("Positions per second: {:.0}\n", positions_per_second));
                }
            }
        }
        report
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
        Self { kind: TimingKind::Search }
    }

    pub fn new_evaluation() -> Self {
        SearchMetrics::start_evaluation_timing();
        Self { kind: TimingKind::Evaluation }
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
    pub fn new_search() -> Self { Self }
    pub fn new_evaluation() -> Self { Self }
}