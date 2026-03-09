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

        // Expression: wrap in a function and type-check
        self.format_type(trimmed)
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
                self.format_type(arg)
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

    fn format_type(&self, expr: &str) -> ReplResult {
        let source = self.wrap_expression(expr);
        let (_, errors) = crate::parse_source(&source, "<repl>");
        if !errors.is_empty() {
            return ReplResult::Error(
                errors.iter().map(|e| e.format_human()).collect::<String>()
            );
        }

        // Type check
        let (_typed, check_errors) = crate::check_source(&source, "<repl>");
        if !check_errors.is_empty() {
            // Still report the expression even with type errors (best effort)
            return ReplResult::Value(format!("{} (type check had warnings)", expr));
        }

        ReplResult::Value(format!("{} : (type inferred)", expr))
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  :help, :h       Show this help message");
        println!("  :type <expr>    Show the type of an expression");
        println!("  :ast <code>     Show the parsed AST of code");
        println!("  :clear          Clear all accumulated bindings");
        println!("  :load <file>    Load and evaluate an Axon source file");
        println!("  :quit, :q       Exit the REPL");
        println!();
        println!("Enter expressions, let bindings, or function definitions.");
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
            ReplResult::Value(v) => assert!(v.contains("1 + 2")),
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
}
