use crate::bit_boards::*;
use crate::board::*;
use crate::moves::*;
use crate::polyglot_zobrists::*;

struct PinAndCheckInfos {
    sliding_checkers: u64,
    stop_check_targets: u64,
    bishop_pinned_pieces: u64,
    rook_pinned_pieces: u64,
}

impl Board {
    //TODO: merge with gen-pseudo-legal-move-gen
    pub fn get_enemy_pawn_and_knight_checkers(&self) -> u64 {
        let king_index = if self.white_to_move {
            bitboard_to_square_index(self.white_king)
        } else {
            bitboard_to_square_index(self.black_king)
        };

        let enemy_pawns = if self.white_to_move {
            self.black_pawns
        } else {
            self.white_pawns
        };

        let enemy_knights = if self.white_to_move {
            self.black_knights
        } else {
            self.white_knights
        };

        #[allow(non_snake_case)]
        let ATTACKS_LOOKUP = if self.white_to_move {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ATTACKS_LOOKUP
        };

        let pawn_attacks = ATTACKS_LOOKUP[king_index] & enemy_pawns;

        let knight_attacks = FREE_KNIGHT_LOOKUP[king_index] & enemy_knights;

        knight_attacks | pawn_attacks
    }

    //TODO: add pawn structure lookup table
    pub fn gen_pawn_attack_squares(&self, for_white: bool) -> u64 {
        let mut pawns = if for_white {
            self.white_pawns
        } else {
            self.black_pawns
        };

        #[allow(non_snake_case)]
        let ATTACKS_LOOKUP = if for_white {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ATTACKS_LOOKUP
        };

        let mut pawn_attacks = 0u64;
        while pawns != 0 {
            let pawn_index = pop_lsb(&mut pawns);
            // 1. Generate attacks
            pawn_attacks |= ATTACKS_LOOKUP[pawn_index];
        }
        pawn_attacks
    }

    pub fn generate_pawn_moves(
        &self,
        moves: &mut MoveList,
        rook_pinned_pieces: u64,
        bishop_pinned_pieces: u64,
        to_mask: u64,
    ) {
        let mut pawns = if self.white_to_move {
            self.white_pawns
        } else {
            self.black_pawns
        };

        let enemy_pieces = if self.white_to_move {
            self.black_pieces
        } else {
            self.white_pieces
        };

        let own_king = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };

        #[allow(non_snake_case)]
        let ADVANCE_LOOKUP = if self.white_to_move {
            &WHITE_FREE_PAWN_ADVANCE_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ADVANCE_LOOKUP
        };

        #[allow(non_snake_case)]
        let ATTACKS_LOOKUP = if self.white_to_move {
            &WHITE_FREE_PAWN_ATTACKS_LOOKUP
        } else {
            &BLACK_FREE_PAWN_ATTACKS_LOOKUP
        };

