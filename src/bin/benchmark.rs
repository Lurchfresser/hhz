use csv::{Writer, WriterBuilder};
use hhz::board::Board;
use hhz::metrics::{calculate_and_update_derived_metrics, SearchMetrics, SearchMetricsData};
use hhz::search::search_entry;
use hhz::tt_table::TT_Table;
use std::fs::OpenOptions;
use std::time::{Duration, Instant, SystemTime};

pub mod generate_attack_lookup;

static FEATURE_NAME: &str = "PV-then-cut-then-att-then-victim-then-all-node";
static FEATURE_NUMBER: u32 = 49;

fn main() {
    println!("board size: {}", std::mem::size_of::<Board>());

    if cfg!(debug_assertions) {
        panic!("not in release mode");
    }
    let file_path = &format!(
        "{}/{}_{}.csv",
        "benchmarks".to_owned(),
        FEATURE_NUMBER,
        FEATURE_NAME
    );
    if std::path::Path::new(file_path).exists() {
        panic!(
            "Metrics file {} already exists. Please remove it before running the benchmark.",
            file_path
        );
    }

    check_existing_benchmark_files();

    let mut metrics_data: Vec<SearchMetricsData> = Vec::new();

    let positions = [
        (
            "Starting position",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ),
        (
            "crowded middlegame",
            "rnbqkb1r/p4pp1/2p1pn1p/1p2P1B1/2pP4/2N2N2/PP3PPP/R2QKB1R w KQkq - 0 8",
        ),
        (
            "early endgame",
            "3k4/p3n1R1/4p3/2Pb2P1/2p5/2K5/1P3P2/8 w - - 4 39",
        ),
        (
            "pawns vs knight",
            "8/p1Pk2n1/4pKP1/8/1P6/8/5P2/8 b - - 2 49",
        ),
    ];

    let max_depth = 6u8;

    println!("Chess Engine Benchmark");
    println!("=====================");

    SearchMetrics::initialize();

    let mut writer = Writer::from_path(file_path)
        .expect(&("Failed to create CSV writer for path".to_owned() + file_path));

    for (position_name, fen) in positions {
        println!("\nPosition: {}", fen);

        let mut tt_table = TT_Table::new();
        for depth in 0..max_depth + 1 {
            println!(
                "\nSearching at depth {}, current time: {}, for {}",
                depth,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                position_name
            );

            SearchMetrics::new_measurement(FEATURE_NAME, depth, position_name, fen);

            let board = Board::from_fen(fen).unwrap();
            let start = Instant::now();

            let best_move = search_entry(&board, depth, &mut tt_table);

            let elapsed = start.elapsed();

            println!("Best move found: {:?}", best_move);
            println!("Total time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);

            unsafe {
                let metrics = SearchMetrics::get_metrics();
                metrics_data.push(metrics);
                writer.serialize(metrics).unwrap();
                writer.flush().unwrap();
            }
        }
    }

    calculate_and_write_summary(&metrics_data, 6);
}

fn calculate_and_write_summary(all_metrics: &[SearchMetricsData], target_depth: u8) {
    let depth_metrics: Vec<_> = all_metrics
        .iter()
        .filter(|m| m.depth == target_depth)
        .collect();

    if depth_metrics.is_empty() {
        println!(
            "No metrics found for depth {}. Skipping summary.",
            target_depth
        );
        return;
    }

    let count = depth_metrics.len();
    let mut avg_data = SearchMetricsData::new(FEATURE_NAME, target_depth, "AVERAGE", "AVERAGE");

    for m in &depth_metrics {
        // Sum raw counters
        avg_data.normal_search_positions_generated += m.normal_search_positions_generated;
        avg_data.q_search_positions_generated += m.q_search_positions_generated;
        avg_data.normal_search_entries += m.normal_search_entries;
        avg_data.q_search_entries += m.q_search_entries;
        avg_data.stand_pat_cutoffs += m.stand_pat_cutoffs;
        avg_data.normal_search_cutoffs += m.normal_search_cutoffs;
        avg_data.q_search_cutoffs += m.q_search_cutoffs;
        avg_data.normal_search_best_move_first_count += m.normal_search_best_move_first_count;
        avg_data.q_search_best_move_first_count += m.q_search_best_move_first_count;
        avg_data.normal_search_nodes_with_best_move += m.normal_search_nodes_with_best_move;
        avg_data.q_search_nodes_with_best_move += m.q_search_nodes_with_best_move;
        avg_data.normal_search_sum_of_cutoff_indices += m.normal_search_sum_of_cutoff_indices;
        avg_data.q_search_sum_of_cutoff_indices += m.q_search_sum_of_cutoff_indices;
        avg_data.normal_search_tt_probes += m.normal_search_tt_probes;
        avg_data.normal_search_tt_hits += m.normal_search_tt_hits;
        avg_data.normal_search_tt_cutoffs += m.normal_search_tt_cutoffs;
        avg_data.q_search_tt_probes += m.q_search_tt_probes;
        avg_data.q_search_tt_hits += m.q_search_tt_hits;
        avg_data.q_search_tt_cutoffs += m.q_search_tt_cutoffs;
        avg_data.pv_nodes_found_in_move_ordering += m.pv_nodes_found_in_move_ordering;
        avg_data.search_time += m.search_time;
        avg_data.q_search_time += m.q_search_time;
        avg_data.evaluation_time += m.evaluation_time;
        avg_data.normal_search_move_gen_time += m.normal_search_move_gen_time;
        avg_data.q_search_move_gen_time += m.q_search_move_gen_time;
        avg_data.normal_search_move_ordering_time += m.normal_search_move_ordering_time;
        avg_data.q_search_move_ordering_time += m.q_search_move_ordering_time;
        avg_data.total_time += m.total_time;

        // Sum derived metrics
        avg_data.avg_normal_search_cutoff_index += m.avg_normal_search_cutoff_index;
        avg_data.avg_q_search_cutoff_index += m.avg_q_search_cutoff_index;
        avg_data.normal_search_best_move_first_pct += m.normal_search_best_move_first_pct;
        avg_data.q_search_best_move_first_pct += m.q_search_best_move_first_pct;
        avg_data.stand_pat_cutoff_pct += m.stand_pat_cutoff_pct;
    }

    // Average raw counters
    avg_data.normal_search_positions_generated /= count as u64;
    avg_data.q_search_positions_generated /= count as u64;
    avg_data.normal_search_entries /= count as u64;
    avg_data.q_search_entries /= count as u64;
    avg_data.stand_pat_cutoffs /= count as u64;
    avg_data.normal_search_cutoffs /= count as u64;
    avg_data.q_search_cutoffs /= count as u64;
    avg_data.normal_search_best_move_first_count /= count as u64;
    avg_data.q_search_best_move_first_count /= count as u64;
    avg_data.normal_search_nodes_with_best_move /= count as u64;
    avg_data.q_search_nodes_with_best_move /= count as u64;
    avg_data.normal_search_sum_of_cutoff_indices /= count as u64;
    avg_data.q_search_sum_of_cutoff_indices /= count as u64;
    avg_data.normal_search_tt_probes /= count as u64;
    avg_data.normal_search_tt_hits /= count as u64;
    avg_data.normal_search_tt_cutoffs /= count as u64;
    avg_data.q_search_tt_probes /= count as u64;
    avg_data.q_search_tt_hits /= count as u64;
    avg_data.q_search_tt_cutoffs /= count as u64;
    avg_data.pv_nodes_found_in_move_ordering /= count as u64;
    avg_data.search_time /= count as u32;
    avg_data.q_search_time /= count as u32;
    avg_data.evaluation_time /= count as u32;
    avg_data.normal_search_move_gen_time /= count as u32;
    avg_data.q_search_move_gen_time /= count as u32;
    avg_data.normal_search_move_ordering_time /= count as u32;
    avg_data.q_search_move_ordering_time /= count as u32;
    avg_data.total_time /= count as u32;

    // Average derived metrics
    let count_f64 = count as f64;
    avg_data.avg_normal_search_cutoff_index /= count_f64;
    avg_data.avg_q_search_cutoff_index /= count_f64;
    avg_data.normal_search_best_move_first_pct /= count_f64;
    avg_data.q_search_best_move_first_pct /= count_f64;
    avg_data.stand_pat_cutoff_pct /= count_f64;

    write_summary_record(&avg_data).expect("Failed to write summary record");
}

fn write_summary_record(data: &SearchMetricsData) -> Result<(), Box<dyn std::error::Error>> {
    let summary_path = "benchmarks/summary_by_version.csv";
    let file_exists = std::path::Path::new(summary_path).exists();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(summary_path)?;

    let mut wtr = WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);

    wtr.serialize(data)?;
    wtr.flush()?;
    println!("\nAppended summary to {}", summary_path);
    Ok(())
}

fn check_existing_benchmark_files() {
    let benchmark_dir = "benchmarks/";
    let feature_number_prefix = format!("{}_", FEATURE_NUMBER);

    if let Ok(entries) = std::fs::read_dir(benchmark_dir) {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with(&feature_number_prefix) {
                    println!(
                        "Warning: Found existing benchmark file with same feature number: {}",
                        file_name
                    );
                    println!("Consider changing FEATURE_NUMBER or removing the existing file.");
                    panic!(
                        "Benchmark file with feature number {} already exists",
                        FEATURE_NUMBER
                    );
                }
            }
        }
    } else {
        println!("Note: Benchmark directory not found or not accessible");
    }
}