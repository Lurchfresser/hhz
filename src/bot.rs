use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread::{self, sleep},
};

use crate::tt_table::TT_Table;
use crate::{board::*, moves::*, search::*};
use core::time::Duration;

// Protocol: Messages sent FROM the main thread TO the bot thread.
#[derive(Debug)]
pub enum BotCommand {
    SetBoard(Board, [u64; 100], u8),
    Search(SearchSpecs),
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum SearchSpecs {
    Infinite,
    TimeLeft {
        /// White's time on the clock, in milliseconds.
        white_time: Option<Duration>,

        /// Black's time on the clock, in milliseconds.
        black_time: Option<Duration>,

        /// White's increment per move, in milliseconds.
        white_increment: Option<Duration>,

        /// Black's increment per move, in milliseconds.
        black_increment: Option<Duration>,

        /// The number of moves to go to the next time control.
        moves_to_go: Option<u8>,
    },
    MoveTime(Duration),
}

// Protocol: Messages sent FROM the bot thread TO the main thread.
#[derive(Debug)]
pub enum BotMessage {
    Info { best_move: Move, depth: u8 },
    BestMove(Move),
}

/// The public-facing Bot API.
/// This is a lightweight handle that you interact with from your main thread.
/// It just sends commands to the actual worker thread.
pub struct Bot {
    command_tx: Sender<BotCommand>,
    // The handle is optional, in case we want to join it on quit.
    thread_handle: Option<thread::JoinHandle<()>>,
    is_searching: Arc<AtomicBool>,
    board: Board,
}

impl Bot {
    /// Creates a new Bot and spawns its worker thread.
    /// It takes a Sender for the worker to send messages (like moves and info) back to the main thread.
    pub fn new(result_tx: Sender<BotMessage>) -> Self {
        let is_searching = Arc::new(AtomicBool::new(false));
        let (command_tx, command_rx) = mpsc::channel();
        let mut worker = BotWorker::new(result_tx, is_searching.clone());

        let thread_handle = thread::spawn(move || {
            // The worker thread's main loop. It blocks here waiting for commands.
            for command in command_rx {
                match command {
                    BotCommand::SetBoard(board, repetition_lookup, num_resetting_moves) => {
                        worker.set_position(board, repetition_lookup, num_resetting_moves)
                    }
                    BotCommand::Search(specs) => {
                        worker.search();
                    }
                    BotCommand::Quit => break, // Exit the loop and end the thread
                }
            }
        });

        Self {
            command_tx,
            thread_handle: Some(thread_handle),
            is_searching,
            board: Board::default()
        }
    }

    /// Sets the board position for the bot.
    pub fn set_position(
        &mut self,
        board: Board,
        repetition_lookup: [u64; 100],
        num_resetting_moves: u8,
    ) {
        self.board = board;
        self.stop();
        self.command_tx
            .send(BotCommand::SetBoard(
                board,
                repetition_lookup,
                num_resetting_moves,
            ))
            .unwrap();
    }

    /// Tells the bot to start searching for the best move. This returns immediately.
    pub fn start_searching(&self, specs: SearchSpecs) {
        match specs {
            SearchSpecs::Infinite => {
                self.command_tx
                    .send(BotCommand::Search(SearchSpecs::Infinite))
                    .unwrap();
            }
            SearchSpecs::TimeLeft {
                white_time: white_time,
                black_time: black_time,
                white_increment: white_increment,
                black_increment: black_increment,
                moves_to_go: moves_to_go,
            } => {
                let my_time = if self.board.white_to_move {
                    white_time
                } else {
                    black_time
                };
                if my_time.is_none() {
                    return;
                }
                let my_increment = if self.board.white_to_move {
                    white_increment
                } else {
                    black_increment
                };
                let move_time = (my_time.unwrap() / 40) + my_increment.unwrap_or(
                    Duration::from_millis(0)
                );

                let is_searching = self.is_searching.clone();
                set_time_out(move_time, move || {
                    is_searching.store(false, Ordering::Relaxed);
                });
                self.command_tx
                    .send(BotCommand::Search(SearchSpecs::Infinite))
                    .unwrap();
            }
            SearchSpecs::MoveTime(move_time) => {
                let is_searching = self.is_searching.clone();
                set_time_out(move_time, move || {
                    is_searching.store(false, Ordering::Relaxed);
                });
                self.command_tx
                    .send(BotCommand::Search(SearchSpecs::Infinite))
                    .unwrap();
            }
        };
    }

    /// Tells the bot to stop its current search.
    pub fn stop(&self) {
        self.is_searching.store(false, Ordering::Relaxed);
    }

    /// Tells the bot to quit and cleans up the thread.
    pub fn quit(&mut self) {
        self.command_tx.send(BotCommand::Quit).unwrap();
        if let Some(handle) = self.thread_handle.take() {
            handle.join().expect("Bot thread panicked during quit");
        }
    }
}

/// The internal worker that lives in its own thread and does all the heavy lifting.
struct BotWorker {
    board: Board,
    tt_table: TT_Table,
    result_tx: Sender<BotMessage>,
    // This flag is essential for stopping the search gracefully.
    is_searching: Arc<AtomicBool>,
    repetition_lookup: [u64; 100],
    num_resetting_moves: u8,
}

impl BotWorker {
    fn new(result_tx: Sender<BotMessage>, is_searching: Arc<AtomicBool>) -> Self {
        Self {
            board: Board::default(),
            tt_table: TT_Table::new(),
            result_tx,
            is_searching,
            repetition_lookup: [0; 100],
            num_resetting_moves: 0,
        }
    }

    fn set_position(
        &mut self,
        board: Board,
        repetition_lookup: [u64; 100],
        num_resetting_moves: u8,
    ) {
        self.board = board;
        self.repetition_lookup = repetition_lookup;
        self.num_resetting_moves = num_resetting_moves;
    }

    /// The main search entry point, implementing iterative deepening.
    fn search(&mut self) {
        // Set the searching flag to true and clone it so the search function can check it.
        self.is_searching.store(true, Ordering::Relaxed);
        let mut best_move_so_far = None;

        // --- Iterative Deepening Loop ---
        let mut depth = 0;
        loop {
            // Go up to a max depth
            // Check if we were told to stop BEFORE starting the next depth.
            if !self.is_searching.load(Ordering::Relaxed) {
                break;
            }

            // You'll need to adapt your search function to accept the stop flag.
            let result = search_entry(
                &self.board,
                depth,
                &mut self.tt_table,
                &mut self.repetition_lookup,
                self.num_resetting_moves,
                &self.is_searching,
            );

            // If the search was stopped mid-way (result is None) or if there are no moves, break.
            if let Some(best_move_at_depth) = result {
                best_move_so_far = Some(best_move_at_depth);
                // Send an 'info' message back to the main thread.
                let info_msg = BotMessage::Info {
                    best_move: best_move_at_depth,
                    depth,
                };
                if let Err(e) = self.result_tx.send(info_msg) {
                    panic!("{}", e)
                }
            } else {
                // Search was stopped or completed without finding a better move
                break;
            }
            depth += 1;
        }

        // After the loop (or when stopped), send the best move found so far.
        self.is_searching.store(false, Ordering::Relaxed);
        self.result_tx
            //TODO: error handling
            .send(BotMessage::BestMove(best_move_so_far.unwrap()))
            .unwrap();
    }
}

pub fn set_time_out<F: Fn() + Send + 'static>(duration: Duration, c: F) {
    thread::spawn(move || {
        sleep(duration);
        c();
    });
}
