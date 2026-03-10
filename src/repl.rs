// repl.rs — Interactive REPL for Axon

use std::io::{self, Write, BufRead};

pub struct Repl {
    history: Vec<String>,
    bindings: Vec<String>,
    line_number: usize,
}

#[derive(Debug, PartialEq)]
pub enum ReplResult {
    Value(String),
    Definition(String),
    Error(String),
    Command,
    Exit,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            history: Vec::new(),
            bindings: Vec::new(),
            line_number: 0,
        }
    }

    /// Run the interactive REPL loop.
    pub fn run(&mut self) {
        println!("Axon REPL v0.1.0");
        println!("Type :help for available commands, :quit to exit.");
        println!();

        let stdin = io::stdin();
        loop {
            self.line_number += 1;
            print!("axon:{:03}> ", self.line_number);
            io::stdout().flush().ok();

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error reading input: {}", e);
                    break;
                }
            }

            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            self.history.push(line.clone());

            match self.eval_line(&line) {
                ReplResult::Value(v) => println!("= {}", v),
                ReplResult::Definition(d) => println!("defined: {}", d),
                ReplResult::Error(e) => eprintln!("{}", e),
                ReplResult::Command => {}
                ReplResult::Exit => break,
            }
        }
    }

    /// Evaluate a single line of input.
    pub fn eval_line(&mut self, line: &str) -> ReplResult {
        let trimmed = line.trim();

        // Handle REPL commands
        if trimmed.starts_with(':') {
            return self.handle_command(trimmed);
        }

        // Check if this is a let/fn definition
        if trimmed.starts_with("let ") || trimmed.starts_with("let\t") {
            self.bindings.push(trimmed.to_string());
            let name = extract_let_name(trimmed);
            return ReplResult::Definition(name);
        }

        if trimmed.starts_with("fn ") || trimmed.starts_with("fn\t") {
            self.bindings.push(trimmed.to_string());
            let name = extract_fn_name(trimmed);
            return ReplResult::Definition(name);
        }

        if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") || trimmed.starts_with("trait ") {
            self.bindings.push(trimmed.to_string());
            let name = trimmed.split_whitespace().nth(1).unwrap_or("?").to_string();
            return ReplResult::Definition(name);
        }

        // Expression: wrap in a synthetic function, type-check, and report the type.
        // NOTE: JIT evaluation is deferred to Phase 12. For now, this is a
        // type-evaluation REPL — we show the type of the expression rather than
        // its runtime value, which is still useful for exploring the type system.
        self.eval_expression(trimmed)
    }

    fn handle_command(&mut self, cmd: &str) -> ReplResult {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match command {
            ":quit" | ":q" => ReplResult::Exit,
            ":help" | ":h" => {
                self.print_help();
                ReplResult::Command
            }
            ":clear" => {
                self.bindings.clear();
                println!("All bindings cleared.");
                ReplResult::Command
            }
            ":type" | ":t" => {
                if arg.is_empty() {
                    return ReplResult::Error("usage: :type <expr>".to_string());
                }
                self.eval_expression(arg)
            }
            ":ast" => {
                if arg.is_empty() {
                    return ReplResult::Error("usage: :ast <code>".to_string());
                }
                // Wrap in function to parse
                let wrapped = self.wrap_expression(arg);
                let (program, errors) = crate::parse_source(&wrapped, "<repl>");
                if !errors.is_empty() {
                    // Try parsing as top-level item
                    let (program2, errors2) = crate::parse_source(arg, "<repl>");
                    if !errors2.is_empty() {
                        return ReplResult::Error(
                            errors2.iter().map(|e| e.format_human()).collect::<String>()
                        );
                    }
                    println!("{}", crate::ast_to_json(&program2));
                } else {
                    println!("{}", crate::ast_to_json(&program));
                }
                ReplResult::Command
            }
            ":load" => {
                if arg.is_empty() {
                    return ReplResult::Error("usage: :load <file>".to_string());
                }
                match std::fs::read_to_string(arg) {
                    Ok(source) => {
                        let (_, errors) = crate::parse_source(&source, arg);
                        if !errors.is_empty() {
                            return ReplResult::Error(
                                errors.iter().map(|e| e.format_human()).collect::<String>()
                            );
                        }
                        self.bindings.push(source);
                        ReplResult::Definition(format!("loaded {}", arg))
                    }
                    Err(e) => ReplResult::Error(format!("could not read '{}': {}", arg, e)),
                }
            }
            ":save" | ":s" => {
                if arg.is_empty() {
                    return ReplResult::Error("usage: :save <filename>".to_string());
                }
                self.save_history(arg)
            }
            _ => ReplResult::Error(format!("unknown command: {}", command)),
        }
    }

    fn wrap_expression(&self, expr: &str) -> String {
        let mut source = String::new();
        for binding in &self.bindings {
            source.push_str(binding);
            if !binding.ends_with('}') && !binding.ends_with(';') {
                source.push(';');
            }
            source.push('\n');
        }
        source.push_str(&format!("fn __repl_eval__() {{ {}; }}\n", expr));
        source
    }

    /// Evaluate an expression by wrapping it in a synthetic function,
    /// type-checking, and extracting the inferred type.
    fn eval_expression(&self, expr: &str) -> ReplResult {
        let source = self.wrap_expression(expr);
        let (_, parse_errors) = crate::parse_source(&source, "<repl>");
        if !parse_errors.is_empty() {
            return ReplResult::Error(
                parse_errors.iter().map(|e| e.format_human()).collect::<String>()
            );
        }

        // Type check the wrapped source to infer the expression's type
        let (checker, check_errors) = crate::typeck::check(&source, "<repl>");

        // Look up the synthetic __repl_eval__ function to get its resolved type
        if let Some(sym_id) = checker.symbols.lookup("__repl_eval__") {
            let sym = checker.symbols.get_symbol(sym_id);
            let fn_ty = checker.interner.resolve(sym.ty);
            if let crate::types::Type::Function { ret, .. } = fn_ty {
                let ret_display = format!("{}", checker.interner.resolve(*ret));
                // Only report type errors if they are real errors (not just warnings)
                let has_hard_errors = check_errors.iter().any(|e| {
                    e.severity == crate::error::Severity::Error
                });
                if has_hard_errors {
                    // Still try to show the type, with a warning note
                    return ReplResult::Value(format!("{} : {} (type check had errors)", expr, ret_display));
                }
                return ReplResult::Value(format!("{} : {}", expr, ret_display));
            }
        }

        // Fallback: if we couldn't resolve the function, report based on errors
        if !check_errors.is_empty() {
            let has_hard_errors = check_errors.iter().any(|e| {
                e.severity == crate::error::Severity::Error
            });
            if has_hard_errors {
                return ReplResult::Value(format!("{} (type check had errors)", expr));
            }
        }

        ReplResult::Value(format!("{} : (type inferred)", expr))
    }

    /// Save the REPL history to a file.
    fn save_history(&self, filename: &str) -> ReplResult {
        let content = self.history.join("\n");
        match std::fs::write(filename, content + "\n") {
            Ok(_) => {
                println!("Saved {} line(s) to {}", self.history.len(), filename);
                ReplResult::Command
            }
            Err(e) => ReplResult::Error(format!("could not write '{}': {}", filename, e)),
        }
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  :help, :h       Show this help message");
        println!("  :type <expr>    Show the type of an expression");
        println!("  :ast <code>     Show the parsed AST of code");
        println!("  :clear          Clear all accumulated bindings");
        println!("  :load <file>    Load and evaluate an Axon source file");
        println!("  :save <file>    Save REPL history to a file");
        println!("  :quit, :q       Exit the REPL");
        println!();
        println!("Enter expressions, let bindings, or function definitions.");
        println!("Expressions display their inferred type (e.g. `1 + 2` → `1 + 2 : Int64`).");
        // NOTE: JIT evaluation is deferred to Phase 12.
    }

    /// Get tab completions for a partial input prefix.
    /// Returns a sorted list of matching identifiers from keywords, built-in types,
    /// built-in functions, and currently defined session bindings.
    pub fn tab_complete(&self, prefix: &str) -> Vec<String> {
        let prefix = prefix.trim();
        if prefix.is_empty() {
            return Vec::new();
        }

        let keywords = [
            "fn", "let", "mut", "if", "else", "while", "for", "in",
            "match", "return", "struct", "enum", "trait", "impl",
            "pub", "use", "mod", "type", "as", "true", "false", "self",
            "break", "continue", "unsafe",
        ];

        let builtin_types = [
            "Int8", "Int16", "Int32", "Int64",
            "UInt8", "UInt16", "UInt32", "UInt64",
            "Float16", "Float32", "Float64",
            "Bool", "Char", "String", "Tensor",
            "Vec", "HashMap", "HashSet", "Option", "Result",
        ];

        let builtin_functions = [
            "print", "println", "sin", "cos", "sqrt", "abs",
            "zeros", "ones", "randn", "matmul", "relu", "softmax",
        ];

        let mut completions: Vec<String> = Vec::new();

        for kw in &keywords {
            if kw.starts_with(prefix) {
                completions.push(kw.to_string());
            }
        }
        for ty in &builtin_types {
            if ty.starts_with(prefix) {
                completions.push(ty.to_string());
            }
        }
        for func in &builtin_functions {
            if func.starts_with(prefix) {
                completions.push(func.to_string());
            }
        }

        // Add session-defined names
        for binding in &self.bindings {
            let trimmed = binding.trim();
            if trimmed.starts_with("let ") || trimmed.starts_with("let\t") {
                let name = extract_let_name(trimmed);
                if name.starts_with(prefix) && name != "?" {
                    completions.push(name);
                }
            } else if trimmed.starts_with("fn ") || trimmed.starts_with("fn\t") {
                let name = extract_fn_name(trimmed);
                if name.starts_with(prefix) && name != "?" {
                    completions.push(name);
                }
            } else if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") || trimmed.starts_with("trait ") {
                let name = trimmed.split_whitespace().nth(1).unwrap_or("").to_string();
                if name.starts_with(prefix) && !name.is_empty() {
                    completions.push(name);
                }
            }
        }

        completions.sort();
        completions.dedup();
        completions
    }
}

