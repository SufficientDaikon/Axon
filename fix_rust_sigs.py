"""Fix Rust fn signatures where ') -> ' was incorrectly replaced with '): '."""
import re

files = ['src/codegen/llvm.rs', 'src/doc.rs']

for filepath in files:
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    changed = 0
    new_lines = []
    in_raw = False

    for line in lines:
        # Track raw strings
        if 'r#"' in line and not in_raw:
            in_raw = True
        if '"#' in line and in_raw:
            in_raw = False
            new_lines.append(line)
            continue

        if in_raw:
            new_lines.append(line)
            continue

        # Fix Rust fn signatures: ): Type -> ) -> Type
        # Match patterns like "): Result", "): String", "): &str", "): u32", "): usize"
        if '): ' in line and '"' not in line:
            fixed = re.sub(r'\):\s+', ') -> ', line)
            if fixed != line:
                changed += 1
                new_lines.append(fixed)
                continue

        new_lines.append(line)

    if changed > 0:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.writelines(new_lines)
        print(f"Fixed {filepath}: {changed} lines")
