"""Fix corrupted Rust code: reverse val->let and var->let mut in non-string lines."""
import re

files = [
    'src/typeck.rs',
    'src/mir.rs',
    'src/codegen/llvm.rs',
    'src/parser.rs',
]

def is_rust_code_line(line):
    """Check if a line is Rust code (not inside a string literal)."""
    stripped = line.lstrip()
    # Lines starting with val/var that are Rust code (not inside a raw string)
    # Rust code lines typically have patterns like:
    #   let x = ...; or let mut x = ...;
    # String content would be indented and part of a string block

    # If line starts with val or var and has = or : patterns, it's Rust
    if stripped.startswith('val ') or stripped.startswith('var '):
        return True
    return False

def fix_line(line):
    """Fix a single line if it's corrupted Rust code."""
    stripped = line.lstrip()
    indent = line[:len(line) - len(stripped)]

    if stripped.startswith('var ') and not any(stripped.startswith(f'var {kw}') for kw in ['total_loss', 'i:', 'w ', 'x:', 'y:', 'z:', 'result']):
        # Check if this looks like Rust code (has semicolons, Rust types, etc.)
        if any(p in stripped for p in [' = ', ': ', '.new(', 'Command::', 'format!(', 'String::', 'fs::', 'vec![', '.unwrap', '.is_', '.contains', '.ends_with', '.push', 'usize', '0usize']):
            return indent + 'let mut ' + stripped[4:]

    if stripped.startswith('val '):
        # Check if this looks like Rust code
        if any(p in stripped for p in [' = ', ': ', '.new(', 'Command::', 'format!(', 'String::', 'fs::', '.unwrap', '.is_', '.contains', '.ends_with', 'match ', 'size ', '_ =']):
            return indent + 'let ' + stripped[4:]

    return line

for filepath in files:
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    # Track if we're inside a raw string r#"..."#
    in_raw_string = False
    changed = 0
    new_lines = []

    for line in lines:
        stripped = line.strip()

        # Track raw string boundaries
        if 'r#"' in stripped and not in_raw_string:
            in_raw_string = True
        if '"#' in stripped and in_raw_string:
            # Check if the raw string ends on this line
            # (simplistic: if "#" comes after r#" on same line, still in string)
            last_open = stripped.rfind('r#"')
            last_close = stripped.rfind('"#')
            if last_close > last_open + 3:
                in_raw_string = False
            elif last_open == -1:
                in_raw_string = False

        if not in_raw_string and (stripped.startswith('val ') or stripped.startswith('var ')):
            fixed = fix_line(line)
            if fixed != line:
                changed += 1
            new_lines.append(fixed)
        else:
            new_lines.append(line)

    if changed > 0:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.writelines(new_lines)
        print(f"Fixed {filepath}: {changed} lines restored")
    else:
        print(f"No changes needed in {filepath}")
