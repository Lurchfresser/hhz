use dotenv::dotenv;
use futures_util::StreamExt;
use hhz::board::Board;
use hhz::board::DEFAULT_FEN;
use hhz::bot::{Bot, BotMessage};
use licheszter::models::game::{Color as LichessColor, GameEventInfo, GameStatus};
use licheszter::{client::Licheszter, models::board::BoardState};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, mpsc};
use std::thread::sleep;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::{Mutex, mpsc::Receiver as TokioReceiver};
use tokio::task;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load environment variables from .env file

    let client = Licheszter::builder()
        .with_authentication(dotenv::var("LICHESS_API_TOKEN").expect("LICHESS_API_TOKEN not set"))
        .build();

    let client_guard: Arc<Mutex<Licheszter>> = Arc::new(Mutex::new(client));

    println!("Connected to Lichess event stream.");

    let binding = client_guard.clone();
    let client = binding.lock().await;

    let mut events = client.connect().await.expect("Failed connect");
    drop(client);

    while let Ok(event) = events.next().await.unwrap() {
        println!("received Event in main loop: {:?}", event);
        match event {
            licheszter::models::board::Event::GameStart { game } => {
                spawn(handle_game(game, client_guard.clone()));
            }
            licheszter::models::board::Event::GameFinish { game } => {
                println!("Game finished: {}", game.id);
            }
            licheszter::models::board::Event::Challenge { challenge } => {
                println!("Challenge received from: {}", challenge.challenger.name);
                let client = client_guard.lock().await;
                let response = client.challenge_accept(&challenge.id);
                match response.await {
                    Ok(_) => println!("Challenge accepted: {}", challenge.id),
                    Err(e) => eprintln!("Failed to accept challenge: {}", e),
                }
            }
            licheszter::models::board::Event::ChallengeCanceled { challenge } => {
                println!("Challenge canceled: {}", challenge.id);
            }
            licheszter::models::board::Event::ChallengeDeclined { challenge } => {
                println!("Challenge declined: {}", challenge.id);
            }
        }
    }
}

async fn handle_game(game: GameEventInfo, client_guard: Arc<Mutex<Licheszter>>) {
    let (sender, receiver) = mpsc::channel::<BotMessage>();
    // 2. Create the asynchronous channel for our Tokio tasks.
    let (async_sender, async_receiver) = tokio::sync::mpsc::channel::<BotMessage>(32);

    // 3. Spawn the bridge task.
    let game_id_clone = game.id.clone();
    task::spawn_blocking(move || {
        // This `for` loop on a std::sync::mpsc::Receiver blocks the current
        // (dedicated blocking) thread until a message is available.
        for bot_message in receiver {
            // Forward the message to the async channel.
            if async_sender.blocking_send(bot_message).is_err() {
                // This happens if the async receiver is dropped (e.g., the game ends).
                // We can then gracefully shut down the bridge.
                println!(
                    "Bridge: Async receiver for game {} closed. Shutting down bridge.",
                    game_id_clone
                );
                break;
            }
            sleep(Duration::from_millis(1));
        }
    });
    spawn(send_bot_moves(
        game.clone(),
        async_receiver,
        client_guard.clone(),
    ));
    let mut bot = Bot::new(sender);

    let playing_white = if game.color == LichessColor::White {
        true
    } else {
        false
    };

    let client = client_guard.lock().await;
    let mut game_stream = client.bot_game_connect(&game.id).await.unwrap();
    drop(client);

    while let Some(Ok(board_state)) = game_stream.next().await {
        println!("Board state: {:#?}", board_state);
        match board_state {
            BoardState::GameState(game_state) => {
                println!("received game status: {:?}", game_state.status);
                match game_state.status {
                    GameStatus::Created => {}
                    GameStatus::Started => {}
                    // game ending
                    GameStatus::Aborted
                    | GameStatus::Mate
                    | GameStatus::Resign
                    | GameStatus::Stalemate
                    | GameStatus::Timeout
                    | GameStatus::Draw
                    | GameStatus::OutOfTime
                    | GameStatus::Cheat => continue,
                    // unkown
                    GameStatus::NoStart => {}
                    GameStatus::UnknownFinish => {}
                    GameStatus::VariantEnd => {}
                };
                //TODO: use dynamic fen
                let (board, rep_look_up, num_m_resets) = match Board::from_fen_and_uci_moves(DEFAULT_FEN, &game_state.moves) {
                    Ok(board) => board,
                    Err(err) => {
                        println!("error while parsing board in handle_game: {}", err);
                        break;
                    }
                };
                bot.set_position(board, rep_look_up, num_m_resets as u8);
                if board.white_to_move == playing_white {
                    bot.start_searching();
                } else {
                    //todo: ponder
                };
            }
            BoardState::ChatLine(_) => continue,
            BoardState::GameFull(game_ful) => {
                //TODO: use dynamic fen
                let (board, rep_look_up, num_m_resets) = match Board::from_fen_and_uci_moves(DEFAULT_FEN, &game_ful.state.moves)
                {
                    Ok(board) => board,
                    Err(err) => {
                        println!("error while parsing board in handle_game: {}", err);
                        break;
                    }
                };
                bot.set_position(board, rep_look_up, num_m_resets as u8);
                if board.white_to_move == playing_white {
                    bot.start_searching();
                } else {
                    //todo: ponder
                };
            }
            BoardState::OpponentGone(_) => {
                bot.stop();
                continue;
            }
        };
    }
    bot.quit();
    println!("Finished handling game: {}", game.id);
}

async fn send_bot_moves(
    game: GameEventInfo,
    // It takes the ASYNC receiver.
    mut receiver: TokioReceiver<BotMessage>,
    client_guard: Arc<Mutex<Licheszter>>,
) {
    // This loop `await`s messages without blocking the Tokio runtime.
    // It will wait indefinitely until a message arrives or the channel is closed.
    while let Some(bot_message) = receiver.recv().await {
        println!(
            "Received bot message for game {}: {:#?}",
            game.id, bot_message
        );
        match bot_message {
            BotMessage::Info(move_info) => {
                println!("Info from bot for game {}: {}", game.id, move_info);
            }
            BotMessage::BestMove(best_move) => {
                let uci_move = best_move.to_uci();
                println!("Sending best move {} for game {}", uci_move, game.id);
                let client = client_guard.lock().await;
                // You must .await the future returned by bot_play_move
                match client.bot_play_move(&game.id, &uci_move, false).await {
                    Ok(_) => println!("Successfully sent move {} to Lichess.", uci_move),
                    Err(e) => eprintln!("Failed to send move {} to Lichess: {}", uci_move, e),
                }
            }
        };
    }
    println!(
        "Bot message channel closed for game {}. The send_bot_moves task is ending.",
        game.id
    );
}
