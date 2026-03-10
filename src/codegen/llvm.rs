// codegen/llvm.rs — LLVM IR Code Generator (Phase 4b)
//
// Generates textual LLVM IR (.ll files) from MIR. The generated IR
// is then compiled to native code using `clang` as an external tool.

use crate::codegen::runtime::emit_runtime_declarations;
use crate::mir::*;
use crate::types::{PrimKind, Type, TypeId, TypeInterner};

// ═══════════════════════════════════════════════════════════════
// Optimization Level
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

// ═══════════════════════════════════════════════════════════════
// LLVM IR Code Generator
// ═══════════════════════════════════════════════════════════════

pub struct LlvmCodegen<'a> {
    interner: &'a TypeInterner,
    output: String,
    next_unnamed: u32,
    string_literals: Vec<(String, String)>,
    #[allow(dead_code)]
    opt_level: OptLevel,
    /// Cache of the current function being emitted (for local_type lookups).
    current_func_locals: Vec<MirLocal>,
}

impl<'a> LlvmCodegen<'a> {
    pub fn new(interner: &'a TypeInterner, opt_level: OptLevel) -> Self {
        LlvmCodegen {
            interner,
            output: String::new(),
            next_unnamed: 0,
            string_literals: Vec::new(),
            opt_level,
            current_func_locals: Vec::new(),
        }
    }

    // ── Public entry point ─────────────────────────────────────

    /// Generate complete LLVM IR module from MIR program.
    /// Returns an error if no `main` function is found.
    pub fn generate(&mut self, program: &MirProgram) -> Result<String, String> {
        self.output.clear();
        self.string_literals.clear();

        self.emit_module_header();
        self.output.push('\n');
        self.emit_runtime_decls();
        self.output.push('\n');

        for func in &program.functions {
            self.emit_function(func);
            self.output.push('\n');
        }

        // Emit main wrapper if user defined a main function
        let has_main = program.functions.iter().any(|f| f.name == "main");
        if has_main {
            self.output.push_str("\n; C ABI main wrapper\n");
            self.output.push_str("define i32 @main() {\n");
            self.output.push_str("entry:\n");
            self.output.push_str("  call void @_axon_main()\n");
            self.output.push_str("  ret i32 0\n");
            self.output.push_str("}\n");
        } else {
            return Err("E5009: no `main` function found. Every Axon executable must have a `fn main() { ... }`".to_string());
        }

        // Emit accumulated string literals at the end of the module.
        self.emit_string_literals();

        Ok(self.output.clone())
    }

    // ── Module-level ───────────────────────────────────────────

    fn emit_module_header(&mut self) {
        self.emit_line("; ModuleID = 'axon_module'");
        self.emit_line("source_filename = \"axon_module.axon\"");
        let triple = Self::host_target_triple();
        self.output
            .push_str(&format!("target triple = \"{}\"\n", triple));
    }

    fn emit_runtime_decls(&mut self) {
        let decls = emit_runtime_declarations();
        self.output.push_str(&decls);
    }

    fn emit_string_literals(&mut self) {
        if self.string_literals.is_empty() {
            return;
        }
        self.output.push('\n');
        self.emit_line("; String literals");
        let lits = self.string_literals.clone();
        for (name, value) in &lits {
            let escaped = Self::escape_llvm_string(value);
            let len = value.len() + 1; // +1 for null terminator
            self.output.push_str(&format!(
                "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
                name, len, escaped
            ));
        }
    }

    // ── Type mapping ───────────────────────────────────────────

    fn llvm_type(&self, ty: TypeId) -> String {
        let resolved = self.interner.resolve(ty);
        match resolved {
            Type::Primitive(prim) => self.llvm_prim_type(prim).to_string(),
            Type::Unit => "void".to_string(),
            Type::Never => "void".to_string(),
            Type::Error => "i8*".to_string(),
            Type::Tensor(_) => "{ i8*, i64*, i64, i8 }".to_string(),
            Type::Tuple(fields) => {
                let field_types: Vec<String> =
                    fields.iter().map(|f| self.llvm_type(*f)).collect();
                format!("{{ {} }}", field_types.join(", "))
            }
            Type::Array { elem, size } => {
                let elem_ty = self.llvm_type(*elem);
                format!("[{} x {}]", size, elem_ty)
            }
            Type::Reference { inner, .. } => {
                let inner_ty = self.llvm_type(*inner);
                format!("{}*", inner_ty)
            }
            Type::Function { params, ret } => {
                let ret_ty = self.llvm_type(*ret);
                let param_types: Vec<String> =
                    params.iter().map(|p| self.llvm_type(*p)).collect();
                format!("{} ({})*", ret_ty, param_types.join(", "))
            }
            Type::Struct { fields, .. } => {
                let field_types: Vec<String> =
                    fields.iter().map(|(_, ty)| self.llvm_type(*ty)).collect();
                format!("{{ {} }}", field_types.join(", "))
            }
            Type::Enum { variants, .. } => {
                // Tag (i8) + largest variant payload
                let max_payload = self.largest_enum_variant_size(variants);
                if max_payload > 0 {
                    format!("{{ i8, [{} x i8] }}", max_payload)
                } else {
                    "{ i8 }".to_string()
                }
            }
            Type::Option(inner) => {
                // Represented like an enum: { i1, inner }
                let inner_ty = self.llvm_type(*inner);
                format!("{{ i1, {} }}", inner_ty)
            }
            Type::Result(ok, err) => {
                // Represented like an enum: { i8, max(ok, err) }
                let ok_ty = self.llvm_type(*ok);
                let err_ty = self.llvm_type(*err);
                format!("{{ i8, {{ {}, {} }} }}", ok_ty, err_ty)
            }
            Type::Generic(_) | Type::TypeVar(_) | Type::Named { .. } => {
                "i8*".to_string() // opaque pointer fallback
            }
            Type::Trait { .. } => "i8*".to_string(),
        }
    }

