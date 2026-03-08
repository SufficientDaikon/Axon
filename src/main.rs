// main.rs — Axon compiler CLI (axonc)

use clap::{Parser, Subcommand};
use std::fs;
use std::process;

use axonc::error::ErrorReporter;

#[derive(Parser)]
#[command(name = "axonc")]
#[command(about = "The Axon programming language compiler", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse an Axon source file and print the AST
    Parse {
        /// The source file to parse
        file: String,

        /// Output error messages in JSON format
        #[arg(long = "error-format", value_parser = ["json", "human"])]
        error_format: Option<String>,

        /// Print only errors, not the AST
        #[arg(long)]
        errors_only: bool,
    },

    /// Lex an Axon source file and print the token stream
    Lex {
        /// The source file to lex
        file: String,
    },

    /// Type-check an Axon source file
    Check {
        /// The source file to check
        file: String,

        /// Output error messages in JSON format
        #[arg(long = "error-format", value_parser = ["json", "human"])]
        error_format: Option<String>,

        /// Emit the typed AST as JSON
        #[arg(long = "emit-tast")]
        emit_tast: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            file,
            error_format,
            errors_only,
        } => {
            let json_mode = error_format.as_deref() == Some("json");
            run_parse(&file, json_mode, errors_only);
        }
        Commands::Lex { file } => {
            run_lex(&file);
        }
        Commands::Check { file, error_format, emit_tast } => {
            let json_mode = error_format.as_deref() == Some("json");
            run_check(&file, json_mode, emit_tast);
        }
    }
}

fn run_parse(file: &str, json_errors: bool, errors_only: bool) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    let (program, errors) = axonc::parse_source(&source, file);
    let mut reporter = ErrorReporter::new(json_errors);
    for e in errors {
        reporter.report(e);
    }

    if reporter.has_errors() {
        eprint!("{}", reporter.render());
        process::exit(1);
    }

    if !errors_only {
        println!("{}", axonc::ast_to_json(&program));
    }
}

fn run_lex(file: &str) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    let mut lexer = axonc::lexer::Lexer::new(&source, file);
    let tokens = lexer.tokenize();

    if !lexer.errors.is_empty() {
        for e in &lexer.errors {
            eprint!("{}", e.format_human());
        }
        process::exit(1);
    }

    for token in &tokens {
        println!(
            "{:>4}:{:<3} {:?}",
            token.span.start.line, token.span.start.column, token.kind
        );
    }
}

fn run_check(file: &str, json_errors: bool, emit_tast: bool) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    let (typed_program, errors) = axonc::check_source(&source, file);
    let mut reporter = ErrorReporter::new(json_errors);
    for e in errors {
        reporter.report(e);
    }

    if reporter.has_errors() {
        eprint!("{}", reporter.render());
        process::exit(1);
    }

    if emit_tast {
        println!("{}", axonc::tast_to_json(&typed_program));
    } else {
        println!("OK: type check passed");
    }
}
