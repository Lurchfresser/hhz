use std::mem;
use std::ops::Index;
use std::process::Output;

pub const TT_SIZE: u64 = 1 << 20;
pub const TT_INDEX_MASK: u64 = (TT_SIZE << 1) - 1;

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
}

// //TODO: make smaller, current compiled byte size 16, needed 4
// pub struct TT_Hit {
//     pub eval: i16,
//     pub depth: u8,
//     pub node_type: NodeType,
//     // pub was_hit: bool,
// }

pub struct TT_Table {
    tt_table: [TT_Entry; TT_SIZE as usize],
}

impl  TT_Table {


    //TODO: find more performant output
    #[inline(always)]
    fn index(&self, outside_zobrist: u64) -> Option<TT_Entry> {
        let i = outside_zobrist & TT_INDEX_MASK;
        let tt_entry = self.tt_table[i as usize];

        //decide if it is a hash hit, because we know by indexing, the last part (the part masked to i) is also the same
        if (tt_entry.zobrist_hash_part() & !TT_Entry::TT_INFO_MASK) == i {
            Some(tt_entry)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_tt_entry_encoding_decoding_randomized() {
        let mut rng = rand::rng();
        for _ in 0..1000 {
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
                zobrist_hash & !TT_INFO_MASK,
                "zobrist_hash_part mismatch"
            );
        }
    }
}