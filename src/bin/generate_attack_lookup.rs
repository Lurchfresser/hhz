use core::arch::x86_64::_pdep_u64;
use hhz::bit_boards::*;
use std::io::Write;

fn main() {
    gen_bishop_look_up();
    gen_rook_look_up();
}

fn gen_rook_look_up() -> [u64; (ROOK_LOOK_UP_SIZE * 64) as usize] {
    if cfg!(target_arch = "x86_64") {
        // println!("Running on x86_64 architecture");
    } else {
        panic!("This code is designed to run on x86_64 architecture only.");
    }

    let mut rook_lookup = [0; (ROOK_LOOK_UP_SIZE * 64) as usize];
    unsafe {
        for rook_square_index in 0u64..64u64 {
            let rook_square = square_index_to_square(rook_square_index as usize);
            for sub_index in 0u64..(ROOK_LOOK_UP_SIZE) {
                let blockers: u64 =
                    _pdep_u64(sub_index, ROOK_LOOKUP_MASK[rook_square_index as usize]);
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
                if rook_square.rank <= 6 {
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
                if rook_square.file <= 6 {
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
    rook_file
        .write_all(bytemuck::cast_slice(&rook_lookup))
        .unwrap();

    // println!("done");
    rook_lookup
}

fn gen_bishop_look_up() {
    if cfg!(target_arch = "x86_64") {
        // println!("Running on x86_64 architecture");
    } else {
        panic!("This code is designed to run on x86_64 architecture only.");
    }

    let mut bishop_lookup = [0; (BISHOP_LOOK_UP_SIZE * 64) as usize];
    unsafe {
        for bishop_square_index in 0u64..64u64 {
            let bishop_square = square_index_to_square(bishop_square_index as usize);
            for sub_index in 0u64..(BISHOP_LOOK_UP_SIZE) {
                let blockers: u64 =
                    _pdep_u64(sub_index, BISHOP_LOOKUP_MASK[bishop_square_index as usize]);
                let mut move_bit_board = 0u64;

                // North-East direction
                let mut north_east_index = 0;
                loop {
                    let next_square = Square {
                        file: bishop_square.file + north_east_index,
                        rank: bishop_square.rank + north_east_index,
                    };
                    if next_square.file > 7 || next_square.rank > 7 {
                        break;
                    }
                    move_bit_board |= next_square.to_bit_board();
                    if next_square.to_bit_board() & blockers != 0 {
                        break;
                    }
                    north_east_index += 1;
                }

                // North-West direction
                let mut north_west_index = 0;
                loop {
                    if bishop_square.file < north_west_index {
                        break;
                    }
                    let next_square = Square {
                        file: bishop_square.file - north_west_index,
                        rank: bishop_square.rank + north_west_index,
                    };
                    if next_square.rank > 7 {
                        break;
                    }
                    move_bit_board |= next_square.to_bit_board();
                    if next_square.to_bit_board() & blockers != 0 {
                        break;
                    }
                    north_west_index += 1;
                }

                // South-East direction
                let mut south_east_index = 0;
                loop {
                    if bishop_square.rank < south_east_index {
                        break;
                    }
                    let next_square = Square {
                        file: bishop_square.file + south_east_index,
                        rank: bishop_square.rank - south_east_index,
                    };
                    if next_square.file > 7 {
                        break;
                    }
                    move_bit_board |= next_square.to_bit_board();
                    if next_square.to_bit_board() & blockers != 0 {
                        break;
                    }
                    south_east_index += 1;
                }

                // South-West direction
                let mut south_west_index = 0;
                loop {
                    if bishop_square.file < south_west_index
                        || bishop_square.rank < south_west_index
                    {
                        break;
                    }
                    let next_square = Square {
                        file: bishop_square.file - south_west_index,
                        rank: bishop_square.rank - south_west_index,
                    };
                    // if next_square.file < 0 || next_square.rank < 0 {
                    //     break;
                    // }
                    move_bit_board |= next_square.to_bit_board();
                    if next_square.to_bit_board() & blockers != 0 {
                        break;
                    }
                    south_west_index += 1;
                }
                move_bit_board &= !bishop_square.to_bit_board();
                bishop_lookup[(bishop_square_index * BISHOP_LOOK_UP_SIZE + sub_index) as usize] =
                    move_bit_board;
            }
        }
    }
    let mut bishop_file = std::fs::File::create("assets/bishop_lookup.bin").unwrap();
    bishop_file
        .write_all(bytemuck::cast_slice(&bishop_lookup))
        .unwrap();
}
