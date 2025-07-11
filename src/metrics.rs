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
    depth: u8,

    // --- Split Counters ---
    normal_search_positions_generated: u64,
    q_search_positions_generated: u64,
    normal_search_entries: u64,
    q_search_entries: u64,
    stand_pat_cutoffs: u64,
    normal_search_cutoffs: u64,
    q_search_cutoffs: u64,

    // --- Move Ordering Quality Metrics (already split or specific) ---
    normal_search_best_move_first_count: u64,
    q_search_best_move_first_count: u64,
    normal_search_nodes_with_best_move: u64,
    q_search_nodes_with_best_move: u64,

    // --- Split Sums for Averages ---
    normal_search_sum_of_cutoff_indices: u64,
    q_search_sum_of_cutoff_indices: u64,

    // Note: These typically only apply to normal search, so we can leave them as is.
    #[serde(skip_serializing)]
    killer_move_cutoffs: u64,
    #[serde(skip_serializing)]
    history_heuristic_cutoffs: u64,

    // --- Split TT Metrics ---
    #[serde(skip_serializing)]
    normal_search_tt_probes: u64,
    #[serde(skip_serializing)]
    normal_search_tt_hits: u64,
    #[serde(skip_serializing)]
    normal_search_tt_cutoffs: u64,
    #[serde(skip_serializing)]
    q_search_tt_probes: u64,
    #[serde(skip_serializing)]
    q_search_tt_hits: u64,
    #[serde(skip_serializing)]
    q_search_tt_cutoffs: u64,

    // --- Split Timing Metrics ---
    #[serde(serialize_with = "serialize_duration_as_ms")]
    search_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    q_search_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    evaluation_time: Duration, // Global, as it's a leaf operation
    #[serde(serialize_with = "serialize_duration_as_ms")]
    normal_search_move_gen_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    q_search_move_gen_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    normal_search_move_ordering_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    q_search_move_ordering_time: Duration,
    #[serde(serialize_with = "serialize_duration_as_ms")]
    total_time: Duration,

    // --- Derived Metrics ---
    avg_normal_search_cutoff_index: f64,
    avg_q_search_cutoff_index: f64,
    normal_search_best_move_first_pct: f64,
    q_search_best_move_first_pct: f64,
    stand_pat_cutoff_pct: f64,
}