    fn llvm_prim_type(&self, prim: &PrimKind) -> &'static str {
        match prim {
            PrimKind::Int8 | PrimKind::UInt8 => "i8",
            PrimKind::Int16 | PrimKind::UInt16 => "i16",
            PrimKind::Int32 | PrimKind::UInt32 => "i32",
            PrimKind::Int64 | PrimKind::UInt64 => "i64",
            PrimKind::Float16 => "half",
            PrimKind::Float32 => "float",
            PrimKind::Float64 => "double",
            PrimKind::Bool => "i1",
            PrimKind::Char => "i32",
            PrimKind::String => "{ i8*, i64, i64 }",
        }
    }

    /// Return the LLVM type for a local that is always used via `alloca` (pointer form).
    /// For `void` types we use `{}` instead (can't alloca void).
    fn llvm_alloca_type(&self, ty: TypeId) -> String {
        let t = self.llvm_type(ty);
        if t == "void" {
            "{}".to_string()
        } else {
            t
        }
    }

    // ── Function generation ────────────────────────────────────

    fn emit_function(&mut self, func: &MirFunction) {
        self.next_unnamed = 0;
        self.current_func_locals = func.locals.clone();

        let sig = self.emit_function_signature(func);
        self.output.push_str(&format!("define {} {{\n", sig));

        // Emit entry block with alloca for all locals
        if let Some(first_block) = func.basic_blocks.first() {
            let label = if first_block.id == BlockId(0) {
                "entry".to_string()
            } else {
                self.block_label(first_block.id)
            };
            self.output.push_str(&format!("{}:\n", label));
            self.emit_locals(&func.locals);

            // Store function parameters into their corresponding locals.
            // Params are locals[1..1+params.len()] (locals[0] is the return place).
            for (i, param) in func.params.iter().enumerate() {
                let param_local = LocalId((i + 1) as u32);
                let ty = self.llvm_alloca_type(param.ty);
                self.output.push_str(&format!(
                    "  store {} %arg{}, {}* {}\n",
                    ty,
                    i,
                    ty,
                    self.local_name(param_local)
                ));
            }

            // Emit statements and terminator for entry block
            for stmt in &first_block.stmts {
                self.emit_stmt(stmt, func);
            }
            self.emit_terminator(&first_block.terminator, func);
        }

        // Emit remaining blocks
        for block in func.basic_blocks.iter().skip(1) {
            self.emit_basic_block(block, func);
        }

        self.emit_line("}");
        self.current_func_locals.clear();
    }

    fn emit_function_signature(&self, func: &MirFunction) -> String {
        let ret_ty = self.llvm_type(func.return_ty);
        let ret_str = if ret_ty == "void" {
            "void".to_string()
        } else {
            ret_ty
        };

        let params: Vec<String> = func
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let ty = self.llvm_type(p.ty);
                let ty = if ty == "void" { "{}".to_string() } else { ty };
                format!("{} %arg{}", ty, i)
            })
            .collect();

        let fn_name = if func.name == "main" { "_axon_main".to_string() } else { func.mangled_name.clone() };
        format!("{} @{}({})", ret_str, fn_name, params.join(", "))
    }

    fn emit_locals(&mut self, locals: &[MirLocal]) {
        for local in locals {
            let ty = self.llvm_alloca_type(local.ty);
            self.output.push_str(&format!(
                "  {} = alloca {}\n",
                self.local_name(local.id),
                ty
            ));
        }
    }

    fn emit_basic_block(&mut self, block: &MirBasicBlock, func: &MirFunction) {
        self.output
            .push_str(&format!("{}:\n", self.block_label(block.id)));
        for stmt in &block.stmts {
            self.emit_stmt(stmt, func);
        }
        self.emit_terminator(&block.terminator, func);
    }

    // ── Statement emission ─────────────────────────────────────

    fn emit_stmt(&mut self, stmt: &MirStmt, func: &MirFunction) {
        match stmt {
            MirStmt::Assign { place, rvalue, .. } => {
                self.emit_assign(place, rvalue, func);
            }
            MirStmt::Drop { place, .. } => {
                self.emit_drop(place, func);
            }
            MirStmt::StorageLive { .. } | MirStmt::StorageDead { .. } | MirStmt::Nop => {
                // No-op in LLVM IR; storage annotations are informational.
            }
        }
    }

    fn emit_assign(&mut self, place: &Place, rvalue: &Rvalue, func: &MirFunction) {
        let value = self.emit_rvalue(rvalue, func);
        let ty = self.type_of_rvalue(rvalue, func);
        let ty_str = self.llvm_alloca_type(ty);
        if ty_str == "{}" || ty_str == "void" {
            // Unit-typed assignments are no-ops.
            return;
        }
        self.emit_place_store(place, &value, &ty_str);
    }

    fn emit_drop(&mut self, place: &Place, func: &MirFunction) {
        let ty = self.local_type(func, place.local);
        let resolved = self.interner.resolve(ty);
        match resolved {
            Type::Primitive(PrimKind::String) => {
                // Call runtime dealloc for strings (simplified).
                let _val = self.emit_place_load(place, func);
                // In a full implementation, extract ptr/len and call axon_dealloc.
            }
            Type::Tensor(_) => {
                let val = self.emit_place_load(place, func);
                let name = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = bitcast {{ i8*, i64*, i64, i8 }} {} to i8*\n",
                    name, val
                ));
                self.output
                    .push_str(&format!("  call void @axon_tensor_free(i8* {})\n", name));
            }
            _ => {
                // Scalar / trivially-droppable types: no-op.
            }
        }
    }

    // ── Terminator emission ────────────────────────────────────

    fn emit_terminator(&mut self, term: &Terminator, func: &MirFunction) {
        match term {
            Terminator::Goto { target } => {
                self.output.push_str(&format!(
                    "  br label %{}\n",
                    self.block_label(*target)
                ));
            }
            Terminator::SwitchInt {
                value,
                targets,
                otherwise,
            } => {
                let cond = self.emit_operand(value, func);
                if targets.len() == 1 {
                    // Common case: boolean branch
                    let (val, target) = &targets[0];
                    let cond_val = if *val != 0 {
                        // Branch if true
                        cond.clone()
                    } else {
                        // Branch if false — negate
                        let negated = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = icmp eq i1 {}, 0\n",
                            negated, cond
                        ));
                        negated
                    };
                    self.output.push_str(&format!(
                        "  br i1 {}, label %{}, label %{}\n",
                        cond_val,
                        self.block_label(*target),
                        self.block_label(*otherwise)
                    ));
                } else if targets.is_empty() {
                    self.output.push_str(&format!(
                        "  br label %{}\n",
                        self.block_label(*otherwise)
                    ));
                } else {
                    // General switch
                    self.output.push_str(&format!(
                        "  switch i64 {}, label %{} [\n",
                        cond,
                        self.block_label(*otherwise)
                    ));
                    for (val, target) in targets {
                        self.output.push_str(&format!(
                            "    i64 {}, label %{}\n",
                            val,
                            self.block_label(*target)
                        ));
                    }
                    self.output.push_str("  ]\n");
                }
            }
            Terminator::Return => {
                let ret_ty = self.llvm_type(func.return_ty);
                if ret_ty == "void" {
                    self.output.push_str("  ret void\n");
                } else {
                    let ret_val = self.emit_place_load(
                        &Place {
                            local: LocalId(0),
                            projections: Vec::new(),
                        },
                        func,
                    );
                    self.output.push_str(&format!(
                        "  ret {} {}\n",
                        ret_ty, ret_val
                    ));
                }
            }
            Terminator::Call {
                func: callee,
                args,
                destination,
                target,
            } => {
                // Resolve callee name
                let callee_name = match callee {
                    Operand::Constant(MirConstant::String(name)) => format!("@{}", name),
                    _ => self.emit_operand(callee, func),
                };
                let is_print_bool = callee_name == "@axon_print_bool";
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| {
                        let val = self.emit_operand(a, func);
                        // String constants now emit as { i8*, i64, i64 } structs.
                        // For runtime print functions that expect raw (i8*, i64),
                        // extract the pointer from the struct.
                        let ty_str = if matches!(a, Operand::Constant(MirConstant::String(_))) {
                            // Extract the i8* pointer from the string struct for call args
                            let ptr = self.fresh_name();
                            self.output.push_str(&format!(
                                "  {} = extractvalue {{ i8*, i64, i64 }} {}, 0\n",
                                ptr, val
                            ));
                            return format!("i8* {}", ptr);
                        } else {
                            let ty = self.type_of_operand(a, func);
                            self.llvm_alloca_type(ty)
                        };
                        // ABI fix: axon_print_bool expects i8, but bools are i1 in IR.
                        // Emit zext i1 → i8 to match the C runtime signature.
                        if is_print_bool && ty_str == "i1" {
                            let widened = self.fresh_name();
                            self.output.push_str(&format!(
                                "  {} = zext i1 {} to i8\n",
                                widened, val
                            ));
                            return format!("i8 {}", widened);
                        }
                        format!("{} {}", ty_str, val)
                    })
                    .collect();

                let dest_ty = self.local_type(func, destination.local);
                let dest_ty_str = self.llvm_type(dest_ty);

                if dest_ty_str == "void" {
                    self.output.push_str(&format!(
                        "  call void {}({})\n",
                        callee_name,
                        arg_strs.join(", ")
                    ));
                } else {
                    let result = self.fresh_name();
                    self.output.push_str(&format!(
                        "  {} = call {} {}({})\n",
                        result,
                        dest_ty_str,
                        callee_name,
                        arg_strs.join(", ")
                    ));
                    self.emit_place_store(destination, &result, &dest_ty_str);
                }
                self.output.push_str(&format!(
                    "  br label %{}\n",
                    self.block_label(*target)
                ));
            }
            Terminator::Assert {
                cond,
                msg,
                target,
            } => {
                let cond_val = self.emit_operand(cond, func);
                let ok_label = self.block_label(*target);
                let panic_label = format!("assert_fail_{}", self.next_unnamed);
                self.next_unnamed += 1;

                self.output.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_val, ok_label, panic_label
                ));
                self.output.push_str(&format!("{}:\n", panic_label));

                // Emit panic call with the message
                let msg_global = self.intern_string(msg);
                let msg_len = msg.len() + 1;
                let msg_ptr = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = getelementptr [{} x i8], [{} x i8]* {}, i32 0, i32 0\n",
                    msg_ptr, msg_len, msg_len, msg_global
                ));
                let file_global = self.intern_string("axon_module.axon");
                let file_len = "axon_module.axon".len() + 1;
                let file_ptr = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = getelementptr [{} x i8], [{} x i8]* {}, i32 0, i32 0\n",
                    file_ptr, file_len, file_len, file_global
                ));
                self.output.push_str(&format!(
                    "  call void @axon_panic(i8* {}, i8* {}, i32 0)\n",
                    msg_ptr, file_ptr
                ));
                self.output.push_str("  unreachable\n");
            }
            Terminator::Unreachable => {
                self.output.push_str("  unreachable\n");
            }
        }
    }

    // ── Rvalue emission ────────────────────────────────────────

    fn emit_rvalue(&mut self, rvalue: &Rvalue, func: &MirFunction) -> String {
        match rvalue {
            Rvalue::Use(operand) => self.emit_operand(operand, func),
            Rvalue::BinaryOp { op, left, right } => self.emit_binary_op(op, left, right, func),
            Rvalue::UnaryOp { op, operand } => self.emit_unary_op(op, operand, func),
            Rvalue::Ref { place, .. } => self.emit_place(place),
            Rvalue::Aggregate { kind, fields } => self.emit_aggregate(kind, fields, func),
            Rvalue::Cast {
                operand,
                target_ty,
            } => self.emit_cast(operand, *target_ty, func),
            Rvalue::Len { place } => {
                // Return the compile-time known length for arrays, or load from metadata
                let ty = self.local_type(func, place.local);
                let resolved = self.interner.resolve(ty);
                match resolved {
                    Type::Array { size, .. } => {
                        // Known-size array: return the compile-time constant
                        let name = self.fresh_name();
                        self.output
                            .push_str(&format!("  {} = add i64 0, {}\n", name, size));
                        name
                    }
                    Type::Tensor(tensor_ty) => {
                        // For tensors, compute the total length from shape dimensions
                        // Use the first dimension if available at compile time
                        let mut known_len: Option<i64> = None;
                        if !tensor_ty.shape.is_empty() {
                            let mut product: i64 = 1;
                            let mut all_known = true;
                            for dim in &tensor_ty.shape {
                                if let crate::types::ShapeDimResolved::Known(n) = dim {
                                    product *= n;
                                } else {
                                    all_known = false;
                                    break;
                                }
                            }
                            if all_known {
                                known_len = Some(product);
                            }
                        }
                        if let Some(len) = known_len {
                            let name = self.fresh_name();
                            self.output
                                .push_str(&format!("  {} = add i64 0, {}\n", name, len));
                            name
                        } else {
                            // Dynamic tensor: load ndim from the tensor header
                            let place_name = self.emit_place(place);
                            let ndim_ptr = self.fresh_name();
                            let ty_str = self.llvm_alloca_type(ty);
                            self.output.push_str(&format!(
                                "  {} = getelementptr {}, {}* {}, i32 0, i32 2\n",
                                ndim_ptr, ty_str, ty_str, place_name
                            ));
                            let ndim_val = self.fresh_name();
                            self.output.push_str(&format!(
                                "  {} = load i64, i64* {}\n",
                                ndim_val, ndim_ptr
                            ));
                            ndim_val
                        }
                    }
                    Type::Primitive(PrimKind::String) => {
                        // String length: extract from { i8*, i64, i64 } struct
                        let _place_name = self.emit_place(place);
                        let val = self.emit_place_load(place, func);
                        let len_name = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = extractvalue {{ i8*, i64, i64 }} {}, 1\n",
                            len_name, val
                        ));
                        len_name
                    }
                    _ => {
                        // Fallback: return 0
                        let _val = self.emit_place(place);
                        let name = self.fresh_name();
                        self.output
                            .push_str(&format!("  {} = add i64 0, 0\n", name));
                        name
                    }
                }
            }
            Rvalue::TensorOp { kind, operands } => self.emit_tensor_op(kind, operands, func),
            Rvalue::Discriminant { place } => {
                // Load the discriminant (tag) of an enum/option
                let place_name = self.emit_place(place);
                let tag_ptr = self.fresh_name();
                let ty = self.local_type(func, place.local);
                let ty_str = self.llvm_alloca_type(ty);
                self.output.push_str(&format!(
                    "  {} = getelementptr {}, {}* {}, i32 0, i32 0\n",
                    tag_ptr, ty_str, ty_str, place_name
                ));
                let tag_val = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = load i8, i8* {}\n",
                    tag_val, tag_ptr
                ));
                let tag_ext = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = sext i8 {} to i64\n",
                    tag_ext, tag_val
                ));
                tag_ext
            }
        }
    }

    fn emit_binary_op(
        &mut self,
        op: &MirBinOp,
        left: &Operand,
        right: &Operand,
        func: &MirFunction,
    ) -> String {
        let lhs = self.emit_operand(left, func);
        let rhs = self.emit_operand(right, func);
        let ty = self.type_of_operand(left, func);
        let resolved = self.interner.resolve(ty);

        let is_float = matches!(resolved, Type::Primitive(p) if p.is_float());
        let is_unsigned = matches!(
            resolved,
            Type::Primitive(
                PrimKind::UInt8 | PrimKind::UInt16 | PrimKind::UInt32 | PrimKind::UInt64
            )
        );
        let ty_str = self.llvm_alloca_type(ty);
        let name = self.fresh_name();

        match op {
            MirBinOp::Add => {
                let inst = if is_float { "fadd" } else { "add" };
                self.output
                    .push_str(&format!("  {} = {} {} {}, {}\n", name, inst, ty_str, lhs, rhs));
            }
            MirBinOp::Sub => {
                let inst = if is_float { "fsub" } else { "sub" };
                self.output
                    .push_str(&format!("  {} = {} {} {}, {}\n", name, inst, ty_str, lhs, rhs));
            }
            MirBinOp::Mul => {
                let inst = if is_float { "fmul" } else { "mul" };
                self.output
                    .push_str(&format!("  {} = {} {} {}, {}\n", name, inst, ty_str, lhs, rhs));
            }
            MirBinOp::Div => {
                let inst = if is_float {
                    "fdiv"
                } else if is_unsigned {
                    "udiv"
                } else {
                    "sdiv"
                };
                self.output
                    .push_str(&format!("  {} = {} {} {}, {}\n", name, inst, ty_str, lhs, rhs));
            }
            MirBinOp::Mod => {
                let inst = if is_float {
                    "frem"
                } else if is_unsigned {
                    "urem"
                } else {
                    "srem"
                };
                self.output
                    .push_str(&format!("  {} = {} {} {}, {}\n", name, inst, ty_str, lhs, rhs));
            }
            MirBinOp::Eq => {
                let inst = if is_float {
                    format!("fcmp oeq {} {}, {}", ty_str, lhs, rhs)
                } else {
                    format!("icmp eq {} {}, {}", ty_str, lhs, rhs)
                };
                self.output
                    .push_str(&format!("  {} = {}\n", name, inst));
            }
            MirBinOp::Ne => {
                let inst = if is_float {
                    format!("fcmp one {} {}, {}", ty_str, lhs, rhs)
                } else {
                    format!("icmp ne {} {}, {}", ty_str, lhs, rhs)
                };
                self.output
                    .push_str(&format!("  {} = {}\n", name, inst));
            }
            MirBinOp::Lt => {
                let inst = if is_float {
                    format!("fcmp olt {} {}, {}", ty_str, lhs, rhs)
                } else if is_unsigned {
                    format!("icmp ult {} {}, {}", ty_str, lhs, rhs)
                } else {
                    format!("icmp slt {} {}, {}", ty_str, lhs, rhs)
                };
                self.output
                    .push_str(&format!("  {} = {}\n", name, inst));
            }
            MirBinOp::Le => {
                let inst = if is_float {
                    format!("fcmp ole {} {}, {}", ty_str, lhs, rhs)
                } else if is_unsigned {
                    format!("icmp ule {} {}, {}", ty_str, lhs, rhs)
                } else {
                    format!("icmp sle {} {}, {}", ty_str, lhs, rhs)
                };
                self.output
                    .push_str(&format!("  {} = {}\n", name, inst));
            }
            MirBinOp::Gt => {
                let inst = if is_float {
                    format!("fcmp ogt {} {}, {}", ty_str, lhs, rhs)
                } else if is_unsigned {
                    format!("icmp ugt {} {}, {}", ty_str, lhs, rhs)
                } else {
                    format!("icmp sgt {} {}, {}", ty_str, lhs, rhs)
                };
                self.output
                    .push_str(&format!("  {} = {}\n", name, inst));
            }
            MirBinOp::Ge => {
                let inst = if is_float {
                    format!("fcmp oge {} {}, {}", ty_str, lhs, rhs)
                } else if is_unsigned {
                    format!("icmp uge {} {}, {}", ty_str, lhs, rhs)
                } else {
                    format!("icmp sge {} {}, {}", ty_str, lhs, rhs)
                };
                self.output
                    .push_str(&format!("  {} = {}\n", name, inst));
            }
            MirBinOp::And => {
                self.output
                    .push_str(&format!("  {} = and {} {}, {}\n", name, ty_str, lhs, rhs));
            }
            MirBinOp::Or => {
                self.output
                    .push_str(&format!("  {} = or {} {}, {}\n", name, ty_str, lhs, rhs));
            }
            MirBinOp::Shl => {
                self.output
                    .push_str(&format!("  {} = shl {} {}, {}\n", name, ty_str, lhs, rhs));
            }
            MirBinOp::Shr => {
                let inst = if is_unsigned { "lshr" } else { "ashr" };
                self.output
                    .push_str(&format!("  {} = {} {} {}, {}\n", name, inst, ty_str, lhs, rhs));
            }
        }

        name
    }

    fn emit_unary_op(
        &mut self,
        op: &MirUnaryOp,
        operand: &Operand,
        func: &MirFunction,
    ) -> String {
        let val = self.emit_operand(operand, func);
        let ty = self.type_of_operand(operand, func);
        let resolved = self.interner.resolve(ty);
        let is_float = matches!(resolved, Type::Primitive(p) if p.is_float());
        let ty_str = self.llvm_alloca_type(ty);
        let name = self.fresh_name();

        match op {
            MirUnaryOp::Neg => {
                if is_float {
                    self.output
                        .push_str(&format!("  {} = fneg {} {}\n", name, ty_str, val));
                } else {
                    self.output.push_str(&format!(
                        "  {} = sub {} 0, {}\n",
                        name, ty_str, val
                    ));
                }
            }
            MirUnaryOp::Not => {
                // For booleans, xor with 1; for integers, xor with -1
                let is_bool = matches!(resolved, Type::Primitive(PrimKind::Bool));
                if is_bool {
                    self.output.push_str(&format!(
                        "  {} = xor i1 {}, true\n",
                        name, val
                    ));
                } else {
                    self.output.push_str(&format!(
                        "  {} = xor {} {}, -1\n",
                        name, ty_str, val
                    ));
                }
            }
            MirUnaryOp::Deref => {
                // Load through pointer
                self.output.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    name, ty_str, ty_str, val
                ));
            }
        }

        name
    }

    fn emit_cast(
        &mut self,
        operand: &Operand,
        target_ty: TypeId,
        func: &MirFunction,
    ) -> String {
        let val = self.emit_operand(operand, func);
        let src_ty = self.type_of_operand(operand, func);
        let src_resolved = self.interner.resolve(src_ty);
        let dst_resolved = self.interner.resolve(target_ty);
        let src_str = self.llvm_alloca_type(src_ty);
        let dst_str = self.llvm_alloca_type(target_ty);
        let name = self.fresh_name();

        let src_is_float = matches!(src_resolved, Type::Primitive(p) if p.is_float());
        let dst_is_float = matches!(dst_resolved, Type::Primitive(p) if p.is_float());
        let src_is_unsigned = matches!(
            src_resolved,
            Type::Primitive(
                PrimKind::UInt8 | PrimKind::UInt16 | PrimKind::UInt32 | PrimKind::UInt64
            )
        );

        if src_is_float && dst_is_float {
            // float <-> float
            let src_bits = Self::float_bits(src_resolved);
            let dst_bits = Self::float_bits(dst_resolved);
            if dst_bits > src_bits {
                self.output.push_str(&format!(
                    "  {} = fpext {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            } else if dst_bits < src_bits {
                self.output.push_str(&format!(
                    "  {} = fptrunc {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            } else {
                // same size, bitcast
                self.output.push_str(&format!(
                    "  {} = bitcast {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            }
        } else if src_is_float && !dst_is_float {
            // float -> int
            if src_is_unsigned {
                self.output.push_str(&format!(
                    "  {} = fptoui {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            } else {
                self.output.push_str(&format!(
                    "  {} = fptosi {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            }
        } else if !src_is_float && dst_is_float {
            // int -> float
            if src_is_unsigned {
                self.output.push_str(&format!(
                    "  {} = uitofp {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            } else {
                self.output.push_str(&format!(
                    "  {} = sitofp {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            }
        } else {
            // int <-> int
            let src_bits = Self::int_bits(src_resolved);
            let dst_bits = Self::int_bits(dst_resolved);
            if dst_bits > src_bits {
                if src_is_unsigned {
                    self.output.push_str(&format!(
                        "  {} = zext {} {} to {}\n",
                        name, src_str, val, dst_str
                    ));
                } else {
                    self.output.push_str(&format!(
                        "  {} = sext {} {} to {}\n",
                        name, src_str, val, dst_str
                    ));
                }
            } else if dst_bits < src_bits {
                self.output.push_str(&format!(
                    "  {} = trunc {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            } else {
                self.output.push_str(&format!(
                    "  {} = bitcast {} {} to {}\n",
                    name, src_str, val, dst_str
                ));
            }
        }

        name
    }

    fn emit_aggregate(
        &mut self,
        kind: &AggregateKind,
        fields: &[Operand],
        func: &MirFunction,
    ) -> String {
        // Build an aggregate by successive insertvalue instructions.
        match kind {
            AggregateKind::Tuple | AggregateKind::Struct(_) | AggregateKind::Array => {
                // Determine the aggregate LLVM type from fields
                let field_types: Vec<String> = fields
                    .iter()
                    .map(|f| self.llvm_alloca_type(self.type_of_operand(f, func)))
                    .collect();
                let agg_ty = match kind {
                    AggregateKind::Array if !field_types.is_empty() => {
                        format!("[{} x {}]", fields.len(), field_types[0])
                    }
                    _ => format!("{{ {} }}", field_types.join(", ")),
                };

                let mut current_name = "undef".to_string();
                for (i, field) in fields.iter().enumerate() {
                    let val = self.emit_operand(field, func);
                    let name = self.fresh_name();
                    let field_ty = self.llvm_alloca_type(self.type_of_operand(field, func));
                    self.output.push_str(&format!(
                        "  {} = insertvalue {} {}, {} {}, {}\n",
                        name, agg_ty, current_name, field_ty, val, i
                    ));
                    current_name = name;
                }
                if fields.is_empty() {
                    // Return undef for empty aggregate
                    let name = self.fresh_name();
                    self.output.push_str(&format!(
                        "  {} = insertvalue {{}} undef, {{}} undef, 0 ; empty agg\n",
                        name
                    ));
                    // Actually, for empty tuple just use zeroinitializer
                    return "undef".to_string();
                }
                current_name
            }
            AggregateKind::Enum(name, variant_idx) => {
                // Determine the full enum LLVM type
                let enum_ty_id = self.find_enum_type(name);
                let enum_ty_str = self.llvm_alloca_type(enum_ty_id);

                // Insert tag at index 0
                let name_tag = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = insertvalue {} undef, i8 {}, 0\n",
                    name_tag, enum_ty_str, variant_idx
                ));

                if fields.is_empty() {
                    // Unit variant — just the tag
                    name_tag
                } else {
                    // Payload variant: insert each payload field at subsequent indices
                    // For enum layout { i8, [N x i8] }, we need to bitcast payload fields
                    // into the byte array. For simplicity, use insertvalue on the payload area.
                    let mut current_name = name_tag;
                    for (i, field) in fields.iter().enumerate() {
                        let val = self.emit_operand(field, func);
                        let field_ty = self.llvm_alloca_type(self.type_of_operand(field, func));
                        let next = self.fresh_name();
                        // Payload fields go at index 1, 2, ... in the enum struct
                        // For { i8, [N x i8] } layout, we insertvalue into the byte array
                        // For simpler enums, insert directly
                        self.output.push_str(&format!(
                            "  {} = insertvalue {} {}, {} {}, {}\n",
                            next, enum_ty_str, current_name, field_ty, val, i + 1
                        ));
                        current_name = next;
                    }
                    current_name
                }
            }
        }
    }

    fn emit_tensor_op(
        &mut self,
        kind: &TensorOpKind,
        operands: &[Operand],
        func: &MirFunction,
    ) -> String {
        // Tensor ops delegate to runtime function call stubs.
        // Each op emits a call to @axon_tensor_<op>(i8*, ...) -> i8*
        // which returns an opaque tensor pointer. The actual implementation
        // is provided by the tensor runtime library (linked at compile time).
        match kind {
            TensorOpKind::MatMul => {
                let lhs = self.emit_operand(&operands[0], func);
                let rhs = self.emit_operand(&operands[1], func);
                let lhs_ptr = self.fresh_name();
                let rhs_ptr = self.fresh_name();
                let result = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = inttoptr i64 {} to i8*\n",
                    lhs_ptr, lhs
                ));
                self.output.push_str(&format!(
                    "  {} = inttoptr i64 {} to i8*\n",
                    rhs_ptr, rhs
                ));
                self.output.push_str(&format!(
                    "  {} = call i8* @axon_tensor_matmul(i8* {}, i8* {})\n",
                    result, lhs_ptr, rhs_ptr
                ));
                result
            }
            TensorOpKind::Add
            | TensorOpKind::Sub
            | TensorOpKind::Mul
            | TensorOpKind::Div => {
                let op_name = match kind {
                    TensorOpKind::Add => "add",
                    TensorOpKind::Sub => "sub",
                    TensorOpKind::Mul => "mul",
                    TensorOpKind::Div => "div",
                    _ => unreachable!(),
                };
                if operands.len() >= 2 {
                    let lhs = self.emit_operand(&operands[0], func);
                    let rhs = self.emit_operand(&operands[1], func);
                    let lhs_ptr = self.fresh_name();
                    let rhs_ptr = self.fresh_name();
                    let result = self.fresh_name();
                    self.output.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        lhs_ptr, lhs
                    ));
                    self.output.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        rhs_ptr, rhs
                    ));
                    self.output.push_str(&format!(
                        "  {} = call i8* @axon_tensor_{}(i8* {}, i8* {})\n",
                        result, op_name, lhs_ptr, rhs_ptr
                    ));
                    result
                } else if !operands.is_empty() {
                    self.emit_operand(&operands[0], func)
                } else {
                    "zeroinitializer".to_string()
                }
            }
            TensorOpKind::Reshape(_) => {
                if !operands.is_empty() {
                    let src = self.emit_operand(&operands[0], func);
                    let src_ptr = self.fresh_name();
                    let result = self.fresh_name();
                    self.output.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        src_ptr, src
                    ));
                    self.output.push_str(&format!(
                        "  {} = call i8* @axon_tensor_reshape(i8* {})\n",
                        result, src_ptr
                    ));
                    result
                } else {
                    "zeroinitializer".to_string()
                }
            }
            TensorOpKind::Transpose => {
                if !operands.is_empty() {
                    let src = self.emit_operand(&operands[0], func);
                    let src_ptr = self.fresh_name();
                    let result = self.fresh_name();
                    self.output.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        src_ptr, src
                    ));
                    self.output.push_str(&format!(
                        "  {} = call i8* @axon_tensor_transpose(i8* {})\n",
                        result, src_ptr
                    ));
                    result
                } else {
                    "zeroinitializer".to_string()
                }
            }
            TensorOpKind::Broadcast => {
                if !operands.is_empty() {
                    let src = self.emit_operand(&operands[0], func);
                    let src_ptr = self.fresh_name();
                    let result = self.fresh_name();
                    self.output.push_str(&format!(
                        "  {} = inttoptr i64 {} to i8*\n",
                        src_ptr, src
                    ));
                    self.output.push_str(&format!(
                        "  {} = call i8* @axon_tensor_broadcast(i8* {})\n",
                        result, src_ptr
                    ));
                    result
                } else {
                    "zeroinitializer".to_string()
                }
            }
        }
    }

    // ── Operand emission ───────────────────────────────────────

    fn emit_operand(&mut self, operand: &Operand, func: &MirFunction) -> String {
        match operand {
            Operand::Place(place) => self.emit_place_load(place, func),
            Operand::Constant(constant) => self.emit_constant(constant),
        }
    }

    fn emit_place(&self, place: &Place) -> String {
        if place.projections.is_empty() {
            self.local_name(place.local)
        } else {
            // For projected places we'd need GEP chains; return base for now.
            self.local_name(place.local)
        }
    }

    fn emit_place_load(&mut self, place: &Place, func: &MirFunction) -> String {
        let ty = self.local_type(func, place.local);
        let ty_str = self.llvm_alloca_type(ty);
        if ty_str == "{}" || ty_str == "void" {
            return "undef".to_string();
        }

        if place.projections.is_empty() {
            let name = self.fresh_name();
            self.output.push_str(&format!(
                "  {} = load {}, {}* {}\n",
                name,
                ty_str,
                ty_str,
                self.local_name(place.local)
            ));
            name
        } else {
            // Handle projections with GEP
            let mut current_ptr = self.local_name(place.local);
            let mut current_ty_id = ty;
            let mut current_ty_str = ty_str.clone();
            for proj in &place.projections {
                match proj {
                    Projection::Field(idx) => {
                        let gep = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                            gep, current_ty_str, current_ty_str, current_ptr, idx
                        ));
                        current_ptr = gep;
                        // Resolve the actual field type
                        current_ty_id = self.field_type_of(current_ty_id, *idx);
                        current_ty_str = self.llvm_alloca_type(current_ty_id);
                    }
                    Projection::Index(idx_op) => {
                        let idx_val = self.emit_operand(idx_op, func);
                        let gep = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                            gep, current_ty_str, current_ty_str, current_ptr, idx_val
                        ));
                        current_ptr = gep;
                        // Resolve element type
                        let resolved = self.interner.resolve(current_ty_id);
                        if let Type::Array { elem, .. } = resolved {
                            current_ty_id = *elem;
                            current_ty_str = self.llvm_alloca_type(current_ty_id);
                        }
                    }
                    Projection::Deref => {
                        let loaded = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = load {}*, {}** {}\n",
                            loaded, current_ty_str, current_ty_str, current_ptr
                        ));
                        current_ptr = loaded;
                        // Resolve deref type
                        let resolved = self.interner.resolve(current_ty_id);
                        if let Type::Reference { inner, .. } = resolved {
                            current_ty_id = *inner;
                            current_ty_str = self.llvm_alloca_type(current_ty_id);
                        }
                    }
                }
            }
            let final_val = self.fresh_name();
            self.output.push_str(&format!(
                "  {} = load {}, {}* {}\n",
                final_val, current_ty_str, current_ty_str, current_ptr
            ));
            final_val
        }
    }

    fn emit_place_store(&mut self, place: &Place, value: &str, ty_str: &str) {
        if place.projections.is_empty() {
            self.output.push_str(&format!(
                "  store {} {}, {}* {}\n",
                ty_str,
                value,
                ty_str,
                self.local_name(place.local)
            ));
        } else {
            // For projected stores, use GEP chain to find the target field address.
            let base_ty = self.local_type_from_cache(place.local);
            let mut current_ptr = self.local_name(place.local);
            let mut current_ty_id = base_ty;
            let mut current_ty_str = self.llvm_alloca_type(base_ty);

            for proj in &place.projections {
                match proj {
                    Projection::Field(idx) => {
                        let gep = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = getelementptr {}, {}* {}, i32 0, i32 {}\n",
                            gep, current_ty_str, current_ty_str, current_ptr, idx
                        ));
                        current_ptr = gep;
                        // Resolve the field type
                        current_ty_id = self.field_type_of(current_ty_id, *idx);
                        current_ty_str = self.llvm_alloca_type(current_ty_id);
                    }
                    Projection::Index(idx_op) => {
                        // For arrays, we need to GEP into the array element
                        // We need a mutable borrow of self for emit_operand_no_func
                        let idx_val = self.emit_operand_index(idx_op);
                        let gep = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = getelementptr {}, {}* {}, i64 0, i64 {}\n",
                            gep, current_ty_str, current_ty_str, current_ptr, idx_val
                        ));
                        current_ptr = gep;
                        // Resolve element type for arrays
                        let resolved = self.interner.resolve(current_ty_id);
                        if let Type::Array { elem, .. } = resolved {
                            current_ty_id = *elem;
                            current_ty_str = self.llvm_alloca_type(current_ty_id);
                        }
                    }
                    Projection::Deref => {
                        let loaded = self.fresh_name();
                        self.output.push_str(&format!(
                            "  {} = load {}*, {}** {}\n",
                            loaded, current_ty_str, current_ty_str, current_ptr
                        ));
                        current_ptr = loaded;
                    }
                }
            }
            // Store the value at the final address
            self.output.push_str(&format!(
                "  store {} {}, {}* {}\n",
                ty_str,
                value,
                ty_str,
                current_ptr
            ));
        }
    }

    /// Emit an operand value for index projections (without needing func context).
    /// This handles the common cases: constant integers and place loads using cached locals.
    fn emit_operand_index(&mut self, operand: &Operand) -> String {
        match operand {
            Operand::Constant(c) => self.emit_constant(c),
            Operand::Place(place) => {
                // Use cached local types for place loads in store context
                let ty = self.local_type_from_cache(place.local);
                let ty_str = self.llvm_alloca_type(ty);
                let name = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = load {}, {}* {}\n",
                    name, ty_str, ty_str, self.local_name(place.local)
                ));
                name
            }
        }
    }

    /// Look up a local's type from the cached current function locals.
    fn local_type_from_cache(&self, local: LocalId) -> TypeId {
        self.current_func_locals
            .get(local.0 as usize)
            .map(|l| l.ty)
            .unwrap_or(TypeId::UNIT)
    }

    fn emit_constant(&mut self, constant: &MirConstant) -> String {
        match constant {
            MirConstant::Int(v) => format!("{}", v),
            MirConstant::Float(v) => {
                // LLVM requires hex format for exact representation of floats
                let bits = (*v as f64).to_bits();
                format!("0x{:016X}", bits)
            }
            MirConstant::Bool(v) => {
                if *v {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            MirConstant::Char(c) => format!("{}", *c as u32),
            MirConstant::String(s) => {
                let global = self.intern_string(s);
                let len = s.len() + 1;
                let ptr_name = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = getelementptr [{} x i8], [{} x i8]* {}, i32 0, i32 0\n",
                    ptr_name, len, len, global
                ));
                // Build a { i8*, i64, i64 } struct with (ptr, len, capacity)
                let str_len = s.len() as i64;
                let s1 = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = insertvalue {{ i8*, i64, i64 }} undef, i8* {}, 0\n",
                    s1, ptr_name
                ));
                let s2 = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = insertvalue {{ i8*, i64, i64 }} {}, i64 {}, 1\n",
                    s2, s1, str_len
                ));
                let s3 = self.fresh_name();
                self.output.push_str(&format!(
                    "  {} = insertvalue {{ i8*, i64, i64 }} {}, i64 {}, 2\n",
                    s3, s2, str_len
                ));
                s3
            }
            MirConstant::Unit => "undef".to_string(),
        }
    }

    // ── Helpers ────────────────────────────────────────────────

    fn fresh_name(&mut self) -> String {
        let n = self.next_unnamed;
        self.next_unnamed += 1;
        format!("%{}", n)
    }

    fn local_name(&self, local: LocalId) -> String {
        format!("%_{}", local.0)
    }

    fn block_label(&self, block: BlockId) -> String {
        format!("bb{}", block.0)
    }

    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn intern_string(&mut self, s: &str) -> String {
        // Check if already interned
        for (name, val) in &self.string_literals {
            if val == s {
                return name.clone();
            }
        }
        let idx = self.string_literals.len();
        let name = format!("@.str.{}", idx);
        self.string_literals.push((name.clone(), s.to_string()));
        name
    }

    fn local_type(&self, func: &MirFunction, local: LocalId) -> TypeId {
        func.locals
            .get(local.0 as usize)
            .map(|l| l.ty)
            .unwrap_or(TypeId::UNIT)
    }

    /// Determine the TypeId of an operand within the context of a function.
    fn type_of_operand(&self, operand: &Operand, func: &MirFunction) -> TypeId {
        match operand {
            Operand::Place(place) => {
                if place.projections.is_empty() {
                    self.local_type(func, place.local)
                } else {
                    // Resolve type through projections
                    self.type_of_place(place, func)
                }
            }
            Operand::Constant(c) => match c {
                MirConstant::Int(_) => TypeId::INT64,
                MirConstant::Float(_) => TypeId::FLOAT64,
                MirConstant::Bool(_) => TypeId::BOOL,
                MirConstant::Char(_) => TypeId::CHAR,
                MirConstant::String(_) => TypeId::STRING,
                MirConstant::Unit => TypeId::UNIT,
            },
        }
    }

    /// Determine the type produced by an rvalue.
    fn type_of_rvalue(&self, rvalue: &Rvalue, func: &MirFunction) -> TypeId {
        match rvalue {
            Rvalue::Use(op) => self.type_of_operand(op, func),
            Rvalue::BinaryOp { op, left, .. } => {
                // Comparison ops return Bool
                match op {
                    MirBinOp::Eq | MirBinOp::Ne | MirBinOp::Lt | MirBinOp::Le
                    | MirBinOp::Gt | MirBinOp::Ge => TypeId::BOOL,
                    _ => self.type_of_operand(left, func),
                }
            }
            Rvalue::UnaryOp { operand, .. } => self.type_of_operand(operand, func),
            Rvalue::Ref { place, mutable } => {
                // Reference type: look up the inner type and find/create a reference TypeId.
                // Since we only need the LLVM type string, we look up the place's type
                // and the llvm_type for Reference already produces "T*".
                // However, type_of_rvalue returns TypeId so we search the interner.
                let inner_ty = self.local_type(func, place.local);
                self.find_or_approximate_ref_type(inner_ty, *mutable)
            }
            Rvalue::Aggregate { kind, fields } => {
                // Determine the aggregate type from the kind and fields.
                match kind {
                    AggregateKind::Tuple => {
                        // Build a tuple type from field types
                        let field_tys: Vec<TypeId> = fields
                            .iter()
                            .map(|f| self.type_of_operand(f, func))
                            .collect();
                        // Search the interner for a matching tuple type
                        self.find_tuple_type(&field_tys)
                    }
                    AggregateKind::Struct(name) => {
                        // Search the interner for a struct with this name
                        self.find_struct_type(name)
                    }
                    AggregateKind::Array => {
                        // Build array type from fields
                        if let Some(first) = fields.first() {
                            let elem_ty = self.type_of_operand(first, func);
                            self.find_array_type(elem_ty, fields.len())
                        } else {
                            TypeId::UNIT
                        }
                    }
                    AggregateKind::Enum(name, _variant_idx) => {
                        // Search the interner for an enum with this name
                        self.find_enum_type(name)
                    }
                }
            }
            Rvalue::Cast { target_ty, .. } => *target_ty,
            Rvalue::Len { .. } => TypeId::INT64,
            Rvalue::TensorOp { operands, .. } => {
                if let Some(op) = operands.first() {
                    self.type_of_operand(op, func)
                } else {
                    TypeId::UNIT
                }
            }
            Rvalue::Discriminant { .. } => TypeId::INT64,
        }
    }

    /// Find or approximate a reference type in the interner.
    /// Since we can't mutate the interner (we only have &), we search for an existing one.
    /// If not found, we approximate: references are pointers in LLVM, so we return
    /// a sentinel that maps to pointer type. We search the interner linearly.
    fn find_or_approximate_ref_type(&self, inner: TypeId, mutable: bool) -> TypeId {
        let target = Type::Reference { mutable, inner };
        for i in 0..self.interner.len() {
            let tid = TypeId(i as u32);
            if *self.interner.resolve(tid) == target {
                return tid;
            }
        }
        // Also try the opposite mutability — LLVM type is the same (T*)
        let target_alt = Type::Reference { mutable: !mutable, inner };
        for i in 0..self.interner.len() {
            let tid = TypeId(i as u32);
            if *self.interner.resolve(tid) == target_alt {
                return tid;
            }
        }
        // Fallback: INT64 has same size as a pointer on 64-bit
        TypeId::INT64
    }

    /// Find a tuple type in the interner matching the given field types.
    fn find_tuple_type(&self, field_tys: &[TypeId]) -> TypeId {
        let target = Type::Tuple(field_tys.to_vec());
        for i in 0..self.interner.len() {
            let tid = TypeId(i as u32);
            if *self.interner.resolve(tid) == target {
                return tid;
            }
        }
        TypeId::UNIT
    }

    /// Find a struct type in the interner by name.
    fn find_struct_type(&self, name: &str) -> TypeId {
        for i in 0..self.interner.len() {
            let tid = TypeId(i as u32);
            if let Type::Struct { name: n, .. } = self.interner.resolve(tid) {
                if n == name {
                    return tid;
                }
            }
        }
        TypeId::UNIT
    }

    /// Find an enum type in the interner by name.
    fn find_enum_type(&self, name: &str) -> TypeId {
        for i in 0..self.interner.len() {
            let tid = TypeId(i as u32);
            if let Type::Enum { name: n, .. } = self.interner.resolve(tid) {
                if n == name {
                    return tid;
                }
            }
        }
        TypeId::UNIT
    }

    /// Find an array type in the interner.
    fn find_array_type(&self, elem: TypeId, size: usize) -> TypeId {
        let target = Type::Array { elem, size };
        for i in 0..self.interner.len() {
            let tid = TypeId(i as u32);
            if *self.interner.resolve(tid) == target {
                return tid;
            }
        }
        TypeId::UNIT
    }

    /// Resolve the type of a place, including projections.
    /// For a place like `local.field(2)`, returns the type of field 2 of the struct.
    fn type_of_place(&self, place: &Place, func: &MirFunction) -> TypeId {
        let mut current_ty = self.local_type(func, place.local);
        for proj in &place.projections {
            match proj {
                Projection::Field(idx) => {
                    current_ty = self.field_type_of(current_ty, *idx);
                }
                Projection::Index(_) => {
                    // For arrays, indexing yields the element type
                    let resolved = self.interner.resolve(current_ty);
                    if let Type::Array { elem, .. } = resolved {
                        current_ty = *elem;
                    }
                }
                Projection::Deref => {
                    // Deref a reference -> inner type
                    let resolved = self.interner.resolve(current_ty);
                    if let Type::Reference { inner, .. } = resolved {
                        current_ty = *inner;
                    }
                }
            }
        }
        current_ty
    }

    /// Get the type of field `idx` within a struct/tuple/enum type.
    fn field_type_of(&self, ty: TypeId, idx: u32) -> TypeId {
        let resolved = self.interner.resolve(ty);
        match resolved {
            Type::Struct { fields, .. } => {
                if let Some((_, field_ty)) = fields.get(idx as usize) {
                    *field_ty
                } else {
                    TypeId::INT64 // fallback
                }
            }
            Type::Tuple(elems) => {
                if let Some(elem_ty) = elems.get(idx as usize) {
                    *elem_ty
                } else {
                    TypeId::INT64
                }
            }
            Type::Enum { .. } => {
                // Enum fields: index 0 is tag (i8), rest are payload
                if idx == 0 {
                    TypeId::INT64 // tag, approximated
                } else {
                    TypeId::INT64 // payload field, approximated
                }
            }
            _ => TypeId::INT64,
        }
    }

    /// Escape a string for LLVM IR string constant syntax.
    fn escape_llvm_string(s: &str) -> String {
        let mut out = String::new();
        for byte in s.bytes() {
            match byte {
                b'\\' => out.push_str("\\5C"),
                b'"' => out.push_str("\\22"),
                b'\n' => out.push_str("\\0A"),
                b'\r' => out.push_str("\\0D"),
                b'\t' => out.push_str("\\09"),
                0x20..=0x7E => out.push(byte as char),
                _ => out.push_str(&format!("\\{:02X}", byte)),
            }
        }
        out
    }

    fn host_target_triple() -> &'static str {
        if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin"
        } else {
            "x86_64-unknown-linux-gnu"
        }
    }

    fn largest_enum_variant_size(
        &self,
        variants: &[(String, crate::types::EnumVariantType)],
    ) -> usize {
        use crate::types::EnumVariantType;
        let mut max_size = 0usize;
        for (_, variant) in variants {
            let size = match variant {
                EnumVariantType::Unit => 0,
                EnumVariantType::Tuple(fields) => fields.len() * 8, // rough estimate
                EnumVariantType::Struct(fields) => fields.len() * 8,
            };
            if size > max_size {
                max_size = size;
            }
        }
        max_size
    }

    fn float_bits(ty: &Type) -> u32 {
        match ty {
            Type::Primitive(PrimKind::Float16) => 16,
            Type::Primitive(PrimKind::Float32) => 32,
            Type::Primitive(PrimKind::Float64) => 64,
            _ => 64,
        }
    }

    fn int_bits(ty: &Type) -> u32 {
        match ty {
            Type::Primitive(PrimKind::Int8 | PrimKind::UInt8) => 8,
            Type::Primitive(PrimKind::Int16 | PrimKind::UInt16) => 16,
            Type::Primitive(PrimKind::Int32 | PrimKind::UInt32) => 32,
            Type::Primitive(PrimKind::Int64 | PrimKind::UInt64) => 64,
            Type::Primitive(PrimKind::Bool) => 1,
            Type::Primitive(PrimKind::Char) => 32,
            _ => 64,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Compilation Helpers
// ═══════════════════════════════════════════════════════════════

/// Compile LLVM IR text to a native binary using clang.
pub fn compile_ir_to_binary(
    ir: &str,
    output_path: &str,
    opt_level: OptLevel,
) -> Result<(), String> {
    use std::process::Command;

    let ir_path = format!("{}.ll", output_path);
    std::fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    let opt_flag = match opt_level {
        OptLevel::O0 => "-O0",
        OptLevel::O1 => "-O1",
        OptLevel::O2 => "-O2",
        OptLevel::O3 => "-O3",
    };

    let output = Command::new("clang")
        .args(&[&ir_path, "-o", output_path, opt_flag])
        .output()
        .map_err(|e| format!("Failed to invoke clang: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("clang failed: {}", stderr));
    }

    Ok(())
}

/// Compile LLVM IR text to an object file.
pub fn compile_ir_to_object(
    ir: &str,
    output_path: &str,
    opt_level: OptLevel,
) -> Result<(), String> {
    let ir_path = format!("{}.ll", output_path);
    std::fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR: {}", e))?;

    let opt_flag = match opt_level {
        OptLevel::O0 => "-O0",
        OptLevel::O1 => "-O1",
        OptLevel::O2 => "-O2",
        OptLevel::O3 => "-O3",
    };

    let obj_path = format!("{}.o", output_path);
    let output = std::process::Command::new("clang")
        .args(&["-c", &ir_path, "-o", &obj_path, opt_flag])
        .output()
        .map_err(|e| format!("Failed to invoke clang: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("clang failed: {}", stderr));
    }

    Ok(())
}

/// Compile LLVM IR + Axon C runtime into a native binary.
pub fn compile_and_link(
    ir: &str,
    output_path: &str,
    opt_level: OptLevel,
    keep_temps: bool,
) -> Result<(), String> {
    use std::process::Command;
    use std::fs;
    use crate::codegen::runtime::generate_runtime_c_source;

    // Create build directory
    let build_dir = format!("{}.axon_build", output_path);
    fs::create_dir_all(&build_dir).map_err(|e| format!("E5004: Failed to create build dir: {}", e))?;

    let ir_path = format!("{}/program.ll", build_dir);
    let rt_c_path = format!("{}/axon_runtime.c", build_dir);
    let rt_o_path = format!("{}/axon_runtime.o", build_dir);

    // Write IR
    fs::write(&ir_path, ir).map_err(|e| format!("E5004: Failed to write IR: {}", e))?;

    // Write runtime C source
    let rt_source = generate_runtime_c_source();
    fs::write(&rt_c_path, &rt_source).map_err(|e| format!("E5004: Failed to write runtime: {}", e))?;

    let opt_flag = match opt_level {
        OptLevel::O0 => "-O0",
        OptLevel::O1 => "-O1",
        OptLevel::O2 => "-O2",
        OptLevel::O3 => "-O3",
    };

    // Detect compiler
    let cc = detect_c_compiler()?;

    // Check if using MSVC cl.exe (careful not to match "clang")
    let is_msvc = cc.ends_with("cl") || cc.ends_with("cl.exe");

    if is_msvc {
        // ── MSVC path ────────────────────────────────────────────
        let rt_obj_path = format!("{}/axon_runtime.obj", build_dir);

        // Step 1: Compile runtime with cl.exe
        let rt_compile = Command::new(&cc)
            .args(&["/c", &rt_c_path, &format!("/Fo:{}", rt_obj_path), "/O2", "/nologo"])
            .output()
            .map_err(|e| format!("E5001: Failed to invoke {}: {}", cc, e))?;

        if !rt_compile.status.success() {
            let stderr = String::from_utf8_lossy(&rt_compile.stderr);
            return Err(format!("E5002: Runtime compilation failed:\n{}", stderr));
        }

        // Step 2: Compile IR + link with runtime
        let link_output = Command::new(&cc)
            .args(&[
                &ir_path,
                &rt_obj_path,
                &format!("/Fe:{}", output_path),
                "/nologo",
            ])
            .output()
            .map_err(|e| format!("E5001: Failed to invoke {}: {}", cc, e))?;

        if !link_output.status.success() {
            let stderr = String::from_utf8_lossy(&link_output.stderr);
            return Err(format!("E5003: Linking failed:\n{}", stderr));
        }
    } else {
        // ── GCC/Clang path ───────────────────────────────────────

        // Step 1: Compile runtime
        let rt_compile = Command::new(&cc)
            .args(&["-c", &rt_c_path, "-o", &rt_o_path, "-O2"])
            .output()
            .map_err(|e| format!("E5001: Failed to invoke {}: {}", cc, e))?;

        if !rt_compile.status.success() {
            let stderr = String::from_utf8_lossy(&rt_compile.stderr);
            return Err(format!("E5002: Runtime compilation failed:\n{}", stderr));
        }

        // Step 2: Compile IR + link with runtime
        let mut link_args = vec![
            ir_path.clone(),
            rt_o_path.clone(),
            "-o".to_string(),
            output_path.to_string(),
            opt_flag.to_string(),
            "-lm".to_string(),
        ];

        // On Windows, don't pass -lm
        if cfg!(target_os = "windows") {
            link_args.retain(|a| a != "-lm");
        }

        let link_output = Command::new(&cc)
            .args(&link_args)
            .output()
            .map_err(|e| format!("E5001: Failed to invoke {}: {}", cc, e))?;

        if !link_output.status.success() {
            let stderr = String::from_utf8_lossy(&link_output.stderr);
            return Err(format!("E5003: Linking failed:\n{}", stderr));
        }
    }

    // Clean up
    if !keep_temps {
        let _ = fs::remove_dir_all(&build_dir);
    }

    Ok(())
}

fn detect_c_compiler() -> Result<String, String> {
    // Try compilers on PATH first
    for compiler in &["clang", "gcc", "cc"] {
        if let Ok(output) = std::process::Command::new(compiler)
            .arg("--version")
            .output()
        {
            if output.status.success() {
                return Ok(compiler.to_string());
            }
        }
    }
    // On Windows, check well-known LLVM install path
    if cfg!(target_os = "windows") {
        let llvm_clang = r"C:\Program Files\LLVM\bin\clang.exe";
        if std::path::Path::new(llvm_clang).exists() {
            return Ok(llvm_clang.to_string());
        }
        // Try cl.exe as last resort
        if let Ok(output) = std::process::Command::new("cl.exe")
            .arg("/?")
            .output()
        {
            if output.status.success() || output.status.code() == Some(0) {
                return Ok("cl.exe".to_string());
            }
        }
    }
    Err("E5001: No C compiler found. Install clang, gcc, or MSVC.".to_string())
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeInterner;

    /// Helper: compile Axon source through the full pipeline and generate LLVM IR.
    fn generate_ir(source: &str) -> String {
        let (typed_program, _errors) = crate::check_source(source, "test.axon");
        let (checker, _) = crate::typeck::check(source, "test.axon");
        let mut builder = crate::mir::MirBuilder::new(&checker.interner);
        let mir = builder.build(&typed_program);
        let mut codegen = LlvmCodegen::new(&checker.interner, OptLevel::O0);
        codegen.generate(&mir).expect("generate() failed — no main function?")
    }

    // 1. Type mapping: Int32
    #[test]
    fn test_llvm_type_int32() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::INT32), "i32");
    }

    // 2. Type mapping: Float64
    #[test]
    fn test_llvm_type_float64() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::FLOAT64), "double");
    }

    // 3. Type mapping: Bool
    #[test]
    fn test_llvm_type_bool() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::BOOL), "i1");
    }

    // 4. Type mapping: Unit
    #[test]
    fn test_llvm_type_unit() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::UNIT), "void");
    }

    // 5. Type mapping: String
    #[test]
    fn test_llvm_type_string() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        let ty = cg.llvm_type(TypeId::STRING);
        assert!(ty.contains("i8*"), "String type should contain i8*: {}", ty);
        assert!(ty.contains("i64"), "String type should contain i64: {}", ty);
    }

    // 6. Generate empty function
    #[test]
    fn test_generate_empty_function() {
        let ir = generate_ir("fn main() {}");
        assert!(ir.contains("define"), "IR should contain define: {}", ir);
        assert!(ir.contains("@main"), "IR should contain @main: {}", ir);
        assert!(ir.contains("ret"), "IR should contain ret: {}", ir);
    }

    // 7. Generate return constant
    #[test]
    fn test_generate_return_constant() {
        let ir = generate_ir("fn answer() -> Int64 { return 42; }\nfn main() { let x: Int64 = answer(); }");
        assert!(
            ir.contains("define"),
            "IR should contain define: {}",
            ir
        );
        assert!(ir.contains("42"), "IR should contain constant 42: {}", ir);
        assert!(ir.contains("ret"), "IR should contain ret: {}", ir);
    }

    // 8. Generate binary op
    #[test]
    fn test_generate_binary_op() {
        let ir = generate_ir(
            "fn test_add() -> Int64 { let x: Int64 = 1 + 2; return x; }\nfn main() { let r: Int64 = test_add(); }",
        );
        assert!(ir.contains("add"), "IR should contain add instruction: {}", ir);
    }

    // 9. Generate if-else
    #[test]
    fn test_generate_if_else() {
        let ir = generate_ir(
            "fn test_if(x: Bool) -> Int64 { if x { return 1; } else { return 2; } }\nfn main() { let r: Int64 = test_if(true); }",
        );
        assert!(ir.contains("br"), "IR should contain branch: {}", ir);
        assert!(
            ir.contains("bb"),
            "IR should contain basic block labels: {}",
            ir
        );
    }

    // 10. Generate while loop
    #[test]
    fn test_generate_while_loop() {
        let ir = generate_ir(
            "fn test_while() { let mut i: Int64 = 0; while i < 10 { i = i + 1; } }\nfn main() { test_while(); }",
        );
        assert!(ir.contains("br"), "IR should contain branch: {}", ir);
        // Loop generates multiple basic blocks
        assert!(ir.contains("bb"), "IR should contain bb labels: {}", ir);
    }

    // 11. Generate function call
    #[test]
    fn test_generate_function_call() {
        let ir = generate_ir(
            "fn add(a: Int64, b: Int64) -> Int64 { return a + b; }\nfn main() { let x: Int64 = add(1, 2); }",
        );
        assert!(ir.contains("call"), "IR should contain call: {}", ir);
    }

    // 12. Generate string literal
    #[test]
    fn test_generate_string_literal() {
        let ir = generate_ir(
            "fn main() { let s: String = \"hello\"; }",
        );
        assert!(
            ir.contains("hello") || ir.contains("@.str"),
            "IR should contain string literal: {}",
            ir
        );
    }

    // 13. Module header
    #[test]
    fn test_module_header() {
        let ir = generate_ir("fn main() {}");
        assert!(
            ir.starts_with("; ModuleID"),
            "IR should start with ModuleID: {}",
            ir
        );
        assert!(
            ir.contains("target triple"),
            "IR should contain target triple: {}",
            ir
        );
    }

    // 14. Runtime declarations
    #[test]
    fn test_runtime_declarations() {
        let ir = generate_ir("fn main() {}");
        assert!(
            ir.contains("declare"),
            "IR should contain runtime declarations: {}",
            ir
        );
        assert!(
            ir.contains("@axon_alloc"),
            "IR should contain axon_alloc: {}",
            ir
        );
        assert!(
            ir.contains("@axon_panic"),
            "IR should contain axon_panic: {}",
            ir
        );
    }

    // 15. Multiple functions
    #[test]
    fn test_multiple_functions() {
        let ir = generate_ir(
            "fn foo() -> Int64 { return 1; }\nfn bar() -> Int64 { return 2; }\nfn main() { }",
        );
        assert!(
            ir.contains("@foo") || ir.contains("@_AX"),
            "IR should contain foo function: {}",
            ir
        );
        assert!(
            ir.contains("@bar") || ir.contains("@_AX"),
            "IR should contain bar function: {}",
            ir
        );
        assert!(
            ir.contains("@main"),
            "IR should contain main function: {}",
            ir
        );
        // Count the number of define directives
        let define_count = ir.matches("define ").count();
        assert!(
            define_count >= 3,
            "IR should have at least 3 function definitions, got {}: {}",
            define_count,
            ir
        );
    }

    // 16. Type mapping: additional types
    #[test]
    fn test_llvm_type_int64() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::INT64), "i64");
    }

    #[test]
    fn test_llvm_type_float32() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::FLOAT32), "float");
    }

    #[test]
    fn test_llvm_type_char() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::CHAR), "i32");
    }

    #[test]
    fn test_llvm_type_never() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.llvm_type(TypeId::NEVER), "void");
    }

    #[test]
    fn test_fresh_name_increments() {
        let interner = TypeInterner::new();
        let mut cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.fresh_name(), "%0");
        assert_eq!(cg.fresh_name(), "%1");
        assert_eq!(cg.fresh_name(), "%2");
    }

    #[test]
    fn test_local_name_format() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.local_name(LocalId(0)), "%_0");
        assert_eq!(cg.local_name(LocalId(5)), "%_5");
    }

    #[test]
    fn test_block_label_format() {
        let interner = TypeInterner::new();
        let cg = LlvmCodegen::new(&interner, OptLevel::O0);
        assert_eq!(cg.block_label(BlockId(0)), "bb0");
        assert_eq!(cg.block_label(BlockId(3)), "bb3");
    }

    #[test]
    fn test_escape_llvm_string() {
        assert_eq!(LlvmCodegen::escape_llvm_string("hello"), "hello");
        assert_eq!(LlvmCodegen::escape_llvm_string("a\nb"), "a\\0Ab");
        assert_eq!(LlvmCodegen::escape_llvm_string("a\\b"), "a\\5Cb");
    }

    #[test]
    fn test_intern_string_deduplicates() {
        let interner = TypeInterner::new();
        let mut cg = LlvmCodegen::new(&interner, OptLevel::O0);
        let s1 = cg.intern_string("hello");
        let s2 = cg.intern_string("hello");
        assert_eq!(s1, s2, "Same string should return same global name");
        let s3 = cg.intern_string("world");
        assert_ne!(s1, s3, "Different strings should get different names");
    }

    // E5009: Missing main function validation
    #[test]
    fn test_generate_no_main_returns_error() {
        let source = "fn helper() -> Int64 { return 42; }";
        let (typed_program, _errors) = crate::check_source(source, "test.axon");
        let (checker, _) = crate::typeck::check(source, "test.axon");
        let mut builder = crate::mir::MirBuilder::new(&checker.interner);
        let mir = builder.build(&typed_program);
        let mut codegen = LlvmCodegen::new(&checker.interner, OptLevel::O0);
        let result = codegen.generate(&mir);
        assert!(result.is_err(), "generate() should return Err when no main function");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("E5009"), "Error should contain E5009 code: {}", err_msg);
        assert!(err_msg.contains("main"), "Error should mention 'main': {}", err_msg);
    }
}
