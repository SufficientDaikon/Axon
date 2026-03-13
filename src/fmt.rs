// fmt.rs — Code formatter for Axon source files

use crate::ast::*;
use crate::error::CompileError;

pub struct Formatter {
    indent: usize,
    output: String,
    #[allow(dead_code)]
    line_width: usize,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            indent: 0,
            output: String::new(),
            line_width: 100,
        }
    }

    /// Format Axon source code by parsing to AST and re-emitting.
    pub fn format(source: &str, filename: &str) -> Result<String, Vec<CompileError>> {
        let (program, errors) = crate::parse_source(source, filename);
        if !errors.is_empty() {
            return Err(errors);
        }
        let mut fmt = Formatter::new();
        fmt.format_program(&program);
        Ok(fmt.output.trim_end().to_string() + "\n")
    }

    fn format_program(&mut self, program: &Program) {
        for (i, item) in program.items.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.format_item(item);
        }
    }

    fn format_item(&mut self, item: &Item) {
        for attr in &item.attributes {
            self.write_indent();
            self.format_attribute(attr);
            self.newline();
        }
        let vis_prefix = match item.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };
        match &item.kind {
            ItemKind::Function(f) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_function(f);
            }
            ItemKind::Struct(s) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_struct(s);
            }
            ItemKind::Enum(e) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_enum(e);
            }
            ItemKind::Impl(imp) => {
                self.write_indent();
                self.format_impl_block(imp);
            }
            ItemKind::Trait(t) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_trait_def(t);
            }
            ItemKind::TypeAlias(ta) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_type_alias(ta);
            }
            ItemKind::Module(m) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_module(m);
            }
            ItemKind::Use(u) => {
                self.write_indent();
                self.write(vis_prefix);
                self.format_use(u);
            }
        }
    }

    fn format_attribute(&mut self, attr: &Attribute) {
        // Axon uses @-prefixed device annotations (not #[...] Rust-style attributes).
        match attr {
            Attribute::Cpu => self.write("@cpu"),
            Attribute::Gpu => self.write("@gpu"),
            Attribute::Device(expr) => {
                self.write("@device(");
                self.format_expr(expr);
                self.write(")");
            }
        }
    }

    fn format_function(&mut self, decl: &FnDecl) {
        self.write("fn ");
        self.write(&decl.name);
        self.format_generics(&decl.generics);
        self.write("(");
        for (i, param) in decl.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.format_fn_param(param);
        }
        self.write(")");
        if let Some(ret) = &decl.return_type {
            self.write(": ");
            self.format_type_expr(ret);
        }
        if let Some(body) = &decl.body {
            self.write(" ");
            self.format_block_braces(body);
        } else {
            self.write(";");
        }
        self.newline();
    }

    fn format_generics(&mut self, generics: &[GenericParam]) {
        if generics.is_empty() {
            return;
        }
        self.write("<");
        for (i, g) in generics.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&g.name);
            if !g.bounds.is_empty() {
                self.write(": ");
                for (j, b) in g.bounds.iter().enumerate() {
                    if j > 0 {
                        self.write(" + ");
                    }
                    self.format_type_expr(b);
                }
            }
        }
        self.write(">");
    }

    fn format_fn_param(&mut self, param: &FnParam) {
        match &param.kind {
            FnParamKind::Typed { name, ty, default } => {
                self.write(name);
                self.write(": ");
                self.format_type_expr(ty);
                if let Some(def) = default {
                    self.write(" = ");
                    self.format_expr(def);
                }
            }
            FnParamKind::SelfOwned => self.write("self"),
            FnParamKind::SelfRef => self.write("&self"),
            FnParamKind::SelfMutRef => self.write("&mut self"),
        }
    }

    fn format_struct(&mut self, s: &StructDecl) {
        self.write("model ");
        self.write(&s.name);
        self.format_generics(&s.generics);
        self.write(" {");
        self.newline();
        self.indent();
        for field in &s.fields {
            self.write_indent();
            if field.visibility == Visibility::Public {
                self.write("pub ");
            }
            self.write(&field.name);
            self.write(": ");
            self.format_type_expr(&field.ty);
            self.write(",");
            self.newline();
        }
        self.dedent();
        self.write_indent();
        self.write("}");
        self.newline();
    }

    fn format_enum(&mut self, e: &EnumDecl) {
        self.write("enum ");
        self.write(&e.name);
        self.format_generics(&e.generics);
        self.write(" {");
        self.newline();
        self.indent();
        for variant in &e.variants {
            self.write_indent();
            self.write(&variant.name);
            match &variant.fields {
                EnumVariantKind::Unit => {}
                EnumVariantKind::Tuple(types) => {
                    self.write("(");
                    for (i, ty) in types.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.format_type_expr(ty);
                    }
                    self.write(")");
                }
                EnumVariantKind::Struct(fields) => {
                    self.write(" {");
                    for (i, f) in fields.iter().enumerate() {
                        if i > 0 {
                            self.write(",");
                        }
                        self.write(" ");
                        self.write(&f.name);
                        self.write(": ");
                        self.format_type_expr(&f.ty);
                    }
                    self.write(" }");
                }
            }
            self.write(",");
            self.newline();
        }
        self.dedent();
        self.write_indent();
        self.write("}");
        self.newline();
    }

    fn format_impl_block(&mut self, imp: &ImplBlock) {
        self.write("extend");
        self.format_generics(&imp.generics);
        self.write(" ");
        if let Some(trait_name) = &imp.trait_name {
            self.format_type_expr(trait_name);
            self.write(" for ");
        }
        self.format_type_expr(&imp.type_name);
        self.write(" {");
        self.newline();
        self.indent();
        for (i, item) in imp.items.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.format_item(item);
        }
        self.dedent();
        self.write_indent();
        self.write("}");
        self.newline();
    }

    fn format_trait_def(&mut self, t: &TraitDecl) {
        self.write("trait ");
        self.write(&t.name);
        self.format_generics(&t.generics);
        if !t.supertraits.is_empty() {
            self.write(": ");
            for (i, st) in t.supertraits.iter().enumerate() {
                if i > 0 {
                    self.write(" + ");
                }
                self.format_type_expr(st);
            }
        }
        self.write(" {");
        self.newline();
        self.indent();
        for (i, item) in t.items.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.format_item(item);
        }
        self.dedent();
        self.write_indent();
        self.write("}");
        self.newline();
    }

    fn format_type_alias(&mut self, ta: &TypeAliasDecl) {
        self.write("type ");
        self.write(&ta.name);
        self.format_generics(&ta.generics);
        self.write(" = ");
        self.format_type_expr(&ta.ty);
        self.write(";");
        self.newline();
    }

    fn format_module(&mut self, m: &ModuleDecl) {
        self.write("mod ");
        self.write(&m.name);
        if let Some(items) = &m.items {
            self.write(" {");
            self.newline();
            self.indent();
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    self.newline();
                }
                self.format_item(item);
            }
            self.dedent();
            self.write_indent();
            self.write("}");
        } else {
            self.write(";");
        }
        self.newline();
    }

    fn format_use(&mut self, u: &UseDecl) {
        self.write("use ");
        self.write(&u.path.join("."));
        if let Some(alias) = &u.alias {
            self.write(" as ");
            self.write(alias);
        }
        self.write(";");
        self.newline();
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { name, mutable, ty, initializer } => {
                self.write_indent();
                if *mutable {
                    self.write("var ");
                } else {
                    self.write("val ");
                }
                self.format_pattern(name);
                if let Some(ty) = ty {
                    self.write(": ");
                    self.format_type_expr(ty);
                }
                if let Some(init) = initializer {
                    self.write(" = ");
                    self.format_expr(init);
                }
                self.write(";");
                self.newline();
            }
            StmtKind::Expr(expr) => {
                self.write_indent();
                self.format_expr(expr);
                self.write(";");
                self.newline();
            }
            StmtKind::Return(expr) => {
                self.write_indent();
                self.write("return");
                if let Some(e) = expr {
                    self.write(" ");
                    self.format_expr(e);
                }
                self.write(";");
                self.newline();
            }
            StmtKind::While { condition, body } => {
                self.write_indent();
                self.write("while ");
                self.format_expr(condition);
                self.write(" ");
                self.format_block_braces(body);
                self.newline();
            }
            StmtKind::For { pattern, iterator, body } => {
                self.write_indent();
                self.write("for ");
                self.format_pattern(pattern);
                self.write(" in ");
                self.format_expr(iterator);
                self.write(" ");
                self.format_block_braces(body);
                self.newline();
            }
            StmtKind::Item(item) => {
                self.format_item(item);
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(lit) => self.format_literal(lit),
            ExprKind::Identifier(name) => self.write(name),
            ExprKind::Path(parts) => self.write(&parts.join(".")),
            ExprKind::BinaryOp { left, op, right } => {
                self.format_expr(left);
                self.write(" ");
                self.write(Self::binop_str(op));
                self.write(" ");
                self.format_expr(right);
            }
            ExprKind::UnaryOp { op, operand } => {
                self.write(Self::unaryop_str(op));
                self.format_expr(operand);
            }
            ExprKind::FnCall { function, args } => {
                self.format_expr(function);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_expr(arg);
                }
                self.write(")");
            }
            ExprKind::MethodCall { receiver, method, args } => {
                self.format_expr(receiver);
                self.write(".");
                self.write(method);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_expr(arg);
                }
                self.write(")");
            }
            ExprKind::FieldAccess { object, field } => {
                self.format_expr(object);
                self.write(".");
                self.write(field);
            }
            ExprKind::Index { object, index } => {
                self.format_expr(object);
                self.write("[");
                self.format_expr(index);
                self.write("]");
            }
            ExprKind::Slice { object, start, end } => {
                self.format_expr(object);
                self.write("[");
                if let Some(s) = start {
                    self.format_expr(s);
                }
                self.write("..");
                if let Some(e) = end {
                    self.format_expr(e);
                }
                self.write("]");
            }
            ExprKind::IfElse { condition, then_block, else_block } => {
                self.write("if ");
                self.format_expr(condition);
                self.write(" ");
                self.format_block_braces(then_block);
                if let Some(else_clause) = else_block {
                    self.write(" else ");
                    match else_clause {
                        ElseClause::ElseBlock(block) => {
                            self.format_block_braces(block);
                        }
                        ElseClause::ElseIf(expr) => {
                            self.format_expr(expr);
                        }
                    }
                }
            }
            ExprKind::Match { expr, arms } => {
                self.write("match ");
                self.format_expr(expr);
                self.write(" {");
                self.newline();
                self.indent();
                for arm in arms {
                    self.write_indent();
                    self.format_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.write(" if ");
                        self.format_expr(guard);
                    }
                    self.write(" => ");
                    self.format_expr(&arm.body);
                    self.write(",");
                    self.newline();
                }
                self.dedent();
                self.write_indent();
                self.write("}");
            }
            ExprKind::Block(block) => {
                self.format_block_braces(block);
            }
            ExprKind::Reference { mutable, expr } => {
                if *mutable {
                    self.write("&mut ");
                } else {
                    self.write("&");
                }
                self.format_expr(expr);
            }
            ExprKind::TypeCast { expr, target_type } => {
                self.format_expr(expr);
                self.write(" as ");
                self.format_type_expr(target_type);
            }
            ExprKind::ErrorPropagation(expr) => {
                self.format_expr(expr);
                self.write("?");
            }
            ExprKind::Range { start, end } => {
                if let Some(s) = start {
                    self.format_expr(s);
                }
                self.write("..");
                if let Some(e) = end {
                    self.format_expr(e);
                }
            }
            ExprKind::Assignment { target, op, value } => {
                self.format_expr(target);
                self.write(" ");
                self.write(Self::assignop_str(op));
                self.write(" ");
                self.format_expr(value);
            }
            ExprKind::Tuple(exprs) => {
                self.write("(");
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_expr(e);
                }
                self.write(")");
            }
            ExprKind::StructLiteral { name, fields } => {
                self.write(&name.join("."));
                self.write(" { ");
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&f.name);
                    self.write(": ");
                    self.format_expr(&f.value);
                }
                self.write(" }");
            }
            ExprKind::Closure { params, return_type, body } => {
                self.write("|");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&p.name);
                    if let Some(ty) = &p.ty {
                        self.write(": ");
                        self.format_type_expr(ty);
                    }
                }
                self.write("|");
                if let Some(ret) = return_type {
                    self.write(": ");
                    self.format_type_expr(ret);
                }
                self.write(" ");
                self.format_expr(body);
            }
        }
    }

    fn format_type_expr(&mut self, ty: &TypeExpr) {
        match &ty.kind {
            TypeExprKind::Named(name) => self.write(name),
            TypeExprKind::Path(parts) => self.write(&parts.join(".")),
            TypeExprKind::Generic { name, args } => {
                self.write(name);
                self.write("<");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    match arg {
                        TypeArg::Type(t) => self.format_type_expr(t),
                        TypeArg::Shape(dims) => {
                            self.write("[");
                            for (j, dim) in dims.iter().enumerate() {
                                if j > 0 {
                                    self.write(", ");
                                }
                                self.format_shape_dim(dim);
                            }
                            self.write("]");
                        }
                    }
                }
                self.write(">");
            }
            TypeExprKind::Tensor { dtype, shape } => {
                self.write("Tensor<");
                self.format_type_expr(dtype);
                self.write(", [");
                for (i, dim) in shape.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_shape_dim(dim);
                }
                self.write("]>");
            }
            TypeExprKind::Reference { mutable, inner } => {
                if *mutable {
                    self.write("&mut ");
                } else {
                    self.write("&");
                }
                self.format_type_expr(inner);
            }
            TypeExprKind::Function { params, return_type } => {
                self.write("fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_type_expr(p);
                }
                self.write("): ");
                self.format_type_expr(return_type);
            }
            TypeExprKind::Tuple(types) => {
                self.write("(");
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_type_expr(t);
                }
                self.write(")");
            }
            TypeExprKind::Array { element, size } => {
                self.write("[");
                self.format_type_expr(element);
                self.write(&format!("; {}]", size));
            }
            TypeExprKind::Inferred => self.write("_"),
        }
    }

    fn format_shape_dim(&mut self, dim: &ShapeDim) {
        match dim {
            ShapeDim::Constant(n) => self.write(&n.to_string()),
            ShapeDim::Dynamic => self.write("?"),
            ShapeDim::Named(name) => self.write(name),
        }
    }

    fn format_pattern(&mut self, pattern: &Pattern) {
        match &pattern.kind {
            PatternKind::Identifier(name) => self.write(name),
            PatternKind::Literal(lit) => self.format_literal(lit),
            PatternKind::Wildcard => self.write("_"),
            PatternKind::Tuple(pats) => {
                self.write("(");
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_pattern(p);
                }
                self.write(")");
            }
            PatternKind::Struct { name, fields } => {
                self.write(&name.join("."));
                self.write(" { ");
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&f.name);
                    if let Some(p) = &f.pattern {
                        self.write(": ");
                        self.format_pattern(p);
                    }
                }
                self.write(" }");
            }
            PatternKind::EnumVariant { path, fields } => {
                self.write(&path.join("."));
                if !fields.is_empty() {
                    self.write("(");
                    for (i, f) in fields.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.format_pattern(f);
                    }
                    self.write(")");
                }
            }
        }
    }

    fn format_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(n) => self.write(&n.to_string()),
            Literal::Float(f) => self.write(&format!("{}", f)),
            Literal::String(s) => self.write(&format!("\"{}\"", s)),
            Literal::Bool(b) => self.write(if *b { "true" } else { "false" }),
            Literal::Char(c) => self.write(&format!("'{}'", c)),
        }
    }

    fn format_block_braces(&mut self, block: &Block) {
        self.write("{");
        if block.stmts.is_empty() && block.tail_expr.is_none() {
            self.write("}");
            return;
        }
        self.newline();
        self.indent();
        for stmt in &block.stmts {
            self.format_stmt(stmt);
        }
        if let Some(tail) = &block.tail_expr {
            self.write_indent();
            self.format_expr(tail);
            self.newline();
        }
        self.dedent();
        self.write_indent();
        self.write("}");
    }

    fn binop_str(op: &BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::MatMul => "@",
            BinOp::Eq => "==",
            BinOp::NotEq => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::LtEq => "<=",
            BinOp::GtEq => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }

    fn unaryop_str(op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::Ref => "&",
            UnaryOp::MutRef => "&mut ",
        }
    }

    fn assignop_str(op: &AssignOp) -> &'static str {
        match op {
            AssignOp::Assign => "=",
            AssignOp::AddAssign => "+=",
            AssignOp::SubAssign => "-=",
            AssignOp::MulAssign => "*=",
            AssignOp::DivAssign => "/=",
        }
    }

    // Indentation helpers
    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    #[allow(dead_code)]
    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn newline(&mut self) {
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_function() {
        let src = "fn  add( a :  Int32 , b : Int32 ): Int32 { return a + b ; }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("fn add(a: Int32, b: Int32): Int32 {"));
        assert!(result.contains("    return a + b;"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_struct() {
        let src = "model Point { x : Float64 , y : Float64 , }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("model Point {"));
        assert!(result.contains("    x: Float64,"));
        assert!(result.contains("    y: Float64,"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_if_else() {
        let src = "fn test() { if  x == 1 { return  1 ; } else { return 2 ; } }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("if x == 1 {"));
        assert!(result.contains("} else {"));
    }

    #[test]
    fn test_idempotent() {
        let src = "fn add(a: Int32, b: Int32): Int32 {\n    return a + b;\n}\n";
        let first = Formatter::format(src, "test.axon").unwrap();
        let second = Formatter::format(&first, "test.axon").unwrap();
        assert_eq!(first, second, "Formatting should be idempotent");
    }

    #[test]
    fn test_format_let_binding() {
        let src = "fn main() { val   x :  Int32   =  42; }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("val x: Int32 = 42;"));
    }

    #[test]
    fn test_format_enum() {
        let src = "enum  Color  { Red , Green , Blue , }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("enum Color {"));
        assert!(result.contains("    Red,"));
        assert!(result.contains("    Green,"));
        assert!(result.contains("    Blue,"));
    }

    #[test]
    fn test_format_while_loop() {
        let src = "fn test() { while  x  <  10  { x = x + 1 ; } }";
        let result = Formatter::format(src, "test.axon").unwrap();
        assert!(result.contains("while x < 10 {"));
    }
}
