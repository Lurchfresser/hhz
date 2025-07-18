use crate::moves::Move;
use std::mem;

pub const TT_SIZE: usize = 1 << 20;
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u64)]
pub enum NodeType {
    PvNode = 0,  // Exact score
    CutNode = 1, // Lower bound (fail-high)
    AllNode = 2, // Upper bound (fail-low)
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct TT_Entry {
    // 64 bits for the full Zobrist hash for collision-free lookups.
    zobrist_hash: u64,
    // 64 bits for all associated data.
    data: u64,
}

impl TT_Entry {
    // --- Bit-packing layout for the 64-bit `data` field ---

    // [Bits 0-15]  16 bits for evaluation score (i16)
    const EVAL_MASK: u64 = (1 << 16) - 1;

    // [Bits 16-31] 16 bits to store the Move's raw mask directly
    const MOVE_SHIFT: u64 = 16;
    const MOVE_MASK: u64 = ((1 << 16) - 1) << Self::MOVE_SHIFT;

    // [Bits 32-39] 8 bits for the search depth
    const DEPTH_SHIFT: u64 = Self::MOVE_SHIFT + 16;
    const DEPTH_MASK: u64 = ((1 << 8) - 1) << Self::DEPTH_SHIFT;

    // [Bits 40-46] 7 bits for the halfmove clock
    const HALFMOVE_SHIFT: u64 = Self::DEPTH_SHIFT + 8;
    const HALFMOVE_MASK: u64 = ((1 << 7) - 1) << Self::HALFMOVE_SHIFT;

    // [Bits 47-48] 2 bits for the node type (PV, Cut, All)
    const NODE_TYPE_SHIFT: u64 = Self::HALFMOVE_SHIFT + 7;
    const NODE_TYPE_MASK: u64 = ((1 << 2) - 1) << Self::NODE_TYPE_SHIFT;

    // [Bits 49-55] 7 bits for the number of resetting moves
    const NUM_RESETS_SHIFT: u64 = Self::NODE_TYPE_SHIFT + 2;
    const NUM_RESETS_MASK: u64 = ((1 << 7) - 1) << Self::NUM_RESETS_SHIFT;

    // [Bits 56-63] 8 bits are unused and available for future data.

    /// Creates a new transposition table entry.
    #[inline(always)]
    pub fn new(
        zobrist_hash: u64,
        depth: u8,
        eval: i16,
        node_type: NodeType,
        best_move: Move,
        halfmove_clock: u8,
        num_resetting_moves: u8,
    ) -> Self {
        // We now store the move's raw 16-bit mask.
        let move_mask = best_move.mask as u64;

        let data = (eval as u16 as u64)
            | (move_mask << Self::MOVE_SHIFT)
            | ((depth as u64) << Self::DEPTH_SHIFT)
            | ((halfmove_clock as u64) << Self::HALFMOVE_SHIFT)
            | ((node_type as u64) << Self::NODE_TYPE_SHIFT)
            | ((num_resetting_moves as u64) << Self::NUM_RESETS_SHIFT);

        Self { zobrist_hash, data }
    }

    #[inline(always)]
    pub fn init() -> Self {
        TT_Entry {
            zobrist_hash: 0,
            data: 0,
        }
    }

    // --- Accessor Methods ---

    #[inline(always)]
    pub fn eval(&self) -> i16 {
        (self.data & Self::EVAL_MASK) as i16
    }

    /// Unpacks the stored 16-bit mask back into a `Move`.
    /// Returns `None` if no move was stored (mask is 0).
    #[inline(always)]
    pub fn best_move(&self) -> Option<Move> {
        let move_mask = ((self.data & Self::MOVE_MASK) >> Self::MOVE_SHIFT) as u16;
        if move_mask == 0 {
            None
        } else {
            Some(Move { mask: move_mask })
        }
    }

    #[inline(always)]
    pub fn depth(&self) -> u8 {
        ((self.data & Self::DEPTH_MASK) >> Self::DEPTH_SHIFT) as u8
    }

    #[inline(always)]
    pub fn halfmove_clock(&self) -> u8 {
        ((self.data & Self::HALFMOVE_MASK) >> Self::HALFMOVE_SHIFT) as u8
    }

    #[inline(always)]
    pub fn node_type(&self) -> NodeType {
        let node_type_val = (self.data & Self::NODE_TYPE_MASK) >> Self::NODE_TYPE_SHIFT;
        unsafe { mem::transmute(node_type_val) }
    }

