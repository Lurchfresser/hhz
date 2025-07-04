use hhz::board::Board;
use rouille::input::json_input;
use rouille::{Response, router, try_or_400};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::env::{var};
use std::fmt::format;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GameRequest {
    pub fen: String, // Changed from &'a str to String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MoveRepresentation {
    pub uci_move: String,
    pub resul_fen: String,
}

fn main() {
    let board = Arc::new(Mutex::new(Board::default()));

    let url_base = std::env::var("URL_BASE").unwrap_or("localhost".parse().unwrap());

    let url = url_base + ":42069";
    println!("Starting server on {}", url);
    rouille::start_server(url, move |request| {
        // Rest of closure remains the same
        println!("received request on url: {}", request.url());
        if request.method() == "OPTIONS" {
            // CORS handling remains unchanged
            return Response::text("")
                .with_status_code(200)
                .with_additional_header("Access-Control-Allow-Origin", "*")
                .with_additional_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                .with_additional_header("Access-Control-Allow-Headers", "Content-Type");
        }
        router!(request,
            (POST) (/startgame) => {
                let game_request: GameRequest = try_or_400!(json_input(request));
                let mut board_guard = board.lock().unwrap();
                match Board::from_fen(&game_request.fen) {
                    Ok(new_board) => {
                        *board_guard = new_board;
                        println!("game fen: {}", game_request.fen);
                        Response::text("success").with_status_code(200)
                    }
                    Err(e) => {
                        eprintln!("Invalid FEN: {}", e);
                        Response::text(format!("Invalid FEN: {}", e)).with_status_code(400)
                    }
                }
            },
            (POST) (/move) => {
                let move_request: MoveRepresentation = try_or_400!(json_input(request));
                println!("received move: {}", move_request.uci_move);
                
                let old_board = board.lock().unwrap();

                let moves = old_board.generate_legal_moves_temp();
                
                // Find the requested move
                let requested_move = match moves.iter().find(|m| {
                    m.to_uci() == move_request.uci_move
                }) {
                    Some(mv) => *mv,
                    None => {
                        eprintln!("Invalid move: {}", move_request.uci_move);
                        return Response::text(format!("Invalid move: {}", move_request.uci_move))
                            .with_status_code(400);
                    }
                };
                
                // Make the move
                let new_board = old_board.make_move_temp(requested_move);
                drop(old_board);
                // Check if the FEN matches what was expected
                let current_fen = new_board.to_fen();
                if move_request.resul_fen != current_fen {
                    eprintln!("FEN mismatch! Expected: {}, Got: {}", move_request.resul_fen, current_fen);
                    return Response::text(format!(
                        "FEN mismatch! Expected: {}, Got: {}", 
                        move_request.resul_fen,
                        current_fen
                    )).with_status_code(409); // 409 Conflict
                }
                
                // Generate legal moves for the current position
                let my_legal_moves = new_board.generate_legal_moves_temp();
                
                // Check if there are any legal moves available
                let next_move = match my_legal_moves.first() {
                    Some(mv) => *mv,
                    None => {
                        eprintln!("No legal moves available");
                        return Response::text("No legal moves available")
                            .with_status_code(422); // 422 Unprocessable Entity
                    }
                };
                
                // Make the AI's move
                let after_my_move = new_board.make_move_temp(next_move);

                *board.lock().unwrap() = after_my_move;

                Response::json(
                    &MoveRepresentation {
                        uci_move: next_move.to_uci(),
                        resul_fen: after_my_move.to_fen()
                    }
                ).with_status_code(200)
            },
            _ => Response::text("Not found").with_status_code(404)
        )
            .with_additional_header("Access-Control-Allow-Origin", "*")
            .with_additional_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .with_additional_header("Access-Control-Allow-Headers", "Content-Type")
    });
}
