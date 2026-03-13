// tooling_tests.rs — Integration tests for Phase 7 tooling
//
// Tests for: Formatter, Linter, REPL, DocGenerator, LSP protocol, Package Manager

// ═══════════════════════════════════════════════════════════════
// Formatter tests
// ═══════════════════════════════════════════════════════════════

mod formatter_tests {
    use axonc::fmt::Formatter;

    #[test]
    fn format_simple_function() {
        let src = "fn   hello(  )   {  }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("fn hello()"));
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn format_struct() {
        let src = "model Point { x: Int32, y: Int32, }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("model Point"));
        assert!(result.contains("x: Int32,"));
        assert!(result.contains("y: Int32,"));
    }

    #[test]
    fn format_if_else_braces_same_line() {
        let src = "fn test() { if true { val _x: Int32 = 1; } }";
        let result = Formatter::format(src, "test.axon").unwrap();
        // Braces should appear on the same line as `if`
        assert!(result.contains("if true {"));
    }

    #[test]
    fn format_match_expression() {
        let src = "fn test() {\n    val _r: Int32 = match x {\n        1 => 0,\n        _ => 1,\n    };\n}";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("match x"));
    }

    #[test]
    fn format_let_bindings() {
        let src = "fn main() { val x: Int32 = 0; val y: Int32 = 1; }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("val x: Int32 = 0;"));
        assert!(result.contains("val y: Int32 = 1;"));
    }

    #[test]
    fn format_idempotent() {
        let src = "fn add(a: Int32, b: Int32): Int32 { return a + b; }";
        let first = Formatter::format(src, "test.axon").unwrap();
        let second = Formatter::format(&first, "test.axon").unwrap();
        assert_eq!(first, second, "formatting should be idempotent");
    }

    #[test]
    fn format_multiple_functions() {
        let src = "fn foo() {} fn bar() {}";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("fn foo()"));
        assert!(result.contains("fn bar()"));
    }

    #[test]
    fn format_enum_definition() {
        let src = "enum Color { Red, Green, Blue, }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("enum Color"));
        assert!(result.contains("Red,"));
        assert!(result.contains("Green,"));
        assert!(result.contains("Blue,"));
    }

    #[test]
    fn format_public_function() {
        let src = "pub fn greet() {}";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("pub fn greet()"));
    }

    #[test]
    fn format_parse_error_returns_err() {
        let src = "fn ??? broken";
        let result = Formatter::format(src, "test.axon");
        assert!(result.is_err());
    }
}

// ═══════════════════════════════════════════════════════════════
// Linter tests
// ═══════════════════════════════════════════════════════════════

mod linter_tests {
    use axonc::lint::Linter;

    #[test]
    fn detect_unused_variable() {
        let src = "fn main() { val x: Int32 = 1; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            warnings.iter().any(|w| w.code == "W5001" && w.message.contains("unused variable")),
            "should detect unused variable `x`"
        );
    }

    #[test]
    fn allow_underscore_prefixed_variables() {
        let src = "fn main() { val _unused: Int32 = 1; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            !warnings.iter().any(|w| w.code == "W5001" && w.message.contains("_unused")),
            "_-prefixed variables should not trigger unused warnings"
        );
    }

    #[test]
    fn detect_empty_function_body() {
        let src = "fn noop() {}";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5003"), "should detect empty function body");
    }

    #[test]
    fn detect_too_many_parameters() {
        let src = "fn many(a: Int32, b: Int32, c: Int32, d: Int32, e: Int32, f: Int32, g: Int32, h: Int32) {}";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5006"), "should detect too many parameters");
    }

    #[test]
    fn detect_non_snake_case_function() {
        let src = "fn MyFunction() { val _x: Int32 = 0; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            warnings.iter().any(|w| w.code == "W5004" && w.message.contains("snake_case")),
            "should detect non-snake_case function name"
        );
    }

    #[test]
    fn detect_non_pascal_case_struct() {
        let src = "model my_point { x: Int32, }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5005"), "should detect non-PascalCase model name");
    }

    #[test]
    fn multiple_warnings_in_one_file() {
        // Non-snake function + empty body + too many params
        let src = "fn MyFunc() {} fn another(a: Int32, b: Int32, c: Int32, d: Int32, e: Int32, f: Int32, g: Int32, h: Int32) {}";
        let warnings = Linter::lint(src, "test.axon");
        let codes: Vec<&str> = warnings.iter().map(|w| w.code.as_str()).collect();
        assert!(codes.contains(&"W5004"), "should have naming warning");
        assert!(codes.contains(&"W5003"), "should have empty body warning");
        assert!(codes.contains(&"W5006"), "should have too-many-params warning");
    }

    #[test]
    fn clean_file_produces_no_warnings() {
        let src = "fn main() { val _x: Int32 = 0; }";
        let warnings = Linter::lint(src, "test.axon");
        // Filter out magic number warnings for 0 (they are exempt)
        let real_warnings: Vec<_> = warnings.iter().filter(|w| w.code != "W5009").collect();
        assert!(
            real_warnings.is_empty(),
            "clean file should produce no warnings, got: {:?}",
            real_warnings.iter().map(|w| &w.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn detect_todo_comment() {
        let src = "// TODO: fix this\nfn main() { val _x: Int32 = 0; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5009"), "should detect TODO comment");
    }

    #[test]
    fn lint_warning_format_human() {
        let src = "fn MyBad() {}";
        let warnings = Linter::lint(src, "test.axon");
        let naming_warning = warnings.iter().find(|w| w.code == "W5004").unwrap();
        let formatted = naming_warning.format_human();
        assert!(formatted.contains("warning[W5004]"));
        assert!(formatted.contains("snake_case"));
    }
}