    #[inline(always)]
    pub fn num_resetting_moves(&self) -> u8 {
        ((self.data & Self::NUM_RESETS_MASK) >> Self::NUM_RESETS_SHIFT) as u8
    }
}
#[allow(non_camel_case_types)]
pub struct TT_Table {
    //vec because rust can allocate an array directly on the heap and this would cause a stack overflow
    tt_table: Vec<TT_Entry>,
}

impl TT_Table {
    pub fn new() -> Self {
        TT_Table {
            tt_table: vec![TT_Entry::init(); TT_SIZE],
        }
    }
    //TODO: find more performant output
    #[inline(always)]
    pub fn probe(&self, outside_zobrist: u64) -> Option<&TT_Entry> {
        // This rare edge case will be almost never hit, and when depth = 0 will make it discard the value
        // if outside_zobrist & (TT_INDEX_MASK | !TT_Entry::TT_INFO_MASK) == 0 {
        //     return None
        // }
        let i = (outside_zobrist as usize) % TT_SIZE;
        let maybe_hit = &self.tt_table[i];
        //decide if it is a hash hit, because we know by indexing, the last part (the part masked to i) is also the same
        if maybe_hit.zobrist_hash == outside_zobrist {
            Some(maybe_hit)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn insert(
        &mut self,
        zobrist: u64,
        eval: i16,
        depth: u8,
        node_type: NodeType,
        best_move: Move, // Now takes the Move struct directly
        halfmove_clock: u8,
        //TODO: overflow
        num_resetting_moves: u8,
    ) {
        //TODO: insertion strategy
        let index = zobrist as usize % TT_SIZE;
        let existing_entry = &self.tt_table[index];

        let is_same_position = existing_entry.zobrist_hash == zobrist;

        let new_node_quality = (depth as u16 * 3) + node_type as u16;
        let existing_node_quality =
            (existing_entry.depth() as u16 * 3) + existing_entry.node_type() as u16;

        if !is_same_position || new_node_quality >= existing_node_quality {
            self.tt_table[index] = TT_Entry::new(
                zobrist,
                depth,
                eval,
                node_type,
                best_move,
                halfmove_clock,
                num_resetting_moves,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distr::Distribution;
    use rand::distr::Uniform;
    use rand::{Rng, rng};

    // Helper function to create a new, heap-allocated TT_Table to prevent a stack overflow,
    // which can happen with large arrays like `tt_table`. The table is zero-initialized.
    fn new_test_tt_table() -> TT_Table {
        TT_Table::new()
    }

    #[test]
    fn test_tt_entry_encoding_decoding_randomized() {
        let mut rng = rng();
        for _ in 0..10000 {
            let zobrist_hash: u64 = rng.random();
            let depth: u8 = rng.random();
            let eval: i16 = rng.random();
            let flag: NodeType = match rng.random_range(0..=2) {
                0 => NodeType::PvNode,
                1 => NodeType::CutNode,
                _ => NodeType::AllNode,
            };

            let entry = TT_Entry::new(zobrist_hash, depth, eval, flag);

            assert_eq!(entry.eval(), eval, "eval mismatch");
            assert_eq!(entry.depth(), depth, "depth mismatch");
            assert_eq!(entry.node_type(), flag, "node_type mismatch");
            assert_eq!(
                entry.zobrist_hash_part(),
                zobrist_hash & !TT_Entry::TT_INFO_MASK,
                "zobrist_hash_part mismatch"
            );
        }
    }

    #[test]
    fn test_tt_table_insert_and_index_hit() {
        let mut tt_table = new_test_tt_table();
        let mut rng = rng();
        let depth_range: Uniform<u8> = Uniform::new(1, 120).unwrap();
        let eval_range: Uniform<i16> = Uniform::new(-30000, 30000).unwrap();

        for _ in 0..10 {
            let zobrist_hash: u64 = rng.random();

            let depth: u8 = depth_range.sample(&mut rng);
            let eval: i16 = eval_range.sample(&mut rng);
            let flag: NodeType = match rng.random_range(0..=2) {
                0 => NodeType::PvNode,
                1 => NodeType::CutNode,
                _ => NodeType::AllNode,
            };

            tt_table.insert(zobrist_hash, eval, depth, flag);
            let result = tt_table.probe(zobrist_hash);

            assert!(
                result.is_some(),
                "A hit was expected, but index() returned None."
            );

            if let Some(entry) = result {
                assert_eq!(entry.eval(), eval);
                assert_eq!(entry.depth(), depth);
                assert_eq!(entry.node_type(), flag);
            }
        }
    }

    #[test]
    fn test_tt_table_index_miss_for_non_existent_entry() {
        let tt_table = new_test_tt_table();
        let mut rng = rng();

        for _ in 0..10000 {
            let zobrist_hash: u64 = rng.random();

            let result = tt_table.probe(zobrist_hash);
            assert!(
                result.is_none(),
                "Expected a miss for a non-existent entry, but found one."
            );
        }
    }
}
