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

    /// Format an Axon source file
    Fmt {
        /// The source file to format
        file: String,
    },

    /// Lint an Axon source file
    Lint {
        /// The source file to lint
        file: String,
    },

    /// Start the Axon REPL
    Repl {},

    /// Generate documentation for an Axon source file
    Doc {
        /// The source file to document
        file: String,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Start the Axon Language Server (LSP over stdio)
    Lsp {},

    /// Launch the debugger for an Axon source file
    Debug {
        /// The source file to debug
        file: String,
    },

    /// Package manager commands
    Pkg {
        #[command(subcommand)]
        action: PkgAction,
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

        /// Keep temporary build files
        #[arg(long = "keep-temps")]
        keep_temps: bool,

        /// Output error messages in JSON format
        #[arg(long = "error-format", value_parser = ["json", "human"])]
        error_format: Option<String>,
    },
}

#[derive(Subcommand)]
enum PkgAction {
    /// Create a new Axon project
    New { name: String },
    /// Initialize Axon project in current directory
    Init,
    /// Build the current project
    Build,
    /// Run the current project
    Run,
    /// Run tests
    Test,
    /// Add a dependency
    Add {
        package: String,
        #[arg(long)]
        version: Option<String>,
    },
    /// Remove a dependency
    Remove { package: String },
    /// Clean build artifacts
    Clean,
    /// Format source files
    Fmt,
    /// Lint source files
    Lint,
    /// Publish package to the registry
    Publish,
    /// Search the package registry
    Search { query: String },
    /// Update dependencies to latest compatible versions
    Update,
    /// Run benchmarks
    Bench,
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
        Commands::Fmt { file } => {
            run_fmt(&file);
        }
        Commands::Lint { file } => {
            run_lint(&file);
        }
        Commands::Repl {} => {
            run_repl();
        }
        Commands::Doc { file, output } => {
            run_doc(&file, output.as_deref());
        }
        Commands::Lsp {} => {
            run_lsp();
        }
        Commands::Debug { file } => {
            run_debug(&file);
        }
        Commands::Pkg { action } => {
            run_pkg(action);
        }
        Commands::Build { file, output, opt_level, emit_llvm, emit_mir, emit_obj, gpu, keep_temps, error_format } => {
            let json_mode = error_format.as_deref() == Some("json");
            run_build(&file, output.as_deref(), &opt_level, emit_llvm, emit_mir, emit_obj, &gpu, json_mode, keep_temps);
        }
    }
}

fn run_pkg(action: PkgAction) {
    let result = match action {
        PkgAction::New { name } => axonc::pkg::commands::cmd_new(&name),
        PkgAction::Init => axonc::pkg::commands::cmd_init(),
        PkgAction::Build => axonc::pkg::commands::cmd_build(),
        PkgAction::Run => axonc::pkg::commands::cmd_run(),
        PkgAction::Test => axonc::pkg::commands::cmd_test(),
        PkgAction::Add { package, version } => {
            axonc::pkg::commands::cmd_add(&package, version.as_deref())
        }
        PkgAction::Remove { package } => axonc::pkg::commands::cmd_remove(&package),
        PkgAction::Clean => axonc::pkg::commands::cmd_clean(),
        PkgAction::Fmt => axonc::pkg::commands::cmd_fmt(),
        PkgAction::Lint => axonc::pkg::commands::cmd_lint(),
        PkgAction::Publish => axonc::pkg::commands::cmd_publish(),
        PkgAction::Search { query } => axonc::pkg::commands::cmd_search(&query),
        PkgAction::Update => axonc::pkg::commands::cmd_update(),
        PkgAction::Bench => axonc::pkg::commands::cmd_bench(),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        process::exit(1);
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

fn run_build(file: &str, output: Option<&str>, opt_level_str: &str, emit_llvm: bool, emit_mir: bool, emit_obj: bool, gpu: &str, json_errors: bool, keep_temps: bool) {
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
    let (typed_program, check_errors, checker) = axonc::check_source_full(&source, file);

    let mut reporter = ErrorReporter::new(json_errors);
    for e in &check_errors {
        reporter.report(e.clone());
    }

    if reporter.has_errors() {
        eprint!("{}", reporter.render());
        process::exit(1);
    }

    // Build MIR
    let mut mir_builder = axonc::mir::MirBuilder::new(&checker.interner);
    let mir_program = mir_builder.build(&typed_program);

    // If --emit-mir, print MIR and exit
    if emit_mir {
        println!("{}", mir_program);
        return;
    }

    // Validate that a main function exists (E5009)
    let has_main = mir_program.functions.iter().any(|f| f.name == "main");
    if !has_main {
        eprintln!("error[E5009]: no `main` function found. Every Axon executable must have a `fn main() {{ ... }}`");
        process::exit(1);
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
    let llvm_ir = match codegen.generate(&mir_program) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("error[E5009]: {}", e);
            process::exit(1);
        }
    };

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
            Ok(_) => {
                // Write companion runtime C source for manual linking
                let rt_c_path = format!("{}.runtime.c", output_path);
                let rt_source = axonc::codegen::runtime::generate_runtime_c_source();
                match fs::write(&rt_c_path, &rt_source) {
                    Ok(_) => {
                        println!("Wrote object file to {}.o", output_path);
                        println!("Wrote runtime to {}", rt_c_path);
                        println!("Link with runtime: clang {}.o {} -o {}", output_path, rt_c_path, output_path);
                    }
                    Err(e) => {
                        println!("Wrote object file to {}.o", output_path);
                        eprintln!("warning: could not write runtime source '{}': {}", rt_c_path, e);
                    }
                }
            }
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

    match axonc::codegen::llvm::compile_and_link(&llvm_ir, &exe_output, opt_level, keep_temps) {
        Ok(_) => println!("Compiled to {}", exe_output),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

fn run_fmt(file: &str) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    match axonc::fmt::Formatter::format(&source, file) {
        Ok(formatted) => {
            match fs::write(file, &formatted) {
                Ok(_) => println!("Formatted {}", file),
                Err(e) => {
                    eprintln!("error: could not write file '{}': {}", file, e);
                    process::exit(1);
                }
            }
        }
        Err(errors) => {
            for e in &errors {
                eprint!("{}", e.format_human());
            }
            process::exit(1);
        }
    }
}

fn run_lint(file: &str) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    let warnings = axonc::lint::Linter::lint(&source, file);
    if warnings.is_empty() {
        println!("OK: no lint warnings");
    } else {
        for w in &warnings {
            eprint!("{}", w.format_human());
        }
        println!("{} warning(s) emitted", warnings.len());
    }
}

fn run_repl() {
    let mut repl = axonc::repl::Repl::new();
    repl.run();
}

fn run_doc(file: &str, output: Option<&str>) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file, e);
            process::exit(1);
        }
    };

    let html = axonc::doc::DocGenerator::generate(&source, file);

    if let Some(out_path) = output {
        match fs::write(out_path, &html) {
            Ok(_) => println!("Wrote documentation to {}", out_path),
            Err(e) => {
                eprintln!("error: could not write file '{}': {}", out_path, e);
                process::exit(1);
            }
        }
    } else {
        println!("{}", html);
    }
}

fn run_lsp() {
    let mut server = axonc::lsp::LspServer::new();
    server.run();
}

fn run_debug(_file: &str) {
    eprintln!("Debugger support is coming in a future release.");
    eprintln!("Use --emit-llvm and lldb/gdb for now:");
    eprintln!("  axonc build --emit-llvm {}", _file);
    eprintln!("  # Then debug the generated IR with your preferred debugger.");
    // See src/debugger.rs for the planned DAP architecture.
}
