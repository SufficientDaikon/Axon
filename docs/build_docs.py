#!/usr/bin/env python3
"""
build_docs.py  --  Assemble axon/docs/documentation.html
Reads CSS, JS, and all Markdown files, converts to a single-page HTML doc.
"""

import os
import re
import sys

DOCS_DIR = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# 1. Read helper
# ---------------------------------------------------------------------------
def read_file(path):
    """Read a file, return its text or None on failure."""
    try:
        with open(path, "r", encoding="utf-8") as f:
            return f.read()
    except Exception as e:
        print(f"  WARNING: Could not read {path}: {e}")
        return None

# ---------------------------------------------------------------------------
# 2. Markdown -> HTML converter
# ---------------------------------------------------------------------------

AXON_KEYWORDS = (
    "fn", "val", "var", "mut", "return", "if", "else", "for", "in", "while",
    "match", "model", "enum", "extend", "trait", "pub", "use", "import",
    "true", "false", "type", "where", "const", "static", "async", "await",
    "move", "ref", "as", "is", "from", "with", "defer", "grad", "loop",
    "break", "continue", "self", "super",
)

AXON_TYPES = (
    "Tensor", "int", "float", "bool", "string", "void", "Self", "Option",
    "Result", "i32", "i64", "f32", "f64", "u8", "u16", "u32", "u64",
    "Float32", "Float64", "Int32", "Int64", "String", "Vec", "Array",
    "Map", "Set", "Shape", "Device", "Gradient", "Model", "Layer",
    "Optimizer", "Loss",
)

def html_escape(text):
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")

def slugify(text):
    text = text.lower().strip()
    text = re.sub(r'[^\w\s-]', '', text)
    text = re.sub(r'[\s_]+', '-', text)
    text = re.sub(r'^-+|-+$', '', text)
    return text

def highlight_axon(code_text):
    """
    Apply syntax highlighting to HTML-escaped Axon code using a single-pass
    tokenizer.  This avoids the problem of later regex passes matching inside
    already-inserted <span> tags.

    Strategy: scan the text left-to-right with a combined regex that tries
    every token type at each position.  Matched tokens get wrapped in spans;
    unmatched characters pass through verbatim.
    """
    kw_set = set(AXON_KEYWORDS)
    typ_set = set(AXON_TYPES)

    # Build one big alternation.  Order matters -- first match wins.
    # Group names encode the token type.
    token_re = re.compile('|'.join([
        r'(?P<comment>//[^\n]*)',                              # // comment
        r'(?P<string>&quot;(?:\\.|[^&])*?&quot;)',             # "string" (HTML-escaped quotes)
        r'(?P<string2>"(?:\\.|[^"\\])*?")',                    # "string" (literal quotes)
        r'(?P<number>\b\d+(?:\.\d+)?(?:e[+-]?\d+)?\b)',       # 42, 3.14, 1e-5
        r'(?P<word>\b[a-zA-Z_]\w*\b)',                         # identifier / keyword / type
        r'(?P<op>==|!=|&gt;=|&lt;=|=&gt;|\|&gt;|\.\.|\|'
        r'|&amp;&amp;|\|\||&gt;|&lt;|[=+\-*/!@&|:])',          # operators
    ]))

    parts = []
    last_end = 0

    for m in token_re.finditer(code_text):
        # Append any text between the last match and this one verbatim
        if m.start() > last_end:
            parts.append(code_text[last_end:m.start()])

        text = m.group()

        if m.group('comment'):
            parts.append(f'<span class="cmt">{text}</span>')
        elif m.group('string') or m.group('string2'):
            parts.append(f'<span class="str">{text}</span>')
        elif m.group('number'):
            parts.append(f'<span class="num">{text}</span>')
        elif m.group('word'):
            # Decide: keyword, type, or function call (peek for `(` after the word)
            after = code_text[m.end():m.end()+1]
            if text in kw_set:
                parts.append(f'<span class="kw">{text}</span>')
            elif text in typ_set:
                parts.append(f'<span class="typ">{text}</span>')
            elif after == '(':
                parts.append(f'<span class="fn">{text}</span>')
            else:
                parts.append(text)
        elif m.group('op'):
            parts.append(f'<span class="op">{text}</span>')
        else:
            parts.append(text)

        last_end = m.end()

    # Append anything remaining after the last match
    if last_end < len(code_text):
        parts.append(code_text[last_end:])

    return ''.join(parts)


