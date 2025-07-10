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
    num_alpha_beta_cutoffs: u64,

    // --- Move Ordering Quality Metrics ---
    /// Number of nodes where the first move searched was the best one found.
    best_move_first_count: u64,
    /// Total number of nodes where a best move was established (i.e., alpha was raised).
    nodes_with_best_move_count: u64,
    /// Sum of the 1-based indices of all moves that caused a beta cutoff. Used to calculate the average.
    sum_of_cutoff_move_indices: u64,
    /// Number of beta cutoffs caused by a "killer move".
    killer_move_cutoffs: u64,
    /// Number of beta cutoffs caused by a move ordered high by the history heuristic.
    history_heuristic_cutoffs: u64,

    // --- Transposition Table (TT) Metrics ---
    /// Total number of times the TT was probed for an entry.
    tt_probes: u64,
    /// Number of times a probe found a valid entry in the TT.
    tt_hits: u64,
    /// Number of times a TT hit resulted in an immediate cutoff, avoiding a search of the node.
    tt_cutoffs: u64,

    // --- Timing Metrics ---
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
    total_time: Duration,

    pub avg_cutoff_move_index: f64,
    pub best_move_first_pct: f64,
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

            // Initialize new metrics
            best_move_first_count: 0,
            nodes_with_best_move_count: 0,
            sum_of_cutoff_move_indices: 0,
            killer_move_cutoffs: 0,
            history_heuristic_cutoffs: 0,
            tt_probes: 0,
            tt_hits: 0,
            tt_cutoffs: 0,

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

    // Transposition Table
    #[cfg(feature = "metrics")]
    pub fn increment_tt_probes() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.tt_probes += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_tt_probes() {}

    #[cfg(feature = "metrics")]
    pub fn increment_tt_hits() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.tt_hits += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_tt_hits() {}

    #[cfg(feature = "metrics")]
    pub fn increment_tt_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.tt_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_tt_cutoffs() {}

    // Move Ordering Quality
    #[cfg(feature = "metrics")]
    pub fn increment_best_move_first_count() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.best_move_first_count += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_best_move_first_count() {}

    #[cfg(feature = "metrics")]
    pub fn increment_nodes_with_best_move_count() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.nodes_with_best_move_count += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_nodes_with_best_move_count() {}

    #[cfg(feature = "metrics")]
    pub fn add_to_cutoff_move_index(move_number: u64) {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.sum_of_cutoff_move_indices += move_number;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn add_to_cutoff_move_index(_move_number: u64) {}

    #[cfg(feature = "metrics")]
    pub fn increment_killer_move_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.killer_move_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_killer_move_cutoffs() {}

    #[cfg(feature = "metrics")]
    pub fn increment_history_heuristic_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.history_heuristic_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_history_heuristic_cutoffs() {}

    #[cfg(feature = "metrics")]
    pub fn report() -> String {
        let mut report = String::new();
        unsafe {
            if let (Some(start_time), Some(metrics)) = (START_TIME, &mut METRICS) {
                metrics.total_time = start_time.elapsed();
            }

            if let Some(metrics) = &METRICS {
                report.push_str("=== Search Metrics ===\n");
                // ... (existing general metrics reporting) ...
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

                // --- TT STATISTICS REPORTING ---
                report.push_str("\n--- Transposition Table ---\n");
                report.push_str(&format!("TT Probes: {}\n", metrics.tt_probes));
                report.push_str(&format!("TT Hits: {}\n", metrics.tt_hits));
                report.push_str(&format!("TT Cutoffs: {}\n", metrics.tt_cutoffs));
                if metrics.tt_probes > 0 {
                    let hit_rate = (metrics.tt_hits as f64 / metrics.tt_probes as f64) * 100.0;
                    report.push_str(&format!("TT Hit Rate: {:.2}%\n", hit_rate));
                }
                if metrics.tt_hits > 0 {
                    let cutoff_rate = (metrics.tt_cutoffs as f64 / metrics.tt_hits as f64) * 100.0;
                    report.push_str(&format!("TT Cutoff Rate (of hits): {:.2}%\n", cutoff_rate));
                }

                // --- MOVE ORDERING QUALITY REPORTING ---
                report.push_str("\n--- Move Ordering Quality ---\n");
                if metrics.nodes_with_best_move_count > 0 {
                    let best_first_pct = (metrics.best_move_first_count as f64
                        / metrics.nodes_with_best_move_count as f64)
                        * 100.0;
                    report.push_str(&format!(
                        "Best-Move-First Percentage: {:.2}%\n",
                        best_first_pct
                    ));
                }
                if metrics.num_alpha_beta_cutoffs > 0 {
                    let avg_cutoff_idx = metrics.sum_of_cutoff_move_indices as f64
                        / metrics.num_alpha_beta_cutoffs as f64;
                    report.push_str(&format!(
                        "Average Cutoff Move Index: {:.2}\n",
                        avg_cutoff_idx
                    ));

                    let killer_pct = (metrics.killer_move_cutoffs as f64
                        / metrics.num_alpha_beta_cutoffs as f64)
                        * 100.0;
                    report.push_str(&format!(
                        "Cutoffs from Killer Moves: {} ({:.2}%)\n",
                        metrics.killer_move_cutoffs, killer_pct
                    ));

                    let history_pct = (metrics.history_heuristic_cutoffs as f64
                        / metrics.num_alpha_beta_cutoffs as f64)
                        * 100.0;
                    report.push_str(&format!(
                        "Cutoffs from History Moves: {} ({:.2}%)\n",
                        metrics.history_heuristic_cutoffs, history_pct
                    ));
                }

                // --- TIMING REPORTING ---
                report.push_str("\n--- Timing ---\n");
                // ... (rest of your timing report) ...
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
