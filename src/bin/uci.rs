use std::fs::OpenOptions;
use std::io::{self, BufRead, Write,};
use hhz::board::{Board};
use hhz::bot::Bot;
use vampirc_uci::{UciMessage, parse_one};

fn main() {
    // A simple loop to make it easier to attach the debugger.
    // This will pause the engine for 5 seconds when it starts.
    // Set a breakpoint AFTER this loop, for example on the `for line in...` line.
    // #[cfg(debug_assertions)]
    // {
    //     let start = Instant::now();
    //     while start.elapsed() < Duration::from_secs(5) {
    //         // Loop for 5 seconds to give you time to attach.
    //     }
    // }

    let mut stdout = io::stdout();

    for line in io::stdin().lock().lines() {
        let line_str = line.unwrap();
        let msg: UciMessage = parse_one(&line_str);

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
                stdout.flush().unwrap();
            }
            UciMessage::Debug(_) => todo!(),
            UciMessage::IsReady => {
                let ready_ok = UciMessage::ReadyOk;
                println!("{ready_ok}");
            },
            // UciMessage::Register { later, name, code } => todo!(),
            UciMessage::Position { startpos, fen, moves } => {
                let board = if startpos {
                    Board::default()
                } else if let Some(fen) = fen {
                    Board::from_fen(fen.as_str()).unwrap()
                } else {
                    panic!("No fen and no startpos send");
                };

                let bot = Bot::new(board, 1000);
            },
            // UciMessage::SetOption { name, value } => todo!(),
            // UciMessage::UciNewGame => todo!(),
            // UciMessage::Stop => todo!(),
            // UciMessage::PonderHit => todo!(),
            // UciMessage::Quit => todo!(),
            // UciMessage::Go { time_control, search_control } => todo!(),
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

        // Your engine logic would go here...
        // println!("Received message: {}", msg);
    }
}
