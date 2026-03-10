// stdlib/nn.rs — Neural network layers, activations, and initializers.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct, def_trait};

/// Register neural network types and functions.
pub fn register_nn(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_module_trait(interner, symbols);
    register_layer_structs(interner, symbols);
    register_activation_structs(interner, symbols);
    register_construction_fns(interner, symbols);
    register_init_fns(interner, symbols);
    register_layer_methods(interner, symbols);
    register_activation_methods(interner, symbols);
    register_sequential_methods(interner, symbols);
}

// -- Module trait --------------------------------------------------------------

fn register_module_trait(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let forward_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::INT64,
    });
    let parameters_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::INT64,
    });
    let train_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    let eval_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    let to_device_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::UNIT,
    });
    let save_ty = interner.intern(Type::Function {
        params: vec![TypeId::STRING],
        ret: TypeId::BOOL,
    });
    let load_ty = interner.intern(Type::Function {
        params: vec![TypeId::STRING],
        ret: TypeId::BOOL,
    });
    def_trait(
        symbols,
        interner,
        "Module",
        vec![
            ("forward".into(), forward_ty),
            ("parameters".into(), parameters_ty),
            ("train".into(), train_ty),
            ("eval".into(), eval_ty),
            ("to_device".into(), to_device_ty),
            ("save".into(), save_ty),
            ("load".into(), load_ty),
        ],
        vec![],
    );
}

// -- Layer structs ------------------------------------------------------------

