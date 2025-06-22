use core::arch::x86_64::_pext_u64;
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

pub static ROOK_LOOKUP_MASK: [u64; 64] = gen_free_rook_mask();
pub static KNIGHT_LOOKUP: [u64; 64] = gen_knight_lookup();
pub static KING_LOOKUP: [u64; 65] = gen_king_moves();
static ROOK_LOOKUP: &[u8] = include_bytes!("../assets/rook_lookup.bin");

// TODO: rename or multiply by 64
// If rook is in corner it is 2¹², if at edge 2¹¹ otherwise 2¹⁰
// We get a lookup a bit too large, but that is ok
pub const ROOK_LOOK_UP_SIZE: u64 = 1u64 << 12;

#[inline(always)]
pub const fn square_index_to_bitboard(index: u64) -> u64 {
    1 << index
}

//TODO: measure different methods
#[inline(always)]
pub fn bitboard_to_square_index(bitboard: u64) -> i64 {
    unsafe { _mm_tzcnt_64(bitboard) }
}

#[inline(always)]
pub const fn square_index_to_square(index: u64) -> Square {
    let file = index % 8;
    let rank = index / 8;
    Square { file, rank }
}

pub struct Square {
    pub file: u64,
    pub rank: u64,
}

impl Square {
    pub const fn get_whole_rank(&self) -> u64 {
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

    pub const fn get_whole_file(&self) -> u64 {
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

pub fn pop_lsb(bit_board: &mut u64) -> u32 {
    let trailing = bit_board.trailing_zeros();
    *bit_board ^= 1 << trailing;
    trailing
}

pub fn get_rook_moves(square: u32, blockers: u64) -> u64 {
    let rook_moves = ROOK_LOOKUP_MASK[square as usize];
    let lookup_index = unsafe { _pext_u64(blockers, rook_moves) };
    let test = unsafe {
        let ptr: *const u64 = ROOK_LOOKUP
            .as_ptr()
            .add(((square as u64) * 8u64 * ROOK_LOOK_UP_SIZE + lookup_index * 8u64) as usize)
            as *const u64;
        *ptr
    };
    println!("{}", test);
    test
}

//Size 65 for king free positions
pub const fn gen_king_moves() -> [u64; 65] {
    let mut king_moves = [0u64; 65];
    let mut i = 0;
    while i < 64 {
        let square = square_index_to_square(i);
        let mut attacks = 0u64;

        // North (up)
        if square.rank > 0 {
            attacks |= 1 << (i - 8);

            // Northwest (up-left)
            if square.file > 0 {
                attacks |= 1 << (i - 9);
            }

            // Northeast (up-right)
            if square.file < 7 {
                attacks |= 1 << (i - 7);
            }
        }

        // South (down)
        if square.rank < 7 {
            attacks |= 1 << (i + 8);

            // Southwest (down-left)
            if square.file > 0 {
                attacks |= 1 << (i + 7);
            }

            // Southeast (down-right)
            if square.file < 7 {
                attacks |= 1 << (i + 9);
            }
        }

        // West (left)
        if square.file > 0 {
            attacks |= 1 << (i - 1);
        }

        // East (right)
        if square.file < 7 {
            attacks |= 1 << (i + 1);
        }

        king_moves[i as usize] = attacks;
        i += 1;
    }
    king_moves
}

const fn gen_free_rook_mask() -> [u64; 64] {
    let mut rook_free_board_lookup = [0u64; 64];
    let mut s = 0;
    while s < 64 {
        let square = square_index_to_square(s);
        let mut bit_board: u64 = 0;

        bit_board |= square.get_whole_file();
        bit_board |= square.get_whole_rank();

        bit_board ^= square_index_to_bitboard(s);

        if square.rank != 0 {
            bit_board &= !RANK_A;
        }
        if square.rank != 7 {
            bit_board &= !RANK_H;
        }
        if square.file != 0 {
            bit_board &= !FILE_1;
        }
        if square.file != 7 {
            bit_board &= !FILE_8
        }
        rook_free_board_lookup[s as usize] = bit_board;
        // println!(
        //     "square {} and mask {}",
        //     square_index_to_bitboard(s),
        //     bit_board
        // );
        s += 1;
    }
    rook_free_board_lookup
}

const fn gen_knight_lookup() -> [u64; 64] {
    let mut knight_lookup = [0u64; 64];
    let mut i = 0;
    while i < 64 {
        let mut moves: u64 = 0;

        // links nach rechts
        let file = i % 8;
        // von unten nach oben
        let rank = i / 8;

        // north
        if rank <= 5 {
            // north-west
            if file >= 1 {
                moves |= 1 << i + 15; // up-right
            }
            if file <= 6 {
                moves |= 1 << i + 17; // up-left
            }
        }
        // south
        if rank >= 2 {
            // south-west
            if file >= 1 {
                moves |= 1 << i - 17; // down-right
            }
            if file <= 6 {
                moves |= 1 << i - 15; // down-left
            }
        }
        // west
        if file >= 2 {
            if rank >= 1 {
                moves |= 1 << i - 10; // up-left
            }
            if rank <= 6 {
                moves |= 1 << i + 6; // down-left
            }
        }
        // east
        if file <= 5 {
            if rank >= 1 {
                moves |= 1 << i - 6; // up-right
            }
            if rank <= 6 {
                moves |= 1 << i + 10; // down-right
            }
        }
        knight_lookup[i as usize] = moves;
        i += 1;
    }
    knight_lookup
}
