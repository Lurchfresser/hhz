use chessie::Game;
use std::time::Instant;
use hhz::metrics::SearchMetrics;
use hhz::search::search_entry;

fn main() {
    // List of positions to benchmark
    let positions = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",  // Starting position
        // "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",  // After 1.e4 e5 2.Nf3 Nc6
        // "r1bqk2r/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQK2R w KQkq - 0 6",  // Italian Game
        // Add more positions as needed
    ];

    let depths = [1, 2]; // Depths to test

    println!("Chess Engine Benchmark");
    println!("=====================");

    // Initialize metrics once
    SearchMetrics::init();

    for position in positions {
        println!("\nPosition: {}", position);

        for depth in depths {
            println!("\nSearching at depth {}", depth);

            // Reset metrics for this test
            SearchMetrics::reset();

            // Parse FEN to create game
            let game = Game::from_fen(position).unwrap();

            // Measure time with Rust's timing
            let start = Instant::now();
            let best_move = search_entry(&game, depth);
            let elapsed = start.elapsed();

            println!("Best move found: {:?}", best_move);
            println!("Total time: {:.3} ms", elapsed.as_secs_f64() * 1000.0);

            // Print metrics from our collector
            println!("{}", SearchMetrics::report());
        }
    }
}