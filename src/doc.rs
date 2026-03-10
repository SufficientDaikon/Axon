// doc.rs — Documentation generator for Axon source files

use crate::ast::*;

pub struct DocGenerator {
    items: Vec<DocItem>,
}

#[derive(Debug, Clone)]
pub struct DocItem {
    pub name: String,
    pub kind: DocItemKind,
    pub doc_comment: Option<String>,
    pub signature: String,
    pub children: Vec<DocItem>,
    pub visibility: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Field,
    Method,
    Variant,
}

impl DocItemKind {
    fn label(&self) -> &'static str {
        match self {
            DocItemKind::Function => "Function",
            DocItemKind::Struct => "Struct",
            DocItemKind::Enum => "Enum",
            DocItemKind::Trait => "Trait",
            DocItemKind::Impl => "Impl",
            DocItemKind::Module => "Module",
            DocItemKind::Field => "Field",
            DocItemKind::Method => "Method",
            DocItemKind::Variant => "Variant",
        }
    }
}

impl DocGenerator {
    pub fn new() -> Self {
        DocGenerator { items: Vec::new() }
    }

    /// Generate HTML documentation from Axon source.
    pub fn generate(source: &str, filename: &str) -> String {
        let (program, errors) = crate::parse_source(source, filename);
        if !errors.is_empty() {
            return format!("<!-- parse errors: {} -->", errors.len());
        }
        let mut gen = DocGenerator::new();
        let doc_comments = gen.extract_doc_comments(source);
        gen.collect_items(&program, &doc_comments);
        gen.render_html(filename)
    }

    /// Generate Markdown documentation from Axon source.
    pub fn generate_markdown(source: &str, filename: &str) -> String {
        let (program, errors) = crate::parse_source(source, filename);
        if !errors.is_empty() {
            return format!("<!-- parse errors: {} -->\n", errors.len());
        }
        let mut gen = DocGenerator::new();
        let doc_comments = gen.extract_doc_comments(source);
        gen.collect_items(&program, &doc_comments);
        gen.render_markdown(filename)
    }

    /// Extract doc comments (lines starting with `///`) and map line number -> comment text.
    fn extract_doc_comments(&self, source: &str) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        let mut current_comment = String::new();
        let mut comment_start_line = 0;

        for (idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(doc) = trimmed.strip_prefix("///") {
                if current_comment.is_empty() {
                    comment_start_line = idx + 1;
                } else {
                    current_comment.push('\n');
                }
                current_comment.push_str(doc.strip_prefix(' ').unwrap_or(doc));
            } else {
                if !current_comment.is_empty() {
                    // The doc comment applies to the next non-comment line
                    result.push((idx + 1, current_comment.clone()));
                    current_comment.clear();
                }
                let _ = comment_start_line;
            }
        }
        // Trailing doc comment
        if !current_comment.is_empty() {
            result.push((source.lines().count() + 1, current_comment));
        }

