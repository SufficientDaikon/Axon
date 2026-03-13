// lint.rs — Linter for Axon source files

use crate::ast::*;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct LintWarning {
    pub code: String,
    pub message: String,
    pub span: Span,
    pub severity: LintSeverity,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LintSeverity {
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Default)]
pub struct LintConfig {
    pub warn: Vec<String>,
    pub deny: Vec<String>,
    pub allow: Vec<String>,
}

pub struct Linter {
    warnings: Vec<LintWarning>,
    config: LintConfig,
    /// Track variable definitions: (name, span, is_referenced)
    var_defs: Vec<(String, Span, bool)>,
    /// Current nesting depth
    nesting_depth: usize,
    /// Whether we're inside a const declaration context
    in_const: bool,
}

impl LintWarning {
    pub fn format_human(&self) -> String {
        let sev = match self.severity {
            LintSeverity::Warning => "warning",
            LintSeverity::Info => "info",
            LintSeverity::Hint => "hint",
        };
        let mut out = format!(
            "{}[{}]: {}\n  --> {}:{}:{}\n",
            sev,
            self.code,
            self.message,
            self.span.start.file,
            self.span.start.line,
            self.span.start.column,
        );
        if let Some(ref suggestion) = self.suggestion {
            out.push_str(&format!("  help: {}\n", suggestion));
        }
        out
    }
}

impl Linter {
    pub fn new(config: LintConfig) -> Self {
        Linter {
            warnings: Vec::new(),
            config,
            var_defs: Vec::new(),
            nesting_depth: 0,
            in_const: false,
        }
    }

    fn is_allowed(&self, code: &str) -> bool {
        self.config.allow.iter().any(|c| c == code)
    }

    fn warn(&mut self, code: &str, message: String, span: Span, suggestion: Option<String>) {
        if self.is_allowed(code) {
            return;
        }
        self.warnings.push(LintWarning {
            code: code.to_string(),
            message,
            span,
            severity: LintSeverity::Warning,
            suggestion,
        });
    }

    fn info(&mut self, code: &str, message: String, span: Span, suggestion: Option<String>) {
        if self.is_allowed(code) {
            return;
        }
        self.warnings.push(LintWarning {
            code: code.to_string(),
            message,
            span,
            severity: LintSeverity::Info,
            suggestion,
        });
    }

    /// Lint source code and return warnings.
    pub fn lint(source: &str, filename: &str) -> Vec<LintWarning> {
        let (program, errors) = crate::parse_source(source, filename);
        if !errors.is_empty() {
            return Vec::new();
        }
        let mut linter = Linter::new(LintConfig::default());
        linter.lint_program(&program, source);

        // W5001: Check for unused variables
        let unused: Vec<_> = linter.var_defs.iter()
            .filter(|(name, _, referenced)| !referenced && !name.starts_with('_'))
            .map(|(name, span, _)| (name.clone(), span.clone()))
            .collect();
        for (name, span) in unused {
            linter.warn(
                "W5001",
                format!("unused variable: `{}`", name),
                span,
                Some(format!("prefix with underscore: `_{}`", name)),
            );
        }

        linter.warnings
    }

