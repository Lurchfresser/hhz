use dotenv::dotenv;
use futures_util::StreamExt;
use licheszter::models::game::{Color as LichessColor, GameStatus};
use licheszter::{client::Licheszter, models::board::BoardState};
use hhz::board::Board;
use hhz::search::search_entry;

#[tokio::main]
async fn main() {
    let depth = 4; // Set the search depth for the bot
    dotenv().ok(); // Load environment variables from .env file
    //let lichess_id = dotenv::var("LICHESS_ID").expect("LICHESS_ID not set");
    // Create a new Licheszter with your account token
    let client = Licheszter::builder()
        .with_authentication(dotenv::var("LICHESS_API_TOKEN").expect("LICHESS_API_TOKEN not set"))
        .build();

    // ...or open the event stream
    let mut events = client.connect().await.unwrap();
    while let Ok(event) = events.next().await.unwrap() {
        println!("Event: {:?}", event);
        match event {
            licheszter::models::board::Event::GameStart { game } => {
                let result = client.bot_game_connect(&game.id).await;
                let mut board = Board::from_fen(&game.fen).unwrap();
                let playing_white = game.color == LichessColor::White;
                if board.white_to_move == playing_white {
                    println!("Playing first move of session");
                    calculate_and_play_move(board, depth, &client, &game.id).await;
                }
                match result {
                    Ok(mut stream) => {
                        println!("Connected to game stream: {}", game.id);
                        // Here you can handle the game stream, e.g., read moves and respond
                        while let Some(Ok(stream_event)) = stream.next().await {
                            println!("Stream event: {:?}", stream_event);
                            match stream_event {
                                BoardState::GameFull(game_full) => {
                                    println!("Game full state: {:?}", game_full);
                                }
                                BoardState::GameState(game_state) => {
                                    if game_state.status != GameStatus::Started {
                                        continue;
                                    }
                                    board = board
                                        .make_uci_move_temp(game_state.moves.split(" ").last().unwrap());
                                    if board.white_to_move != playing_white {
                                        println!(
                                            "It's not my turn, waiting for opponent's move..."
                                        );
                                        continue; // Wait for the opponent's move
                                    }
                                    calculate_and_play_move(board, depth, &client, &game.id)
                                        .await;
                                }
                                BoardState::ChatLine(chat_line) => {
                                    println!("Chat line: {}", chat_line.text);
                                }
                                BoardState::OpponentGone(opponent_gone) => {
                                    println!(
                                        "Opponent gone: {}",
                                        opponent_gone.claim_win_in_seconds
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to connect to game stream: {}", e),
                }
            }
            licheszter::models::board::Event::GameFinish { game } => {
                // Handle game finish event, e.g., log the result or clean up resources
                println!("Game finished: {}", game.id);
            }
            licheszter::models::board::Event::Challenge { challenge } => {
                // Here you can handle the challenge event, e.g., accept or decline it
                // For example, you could automatically accept challenges:
                let response = client.challenge_accept(&challenge.id);
                match response.await {
                    Ok(_) => println!("Challenge accepted: {}", challenge.id),
                    Err(e) => eprintln!("Failed to accept challenge: {}", e),
                }
            }
            licheszter::models::board::Event::ChallengeCanceled { challenge } => {
                // Handle challenge cancellation, e.g., log it or notify the user
                println!("Challenge canceled: {}", challenge.id);
            }
            licheszter::models::board::Event::ChallengeDeclined { challenge } => {
                // Handle challenge decline, e.g., log it or notify the user
                println!("Challenge declined: {}", challenge.id);
            }
        }
    }
}

async fn calculate_and_play_move(
    board: Board,
    depth: u32,
    client: &Licheszter,
    game_id: &str,
) {
    println!("Calculating best move for depth: {}", depth);
    if let Some(best_move) = search_entry(&board, depth) {
        println!("Best move: {}", best_move);
        let best_move_uci = best_move.to_uci();
        // Send the best move back to the game
        let response = client.bot_play_move(game_id, &best_move_uci, false);
        match response.await {
            Ok(_) => println!("Move sent: {}", best_move),
            Err(e) => eprintln!("Failed to send move: {}", e),
        }
    } else {
        println!("No legal moves available.");
    }
}
