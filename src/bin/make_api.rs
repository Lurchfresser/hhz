use chrono::Local;
use chrono::DateTime;
use hhz::board::Board;
use hhz::search::search_entry;
use rouille::Request;
use rouille::input::json_input;
use rouille::{Response, router, try_or_400};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GameRequest {
    pub fen: String, // Changed from &'a str to String
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct MoveRepresentation {
    pub uci_move: String,
    pub resul_fen: String,
}

fn main() {
    let depth = 2;
    let board = Arc::new(Mutex::new(Board::default()));

    let url_base = std::env::var("URL_BASE").unwrap_or("0.0.0.0".parse().unwrap());

    let url = url_base + ":42069";
    println!("Starting server on {}", url);
    rouille::start_server(url, move |request| {
        // Rest of closure remains the same
        let current_local: DateTime<Local> = Local::now();
        println!(
            "{}: {} request on url: {}",
            current_local,
            request.method(),
            request.url()
        );
        if request.method() == "OPTIONS" {
            // CORS handling remains unchanged
            return Response::text("")
                .with_status_code(200)
                .with_additional_header("Access-Control-Allow-Origin", "*")
                .with_additional_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                .with_additional_header("Access-Control-Allow-Headers", "Content-Type");
        }
        router!(request,
            (POST) (/startgame) => {new_game(request, board.clone(), depth)},
            (POST) (/move) => { on_move(request, board.clone(), depth)},
            _ => Response::text("Not found").with_status_code(404)
        )
        .with_additional_header("Access-Control-Allow-Origin", "*")
        .with_additional_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .with_additional_header("Access-Control-Allow-Headers", "Content-Type")
    });
}

fn new_game(request: &Request, board: Arc<Mutex<Board>>, depth: u8) -> Response {
    let game_request: GameRequest = try_or_400!(json_input(request));
    let mut board_guard = board.lock().unwrap();
    match Board::from_fen(&game_request.fen) {
        Ok(new_board) => {
            *board_guard = new_board;
            println!("game fen: {}", game_request.fen);
            if new_board.white_to_move == (game_request.color == "white") {
                let next_move = search_entry(&new_board, depth);
                // Make the AI's move
                let after_my_move = new_board.make_move_temp(
                    next_move.expect("No move found, maybe stalemate or checkmate?"),
                );

                *board_guard = after_my_move;

                Response::json(&MoveRepresentation {
                    uci_move: next_move
                        .expect("No move found, maybe stalemate or checkmate?")
                        .to_uci(),
                    resul_fen: after_my_move.to_fen(),
                })
                .with_status_code(200)
            } else {
                Response::text("success").with_status_code(200)
            }
        }
        Err(e) => {
            eprintln!("Invalid FEN: {}", e);
            Response::text(format!("Invalid FEN: {}", e)).with_status_code(400)
        }
    }
}

fn on_move(request: &Request, board: Arc<Mutex<Board>>, depth: u8) -> Response {
    let move_request: MoveRepresentation = try_or_400!(json_input(request));
    println!("received move: {}", move_request.uci_move);
    let old_board = board.lock().unwrap();

    let moves = old_board.generate_legal_moves_temp();
    // Find the requested move
    let requested_move = match moves.iter().find(|m| m.to_uci() == move_request.uci_move) {
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
        eprintln!(
            "FEN mismatch! Expected: {}, Got: {}",
            move_request.resul_fen, current_fen
        );
        return Response::text(format!(
            "FEN mismatch! Expected: {}, Got: {}",
            current_fen, move_request.resul_fen
        ))
        .with_status_code(409); // 409 Conflict
    }
    let next_move = search_entry(&new_board, depth);
    // Make the AI's move
    let after_my_move =
        new_board.make_move_temp(next_move.expect("No move found, maybe stalemate or checkmate?"));

    *board.lock().unwrap() = after_my_move;

    return Response::json(&MoveRepresentation {
        uci_move: next_move
            .expect("No move found, maybe stalemate or checkmate?")
            .to_uci(),
        resul_fen: after_my_move.to_fen(),
    })
    .with_status_code(200);
}
