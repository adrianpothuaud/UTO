//! Inference abstractions for Phase 3 vision detection.

use crate::error::{UtoError, UtoResult};

/// Raw model output before post-processing.
#[derive(Debug, Clone)]
pub struct RawDetection {
    /// Bounding box in (x, y, width, height), using pixel-space float values.
    pub bbox: (f32, f32, f32, f32),
    /// Confidence score from model output.
    pub confidence: f32,
    /// Predicted element type, e.g. button/input/link.
    pub element_type: String,
    /// Optional label extracted by model or OCR pipeline.
    pub label: Option<String>,
}

/// Inference engine contract for vision model execution.
pub trait InferenceEngine {
    /// Engine/model identifier used in metadata and diagnostics.
    fn model_name(&self) -> &'static str;

    /// Runs inference on a normalized RGB tensor-like input.
    fn infer(
        &self,
        normalized_pixels: &[f32],
        original_image_size: (u32, u32),
    ) -> UtoResult<Vec<RawDetection>>;
}

/// Placeholder ONNX adapter.
///
/// This keeps the Phase 3 preprocessing/post-processing path wired while ONNX
/// runtime integration is introduced incrementally.
pub struct StubOnnxEngine;

impl StubOnnxEngine {
    /// Creates a new stub ONNX inference adapter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubOnnxEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine for StubOnnxEngine {
    fn model_name(&self) -> &'static str {
        "onnx-stub"
    }

    fn infer(
        &self,
        _normalized_pixels: &[f32],
        _original_image_size: (u32, u32),
    ) -> UtoResult<Vec<RawDetection>> {
        Err(UtoError::Internal(
            "ONNX inference adapter not configured yet. Use detect_ui_elements_with_engine() with a concrete engine in tests or wire the runtime in Phase 3.1.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_engine_returns_clear_error() {
        let engine = StubOnnxEngine::new();
        let err = engine
            .infer(&[], (1, 1))
            .expect_err("stub engine should signal unconfigured runtime");

        assert!(
            err.to_string()
                .contains("ONNX inference adapter not configured yet"),
            "error should explain why inference cannot run"
        );
    }
}
