// tests/ai_framework_tests.rs — Integration tests for Axon AI framework (Phase 6).
//
// Verifies that all AI framework functions registered in stdlib modules
// (nn, autograd, optim, loss, train, metrics, transforms, export)
// are correctly resolved and type-checked through `typeck::check`.

use axonc::typeck;
use axonc::error::CompileError;

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn check_ok(source: &str) {
    let (_, errors) = typeck::check(source, "test.axon");
    assert!(
        errors.is_empty(),
        "Expected no errors, got: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

fn check_err(source: &str) -> Vec<CompileError> {
    let (_, errors) = typeck::check(source, "test.axon");
    assert!(!errors.is_empty(), "Expected errors but got none");
    errors
}

fn check_has_error_code(source: &str, code: &str) -> Vec<CompileError> {
    let (_, errors) = typeck::check(source, "test.axon");
    assert!(
        errors.iter().any(|e| e.error_code == code),
        "Expected error code {}, got: {:?}",
        code,
        errors.iter().map(|e| format!("{}: {}", e.error_code, e.message)).collect::<Vec<_>>()
    );
    errors
}

// ═══════════════════════════════════════════════════════════════
// 1. NN layer construction
// ═══════════════════════════════════════════════════════════════

#[test]
fn nn_linear_new() {
    check_ok("fn main() { linear_new(784, 256); }");
}

#[test]
fn nn_conv2d_new() {
    check_ok("fn main() { conv2d_new(3, 64, 3); }");
}

#[test]
fn nn_batchnorm_new() {
    check_ok("fn main() { batchnorm_new(64); }");
}

#[test]
fn nn_layernorm_new() {
    check_ok("fn main() { layernorm_new(512); }");
}

#[test]
fn nn_dropout_new() {
    check_ok("fn main() { let p: Float32; dropout_new(p); }");
}

#[test]
fn nn_maxpool2d_new() {
    check_ok("fn main() { maxpool2d_new(2); }");
}

#[test]
fn nn_avgpool2d_new() {
    check_ok("fn main() { avgpool2d_new(2); }");
}

#[test]
fn nn_adaptive_avgpool2d_new() {
    check_ok("fn main() { adaptive_avgpool2d_new(1); }");
}

#[test]
fn nn_lstm_new() {
    check_ok("fn main() { lstm_new(128, 256, 2); }");
}

#[test]
fn nn_gru_new() {
    check_ok("fn main() { gru_new(128, 256, 2); }");
}

#[test]
fn nn_multihead_attention_new() {
    check_ok("fn main() { multihead_attention_new(512, 8); }");
}

#[test]
fn nn_transformer_encoder_layer_new() {
    check_ok("fn main() { transformer_encoder_layer_new(512, 8); }");
}

#[test]
fn nn_transformer_encoder_new() {
    check_ok("fn main() { transformer_encoder_new(6); }");
}

#[test]
fn nn_embedding_new() {
    check_ok("fn main() { embedding_new(10000, 512); }");
}

#[test]
fn nn_sequential_new() {
    check_ok("fn main() { sequential_new(); }");
}

// ═══════════════════════════════════════════════════════════════
// 2. Activation construction (ReLU, GELU, SiLU, Softmax exist
//    as structs; test sequential_new as container)
// ═══════════════════════════════════════════════════════════════

#[test]
fn nn_linear_new_returns_int64() {
    check_ok("fn main() -> Int64 { return linear_new(10, 10); }");
}

#[test]
fn nn_sequential_new_returns_int64() {
    check_ok("fn main() -> Int64 { return sequential_new(); }");
}

// ═══════════════════════════════════════════════════════════════
// 3. Autograd functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn autograd_grad_tensor_new() {
    check_ok("fn main() { grad_tensor_new(true); }");
}

#[test]
fn autograd_backward() {
    check_ok("fn main() { backward(); }");
}

#[test]
fn autograd_zero_grad() {
    check_ok("fn main() { zero_grad(); }");
}

#[test]
fn autograd_detach() {
    check_ok("fn main() { detach(1); }");
}

#[test]
fn autograd_no_grad() {
    check_ok("fn main() { no_grad(); }");
}

#[test]
fn autograd_grad_checkpoint() {
    check_ok("fn main() { grad_checkpoint(1); }");
}

#[test]
fn autograd_grad_tensor_new_returns_int64() {
    check_ok("fn main() -> Int64 { return grad_tensor_new(false); }");
}

#[test]
fn autograd_detach_returns_int64() {
    check_ok("fn main() -> Int64 { return detach(42); }");
}

// ═══════════════════════════════════════════════════════════════
// 4. Optimizer construction
// ═══════════════════════════════════════════════════════════════

#[test]
fn optim_sgd_new() {
    check_ok("fn main() { let lr: Float32; sgd_new(lr); }");
}

#[test]
fn optim_adam_new() {
    check_ok("fn main() { let lr: Float32; adam_new(lr); }");
}

#[test]
fn optim_adamw_new() {
    check_ok("fn main() { let lr: Float32; adamw_new(lr); }");
}

#[test]
fn optim_rmsprop_new() {
    check_ok("fn main() { let lr: Float32; rmsprop_new(lr); }");
}

#[test]
fn optim_adagrad_new() {
    check_ok("fn main() { let lr: Float32; adagrad_new(lr); }");
}

#[test]
fn optim_step_lr_new() {
    check_ok("fn main() { let g: Float32; step_lr_new(10, g); }");
}

#[test]
fn optim_cosine_annealing_lr_new() {
    check_ok("fn main() { cosine_annealing_lr_new(100); }");
}

#[test]
fn optim_reduce_lr_on_plateau_new() {
    check_ok("fn main() { reduce_lr_on_plateau_new(5); }");
}

#[test]
fn optim_one_cycle_lr_new() {
    check_ok("fn main() { let lr: Float32; one_cycle_lr_new(lr, 1000); }");
}

// ═══════════════════════════════════════════════════════════════
// 5. Loss function construction
// ═══════════════════════════════════════════════════════════════

#[test]
fn loss_mse_loss() {
    check_ok("fn main() { mse_loss(); }");
}

#[test]
fn loss_l1_loss() {
    check_ok("fn main() { l1_loss(); }");
}

#[test]
fn loss_cross_entropy_loss() {
    check_ok("fn main() { cross_entropy_loss(); }");
}

#[test]
fn loss_bce_loss() {
    check_ok("fn main() { bce_loss(); }");
}

#[test]
fn loss_bce_with_logits_loss() {
    check_ok("fn main() { bce_with_logits_loss(); }");
}

#[test]
fn loss_nll_loss() {
    check_ok("fn main() { nll_loss(); }");
}

#[test]
fn loss_huber_loss() {
    check_ok("fn main() { let d: Float32; huber_loss(d); }");
}

#[test]
fn loss_kl_div_loss() {
    check_ok("fn main() { kl_div_loss(); }");
}

#[test]
fn loss_cosine_embedding_loss() {
    check_ok("fn main() { cosine_embedding_loss(); }");
}

#[test]
fn loss_triplet_margin_loss() {
    check_ok("fn main() { let m: Float32; triplet_margin_loss(m); }");
}

#[test]
fn loss_ctc_loss() {
    check_ok("fn main() { ctc_loss(); }");
}

// ═══════════════════════════════════════════════════════════════
// 6. Training utilities
// ═══════════════════════════════════════════════════════════════

#[test]
fn train_trainer_new() {
    check_ok("fn main() { trainer_new(); }");
}

#[test]
fn train_checkpoint_save() {
    check_ok(r#"fn main() { checkpoint_save("model.ckpt", 10); }"#);
}

#[test]
fn train_checkpoint_load() {
    check_ok(r#"fn main() { checkpoint_load("model.ckpt"); }"#);
}

#[test]
fn train_grad_scaler_new() {
    check_ok("fn main() { let s: Float32; grad_scaler_new(s); }");
}

#[test]
fn train_autocast() {
    check_ok("fn main() { autocast(); }");
}

// ═══════════════════════════════════════════════════════════════
// 7. Metrics
// ═══════════════════════════════════════════════════════════════

#[test]
fn metrics_accuracy() {
    check_ok("fn main() { accuracy(1, 1); }");
}

#[test]
fn metrics_precision() {
    check_ok("fn main() { precision(1, 1); }");
}

#[test]
fn metrics_recall() {
    check_ok("fn main() { recall(1, 1); }");
}

#[test]
fn metrics_f1_score() {
    check_ok("fn main() { f1_score(1, 1); }");
}

#[test]
fn metrics_confusion_matrix() {
    check_ok("fn main() { confusion_matrix(1, 1, 10); }");
}

#[test]
fn metrics_roc_auc() {
    check_ok("fn main() { roc_auc(1, 1); }");
}

#[test]
fn metrics_mean_squared_error() {
    check_ok("fn main() { mean_squared_error(1, 1); }");
}

#[test]
fn metrics_mean_absolute_error() {
    check_ok("fn main() { mean_absolute_error(1, 1); }");
}

#[test]
fn metrics_r2_score() {
    check_ok("fn main() { r2_score(1, 1); }");
}

// ═══════════════════════════════════════════════════════════════
// 8. Transforms
// ═══════════════════════════════════════════════════════════════

#[test]
fn transforms_resize() {
    check_ok("fn main() { resize(224, 224); }");
}

#[test]
fn transforms_center_crop() {
    check_ok("fn main() { center_crop(224, 224); }");
}

#[test]
fn transforms_random_crop() {
    check_ok("fn main() { random_crop(224, 224); }");
}

#[test]
fn transforms_random_horizontal_flip() {
    check_ok("fn main() { let p: Float32; random_horizontal_flip(p); }");
}

#[test]
fn transforms_normalize_transform() {
    check_ok("fn main() { normalize_transform(0.5, 0.5); }");
}

#[test]
fn transforms_to_tensor_transform() {
    check_ok("fn main() { to_tensor_transform(); }");
}

#[test]
fn transforms_tokenize() {
    check_ok(r#"fn main() { tokenize("hello world"); }"#);
}

#[test]
fn transforms_pad_sequence() {
    check_ok("fn main() { pad_sequence(128); }");
}

// ═══════════════════════════════════════════════════════════════
// 9. Export
// ═══════════════════════════════════════════════════════════════

#[test]
fn export_onnx_export() {
    check_ok(r#"fn main() { let v: Int32; onnx_export("model.onnx", v); }"#);
}

#[test]
fn export_save_model() {
    check_ok(r#"fn main() { save_model("model.axon"); }"#);
}

#[test]
fn export_load_model() {
    check_ok(r#"fn main() { load_model("model.axon"); }"#);
}

#[test]
fn export_save_tensor() {
    check_ok(r#"fn main() { save_tensor("tensor.bin"); }"#);
}

#[test]
fn export_load_tensor() {
    check_ok(r#"fn main() { load_tensor("tensor.bin"); }"#);
}

// ═══════════════════════════════════════════════════════════════
// 10. Weight initialization
// ═══════════════════════════════════════════════════════════════

#[test]
fn init_xavier_uniform() {
    check_ok("fn main() { xavier_uniform(1); }");
}

#[test]
fn init_xavier_normal() {
    check_ok("fn main() { xavier_normal(1); }");
}

#[test]
fn init_kaiming_uniform() {
    check_ok("fn main() { kaiming_uniform(1); }");
}

#[test]
fn init_kaiming_normal() {
    check_ok("fn main() { kaiming_normal(1); }");
}

#[test]
fn init_uniform() {
    check_ok("fn main() { init_uniform(1, 0.0, 1.0); }");
}

#[test]
fn init_normal() {
    check_ok("fn main() { init_normal(1, 0.0, 1.0); }");
}

#[test]
fn init_zeros() {
    check_ok("fn main() { init_zeros(1); }");
}

#[test]
fn init_ones() {
    check_ok("fn main() { init_ones(1); }");
}

#[test]
fn init_constant() {
    check_ok("fn main() { init_constant(1, 0.5); }");
}

// ═══════════════════════════════════════════════════════════════
// 11. Return type verification
// ═══════════════════════════════════════════════════════════════

#[test]
fn metrics_accuracy_returns_float32() {
    // accuracy returns Float32, not Int64
    check_err("fn main() -> Int64 { return accuracy(1, 1); }");
}

#[test]
fn export_save_model_returns_bool() {
    check_ok(r#"fn main() -> Bool { return save_model("m.axon"); }"#);
}

#[test]
fn export_load_model_returns_int64() {
    check_ok(r#"fn main() -> Int64 { return load_model("m.axon"); }"#);
}

#[test]
fn export_onnx_export_returns_bool() {
    check_ok(r#"fn main() -> Bool { let v: Int32; return onnx_export("m.onnx", v); }"#);
}

// ═══════════════════════════════════════════════════════════════
// 12. Negative tests — wrong argument types
// ═══════════════════════════════════════════════════════════════

#[test]
fn neg_linear_new_wrong_type() {
    // linear_new expects (Int64, Int64), not (String, Int64)
    check_err(r#"fn main() { linear_new("ten", 10); }"#);
}

#[test]
fn neg_grad_tensor_new_wrong_type() {
    // grad_tensor_new expects Bool, not Int64
    check_err("fn main() { grad_tensor_new(42); }");
}

#[test]
fn neg_checkpoint_save_wrong_type() {
    // checkpoint_save expects (String, Int64), not (Int64, Int64)
    check_err("fn main() { checkpoint_save(1, 2); }");
}

#[test]
fn neg_tokenize_wrong_type() {
    // tokenize expects String, not Int64
    check_err("fn main() { tokenize(42); }");
}

#[test]
fn neg_accuracy_wrong_type() {
    // accuracy expects (Int64, Int64), not (Bool, Bool)
    check_err("fn main() { accuracy(true, false); }");
}

// ═══════════════════════════════════════════════════════════════
// 13. Negative tests — wrong arity
// ═══════════════════════════════════════════════════════════════

#[test]
fn neg_linear_new_wrong_arity() {
    // linear_new expects 2 args, got 1
    check_has_error_code("fn main() { linear_new(10); }", "E2003");
}

#[test]
fn neg_conv2d_new_wrong_arity() {
    // conv2d_new expects 3 args, got 2
    check_has_error_code("fn main() { conv2d_new(3, 64); }", "E2003");
}

#[test]
fn neg_backward_wrong_arity() {
    // backward expects 0 args, got 1
    check_has_error_code("fn main() { backward(1); }", "E2003");
}

#[test]
fn neg_mse_loss_wrong_arity() {
    // mse_loss expects 0 args, got 1
    check_has_error_code("fn main() { mse_loss(1); }", "E2003");
}

#[test]
fn neg_trainer_new_wrong_arity() {
    // trainer_new expects 0 args, got 1
    check_has_error_code("fn main() { trainer_new(1); }", "E2003");
}

#[test]
fn neg_confusion_matrix_wrong_arity() {
    // confusion_matrix expects 3 args, got 2
    check_has_error_code("fn main() { confusion_matrix(1, 1); }", "E2003");
}

#[test]
fn neg_embedding_new_wrong_arity() {
    // embedding_new expects 2 args, got 1
    check_has_error_code("fn main() { embedding_new(10000); }", "E2003");
}

#[test]
fn neg_sequential_new_wrong_arity() {
    // sequential_new expects 0 args, got 1
    check_has_error_code("fn main() { sequential_new(1); }", "E2003");
}
