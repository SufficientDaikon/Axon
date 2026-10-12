"""Bulk-replace old Axon syntax inside string literals in .rs files."""
import re
import glob
import os

def replace_in_strings(content, old, new):
    """Replace old->new only inside string literals (both regular and raw strings)."""
    result = []
    i = 0
    n = len(content)
    while i < n:
        # Raw string r#"..."#
        if content[i:i+3] == 'r#"':
            end = content.find('"#', i + 3)
            if end == -1:
                result.append(content[i:])
                break
            s = content[i+3:end]
            s = s.replace(old, new)
            result.append('r#"' + s + '"#')
            i = end + 2
        # Regular string "..." (not preceded by unescaped backslash)
        elif content[i] == '"' and (i == 0 or content[i-1] != chr(92)):
            j = i + 1
            while j < n:
                if content[j] == chr(92):
                    j += 2
                    continue
                if content[j] == '"':
                    break
                j += 1
            s = content[i+1:j]
            s = s.replace(old, new)
            result.append('"' + s + '"')
            i = j + 1
        else:
            result.append(content[i])
            i += 1
    return ''.join(result)


files = (
    glob.glob('src/typeck.rs') + glob.glob('src/mir.rs') +
    glob.glob('src/codegen/llvm.rs') + glob.glob('src/fmt.rs') +
    glob.glob('src/doc.rs') + glob.glob('src/lint.rs') +
    glob.glob('src/tast.rs') + glob.glob('src/repl.rs') +
    glob.glob('src/lsp/server.rs') +
    glob.glob('src/parser.rs') +
    glob.glob('tests/type_tests.rs') + glob.glob('tests/codegen_tests.rs') +
    glob.glob('tests/integration_tests.rs') + glob.glob('tests/tooling_tests.rs') +
    glob.glob('tests/ai_framework_tests.rs') + glob.glob('tests/fuzz_tests.rs') +
    glob.glob('tests/stdlib_tests.rs')
)

total_changes = 0
for f in files:
    with open(f, 'r', encoding='utf-8') as fh:
        content = fh.read()
    original = content

    # Order matters! Do 'let mut' before 'let'
    content = replace_in_strings(content, 'let mut ', 'var ')
    content = replace_in_strings(content, 'let ', 'val ')
    content = replace_in_strings(content, ') -> ', '): ')
    content = replace_in_strings(content, 'struct ', 'model ')
    content = replace_in_strings(content, 'impl ', 'extend ')

    if content != original:
        with open(f, 'w', encoding='utf-8') as fh:
            fh.write(content)
        changes = sum(1 for a, b in zip(original.split('\n'), content.split('\n')) if a != b)
        print(f'  Updated {f}: ~{changes} lines changed')
        total_changes += changes

print(f'\nTotal: ~{total_changes} lines changed across {len(files)} files')
