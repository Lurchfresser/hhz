use core::arch::x86_64::{_pdep_u64, _pext_u64};
use std::collections::HashSet;
use hhz::bit_boards::*;

fn main() {
    if cfg!(target_arch = "x86_64") {
        println!("Running on x86_64 architecture");
    } else {
        panic!("This code is designed to run on x86_64 architecture only.");
    }

    let mut knight_lookup: Vec<u64> = vec![0; 64];
    for i in 0u64..64u64 {
        let mut moves: u64 = 0;

        // links nach rechts
        let file = i % 8;
        // von unten nach oben
        let rank = i / 8;

        // north
        if rank <= 5 {
            // north-west
            if file >= 1 {
                moves |= (1 << i + 15); // up-right
            }
            if file <= 6 {
                moves |= (1 << i + 17); // up-left
            }
        }
        // south
        if rank >= 2 {
            // south-west
            if file >= 1 {
                moves |= (1 << i - 17); // down-right
            }
            if file <= 6 {
                moves |= (1 << i - 15); // down-left
            }
        }
        // west
        if file >= 2 {
            if rank >= 2 {
                moves |= (1 << i - 10); // up-left
            }
            if rank <= 6 {
                moves |= (1 << i + 6); // down-left
            }
        }
        // east
        if file <= 5 {
            if rank >= 2 {
                moves |= (1 << i - 6); // up-right
            }
            if rank <= 6 {
                moves |= (1 << i + 10); // down-right
            }
        }
        //  println!("i: {}, debug hits: {}", i, debug_hits);
        knight_lookup[i as usize] = moves;
        // println!(
        //     "square {} and bit mask: {}",
        //     1u64 << i,
        //     knight_lookup[i as usize]
        // );
    }
    let mut rook_free_board_lookup: Vec<u64> = vec![0; 64];

    for s in 0u64..64u64 {
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
    }

    println!("---------------");

    let mut rook_lookup = vec![0, (1 << 12) * 64];
    unsafe {
        'outer: for index in 0u64..(1u64 << 12) {
            let rook_square = square_index_to_square(27);
            let rook_sqaure_bit_board = rook_square.to_bit_board();
            let blockers: u64 = _pdep_u64(index, rook_free_board_lookup[27 as usize]);
            let mut move_bit_mask = 0;

            if rook_square.rank > 1 {
                'inner: for i in (0..rook_square.rank).rev() {
                    let square_2 = Square {
                        rank: i,
                        file: rook_square.file,
                    };
                    let square_bit_board = square_2.to_bit_board();
                    move_bit_mask |= square_bit_board;
                    if blockers & square_bit_board != 0 {
                        break 'inner;
                    }
                }
            }
            if rook_square.rank < 5 {
                'inner: for i in (rook_square.rank + 1 .. 8) {
                    let square_2 = Square {
                        rank: i,
                        file: rook_square.file,
                    };
                    let square_bit_board = square_2.to_bit_board();
                    move_bit_mask |= square_bit_board;
                    if blockers & square_bit_board != 0 {
                        break 'inner;
                    }
                }
            }

            if rook_square.file > 1 {
                'inner: for i in (0..rook_square.file).rev() {
                    let square_2 = Square {
                        file: i,
                        rank: rook_square.rank,
                    };
                    let square_bit_board = square_2.to_bit_board();
                    move_bit_mask |= square_bit_board;
                    if blockers & square_bit_board != 0 {
                        break 'inner;
                    }
                }
            }
            if rook_square.file < 5 {
                'inner: for i in (rook_square.file + 1 .. 8) {
                    let square_2 = Square {
                        file: i,
                        rank: rook_square.rank,
                    };
                    let square_bit_board = square_2.to_bit_board();
                    move_bit_mask |= square_bit_board;
                    if blockers & square_bit_board != 0 {
                        break 'inner;
                    }
                }
            }

            println!("blockers {} and moves {}", blockers, move_bit_mask);
        }
    }
}
