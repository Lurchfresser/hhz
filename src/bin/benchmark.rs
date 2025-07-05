use csv::Writer;
use hhz::metrics::{SearchMetrics, SearchMetricsData};
use hhz::search::search_entry;
use std::time::{Instant, SystemTime};
use hhz::board::Board;

pub mod generate_attack_lookup;

static FEATURE_NAME: &str = "Own move gen";
static FEATURE_NUMBER: u32 = 6;

fn main() {
    let file_path = &format!(
        "{}/{}_{}.csv",
        "benchmarks".to_owned(),
        FEATURE_NUMBER,
        FEATURE_NAME
    );
    // if {FEATURE_NAME}.csv exists panic with a message
    if std::path::Path::new(file_path).exists() {
        panic!(
            "Metrics file {} already exists. Please remove it before running the benchmark.",
            file_path
        );
    }

    check_existing_benchmark_files();

    let mut metrics_data: Vec<SearchMetricsData> = Vec::new();

    // List of positions to benchmark
    let positions = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
                                                                    // "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",  // After 1.e4 e5 2.Nf3 Nc6
                                                                    // "r1bqk2r/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R w KQkq - 0 6",  // Italian Game
                                                                    // Add more positions as needed
    ];

    let max_depth = 7; // Depths to test

    println!("Chess Engine Benchmark");
    println!("=====================");

    SearchMetrics::initialize();

    let mut writer = Writer::from_path(file_path).expect("Failed to create CSV writer");

    for position in positions {
        println!("\nPosition: {}", position);

        for depth in 0..max_depth + 1 {
            println!(
                "\nSearching at depth {}, current time: {}",
                depth,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );

            // Reset metrics for this test
            SearchMetrics::new_measurement(FEATURE_NAME, depth);

            // Parse FEN to create game
            let board = Board::from_fen(position).unwrap();

            // Measure time with Rust's timing
            let start = Instant::now();

            //
            //
            // -------- full measurement --------
            let best_move = search_entry(&board, depth);
            // ----------------------------------------
            //
            //

            let elapsed = start.elapsed();

            println!("Best move found: {:?}", best_move);
            println!("Total time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);

            SearchMetrics::report();
            // Collect metrics
            unsafe {
                metrics_data.push(SearchMetrics::get_metrics());
            }
            writer
                .serialize(metrics_data.last().unwrap().clone())
                .unwrap();
            writer.flush().unwrap();
        }
    }
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