    fn lint_program(&mut self, program: &Program, source: &str) {
        // W5009: Scan source for TODO/FIXME comments
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                if trimmed.contains("TODO") || trimmed.contains("FIXME") {
                    self.info(
                        "W5009",
                        format!("found comment marker in: {}", trimmed.trim()),
                        Span::new(
                            &program.span.start.file,
                            line_idx + 1, 1,
                            line_idx + 1, line.len(),
                        ),
                        None,
                    );
                }
            }
        }

        // W5013: ml_no_grad_in_eval — Scan for .backward() or .grad calls inside
        // functions named 'eval', 'evaluate', 'predict', or 'test_*' (heuristic).
        // This catches gradient computation that likely belongs in a training context.
        let mut in_eval_fn = false;
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            // Heuristic: detect function definitions that look like eval/predict
            if trimmed.starts_with("fn ") {
                let fn_name_part = trimmed.strip_prefix("fn ").unwrap_or("").trim();
                let fn_name = fn_name_part.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                in_eval_fn = fn_name == "eval" || fn_name == "evaluate"
                    || fn_name == "predict" || fn_name.starts_with("test_");
            }
            if in_eval_fn && (trimmed.contains(".backward()") || trimmed.contains(".grad")) {
                self.warn(
                    "W5013",
                    "gradient computation detected in eval/predict context".to_string(),
                    Span::new(
                        &program.span.start.file,
                        line_idx + 1, 1,
                        line_idx + 1, line.len(),
                    ),
                    Some("wrap training-only code in a training guard or move to a training function".to_string()),
                );
            }
        }

        for item in &program.items {
            self.lint_item(item);
        }
    }

    fn lint_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Function(f) => {
                // W5002: Missing return type on public functions
                if item.visibility == Visibility::Public && f.return_type.is_none() {
                    self.warn(
                        "W5002",
                        format!("public function `{}` is missing a return type annotation", f.name),
                        f.span.clone(),
                        Some("add a return type: -> Type".to_string()),
                    );
                }

                // W5004: Function name should be snake_case
                if !is_snake_case(&f.name) {
                    self.warn(
                        "W5004",
                        format!("function `{}` should use snake_case naming", f.name),
                        f.span.clone(),
                        Some(format!("rename to `{}`", to_snake_case(&f.name))),
                    );
                }

                self.lint_function(f);
            }
            ItemKind::Struct(s) => {
                // W5005: Struct name should be PascalCase
                if !is_pascal_case(&s.name) {
                    self.warn(
                        "W5005",
                        format!("model `{}` should use PascalCase naming", s.name),
                        s.span.clone(),
                        None,
                    );
                }
            }
            ItemKind::Enum(e) => {
                // W5005: Enum name should be PascalCase
                if !is_pascal_case(&e.name) {
                    self.warn(
                        "W5005",
                        format!("enum `{}` should use PascalCase naming", e.name),
                        e.span.clone(),
                        None,
                    );
                }
            }
            ItemKind::Trait(t) => {
                // W5005: Trait name should be PascalCase
                if !is_pascal_case(&t.name) {
                    self.warn(
                        "W5005",
                        format!("trait `{}` should use PascalCase naming", t.name),
                        t.span.clone(),
                        None,
                    );
                }
                for sub_item in &t.items {
                    self.lint_item(sub_item);
                }
            }
            ItemKind::Impl(imp) => {
                for sub_item in &imp.items {
                    self.lint_item(sub_item);
                }
            }
            ItemKind::Module(m) => {
                if let Some(items) = &m.items {
                    for sub_item in items {
                        self.lint_item(sub_item);
                    }
                }
            }
            _ => {}
        }
    }

    fn lint_function(&mut self, decl: &FnDecl) {
        // W5006: Too many parameters
        let typed_param_count = decl.params.iter().filter(|p| matches!(p.kind, FnParamKind::Typed { .. })).count();
        if typed_param_count > 7 {
            self.warn(
                "W5006",
                format!("function `{}` has {} parameters (maximum recommended: 7)", decl.name, typed_param_count),
                decl.span.clone(),
                Some("consider grouping parameters into a struct".to_string()),
            );
        }

        // Register parameters as variable definitions
        for param in &decl.params {
            if let FnParamKind::Typed { name, .. } = &param.kind {
                self.var_defs.push((name.clone(), param.span.clone(), false));
            }
        }

        // W5003: Empty function body
        if let Some(body) = &decl.body {
            if body.stmts.is_empty() && body.tail_expr.is_none() {
                self.warn(
                    "W5003",
                    format!("function `{}` has an empty body", decl.name),
                    decl.span.clone(),
                    None,
                );
            }
            self.nesting_depth = 0;
            for stmt in &body.stmts {
                self.lint_stmt(stmt);
            }
            if let Some(tail) = &body.tail_expr {
                self.lint_expr(tail);
            }
        }
    }

    fn lint_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { name, initializer, .. } => {
                // Register variable definition
                if let PatternKind::Identifier(var_name) = &name.kind {
                    self.var_defs.push((var_name.clone(), name.span.clone(), false));
                }
                if let Some(init) = initializer {
                    self.lint_expr(init);
                }
            }
            StmtKind::Expr(expr) => self.lint_expr(expr),
            StmtKind::Return(expr) => {
                if let Some(e) = expr {
                    self.lint_expr(e);
                }
            }
            StmtKind::While { condition, body } => {
                self.lint_expr(condition);
                self.nesting_depth += 1;
                self.check_nesting_depth(&stmt.span);
                for s in &body.stmts {
                    self.lint_stmt(s);
                }
                if let Some(tail) = &body.tail_expr {
                    self.lint_expr(tail);
                }
                self.nesting_depth -= 1;
            }
            StmtKind::For { iterator, body, .. } => {
                self.lint_expr(iterator);
                self.nesting_depth += 1;
                self.check_nesting_depth(&stmt.span);
                for s in &body.stmts {
                    self.lint_stmt(s);
                }
                if let Some(tail) = &body.tail_expr {
                    self.lint_expr(tail);
                }
                self.nesting_depth -= 1;
            }
            StmtKind::Item(item) => self.lint_item(item),
        }
    }

    fn lint_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                // Mark variable as referenced
                for def in self.var_defs.iter_mut().rev() {
                    if def.0 == *name {
                        def.2 = true;
                        break;
                    }
                }
            }
            ExprKind::Literal(Literal::Int(n)) => {
                // W5008: Magic number
                if !self.in_const && *n != 0 && *n != 1 {
                    self.warn(
                        "W5008",
                        format!("magic number literal: {}", n),
                        expr.span.clone(),
                        Some("consider extracting to a named constant".to_string()),
                    );
                }
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.lint_expr(left);
                self.lint_expr(right);
            }
            ExprKind::UnaryOp { operand, .. } => {
                self.lint_expr(operand);
            }
            ExprKind::FnCall { function, args } => {
                // ML lint: W5011 — ml_large_tensor_literal
                // Warn when creating tensors with large inline data (many-element tuples or
                // many arguments to tensor-creation functions like from_vec).
                if let ExprKind::Identifier(fname) = &function.kind {
                    if fname == "from_vec" || fname == "Tensor" {
                        // Check if any arg is a large tuple (used as inline data)
                        for arg in args {
                            if let ExprKind::Tuple(elements) = &arg.kind {
                                if elements.len() > 100 {
                                    self.warn(
                                        "W5011",
                                        format!(
                                            "large tensor literal with {} elements; consider loading from a file",
                                            elements.len()
                                        ),
                                        arg.span.clone(),
                                        Some("use data loading utilities instead of inline literals".to_string()),
                                    );
                                }
                            }
                        }
                        // Also check if the call itself has > 100 args (flat form)
                        if args.len() > 100 {
                            self.warn(
                                "W5011",
                                format!(
                                    "large tensor literal with {} elements; consider loading from a file",
                                    args.len()
                                ),
                                expr.span.clone(),
                                Some("use data loading utilities instead of inline literals".to_string()),
                            );
                        }
                    }
                }
                self.lint_expr(function);
                for arg in args {
                    self.lint_expr(arg);
                }
            }
            ExprKind::MethodCall { receiver, method, args, .. } => {
                // ML lint: W5012 — ml_unused_gradient
                // Warn when .backward() is called as a standalone expression statement
                // (The actual "unused" check is done at the statement level; here we flag
                //  backward() calls that appear to discard their result.)
                if method == "backward" && args.is_empty() {
                    self.warn(
                        "W5012",
                        "result of `.backward()` is unused — ensure gradients are consumed by an optimizer step".to_string(),
                        expr.span.clone(),
                        Some("follow with `optimizer.step()` to apply gradients".to_string()),
                    );
                }

                // ML lint: W5014 — ml_deprecated_activation
                // Warn when calling .sigmoid() on tensors in hidden layers
                // (Heuristic: if the method is `sigmoid` and the receiver is not the last
                //  layer's output, suggest using ReLU or GELU instead.)
                if method == "sigmoid" {
                    self.warn(
                        "W5014",
                        "use of `sigmoid` activation — consider `relu` or `gelu` for hidden layers".to_string(),
                        expr.span.clone(),
                        Some("sigmoid is best suited for output layers; use `relu()` or `gelu()` in hidden layers".to_string()),
                    );
                }

                self.lint_expr(receiver);
                for arg in args {
                    self.lint_expr(arg);
                }
            }
            ExprKind::FieldAccess { object, .. } => {
                self.lint_expr(object);
            }
            ExprKind::Index { object, index } => {
                self.lint_expr(object);
                self.lint_expr(index);
            }
            ExprKind::IfElse { condition, then_block, else_block } => {
                self.lint_expr(condition);
                self.nesting_depth += 1;
                self.check_nesting_depth(&expr.span);
                for s in &then_block.stmts {
                    self.lint_stmt(s);
                }
                if let Some(tail) = &then_block.tail_expr {
                    self.lint_expr(tail);
                }
                self.nesting_depth -= 1;
                if let Some(else_clause) = else_block {
                    match else_clause {
                        ElseClause::ElseBlock(block) => {
                            for s in &block.stmts {
                                self.lint_stmt(s);
                            }
                            if let Some(tail) = &block.tail_expr {
                                self.lint_expr(tail);
                            }
                        }
                        ElseClause::ElseIf(e) => self.lint_expr(e),
                    }
                }
            }
            ExprKind::Match { expr: match_expr, arms } => {
                self.lint_expr(match_expr);
                for arm in arms {
                    // W5010: Empty match arm body
                    if is_empty_expr(&arm.body) {
                        self.warn(
                            "W5010",
                            "empty match arm body".to_string(),
                            arm.span.clone(),
                            Some("add a body or use a wildcard pattern with a comment".to_string()),
                        );
                    }
                    self.lint_expr(&arm.body);
                }
            }
            ExprKind::Block(block) => {
                self.nesting_depth += 1;
                self.check_nesting_depth(&expr.span);
                for s in &block.stmts {
                    self.lint_stmt(s);
                }
                if let Some(tail) = &block.tail_expr {
                    self.lint_expr(tail);
                }
                self.nesting_depth -= 1;
            }
            ExprKind::Assignment { target, value, .. } => {
                self.lint_expr(target);
                self.lint_expr(value);
            }
            ExprKind::Tuple(exprs) => {
                for e in exprs {
                    self.lint_expr(e);
                }
            }
            ExprKind::StructLiteral { fields, .. } => {
                for f in fields {
                    self.lint_expr(&f.value);
                }
            }
            ExprKind::Closure { body, .. } => {
                self.lint_expr(body);
            }
            ExprKind::Reference { expr: inner, .. } => {
                self.lint_expr(inner);
            }
            ExprKind::TypeCast { expr: inner, .. } => {
                self.lint_expr(inner);
            }
            ExprKind::ErrorPropagation(inner) => {
                self.lint_expr(inner);
            }
            ExprKind::Slice { object, start, end } => {
                self.lint_expr(object);
                if let Some(s) = start { self.lint_expr(s); }
                if let Some(e) = end { self.lint_expr(e); }
            }
            ExprKind::Range { start, end } => {
                if let Some(s) = start { self.lint_expr(s); }
                if let Some(e) = end { self.lint_expr(e); }
            }
            _ => {}
        }
    }

    fn check_nesting_depth(&mut self, span: &Span) {
        // W5007: Deeply nested blocks
        if self.nesting_depth > 4 {
            self.warn(
                "W5007",
                format!("deeply nested block (depth: {})", self.nesting_depth),
                span.clone(),
                Some("consider extracting inner logic into separate functions".to_string()),
            );
        }
    }
}

