use hhz::board::Board;
use hhz::bot::{Bot, BotMessage};
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write, stdout};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use vampirc_uci::{UciMessage, UciMove, UciPiece, UciSquare, parse_one};

fn main() {
    let mut stdout = io::stdout();

    // --- Setup Logging ---
    let log_path = "/home/lurchfresser/Desktop/coding/chess/hhz/uci_log.txt";
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true) // Use write to clear the file on each start
        .truncate(true)
        .open(log_path)
        .unwrap();

    // --- Channel for UCI commands from the GUI ---
    let (stdin_tx, stdin_rx) = mpsc::channel();

    // --- Spawn a dedicated thread to read stdin without blocking the main loop ---
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line_str) = line {
                // If the main thread has shut down, we can stop.
                if stdin_tx.send(line_str).is_err() {
                    break;
                }
            }
        }
    });

    // --- Bot and its result channel ---
    let (result_tx, result_rx) = mpsc::channel::<BotMessage>();
    let mut bot = Bot::new(result_tx);
    loop {
        while let Ok(bot_message) = result_rx.try_recv() {
            match bot_message {
                BotMessage::Info(_) => {
                    // let uci_msg = UciMessage::Info(

                    // );
                    // println!("{}", uci_msg);
                }
                BotMessage::BestMove(_move) => {
                    let uci_message = UciMessage::best_move(string_to_uci_move(_move.to_uci()));
                    println!("{uci_message}");
                }
            }
            stdout.flush().unwrap();
        }

        if let Ok(line_str) = stdin_rx.try_recv() {
            writeln!(log_file, "<- {}", &line_str).unwrap();
            log_file.flush().unwrap();

            let msg = parse_one(&line_str);
            // Only log in debug builds
            if cfg!(debug_assertions) {
                // Use an absolute path to ensure the log file is always in your project folder
                let log_path = "/home/lurchfresser/Desktop/coding/chess/hhz/uci_log.txt";
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_path)
                    .unwrap();

                // Write the message and immediately flush it to the file
                writeln!(file, "Received: {}", msg).unwrap();
                file.flush().unwrap();
            }

            match msg {
                UciMessage::Uci => {
                    let message = UciMessage::Id {
                        name: Some("hhz".to_string()),
                        author: Some("lurchfresser".to_string()),
                    };
                    println!("{message}");
                    let ok_meassage = UciMessage::UciOk;

                    println!("{ok_meassage}"); // Outputs "option name Selectivity type spin default 2 min 0 max 4"
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
                    let mut board = if startpos {
                        Board::default()
                    } else if let Some(fen) = fen {
                        Board::from_fen(fen.as_str()).unwrap()
                    } else {
                        panic!("No fen and no startpos send");
                    };
                    for _move in moves {
                        board = board.make_uci_move_temp(&uci_move_to_string(_move));
                    }

                    bot.set_position(board);
                }
                // UciMessage::SetOption { name, value } => todo!(),
                // UciMessage::UciNewGame => todo!(),
                // UciMessage::Stop => todo!(),
                // UciMessage::PonderHit => todo!(),
                // UciMessage::Quit => todo!(),
                UciMessage::Go {
                    time_control,
                    search_control,
                } => {
                    bot.start_searching();
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
        // Your engine logic would go here...
        // println!("Received message: {}", msg);
    }
}

fn string_to_uci_move(uci_string: String) -> UciMove {
    let from = UciSquare {
        file: uci_string.chars().nth(0).unwrap(),
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

fn uci_move_to_string(uci_move: UciMove) -> String {
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
