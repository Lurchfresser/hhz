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
    """
    Reads the length from the inner ArrayVec of a MoveList and provides a clean summary.
    """
    try:
        # 1. Access the private 'moves' field of the MoveList struct.
        #    This 'moves' field is the ArrayVec.
        array_vec_obj = valobj.GetChildMemberWithName('moves')

        if not array_vec_obj.IsValid():
            return "Error: Could not find 'moves' field"

        # 2. From the ArrayVec, get its 'len' field.
        len_obj = array_vec_obj.GetChildMemberWithName('len')

        if not len_obj.IsValid():
            return "Error: Could not find 'len' field in ArrayVec"

        # 3. Get the integer value of the length.
        length = len_obj.GetValueAsUnsigned()

        # 4. Return a formatted summary string. When the user expands this in the
        #    debugger, LLDB will automatically apply our 'format_move' function
        #    to each element.
        return f"MoveList (len = {length})"

    except Exception as e:
        # If anything goes wrong, show an error in the debugger.
        return f"Error formatting MoveList: {e}"