use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use crate::{board::*, moves::*, search::*};

// Protocol: Messages sent FROM the main thread TO the bot thread.
#[derive(Debug)]
pub enum BotCommand {
    SetBoard(Board),
    Search,
    Stop,
    Quit,
}

// Protocol: Messages sent FROM the bot thread TO the main thread.
#[derive(Debug)]
pub enum BotMessage {
    Info(Move),
    BestMove(Move),
}

/// The public-facing Bot API.
/// This is a lightweight handle that you interact with from your main thread.
/// It just sends commands to the actual worker thread.
pub struct Bot {
    command_tx: Sender<BotCommand>,
    // The handle is optional, in case we want to join it on quit.
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Bot {
    /// Creates a new Bot and spawns its worker thread.
    /// It takes a Sender for the worker to send messages (like moves and info) back to the main thread.
    pub fn new(result_tx: Sender<BotMessage>) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let mut worker = BotWorker::new(result_tx);

        let thread_handle = thread::spawn(move || {
            // The worker thread's main loop. It blocks here waiting for commands.
            for command in command_rx {
                match command {
                    BotCommand::SetBoard(board) => worker.set_position(board),
                    BotCommand::Search => worker.search(),
                    BotCommand::Stop => worker.stop(),
                    BotCommand::Quit => break, // Exit the loop and end the thread
                }
            }
        });

        Self {
            command_tx,
            thread_handle: Some(thread_handle),
        }
    }

    /// Sets the board position for the bot.
    pub fn set_position(&self, board: Board) {
        self.stop();
        self.command_tx.send(BotCommand::SetBoard(board)).unwrap();
    }

    /// Tells the bot to start searching for the best move. This returns immediately.
    pub fn start_searching(&self) {
        self.command_tx.send(BotCommand::Search).unwrap();
    }

    /// Tells the bot to stop its current search.
    pub fn stop(&self) {
        self.command_tx.send(BotCommand::Stop).unwrap();
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
    result_tx: Sender<BotMessage>,
    // This flag is essential for stopping the search gracefully.
    is_searching: Arc<AtomicBool>,
}

impl BotWorker {
    fn new(result_tx: Sender<BotMessage>) -> Self {
        Self {
            board: Board::default(),
            result_tx,
            is_searching: Arc::new(AtomicBool::new(false)),
        }
    }

    fn set_position(&mut self, board: Board) {
        self.stop();
        self.board = board;
    }

    fn stop(&mut self) {
        // Set the flag to false. The search loop will see this and stop.
        self.is_searching.store(false, Ordering::Relaxed);
    }

    /// The main search entry point, implementing iterative deepening.
    fn search(&mut self) {
        // Set the searching flag to true and clone it so the search function can check it.
        self.is_searching.store(true, Ordering::Relaxed);
        let mut best_move_so_far = None;

        // --- Iterative Deepening Loop ---
        for depth in 1..3 {
            // Go up to a max depth
            // Check if we were told to stop BEFORE starting the next depth.
            if !self.is_searching.load(Ordering::Relaxed) {
                break;
            }

            // You'll need to adapt your search function to accept the stop flag.
            let result = search_entry(&self.board, depth);

            // If the search was stopped mid-way (result is None) or if there are no moves, break.
            if let Some(best_move_at_depth) = result {
                best_move_so_far = Some(best_move_at_depth);
                // Send an 'info' message back to the main thread.
                let info_msg = BotMessage::Info(best_move_at_depth);
                self.result_tx.send(info_msg).unwrap();
            } else {
                // Search was stopped or completed without finding a better move
                break;
            }
        }

        // After the loop (or when stopped), send the best move found so far.
        self.is_searching.store(false, Ordering::Relaxed);
        self.result_tx
            //TODO: error handling
            .send(BotMessage::BestMove(best_move_so_far.unwrap()))
            .unwrap();
    }
}
