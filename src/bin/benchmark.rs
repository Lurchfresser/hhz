use csv::Writer;
use hhz::board::Board;
use hhz::metrics::{SearchMetrics, SearchMetricsData, calculate_and_update_derived_metrics};
use hhz::search::search_entry;
use std::time::{Instant, SystemTime};

pub mod generate_attack_lookup;

static FEATURE_NAME: &str = "multiple_test_fens";
static FEATURE_NUMBER: u32 = 14;

fn main() {
    if cfg!(debug_assertions) {
        panic!("not in release mode");
    }
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

    let max_depth = 7u8;

    println!("Chess Engine Benchmark");
    println!("=====================");

    SearchMetrics::initialize();

    let mut writer = Writer::from_path(file_path)
        .expect(&("Failed to create CSV writer for path".to_owned() + file_path));

    for (position_name, fen) in positions {
        println!("\nPosition: {}", fen);

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

            // Reset metrics for this test
            SearchMetrics::new_measurement(FEATURE_NAME, depth, position_name, fen);

            // Parse FEN to create game
            let board = Board::from_fen(fen).unwrap();

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

            // Collect metrics
            unsafe {
                let metrics = SearchMetrics::get_metrics();
                metrics_data.push(metrics);
                writer.serialize(metrics).unwrap();
                writer.flush().unwrap();
            }
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
