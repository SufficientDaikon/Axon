//! MLIR Backend for GPU Code Generation
//!
//! **Status: Deferred to Phase 12 (GPU Backend)**
//!
//! This module provides the GPU compilation interface and target abstractions.
//! The actual MLIR-based code generation is intentionally deferred:
//!
//! - Phase 12 will implement full MLIR lowering for @gpu-annotated functions
//! - The pipeline will be: Axon TAST → Linalg dialect → MemRef → GPU dialect → NVVM/ROCDL/SPIR-V
//! - This requires an MLIR installation and is not needed for CPU-only compilation
//!
//! For now, the `compile_gpu` function returns a descriptive error directing users
//! to use `--gpu none` for CPU-only compilation. This is by design, not a bug.

use serde::Serialize;

/// GPU target for MLIR compilation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum GpuTarget {
    Cuda,
    Rocm,
    Vulkan,
    None,
}

impl GpuTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cuda" => Some(GpuTarget::Cuda),
            "rocm" => Some(GpuTarget::Rocm),
            "vulkan" => Some(GpuTarget::Vulkan),
            "none" => Some(GpuTarget::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GpuTarget::Cuda => "cuda",
            GpuTarget::Rocm => "rocm",
            GpuTarget::Vulkan => "vulkan",
            GpuTarget::None => "none",
        }
    }
}

/// Placeholder for MLIR codegen (deferred to Phase 12).
/// Returns an error explaining GPU compilation is not yet available.
pub fn compile_gpu(_mir: &crate::mir::MirProgram, target: &GpuTarget) -> Result<Vec<u8>, String> {
    Err(format!(
        "GPU compilation via MLIR is planned for Phase 12. \
         Target '{}' is not yet supported. \
         Use --gpu none for CPU-only compilation.",
        target.as_str()
    ))
}

/// Check if any functions in the MIR program have @gpu annotations.
pub fn has_gpu_functions(mir: &crate::mir::MirProgram) -> bool {
    mir.functions.iter().any(|f| {
        f.attributes.iter().any(|a| matches!(a, crate::ast::Attribute::Gpu))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_target_from_str() {
        assert_eq!(GpuTarget::from_str("cuda"), Some(GpuTarget::Cuda));
        assert_eq!(GpuTarget::from_str("ROCM"), Some(GpuTarget::Rocm));
        assert_eq!(GpuTarget::from_str("vulkan"), Some(GpuTarget::Vulkan));
        assert_eq!(GpuTarget::from_str("none"), Some(GpuTarget::None));
        assert_eq!(GpuTarget::from_str("tpu"), None);
    }

    #[test]
    fn compile_gpu_returns_error() {
        let mir = crate::mir::MirProgram { functions: vec![], statics: vec![] };
        let result = compile_gpu(&mir, &GpuTarget::Cuda);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("planned for Phase 12"));
    }

    #[test]
    fn has_gpu_functions_empty() {
        let mir = crate::mir::MirProgram { functions: vec![], statics: vec![] };
        assert!(!has_gpu_functions(&mir));
    }
}
