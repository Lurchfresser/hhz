use chrono::Duration;
use hhz::board::{Board, DEFAULT_FEN};
use hhz::bot::{Bot, BotMessage, SearchSpecs};
use log::{LevelFilter, error, info};
use std::io::{self, BufRead, Write};
use std::panic;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::{env, fs};
use vampirc_uci::UciTimeControl;
use vampirc_uci::{UciInfoAttribute, UciMessage, UciMove, UciPiece, UciSquare, parse_one};

fn main() {
    // Read the engine name that was set at compile time.
    let engine_name = env!("HHZ_ENGINE_NAME");

    // --- Setup Logging ---
    // This closure will be called when a panic occurs.
    panic::set_hook(Box::new(|panic_info| {
        // Log the panic information before the program exits.
        error!("PANIC OCCURRED: {}", panic_info);
    }));

    // --- Determine the correct log path, relative to the project root ---
    // Get the path to the running executable itself.
    let exe_path = env::current_exe().expect("Failed to get executable path");

    // The executable is in '.../versions/engine_name'. We go up two parent directories
    // to find the project root.
    let project_root = exe_path
        .parent() // -> gives '.../versions'
        .and_then(|p| p.parent()) // -> gives '.../' (the project root)
        .expect("Could not determine project root from executable path. Expected layout: '.../project_root/versions/engine_name'.");

    // Now construct the absolute path to the 'logs' directory.
    let log_dir = project_root.join("logs");
    fs::create_dir_all(&log_dir)
        .unwrap_or_else(|e| panic!("Failed to create log directory at {:?}: {}", log_dir, e));

    // Construct the full, absolute path for the log file.
    let log_path: PathBuf = log_dir.join(format!("{}.log", engine_name));

    // Initialize the logger to write to the specified absolute path.
    simple_logging::log_to_file(&log_path, LevelFilter::Info)
        .unwrap_or_else(|e| panic!("Failed to initialize logger for path {:?}: {}", log_path, e));

    info!("--- Logger initialized for {} ---", engine_name);

    let mut stdout = io::stdout();

    // --- Channel for UCI commands from the GUI ---
    let (stdin_tx, stdin_rx) = mpsc::channel();

    // --- Spawn a dedicated thread to read stdin without blocking the main loop ---
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line_str) = line {
                if stdin_tx.send(line_str).is_err() {
                    break;
                }
            } else {
                continue;
            }
        }
    });

    // --- Bot and its result channel ---
    let (result_tx, result_rx) = mpsc::channel::<BotMessage>();
    let mut bot = Bot::new(result_tx);
    loop {
        while let Ok(bot_message) = result_rx.try_recv() {
            match bot_message {
                BotMessage::Info { depth, best_move } => {
                    let uci_msg = UciMessage::Info(vec![
                        UciInfoAttribute::Depth(depth),
                        UciInfoAttribute::Pv(vec![string_to_uci_move(best_move.to_uci())]),
                    ]);
                    println!("{}", uci_msg);
                }
                BotMessage::BestMove(_move) => {
                    info!("Found best move: {}", _move.to_uci());
                    let uci_message = UciMessage::best_move(string_to_uci_move(_move.to_uci()));
                    writeln!(stdout, "{}", uci_message).unwrap();
                }
            }
            stdout.flush().unwrap();
        }

        // Check for commands from the GUI
        if let Ok(line_str) = stdin_rx.try_recv() {
            info!("<- {}", &line_str);
            let msg = parse_one(&line_str);

            match msg {
                UciMessage::Uci => {
                    // Read the engine name that was set at compile time.
                    let engine_name = env!("HHZ_ENGINE_NAME");

                    let message = UciMessage::Id {
                        name: Some(engine_name.to_string()),
                        author: Some("lurchfresser".to_string()),
                    };
                    println!("{message}");
                    let ok_meassage = UciMessage::UciOk;

                    println!("{ok_meassage}");
                }
                UciMessage::Debug(_) => todo!(),
                UciMessage::IsReady => {
                    let ready_ok = UciMessage::ReadyOk;
                    println!("{ready_ok}");
                }
                // UciMessage::Register { later, name, code } => todo!(),
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => {
                    let fen = if startpos {
                        DEFAULT_FEN
                    } else {
                        let x = fen.unwrap();
                        &x.as_str().to_owned()
                    };
                    let (board, rep_look_up, resetting_moves) = Board::from_fen_and_uci_moves(
                        fen,
                        &moves
                            .iter()
                            .map(|m| uci_move_to_string(m))
                            .collect::<Vec<String>>()
                            .join(" "),
                    )
                    .unwrap();

                    bot.set_position(board, rep_look_up, resetting_moves as u8);
                }
                // UciMessage::SetOption { name, value } => todo!(),
                // UciMessage::UciNewGame => todo!(),
                UciMessage::Stop => {
                    bot.stop();
                }
                // UciMessage::PonderHit => todo!(),
                // UciMessage::Quit => todo!(),
                UciMessage::Go {
                    time_control,
                    search_control,
                } => {
                    bot.start_searching(time_control_to_search_specs(time_control));
                }
                // UciMessage::Id { name, author } => todo!(),
                // UciMessage::UciOk => todo!(),
                // UciMessage::ReadyOk => todo!(),
                // UciMessage::BestMove { best_move, ponder } => todo!(),
                // UciMessage::CopyProtection(protection_state) => todo!(),
                // UciMessage::Registration(protection_state) => todo!(),
                // UciMessage::Option(uci_option_config) => todo!(),
                // UciMessage::Info(uci_info_attributes) => todo!(),
                // UciMessage::Unknown(_, error) => todo!(),
                _ => {}
            }
            stdout.flush().unwrap();
        }
        thread::sleep(std::time::Duration::from_millis(1));
    }
    info!("--- Shutting down ---");
}

