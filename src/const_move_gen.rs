use crate::bit_boards::{
    FILE_A, FILE_H, RANK_1, RANK_2, RANK_7, RANK_8, Square, square_index_to_bitboard,
    square_index_to_square,
};

pub const fn gen_white_pawn_attacks() -> [u64; 64] {
    let mut pawn_attacks = [0u64; 64];
    let mut i: u64 = 0;
    while i < (64 - 8) {
        let mut attacks = 0u64;
        let square = square_index_to_square(i as usize);

        if square.file != 0 {
            attacks |= square.to_bit_board() << 7;
        }
        if square.file != 7 {
            attacks |= square.to_bit_board() << 9;
        }
        pawn_attacks[i as usize] = attacks;
        i += 1;
    }
    pawn_attacks
}

pub const fn gen_black_pawn_attacks() -> [u64; 64] {
    let mut pawn_attacks = [0u64; 64];
    let mut i = 8u64;
    while i < 64 {
        let mut attacks = 0u64;
        let square = square_index_to_square(i as usize);

        if square.file != 7 {
            attacks |= square.to_bit_board() >> 7;
        }
        if square.file != 0 {
            attacks |= square.to_bit_board() >> 9;
        }
        pawn_attacks[i as usize] = attacks;
        i += 1;
    }
    pawn_attacks
}

pub const fn gen_white_pawn_advances() -> [u64; 64] {
    let mut pawn_advances = [0u64; 64];
    let mut i = 0u64;
    while i < 64 {
        let mut advance = square_index_to_bitboard(i as usize) << 8;
        if square_index_to_bitboard(i as usize) & RANK_2 != 0 {
            advance |= square_index_to_bitboard(i as usize) << 16;
        }
        pawn_advances[i as usize] = advance;
        i += 1;
    }
    pawn_advances
}

pub const fn gen_black_pawn_advances() -> [u64; 64] {
    let mut pawn_advances = [0u64; 64];
    let mut i = 0u64;
    while i < 64 {
        let mut advance = square_index_to_bitboard(i as usize) >> 8;
        if square_index_to_bitboard(i as usize) & RANK_7 != 0 {
            advance |= square_index_to_bitboard(i as usize) >> 16;
        }
        pawn_advances[i as usize] = advance;
        i += 1;
    }
    pawn_advances
}

pub const fn gen_knight_lookup() -> [u64; 64] {
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

//TODO: const fn
pub const fn gen_free_bishop_mask() -> [u64; 64] {
    let mut free_bishop_lookup = [0u64; 64];
    let mut s = 0;
    while s < 64 {
        let square = square_index_to_square(s);
        let mut move_bit_board = 0u64;

        // North-East direction
        let mut north_east_index = 1;
        loop {
            let next_square = Square {
                file: square.file + north_east_index,
                rank: square.rank + north_east_index,
            };
            if next_square.file > 6 || next_square.rank > 6 {
                break;
            }
            move_bit_board |= next_square.to_bit_board();
            north_east_index += 1;
        }

        // North-West direction
        let mut north_west_index = 1;
        loop {
            if square.file < north_west_index {
                break;
            }
            let next_square = Square {
                file: square.file - north_west_index,
                rank: square.rank + north_west_index,
            };
            if next_square.file < 1 || next_square.rank > 6 {
                break;
            }
            move_bit_board |= next_square.to_bit_board();
            north_west_index += 1;
        }

        // South-East direction
        let mut south_east_index = 1;
        loop {
            if square.rank < south_east_index {
                break;
            }
            let next_square = Square {
                file: square.file + south_east_index,
                rank: square.rank - south_east_index,
            };
            if next_square.file > 6 || next_square.rank < 1 {
                break;
            }
            move_bit_board |= next_square.to_bit_board();
            south_east_index += 1;
        }

        // South-West direction
        let mut south_west_index = 1;
        loop {
            if square.file < south_west_index || square.rank < south_west_index {
                break;
            }
            let next_square = Square {
                file: square.file - south_west_index,
                rank: square.rank - south_west_index,
            };
            if next_square.file < 1 || next_square.rank < 1 {
                break;
            }
            move_bit_board |= next_square.to_bit_board();
            south_west_index += 1;
        }
        free_bishop_lookup[s] = move_bit_board;
        s += 1;
    }
    free_bishop_lookup
}

pub const fn gen_free_rook_mask() -> [u64; 64] {
    let mut rook_free_board_lookup = [0u64; 64];
    let mut s: usize = 0;
    while s < 64 {
        let square = square_index_to_square(s);
        let mut bit_board: u64 = 0;

        bit_board |= square.get_whole_file();
        bit_board |= square.get_whole_rank();

        bit_board ^= square_index_to_bitboard(s);

        if square.rank != 0 {
            bit_board &= !RANK_1;
        }
        if square.rank != 7 {
            bit_board &= !RANK_8;
        }
        if square.file != 0 {
            bit_board &= !FILE_A;
        }
        if square.file != 7 {
            bit_board &= !FILE_H
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

//Size 65 for king free positions
pub const fn gen_king_moves() -> [u64; 65] {
    let mut king_moves = [0u64; 65];
    let mut i = 0;
    while i < 64 {
        let square = square_index_to_square(i);
        let mut attacks = 0u64;

        // North (up)
        if square.rank > 0 {
            attacks |= 1u64 << (i - 8);

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