        result
    }

    fn find_doc_comment(doc_comments: &[(usize, String)], item_line: usize) -> Option<String> {
        // Find the doc comment that immediately precedes this item's line
        for (line, comment) in doc_comments {
            if *line == item_line {
                return Some(comment.clone());
            }
        }
        None
    }

    fn collect_items(&mut self, program: &Program, doc_comments: &[(usize, String)]) {
        for item in &program.items {
            if item.visibility == Visibility::Private {
                continue;
            }
            if let Some(doc_item) = self.collect_item(item, doc_comments) {
                self.items.push(doc_item);
            }
        }
    }

    fn collect_item(&self, item: &Item, doc_comments: &[(usize, String)]) -> Option<DocItem> {
        let is_public = item.visibility == Visibility::Public;
        let line = item.span.start.line;
        let doc = Self::find_doc_comment(doc_comments, line);

        match &item.kind {
            ItemKind::Function(f) => Some(DocItem {
                name: f.name.clone(),
                kind: DocItemKind::Function,
                doc_comment: doc,
                signature: self.function_signature(f),
                children: Vec::new(),
                visibility: is_public,
            }),
            ItemKind::Struct(s) => {
                let children: Vec<DocItem> = s.fields.iter()
                    .filter(|f| f.visibility == Visibility::Public)
                    .map(|f| DocItem {
                        name: f.name.clone(),
                        kind: DocItemKind::Field,
                        doc_comment: None,
                        signature: format!("{}: {}", f.name, self.type_expr_str(&f.ty)),
                        children: Vec::new(),
                        visibility: true,
                    })
                    .collect();

                Some(DocItem {
                    name: s.name.clone(),
                    kind: DocItemKind::Struct,
                    doc_comment: doc,
                    signature: format!("struct {}{}", s.name, self.generics_str(&s.generics)),
                    children,
                    visibility: is_public,
                })
            }
            ItemKind::Enum(e) => {
                let children: Vec<DocItem> = e.variants.iter()
                    .map(|v| {
                        let sig = match &v.fields {
                            EnumVariantKind::Unit => v.name.clone(),
                            EnumVariantKind::Tuple(types) => {
                                let types_str: Vec<String> = types.iter()
                                    .map(|t| self.type_expr_str(t))
                                    .collect();
                                format!("{}({})", v.name, types_str.join(", "))
                            }
                            EnumVariantKind::Struct(fields) => {
                                let fields_str: Vec<String> = fields.iter()
                                    .map(|f| format!("{}: {}", f.name, self.type_expr_str(&f.ty)))
                                    .collect();
                                format!("{} {{ {} }}", v.name, fields_str.join(", "))
                            }
                        };
                        DocItem {
                            name: v.name.clone(),
                            kind: DocItemKind::Variant,
                            doc_comment: None,
                            signature: sig,
                            children: Vec::new(),
                            visibility: true,
                        }
                    })
                    .collect();

                Some(DocItem {
                    name: e.name.clone(),
                    kind: DocItemKind::Enum,
                    doc_comment: doc,
                    signature: format!("enum {}{}", e.name, self.generics_str(&e.generics)),
                    children,
                    visibility: is_public,
                })
            }
            ItemKind::Trait(t) => {
                let children: Vec<DocItem> = t.items.iter()
                    .filter_map(|i| self.collect_item(i, doc_comments))
                    .collect();

                Some(DocItem {
                    name: t.name.clone(),
                    kind: DocItemKind::Trait,
                    doc_comment: doc,
                    signature: format!("trait {}{}", t.name, self.generics_str(&t.generics)),
                    children,
                    visibility: is_public,
                })
            }
            ItemKind::Impl(imp) => {
                let children: Vec<DocItem> = imp.items.iter()
                    .filter_map(|i| {
                        let mut d = self.collect_item(i, doc_comments)?;
                        d.kind = DocItemKind::Method;
                        Some(d)
                    })
                    .collect();

                let name = self.type_expr_str(&imp.type_name);
                let sig = if let Some(trait_name) = &imp.trait_name {
                    format!("impl {} for {}", self.type_expr_str(trait_name), name)
                } else {
                    format!("impl {}", name)
                };

                Some(DocItem {
                    name,
                    kind: DocItemKind::Impl,
                    doc_comment: doc,
                    signature: sig,
                    children,
                    visibility: is_public,
                })
            }
            ItemKind::Module(m) => Some(DocItem {
                name: m.name.clone(),
                kind: DocItemKind::Module,
                doc_comment: doc,
                signature: format!("mod {}", m.name),
                children: Vec::new(),
                visibility: is_public,
            }),
            _ => None,
        }
    }

    fn function_signature(&self, f: &FnDecl) -> String {
        let mut sig = format!("fn {}{}", f.name, self.generics_str(&f.generics));
        sig.push('(');
        let params: Vec<String> = f.params.iter().map(|p| {
            match &p.kind {
                FnParamKind::Typed { name, ty, .. } => format!("{}: {}", name, self.type_expr_str(ty)),
                FnParamKind::SelfOwned => "self".to_string(),
                FnParamKind::SelfRef => "&self".to_string(),
                FnParamKind::SelfMutRef => "&mut self".to_string(),
            }
        }).collect();
        sig.push_str(&params.join(", "));
        sig.push(')');
        if let Some(ret) = &f.return_type {
            sig.push_str(" -> ");
            sig.push_str(&self.type_expr_str(ret));
        }
        sig
    }

    fn generics_str(&self, generics: &[GenericParam]) -> String {
        if generics.is_empty() {
            return String::new();
        }
        let parts: Vec<String> = generics.iter().map(|g| {
            if g.bounds.is_empty() {
                g.name.clone()
            } else {
                let bounds: Vec<String> = g.bounds.iter().map(|b| self.type_expr_str(b)).collect();
                format!("{}: {}", g.name, bounds.join(" + "))
            }
        }).collect();
        format!("<{}>", parts.join(", "))
    }

    fn type_expr_str(&self, ty: &TypeExpr) -> String {
        match &ty.kind {
            TypeExprKind::Named(name) => name.clone(),
            TypeExprKind::Path(parts) => parts.join("::"),
            TypeExprKind::Generic { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| match a {
                    TypeArg::Type(t) => self.type_expr_str(t),
                    TypeArg::Shape(dims) => {
                        let dim_strs: Vec<String> = dims.iter().map(|d| match d {
                            ShapeDim::Constant(n) => n.to_string(),
                            ShapeDim::Dynamic => "?".to_string(),
                            ShapeDim::Named(n) => n.clone(),
                        }).collect();
                        format!("[{}]", dim_strs.join(", "))
                    }
                }).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
            TypeExprKind::Tensor { dtype, shape } => {
                let dim_strs: Vec<String> = shape.iter().map(|d| match d {
                    ShapeDim::Constant(n) => n.to_string(),
                    ShapeDim::Dynamic => "?".to_string(),
                    ShapeDim::Named(n) => n.clone(),
                }).collect();
                format!("Tensor<{}, [{}]>", self.type_expr_str(dtype), dim_strs.join(", "))
            }
            TypeExprKind::Reference { mutable, inner } => {
                if *mutable {
                    format!("&mut {}", self.type_expr_str(inner))
                } else {
                    format!("&{}", self.type_expr_str(inner))
                }
            }
            TypeExprKind::Function { params, return_type } => {
                let params_str: Vec<String> = params.iter().map(|p| self.type_expr_str(p)).collect();
                format!("fn({}) -> {}", params_str.join(", "), self.type_expr_str(return_type))
            }
            TypeExprKind::Tuple(types) => {
                let types_str: Vec<String> = types.iter().map(|t| self.type_expr_str(t)).collect();
                format!("({})", types_str.join(", "))
            }
            TypeExprKind::Array { element, size } => {
                format!("[{}; {}]", self.type_expr_str(element), size)
            }
            TypeExprKind::Inferred => "_".to_string(),
        }
    }

    fn render_html(&self, filename: &str) -> String {
        let module_name = filename.trim_end_matches(".axon");
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str(&format!("<title>{} — Axon Documentation</title>\n", Self::html_escape(module_name)));
        html.push_str("<style>\n");
        html.push_str(DOCS_CSS);
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Header
        html.push_str(&format!("<h1>Module <code>{}</code></h1>\n", Self::html_escape(module_name)));

        // Table of contents
        if !self.items.is_empty() {
            html.push_str("<h2>Contents</h2>\n<ul class=\"toc\">\n");
            for item in &self.items {
                html.push_str(&format!(
                    "<li><a href=\"#{}\">{} <code>{}</code></a></li>\n",
                    Self::html_escape(&item.name),
                    item.kind.label(),
                    Self::html_escape(&item.name),
                ));
            }
            html.push_str("</ul>\n");
        }

        // Items
        for item in &self.items {
            html.push_str(&self.render_item(item));
        }

        html.push_str("</body>\n</html>\n");
        html
    }

    fn render_markdown(&self, filename: &str) -> String {
        let module_name = filename.trim_end_matches(".axon");
        let mut md = String::new();

        md.push_str(&format!("# Module `{}`\n\n", module_name));

        // Table of contents
        if !self.items.is_empty() {
            md.push_str("## Contents\n\n");
            for item in &self.items {
                md.push_str(&format!("- {} `{}`\n", item.kind.label(), item.name));
            }
            md.push('\n');
        }

        // Items
        for item in &self.items {
            md.push_str(&self.render_item_markdown(item));
        }

        md
    }

    fn render_item_markdown(&self, item: &DocItem) -> String {
        let mut md = String::new();

        md.push_str(&format!("## {} `{}`\n\n", item.kind.label(), item.name));
        md.push_str(&format!("```axon\n{}\n```\n\n", item.signature));

        if let Some(ref doc) = item.doc_comment {
            md.push_str(doc);
            md.push_str("\n\n");
        }

        if !item.children.is_empty() {
            for child in &item.children {
                md.push_str(&format!("- **{}** `{}`\n", child.kind.label(), child.signature));
            }
            md.push('\n');
        }

        md
    }

    fn render_item(&self, item: &DocItem) -> String {
        let mut html = String::new();

        html.push_str(&format!(
            "<div class=\"doc-item\" id=\"{}\">\n",
            Self::html_escape(&item.name),
        ));
        html.push_str(&format!(
            "<h3><span class=\"kind\">{}</span> <code class=\"signature\">{}</code></h3>\n",
            item.kind.label(),
            Self::html_escape(&item.signature),
        ));

        if let Some(ref doc) = item.doc_comment {
            html.push_str("<div class=\"doc-comment\">\n");
            html.push_str(&self.markdown_to_html(doc));
            html.push_str("</div>\n");
        }

        if !item.children.is_empty() {
            html.push_str("<div class=\"children\">\n");
            for child in &item.children {
                html.push_str(&format!(
                    "<div class=\"child-item\"><span class=\"kind\">{}</span> <code>{}</code></div>\n",
                    child.kind.label(),
                    Self::html_escape(&child.signature),
                ));
            }
            html.push_str("</div>\n");
        }

        html.push_str("</div>\n");
        html
    }

    #[allow(dead_code)]
    fn render_signature(&self, sig: &str) -> String {
        format!("<code class=\"signature\">{}</code>", Self::html_escape(sig))
    }

    fn markdown_to_html(&self, md: &str) -> String {
        let mut html = String::new();
        let mut in_code_block = false;

        for line in md.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    html.push_str("</code></pre>\n");
                    in_code_block = false;
                } else {
                    let lang = line.strip_prefix("```").unwrap_or("").trim();
                    if lang.is_empty() {
                        html.push_str("<pre><code>");
                    } else {
                        html.push_str(&format!("<pre><code class=\"language-{}\">", Self::html_escape(lang)));
                    }
                    in_code_block = true;
                }
            } else if in_code_block {
                html.push_str(&Self::html_escape(line));
                html.push('\n');
            } else if line.is_empty() {
                html.push_str("<br>\n");
            } else {
                // Inline code
                let processed = Self::process_inline_code(line);
                html.push_str(&format!("<p>{}</p>\n", processed));
            }
        }

        if in_code_block {
            html.push_str("</code></pre>\n");
        }

        html
    }

    fn process_inline_code(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        let mut in_code = false;

        while let Some(c) = chars.next() {
            if c == '`' {
                if in_code {
                    result.push_str("</code>");
                    in_code = false;
                } else {
                    result.push_str("<code>");
                    in_code = true;
                }
            } else {
                match c {
                    '<' => result.push_str("&lt;"),
                    '>' => result.push_str("&gt;"),
                    '&' => result.push_str("&amp;"),
                    '"' => result.push_str("&quot;"),
                    _ => result.push(c),
                }
            }
        }

        if in_code {
            result.push_str("</code>");
        }

        result
    }

    fn html_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}