        while pawns != 0 {
            let mut moves_for_pawn = 0u64;
            let pawn_index = pop_lsb(&mut pawns);
            let pawn_bit_board = square_index_to_bitboard(pawn_index);

            // 1. Generate attacks
            moves_for_pawn |= ATTACKS_LOOKUP[pawn_index]
                //                      mask out illegal en passant captures
                & (enemy_pieces | (self.en_passant_target & !rook_pinned_pieces));

            // 2. Generate advances
            let blockers = self.all_pieces ^ pawn_bit_board;
            let advances = ADVANCE_LOOKUP[pawn_index];

            if self.white_to_move {
                // For white, shift blockers UP to see if they block the double advance
                let invalid_advances = blockers | (blockers << 8);
                moves_for_pawn |= advances & !invalid_advances;
            } else {
                // For black, shift blockers DOWN to see if they block the double advance
                let invalid_advances = blockers | (blockers >> 8);
                moves_for_pawn |= advances & !invalid_advances;
            }

            if pawn_bit_board & bishop_pinned_pieces != 0 {
                if NORTH_EAST_LOOKUP[pawn_index] & own_king != 0 {
                    moves_for_pawn &= NORTH_EAST_LOOKUP[pawn_index]
                } else {
                    moves_for_pawn &= NORTH_WEST_LOOKUP[pawn_index]
                }
            } else if pawn_bit_board & rook_pinned_pieces != 0 {
                if HORIZONTALS_LOOKUP[pawn_index] & own_king != 0 {
                    moves_for_pawn &= HORIZONTALS_LOOKUP[pawn_index]
                } else {
                    moves_for_pawn &= VERTICALSS_LOOKUP[pawn_index]
                }
            }

            moves_for_pawn &= to_mask;

            let promotion_rank = if self.white_to_move { RANK_8 } else { RANK_1 };

            while moves_for_pawn != 0 {
                let to_index = pop_lsb(&mut moves_for_pawn);
                let to_bit_board = square_index_to_bitboard(to_index);
                if to_bit_board & promotion_rank != 0 {
                    self.push_promotion_moves(moves, to_index, pawn_index);
                } else if to_bit_board & self.en_passant_target != 0 {
                    moves.push(Move::en_passant(pawn_index, to_index));
                } else {
                    moves.push(Move::new(
                        pawn_index,
                        to_index,
                        to_bit_board & enemy_pieces != 0,
                    ));
                }
            }
        }
    }

    #[inline(always)]
    fn push_promotion_moves(&self, moves: &mut MoveList, to_index: usize, from_index: usize) {
        let is_capture = (self.all_pieces) & square_index_to_bitboard(to_index) != 0;

        if is_capture {
            moves.push(Move::capture_promotion(
                PieceKind::Queen,
                from_index,
                to_index,
            ));
            moves.push(Move::capture_promotion(
                PieceKind::Knight,
                from_index,
                to_index,
            ));
            moves.push(Move::capture_promotion(
                PieceKind::Rook,
                from_index,
                to_index,
            ));
            moves.push(Move::capture_promotion(
                PieceKind::Bishop,
                from_index,
                to_index,
            ));
        } else {
            moves.push(Move::promotion(PieceKind::Queen, from_index, to_index));
            moves.push(Move::promotion(PieceKind::Knight, from_index, to_index));
            moves.push(Move::promotion(PieceKind::Rook, from_index, to_index));
            moves.push(Move::promotion(PieceKind::Bishop, from_index, to_index));
        }
    }

    pub fn generate_knight_attack_squares(&self, for_white: bool) -> u64 {
        let mut knights = if for_white {
            self.white_knights
        } else {
            self.black_knights
        };
        let mut knight_attacks = 0u64;
        while knights != 0 {
            let knight_index = pop_lsb(&mut knights);
            knight_attacks |= FREE_KNIGHT_LOOKUP[knight_index];
        }
        knight_attacks
    }

    pub fn generate_knight_moves(
        &self,
        moves: &mut MoveList,
        all_pinned_pieces: u64,
        to_mask: u64,
    ) {
        let mut unpinned_knights = if self.white_to_move {
            self.white_knights
        } else {
            self.black_knights
        } & !all_pinned_pieces;

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        while unpinned_knights != 0 {
            let knight_index = pop_lsb(&mut unpinned_knights) as usize;
            let mut knight_attacks = (FREE_KNIGHT_LOOKUP[knight_index] & !own_pieces) & to_mask;
            while knight_attacks != 0 {
                let to_index = pop_lsb(&mut knight_attacks) as usize;
                moves.push(Move::new(
                    knight_index,
                    to_index,
                    self.pieces[to_index] != Piece::None,
                ))
            }
        }
    }

    /// masks out enemy king, because is used for legal king moves
    pub fn generate_bishop_and_queen_attack_squares(&self, for_white: bool) -> u64 {
        let mut bishops = if for_white {
            self.white_bishops | self.white_queens
        } else {
            self.black_bishops | self.black_queens
        };

        let enemy_king = if for_white {
            self.black_king
        } else {
            self.white_king
        };

        let mut bishop_attacks = 0u64;
        while bishops != 0 {
            let bishop_index = pop_lsb(&mut bishops);
            let bishop_attacks_looked_up =
                get_bishop_moves(bishop_index, self.all_pieces ^ enemy_king);
            bishop_attacks |= bishop_attacks_looked_up
        }
        bishop_attacks
    }

    pub fn generate_bishop_and_queen_moves(
        &self,
        moves: &mut MoveList,
        bishop_pinned_pieces: u64,
        rook_pinned_pieces: u64,
        to_mask: u64,
    ) {
        let mut bishops = if self.white_to_move {
            self.white_bishops | self.white_queens
        } else {
            self.black_bishops | self.black_queens
        };

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        let own_king = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };

        while bishops != 0 {
            let square_index = pop_lsb(&mut bishops) as usize;
            let bit_board = square_index_to_bitboard(square_index as usize);
            if bit_board & rook_pinned_pieces != 0 {
                // cant move along rook pin
                continue;
            }

            let bishop_attacks_looked_up = get_bishop_moves(square_index, self.all_pieces);
            let mut bishop_attacks = bishop_attacks_looked_up & !own_pieces;

            if (bit_board & bishop_pinned_pieces) != 0 {
                if NORTH_EAST_LOOKUP[square_index as usize] & own_king != 0 {
                    bishop_attacks &= NORTH_EAST_LOOKUP[square_index as usize]
                } else if NORTH_WEST_LOOKUP[square_index as usize] & own_king != 0 {
                    bishop_attacks &= NORTH_WEST_LOOKUP[square_index as usize]
                } else {
                    panic!("No overlap with king, even though pinned")
                }
            }

            bishop_attacks &= to_mask;
            while bishop_attacks != 0 {
                let to_index = pop_lsb(&mut bishop_attacks) as usize;
                moves.push(Move::new(
                    square_index,
                    to_index,
                    self.pieces[to_index] != Piece::None,
                ));
            }
        }
    }

    pub fn generate_rook_and_queen_attack_squares(&self, for_white: bool) -> u64 {
        let mut rooks = if for_white {
            self.white_rooks | self.white_queens
        } else {
            self.black_rooks | self.black_queens
        };

        let enemy_king = if for_white {
            self.black_king
        } else {
            self.white_king
        };

        let mut rook_attacks = 0u64;

        while rooks != 0 {
            let rook_index = pop_lsb(&mut rooks) as usize;
            rook_attacks |= get_rook_moves(rook_index as u32, self.all_pieces ^ enemy_king);
        }
        rook_attacks
    }

    pub fn generate_rook_and_queen_moves(
        &self,
        moves: &mut MoveList,
        bishop_pinned_pieces: u64,
        rook_pinned_pieces: u64,
        to_mask: u64,
    ) -> u64 {
        let mut rooks = if self.white_to_move {
            self.white_rooks | self.white_queens
        } else {
            self.black_rooks | self.black_queens
        };

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        let own_king = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };

        let mut all_rook_attacks = 0;

        while rooks != 0 {
            let rook_index = pop_lsb(&mut rooks) as usize;
            let rook_bit_board = square_index_to_bitboard(rook_index);
            if rook_bit_board & bishop_pinned_pieces != 0 {
                continue;
            }
            let rook_attacks_looked_up = get_rook_moves(rook_index as u32, self.all_pieces);
            let mut rook_attacks = rook_attacks_looked_up & !own_pieces;
            if rook_bit_board & rook_pinned_pieces != 0 {
                if HORIZONTALS_LOOKUP[rook_index] & own_king != 0 {
                    rook_attacks &= HORIZONTALS_LOOKUP[rook_index]
                } else {
                    rook_attacks &= VERTICALSS_LOOKUP[rook_index]
                }
            }
            rook_attacks &= to_mask;
            all_rook_attacks |= rook_attacks;
            while rook_attacks != 0 {
                let to_index = pop_lsb(&mut rook_attacks);
                moves.push(Move::new(
                    rook_index,
                    to_index as usize,
                    self.pieces[to_index] != Piece::None,
                ))
            }
        }
        all_rook_attacks
    }

    #[inline(always)]
    pub fn generate_king_attack_squares(&self, for_white: bool) -> u64 {
        let king_square = if for_white {
            self.white_king
        } else {
            self.black_king
        };

        FREE_KING_LOOKUP[bitboard_to_square_index(king_square)]
    }

    pub fn generate_king_moves(
        &self,
        moves: &mut MoveList,
        enemy_attack_square: u64,
        to_mask: u64,
    ) {
        let king_index = if self.white_to_move {
            bitboard_to_square_index(self.white_king)
        } else {
            bitboard_to_square_index(self.black_king)
        };

        let own_pieces = if self.white_to_move {
            self.white_pieces
        } else {
            self.black_pieces
        };

        let mut legal_king_moves =
            FREE_KING_LOOKUP[king_index] & (!own_pieces) & (!enemy_attack_square) & to_mask;

        while legal_king_moves != 0 {
            let to_index = pop_lsb(&mut legal_king_moves);

            moves.push(Move::new(
                king_index,
                to_index,
                self.pieces[to_index] != Piece::None,
            ))
        }
    }

    pub fn generate_castling_moves(&self, moves: &mut MoveList, enemy_attack_square: u64) {
        //function assumes king is not in check
        let castling_rights = if self.white_to_move {
            self.white_castling_rights
        } else {
            self.black_castling_rights
        };

        let king_side_mask = if self.white_to_move {
            WHITE_KINGSIDE_CASTLING_MASK
        } else {
            BLACK_KINGSIDE_CASTLING_MASK
        };

        let queen_side_check_mask = if self.white_to_move {
            WHITE_QUEENSIDE_CASTLING_CHECK_MASK
        } else {
            BLACK_QUEENSIDE_CASTLING_CHECK_MASK
        };

        let queen_side_free_squares_mask = if self.white_to_move {
            WHITE_QUEENSIDE_CASTLING_FREE_SQUARES_MASK
        } else {
            BLACK_QUEENSIDE_CASTLING_FREE_SQUARES_MASK
        };

        match castling_rights {
            CastlingRights::All => {
                if (queen_side_check_mask & enemy_attack_square)
                    | (self.all_pieces & queen_side_free_squares_mask)
                    == 0
                {
                    moves.push(Move::castles(false, self.white_to_move));
                }
                if king_side_mask & (enemy_attack_square | self.all_pieces) == 0 {
                    moves.push(Move::castles(true, self.white_to_move));
                }
            }
            CastlingRights::OnlyKingSide => {
                if king_side_mask & (enemy_attack_square | self.all_pieces) == 0 {
                    moves.push(Move::castles(true, self.white_to_move));
                }
            }
            CastlingRights::OnlyQueenSide => {
                if (queen_side_check_mask & enemy_attack_square)
                    | (self.all_pieces & queen_side_free_squares_mask)
                    == 0
                {
                    moves.push(Move::castles(false, self.white_to_move));
                }
            }
            CastlingRights::None => {}
        };
    }

    fn generate_pins_and_sliding_checkers(&self) -> PinAndCheckInfos {
        let mut stop_check_targets = 0u64;

        let kingsquare = if self.white_to_move {
            self.white_king
        } else {
            self.black_king
        };
        let king_square_index = bitboard_to_square_index(kingsquare);

        let enemy_rooks_squares = if self.white_to_move {
            self.black_rooks
        } else {
            self.white_rooks
        };

        let enemy_bishop_squares = if self.white_to_move {
            self.black_bishops
        } else {
            self.white_bishops
        };

        let enemy_queens_squares = if self.white_to_move {
            self.black_queens
        } else {
            self.white_queens
        };

        let mut potential_rook_piners = FREE_ROOK_LOOKUP[bitboard_to_square_index(kingsquare)]
            & (enemy_queens_squares | enemy_rooks_squares);

        let mut rook_pinned_pieces = 0u64;
        let mut sliding_checkers = 0u64;

        while potential_rook_piners != 0 {
            let rook_or_queen_square_index = pop_lsb(&mut potential_rook_piners);
            let ray = ROOK_SQUARE_TO_SQUARE_RAY_LOOKUP
                [rook_or_queen_square_index * 64 + king_square_index];
            // we need to check for all pieces, so no enemy pieces block the pin
            // this is later masked out to 0 with the "my_pieces & ray" instruction
            let pieces_between = ray & self.all_pieces;
            match pieces_between.count_ones() {
                0 => {
                    //Check detected
                    sliding_checkers |= square_index_to_bitboard(rook_or_queen_square_index);
                    stop_check_targets |= ray | sliding_checkers;
                }
                1 => {
                    // let my_pieces = if self.white_to_move {
                    //     self.white_pieces
                    // } else {
                    //     self.black_pieces
                    // };
                    //TODO: maybe also divide in 2 bitboards (now discoverers and pins together)
                    let pins = self.all_pieces & ray;
                    rook_pinned_pieces |= pins;
                }
                // for detecting illegal en passant captures, see:
                // 8/6k1/8/K1Pp1r2/8/8/8/8 w - - 0 1
                2 => 'match_arm: {
                    // only care for same rank
                    if ((rook_or_queen_square_index as i32) - (king_square_index as i32)).abs() >= 8
                    {
                        break 'match_arm;
                    }
                    let own_pawns = if self.white_to_move {
                        self.white_pawns
                    } else {
                        self.black_pawns
                    };
                    let maybe_own_pawn = own_pawns & ray;
                    if maybe_own_pawn.count_ones() != 1 {
                        break 'match_arm;
                    }
                    let pawn_attacks = if self.white_to_move {
                        WHITE_FREE_PAWN_ATTACKS_LOOKUP
                    } else {
                        BLACK_FREE_PAWN_ATTACKS_LOOKUP
                    };
                    rook_pinned_pieces |= pawn_attacks[bitboard_to_square_index(maybe_own_pawn)]
                        & self.en_passant_target
                }
                _ => {}
            }
        }

        let mut potential_bishop_piners = FREE_BISHOP_LOOKUP[bitboard_to_square_index(kingsquare)]
            & (enemy_queens_squares | enemy_bishop_squares);

        let mut bishop_pinned_pieces = 0u64;
        while potential_bishop_piners != 0 {
            let bishop_or_queen_square_index = pop_lsb(&mut potential_bishop_piners);
            let ray = BISHOP_SQUARE_TO_SQUARE_RAY_LOOKUP
                [bishop_or_queen_square_index * 64 + king_square_index];
            // we need to check for all pieces, so no enemy pieces block the pin
            // this is later masked out to 0 with the "my_pieces & ray" instruction
            let pieces_between = ray & self.all_pieces;
            if pieces_between.count_ones() == 0 {
                //Check detected
                sliding_checkers |= square_index_to_bitboard(bishop_or_queen_square_index);
                stop_check_targets |= ray | sliding_checkers;
            } else if pieces_between.count_ones() == 1 {
                // let my_pieces = if self.white_to_move {
                //     self.white_pieces
                // } else {
                //     self.black_pieces
                // };
                //TODO: maybe also divide in 2 bitboards (now discoverers and pins together)
                let pins = self.all_pieces & ray;
                bishop_pinned_pieces |= pins;
            }
        }
        //todo!()
        //TODO: move in struct
        PinAndCheckInfos {
            sliding_checkers,
            stop_check_targets,
            bishop_pinned_pieces,
            rook_pinned_pieces,
        }
    }

    pub fn generate_legal_moves_temp(&self) -> MoveList {
        let mut moves = MoveList::default();

        let pin_and_check_infos = self.generate_pins_and_sliding_checkers();
        let all_pinned_pieces =
            pin_and_check_infos.bishop_pinned_pieces | pin_and_check_infos.rook_pinned_pieces;
        let checkers =
            pin_and_check_infos.sliding_checkers | self.get_enemy_pawn_and_knight_checkers();

        let enemy_pawn_attacks = self.gen_pawn_attack_squares(!self.white_to_move);
        let enemy_knight_attacks = self.generate_knight_attack_squares(!self.white_to_move);
        let enemy_bishop_and_queen_attacks =
            self.generate_bishop_and_queen_attack_squares(!self.white_to_move);
        let enemy_rook_and_queen_attacks =
            self.generate_rook_and_queen_attack_squares(!self.white_to_move);
        let enemy_king_attacks = self.generate_king_attack_squares(!self.white_to_move);
        let all_enemy_attack_squares = enemy_pawn_attacks
            | enemy_knight_attacks
            | enemy_bishop_and_queen_attacks
            | enemy_rook_and_queen_attacks
            | enemy_king_attacks;

        match checkers.count_ones() {
            //No checks
            0 => {
                self.generate_pawn_moves(
                    &mut moves,
                    pin_and_check_infos.rook_pinned_pieces,
                    pin_and_check_infos.bishop_pinned_pieces,
                    u64::MAX,
                );
                self.generate_knight_moves(&mut moves, all_pinned_pieces, u64::MAX);
                self.generate_bishop_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    u64::MAX,
                );
                self.generate_rook_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    u64::MAX,
                );
                self.generate_king_moves(&mut moves, all_enemy_attack_squares, u64::MAX);
                self.generate_castling_moves(&mut moves, all_enemy_attack_squares);
            }
            //One check, king and blocking moves or capturing checker only
            1 => {
                let stop_check_mask = checkers | pin_and_check_infos.stop_check_targets;
                self.generate_pawn_moves(
                    &mut moves,
                    pin_and_check_infos.rook_pinned_pieces,
                    pin_and_check_infos.bishop_pinned_pieces,
                    stop_check_mask,
                );
                self.generate_knight_moves(&mut moves, all_pinned_pieces, stop_check_mask);
                self.generate_bishop_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    stop_check_mask,
                );
                self.generate_rook_and_queen_moves(
                    &mut moves,
                    pin_and_check_infos.bishop_pinned_pieces,
                    pin_and_check_infos.rook_pinned_pieces,
                    stop_check_mask,
                );
                // King should move out of the way and not to the check
                self.generate_king_moves(&mut moves, all_enemy_attack_squares, u64::MAX);
            }
            //2 checks, only king moves possible
            2 => {
                self.generate_king_moves(&mut moves, all_enemy_attack_squares, u64::MAX);
            }
            _ => panic!(
                "There cant be more than 2 checkers, but there are{}",
                checkers.count_ones()
            ),
        }

        moves
    }

    pub fn in_check_temp(&self) -> bool {
        let pin_and_check_infos = self.generate_pins_and_sliding_checkers();
        let checkers =
            pin_and_check_infos.sliding_checkers | self.get_enemy_pawn_and_knight_checkers();
        checkers > 0
    }

    pub fn make_uci_move_temp(&self, uci_move: &str) -> Self {
        let moves = self.generate_legal_moves_temp();
        for m in moves {
            if m.to_uci() == uci_move {
                return self.make_move_temp(m);
            };
        }
        panic!("uci move: {} not found", uci_move);
    }

    pub fn make_move_temp(&self, _move: Move) -> Self {
        let mut new_board = *self;
        new_board.en_passant_target = 0;

        if self.en_passant_target != 0 {
            let ep_square_index = bitboard_to_square_index(self.en_passant_target);
            let ep_file = ep_square_index % 8;
            //undo prev ep_target from zobrist hash
            new_board.zobrist_hash ^= ZOBRISTS_EN_PASSANT_FILE[ep_file];
        }

        let from = _move.from();
        let to = _move.to();
        let from_bb = square_index_to_bitboard(from);
        let to_bb = square_index_to_bitboard(to);
        let move_mask = from_bb | to_bb;

        let moved_piece = self.pieces[from];

        if _move.is_castle_short() {
            new_board.pieces[from] = Piece::None;
            if new_board.white_to_move {
                // Update castling rights in hash
                new_board.zobrist_hash ^= self.white_castling_rights.zobrist_hash(true);
                new_board.white_castling_rights = CastlingRights::None;

                // Remove king from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_KINGS[from]; // Remove king from E1
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_KINGS[WHITE_KINGSIDE_CASTLE_INDEX]; // Add king to G1

                // Remove rook from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[WHITE_KINGSIDE_CASTLE_ROOK_INDEX]; // Remove rook from H1
                new_board.zobrist_hash ^=
                    ZOBRISTS_WHITE_ROOKS[WHITE_KINGSIDE_CASTLE_ROOK_INDEX - 2]; // Add rook to F1

                new_board.white_king = WHITE_KINGSIDE_CASTLE_KING_BIT_BOARD;
                new_board.white_rooks ^= WHITE_KINGSIDE_CASTLE_ROOK_MASK;
                new_board.pieces[WHITE_KINGSIDE_CASTLE_ROOK_INDEX] = Piece::None;
                new_board.pieces[WHITE_KINGSIDE_CASTLE_ROOK_INDEX - 2] =
                    Piece::Rook { white: true };
                new_board.pieces[WHITE_KINGSIDE_CASTLE_INDEX] = Piece::King { white: true };
            } else {
                // Update castling rights in hash
                new_board.zobrist_hash ^= self.black_castling_rights.zobrist_hash(false);
                new_board.black_castling_rights = CastlingRights::None;

                // Remove king from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_KINGS[from]; // Remove king from E8
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_KINGS[BLACK_KINGSIDE_CASTLE_INDEX]; // Add king to G8

                // Remove rook from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[BLACK_KINGSIDE_CASTLE_ROOK_INDEX]; // Remove rook from H8
                new_board.zobrist_hash ^=
                    ZOBRISTS_BLACK_ROOKS[BLACK_KINGSIDE_CASTLE_ROOK_INDEX - 2]; // Add rook to F8

                new_board.black_king = BLACK_KINGSIDE_CASTLE_KING_BIT_BOARD;
                new_board.black_rooks ^= BLACK_KINGSIDE_CASTLE_ROOK_MASK;
                new_board.pieces[BLACK_KINGSIDE_CASTLE_ROOK_INDEX] = Piece::None;
                new_board.pieces[BLACK_KINGSIDE_CASTLE_ROOK_INDEX - 2] =
                    Piece::Rook { white: false };
                new_board.pieces[BLACK_KINGSIDE_CASTLE_INDEX] = Piece::King { white: false };
            }
            new_board.recompute_combined_bit_boards();
            new_board.update_board_state(false, false);

            return new_board;
        } else if _move.is_castle_long() {
            new_board.pieces[from] = Piece::None;
            if new_board.white_to_move {
                // Update castling rights in hash
                new_board.zobrist_hash ^= self.white_castling_rights.zobrist_hash(true);
                new_board.white_castling_rights = CastlingRights::None;

                // Remove king from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_KINGS[from]; // Remove king from E1
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_KINGS[WHITE_QUEENSIDE_CASTLE_INDEX]; // Add king to C1

                // Remove rook from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[WHITE_QUEENSIDE_CASTLE_ROOK_INDEX]; // Remove rook from A1
                new_board.zobrist_hash ^=
                    ZOBRISTS_WHITE_ROOKS[WHITE_QUEENSIDE_CASTLE_ROOK_INDEX + 3]; // Add rook to D1

                new_board.white_king = WHITE_QUEENSIDE_CASTLE_KING_BIT_BOARD;
                new_board.white_rooks ^= WHITE_QUEENSIDE_CASTLE_ROOK_MASK;
                new_board.pieces[WHITE_QUEENSIDE_CASTLE_ROOK_INDEX] = Piece::None;
                new_board.pieces[WHITE_QUEENSIDE_CASTLE_ROOK_INDEX + 3] =
                    Piece::Rook { white: true };
                new_board.pieces[WHITE_QUEENSIDE_CASTLE_INDEX] = Piece::King { white: true };
            } else {
                // Update castling rights in hash
                new_board.zobrist_hash ^= self.black_castling_rights.zobrist_hash(false);
                new_board.black_castling_rights = CastlingRights::None;

                // Remove king from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_KINGS[from]; // Remove king from E8
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_KINGS[BLACK_QUEENSIDE_CASTLE_INDEX]; // Add king to C8

                // Remove rook from original position and add to new position in hash
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[BLACK_QUEENSIDE_CASTLE_ROOK_INDEX]; // Remove rook from A8
                new_board.zobrist_hash ^=
                    ZOBRISTS_BLACK_ROOKS[BLACK_QUEENSIDE_CASTLE_ROOK_INDEX + 3]; // Add rook to D8

                new_board.black_king = BLACK_QUEENSIDE_CASTLE_KING_BIT_BOARD;
                new_board.black_rooks ^= BLACK_QUEENSIDE_CASTLE_ROOK_MASK;
                new_board.pieces[BLACK_QUEENSIDE_CASTLE_ROOK_INDEX] = Piece::None;
                new_board.pieces[BLACK_QUEENSIDE_CASTLE_ROOK_INDEX + 3] =
                    Piece::Rook { white: false };
                new_board.pieces[BLACK_QUEENSIDE_CASTLE_INDEX] = Piece::King { white: false };
            }
            new_board.recompute_combined_bit_boards();
            new_board.update_board_state(false, false);

            return new_board;
        }

        if _move.is_capture() {
            let mut captured_piece = new_board.pieces[to];
            // Additional logic for en-passant capture
            let captured_piece_bb = if _move.is_en_passant() {
                if new_board.white_to_move {
                    captured_piece = new_board.pieces[to + 8];
                    to_bb >> 8
                } else {
                    captured_piece = new_board.pieces[to - 8];
                    to_bb << 8
                }
            } else {
                to_bb
            };

            match captured_piece {
                Piece::Pawn { .. } => {
                    if new_board.white_to_move {
                        new_board.black_pawns ^= captured_piece_bb;
                        // Remove black pawn from hash
                        if _move.is_en_passant() {
                            new_board.zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[to + 8];
                        } else {
                            new_board.zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[to];
                        }
                    } else {
                        new_board.white_pawns ^= captured_piece_bb;
                        // Remove white pawn from hash
                        if _move.is_en_passant() {
                            new_board.zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[to - 8];
                        } else {
                            new_board.zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[to];
                        }
                    }
                }
                Piece::Knight { .. } => {
                    if new_board.white_to_move {
                        new_board.black_knights ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_KNIGHTS[to];
                    } else {
                        new_board.white_knights ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_KNIGHTS[to];
                    }
                }
                Piece::Bishop { .. } => {
                    if new_board.white_to_move {
                        new_board.black_bishops ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_BISHOPS[to];
                    } else {
                        new_board.white_bishops ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_BISHOPS[to];
                    }
                }
                Piece::Rook { .. } => {
                    if new_board.white_to_move {
                        new_board.black_rooks ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[to];
                    } else {
                        new_board.white_rooks ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[to];
                    }
                }
                Piece::Queen { .. } => {
                    if new_board.white_to_move {
                        new_board.black_queens ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_QUEENS[to];
                    } else {
                        new_board.white_queens ^= captured_piece_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_QUEENS[to];
                    }
                }
                // King capture is illegal and should not happen
                Piece::King { .. } => panic!("Cannot capture a king"),
                Piece::None => {
                    if !_move.is_en_passant() {
                        panic!("Capture flag set, but no piece on target square.")
                    }
                }
            }
        }

        new_board.pieces[to] = moved_piece;
        new_board.pieces[from] = Piece::None;

        // Update the bitboard of the moving piece

        match moved_piece {
            Piece::Pawn { .. } => {
                if new_board.white_to_move {
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[to]; // Add to new position
                    new_board.white_pawns ^= move_mask;

                    let was_double_push = to - from == 16;
                    if was_double_push {
                        // Polyglot spec: hash only if a pawn can actually perform the capture.
                        let ep_square_index = to - 8;
                        let adjacent_files_mask = WHITE_FREE_PAWN_ATTACKS_LOOKUP[ep_square_index];
                        let can_capture_ep = (new_board.black_pawns & adjacent_files_mask) != 0;

                        if can_capture_ep {
                            new_board.en_passant_target = square_index_to_bitboard(to - 8);
                            let ep_file = (to % 8) as usize;
                            new_board.zobrist_hash ^= ZOBRISTS_EN_PASSANT_FILE[ep_file];
                        }
                    }
                } else {
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[to]; // Add to new position
                    new_board.black_pawns ^= move_mask;

                    let was_double_push = from - to == 16;
                    if was_double_push {
                        // Polyglot spec: hash only if a pawn can actually perform the capture.
                        let ep_square_index = to + 8;
                        let adjacent_files_mask = BLACK_FREE_PAWN_ATTACKS_LOOKUP[ep_square_index];
                        let can_capture_ep = (new_board.white_pawns & adjacent_files_mask) != 0;

                        if can_capture_ep {
                            new_board.en_passant_target = square_index_to_bitboard(to + 8);
                            let ep_file = (to % 8) as usize;
                            new_board.zobrist_hash ^= ZOBRISTS_EN_PASSANT_FILE[ep_file];
                        }
                    }
                }
            }
            Piece::Knight { .. } => {
                if new_board.white_to_move {
                    new_board.white_knights ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_KNIGHTS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_KNIGHTS[to]; // Add to new position
                } else {
                    new_board.black_knights ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_KNIGHTS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_KNIGHTS[to]; // Add to new position
                }
            }
            Piece::Bishop { .. } => {
                if new_board.white_to_move {
                    new_board.white_bishops ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_BISHOPS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_BISHOPS[to]; // Add to new position
                } else {
                    new_board.black_bishops ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_BISHOPS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_BISHOPS[to]; // Add to new position
                }
            }
            Piece::Rook { .. } => {
                if new_board.white_to_move {
                    new_board.white_rooks ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[to]; // Add to new position
                } else {
                    new_board.black_rooks ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[to]; // Add to new position
                }
            }
            Piece::Queen { .. } => {
                if new_board.white_to_move {
                    new_board.white_queens ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_QUEENS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_QUEENS[to]; // Add to new position
                } else {
                    new_board.black_queens ^= move_mask;
                    // Update Zobrist hash for moving piece
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_QUEENS[from]; // Remove from original position
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_QUEENS[to]; // Add to new position
                }
            }
            Piece::King { .. } => {
                if new_board.white_to_move {
                    new_board.white_king ^= move_mask;
                    new_board.zobrist_hash ^= self.white_castling_rights.zobrist_hash(true);
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_KINGS[from];
                    new_board.zobrist_hash ^= ZOBRISTS_WHITE_KINGS[to];
                    new_board.white_castling_rights = CastlingRights::None;
                } else {
                    new_board.black_king ^= move_mask;
                    new_board.zobrist_hash ^= self.black_castling_rights.zobrist_hash(false);
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_KINGS[from];
                    new_board.zobrist_hash ^= ZOBRISTS_BLACK_KINGS[to];
                    new_board.black_castling_rights = CastlingRights::None;
                }
            }
            Piece::None => panic!(
                "make_move tried to move a piece from an empty square: {}",
                _move.to_uci()
            ),
        }

        // Update castling rights if a rook is moved or captured
        if from == WHITE_KINGSIDE_CASTLE_ROOK_INDEX || to == WHITE_KINGSIDE_CASTLE_ROOK_INDEX {
            new_board.white_castling_rights = new_board
                .white_castling_rights
                .remove_side(CastlingRights::OnlyKingSide);
            new_board.zobrist_hash ^= self.white_castling_rights.zobrist_hash(true);
            new_board.zobrist_hash ^= new_board.white_castling_rights.zobrist_hash(true);
        }
        if from == WHITE_QUEENSIDE_CASTLE_ROOK_INDEX || to == WHITE_QUEENSIDE_CASTLE_ROOK_INDEX {
            new_board.white_castling_rights = new_board
                .white_castling_rights
                .remove_side(CastlingRights::OnlyQueenSide);
            new_board.zobrist_hash ^= self.white_castling_rights.zobrist_hash(true);
            new_board.zobrist_hash ^= new_board.white_castling_rights.zobrist_hash(true);
        }
        if from == BLACK_KINGSIDE_CASTLE_ROOK_INDEX || to == BLACK_KINGSIDE_CASTLE_ROOK_INDEX {
            new_board.black_castling_rights = new_board
                .black_castling_rights
                .remove_side(CastlingRights::OnlyKingSide);
            new_board.zobrist_hash ^= self.black_castling_rights.zobrist_hash(false);
            new_board.zobrist_hash ^= new_board.black_castling_rights.zobrist_hash(false);
        }
        if from == BLACK_QUEENSIDE_CASTLE_ROOK_INDEX || to == BLACK_QUEENSIDE_CASTLE_ROOK_INDEX {
            new_board.black_castling_rights = new_board
                .black_castling_rights
                .remove_side(CastlingRights::OnlyQueenSide);
            new_board.zobrist_hash ^= self.black_castling_rights.zobrist_hash(false);
            new_board.zobrist_hash ^= new_board.black_castling_rights.zobrist_hash(false);
        }

        // Handle promotions
        if _move.is_promotion() {
            if new_board.white_to_move {
                new_board.white_pawns ^= to_bb;
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[to]
            } else {
                new_board.black_pawns ^= to_bb;
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[to]
            }
            let promoted_piece = match _move.promotion_piece() {
                Some(PieceKind::Queen) => {
                    if new_board.white_to_move {
                        new_board.white_queens |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_QUEENS[to];
                    } else {
                        new_board.black_queens |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_QUEENS[to];
                    }
                    Piece::Queen {
                        white: new_board.white_to_move,
                    }
                }
                Some(PieceKind::Rook) => {
                    if new_board.white_to_move {
                        new_board.white_rooks |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_ROOKS[to];
                    } else {
                        new_board.black_rooks |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_ROOKS[to];
                    }
                    Piece::Rook {
                        white: new_board.white_to_move,
                    }
                }
                Some(PieceKind::Bishop) => {
                    if new_board.white_to_move {
                        new_board.white_bishops |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_BISHOPS[to];
                    } else {
                        new_board.black_bishops |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_BISHOPS[to];
                    }
                    Piece::Bishop {
                        white: new_board.white_to_move,
                    }
                }
                Some(PieceKind::Knight) => {
                    if new_board.white_to_move {
                        new_board.white_knights |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_WHITE_KNIGHTS[to];
                    } else {
                        new_board.black_knights |= to_bb;
                        new_board.zobrist_hash ^= ZOBRISTS_BLACK_KNIGHTS[to];
                    }
                    Piece::Knight {
                        white: new_board.white_to_move,
                    }
                }
                _ => panic!("Invalid promotion"),
            };
            new_board.pieces[to] = promoted_piece;
        }

        if _move.is_en_passant() {
            let ep_target_index = if new_board.white_to_move {
                to - 8
            } else {
                to + 8
            };

            new_board.pieces[ep_target_index] = Piece::None;
            if new_board.white_to_move {
                new_board.black_pawns ^= square_index_to_bitboard(ep_target_index);
                new_board.zobrist_hash ^= ZOBRISTS_BLACK_PAWNS[ep_target_index];
            } else {
                new_board.white_pawns ^= square_index_to_bitboard(ep_target_index);
                new_board.zobrist_hash ^= ZOBRISTS_WHITE_PAWNS[ep_target_index];
            }
        }

        new_board.update_board_state(
            moved_piece
                == Piece::Pawn {
                    white: new_board.white_to_move,
                },
            _move.is_capture(),
        );
        new_board.recompute_combined_bit_boards();
        new_board
    }
    fn update_board_state(&mut self, pawn_moved: bool, was_capture: bool) {
        if pawn_moved || was_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if !self.white_to_move {
            self.full_move_number += 1;
        }
        self.white_to_move = !self.white_to_move;
        self.zobrist_hash ^= ZOBRISTS_WHITE_TO_MOVE;
    }

    fn recompute_combined_bit_boards(&mut self) {
        self.white_pieces = self.white_pawns
            | self.white_knights
            | self.white_bishops
            | self.white_rooks
            | self.white_queens
            | self.white_king;
        self.black_pieces = self.black_pawns
            | self.black_knights
            | self.black_bishops
            | self.black_rooks
            | self.black_queens
            | self.black_king;
        self.all_pieces = self.white_pieces | self.black_pieces;
    }
}
// src/move_gen.rs
// ... (rest of the file is unchanged)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    const ZOBRIST_TEST_CASES: &[(&str, &[&str], &str, u64)] = &[
        (
            "starting position",
            &[],
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0x463b96181691fc9c,
        ),
        (
            "position after e2e4",
            &["e2e4"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            0x823c9b50fd114196,
        ),
        (
            "position after e2e4 d7d5",
            &["e2e4", "d7d5"],
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
            0x0756b94461c50fb0,
        ),
        (
            "position after e2e4 d7d5 e4e5",
            &["e2e4", "d7d5", "e4e5"],
            "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2",
            0x662fafb965db29d4,
        ),
        (
            "position after e2e4 d7d5 e4e5 f7f5",
            &["e2e4", "d7d5", "e4e5", "f7f5"],
            "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
            0x22a48b5a8e47ff78,
        ),
        (
            "position after e2e4 d7d5 e4e5 f7f5 e1e2",
            &["e2e4", "d7d5", "e4e5", "f7f5", "e1e2"],
            "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR b kq - 0 3",
            0x652a607ca3f242c1,
        ),
        (
            "position after e2e4 d7d5 e4e5 f7f5 e1e2 e8f7",
            &["e2e4", "d7d5", "e4e5", "f7f5", "e1e2", "e8f7"],
            "rnbq1bnr/ppp1pkpp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR w - - 0 4",
            0x00fdd303c946bdd9,
        ),
        (
            "position after a2a4 b7b5 h2h4 b5b4 c2c4",
            &["a2a4", "b7b5", "h2h4", "b5b4", "c2c4"],
            "rnbqkbnr/p1pppppp/8/8/PpP4P/8/1P1PPPP1/RNBQKBNR b KQkq c3 0 3",
            0x3c8123ea7b067637,
        ),
        (
            "position after a2a4 b7b5 h2h4 b5b4 c2c4 b4c3 a1a3",
            &["a2a4", "b7b5", "h2h4", "b5b4", "c2c4", "b4c3", "a1a3"],
            "rnbqkbnr/p1pppppp/8/8/P6P/R1p5/1P1PPPP1/1NBQKBNR b Kkq - 0 4",
            0x5c3f9b829b279560,
        ),
    ];

    #[test]
    fn test_incremental_zobrist_hashing() {
        for (description, moves, _fen, expected_hash) in ZOBRIST_TEST_CASES.iter() {
            if moves.is_empty() {
                continue;
            }
            let mut board =
                Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .unwrap();
            for uci_move in *moves {
                board = board.make_uci_move_temp(uci_move);
            }
            assert_eq!(
                board.zobrist_hash, *expected_hash,
                "Failed at: {}",
                description
            );
        }
    }

    #[test]
    fn test_zobrist_hashing_from_fen() {
        for (description, _moves, fen, expected_hash) in ZOBRIST_TEST_CASES.iter() {
            let board = Board::from_fen(fen).unwrap();
            assert_eq!(
                board.zobrist_hash, *expected_hash,
                "Failed at: {}",
                description
            );
        }
    }

    #[test]
    fn test_zobrist_transposition() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/1K3BNR w kq - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        let initial_hash = board.zobrist_hash;

        // Sequence of moves that returns to the same position
        board = board.make_uci_move_temp("b1a1");
        board = board.make_uci_move_temp("b8a6");
        board = board.make_uci_move_temp("a1b1");
        board = board.make_uci_move_temp("a6b8");

        assert_eq!(
            initial_hash, board.zobrist_hash,
            "Transposition failed: hashes do not match"
        );
    }

    #[test]
    fn test_zobrist_transposition_knight_moves() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        let initial_hash = board.zobrist_hash;

        // Sequence of moves that returns to the same position
        board = board.make_uci_move_temp("g1f3");
        board = board.make_uci_move_temp("g8f6");
        board = board.make_uci_move_temp("f3g1");
        board = board.make_uci_move_temp("f6g8");

        assert_eq!(
            initial_hash, board.zobrist_hash,
            "Knight move transposition failed: hashes do not match"
        );
    }

    #[test]
    fn test_zobrist_transposition_different_move_orders() {
        // Path 1: 1. e4 e5 2. Nf3 Nc6
        let mut board1 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        board1 = board1.make_uci_move_temp("e2e4");
        board1 = board1.make_uci_move_temp("e7e5");
        board1 = board1.make_uci_move_temp("g1f3");
        board1 = board1.make_uci_move_temp("b8c6");

        // Path 2: 1. Nf3 Nc6 2. e4 e5
        let mut board2 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        board2 = board2.make_uci_move_temp("g1f3");
        board2 = board2.make_uci_move_temp("b8c6");
        board2 = board2.make_uci_move_temp("e2e4");
        board2 = board2.make_uci_move_temp("e7e5");

        assert_eq!(
            board1.zobrist_hash, board2.zobrist_hash,
            "Different move order transposition failed: hashes do not match"
        );
    }

    #[test]
    fn test_zobrist_transposition_with_capture() {
        let mut path1 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        path1 = path1.make_uci_move_temp("f2f4");
        path1 = path1.make_uci_move_temp("b8c6");
        path1 = path1.make_uci_move_temp("f4f5");
        path1 = path1.make_uci_move_temp("e7e5");
        path1 = path1.make_uci_move_temp("f5e6");
        path1 = path1.make_uci_move_temp("d7e6");
        path1 = path1.make_uci_move_temp("e1f2");

        let mut path2 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        path2 = path2.make_uci_move_temp("f2f4");
        path2 = path2.make_uci_move_temp("b8c6");
        path2 = path2.make_uci_move_temp("e1f2");
        path2 = path2.make_uci_move_temp("e7e6");
        path2 = path2.make_uci_move_temp("f4f5");
        path2 = path2.make_uci_move_temp("d8g5");
        path2 = path2.make_uci_move_temp("f5e6");
        path2 = path2.make_uci_move_temp("d7e6");
        path2 = path2.make_uci_move_temp("g1f3");
        path2 = path2.make_uci_move_temp("g5d8");
        path2 = path2.make_uci_move_temp("f3g1");


        assert_eq!(
            path1.zobrist_hash, path2.zobrist_hash,
            "Capture transposition failed: hashes do not match"
        );
    }
}