def convert_markdown(md_text):
    """Convert a markdown string to HTML. Best-effort, not perfect."""
    lines = md_text.split('\n')
    html_parts = []
    i = 0

    def process_inline(text):
        """Handle inline formatting: bold, italic, inline code, links."""
        # Inline code first (so other patterns don't mess with code content)
        result = []
        parts = text.split('`')
        for idx, part in enumerate(parts):
            if idx % 2 == 1:
                # Inside backticks
                result.append('<code>' + html_escape(part) + '</code>')
            else:
                result.append(part)
        text = ''.join(result)

        # Links: [text](url)
        text = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', text)

        # Bold: **text**  or __text__
        text = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', text)
        text = re.sub(r'__(.+?)__', r'<strong>\1</strong>', text)

        # Italic: *text* or _text_  (but not inside words for _)
        text = re.sub(r'(?<!\w)\*(.+?)\*(?!\w)', r'<em>\1</em>', text)
        text = re.sub(r'(?<!\w)_(.+?)_(?!\w)', r'<em>\1</em>', text)

        return text

    while i < len(lines):
        line = lines[i]

        # --- Blank line ---
        if line.strip() == '':
            i += 1
            continue

        # --- Headings ---
        heading_match = re.match(r'^(#{1,6})\s+(.*)', line)
        if heading_match:
            level = len(heading_match.group(1))
            text = heading_match.group(2).strip()
            tag = f'h{level}'
            slug = slugify(text)
            inline_text = process_inline(text)
            if level in (2, 3):
                html_parts.append(f'<{tag} id="{slug}">{inline_text}</{tag}>')
            else:
                html_parts.append(f'<{tag}>{inline_text}</{tag}>')
            i += 1
            continue

        # --- Horizontal rule ---
        if re.match(r'^(-{3,}|_{3,}|\*{3,})\s*$', line):
            html_parts.append('<hr>')
            i += 1
            continue

        # --- Code block ---
        code_match = re.match(r'^```(\w*)', line)
        if code_match:
            lang = code_match.group(1)
            code_lines = []
            i += 1
            while i < len(lines) and not lines[i].startswith('```'):
                code_lines.append(lines[i])
                i += 1
            if i < len(lines):
                i += 1  # skip closing ```
            code_content = html_escape('\n'.join(code_lines))
            # Apply syntax highlighting for axon code (or no-lang blocks)
            if lang in ('axon', 'ax', ''):
                code_content = highlight_axon(code_content)
            lang_label = ''
            if lang:
                lang_label = f'<span class="code-block-lang">{html_escape(lang)}</span>'
            html_parts.append(
                f'<div class="code-block">{lang_label}'
                f'<pre><code>{code_content}</code></pre></div>'
            )
            continue

        # --- Table ---
        if '|' in line and re.match(r'^\s*\|', line):
            table_lines = []
            while i < len(lines) and '|' in lines[i] and lines[i].strip():
                table_lines.append(lines[i])
                i += 1
            if len(table_lines) >= 2:
                html_parts.append(convert_table(table_lines, process_inline))
            continue

        # --- Blockquote ---
        if line.startswith('>'):
            bq_lines = []
            while i < len(lines) and (lines[i].startswith('>') or (lines[i].strip() != '' and i > 0 and lines[i-1].startswith('>'))):
                stripped = re.sub(r'^>\s?', '', lines[i])
                bq_lines.append(stripped)
                i += 1
                # Don't continue into non-quote lines
                if i < len(lines) and not lines[i].startswith('>') and lines[i].strip() == '':
                    break
            bq_html = process_inline('\n'.join(bq_lines))
            # Split into paragraphs by blank lines
            bq_paragraphs = re.split(r'\n{2,}', bq_html)
            inner = ''.join(f'<p>{p.strip()}</p>' for p in bq_paragraphs if p.strip())
            if not inner:
                inner = f'<p>{bq_html}</p>'
            html_parts.append(f'<blockquote>{inner}</blockquote>')
            continue

        # --- Unordered list ---
        if re.match(r'^[\s]*[-*+]\s', line):
            list_items = []
            while i < len(lines) and (re.match(r'^[\s]*[-*+]\s', lines[i]) or (lines[i].startswith('  ') and lines[i].strip())):
                item_match = re.match(r'^[\s]*[-*+]\s+(.*)', lines[i])
                if item_match:
                    list_items.append(item_match.group(1))
                elif list_items:
                    # Continuation line
                    list_items[-1] += ' ' + lines[i].strip()
                i += 1
            html_parts.append('<ul>')
            for item in list_items:
                html_parts.append(f'<li>{process_inline(item)}</li>')
            html_parts.append('</ul>')
            continue

        # --- Ordered list ---
        if re.match(r'^[\s]*\d+\.\s', line):
            list_items = []
            while i < len(lines) and (re.match(r'^[\s]*\d+\.\s', lines[i]) or (lines[i].startswith('  ') and lines[i].strip())):
                item_match = re.match(r'^[\s]*\d+\.\s+(.*)', lines[i])
                if item_match:
                    list_items.append(item_match.group(1))
                elif list_items:
                    list_items[-1] += ' ' + lines[i].strip()
                i += 1
            html_parts.append('<ol>')
            for item in list_items:
                html_parts.append(f'<li>{process_inline(item)}</li>')
            html_parts.append('</ol>')
            continue

        # --- Paragraph (default) ---
        para_lines = []
        while i < len(lines) and lines[i].strip() != '' and not re.match(r'^(#{1,6}\s|```|>|\s*[-*+]\s|\s*\d+\.\s|(-{3,}|_{3,}|\*{3,})\s*$)', lines[i]):
            para_lines.append(lines[i])
            i += 1
        if para_lines:
            para_text = ' '.join(para_lines)
            html_parts.append(f'<p>{process_inline(para_text)}</p>')

    return '\n'.join(html_parts)