fn extract_let_name(s: &str) -> String {
    // "let x: Int32 = 5;" -> "x"
    let rest = s.strip_prefix("let").unwrap_or(s).trim();
    let rest = rest.strip_prefix("mut").map(|r| r.trim()).unwrap_or(rest);
    rest.split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("?")
        .to_string()
}

fn extract_fn_name(s: &str) -> String {
    // "fn foo(...)" -> "foo"
    let rest = s.strip_prefix("fn").unwrap_or(s).trim();
    rest.split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("?")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_expression() {
        let mut repl = Repl::new();
        let result = repl.eval_line("1 + 2");
        match result {
            ReplResult::Value(v) => assert!(v.contains("1 + 2"), "got: {}", v),
            other => panic!("expected Value, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_expression_shows_type() {
        let mut repl = Repl::new();
        let result = repl.eval_line("1 + 2");
        match result {
            ReplResult::Value(v) => {
                // Should contain the expression and a colon separator
                assert!(v.contains("1 + 2"), "should echo expression, got: {}", v);
                assert!(v.contains(":"), "should contain type separator, got: {}", v);
            }
            other => panic!("expected Value, got {:?}", other),
        }
    }

    #[test]
    fn test_eval_let_binding() {
        let mut repl = Repl::new();
        let result = repl.eval_line("let x: Int32 = 42;");
        assert_eq!(result, ReplResult::Definition("x".to_string()));
        assert_eq!(repl.bindings.len(), 1);
    }

    #[test]
    fn test_eval_function_def() {
        let mut repl = Repl::new();
        let result = repl.eval_line("fn add(a: Int32, b: Int32) -> Int32 { return a + b; }");
        assert_eq!(result, ReplResult::Definition("add".to_string()));
    }

    #[test]
    fn test_command_help() {
        let mut repl = Repl::new();
        let result = repl.eval_line(":help");
        assert_eq!(result, ReplResult::Command);
    }

    #[test]
    fn test_command_type() {
        let mut repl = Repl::new();
        let result = repl.eval_line(":type 42");
        match result {
            ReplResult::Value(v) => assert!(v.contains("42")),
            other => panic!("expected Value, got {:?}", other),
        }
    }

    #[test]
    fn test_command_quit() {
        let mut repl = Repl::new();
        assert_eq!(repl.eval_line(":quit"), ReplResult::Exit);
        assert_eq!(repl.eval_line(":q"), ReplResult::Exit);
    }

    #[test]
    fn test_command_clear() {
        let mut repl = Repl::new();
        repl.eval_line("let x: Int32 = 1;");
        assert_eq!(repl.bindings.len(), 1);
        repl.eval_line(":clear");
        assert_eq!(repl.bindings.len(), 0);
    }

    #[test]
    fn test_unknown_command() {
        let mut repl = Repl::new();
        let result = repl.eval_line(":foobar");
        match result {
            ReplResult::Error(e) => assert!(e.contains("unknown command")),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn test_command_save() {
        let mut repl = Repl::new();
        repl.history.push("let x: Int32 = 1;".to_string());
        repl.history.push("x + 2".to_string());

        let tmp = std::env::temp_dir().join("axon_repl_test.axon");
        let result = repl.eval_line(&format!(":save {}", tmp.display()));
        assert_eq!(result, ReplResult::Command);

        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("let x: Int32 = 1;"));
        assert!(content.contains("x + 2"));

        // Cleanup
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_command_save_no_arg() {
        let mut repl = Repl::new();
        let result = repl.eval_line(":save");
        match result {
            ReplResult::Error(e) => assert!(e.contains("usage")),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn test_tab_complete_keywords() {
        let repl = Repl::new();
        let completions = repl.tab_complete("fn");
        assert!(completions.contains(&"fn".to_string()));
    }

    #[test]
    fn test_tab_complete_partial_keyword() {
        let repl = Repl::new();
        let completions = repl.tab_complete("wh");
        assert!(completions.contains(&"while".to_string()));
    }

    #[test]
    fn test_tab_complete_types() {
        let repl = Repl::new();
        let completions = repl.tab_complete("Int");
        assert!(completions.contains(&"Int32".to_string()));
        assert!(completions.contains(&"Int64".to_string()));
    }

    #[test]
    fn test_tab_complete_builtins() {
        let repl = Repl::new();
        let completions = repl.tab_complete("pr");
        assert!(completions.contains(&"print".to_string()));
        assert!(completions.contains(&"println".to_string()));
    }

    #[test]
    fn test_tab_complete_session_bindings() {
        let mut repl = Repl::new();
        repl.eval_line("let my_var: Int32 = 42;");
        repl.eval_line("fn my_func() {}");
        let completions = repl.tab_complete("my");
        assert!(completions.contains(&"my_var".to_string()));
        assert!(completions.contains(&"my_func".to_string()));
    }

    #[test]
    fn test_tab_complete_empty_prefix() {
        let repl = Repl::new();
        let completions = repl.tab_complete("");
        assert!(completions.is_empty(), "empty prefix should return no completions");
    }

    #[test]
    fn test_tab_complete_no_matches() {
        let repl = Repl::new();
        let completions = repl.tab_complete("zzz_nonexistent");
        assert!(completions.is_empty(), "should have no matches");
    }
}