// ═══════════════════════════════════════════════════════════════
// REPL tests
// ═══════════════════════════════════════════════════════════════

mod repl_tests {
    use axonc::repl::{Repl, ReplResult};

    #[test]
    fn eval_let_binding() {
        let mut repl = Repl::new();
        let result = repl.eval_line("val x: Int32 = 42;");
        assert_eq!(result, ReplResult::Definition("x".to_string()));
    }

    #[test]
    fn eval_expression() {
        let mut repl = Repl::new();
        let result = repl.eval_line("1 + 2");
        match result {
            ReplResult::Value(v) => assert!(v.contains("1 + 2")),
            other => panic!("expected Value, got {:?}", other),
        }
    }

    #[test]
    fn eval_function_definition() {
        let mut repl = Repl::new();
        let result = repl.eval_line("fn add(a: Int32, b: Int32): Int32 { return a + b; }");
        assert_eq!(result, ReplResult::Definition("add".to_string()));
    }

    #[test]
    fn type_command() {
        let mut repl = Repl::new();
        let result = repl.eval_line(":type 42");
        match result {
            ReplResult::Value(v) => assert!(v.contains("42")),
            other => panic!("expected Value from :type, got {:?}", other),
        }
    }

    #[test]
    fn help_command() {
        let mut repl = Repl::new();
        let result = repl.eval_line(":help");
        assert_eq!(result, ReplResult::Command);
    }

    #[test]
    fn clear_command() {
        let mut repl = Repl::new();
        repl.eval_line("val x: Int32 = 1;");
        repl.eval_line("val y: Int32 = 2;");
        assert_eq!(repl.eval_line(":clear"), ReplResult::Command);
        // After clear, bindings should be empty — verify by defining again
        let result = repl.eval_line("val x: Int32 = 99;");
        assert_eq!(result, ReplResult::Definition("x".to_string()));
    }

    #[test]
    fn quit_command() {
        let mut repl = Repl::new();
        assert_eq!(repl.eval_line(":quit"), ReplResult::Exit);
        assert_eq!(repl.eval_line(":q"), ReplResult::Exit);
    }