fn register_layer_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(
        symbols, interner, "Linear",
        vec![("in_features".into(), TypeId::INT64), ("out_features".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "Conv2d",
        vec![
            ("in_channels".into(), TypeId::INT64),
            ("out_channels".into(), TypeId::INT64),
            ("kernel_size".into(), TypeId::INT64),
        ],
        vec![],
    );
    def_struct(symbols, interner, "BatchNorm", vec![("num_features".into(), TypeId::INT64)], vec![]);
    def_struct(symbols, interner, "LayerNorm", vec![("normalized_shape".into(), TypeId::INT64)], vec![]);
    def_struct(symbols, interner, "Dropout", vec![("p".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "MaxPool2d", vec![("kernel_size".into(), TypeId::INT64)], vec![]);
    def_struct(symbols, interner, "AvgPool2d", vec![("kernel_size".into(), TypeId::INT64)], vec![]);
    def_struct(symbols, interner, "AdaptiveAvgPool2d", vec![("output_size".into(), TypeId::INT64)], vec![]);
    def_struct(
        symbols, interner, "LSTM",
        vec![
            ("input_size".into(), TypeId::INT64),
            ("hidden_size".into(), TypeId::INT64),
            ("num_layers".into(), TypeId::INT64),
        ],
        vec![],
    );
    def_struct(
        symbols, interner, "GRU",
        vec![
            ("input_size".into(), TypeId::INT64),
            ("hidden_size".into(), TypeId::INT64),
            ("num_layers".into(), TypeId::INT64),
        ],
        vec![],
    );
    def_struct(
        symbols, interner, "MultiHeadAttention",
        vec![("embed_dim".into(), TypeId::INT64), ("num_heads".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "TransformerEncoderLayer",
        vec![("d_model".into(), TypeId::INT64), ("nhead".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "TransformerEncoder",
        vec![("num_layers".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "Embedding",
        vec![("num_embeddings".into(), TypeId::INT64), ("embedding_dim".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(symbols, interner, "Sequential", vec![], vec![]);
}

// -- Activation structs -------------------------------------------------------

fn register_activation_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(symbols, interner, "ReLU", vec![], vec![]);
    def_struct(symbols, interner, "GELU", vec![], vec![]);
    def_struct(symbols, interner, "SiLU", vec![], vec![]);
    def_struct(symbols, interner, "Sigmoid", vec![], vec![]);
    def_struct(symbols, interner, "Tanh", vec![], vec![]);
    def_struct(symbols, interner, "Softmax", vec![("dim".into(), TypeId::INT64)], vec![]);
    def_struct(symbols, interner, "LogSoftmax", vec![("dim".into(), TypeId::INT64)], vec![]);
}

// -- Construction functions ---------------------------------------------------

fn register_construction_fns(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "linear_new", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(
        symbols, interner, "conv2d_new",
        vec![TypeId::INT64, TypeId::INT64, TypeId::INT64],
        TypeId::INT64,
    );
    def_fn(symbols, interner, "batchnorm_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "layernorm_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "dropout_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "maxpool2d_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "avgpool2d_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "adaptive_avgpool2d_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(
        symbols, interner, "lstm_new",
        vec![TypeId::INT64, TypeId::INT64, TypeId::INT64],
        TypeId::INT64,
    );
    def_fn(
        symbols, interner, "gru_new",
        vec![TypeId::INT64, TypeId::INT64, TypeId::INT64],
        TypeId::INT64,
    );
    def_fn(symbols, interner, "multihead_attention_new", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "transformer_encoder_layer_new", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "transformer_encoder_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "embedding_new", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "sequential_new", vec![], TypeId::INT64);
}

// -- Weight initializers ------------------------------------------------------

fn register_init_fns(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "xavier_uniform", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "xavier_normal", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "kaiming_uniform", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "kaiming_normal", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "init_uniform", vec![TypeId::INT64, TypeId::FLOAT64, TypeId::FLOAT64], TypeId::UNIT);
    def_fn(symbols, interner, "init_normal", vec![TypeId::INT64, TypeId::FLOAT64, TypeId::FLOAT64], TypeId::UNIT);
    def_fn(symbols, interner, "init_zeros", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "init_ones", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "init_constant", vec![TypeId::INT64, TypeId::FLOAT64], TypeId::UNIT);
}

// -- Layer methods (Module interface) -----------------------------------------
// Register forward(self, input) -> Tensor and parameters(self) -> Vec<Tensor>
// on every layer struct, plus train(self), eval(self), to_device(self, device).
// Tensor proxy type is INT64 (consistent with stdlib conventions).

fn register_module_methods_for(interner: &mut TypeInterner, symbols: &mut SymbolTable, type_name: &str) {
    // forward(self, input: Tensor) -> Tensor
    def_method(symbols, interner, type_name, "forward",
        vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    // parameters(self) -> Vec<Tensor>  (Vec<Tensor> approximated as INT64)
    def_method(symbols, interner, type_name, "parameters",
        vec![TypeId::INT64], TypeId::INT64);
    // train(self) -> Unit
    def_method(symbols, interner, type_name, "train",
        vec![TypeId::INT64], TypeId::UNIT);
    // eval(self) -> Unit
    def_method(symbols, interner, type_name, "eval",
        vec![TypeId::INT64], TypeId::UNIT);
    // to_device(self, device: Int64) -> Unit
    def_method(symbols, interner, type_name, "to_device",
        vec![TypeId::INT64, TypeId::INT64], TypeId::UNIT);
}

fn register_layer_methods(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let layer_types = [
        "Linear", "Conv2d", "BatchNorm", "LayerNorm", "Dropout",
        "MaxPool2d", "AvgPool2d", "AdaptiveAvgPool2d",
        "LSTM", "GRU", "MultiHeadAttention",
        "TransformerEncoderLayer", "TransformerEncoder",
        "Embedding",
    ];
    for type_name in &layer_types {
        register_module_methods_for(interner, symbols, type_name);
    }
}

fn register_activation_methods(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let activation_types = [
        "ReLU", "GELU", "SiLU", "Sigmoid", "Tanh", "Softmax", "LogSoftmax",
    ];
    for type_name in &activation_types {
        register_module_methods_for(interner, symbols, type_name);
    }
}

// -- Sequential methods -------------------------------------------------------

fn register_sequential_methods(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Sequential gets all Module methods
    register_module_methods_for(interner, symbols, "Sequential");

    // Sequential::add(self, layer: Module) -> Sequential (builder pattern)
    // layer is represented as INT64 (Tensor proxy), returns INT64 (Sequential proxy)
    def_method(symbols, interner, "Sequential", "add",
        vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);

    // Sequential::len(self) -> Int64 — number of layers
    def_method(symbols, interner, "Sequential", "len",
        vec![TypeId::INT64], TypeId::INT64);
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
    fn module_trait_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        assert!(s.lookup("Module").is_some());
    }

    #[test]
    fn layer_structs_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for name in &[
            "Linear", "Conv2d", "BatchNorm", "LayerNorm", "Dropout",
            "MaxPool2d", "AvgPool2d", "AdaptiveAvgPool2d",
            "LSTM", "GRU", "MultiHeadAttention",
            "TransformerEncoderLayer", "TransformerEncoder",
            "Embedding", "Sequential",
        ] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn activation_structs_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for name in &["ReLU", "GELU", "SiLU", "Sigmoid", "Tanh", "Softmax", "LogSoftmax"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn construction_fns_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for name in &["linear_new", "conv2d_new", "lstm_new", "embedding_new", "sequential_new"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn init_fns_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for name in &[
            "xavier_uniform", "xavier_normal", "kaiming_uniform", "kaiming_normal",
            "init_zeros", "init_ones", "init_constant",
        ] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn layer_forward_methods_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for type_name in &[
            "Linear", "Conv2d", "BatchNorm", "LayerNorm", "Dropout",
            "LSTM", "MultiHeadAttention", "Embedding", "Sequential",
        ] {
            let method = format!("{}::forward", type_name);
            assert!(s.lookup(&method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn layer_parameters_methods_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for type_name in &[
            "Linear", "Conv2d", "BatchNorm", "Embedding", "Sequential",
        ] {
            let method = format!("{}::parameters", type_name);
            assert!(s.lookup(&method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn activation_forward_methods_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for type_name in &["ReLU", "Sigmoid", "Tanh", "Softmax", "GELU", "SiLU", "LogSoftmax"] {
            let method = format!("{}::forward", type_name);
            assert!(s.lookup(&method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn sequential_add_method_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        assert!(s.lookup("Sequential::add").is_some(), "Sequential::add should be registered");
        assert!(s.lookup("Sequential::len").is_some(), "Sequential::len should be registered");
    }

    #[test]
    fn layer_train_eval_methods_registered() {
        let (mut i, mut s) = fresh();
        register_nn(&mut i, &mut s);
        for type_name in &["Linear", "Sequential"] {
            let train_m = format!("{}::train", type_name);
            let eval_m = format!("{}::eval", type_name);
            let device_m = format!("{}::to_device", type_name);
            assert!(s.lookup(&train_m).is_some(), "{} should be registered", train_m);
            assert!(s.lookup(&eval_m).is_some(), "{} should be registered", eval_m);
            assert!(s.lookup(&device_m).is_some(), "{} should be registered", device_m);
        }
    }
}
