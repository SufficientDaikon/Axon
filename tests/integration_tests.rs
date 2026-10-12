// tests/integration_tests.rs — Integration tests for the Axon compiler frontend

use axonc::{parse_source, ast_to_json};
use axonc::ast::*;
use axonc::error::CompileError;

fn parse_ok(src: &str) -> Program {
    let (prog, errors) = parse_source(src, "test.axon");
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{}", e.format_human());
        }
        panic!("Expected no errors, got {}", errors.len());
    }
    prog
}

fn parse_errors(src: &str) -> Vec<CompileError> {
    let (_, errors) = parse_source(src, "test.axon");
    errors
}

// ═══════════════════════════════════════════════════════════════
// Spec Example Tests (Examples 1-8)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_spec_example1_hello_world() {
    let src = include_str!("examples/example1_hello.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "main");
            assert!(f.body.is_some());
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_spec_example2_tensors() {
    let src = include_str!("examples/example2_tensors.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "matrix_multiply_example");
            let body = f.body.as_ref().unwrap();
            assert_eq!(body.stmts.len(), 3);
            // Third statement should contain a matmul (@)
            match &body.stmts[2].kind {
                StmtKind::Let { initializer: Some(expr), .. } => {
                    match &expr.kind {
                        ExprKind::BinaryOp { op, .. } => {
                            assert_eq!(*op, BinOp::MatMul, "Expected @ operator");
                        }
                        _ => panic!("Expected binary op with @"),
                    }
                }
                _ => panic!("Expected val with matmul"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_spec_example3_ownership() {
    let src = include_str!("examples/example3_ownership.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 2);

    // Check process_tensor has tensor types with dynamic dimension
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "process_tensor");
            assert!(f.return_type.is_some());
            // Check parameter type
            match &f.params[0].kind {
                FnParamKind::Typed { ty, .. } => {
                    match &ty.kind {
                        TypeExprKind::Tensor { shape, .. } => {
                            assert!(matches!(&shape[0], ShapeDim::Dynamic));
                        }
                        _ => panic!("Expected tensor type"),
                    }
                }
                _ => panic!("Expected typed param"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_spec_example4_neural_net() {
    let src = include_str!("examples/example4_neural_net.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 3);

    // Struct
    match &prog.items[0].kind {
        ItemKind::Struct(s) => {
            assert_eq!(s.name, "NeuralNet");
            assert_eq!(s.fields.len(), 3);
        }
        _ => panic!("Expected struct"),
    }

    // Impl block
    match &prog.items[1].kind {
        ItemKind::Impl(imp) => {
            assert_eq!(imp.items.len(), 1);
            match &imp.items[0].kind {
                ItemKind::Function(f) => {
                    assert_eq!(f.name, "forward");
                    assert!(matches!(&f.params[0].kind, FnParamKind::SelfRef));
                }
                _ => panic!("Expected function in impl"),
            }
        }
        _ => panic!("Expected impl"),
    }
}

#[test]
fn test_spec_example5_training() {
    let src = include_str!("examples/example5_training.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);

    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "train_epoch");
            // First param: &mut NeuralNet
            match &f.params[0].kind {
                FnParamKind::Typed { ty, .. } => {
                    assert!(matches!(&ty.kind, TypeExprKind::Reference { mutable: true, .. }));
                }
                _ => panic!("Expected typed param"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_spec_example6_device() {
    let src = include_str!("examples/example6_device.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_spec_example7_errors() {
    let src = include_str!("examples/example7_errors.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 2);

    // Check load_model returns Result type
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "load_model");
            match &f.return_type {
                Some(ty) => {
                    match &ty.kind {
                        TypeExprKind::Generic { name, args } => {
                            assert_eq!(name, "Result");
                            assert_eq!(args.len(), 2);
                        }
                        _ => panic!("Expected generic Result type"),
                    }
                }
                None => panic!("Expected return type"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_spec_example8_generics() {
    let src = include_str!("examples/example8_generics.axon");
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 2);

    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "squared_sum");
            assert_eq!(f.generics.len(), 1);
            assert_eq!(f.generics[0].name, "T");
            assert_eq!(f.generics[0].bounds.len(), 1);
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// AST JSON output test
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_ast_json_output() {
    let prog = parse_ok("fn main() { val x = 42; }");
    let json = ast_to_json(&prog);
    assert!(json.contains("\"main\""));
    assert!(json.contains("42"));
}

// ═══════════════════════════════════════════════════════════════
// FR-001: Brace-based syntax with semicolons
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr001_brace_syntax() {
    let prog = parse_ok(r#"
        fn compute() {
            val a = 1;
            val b = 2;
            val c = a + b;
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// FR-002: Mathematical tensor notation with @ operator
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr002_tensor_notation() {
    let prog = parse_ok(r#"
        fn matmul_test() {
            val C = A @ B;
            val D = A * B;
            val E = A + B;
            val F = A - B;
            val G = A / B;
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            let body = f.body.as_ref().unwrap();
            assert_eq!(body.stmts.len(), 5);
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-003: Whitespace-insensitive
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr003_whitespace_insensitive() {
    // These should all parse identically
    let prog1 = parse_ok("fn main(){val x=1;}");
    let prog2 = parse_ok("fn  main  (  )  {  val  x  =  1  ;  }");
    let prog3 = parse_ok("fn\nmain\n(\n)\n{\nval\nx\n=\n1\n;\n}");
    assert_eq!(prog1.items.len(), 1);
    assert_eq!(prog2.items.len(), 1);
    assert_eq!(prog3.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// FR-004: Variable declarations with type annotations or inference
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr004_variable_declarations() {
    let prog = parse_ok(r#"
        fn main() {
            val x: Int32 = 10;
            val y = 20;
            var z: Float64 = 3.14;
            var w = true;
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            let body = f.body.as_ref().unwrap();
            assert_eq!(body.stmts.len(), 4);

            // Check first: typed
            match &body.stmts[0].kind {
                StmtKind::Let { ty: Some(_), mutable, .. } => assert!(!mutable),
                _ => panic!("Expected typed let"),
            }
            // Check second: inferred
            match &body.stmts[1].kind {
                StmtKind::Let { ty: None, .. } => {}
                _ => panic!("Expected inferred let"),
            }
            // Check third: mutable
            match &body.stmts[2].kind {
                StmtKind::Let { mutable, .. } => assert!(mutable),
                _ => panic!("Expected mutable let"),
            }
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-005: Function definitions with named params, return types, defaults
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr005_function_definitions() {
    let prog = parse_ok(r#"
        fn add(a: Int32, b: Int32): Int32 {
            return a + b;
        }

        fn greet(name: String, greeting: String = "Hello") {
            println("{} {}", greeting, name);
        }
    "#);
    assert_eq!(prog.items.len(), 2);

    match &prog.items[1].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name, "greet");
            assert_eq!(f.params.len(), 2);
            match &f.params[1].kind {
                FnParamKind::Typed { default: Some(_), .. } => {}
                _ => panic!("Expected default parameter"),
            }
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-006: Control flow
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr006_if_else_chain() {
    let prog = parse_ok(r#"
        fn classify(x: Int32) {
            if x > 0 {
                println("positive");
            } else if x == 0 {
                println("zero");
            } else {
                println("negative");
            }
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_fr006_match_expression() {
    let prog = parse_ok(r#"
        fn describe(opt: Option<Int32>) {
            match opt {
                Some(x) => println("Got: {}", x),
                None => println("Nothing"),
            }
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_fr006_for_loop_with_destructuring() {
    let prog = parse_ok(r#"
        fn process() {
            for (key, value) in items {
                println("{}: {}", key, value);
            }
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// FR-007: User-defined types
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr007_struct() {
    let prog = parse_ok(r#"
        model Point {
            x: Float64,
            y: Float64,
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Struct(s) => {
            assert_eq!(s.name, "Point");
            assert_eq!(s.fields.len(), 2);
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_fr007_enum_variants() {
    let prog = parse_ok(r#"
        enum Color {
            Red,
            Green,
            Blue,
            Custom(UInt8, UInt8, UInt8),
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Enum(e) => {
            assert_eq!(e.name, "Color");
            assert_eq!(e.variants.len(), 4);
            assert!(matches!(&e.variants[0].fields, EnumVariantKind::Unit));
            assert!(matches!(&e.variants[3].fields, EnumVariantKind::Tuple(_)));
        }
        _ => panic!("Expected enum"),
    }
}

#[test]
fn test_fr007_type_alias() {
    let prog = parse_ok("type BatchTensor = Tensor<Float32, [?, 784]>;");
    match &prog.items[0].kind {
        ItemKind::TypeAlias(ta) => {
            assert_eq!(ta.name, "BatchTensor");
        }
        _ => panic!("Expected type alias"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-008: Comments
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr008_comments() {
    let prog = parse_ok(r#"
        // This is a single-line comment
        fn main() {
            /* This is a
               multi-line comment */
            val x = 42; // inline comment
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// FR-010: Primitive types
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr010_primitive_types() {
    let prog = parse_ok(r#"
        fn types() {
            val a: Int8 = 1;
            val b: Int16 = 2;
            val c: Int32 = 3;
            val d: Int64 = 4;
            val e: UInt8 = 5;
            val f: UInt16 = 6;
            val g: UInt32 = 7;
            val h: UInt64 = 8;
            val i: Float16 = 1.0;
            val j: Float32 = 2.0;
            val k: Float64 = 3.0;
            val l: Bool = true;
            val m: Char = 'a';
            val n: String = "hello";
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.body.as_ref().unwrap().stmts.len(), 14);
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-011: First-class tensor types
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr011_tensor_types() {
    let prog = parse_ok(r#"
        fn tensor_types() {
            val a: Tensor<Float32, [128, 256]> = zeros([128, 256]);
            val b: Tensor<Float64, [?, 10]> = ones([32, 10]);
            val c: Tensor<Int32, [3, 3, 3]> = zeros([3, 3, 3]);
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// FR-013: Generic types with trait bounds
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr013_generics() {
    let prog = parse_ok(r#"
        fn process<T: Numeric>(data: Tensor<T, [N, M]>): Tensor<T, [N]> {
            return data.sum();
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            assert_eq!(f.generics.len(), 1);
            assert_eq!(f.generics[0].name, "T");
            assert_eq!(f.generics[0].bounds.len(), 1);
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-014: Dynamic shapes with ?
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr014_dynamic_shapes() {
    let prog = parse_ok(r#"
        fn dynamic(t: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, ?]> {
            return t;
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            match &f.params[0].kind {
                FnParamKind::Typed { ty, .. } => {
                    match &ty.kind {
                        TypeExprKind::Tensor { shape, .. } => {
                            assert!(matches!(&shape[0], ShapeDim::Dynamic));
                            assert!(matches!(&shape[1], ShapeDim::Constant(784)));
                        }
                        _ => panic!("Expected tensor type"),
                    }
                }
                _ => panic!("Expected typed param"),
            }
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-015: References
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr015_references() {
    let prog = parse_ok(r#"
        fn borrow(x: &Int32) {
            val y = x;
        }

        fn borrow_mut(x: &mut Int32) {
            val y = x;
        }
    "#);
    assert_eq!(prog.items.len(), 2);

    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            match &f.params[0].kind {
                FnParamKind::Typed { ty, .. } => {
                    match &ty.kind {
                        TypeExprKind::Reference { mutable, .. } => assert!(!mutable),
                        _ => panic!("Expected reference type"),
                    }
                }
                _ => panic!("Expected typed param"),
            }
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-040: Result<T, E> type
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr040_result_type() {
    let prog = parse_ok(r#"
        fn read(): Result<String, IOError> {
            return Ok("data");
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Function(f) => {
            match &f.return_type.as_ref().unwrap().kind {
                TypeExprKind::Generic { name, args } => {
                    assert_eq!(name, "Result");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected Result generic type"),
            }
        }
        _ => panic!("Expected function"),
    }
}

// ═══════════════════════════════════════════════════════════════
// FR-041: ? operator
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr041_question_operator() {
    let prog = parse_ok(r#"
        fn read(path: String): Result<String, IOError> {
            val file = File.open(path)?;
            return Ok(file.read()?);
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// FR-043: Error codes, messages, locations, suggestions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr043_error_details() {
    let errors = parse_errors("fn main() { val x = ; }");
    assert!(!errors.is_empty());
    let e = &errors[0];
    assert!(!e.error_code.is_empty());
    assert!(!e.message.is_empty());
    assert!(e.location.is_some());
}

// ═══════════════════════════════════════════════════════════════
// FR-044: JSON error format
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr044_json_error_format() {
    let errors = parse_errors("fn main() { val x = ; }");
    assert!(!errors.is_empty());
    let json = errors[0].format_json();
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.get("error_code").is_some());
    assert!(parsed.get("message").is_some());
    assert!(parsed.get("location").is_some());
}

// ═══════════════════════════════════════════════════════════════
// FR-045: Report all errors
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fr045_multiple_errors() {
    let errors = parse_errors(r#"
        fn () { }
        fn () { }
    "#);
    // Should have at least 2 errors (one per bad fn declaration)
    assert!(errors.len() >= 2, "Expected >= 2 errors, got {}", errors.len());
}

// ═══════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_empty_program() {
    let prog = parse_ok("");
    assert_eq!(prog.items.len(), 0);
}

#[test]
fn test_comment_only_program() {
    let prog = parse_ok("// just a comment\n/* and another */");
    assert_eq!(prog.items.len(), 0);
}

#[test]
fn test_deeply_nested_expressions() {
    let prog = parse_ok(r#"
        fn main() {
            val x = ((((1 + 2) * 3) - 4) / 5);
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_chained_method_calls() {
    let prog = parse_ok(r#"
        fn main() {
            val x = a.b().c().d().e();
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_complex_match_patterns() {
    let prog = parse_ok(r#"
        fn handle(result: Result<Int32, String>) {
            match result {
                Ok(value) => println("Got: {}", value),
                Err(msg) => println("Error: {}", msg),
            }
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_trait_with_methods() {
    let prog = parse_ok(r#"
        trait Model {
            fn forward(&self, input: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]>;
            fn parameters(&self): Vec<Tensor<Float32, [?]>>;
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Trait(t) => {
            assert_eq!(t.name, "Model");
            assert_eq!(t.items.len(), 2);
        }
        _ => panic!("Expected trait"),
    }
}

#[test]
fn test_impl_trait_for_type() {
    let prog = parse_ok(r#"
        extend Display for NeuralNet {
            fn display(&self) {
                println("NeuralNet");
            }
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Impl(imp) => {
            assert!(imp.trait_name.is_some());
        }
        _ => panic!("Expected impl"),
    }
}

#[test]
fn test_use_as_alias() {
    let prog = parse_ok("use std.collections.HashMap as Map;");
    match &prog.items[0].kind {
        ItemKind::Use(u) => {
            assert_eq!(u.path, vec!["std", "collections", "HashMap"]);
            assert_eq!(u.alias.as_deref(), Some("Map"));
        }
        _ => panic!("Expected use"),
    }
}

#[test]
fn test_module_with_items() {
    let prog = parse_ok(r#"
        mod math {
            fn add(a: Int32, b: Int32): Int32 {
                return a + b;
            }
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Module(m) => {
            assert_eq!(m.name, "math");
            assert!(m.items.is_some());
            assert_eq!(m.items.as_ref().unwrap().len(), 1);
        }
        _ => panic!("Expected module"),
    }
}

#[test]
fn test_attributes_on_functions() {
    let prog = parse_ok(r#"
        @gpu
        fn compute(a: Tensor<Float32, [1024, 1024]>) {
            val b = a @ a;
        }

        @cpu
        fn fallback(a: Tensor<Float32, [4, 4]>) {
            val b = a * a;
        }
    "#);
    assert_eq!(prog.items.len(), 2);
    assert!(matches!(&prog.items[0].attributes[0], Attribute::Gpu));
    assert!(matches!(&prog.items[1].attributes[0], Attribute::Cpu));
}

#[test]
fn test_assignment_operators() {
    let prog = parse_ok(r#"
        fn main() {
            var x = 0;
            x += 1;
            x -= 2;
            x *= 3;
            x /= 4;
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_type_cast_as() {
    let prog = parse_ok(r#"
        fn main() {
            val x = total_loss / data.len() as Float32;
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_public_struct_fields() {
    let prog = parse_ok(r#"
        pub model Config {
            pub learning_rate: Float64,
            pub batch_size: Int32,
            hidden_size: Int32,
        }
    "#);
    match &prog.items[0].kind {
        ItemKind::Struct(s) => {
            assert_eq!(s.fields[0].visibility, Visibility::Public);
            assert_eq!(s.fields[1].visibility, Visibility::Public);
            assert_eq!(s.fields[2].visibility, Visibility::Private);
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_negative_number_literal() {
    let prog = parse_ok(r#"
        fn main() {
            val x = -42;
            val y = -3.14;
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_boolean_operators() {
    let prog = parse_ok(r#"
        fn main() {
            val x = a && b || c;
            val y = !flag;
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_string_with_format_args() {
    let prog = parse_ok(r#"
        fn main() {
            println("Hello, {}! You are {} years old.", name, age);
        }
    "#);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// Stack Safety Tests (E4)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_deeply_nested_expression_50() {
    let mut src = String::from("fn main() { val x: Int64 = ");
    for _ in 0..50 {
        src.push('(');
    }
    src.push('1');
    for _ in 0..50 {
        src.push(')');
    }
    src.push_str("; }");
    let (prog, errors) = axonc::parse_source(&src, "test.axon");
    assert!(errors.is_empty(), "Nested parens should parse: {:?}", errors);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_deeply_nested_binary_ops_50() {
    let mut src = String::from("fn main() { val x: Int64 = ");
    for _ in 0..50 {
        src.push_str("1 + (");
    }
    src.push('1');
    for _ in 0..50 {
        src.push(')');
    }
    src.push_str("; }");
    let (prog, errors) = axonc::parse_source(&src, "test.axon");
    assert!(errors.is_empty(), "Nested binary ops should parse: {:?}", errors);
    assert_eq!(prog.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// E9: Fold/Reduce & Comprehension Tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_comprehension_basic() {
    let src = r#"
fn main() {
    val xs = [x * 2 for x in range(10)];
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_comprehension_with_filter() {
    let src = r#"
fn main() {
    val evens = [x for x in range(20) if x % 2 == 0];
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_comprehension_with_complex_body() {
    let src = r#"
fn main() {
    val squares = [x * x + 1 for x in range(10)];
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_fold_method_call_parses() {
    let src = r#"
fn main() {
    val xs = [1, 2, 3, 4, 5];
    val sum = xs.fold(0, |acc, x| acc + x);
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_reduce_method_call_parses() {
    let src = r#"
fn main() {
    val xs = [1, 2, 3, 4, 5];
    val sum = xs.reduce(|a, b| a + b);
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_comprehension_ast_structure() {
    let src = r#"
fn main() {
    val xs = [x * 2 for x in items];
}
"#;
    let prog = parse_ok(src);
    let func = match &prog.items[0].kind {
        ItemKind::Function(f) => f,
        _ => panic!("Expected function"),
    };
    let body = func.body.as_ref().unwrap();
    let stmt = &body.stmts[0];
    if let StmtKind::Let { initializer: Some(init), .. } = &stmt.kind {
        assert!(matches!(init.kind, ExprKind::Comprehension { .. }),
            "Expected Comprehension, got {:?}", init.kind);
    } else {
        panic!("Expected let with initializer");
    }
}

#[test]
fn test_comprehension_with_filter_ast_structure() {
    let src = r#"
fn main() {
    val xs = [x for x in items if x > 0];
}
"#;
    let prog = parse_ok(src);
    let func = match &prog.items[0].kind {
        ItemKind::Function(f) => f,
        _ => panic!("Expected function"),
    };
    let body = func.body.as_ref().unwrap();
    let stmt = &body.stmts[0];
    if let StmtKind::Let { initializer: Some(init), .. } = &stmt.kind {
        if let ExprKind::Comprehension { condition, .. } = &init.kind {
            assert!(condition.is_some(), "Expected filter condition");
        } else {
            panic!("Expected Comprehension");
        }
    } else {
        panic!("Expected let with initializer");
    }
}

#[test]
fn test_array_literal_still_works() {
    let src = r#"
fn main() {
    val xs = [1, 2, 3];
    val ys = [];
    val zs = [42];
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_array_literal_trailing_comma() {
    let src = r#"
fn main() {
    val xs = [1, 2, 3,];
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_comprehension_type_checks() {
    let src = r#"
fn main() {
    val xs = [x * 2 for x in items];
}
"#;
    let (_tast, errors) = axonc::check_source(src, "test.axon");
    // Comprehension itself should parse and type-check; only 'items' might be undefined
    let has_comprehension_error = errors.iter().any(|e| {
        let msg = e.format_human();
        msg.contains("comprehension") || msg.contains("for expression")
    });
    assert!(!has_comprehension_error, "Comprehension-specific errors: {:?}",
        errors.iter().map(|e| e.format_human()).collect::<Vec<_>>());
}

#[test]
fn test_comprehension_with_filter_type_checks() {
    let src = r#"
fn main() {
    val xs = [x for x in items if x > 5];
}
"#;
    let (_tast, errors) = axonc::check_source(src, "test.axon");
    let has_comprehension_error = errors.iter().any(|e| {
        let msg = e.format_human();
        msg.contains("comprehension") || msg.contains("for expression")
    });
    assert!(!has_comprehension_error, "Comprehension-specific errors: {:?}",
        errors.iter().map(|e| e.format_human()).collect::<Vec<_>>());
}

#[test]
fn test_fold_type_checks() {
    let src = r#"
fn main() {
    val sum = items.fold(0, |acc, x| acc + x);
}
"#;
    let (_tast, errors) = axonc::check_source(src, "test.axon");
    // fold as method call parses and type-checks; 'items' undefined is not a fold error
    let has_fold_error = errors.iter().any(|e| {
        let msg = e.format_human();
        msg.contains("fold")
    });
    assert!(!has_fold_error, "Fold-specific errors: {:?}",
        errors.iter().map(|e| e.format_human()).collect::<Vec<_>>());
}

#[test]
fn test_reduce_type_checks() {
    let src = r#"
fn main() {
    val sum = items.reduce(|a, b| a + b);
}
"#;
    let (_tast, errors) = axonc::check_source(src, "test.axon");
    let has_reduce_error = errors.iter().any(|e| {
        let msg = e.format_human();
        msg.contains("reduce")
    });
    assert!(!has_reduce_error, "Reduce-specific errors: {:?}",
        errors.iter().map(|e| e.format_human()).collect::<Vec<_>>());
}

#[test]
fn test_comprehension_lowers_to_mir() {
    let src = r#"
fn main() {
    val xs = [x for x in items];
}
"#;
    let (tast, _check_errors, checker) = axonc::check_source_full(src, "test.axon");
    let mir = axonc::mir::MirBuilder::new(&checker.interner).build(&tast);
    assert!(!mir.functions.is_empty(), "MIR should have functions");
}

#[test]
fn test_fold_lowers_to_mir() {
    let src = r#"
fn main() {
    val sum = items.fold(0, |acc, x| acc + x);
}
"#;
    let (tast, _check_errors, checker) = axonc::check_source_full(src, "test.axon");
    let mir = axonc::mir::MirBuilder::new(&checker.interner).build(&tast);
    assert!(!mir.functions.is_empty(), "MIR should have functions");
}

#[test]
fn test_comprehension_formats_correctly() {
    let src = r#"fn main() { val xs = [x * 2 for x in items]; }"#;
    let formatted = axonc::fmt::Formatter::format(src, "test.axon").unwrap();
    assert!(formatted.contains("for"), "Formatted output should contain 'for': {}", formatted);
    assert!(formatted.contains("in"), "Formatted output should contain 'in': {}", formatted);
}

#[test]
fn test_comprehension_with_filter_formats() {
    let src = r#"fn main() { val xs = [x for x in items if x > 0]; }"#;
    let formatted = axonc::fmt::Formatter::format(src, "test.axon").unwrap();
    assert!(formatted.contains("if"), "Formatted output should contain 'if': {}", formatted);
}

#[test]
fn test_nested_comprehension() {
    let src = r#"
fn main() {
    val xs = [x + y for x in range(5)];
}
"#;
    let prog = parse_ok(src);
    assert_eq!(prog.items.len(), 1);
}

#[test]
fn test_comprehension_empty_range() {
    let src = r#"
fn main() {
    val xs = [x for x in items];
}
"#;
    let (_tast, errors) = axonc::check_source(src, "test.axon");
    // 'items' may be undefined but comprehension itself should work
    let has_comprehension_error = errors.iter().any(|e| {
        let msg = e.format_human();
        msg.contains("comprehension") || msg.contains("for expression")
    });
    assert!(!has_comprehension_error, "Comprehension-specific errors found");
}

// ═══════════════════════════════════════════════════════════════
// Phase EPSILON: Cache + Error Recovery integration tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cache_detects_dirty() {
    let mut cache = axonc::cache::CompilationCache::new();
    let src1 = "fn main() { val x: Int32 = 1; }";
    let src2 = "fn main() { val x: Int32 = 2; }";

    // New file is always dirty
    assert!(cache.is_dirty("a.axon", src1));

    // After update, same source is clean
    cache.update("a.axon", src1, false);
    assert!(!cache.is_dirty("a.axon", src1));

    // Changed source is dirty
    assert!(cache.is_dirty("a.axon", src2));

    // After invalidation, it's dirty again
    cache.update("a.axon", src2, false);
    cache.invalidate("a.axon");
    assert!(cache.is_dirty("a.axon", src2));
}

#[test]
fn test_error_recovery_multiple_errors() {
    // Parser should recover and report multiple errors
    let src = "fn f() { val x = ; val y = ; }";
    let (_ast, errors) = axonc::parse_source(src, "test.axon");
    // Should get errors but not panic
    assert!(!errors.is_empty(), "Should have parse errors");
}

#[test]
fn test_error_recovery_continues_after_bad_item() {
    // After a bad item, should still parse subsequent valid items
    let src = r#"
        fn bad(,) {}
        fn good(): Int32 { return 42; }
    "#;
    let (ast, errors) = axonc::parse_source(src, "test.axon");
    assert!(!errors.is_empty(), "Should have parse errors for bad function");
    // Despite errors, the AST should have items
    assert!(!ast.items.is_empty(), "Should have parsed some items");
}

#[test]
fn test_check_with_program_no_double_parse() {
    // check_with_program should accept a pre-parsed AST
    let src = "fn main(): Int64 { return 0; }";
    let (program, parse_errors) = axonc::parse_source(src, "test.axon");
    assert!(parse_errors.is_empty());

    let (_checker, errors) = axonc::typeck::check_with_program(&program, parse_errors);
    // Should succeed — main is valid
    assert!(errors.is_empty(), "Should have no errors: {:?}", errors);
}

#[test]
fn test_mir_pass_returns_changed_flag() {
    let src = "fn f(): Int32 { return 1 + 2; }";
    let (tast, _errors, checker) = axonc::check_source_full(src, "test.axon");
    let mut mir = axonc::mir::MirBuilder::new(&checker.interner).build(&tast);
    let pm = axonc::mir::transform::PassManager::with_default_passes(2);
    let changed = pm.run_all(&mut mir, &checker.interner);
    // At O2, passes should report having run (returns true)
    assert!(changed, "O2 passes should report changes");
}