    #[test]
    fn unknown_command_returns_error() {
        let mut repl = Repl::new();
        match repl.eval_line(":doesnotexist") {
            ReplResult::Error(e) => assert!(e.contains("unknown command")),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn eval_struct_definition() {
        let mut repl = Repl::new();
        let result = repl.eval_line("model Point { x: Int32, y: Int32, }");
        assert_eq!(result, ReplResult::Definition("Point".to_string()));
    }
}

// ═══════════════════════════════════════════════════════════════
// Doc generator tests
// ═══════════════════════════════════════════════════════════════

mod doc_tests {
    use axonc::doc::DocGenerator;

    #[test]
    fn generate_docs_for_function_with_doc_comment() {
        let src = "/// Adds two numbers.\npub fn add(a: Int32, b: Int32): Int32 { return a + b; }";
        let html = DocGenerator::generate(src, "math.axon");
        assert!(html.contains("Adds two numbers"), "doc comment should appear in output");
        assert!(html.contains("add"), "function name should appear in output");
    }

    #[test]
    fn generate_docs_for_struct() {
        let src = "pub model Point { pub x: Int32, pub y: Int32, }";
        let html = DocGenerator::generate(src, "geom.axon");
        assert!(html.contains("Point"), "model name should appear");
        assert!(html.contains("Model"), "kind label should appear");
    }

    #[test]
    fn output_is_valid_html() {
        let src = "pub fn hello() { val _x: Int32 = 0; }";
        let html = DocGenerator::generate(src, "hello.axon");
        assert!(html.contains("<html"), "output should contain <html>");
        assert!(html.contains("<body>"), "output should contain <body>");
        assert!(html.contains("</html>"), "output should contain closing </html>");
    }

    #[test]
    fn doc_comments_extracted_correctly() {
        let src = "/// First line\n/// Second line\npub fn documented() { val _x: Int32 = 0; }";
        let html = DocGenerator::generate(src, "test.axon");
        assert!(html.contains("First line"), "first line of doc comment should appear");
        assert!(html.contains("Second line"), "second line of doc comment should appear");
    }

    #[test]
    fn public_items_included() {
        let src = "pub fn public_fn() { val _x: Int32 = 0; }\nfn private_fn() { val _y: Int32 = 0; }";
        let html = DocGenerator::generate(src, "test.axon");
        assert!(html.contains("public_fn"), "public function should be documented");
        // Private items are excluded by doc generator
        assert!(!html.contains("private_fn"), "private function should not be documented");
    }

    #[test]
    fn generate_docs_multiple_items() {
        let src = r#"
pub fn alpha() { val _x: Int32 = 0; }
pub fn beta() { val _y: Int32 = 0; }
pub model Gamma { pub value: Int32, }
"#;
        let html = DocGenerator::generate(src, "multi.axon");
        assert!(html.contains("alpha"), "first function should appear");
        assert!(html.contains("beta"), "second function should appear");
        assert!(html.contains("Gamma"), "model should appear");
    }

    #[test]
    fn generate_docs_for_enum() {
        let src = "pub enum Direction { North, South, East, West, }";
        let html = DocGenerator::generate(src, "test.axon");
        assert!(html.contains("Direction"), "enum name should appear");
        assert!(html.contains("North"), "variant should appear");
    }

    #[test]
    fn generate_docs_parse_error_produces_comment() {
        let src = "pub fn ???broken";
        let html = DocGenerator::generate(src, "bad.axon");
        assert!(html.contains("parse errors"), "parse errors should produce a comment");
    }
}

// ═══════════════════════════════════════════════════════════════
// LSP protocol tests
// ═══════════════════════════════════════════════════════════════

mod lsp_tests {
    use axonc::lsp::protocol::*;

    #[test]
    fn message_response_serialization() {
        let msg = Message::response(
            serde_json::json!(1),
            serde_json::json!({"capabilities": {}}),
        );
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"result\""));
        // Optional fields should be skipped
        assert!(!json.contains("\"method\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn message_notification_serialization() {
        let msg = Message::notification(
            "textDocument/publishDiagnostics",
            serde_json::json!({"uri": "file:///test.axon", "diagnostics": []}),
        );
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("textDocument/publishDiagnostics"));
        assert!(!json.contains("\"id\""), "notifications should not have id");
    }

    #[test]
    fn message_deserialization() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 42,
            "method": "textDocument/hover",
            "params": {"textDocument": {"uri": "file:///test.axon"}, "position": {"line": 0, "character": 5}}
        }"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.jsonrpc, "2.0");
        assert_eq!(msg.id, Some(serde_json::json!(42)));
        assert_eq!(msg.method, Some("textDocument/hover".to_string()));
    }

    #[test]
    fn completion_item_creation() {
        let item = CompletionItem {
            label: "println".to_string(),
            kind: completion_kind::FUNCTION,
            detail: Some("fn(String)".to_string()),
            documentation: Some("Prints a line to stdout".to_string()),
            insert_text: Some("println(${1})".to_string()),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"println\""));
        assert!(json.contains(&format!("{}", completion_kind::FUNCTION)));
        // Round-trip
        let parsed: CompletionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.label, "println");
        assert_eq!(parsed.kind, completion_kind::FUNCTION);
    }

    #[test]
    fn diagnostic_from_compile_error() {
        let diag = Diagnostic {
            range: Range {
                start: Position { line: 5, character: 10 },
                end: Position { line: 5, character: 20 },
            },
            severity: severity::ERROR,
            message: "type mismatch: expected Int32, got String".to_string(),
            code: Some("E2003".to_string()),
            source: Some("axon".to_string()),
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("type mismatch"));
        assert!(json.contains("E2003"));
        assert_eq!(diag.severity, severity::ERROR);
    }

    #[test]
    fn server_capabilities_serialization() {
        let caps = ServerCapabilities {
            text_document_sync: 1,
            completion_provider: Some(CompletionOptions {
                trigger_characters: vec![".".to_string(), ":".to_string()],
            }),
            hover_provider: true,
            definition_provider: true,
            references_provider: true,
            rename_provider: true,
            code_action_provider: true,
            document_formatting_provider: true,
            document_symbol_provider: true,
            signature_help_provider: None,
        };
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("\"textDocumentSync\":1"));
        assert!(json.contains("\"hoverProvider\":true"));
        assert!(json.contains("\"definitionProvider\":true"));
        assert!(json.contains("\"referencesProvider\":true"));
        assert!(json.contains("\"renameProvider\":true"));
        assert!(json.contains("\"codeActionProvider\":true"));

        // Round-trip
        let parsed: ServerCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.text_document_sync, 1);
        assert!(parsed.hover_provider);
        assert!(parsed.references_provider);
        assert!(parsed.rename_provider);
        assert!(parsed.code_action_provider);
    }

    #[test]
    fn hover_content_generation() {
        let hover = Hover {
            contents: MarkupContent {
                kind: "markdown".to_string(),
                value: "```axon\nfn add(a: Int32, b: Int32): Int32\n```".to_string(),
            },
            range: Some(Range {
                start: Position { line: 0, character: 3 },
                end: Position { line: 0, character: 6 },
            }),
        };
        let json = serde_json::to_string(&hover).unwrap();
        assert!(json.contains("markdown"));
        assert!(json.contains("fn add"));
    }

    #[test]
    fn document_symbol_serialization() {
        let sym = DocumentSymbol {
            name: "main".to_string(),
            kind: symbol_kind::FUNCTION,
            range: Range::default(),
            selection_range: Range::default(),
            children: vec![],
        };
        let json = serde_json::to_string(&sym).unwrap();
        assert!(json.contains("\"main\""));
        assert!(json.contains(&format!("{}", symbol_kind::FUNCTION)));
        // Empty children should be skipped
        assert!(!json.contains("children"));
    }

    #[test]
    fn position_default_is_zero() {
        let pos = Position::default();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn text_document_item_roundtrip() {
        let item = TextDocumentItem {
            uri: "file:///project/src/main.axon".to_string(),
            language_id: "axon".to_string(),
            version: 1,
            text: "fn main() {}".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        let parsed: TextDocumentItem = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.uri, item.uri);
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.text, "fn main() {}");
    }
}