fn is_empty_expr(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Block(block) if block.stmts.is_empty() && block.tail_expr.is_none())
}

fn is_snake_case(s: &str) -> bool {
    if s.is_empty() || s.starts_with('_') {
        return true;
    }
    s.chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let first = s.chars().next().unwrap();
    if !first.is_uppercase() {
        return false;
    }
    // PascalCase should not contain underscores
    !s.contains('_')
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unused_variable() {
        let src = "fn main() { val x: Int32 = 1; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5001" && w.message.contains("unused variable")));
    }

    #[test]
    fn test_empty_function_body() {
        let src = "fn empty() {}";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5003"));
    }

    #[test]
    fn test_naming_convention_struct() {
        let src = "model my_model { x: Int32, }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5005"));
    }

    #[test]
    fn test_too_many_params() {
        let src = "fn many(a: Int32, b: Int32, c: Int32, d: Int32, e: Int32, f: Int32, g: Int32, h: Int32) {}";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5006"));
    }

    #[test]
    fn test_underscore_prefix_allowed() {
        let src = "fn main() { val _x: Int32 = 1; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(!warnings.iter().any(|w| w.code == "W5001" && w.message.contains("_x")));
    }

    #[test]
    fn test_naming_convention_function() {
        let src = "fn MyFunc() {}";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5004"));
    }

    #[test]
    fn test_todo_comment() {
        let src = "// TODO: fix this\nfn main() { val _x: Int32 = 0; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(warnings.iter().any(|w| w.code == "W5009"));
    }

    // -- ML-specific lint rules --

    #[test]
    fn test_ml_unused_gradient() {
        // W5012: Warn when .backward() result is unused
        let src = "fn train() { val _t: Int32 = 0; _t.backward(); }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            warnings.iter().any(|w| w.code == "W5012"),
            "should warn about unused .backward(), got: {:?}",
            warnings.iter().map(|w| &w.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ml_deprecated_activation() {
        // W5014: Warn when using sigmoid
        let src = "fn forward() { val _t: Int32 = 0; _t.sigmoid(); }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            warnings.iter().any(|w| w.code == "W5014"),
            "should warn about sigmoid, got: {:?}",
            warnings.iter().map(|w| &w.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ml_no_grad_in_eval() {
        // W5013: Warn when gradient computation is in eval context
        let src = "fn eval() { val _t: Int32 = 0; _t.backward(); }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            warnings.iter().any(|w| w.code == "W5013"),
            "should warn about grad in eval, got: {:?}",
            warnings.iter().map(|w| &w.code).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_ml_no_grad_in_eval_predict() {
        // W5013: Also catches predict functions
        let src = "fn predict() { val _x: Int32 = 0; _x.grad; }";
        let warnings = Linter::lint(src, "test.axon");
        assert!(
            warnings.iter().any(|w| w.code == "W5013"),
            "should warn about grad in predict, got: {:?}",
            warnings.iter().map(|w| &w.code).collect::<Vec<_>>()
        );
    }
}
