use chessie::Game;
use csv::Writer;
use hhz::metrics::{SearchMetrics, SearchMetricsData};
use hhz::search::search_entry;
use std::time::Instant;

static FEATURE_NAME: &str = "Alpha-Beta-Pruning";

fn main() {
    // if {FEATURE_NAME}.csv exists panic with a message
    if std::path::Path::new(&format!("{}.csv", "benchmarks/".to_owned() + FEATURE_NAME)).exists() {
        panic!(
            "Metrics file {}.csv already exists. Please remove it before running the benchmark.",
            FEATURE_NAME
        );
    }

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

    let mut writer = Writer::from_path(format!("{}.csv", "benchmarks/".to_owned() + FEATURE_NAME))
        .expect("Failed to create CSV writer");

    for position in positions {
        println!("\nPosition: {}", position);

        for depth in 0..max_depth + 1 {
            println!("\nSearching at depth {}", depth);

            // Reset metrics for this test
            SearchMetrics::new_measurement(FEATURE_NAME, depth);

            // Parse FEN to create game
            let game = Game::from_fen(position).unwrap();

            // Measure time with Rust's timing
            let start = Instant::now();
            let best_move = search_entry(&game, depth);
            let elapsed = start.elapsed();

            println!("Best move found: {:?}", best_move);
            println!("Total time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);

            SearchMetrics::report();
            // Collect metrics
            unsafe {
                metrics_data.push(SearchMetrics::get_metrics());
            }
            writer
                .serialize(metrics_data.last().unwrap().clone()).unwrap();
            writer.flush().unwrap();
        }
    }
}