def convert_table(table_lines, process_inline):
    """Convert markdown table lines to HTML table."""
    def parse_row(row_line):
        cells = row_line.strip().strip('|').split('|')
        return [c.strip() for c in cells]

    rows = [parse_row(line) for line in table_lines]

    # Check if row 1 is a separator row (---, :--:, etc.)
    has_header = False
    if len(rows) >= 2:
        sep_row = rows[1]
        if all(re.match(r'^[:\-\s]+$', cell) for cell in sep_row if cell):
            has_header = True

    html = '<table>'
    start = 0
    if has_header:
        html += '<thead><tr>'
        for cell in rows[0]:
            html += f'<th>{process_inline(cell)}</th>'
        html += '</tr></thead>'
        start = 2  # skip header and separator
    html += '<tbody>'
    for row in rows[start:]:
        html += '<tr>'
        for cell in row:
            html += f'<td>{process_inline(cell)}</td>'
        html += '</tr>'
    html += '</tbody></table>'
    return html


# ---------------------------------------------------------------------------
# 3. Navigation structure
# ---------------------------------------------------------------------------

SECTIONS = [
    ("Guide", "guide", [
        ("getting-started", "Getting Started"),
        ("language-tour", "Language Tour"),
        ("tensors", "Tensors"),
        ("ownership-borrowing", "Ownership & Borrowing"),
        ("error-handling", "Error Handling"),
        ("modules-packages", "Modules & Packages"),
        ("gpu-programming", "GPU Programming"),
    ]),
    ("Tutorials", "tutorial", [
        ("01-hello-tensor", "Hello Tensor"),
        ("02-linear-regression", "Linear Regression"),
        ("03-mnist-classifier", "MNIST Classifier"),
        ("04-transformer", "Transformer"),
        ("05-structs-and-enums", "Structs & Enums"),
        ("06-error-handling", "Error Handling"),
    ]),
    ("Migration", "migration", [
        ("from-pytorch", "From PyTorch"),
        ("from-python", "From Python"),
        ("from-rust", "From Rust"),
    ]),
    ("Reference", "reference", [
        ("cli-reference", "CLI Reference"),
        ("compiler-errors", "Compiler Errors"),
    ]),
    ("Internals", "internals", [
        ("architecture", "Architecture"),
        ("security-audit", "Security Audit"),
        ("contributing", "Contributing"),
    ]),
]


# ---------------------------------------------------------------------------
# 4. Build the page
# ---------------------------------------------------------------------------