// ═══════════════════════════════════════════════════════════════
// Package manager tests
// ═══════════════════════════════════════════════════════════════

mod pkg_tests {
    use axonc::pkg::manifest::Manifest;
    use axonc::pkg::scaffold;
    use axonc::pkg::resolver::{Resolver, ResolvedDep, DepSource};
    use axonc::pkg::lockfile::LockFile;
    use std::collections::HashMap;

    #[test]
    fn parse_axon_toml_manifest() {
        let toml = r#"
[package]
name = "my-project"
version = "0.1.0"
edition = "2024"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-project");
        assert_eq!(manifest.package.version, "0.1.0");
        assert_eq!(manifest.package.edition.as_deref(), Some("2024"));
    }

    #[test]
    fn parse_manifest_with_dependencies() {
        let toml = r#"
[package]
name = "with-deps"
version = "1.0.0"

[dependencies]
serde = "1.0"
tokio = "1.28"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.dependencies.len(), 2);
        assert!(manifest.dependencies.contains_key("serde"));
        assert!(manifest.dependencies.contains_key("tokio"));
    }

    #[test]
    fn create_project_scaffolding() {
        let tmp = std::env::temp_dir().join("axon_tooling_test_scaffold");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        scaffold::create_project("test_proj", tmp.to_str().unwrap()).unwrap();

        let project_dir = tmp.join("test_proj");
        assert!(project_dir.join("Axon.toml").exists(), "Axon.toml should be created");
        assert!(project_dir.join("src").join("main.axon").exists(), "src/main.axon should be created");
        assert!(project_dir.join("tests").exists(), "tests/ dir should be created");
        assert!(project_dir.join("README.md").exists(), "README.md should be created");

        let manifest_content = std::fs::read_to_string(project_dir.join("Axon.toml")).unwrap();
        assert!(manifest_content.contains("name = \"test_proj\""));

        // Creating same project again should fail
        assert!(scaffold::create_project("test_proj", tmp.to_str().unwrap()).is_err());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_simple_dependencies() {
        use axonc::pkg::manifest::*;

        let mut deps = HashMap::new();
        deps.insert("serde".to_string(), Dependency::Simple("1.0".to_string()));
        deps.insert("tokio".to_string(), Dependency::Simple("1.28".to_string()));

        let manifest = Manifest {
            package: PackageInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                authors: vec![],
                edition: None,
                description: None,
                license: None,
                repository: None,
            },
            dependencies: deps,
            dev_dependencies: HashMap::new(),
            build: BuildConfig::default(),
            features: HashMap::new(),
            lint: LintConfig::default(),
        };

        let resolved = Resolver::resolve(&manifest).unwrap();
        assert_eq!(resolved.len(), 2);
        assert!(resolved.iter().any(|r| r.name == "serde" && r.version == "1.0"));
        assert!(resolved.iter().any(|r| r.name == "tokio" && r.version == "1.28"));
    }

    #[test]
    fn generate_lock_file() {
        let deps = vec![
            ResolvedDep {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: DepSource::Registry("https://registry.axon-lang.org".to_string()),
            },
        ];
        let lockfile = LockFile::generate(&deps);
        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].name, "serde");
        assert!(lockfile.packages[0].source.starts_with("registry+"));
    }

    #[test]
    fn lock_file_roundtrip() {
        let deps = vec![
            ResolvedDep {
                name: "tokio".to_string(),
                version: "1.28.0".to_string(),
                source: DepSource::Git {
                    url: "https://github.com/tokio-rs/tokio".to_string(),
                    branch: Some("main".to_string()),
                },
            },
            ResolvedDep {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                source: DepSource::Registry("https://registry.axon-lang.org".to_string()),
            },
        ];
        let lockfile = LockFile::generate(&deps);
        let text = lockfile.to_string();
        let parsed = LockFile::from_str(&text).unwrap();
        assert_eq!(parsed.packages.len(), 2);
        for pkg in &parsed.packages {
            assert!(!pkg.name.is_empty());
            assert!(!pkg.version.is_empty());
            assert!(!pkg.source.is_empty());
        }
    }

    #[test]
    fn manifest_with_build_config() {
        let toml = r#"
[package]
name = "optimized"
version = "1.0.0"

[build]
target = "x86_64"
opt_level = 3
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.build.target.as_deref(), Some("x86_64"));
        assert_eq!(manifest.build.opt_level, Some(3));
    }

    #[test]
    fn manifest_with_features() {
        let toml = r#"
[package]
name = "featured"
version = "0.1.0"

[features]
default = ["std"]
std = []
async = ["tokio"]
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert!(manifest.features.contains_key("default"));
        assert!(manifest.features.contains_key("std"));
        assert!(manifest.features.contains_key("async"));
        assert_eq!(manifest.features["default"], vec!["std".to_string()]);
    }

    #[test]
    fn resolve_empty_dependencies() {
        use axonc::pkg::manifest::*;

        let manifest = Manifest {
            package: PackageInfo {
                name: "empty".to_string(),
                version: "0.1.0".to_string(),
                authors: vec![],
                edition: None,
                description: None,
                license: None,
                repository: None,
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build: BuildConfig::default(),
            features: HashMap::new(),
            lint: LintConfig::default(),
        };
        let resolved = Resolver::resolve(&manifest).unwrap();
        assert!(resolved.is_empty());
    }

    #[test]
    fn lockfile_serialize_with_checksum() {
        use axonc::pkg::lockfile::LockedPackage;

        let lockfile = LockFile {
            packages: vec![LockedPackage {
                name: "crypto".to_string(),
                version: "2.0.0".to_string(),
                source: "registry+https://registry.axon-lang.org".to_string(),
                checksum: Some("sha256:abcdef1234567890".to_string()),
            }],
        };
        let text = lockfile.to_string();
        assert!(text.contains("[[package]]"));
        assert!(text.contains("name = \"crypto\""));
        assert!(text.contains("checksum = \"sha256:abcdef1234567890\""));
    }

    #[test]
    fn manifest_missing_package_section_errors() {
        let toml = r#"
[dependencies]
serde = "1.0"
"#;
        let result = Manifest::from_str(toml);
        assert!(result.is_err(), "manifest without [package] should fail");
    }
}
