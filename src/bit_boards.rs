#[macro_export]
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        use $crate::bit_boards::my_macros::AlignedAs;

        static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}

pub mod my_macros {
    #[repr(C)]
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        pub _align: [Align; 0],
        pub bytes: Bytes,
    }
}

use crate::const_move_gen::*;
use core::arch::x86_64::_pext_u64;
use std::arch::x86_64::_mm_tzcnt_64;

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_7: u64 = 0x00FF000000000000;
pub const RANK_8: u64 = 0xFF00000000000000;

pub const WHITE_KINGSIDE_CASTLING_MASK: u64 = 96;
pub const WHITE_QUEENSIDE_CASTLING_CHECK_MASK: u64 = 12;
pub const WHITE_QUEENSIDE_CASTLING_FREE_SQUARES_MASK: u64 = 14;

pub const BLACK_KINGSIDE_CASTLING_MASK: u64 = 6917529027641081856;
pub const BLACK_QUEENSIDE_CASTLING_CHECK_MASK: u64 = 864691128455135232;
pub const BLACK_QUEENSIDE_CASTLING_FREE_SQUARES_MASK: u64 = 1008806316530991104;

pub const WHITE_KINGSIDE_CASTLE_INDEX: usize = 6;
pub const WHITE_QUEENSIDE_CASTLE_INDEX: usize = 2;

pub const BLACK_KINGSIDE_CASTLE_INDEX: usize = 62;
pub const BLACK_QUEENSIDE_CASTLE_INDEX: usize = 58;

pub static WHITE_FREE_PAWN_ADVANCE_LOOKUP: [u64; 64] = gen_free_white_pawn_advances();
pub static BLACK_FREE_PAWN_ADVANCE_LOOKUP: [u64; 64] = gen_free_black_pawn_advances();
pub static WHITE_FREE_PAWN_ATTACKS_LOOKUP: [u64; 64] = gen_free_white_pawn_attacks();
pub static BLACK_FREE_PAWN_ATTACKS_LOOKUP: [u64; 64] = gen_free_black_pawn_attacks();
pub static FREE_KNIGHT_LOOKUP: [u64; 64] = gen_free_kight_moves();
pub static FREE_BISHOP_LOOKUP: [u64; 64] = gen_free_bishop_moves();
pub static BISHOP_SQUARE_TO_SQUARE_RAY_LOOKUP: [u64; 64 * 64] = gen_bishop_square_to_square_ray();
pub static FREE_ROOK_LOOKUP: [u64; 64] = gen_free_rook_moves();
pub static ROOK_SQUARE_TO_SQUARE_RAY_LOOKUP: [u64; 64 * 64] = gen_rook_square_to_square_ray();
pub static FREE_KING_LOOKUP: [u64; 65] = gen_free_king_moves();

pub static HORIZONTALS_LOOKUP: [u64; 64] = gen_horizontal_rays();
pub static VERTICALSS_LOOKUP: [u64; 64] = gen_vertical_rays();

pub static NORTH_EAST_LOOKUP: [u64; 64] = gen_north_east_rays();
pub static NORTH_WEST_LOOKUP: [u64; 64] = gen_north_west_rays();

pub static BISHOP_LOOKUP_MASK: [u64; 64] = gen_free_bishop_mask_edges_removed();
static BISHOP_LOOKUP: &[u8] = include_bytes_align_as!(u64, "../assets/bishop_lookup.bin");

pub static ROOK_LOOKUP_MASK: [u64; 64] = gen_free_rook_mask_edges_removed();
static ROOK_LOOKUP: &[u8] = include_bytes_align_as!(u64, "../assets/rook_lookup.bin");

// TODO: rename or multiply by 64
// If rook is in corner it is 2¹², if at edge 2¹¹ otherwise 2¹⁰
// We get a lookup a bit too large, but that is ok
pub const ROOK_LOOK_UP_SIZE: u64 = 1u64 << 12;
pub const BISHOP_LOOK_UP_SIZE: u64 = 1u64 << 9;

#[inline(always)]
pub const fn square_index_to_bitboard(index: usize) -> u64 {
    1 << index
}

//TODO: measure different methods
#[inline(always)]
pub fn bitboard_to_square_index(bitboard: u64) -> usize {
    unsafe { _mm_tzcnt_64(bitboard) as usize }
}

#[inline(always)]
pub const fn square_index_to_square(index: usize) -> Square {
    let file = (index % 8) as u64;
    let rank = (index / 8) as u64;
    Square { rank, file }
}

//TODO: if used outside of const fns use binary repersentation instead
pub struct Square {
    pub file: u64,
    pub rank: u64,
}

impl Square {
    pub const fn get_whole_rank(&self) -> u64 {
        match self.rank {
            0 => RANK_1,
            1 => RANK_2,
            2 => RANK_3,
            3 => RANK_4,
            4 => RANK_5,
            5 => RANK_6,
            6 => RANK_7,
            7 => RANK_8,
            _ => panic!("Invalid rank"),
        }
    }

    pub const fn get_whole_file(&self) -> u64 {
        match self.file {
            0 => FILE_A,
            1 => FILE_B,
            2 => FILE_C,
            3 => FILE_D,
            4 => FILE_E,
            5 => FILE_F,
            6 => FILE_G,
            7 => FILE_H,
            _ => panic!("Invalid file"),
        }
    }

    #[inline(always)]
    pub const fn to_bit_board(&self) -> u64 {
        1 << (self.rank * 8 + self.file)
    }
}

pub fn pop_lsb(bit_board: &mut u64) -> usize {
    let trailing = bit_board.trailing_zeros();
    *bit_board ^= 1 << trailing;
    trailing as usize
}

pub fn get_rook_moves(square: u32, blockers: u64) -> u64 {
    let rook_moves = ROOK_LOOKUP_MASK[square as usize];
    let lookup_index = unsafe { _pext_u64(blockers, rook_moves) };
    
    unsafe {
        let ptr: *const u64 = ROOK_LOOKUP
            .as_ptr()
            .add(((square as u64) * 8u64 * ROOK_LOOK_UP_SIZE + lookup_index * 8u64) as usize)
            as *const u64;
        *ptr
    }
}

pub fn get_bishop_moves(square: usize, blockers: u64) -> u64 {
    let bishop_moves = BISHOP_LOOKUP_MASK[square];
    let lookup_index = unsafe { _pext_u64(blockers, bishop_moves) };
    
    unsafe {
        let ptr: *const u64 = BISHOP_LOOKUP
            .as_ptr()
            .add(((square as u64) * 8u64 * BISHOP_LOOK_UP_SIZE + lookup_index * 8u64) as usize)
            as *const u64;
        *ptr
    }
}
