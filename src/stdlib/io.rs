// stdlib/io.rs — I/O traits, File, and standard streams.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct, def_trait};

/// Register I/O traits, types, and functions.
pub fn register_io(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_traits(interner, symbols);
    register_file(interner, symbols);
    register_streams(interner, symbols);
    register_formatting(interner, symbols);
}

// -- Read / Write traits ------------------------------------------------------

fn register_traits(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Read trait: fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>
    let read_method = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::INT64,
    });
    let read_to_string = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::STRING,
    });
    def_trait(
        symbols,
        interner,
        "Read",
        vec![
            ("read".into(), read_method),
            ("read_to_string".into(), read_to_string),
        ],
        vec![],
    );

    // Write trait: fn write(&mut self, buf: &[u8]) -> Result<usize, Error>
    let write_method = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::INT64,
    });
    let flush_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    let write_str = interner.intern(Type::Function {
        params: vec![TypeId::STRING],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "Write",
        vec![
            ("write".into(), write_method),
            ("flush".into(), flush_method),
            ("write_str".into(), write_str),
        ],
        vec![],
    );

    // BufRead trait
    let read_line = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::STRING,
    });
    let lines_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "BufRead",
        vec![
            ("read_line".into(), read_line),
            ("lines".into(), lines_method),
        ],
        vec![],
    );
}

// -- File ---------------------------------------------------------------------

fn register_file(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let file_ty = def_struct(symbols, interner, "File", vec![], vec![]);

    // File::open(path: String) -> Result<File, String>
    let result_ty = interner.intern(Type::Result(file_ty, TypeId::STRING));
    def_method(symbols, interner, "File", "open", vec![TypeId::STRING], result_ty);
    def_method(symbols, interner, "File", "create", vec![TypeId::STRING], result_ty);
    def_method(symbols, interner, "File", "read_to_string", vec![file_ty], TypeId::STRING);
    def_method(symbols, interner, "File", "write_all", vec![file_ty, TypeId::STRING], TypeId::UNIT);
    def_method(symbols, interner, "File", "flush", vec![file_ty], TypeId::UNIT);
    def_method(symbols, interner, "File", "close", vec![file_ty], TypeId::UNIT);
}

// -- Standard streams ---------------------------------------------------------

fn register_streams(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let stdin_ty = def_struct(symbols, interner, "Stdin", vec![], vec![]);
    let stdout_ty = def_struct(symbols, interner, "Stdout", vec![], vec![]);
    let stderr_ty = def_struct(symbols, interner, "Stderr", vec![], vec![]);

    def_fn(symbols, interner, "stdin", vec![], stdin_ty);
    def_fn(symbols, interner, "stdout", vec![], stdout_ty);
    def_fn(symbols, interner, "stderr", vec![], stderr_ty);

    // Stdin methods
    def_method(symbols, interner, "Stdin", "read_line", vec![stdin_ty], TypeId::STRING);
    def_method(symbols, interner, "Stdin", "lines", vec![stdin_ty], TypeId::UNIT);

    // Stdout/Stderr methods
    def_method(symbols, interner, "Stdout", "write", vec![stdout_ty, TypeId::STRING], TypeId::UNIT);
    def_method(symbols, interner, "Stdout", "flush", vec![stdout_ty], TypeId::UNIT);
    def_method(symbols, interner, "Stderr", "write", vec![stderr_ty, TypeId::STRING], TypeId::UNIT);
    def_method(symbols, interner, "Stderr", "flush", vec![stderr_ty], TypeId::UNIT);
}

// -- Formatting helpers -------------------------------------------------------

fn register_formatting(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "format", vec![TypeId::STRING], TypeId::STRING);
    def_fn(symbols, interner, "to_string", vec![TypeId::UNIT], TypeId::STRING);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymbolTable;

    fn fresh() -> (TypeInterner, SymbolTable) {
        (TypeInterner::new(), SymbolTable::new())
    }

    #[test]
    fn read_write_traits_registered() {
        let (mut i, mut s) = fresh();
        register_io(&mut i, &mut s);
        assert!(s.lookup("Read").is_some());
        assert!(s.lookup("Write").is_some());
        assert!(s.lookup("BufRead").is_some());
    }

    #[test]
    fn file_type_registered() {
        let (mut i, mut s) = fresh();
        register_io(&mut i, &mut s);
        assert!(s.lookup("File").is_some());
        assert!(s.lookup("File::open").is_some());
        assert!(s.lookup("File::create").is_some());
    }

    #[test]
    fn standard_streams_registered() {
        let (mut i, mut s) = fresh();
        register_io(&mut i, &mut s);
        assert!(s.lookup("stdin").is_some());
        assert!(s.lookup("stdout").is_some());
        assert!(s.lookup("stderr").is_some());
    }

    #[test]
    fn format_registered() {
        let (mut i, mut s) = fresh();
        register_io(&mut i, &mut s);
        assert!(s.lookup("format").is_some());
    }

    #[test]
    fn file_open_returns_result() {
        let (mut i, mut s) = fresh();
        register_io(&mut i, &mut s);
        let sym_id = s.lookup("File::open").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { ret, .. } => {
                assert!(matches!(i.resolve(*ret), Type::Result(_, _)));
            }
            _ => panic!("File::open should be a function"),
        }
    }
}
