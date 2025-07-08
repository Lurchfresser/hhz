use crate::board::Board;

// Add this enum somewhere before main()
enum BotCommand {
    Position {
        startpos: bool,
        fen: Option<String>,
        moves: Vec<vampirc_uci::UciMove>,
    },
    Go,
    Quit,
    // You can add more later, like Stop, SetOption, etc.
}

pub struct Bot {
    board: Board,
    tt_table: Vec<u64>,
}

impl Bot {
    pub fn new(board: Board, tt_size_mb: usize) -> Self {
        let tt_size_bytes = tt_size_mb * 1024 * 1024;
        let tt_entries = tt_size_bytes / std::mem::size_of::<u64>();
        let tt_table = vec![0; tt_entries];

        Self { board, tt_table }
    }

    pub fn make_uci_move_on_board(&mut self, uci_move: &str) {
        self.board = self.board.make_uci_move_temp(uci_move)
    }

    pub fn calculate_best_move() {}
}
