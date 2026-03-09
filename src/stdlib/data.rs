// stdlib/data.rs — CSV/JSON loaders, DataLoader, Dataset, DataFrame.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct};

/// Register data loading types and functions.
pub fn register_data(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_loaders(interner, symbols);
    register_dataset(interner, symbols);
    register_dataframe(interner, symbols);
}

fn register_loaders(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let dataloader_ty = def_struct(symbols, interner, "DataLoader", vec![], vec![]);

    // Free functions
    def_fn(symbols, interner, "read_csv", vec![TypeId::STRING], TypeId::UNIT);
    def_fn(symbols, interner, "read_json", vec![TypeId::STRING], TypeId::UNIT);
    def_fn(symbols, interner, "parse_json", vec![TypeId::STRING], TypeId::UNIT);
    def_fn(symbols, interner, "write_csv", vec![TypeId::STRING, TypeId::UNIT], TypeId::UNIT);
    def_fn(symbols, interner, "write_json", vec![TypeId::STRING, TypeId::UNIT], TypeId::UNIT);

    // DataLoader methods
    def_method(symbols, interner, "DataLoader", "new", vec![TypeId::UNIT, TypeId::INT64], dataloader_ty);
    def_method(symbols, interner, "DataLoader", "batch_size", vec![dataloader_ty], TypeId::INT64);
    def_method(symbols, interner, "DataLoader", "shuffle", vec![dataloader_ty, TypeId::BOOL], dataloader_ty);
    def_method(symbols, interner, "DataLoader", "iter", vec![dataloader_ty], TypeId::UNIT);
    def_method(symbols, interner, "DataLoader", "len", vec![dataloader_ty], TypeId::INT64);
}

fn register_dataset(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let dataset_ty = def_struct(symbols, interner, "Dataset", vec![], vec![]);

    def_method(symbols, interner, "Dataset", "new", vec![TypeId::UNIT], dataset_ty);
    def_method(symbols, interner, "Dataset", "len", vec![dataset_ty], TypeId::INT64);
    def_method(symbols, interner, "Dataset", "get", vec![dataset_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Dataset", "split", vec![dataset_ty, TypeId::FLOAT64], TypeId::UNIT);
    def_method(symbols, interner, "Dataset", "shuffle", vec![dataset_ty], dataset_ty);
    def_method(symbols, interner, "Dataset", "map", vec![dataset_ty, TypeId::UNIT], dataset_ty);
}

fn register_dataframe(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let df_ty = def_struct(symbols, interner, "DataFrame", vec![], vec![]);

    def_method(symbols, interner, "DataFrame", "new", vec![], df_ty);
    def_method(symbols, interner, "DataFrame", "from_csv", vec![TypeId::STRING], df_ty);
    def_method(symbols, interner, "DataFrame", "shape", vec![df_ty], TypeId::UNIT);
    def_method(symbols, interner, "DataFrame", "columns", vec![df_ty], TypeId::UNIT);
    def_method(symbols, interner, "DataFrame", "head", vec![df_ty, TypeId::INT64], df_ty);
    def_method(symbols, interner, "DataFrame", "tail", vec![df_ty, TypeId::INT64], df_ty);
    def_method(symbols, interner, "DataFrame", "select", vec![df_ty, TypeId::UNIT], df_ty);
    def_method(symbols, interner, "DataFrame", "filter", vec![df_ty, TypeId::UNIT], df_ty);
    def_method(symbols, interner, "DataFrame", "sort_by", vec![df_ty, TypeId::STRING], df_ty);
    def_method(symbols, interner, "DataFrame", "group_by", vec![df_ty, TypeId::STRING], df_ty);
    def_method(symbols, interner, "DataFrame", "describe", vec![df_ty], df_ty);
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
    fn read_csv_registered() {
        let (mut i, mut s) = fresh();
        register_data(&mut i, &mut s);
        assert!(s.lookup("read_csv").is_some());
    }

    #[test]
    fn read_json_registered() {
        let (mut i, mut s) = fresh();
        register_data(&mut i, &mut s);
        assert!(s.lookup("read_json").is_some());
        assert!(s.lookup("parse_json").is_some());
    }

    #[test]
    fn dataloader_registered() {
        let (mut i, mut s) = fresh();
        register_data(&mut i, &mut s);
        assert!(s.lookup("DataLoader").is_some());
        assert!(s.lookup("DataLoader::new").is_some());
    }

    #[test]
    fn dataset_registered() {
        let (mut i, mut s) = fresh();
        register_data(&mut i, &mut s);
        assert!(s.lookup("Dataset").is_some());
        assert!(s.lookup("Dataset::len").is_some());
    }

    #[test]
    fn dataframe_registered() {
        let (mut i, mut s) = fresh();
        register_data(&mut i, &mut s);
        assert!(s.lookup("DataFrame").is_some());
        assert!(s.lookup("DataFrame::from_csv").is_some());
        assert!(s.lookup("DataFrame::describe").is_some());
    }
}
