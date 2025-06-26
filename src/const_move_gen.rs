use crate::bit_boards::{
    FILE_A, FILE_H, RANK_1, RANK_2, RANK_7, RANK_8, Square, square_index_to_bitboard,
    square_index_to_square,
};

pub const fn gen_free_white_pawn_attacks() -> [u64; 64] {
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

pub const fn gen_free_black_pawn_attacks() -> [u64; 64] {
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

pub const fn gen_free_white_pawn_advances() -> [u64; 64] {
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

pub const fn gen_free_black_pawn_advances() -> [u64; 64] {
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

pub const fn gen_free_kight_moves() -> [u64; 64] {
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

pub const fn gen_free_bishop_moves() -> [u64; 64] {
    let mut free_bishop_lookup = [0u64; 64];

    // --- Part 1: Generate all diagonal masks into local arrays ---

    // There are 15 main-diagonals (a1-h8 style) and 15 anti-diagonals (a8-h1 style).
    let mut main_diagonals = [0u64; 15];
    let mut anti_diagonals = [0u64; 15];

    let mut i: usize = 0;
    while i < 64 {
        let rank = (i / 8) as i8;
        let file = (i % 8) as i8;
        let bit = 1u64 << i;

        // For main-diagonals, the difference (rank - file) is constant.
        // It ranges from -7 (h1) to +7 (a8). We add 7 to map this to an index [0, 14].
        let main_idx = (rank - file + 7) as usize;
        main_diagonals[main_idx] |= bit;

        // For anti-diagonals, the sum (rank + file) is constant.
        // It ranges from 0 (a1) to 14 (h8), which is already a valid index range [0, 14].
        let anti_idx = (rank + file) as usize;
        anti_diagonals[anti_idx] |= bit;

        i += 1;
    }

    // --- Part 2: Use the generated masks to build the final lookup table ---

    let mut s: usize = 0;
    while s < 64 {
        let rank = (s / 8) as i8;
        let file = (s % 8) as i8;

        // Find the index for the current square's main-diagonal.
        let main_diagonal_idx = (rank - file + 7) as usize;
        let main_diagonal_mask = main_diagonals[main_diagonal_idx];

        // Find the index for the current square's anti-diagonal.
        let anti_diagonal_idx = (rank + file) as usize;
        let anti_diagonal_mask = anti_diagonals[anti_diagonal_idx];

        // The moves for a bishop are the combination of its two diagonals.
        // Using XOR (^) combines the bitmasks and cleverly removes the starting
        // square itself, since it's the only square present in both masks.
        free_bishop_lookup[s] = main_diagonal_mask ^ anti_diagonal_mask;

        s += 1;
    }

    free_bishop_lookup
}

pub const fn gen_free_bishop_mask_edges_removed() -> [u64; 64] {
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

/// Generates a bitmask for the ray between two squares for a bishop.
/// The mask contains only the squares *between* the from and to squares.
pub const fn gen_bishop_square_to_square_ray() -> [u64; 64 * 64] {
    let mut bishop_square_to_square_ray_lookup = [0u64; 64 * 64];
    let mut from_index = 0;
    while from_index < 64 {
        let mut to_index = 0;
        while to_index < 64 {
            // No ray if it's the same square
            if from_index == to_index {
                to_index += 1;
                continue;
            }
            let from_square = square_index_to_square(from_index);
            let to_square = square_index_to_square(to_index);

            let mut bishop_square_to_square_ray = 0u64;

            let mut rank_diff = (to_square.rank as i16) - (from_square.rank as i16);
            let mut file_diff = (to_square.file as i16) - (from_square.file as i16);

            // Check if they are on the same diagonal
            if rank_diff.abs() == file_diff.abs() {
                // Loop while the distance is greater than 1 to get only squares between
                while rank_diff.abs() > 1 {
                    // Move one step closer to the from_square
                    if rank_diff > 0 {
                        rank_diff -= 1;
                    } else {
                        rank_diff += 1;
                    }
                    if file_diff > 0 {
                        file_diff -= 1;
                    } else {
                        file_diff += 1;
                    }

                    let new_rank = (from_square.rank as i16) + rank_diff;
                    let new_file = (from_square.file as i16) + file_diff;

                    bishop_square_to_square_ray |= Square {
                        rank: new_rank as u64,
                        file: new_file as u64,
                    }
                    .to_bit_board();
                }
            }

            bishop_square_to_square_ray_lookup[from_index * 64 + to_index] =
                bishop_square_to_square_ray;

            to_index += 1;
        }
        from_index += 1;
    }
    bishop_square_to_square_ray_lookup
}

pub const fn gen_free_rook_moves() -> [u64; 64] {
    let mut free_rook_lookup = [0u64; 64];
    let mut s: usize = 0;
    while s < 64 {
        let square = square_index_to_square(s);
        let mut bit_board: u64 = 0;

        bit_board ^= square.get_whole_file();
        bit_board ^= square.get_whole_rank();

        free_rook_lookup[s as usize] = bit_board;
        s += 1;
    }
    free_rook_lookup
}

pub const fn gen_free_rook_mask_edges_removed() -> [u64; 64] {
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
        s += 1;
    }
    rook_free_board_lookup
}

pub const fn gen_rook_square_to_square_ray() -> [u64; 64 * 64] {
    let mut rook_square_to_square_ray_lookup = [0u64; 64 * 64];
    let mut from_index = 0;
    while from_index < 64 {
        let mut to_index = 0;
        while to_index < 64 {
            if from_index == to_index {
                to_index += 1;
                continue;
            }
            let from_square = square_index_to_square(from_index);
            let to_square = square_index_to_square(to_index);

            let mut rook_square_to_square_ray = 0;

            if from_square.rank == to_square.rank {
                let mut horizontal_distance = (to_square.file as i16) - (from_square.file as i16);
                while horizontal_distance.abs() != 0 {
                    let new_file = (from_square.file as i16) + horizontal_distance;
                    rook_square_to_square_ray |= Square {
                        rank: from_square.rank,
                        file: new_file as u64,
                    }
                    .to_bit_board();
                    if horizontal_distance > 0 {
                        horizontal_distance -= 1
                    } else {
                        horizontal_distance += 1
                    }
                }
                rook_square_to_square_ray &= !to_square.to_bit_board();
            }

            if from_square.file == to_square.file {
                let mut vertical_distance = (to_square.rank as i16) - (from_square.rank as i16);
                while vertical_distance.abs() != 0 {
                    let new_rank = (from_square.rank as i16) + vertical_distance;
                    rook_square_to_square_ray |= Square {
                        file: from_square.file,
                        rank: new_rank as u64,
                    }
                    .to_bit_board();
                    if vertical_distance > 0 {
                        vertical_distance -= 1
                    } else {
                        vertical_distance += 1
                    }
                }
                rook_square_to_square_ray &= !to_square.to_bit_board();
            }

            rook_square_to_square_ray_lookup[from_index * 64 + to_index] =
                rook_square_to_square_ray;

            to_index += 1
        }
        from_index += 1;
    }
    rook_square_to_square_ray_lookup
}

//Size 65 for king free positions
pub const fn gen_free_king_moves() -> [u64; 65] {
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
