// lsp/handlers.rs — LSP request/notification handlers

use super::protocol::*;
use super::server::{DocumentState, LspServer};
use crate::ast::{Item, ItemKind};
use crate::error::Severity;

// ═══════════════════════════════════════════════════════════════
// Lifecycle handlers
// ═══════════════════════════════════════════════════════════════

impl LspServer {
    pub fn handle_initialize(&mut self, id: serde_json::Value, _params: serde_json::Value) {
        self.initialized = true;

        let result = InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: 1, // Full
                completion_provider: Some(CompletionOptions {
                    trigger_characters: vec![".".to_string(), ":".to_string()],
                }),
                hover_provider: true,
                definition_provider: true,
                document_formatting_provider: true,
                document_symbol_provider: true,
            },
            server_info: Some(ServerInfo {
                name: "axon-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        };

        self.send_response(id, serde_json::to_value(result).unwrap());
    }

    pub fn handle_initialized(&mut self) {
        // Client acknowledged initialization — nothing to do
    }

    pub fn handle_shutdown(&mut self, id: serde_json::Value) {
        self.shutdown_requested = true;
        self.send_response(id, serde_json::json!(null));
    }

    // ═══════════════════════════════════════════════════════════
    // Document synchronization
    // ═══════════════════════════════════════════════════════════

    pub fn handle_did_open(&mut self, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<DidOpenTextDocumentParams>(params) {
            let uri = p.text_document.uri.clone();
            self.documents.insert(
                uri.clone(),
                DocumentState {
                    uri: uri.clone(),
                    text: p.text_document.text.clone(),
                    version: p.text_document.version,
                },
            );
            self.publish_diagnostics(&uri);
        }
    }

    pub fn handle_did_change(&mut self, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<DidChangeTextDocumentParams>(params) {
            let uri = p.text_document.uri.clone();
            if let Some(change) = p.content_changes.into_iter().last() {
                if let Some(doc) = self.documents.get_mut(&uri) {
                    doc.text = change.text;
                    doc.version = p.text_document.version;
                }
            }
            self.publish_diagnostics(&uri);
        }
    }

    pub fn handle_did_close(&mut self, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<DidCloseTextDocumentParams>(params) {
            self.documents.remove(&p.text_document.uri);
            // Clear diagnostics
            self.send_notification(
                "textDocument/publishDiagnostics",
                serde_json::to_value(PublishDiagnosticsParams {
                    uri: p.text_document.uri,
                    diagnostics: vec![],
                })
                .unwrap(),
            );
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Diagnostics
    // ═══════════════════════════════════════════════════════════

    pub(crate) fn publish_diagnostics(&mut self, uri: &str) {
        let (source, filename) = match self.documents.get(uri) {
            Some(doc) => (doc.text.clone(), uri_to_filename(uri)),
            None => return,
        };

        let diagnostics = self.compile_and_get_diagnostics(&source, &filename);

        self.send_notification(
            "textDocument/publishDiagnostics",
            serde_json::to_value(PublishDiagnosticsParams {
                uri: uri.to_string(),
                diagnostics,
            })
            .unwrap(),
        );
    }

    pub(crate) fn compile_and_get_diagnostics(
        &self,
        source: &str,
        filename: &str,
    ) -> Vec<Diagnostic> {
        let (_typed_program, errors) = crate::check_source(source, filename);

        errors
            .iter()
            .map(|e| {
                let range = match &e.location {
                    Some(span) => Range {
                        start: Position {
                            line: span.start.line.saturating_sub(1) as u32,
                            character: span.start.column.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: span.end.line.saturating_sub(1) as u32,
                            character: span.end.column.saturating_sub(1) as u32,
                        },
                    },
                    None => Range::default(),
                };

                let sev = match e.severity {
                    Severity::Error => severity::ERROR,
                    Severity::Warning => severity::WARNING,
                    Severity::Note => severity::HINT,
                };

                Diagnostic {
                    range,
                    severity: sev,
                    message: e.message.clone(),
                    code: Some(e.error_code.clone()),
                    source: Some("axon".to_string()),
                }
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════
    // Completion
    // ═══════════════════════════════════════════════════════════

    pub fn handle_completion(&mut self, id: serde_json::Value, _params: serde_json::Value) {
        let mut items = Vec::new();
        items.extend(self.get_keyword_completions());
        items.extend(self.get_type_completions());
        items.extend(self.get_stdlib_completions());

        self.send_response(id, serde_json::to_value(&items).unwrap());
    }

    pub(crate) fn get_keyword_completions(&self) -> Vec<CompletionItem> {
        let keywords = [
            ("fn", "Function declaration"),
            ("let", "Variable binding"),
            ("mut", "Mutable binding"),
            ("if", "Conditional"),
            ("else", "Else branch"),
            ("while", "While loop"),
            ("for", "For loop"),
            ("in", "In keyword"),
            ("match", "Match expression"),
            ("return", "Return from function"),
            ("struct", "Struct definition"),
            ("enum", "Enum definition"),
            ("trait", "Trait definition"),
            ("impl", "Implementation block"),
            ("pub", "Public visibility"),
            ("use", "Import declaration"),
            ("mod", "Module declaration"),
            ("type", "Type alias"),
            ("as", "Type cast"),
            ("unsafe", "Unsafe block"),
            ("true", "Boolean true"),
            ("false", "Boolean false"),
            ("self", "Self reference"),
        ];

        keywords
            .iter()
            .map(|(kw, desc)| CompletionItem {
                label: kw.to_string(),
                kind: completion_kind::KEYWORD,
                detail: Some(desc.to_string()),
                documentation: None,
                insert_text: None,
            })
            .collect()
    }

    pub(crate) fn get_type_completions(&self) -> Vec<CompletionItem> {
        let types = [
            ("Int8", "8-bit signed integer"),
            ("Int16", "16-bit signed integer"),
            ("Int32", "32-bit signed integer"),
            ("Int64", "64-bit signed integer"),
            ("UInt8", "8-bit unsigned integer"),
            ("UInt16", "16-bit unsigned integer"),
            ("UInt32", "32-bit unsigned integer"),
            ("UInt64", "64-bit unsigned integer"),
            ("Float16", "16-bit floating point"),
            ("Float32", "32-bit floating point"),
            ("Float64", "64-bit floating point"),
            ("Bool", "Boolean type"),
            ("Char", "Character type"),
            ("String", "String type"),
            ("Tensor", "Tensor type"),
            ("Vec", "Dynamic array"),
            ("HashMap", "Hash map"),
            ("HashSet", "Hash set"),
            ("Option", "Optional value"),
            ("Result", "Result type"),
        ];

        types
            .iter()
            .map(|(ty, desc)| CompletionItem {
                label: ty.to_string(),
                kind: completion_kind::CLASS,
                detail: Some(desc.to_string()),
                documentation: None,
                insert_text: None,
            })
            .collect()
    }

    pub(crate) fn get_stdlib_completions(&self) -> Vec<CompletionItem> {
        let functions = [
            ("print", "fn(String) -> ()", "Print to stdout"),
            ("println", "fn(String) -> ()", "Print line to stdout"),
            ("sin", "fn(Float64) -> Float64", "Sine function"),
            ("cos", "fn(Float64) -> Float64", "Cosine function"),
            ("sqrt", "fn(Float64) -> Float64", "Square root"),
            ("abs", "fn(Float64) -> Float64", "Absolute value"),
            ("zeros", "fn(Vec<Int64>) -> Tensor", "Create zero tensor"),
            ("ones", "fn(Vec<Int64>) -> Tensor", "Create ones tensor"),
            ("randn", "fn(Vec<Int64>) -> Tensor", "Random normal tensor"),
            ("matmul", "fn(Tensor, Tensor) -> Tensor", "Matrix multiply"),
            ("Linear", "struct(in: Int64, out: Int64)", "Linear layer"),
            ("Conv2d", "struct(...)", "2D convolution layer"),
            ("relu", "fn(Tensor) -> Tensor", "ReLU activation"),
            ("softmax", "fn(Tensor) -> Tensor", "Softmax function"),
            ("cross_entropy", "fn(Tensor, Tensor) -> Tensor", "Cross-entropy loss"),
            ("SGD", "struct(lr: Float64)", "SGD optimizer"),
            ("Adam", "struct(lr: Float64)", "Adam optimizer"),
        ];

        functions
            .iter()
            .map(|(name, sig, doc)| CompletionItem {
                label: name.to_string(),
                kind: completion_kind::FUNCTION,
                detail: Some(sig.to_string()),
                documentation: Some(doc.to_string()),
                insert_text: None,
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════
    // Hover
    // ═══════════════════════════════════════════════════════════

    pub fn handle_hover(&mut self, id: serde_json::Value, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<TextDocumentPositionParams>(params) {
            let result = self.get_hover_info(&p.text_document.uri, p.position);
            match result {
                Some(hover) => self.send_response(id, serde_json::to_value(hover).unwrap()),
                None => self.send_response(id, serde_json::json!(null)),
            }
        } else {
            self.send_response(id, serde_json::json!(null));
        }
    }

    fn get_hover_info(&self, uri: &str, position: Position) -> Option<Hover> {
        let doc = self.documents.get(uri)?;
        let word = get_word_at_position(&doc.text, position)?;

        // Check if it's a keyword
        let keyword_info = match word.as_str() {
            "fn" => Some("**fn** — Declare a function"),
            "let" => Some("**let** — Bind a value to a variable"),
            "mut" => Some("**mut** — Make a binding mutable"),
            "struct" => Some("**struct** — Define a struct type"),
            "enum" => Some("**enum** — Define an enum type"),
            "trait" => Some("**trait** — Define a trait"),
            "impl" => Some("**impl** — Implement methods or traits"),
            "if" => Some("**if** — Conditional expression"),
            "match" => Some("**match** — Pattern matching expression"),
            "for" => Some("**for** — For loop"),
            "while" => Some("**while** — While loop"),
            "return" => Some("**return** — Return from function"),
            "pub" => Some("**pub** — Public visibility modifier"),
            "use" => Some("**use** — Import items"),
            _ => None,
        };

        if let Some(info) = keyword_info {
            return Some(Hover {
                contents: MarkupContent {
                    kind: "markdown".to_string(),
                    value: info.to_string(),
                },
                range: None,
            });
        }

        // Try to find type info by running the checker
        let filename = uri_to_filename(uri);
        let (checker, _errors) = crate::typeck::check(&doc.text, &filename);

        if let Some(sym_id) = checker.symbols.lookup(&word) {
            let sym = checker.symbols.get_symbol(sym_id);
            let type_name = format!("{}", checker.interner.resolve(sym.ty));
            let kind_str = match sym.kind {
                crate::symbol::SymbolKind::Function => "function",
                crate::symbol::SymbolKind::Variable => "variable",
                crate::symbol::SymbolKind::Parameter => "parameter",
                crate::symbol::SymbolKind::StructDef => "struct",
                crate::symbol::SymbolKind::EnumDef => "enum",
                crate::symbol::SymbolKind::TraitDef => "trait",
                crate::symbol::SymbolKind::TypeAlias => "type alias",
                crate::symbol::SymbolKind::Module => "module",
                crate::symbol::SymbolKind::Field => "field",
                crate::symbol::SymbolKind::Method => "method",
                crate::symbol::SymbolKind::GenericParam => "generic parameter",
                crate::symbol::SymbolKind::EnumVariant => "enum variant",
            };

            return Some(Hover {
                contents: MarkupContent {
                    kind: "markdown".to_string(),
                    value: format!("```axon\n{} {}: {}\n```", kind_str, word, type_name),
                },
                range: None,
            });
        }

        None
    }

    // ═══════════════════════════════════════════════════════════
    // Go to Definition
    // ═══════════════════════════════════════════════════════════

    pub fn handle_definition(&mut self, id: serde_json::Value, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<TextDocumentPositionParams>(params) {
            let result = self.get_definition(&p.text_document.uri, p.position);
            match result {
                Some(loc) => self.send_response(id, serde_json::to_value(loc).unwrap()),
                None => self.send_response(id, serde_json::json!(null)),
            }
        } else {
            self.send_response(id, serde_json::json!(null));
        }
    }

    fn get_definition(&self, uri: &str, position: Position) -> Option<Location> {
        let doc = self.documents.get(uri)?;
        let word = get_word_at_position(&doc.text, position)?;

        let filename = uri_to_filename(uri);
        let (checker, _errors) = crate::typeck::check(&doc.text, &filename);

        let sym_id = checker.symbols.lookup(&word)?;
        let sym = checker.symbols.get_symbol(sym_id);

        Some(Location {
            uri: uri.to_string(),
            range: span_to_range(&sym.span),
        })
    }

    // ═══════════════════════════════════════════════════════════
    // Document Symbols
    // ═══════════════════════════════════════════════════════════

    pub fn handle_document_symbols(&mut self, id: serde_json::Value, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<serde_json::Value>(params) {
            let uri = p
                .get("textDocument")
                .and_then(|td| td.get("uri"))
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let symbols = self.get_document_symbols(uri);
            self.send_response(id, serde_json::to_value(&symbols).unwrap());
        } else {
            self.send_response(id, serde_json::json!([]));
        }
    }

    fn get_document_symbols(&self, uri: &str) -> Vec<DocumentSymbol> {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let filename = uri_to_filename(uri);
        let (program, _errors) = crate::parse_source(&doc.text, &filename);

        let mut symbols = Vec::new();
        for item in &program.items {
            if let Some(sym) = item_to_document_symbol(item) {
                symbols.push(sym);
            }
        }
        symbols
    }

    // ═══════════════════════════════════════════════════════════
    // Formatting
    // ═══════════════════════════════════════════════════════════

    pub fn handle_formatting(&mut self, id: serde_json::Value, params: serde_json::Value) {
        if let Ok(p) = serde_json::from_value::<DocumentFormattingParams>(params) {
            let result = self.format_document(&p.text_document.uri);
            match result {
                Some(edits) => self.send_response(id, serde_json::to_value(&edits).unwrap()),
                None => self.send_response(id, serde_json::json!(null)),
            }
        } else {
            self.send_response(id, serde_json::json!(null));
        }
    }

    fn format_document(&self, uri: &str) -> Option<Vec<TextEdit>> {
        let doc = self.documents.get(uri)?;
        let filename = uri_to_filename(uri);

        match crate::fmt::Formatter::format(&doc.text, &filename) {
            Ok(formatted) => {
                let line_count = doc.text.lines().count() as u32;
                let last_line_len = doc.text.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

                Some(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position {
                            line: line_count,
                            character: last_line_len,
                        },
                    },
                    new_text: formatted,
                }])
            }
            Err(_) => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn uri_to_filename(uri: &str) -> String {
    uri.strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .unwrap_or(uri)
        .to_string()
}

fn span_to_range(span: &crate::span::Span) -> Range {
    Range {
        start: Position {
            line: span.start.line.saturating_sub(1) as u32,
            character: span.start.column.saturating_sub(1) as u32,
        },
        end: Position {
            line: span.end.line.saturating_sub(1) as u32,
            character: span.end.column.saturating_sub(1) as u32,
        },
    }
}

fn get_word_at_position(text: &str, position: Position) -> Option<String> {
    let line = text.lines().nth(position.line as usize)?;
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

    let bytes = line.as_bytes();
    let mut start = col;
    let mut end = col;

    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(line[start..end].to_string())
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn item_to_document_symbol(item: &Item) -> Option<DocumentSymbol> {
    let range = span_to_range(&item.span);
    match &item.kind {
        ItemKind::Function(f) => Some(DocumentSymbol {
            name: f.name.clone(),
            kind: symbol_kind::FUNCTION,
            range,
            selection_range: span_to_range(&f.span),
            children: vec![],
        }),
        ItemKind::Struct(s) => Some(DocumentSymbol {
            name: s.name.clone(),
            kind: symbol_kind::STRUCT,
            range,
            selection_range: span_to_range(&s.span),
            children: vec![],
        }),
        ItemKind::Enum(e) => Some(DocumentSymbol {
            name: e.name.clone(),
            kind: symbol_kind::ENUM,
            range,
            selection_range: span_to_range(&e.span),
            children: vec![],
        }),
        ItemKind::Trait(t) => Some(DocumentSymbol {
            name: t.name.clone(),
            kind: symbol_kind::INTERFACE,
            range,
            selection_range: span_to_range(&t.span),
            children: vec![],
        }),
        ItemKind::Impl(imp) => {
            let children: Vec<DocumentSymbol> = imp
                .items
                .iter()
                .filter_map(|i| item_to_document_symbol(i))
                .collect();
            let name = format!("impl {}", format_type_expr_brief(&imp.type_name));
            Some(DocumentSymbol {
                name,
                kind: symbol_kind::STRUCT,
                range,
                selection_range: span_to_range(&imp.span),
                children,
            })
        }
        _ => None,
    }
}

fn format_type_expr_brief(ty: &crate::ast::TypeExpr) -> String {
    match &ty.kind {
        crate::ast::TypeExprKind::Named(name) => name.clone(),
        crate::ast::TypeExprKind::Generic { name, .. } => name.clone(),
        _ => "...".to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_to_filename() {
        assert_eq!(uri_to_filename("file:///home/test.axon"), "home/test.axon");
        assert_eq!(uri_to_filename("file:///C:/test.axon"), "C:/test.axon");
        assert_eq!(uri_to_filename("test.axon"), "test.axon");
    }

    #[test]
    fn test_get_word_at_position() {
        let text = "fn main() {\n    let x = 42;\n}";
        // "fn" at line 0, col 0
        let word = get_word_at_position(text, Position { line: 0, character: 0 });
        assert_eq!(word, Some("fn".to_string()));

        // "main" at line 0, col 4
        let word = get_word_at_position(text, Position { line: 0, character: 4 });
        assert_eq!(word, Some("main".to_string()));

        // "let" at line 1, col 4
        let word = get_word_at_position(text, Position { line: 1, character: 5 });
        assert_eq!(word, Some("let".to_string()));

        // "x" at line 1, col 8
        let word = get_word_at_position(text, Position { line: 1, character: 8 });
        assert_eq!(word, Some("x".to_string()));
    }

    #[test]
    fn test_get_word_at_position_boundary() {
        let text = "hello_world";
        let word = get_word_at_position(text, Position { line: 0, character: 5 });
        assert_eq!(word, Some("hello_world".to_string()));
    }

    #[test]
    fn test_diagnostics_from_valid_source() {
        let server = LspServer::new();
        let source = "fn main() {\n    let x = 42;\n}";
        let diags = server.compile_and_get_diagnostics(source, "test.axon");
        assert!(diags.is_empty(), "valid source should have no diagnostics, got: {:?}", diags);
    }

    #[test]
    fn test_diagnostics_from_invalid_source() {
        let server = LspServer::new();
        let source = "fn main( {";
        let diags = server.compile_and_get_diagnostics(source, "test.axon");
        assert!(!diags.is_empty(), "invalid source should produce diagnostics");
        assert_eq!(diags[0].source, Some("axon".to_string()));
    }

    #[test]
    fn test_keyword_completions() {
        let server = LspServer::new();
        let completions = server.get_keyword_completions();
        assert!(!completions.is_empty());
        let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"fn"));
        assert!(labels.contains(&"let"));
        assert!(labels.contains(&"struct"));
        assert!(labels.contains(&"match"));
    }

    #[test]
    fn test_type_completions() {
        let server = LspServer::new();
        let completions = server.get_type_completions();
        let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"Int32"));
        assert!(labels.contains(&"Float64"));
        assert!(labels.contains(&"Tensor"));
        assert!(labels.contains(&"Vec"));
        assert!(labels.contains(&"Option"));
    }

    #[test]
    fn test_stdlib_completions() {
        let server = LspServer::new();
        let completions = server.get_stdlib_completions();
        let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"print"));
        assert!(labels.contains(&"println"));
        assert!(labels.contains(&"zeros"));
        assert!(labels.contains(&"matmul"));
    }

    #[test]
    fn test_document_symbols_extraction() {
        let mut server = LspServer::new();
        let uri = "file:///test.axon";
        server.documents.insert(
            uri.to_string(),
            DocumentState {
                uri: uri.to_string(),
                text: "fn foo() {}\nstruct Bar {}\nenum Baz {}".to_string(),
                version: 1,
            },
        );
        let symbols = server.get_document_symbols(uri);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"foo"), "should find fn foo, got: {:?}", names);
        assert!(names.contains(&"Bar"), "should find struct Bar, got: {:?}", names);
        assert!(names.contains(&"Baz"), "should find enum Baz, got: {:?}", names);
    }

    #[test]
    fn test_span_to_range() {
        let span = crate::span::Span::new("test.axon", 1, 1, 1, 5);
        let range = span_to_range(&span);
        assert_eq!(range.start.line, 0);
        assert_eq!(range.start.character, 0);
        assert_eq!(range.end.line, 0);
        assert_eq!(range.end.character, 4);
    }

    #[test]
    fn test_is_ident_char() {
        assert!(is_ident_char(b'a'));
        assert!(is_ident_char(b'Z'));
        assert!(is_ident_char(b'_'));
        assert!(is_ident_char(b'9'));
        assert!(!is_ident_char(b' '));
        assert!(!is_ident_char(b'('));
    }
}