#[cfg(feature = "metrics")]
impl SearchMetricsData {
    pub fn new(feature_name: &'static str, depth: u8) -> Self {
        Self {
            feature_name,
            depth,

            // Initialize all new split counters
            normal_search_positions_generated: 0,
            q_search_positions_generated: 0,
            normal_search_entries: 0,
            q_search_entries: 0,
            stand_pat_cutoffs: 0,
            normal_search_cutoffs: 0,
            q_search_cutoffs: 0,

            normal_search_best_move_first_count: 0,
            q_search_best_move_first_count: 0,
            normal_search_nodes_with_best_move: 0,
            q_search_nodes_with_best_move: 0,

            normal_search_sum_of_cutoff_indices: 0,
            q_search_sum_of_cutoff_indices: 0,

            killer_move_cutoffs: 0,
            history_heuristic_cutoffs: 0,

            normal_search_tt_probes: 0,
            normal_search_tt_hits: 0,
            normal_search_tt_cutoffs: 0,
            q_search_tt_probes: 0,
            q_search_tt_hits: 0,
            q_search_tt_cutoffs: 0,

            // Initialize new split timing metrics
            search_time: Duration::default(),
            q_search_time: Duration::default(),
            evaluation_time: Duration::default(),
            normal_search_move_gen_time: Duration::default(),
            q_search_move_gen_time: Duration::default(),
            normal_search_move_ordering_time: Duration::default(),
            q_search_move_ordering_time: Duration::default(),
            total_time: Duration::default(),

            // Initialize derived metrics
            avg_normal_search_cutoff_index: 0.0,
            avg_q_search_cutoff_index: 0.0,
            normal_search_best_move_first_pct: 0.0,
            q_search_best_move_first_pct: 0.0,
            stand_pat_cutoff_pct: 0.0,
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
    if duration.as_millis() > 5 {
        let ms = duration.as_millis();
        serializer.serialize_u128(ms)
    } else {
        serializer.serialize_f64(duration.as_secs_f64() * 1000.0)
    }
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
    pub fn initialize() {}

    #[cfg(feature = "metrics")]
    pub fn new_measurement(feature_name: &'static str, depth: u8) {
        unsafe {
            if let Some(metrics) = &mut METRICS {
                *metrics = SearchMetricsData::new(feature_name, depth);
            }
            LAST_TIMING_CHANGE = None;
            CURRENT_TIMING_KIND = None;
            START_TIME = Some(Instant::now());
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn new_measurement(_feature_name: &'static str, _depth: u8) {}

    // --- General Counters ---

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_positions_generated(count: u64) {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_positions_generated += count;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_positions_generated(_count: u64) {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_positions_generated(count: u64) {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_positions_generated += count;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_positions_generated(_count: u64) {}

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_entries() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_entries += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_entries() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_entries() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_entries += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_entries() {}

    // --- Cutoff Counters ---

    #[cfg(feature = "metrics")]
    pub fn increment_stand_pat_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.stand_pat_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_stand_pat_cutoffs() {}

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_cutoffs() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_cutoffs() {}

    // --- Cutoff Index Sums ---

    #[cfg(feature = "metrics")]
    pub fn add_to_normal_search_sum_of_cutoff_indices(move_number: u64) {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_sum_of_cutoff_indices += move_number;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn add_to_normal_search_sum_of_cutoff_indices(_move_number: u64) {}

    #[cfg(feature = "metrics")]
    pub fn add_to_q_search_sum_of_cutoff_indices(move_number: u64) {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_sum_of_cutoff_indices += move_number;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn add_to_q_search_sum_of_cutoff_indices(_move_number: u64) {}

    // --- Move Ordering Quality ---

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_best_move_first_count() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_best_move_first_count += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_best_move_first_count() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_best_move_first_count() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_best_move_first_count += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_best_move_first_count() {}

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_nodes_with_best_move() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_nodes_with_best_move += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_nodes_with_best_move() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_nodes_with_best_move() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_nodes_with_best_move += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_nodes_with_best_move() {}

    // --- Killer/History Heuristics ---

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

    // --- Transposition Table ---

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_tt_probes() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_tt_probes += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_tt_probes() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_tt_probes() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_tt_probes += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_tt_probes() {}

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_tt_hits() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_tt_hits += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_tt_hits() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_tt_hits() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_tt_hits += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_tt_hits() {}

    #[cfg(feature = "metrics")]
    pub fn increment_normal_search_tt_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.normal_search_tt_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_normal_search_tt_cutoffs() {}

    #[cfg(feature = "metrics")]
    pub fn increment_q_search_tt_cutoffs() {
        unsafe {
            if let Some(m) = &mut METRICS {
                m.q_search_tt_cutoffs += 1;
            }
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn increment_q_search_tt_cutoffs() {}

    // --- Timing ---

    #[cfg(feature = "metrics")]
    pub fn change_timing_kind(new_kind: TimingKind) {
        unsafe {
            if let Some(start_time) = LAST_TIMING_CHANGE.take() {
                let elapsed = start_time.elapsed();
                if let Some(metrics) = &mut METRICS {
                    if let Some(current_kind) = &CURRENT_TIMING_KIND {
                        match current_kind {
                            TimingKind::Search => metrics.search_time += elapsed,
                            TimingKind::QSearch => metrics.q_search_time += elapsed,
                            TimingKind::Evaluation => metrics.evaluation_time += elapsed,
                            TimingKind::NormalMoveGen => {
                                metrics.normal_search_move_gen_time += elapsed
                            }
                            TimingKind::QMoveGen => metrics.q_search_move_gen_time += elapsed,
                            TimingKind::NormalMoveOrdering => {
                                metrics.normal_search_move_ordering_time += elapsed
                            }
                            TimingKind::QMoveOrdering => {
                                metrics.q_search_move_ordering_time += elapsed
                            }
                        }
                    }
                }
            }
            LAST_TIMING_CHANGE = Some(Instant::now());
            CURRENT_TIMING_KIND = Some(new_kind);
        }
    }

    #[cfg(not(feature = "metrics"))]
    pub fn change_timing_kind(_new_kind: TimingKind) {}

    #[cfg(feature = "metrics")]
    pub unsafe fn get_metrics() -> SearchMetricsData {
        if let Some(metrics) = &mut METRICS {
            // --- ADD THIS BLOCK ---
            // Ensure total_time is up-to-date before we clone the data.
            if let Some(start_time) = START_TIME {
                metrics.total_time = start_time.elapsed();
            }
            // --- END ADD ---

            // Now return the clone with the correct total_time
            metrics.clone()
        } else {
            panic!("Metrics not initialized");
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn get_metrics() -> SearchMetricsData {
        SearchMetricsData {}
    }
}

#[cfg(feature = "metrics")]
pub fn calculate_and_update_derived_metrics(metrics: &SearchMetricsData) -> SearchMetricsData {
    let mut new_metrics = metrics.clone();

    // --- 1. Average Cutoff Move Indices (Now Split) ---

    // Calculate for Normal Search
    if new_metrics.normal_search_cutoffs > 0 {
        new_metrics.avg_normal_search_cutoff_index = new_metrics.normal_search_sum_of_cutoff_indices
            as f64
            / new_metrics.normal_search_cutoffs as f64;
    } else {
        new_metrics.avg_normal_search_cutoff_index = 0.0;
    }

    // Calculate for Quiescence Search
    if new_metrics.q_search_cutoffs > 0 {
        new_metrics.avg_q_search_cutoff_index =
            new_metrics.q_search_sum_of_cutoff_indices as f64 / new_metrics.q_search_cutoffs as f64;
    } else {
        new_metrics.avg_q_search_cutoff_index = 0.0;
    }

    // --- 2. Stand-Pat Cutoff Percentage (Unchanged) ---
    // This logic is correct as it's inherently part of q-search.
    if new_metrics.q_search_entries > 0 {
        // Note: Your struct has `stand_pat_cutoff_pct` while your old function had a typo `pub_...`.
        // Using the correct name from the struct.
        new_metrics.stand_pat_cutoff_pct =
            new_metrics.stand_pat_cutoffs as f64 / new_metrics.q_search_entries as f64;
    } else {
        new_metrics.stand_pat_cutoff_pct = 0.0;
    }

    // --- 3. Best-Move-First Percentages (Using correct split denominators) ---

    // Calculate for Normal Search
    if new_metrics.normal_search_nodes_with_best_move > 0 {
        new_metrics.normal_search_best_move_first_pct =
            new_metrics.normal_search_best_move_first_count as f64
                / new_metrics.normal_search_nodes_with_best_move as f64;
    } else {
        new_metrics.normal_search_best_move_first_pct = 0.0;
    }

    // Calculate for Quiescence Search
    if new_metrics.q_search_nodes_with_best_move > 0 {
        new_metrics.q_search_best_move_first_pct = new_metrics.q_search_best_move_first_count
            as f64
            / new_metrics.q_search_nodes_with_best_move as f64;
    } else {
        new_metrics.q_search_best_move_first_pct = 0.0;
    }

    new_metrics
}
pub enum TimingKind {
    Search,
    QSearch,
    Evaluation,
    NormalMoveGen,      // Differentiated
    QMoveGen,           // Differentiated
    NormalMoveOrdering, // Differentiated
    QMoveOrdering,      // Differentiated
}
