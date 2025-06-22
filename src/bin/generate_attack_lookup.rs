use core::arch::x86_64::_pdep_u64;
use hhz::bit_boards::*;
use std::io::Write;

fn main() {
    if cfg!(target_arch = "x86_64") {
        println!("Running on x86_64 architecture");
    } else {
        panic!("This code is designed to run on x86_64 architecture only.");
    }

    let mut rook_lookup = [0; (ROOK_LOOK_UP_SIZE * 64) as usize];
    unsafe {
        for rook_square_index in 0u64..64u64 {
            let rook_square = square_index_to_square(rook_square_index);
            for sub_index in 0u64..(ROOK_LOOK_UP_SIZE) {
                let blockers: u64 = _pdep_u64(
                    sub_index,
                    ROOK_LOOKUP_MASK[rook_square_index as usize],
                );
                let mut move_bit_mask = 0;

                if rook_square.rank >= 1 {
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
                if rook_square.rank <= 5 {
                    'inner: for i in rook_square.rank + 1..8 {
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

                if rook_square.file >= 1 {
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
                if rook_square.file <= 5 {
                    'inner: for i in rook_square.file + 1..8 {
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
                rook_lookup[(rook_square_index * ROOK_LOOK_UP_SIZE + sub_index) as usize] =
                    move_bit_mask;
            }
        }
    }
    let mut rook_file = std::fs::File::create("assets/rook_lookup.bin").unwrap();
    rook_file.write_all(bytemuck::cast_slice(&rook_lookup)).unwrap();



    println!("done");
}
