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

    /// Compile an Axon source file to a native binary
    Build {
        /// The source file to compile
        file: String,

        /// Output file path (default: input filename without extension)
        #[arg(short, long)]
        output: Option<String>,

        /// Optimization level
        #[arg(short = 'O', long, default_value = "0", value_parser = ["0", "1", "2", "3"])]
        opt_level: String,

        /// Emit LLVM IR instead of compiling
        #[arg(long = "emit-llvm")]
        emit_llvm: bool,

        /// Emit Axon MIR (debug)
        #[arg(long = "emit-mir")]
        emit_mir: bool,

        /// Emit object file (.o) instead of binary
        #[arg(long = "emit-obj")]
        emit_obj: bool,

        /// GPU target (cuda, rocm, vulkan, none)
        #[arg(long, default_value = "none")]
        gpu: String,

        /// Output error messages in JSON format
        #[arg(long = "error-format", value_parser = ["json", "human"])]
        error_format: Option<String>,
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
        Commands::Build { file, output, opt_level, emit_llvm, emit_mir, emit_obj, gpu, error_format } => {
            let json_mode = error_format.as_deref() == Some("json");
            run_build(&file, output.as_deref(), &opt_level, emit_llvm, emit_mir, emit_obj, &gpu, json_mode);
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

fn run_build(file: &str, output: Option<&str>, opt_level_str: &str, emit_llvm: bool, emit_mir: bool, emit_obj: bool, gpu: &str, json_errors: bool) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    // Parse optimization level
    let opt_level = match opt_level_str {
        "0" => axonc::codegen::llvm::OptLevel::O0,
        "1" => axonc::codegen::llvm::OptLevel::O1,
        "2" => axonc::codegen::llvm::OptLevel::O2,
        "3" => axonc::codegen::llvm::OptLevel::O3,
        _ => axonc::codegen::llvm::OptLevel::O0,
    };

    // Type check
    let (typed_program, check_errors) = axonc::check_source(&source, file);

    let mut reporter = ErrorReporter::new(json_errors);
    for e in &check_errors {
        reporter.report(e.clone());
    }

    if reporter.has_errors() {
        eprint!("{}", reporter.render());
        process::exit(1);
    }

    // Build MIR
    let (checker, _) = axonc::typeck::check(&source, file);
    let mut mir_builder = axonc::mir::MirBuilder::new(&checker.interner);
    let mir_program = mir_builder.build(&typed_program);

    // If --emit-mir, print MIR and exit
    if emit_mir {
        println!("{}", mir_program);
        return;
    }

    // Check for GPU functions
    let gpu_target = axonc::codegen::mlir::GpuTarget::from_str(gpu).unwrap_or(axonc::codegen::mlir::GpuTarget::None);
    if gpu_target != axonc::codegen::mlir::GpuTarget::None && axonc::codegen::mlir::has_gpu_functions(&mir_program) {
        match axonc::codegen::mlir::compile_gpu(&mir_program, &gpu_target) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("warning: {}", e);
            }
        }
    }

    // Generate LLVM IR
    let mut codegen = axonc::codegen::llvm::LlvmCodegen::new(&checker.interner, opt_level);
    let llvm_ir = codegen.generate(&mir_program);

    // Determine output path
    let default_output = file.trim_end_matches(".axon");
    let output_path = output.unwrap_or(default_output);

    if emit_llvm {
        // Write .ll file
        let ll_path = format!("{}.ll", output_path);
        match fs::write(&ll_path, &llvm_ir) {
            Ok(_) => println!("Wrote LLVM IR to {}", ll_path),
            Err(e) => {
                eprintln!("error: could not write file '{}': {}", ll_path, e);
                process::exit(1);
            }
        }
        return;
    }

    if emit_obj {
        // Compile to object file
        match axonc::codegen::llvm::compile_ir_to_object(&llvm_ir, output_path, opt_level) {
            Ok(_) => println!("Wrote object file to {}.o", output_path),
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        }
        return;
    }

    // Default: compile to native binary
    let exe_output = if cfg!(windows) && !output_path.ends_with(".exe") {
        format!("{}.exe", output_path)
    } else {
        output_path.to_string()
    };

    match axonc::codegen::llvm::compile_ir_to_binary(&llvm_ir, &exe_output, opt_level) {
        Ok(_) => println!("Compiled to {}", exe_output),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