fn string_to_uci_move(uci_string: String) -> UciMove {
    let from = UciSquare {
        file: uci_string.chars().next().unwrap(),
        rank: uci_string.chars().nth(1).unwrap().to_digit(10).unwrap() as u8,
    };
    let to = UciSquare {
        file: uci_string.chars().nth(2).unwrap(),
        rank: uci_string.chars().nth(3).unwrap().to_digit(10).unwrap() as u8,
    };
    UciMove {
        from,
        to,
        promotion: if let Some(piece_char) = uci_string.chars().nth(4) {
            UciPiece::from_str(&piece_char.to_string()).ok()
        } else {
            None
        },
    }
}

fn uci_move_to_string(uci_move: &UciMove) -> String {
    format!(
        "{}{}{}{}{}",
        uci_move.from.file,
        uci_move.from.rank.to_ascii_lowercase(),
        uci_move.to.file,
        uci_move.to.rank.to_ascii_lowercase(),
        uci_move
            .promotion
            // Now, this closure returns an owned String, not a reference.
            .map_or(String::new(), |p| p
                .as_char()
                .unwrap()
                .to_string()
                .to_lowercase())
    )
}

fn time_control_to_search_specs(time_control: Option<UciTimeControl>) -> SearchSpecs {
    if let Some(time_control) = time_control {
        match time_control {
            UciTimeControl::Ponder => SearchSpecs::Infinite,
            UciTimeControl::Infinite => SearchSpecs::Infinite,
            UciTimeControl::TimeLeft {
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
            } => SearchSpecs::TimeLeft {
                white_time: time_delta_to_duration(white_time),
                black_time: time_delta_to_duration(black_time),
                white_increment: time_delta_to_duration(white_increment),
                black_increment: time_delta_to_duration(black_increment),
                moves_to_go,
            },
            UciTimeControl::MoveTime(move_time) => {
                SearchSpecs::MoveTime(time_delta_to_duration(Some(move_time)).unwrap())
            }
        }
    } else {
        SearchSpecs::Infinite
    }
}

fn time_delta_to_duration(duration: Option<Duration>) -> Option<core::time::Duration> {
    if let Some(duration) = duration {
        Some(core::time::Duration::from_nanos(
            duration.num_nanoseconds().unwrap().try_into().unwrap(),
        ))
    } else {
        None
    }
}
