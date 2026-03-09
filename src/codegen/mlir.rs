//! MLIR Backend for GPU Code Generation (Phase 4c)
//!
//! This module will lower @gpu-annotated functions to MLIR dialects
//! for compilation to CUDA/ROCm/Vulkan targets.
//!
//! Architecture:
//! - Axon TAST → Linalg dialect (tensor operations)
//! - Linalg → MemRef dialect (bufferization)
//! - MemRef → GPU dialect → NVVM/ROCDL/SPIR-V
//!
//! Status: Stub — requires MLIR installation for full implementation.

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

/// Placeholder for MLIR codegen.
/// Returns an error explaining MLIR is not yet available.
pub fn compile_gpu(_mir: &crate::mir::MirProgram, target: &GpuTarget) -> Result<Vec<u8>, String> {
    Err(format!(
        "MLIR backend for {} is not yet implemented. \
         GPU code generation requires MLIR installation. \
         Use --gpu=none or omit @gpu annotations for CPU-only compilation.",
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
        assert!(result.unwrap_err().contains("not yet implemented"));
    }

    #[test]
    fn has_gpu_functions_empty() {
        let mir = crate::mir::MirProgram { functions: vec![], statics: vec![] };
        assert!(!has_gpu_functions(&mir));
    }
}
