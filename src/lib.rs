// lib.rs — Axon compiler library root

pub mod span;
pub mod token;
pub mod error;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod types;
pub mod symbol;
pub mod typeck;
pub mod shapes;
pub mod stdlib;
pub mod borrow;
pub mod tast;
pub mod mir;
pub mod codegen;
pub mod fmt;
pub mod lint;
pub mod repl;
pub mod doc;
pub mod lsp;
pub mod pkg;
pub mod debugger;

/// Parse an Axon source fileand return the AST along with any errors.
pub fn parse_source(source: &str, filename: &str) -> (ast::Program, Vec<error::CompileError>) {
    let mut lexer = lexer::Lexer::new(source, filename);
    let tokens = lexer.tokenize();
    let mut parser = parser::Parser::new(tokens, filename);
    let program = parser.parse_program();

    let mut all_errors = lexer.errors;
    all_errors.extend(parser.errors);

    (program, all_errors)
}

/// Format the AST as pretty-printed JSON.
pub fn ast_to_json(program: &ast::Program) -> String {
    serde_json::to_string_pretty(program).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

/// Type-check an Axon source file and return the typed AST with any errors.
pub fn check_source(source: &str, filename: &str) -> (tast::TypedProgram, Vec<error::CompileError>) {
    let (program, parse_errors) = parse_source(source, filename);
    if !parse_errors.is_empty() {
        let empty = tast::TypedProgram { items: vec![], span: crate::span::Span::dummy() };
        return (empty, parse_errors);
    }

    let (checker, check_errors) = typeck::check(source, filename);

    // Build TAST from AST + type info
    let builder = tast::TastBuilder::new(&checker);
    let typed_program = builder.build(&program);

    // Run borrow checker
    let mut borrow_checker = borrow::BorrowChecker::new(&checker.interner, &checker.symbols);
    borrow_checker.check_program(&program);
    let borrow_errors = borrow_checker.take_errors();

    let mut all_errors = check_errors;
    all_errors.extend(borrow_errors);

    (typed_program, all_errors)
}

/// Format the typed AST as pretty-printed JSON.
pub fn tast_to_json(program: &tast::TypedProgram) -> String {
    serde_json::to_string_pretty(program).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}