const DOCS_CSS: &str = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
    color: #333;
    line-height: 1.6;
}
h1 { color: #2a5db0; border-bottom: 2px solid #2a5db0; padding-bottom: 0.5rem; }
h2 { color: #444; margin-top: 2rem; }
h3 { margin-top: 1.5rem; }
code { background: #f4f4f4; padding: 0.1em 0.3em; border-radius: 3px; font-size: 0.9em; }
pre code { display: block; padding: 1em; overflow-x: auto; }
.doc-item { border-left: 3px solid #2a5db0; padding-left: 1rem; margin: 1.5rem 0; }
.kind { color: #888; font-size: 0.85em; text-transform: uppercase; }
.signature { font-size: 1.05em; font-weight: bold; }
.doc-comment { margin: 0.5rem 0; color: #555; }
.children { margin-left: 1.5rem; }
.child-item { margin: 0.3rem 0; }
.toc { list-style: none; padding: 0; }
.toc li { margin: 0.3rem 0; }
.toc a { text-decoration: none; color: #2a5db0; }
.toc a:hover { text-decoration: underline; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_comments() {
        let source = "/// This is a doc comment\n/// Second line\npub fn foo() {}";
        let gen = DocGenerator::new();
        let comments = gen.extract_doc_comments(source);
        assert!(!comments.is_empty());
        assert!(comments[0].1.contains("This is a doc comment"));
        assert!(comments[0].1.contains("Second line"));
    }

    #[test]
    fn test_generate_function_doc() {
        let src = "/// Adds two numbers.\npub fn add(a: Int32, b: Int32) -> Int32 { return a + b; }";
        let html = DocGenerator::generate(src, "math.axon");
        assert!(html.contains("add"));
        assert!(html.contains("Function"));
        assert!(html.contains("fn add(a: Int32, b: Int32) -&gt; Int32"));
    }

    #[test]
    fn test_generate_struct_doc() {
        let src = "/// A 2D point.\npub struct Point { pub x: Float64, pub y: Float64, }";
        let html = DocGenerator::generate(src, "geom.axon");
        assert!(html.contains("Point"));
        assert!(html.contains("Struct"));
        assert!(html.contains("A 2D point."));
    }

    #[test]
    fn test_html_output() {
        let src = "pub fn hello() -> String { return \"world\"; }";
        let html = DocGenerator::generate(src, "test.axon");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>"));
        assert!(html.contains("</html>"));
        assert!(html.contains("Module"));
    }

    #[test]
    fn test_no_private_items() {
        let src = "fn private_fn() {}\npub fn public_fn() {}";
        let html = DocGenerator::generate(src, "test.axon");
        assert!(!html.contains("private_fn"));
        assert!(html.contains("public_fn"));
    }

    #[test]
    fn test_generate_enum_doc() {
        let src = "/// Color options.\npub enum Color { Red, Green, Blue, }";
        let html = DocGenerator::generate(src, "colors.axon");
        assert!(html.contains("Color"));
        assert!(html.contains("Enum"));
        assert!(html.contains("Red"));
        assert!(html.contains("Green"));
        assert!(html.contains("Blue"));
    }

    #[test]
    fn test_markdown_inline_code() {
        let gen = DocGenerator::new();
        let result = gen.markdown_to_html("Use `foo()` to call");
        assert!(result.contains("<code>foo()</code>"));
    }

    #[test]
    fn test_generate_markdown_output() {
        let src = "/// Adds two numbers.\npub fn add(a: Int32, b: Int32) -> Int32 { return a + b; }";
        let md = DocGenerator::generate_markdown(src, "math.axon");
        assert!(md.contains("# Module `math`"), "should have module header");
        assert!(md.contains("## Function `add`"), "should have function section");
        assert!(md.contains("```axon"), "should have code block");
        assert!(md.contains("fn add(a: Int32, b: Int32) -> Int32"), "should have signature");
        assert!(md.contains("Adds two numbers."), "should have doc comment");
    }

    #[test]
    fn test_generate_markdown_struct() {
        let src = "/// A point.\npub struct Point { pub x: Float64, pub y: Float64, }";
        let md = DocGenerator::generate_markdown(src, "geom.axon");
        assert!(md.contains("# Module `geom`"));
        assert!(md.contains("## Struct `Point`"));
        assert!(md.contains("A point."));
        assert!(md.contains("**Field**"));
    }

    #[test]
    fn test_generate_markdown_enum() {
        let src = "pub enum Color { Red, Green, Blue, }";
        let md = DocGenerator::generate_markdown(src, "colors.axon");
        assert!(md.contains("## Enum `Color`"));
        assert!(md.contains("Red"));
        assert!(md.contains("Green"));
        assert!(md.contains("Blue"));
    }
}
