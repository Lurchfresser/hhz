use std::mem;
use std::ops::Index;
use std::process::Output;

pub const TT_SIZE: usize = 1 << 20;
pub const TT_INDEX_MASK: u64 = (TT_SIZE as u64) - 1;

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u64)]
pub enum NodeType {
    PvNode = 0,  // Exact score
    CutNode = 1, // Lower bound (fail-high)
    AllNode = 2, // Upper bound (fail-low)
}

#[derive(Clone, Copy)]
pub struct TT_Entry {
    // layout:
    // 0 - 15: eval: 16 bits
    // 16 - 23: depth: 8 bits
    // 24 - 25: node type: 2 bits
    // 26 - 63: remaining zobrist hash: 38 bits
    mask: u64,
}

impl TT_Entry {
    const TT_EVAL_MASK: u64 = (1 << 16) - 1;
    const TT_DEPTH_SHIFT: u64 = 16;
    const TT_DEPTH_MASK: u64 = ((1 << 8) - 1) << Self::TT_DEPTH_SHIFT;
    const TT_NODE_TYPE_SHIFT: u64 = Self::TT_DEPTH_SHIFT + 8;
    const TT_NODE_TYPE_MASK: u64 = ((1 << 2) - 1) << Self::TT_NODE_TYPE_SHIFT;
    const TT_ZOBRIST_SHIFT: u64 = Self::TT_NODE_TYPE_SHIFT + 2;
    pub const TT_INFO_MASK: u64 = (1 << Self::TT_ZOBRIST_SHIFT) - 1;

    #[inline(always)]
    pub fn init() -> Self {
        Self { mask: 0 }
    }

    #[inline(always)]
    pub fn new(zobrist_hash: u64, depth: u8, eval: i16, flag: NodeType) -> Self {
        let mask = (zobrist_hash & !Self::TT_INFO_MASK)
            | (eval as u16 as u64)
            | ((depth as u64) << Self::TT_DEPTH_SHIFT)
            | ((flag as u64) << Self::TT_NODE_TYPE_SHIFT);
        Self { mask }
    }

    #[inline(always)]
    pub fn eval(&self) -> i16 {
        (self.mask & Self::TT_EVAL_MASK) as i16
    }

    #[inline(always)]
    pub fn depth(&self) -> u8 {
        ((self.mask & Self::TT_DEPTH_MASK) >> Self::TT_DEPTH_SHIFT) as u8
    }

    #[inline(always)]
    pub fn node_type(&self) -> NodeType {
        let node_type_val = (self.mask & Self::TT_NODE_TYPE_MASK) >> Self::TT_NODE_TYPE_SHIFT;
        unsafe { mem::transmute(node_type_val) }
    }

    #[inline(always)]
    pub fn zobrist_hash_part(&self) -> u64 {
        self.mask & !Self::TT_INFO_MASK
    }

    #[inline(always)]
    pub fn is_init(&self) -> bool {
        self.mask == 0
    }
}

// //TODO: make smaller, current compiled byte size 16, needed 4
// pub struct TT_Hit {
//     pub eval: i16,
//     pub depth: u8,
//     pub node_type: NodeType,
//     // pub was_hit: bool,
// }

pub struct TT_Table {
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
    fn probe(&self, outside_zobrist: u64) -> Option<&TT_Entry> {
        // This rare edge case will be almost never hit, and when depth = 0 will make it discard the value
        // if outside_zobrist & (TT_INDEX_MASK | !TT_Entry::TT_INFO_MASK) == 0 {
        //     return None
        // }
        let i = outside_zobrist & TT_INDEX_MASK;
        //decide if it is a hash hit, because we know by indexing, the last part (the part masked to i) is also the same
        if self.tt_table[i as usize].zobrist_hash_part() == outside_zobrist & !TT_Entry::TT_INFO_MASK {
            Some(&self.tt_table[i as usize])
        } else {
            None
        }
    }

    #[inline(always)]
    fn insert(&mut self, zobrist: u64, eval: i16, depth: u8, node_type: NodeType) {
        //TODO: insertion strategy
        self.tt_table[(zobrist & TT_INDEX_MASK) as usize] =
            TT_Entry::new(zobrist, depth, eval, node_type);
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

            assert!(result.is_some(), "A hit was expected, but index() returned None.");

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
