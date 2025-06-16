use std::fmt::Debug;

use chessie::prelude::*;
use dotenv::dotenv;
use futures_util::StreamExt;
use licheszter::models::game::Color as LichessColor;
use licheszter::{client::Licheszter, models::board::BoardState};
#[tokio::main]
async fn main() {
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
                let mut chessie_game = Game::from_fen(&game.fen).unwrap();
                let my_color = if game.color == LichessColor::White {
                    chessie::Color::White
                } else {
                    chessie::Color::Black
                };
                if chessie_game.position().side_to_move() == my_color {
                    println!("Playing first move of session");
                    let depth = 0; // Set the search depth
                    if let Some(best_move) = simple_search(&chessie_game, depth) {
                        println!("Best move: {}", best_move);
                        let best_move_uci = best_move.to_uci();
                        // Send the best move back to the game
                        let response = client.bot_play_move(&game.id, &best_move_uci, false);
                        match response.await {
                            Ok(_) => println!("Move sent: {}", best_move),
                            Err(e) => eprintln!("Failed to send move: {}", e),
                        }
                    } else {
                        println!("No legal moves available.");
                    }
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
                                    let chessie_result = chessie_game
                                        .make_move_uci(game_state.moves.split(" ").last().unwrap());
                                    if let Err(e) = chessie_result {
                                        eprintln!("Failed to make move: {}", e);
                                        panic!();
                                    }
                                    if chessie_game.position().side_to_move() != my_color {
                                        println!(
                                            "It's not my turn, waiting for opponent's move..."
                                        );
                                        continue; // Wait for the opponent's move
                                    }
                                    let depth = 3; // Set the search depth
                                    if let Some(best_move) = simple_search(&chessie_game, depth) {
                                        println!("Best move: {}", best_move);
                                        let best_move_uci = best_move.to_uci();
                                        // Send the best move back to the game
                                        let response =
                                            client.bot_play_move(&game.id, &best_move_uci, false);
                                        match response.await {
                                            Ok(_) => println!("Move sent: {}", best_move),
                                            Err(e) => eprintln!("Failed to send move: {}", e),
                                        }
                                    } else {
                                        println!("No legal moves available.");
                                    }
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

fn simple_search(game: &Game, depth: u32) -> Option<Move> {
    let legal_moves = game.get_legal_moves();
    if legal_moves.is_empty() {
        return None; // No legal moves available
    }
    let mut best_move = None;
    let mut best_score = if game.position().side_to_move() == Color::White {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };
    for move_ in legal_moves {
        let new_game = game.with_move_made(move_);
        let score = min_max_search(&new_game, depth);
        if score > best_score && game.position().side_to_move() == Color::White
            || score < best_score && game.position().side_to_move() == Color::Black
        {
            best_score = score;
            best_move = Some(move_);
        }
    }
    best_move
}

fn min_max_search(game: &Game, depth: u32) -> i32 {
    if depth == 0 {
        return evaluate_board(game);
    }
    let mut best_score = if (game.position().side_to_move() == Color::White) {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };
    let legal_moves = game.get_legal_moves();
    match check_game_result(game) {
        GameResult::WhiteWins => return i32::MAX,
        GameResult::BlackWins => return i32::MIN,
        GameResult::Draw(_) => return 0,
        GameResult::Ongoing => {}
    }
    for move_ in legal_moves {
        let new_game = game.with_move_made(move_);
        let score = min_max_search(&new_game, depth - 1);
        if (score > best_score && game.position().side_to_move() == Color::White)
            || (score < best_score && game.position().side_to_move() == Color::Black)
        {
            best_score = score;
        }
    }
    best_score
}

pub fn evaluate_board(game: &Game) -> i32 {
    // A simple evaluation function that counts material balance
    game.board().score()
}

pub trait PiecesScore {
    fn score(&self) -> i32;
}

impl PiecesScore for Board {
    fn score(&self) -> i32 {
        let mut score: i32 = 0;
        score += i32::from(self.pawns(Color::White).population()) * 100;
        score += i32::from(self.knights(Color::White).population()) * 300;
        score += i32::from(self.bishops(Color::White).population()) * 320;
        score += i32::from(self.rooks(Color::White).population()) * 500;
        score += i32::from(self.queens(Color::White).population()) * 900;
        score -= i32::from(self.pawns(Color::Black).population()) * 100;
        score -= i32::from(self.knights(Color::Black).population()) * 300;
        score -= i32::from(self.bishops(Color::Black).population()) * 320;
        score -= i32::from(self.rooks(Color::Black).population()) * 500;
        score -= i32::from(self.queens(Color::Black).population()) * 900;
        score
    }
}


#[derive(Debug, PartialEq)]
enum GameResult {
    Ongoing,
    WhiteWins,
    BlackWins,
    Draw(DrawReason),
}

#[derive(Debug, PartialEq)]
enum DrawReason {
    Stalemate,
    FiftyMoveRule,
    InsufficientMaterial,
    Repetition,
}

fn check_game_result(game: &Game) -> GameResult {
    // Check for draw conditions first
    if game.can_draw_by_fifty() {
        return GameResult::Draw(DrawReason::FiftyMoveRule);
    }
    
    if game.can_draw_by_insufficient_material() {
        return GameResult::Draw(DrawReason::InsufficientMaterial);
    }
    
    // Check if there are any legal moves
    let legal_moves = game.get_legal_moves();
    
    if legal_moves.is_empty() {
        if game.is_in_check() {
            // Checkmate - the opponent wins
            match game.side_to_move() {
                chessie::Color::White => GameResult::BlackWins,
                chessie::Color::Black => GameResult::WhiteWins,
            }
        } else {
            // Stalemate
            GameResult::Draw(DrawReason::Stalemate)
        }
    } else {
        GameResult::Ongoing
    }
}