def build_sidebar_html():
    """Generate the sidebar navigation HTML."""
    parts = []
    for group_label, category, pages in SECTIONS:
        parts.append(f'      <div class="sidebar-group nav-group">')
        parts.append(f'        <div class="sidebar-group-header nav-group-header">')
        parts.append(f'          <span>{group_label}</span>')
        parts.append(f'          <span class="chevron">&#9662;</span>')
        parts.append(f'        </div>')
        parts.append(f'        <ul class="sidebar-group-items">')
        for slug, title in pages:
            page_id = f'{category}/{slug}'
            parts.append(
                f'          <li><a href="#{page_id}" class="nav-link" '
                f'data-page="{page_id}">{title}</a></li>'
            )
        parts.append(f'        </ul>')
        parts.append(f'      </div>')
    return '\n'.join(parts)


def build_articles_html():
    """Read all markdown files and convert to article elements."""
    articles = []
    for group_label, category, pages in SECTIONS:
        for slug, title in pages:
            page_id = f'{category}/{slug}'
            md_path = os.path.join(DOCS_DIR, category, f'{slug}.md')
            md_content = read_file(md_path)
            if md_content is None:
                print(f"  SKIP: {md_path} (file not found or unreadable)")
                # Create a stub article
                inner_html = f'<h1>{html_escape(title)}</h1>\n<p>Content not available.</p>'
            else:
                inner_html = convert_markdown(md_content)
            articles.append(
                f'    <article data-page="{page_id}" class="doc-page">\n'
                f'{inner_html}\n'
                f'    </article>'
            )
    return '\n\n'.join(articles)


def build_documentation_html():
    """Assemble the full documentation.html file."""
    css_content = read_file(os.path.join(DOCS_DIR, 'docs.css'))
    js_content = read_file(os.path.join(DOCS_DIR, 'docs.js'))

    if css_content is None:
        css_content = '/* docs.css not found */'
    if js_content is None:
        js_content = '/* docs.js not found */'

    sidebar_html = build_sidebar_html()
    articles_html = build_articles_html()

    page = f'''<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Axon Documentation</title>
  <meta name="description" content="Axon programming language documentation - guides, tutorials, reference, and internals.">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
  <style>
{css_content}
  </style>
</head>
<body>

  <!-- ══════════════════════════════════════════════════════════════════
       NAVIGATION BAR
       ══════════════════════════════════════════════════════════════════ -->
  <nav class="nav">
    <div class="nav-inner">
      <a href="#" class="nav-logo">
        <div class="nav-logo-icon">Ax</div>
        Axon Docs
      </a>
      <ul class="nav-links">
        <li><a href="#guide/getting-started" class="active">Guide</a></li>
        <li><a href="#tutorial/01-hello-tensor">Tutorials</a></li>
        <li><a href="#reference/cli-reference">Reference</a></li>
        <li><a href="#internals/architecture">Internals</a></li>
      </ul>
      <div class="nav-search">
        <span class="nav-search-icon">&#128269;</span>
        <input type="text" id="search-input" class="search-input" placeholder="Search docs..." autocomplete="off">
        <span class="nav-search-kbd">Ctrl+K</span>
      </div>
      <button class="nav-mobile-toggle" id="sidebar-toggle" aria-label="Toggle sidebar">&#9776;</button>
    </div>
  </nav>

  <!-- ══════════════════════════════════════════════════════════════════
       LAYOUT SHELL
       ══════════════════════════════════════════════════════════════════ -->
  <div class="docs-layout">

    <!-- ── Sidebar ──────────────────────────────────────────────────── -->
    <aside class="sidebar" id="sidebar">
{sidebar_html}
    </aside>

    <div class="sidebar-overlay"></div>

    <!-- ── Main Content ─────────────────────────────────────────────── -->
    <main class="docs-content">
      <div class="docs-content-inner">

{articles_html}

      </div>
    </main>

    <!-- ── Table of Contents (right sticky) ─────────────────────────── -->
    <div class="toc" id="toc"></div>

  </div>

  <!-- ══════════════════════════════════════════════════════════════════
       JAVASCRIPT
       ══════════════════════════════════════════════════════════════════ -->
  <script>
{js_content}
  </script>

</body>
</html>'''

    return page


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("Building documentation.html ...")
    html = build_documentation_html()

    out_path = os.path.join(DOCS_DIR, 'documentation.html')
    try:
        with open(out_path, 'w', encoding='utf-8') as f:
            f.write(html)
    except Exception as e:
        print(f"ERROR: Could not write {out_path}: {e}")
        sys.exit(1)

    size_kb = os.path.getsize(out_path) / 1024
    print(f"  Output: {out_path}")
    print(f"  Size:   {size_kb:.1f} KB")
    print("Done!")


if __name__ == '__main__':
    main()
