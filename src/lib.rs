// lib.rs — Axon compiler library root

pub mod span;
pub mod token;
pub mod error;
pub mod ast;
pub mod lexer;
pub mod parser;

/// Parse an Axon source file and return the AST along with any errors.
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
