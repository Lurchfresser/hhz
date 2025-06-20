use core::arch::x86_64;
use std::arch::x86_64::_mm_tzcnt_64;

pub static RANK_A: u64 = 0x00000000000000FF;
pub static RANK_B: u64 = 0x000000000000FF00;
pub static RANK_C: u64 = 0x0000000000FF0000;
pub static RANK_D: u64 = 0x00000000FF000000;
pub static RANK_E: u64 = 0x000000FF00000000;
pub static RANK_F: u64 = 0x0000FF0000000000;
pub static RANK_G: u64 = 0x00FF000000000000;
pub static RANK_H: u64 = 0xFF00000000000000;

pub static FILE_1: u64 = 0x0101010101010101;
pub static FILE_2: u64 = 0x0202020202020202;
pub static FILE_3: u64 = 0x0404040404040404;
pub static FILE_4: u64 = 0x0808080808080808;
pub static FILE_5: u64 = 0x1010101010101010;
pub static FILE_6: u64 = 0x2020202020202020;
pub static FILE_7: u64 = 0x4040404040404040;
pub static FILE_8: u64 = 0x8080808080808080;

#[inline(always)]
pub fn square_index_to_bitboard(index: u64) -> u64 {
    1 << index
}


//TODO: measure different methods
#[inline(always)]
pub fn bitboard_to_square_index(bitboard: u64) -> i64 {
    unsafe { _mm_tzcnt_64(bitboard) }
}

#[inline(always)]
pub fn square_index_to_square(index: u64) -> Square {
    let file = (index % 8);
    let rank = (index / 8);
    Square { file, rank }
}

pub struct Square {
    pub file: u64,
    pub rank: u64,
}

impl Square {
    pub fn get_whole_rank(&self) -> u64 {
        match self.rank {
            0 => RANK_A,
            1 => RANK_B,
            2 => RANK_C,
            3 => RANK_D,
            4 => RANK_E,
            5 => RANK_F,
            6 => RANK_G,
            7 => RANK_H,
            _ => panic!("Invalid rank"),
        }
    }
    
    pub fn get_whole_file(&self) -> u64 {
        match self.file {
            0 => FILE_1,
            1 => FILE_2,
            2 => FILE_3,
            3 => FILE_4,
            4 => FILE_5,
            5 => FILE_6,
            6 => FILE_7,
            7 => FILE_8,
            _ => panic!("Invalid file"),
        }
    }

    #[inline(always)]
    pub fn to_bit_board(&self) -> u64 {
        1 << (self.rank * 8 + self.file)
    }
}
