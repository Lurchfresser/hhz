# .vscode/move_formatter.py

# This function will be called by LLDB for any variable of type 'hhz::board::Move'
def format_move(valobj, internal_dict):
    """
    Reads the raw 'mask' from a hhz::board::Move struct
    and formats it into algebraic notation like 'e2e4'.
    """
    try:
        # Helper function to convert a square index (0-63) to algebraic notation (e.g., "e4")
        def to_algebraic(index):
            if not (0 <= index <= 63):
                return "??"
            file_char = chr(ord('a') + (index % 8))
            rank_char = str((index // 8) + 1)
            return f"{file_char}{rank_char}"

        # 1. Get the 'mask' field from the Rust struct.
        # valobj represents the `Move` variable in the debugger.
        mask_val_obj = valobj.GetChildMemberWithName('mask')

        # Error checking is good practice.
        if not mask_val_obj.IsValid():
            return "Error: Could not find 'mask' field"

        # 2. Get the value of the mask as an integer.
        # GetValueAsUnsigned() is more direct than GetValue() and parsing from a string.
        mask = mask_val_obj.GetValueAsUnsigned()

        # 3. Decode the 'from' and 'to' square indices from the mask.
        # This logic mirrors your Rust implementation exactly.
        from_square = mask & 0x3F          # Lower 6 bits for the 'from' square
        to_square = (mask >> 6) & 0x3F     # Next 6 bits for the 'to' square

        # 4. Convert indices to algebraic notation and return the final string.
        return f"{to_algebraic(from_square)}{to_algebraic(to_square)}"

    except Exception as e:
        # If anything goes wrong, return an error message to see in the debugger.
        return f"Error formatting move: {e}"

def format_move_list(valobj, internal_dict):
    # ... (existing implementation)
    pass

# This function will be called by LLDB for any variable of type 'hhz::board::Board'
# (v4, Bitboard-based implementation for maximum reliability)
def format_board(valobj, internal_dict):
    """
    Reads the bitboards from a hhz::board::Board struct
    and formats them into a FEN string. This avoids fragile enum parsing.
    """
    try:
        # 1. Reconstruct the board from bitboards
        pieces = [None] * 64
        
        piece_map = {
            'white_pawns': 'P', 'white_knights': 'N', 'white_bishops': 'B',
            'white_rooks': 'R', 'white_queens': 'Q', 'white_king': 'K',
            'black_pawns': 'p', 'black_knights': 'n', 'black_bishops': 'b',
            'black_rooks': 'r', 'black_queens': 'q', 'black_king': 'k',
        }

        for field_name, piece_char in piece_map.items():
            # Read the u64 value for the current bitboard
            bitboard_val = valobj.GetChildMemberWithName(field_name).GetValueAsUnsigned()
            
            # Iterate through the bits of the bitboard
            square_index = 0
            while bitboard_val > 0:
                if (bitboard_val & 1) == 1:
                    # If the bit is set, place the piece in our array
                    pieces[square_index] = piece_char
                
                bitboard_val >>= 1
                square_index += 1

        # 2. Generate FEN from the reconstructed piece array
        fen_parts = []
        for rank in range(7, -1, -1):
            empty_squares = 0
            rank_str = ""
            for file in range(8):
                index = rank * 8 + file
                piece_char = pieces[index]
                
                if piece_char is None:
                    empty_squares += 1
                else:
                    if empty_squares > 0:
                        rank_str += str(empty_squares)
                        empty_squares = 0
                    rank_str += piece_char
            
            if empty_squares > 0:
                rank_str += str(empty_squares)
            fen_parts.append(rank_str)
        
        fen = "/".join(fen_parts)

        # 3. Add the rest of the FEN data (which was already working)
        # Helper to convert square index to algebraic notation
        def to_algebraic(index):
            file = chr(ord('a') + (index % 8))
            rank = str((index // 8) + 1)
            return f"{file}{rank}"

        # Active Color
        white_to_move = valobj.GetChildMemberWithName('white_to_move').GetValueAsUnsigned()
        fen += " w" if white_to_move else " b"

        # Castling Rights
        castling_str = ""
        white_rights = valobj.GetChildMemberWithName('white_castling_rights').GetChildAtIndex(0).GetName()
        black_rights = valobj.GetChildMemberWithName('black_castling_rights').GetChildAtIndex(0).GetName()
        
        if white_rights == "All": castling_str += "KQ"
        elif white_rights == "OnlyKingSide": castling_str += "K"
        elif white_rights == "OnlyQueenSide": castling_str += "Q"

        if black_rights == "All": castling_str += "kq"
        elif black_rights == "OnlyKingSide": castling_str += "k"
        elif black_rights == "OnlyQueenSide": castling_str += "q"
        
        fen += f" {castling_str or '-'}"

        # En Passant Target
        ep_target = valobj.GetChildMemberWithName('en_passant_target').GetValueAsUnsigned()
        if ep_target == 0:
            fen += " -"
        else:
            ep_index = ep_target.bit_length() - 1
            fen += f" {to_algebraic(ep_index)}"

        # Halfmove Clock & Fullmove Number
        halfmove = valobj.GetChildMemberWithName('halfmove_clock').GetValueAsUnsigned()
        fullmove = valobj.GetChildMemberWithName('full_move_number').GetValueAsUnsigned()
        fen += f" {halfmove} {fullmove}"

        return fen

    except Exception as e:
        # It's useful to see the error message directly in the debugger
        return f"Error in format_board: {e}"