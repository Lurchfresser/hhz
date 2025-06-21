fn main() {
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
        // knight_lookup[i as usize] = moves;
        println!(
            "square {} and bit mask: {}",
            1u64 << i,
            knight_lookup[i as usize]
        );
    }
}
