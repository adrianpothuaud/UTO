/// Vision-based UI element detection using ONNX models.
///
/// Phase 3 introduces computer vision capabilities to enable intent-driven automation
/// that does not rely on brittle CSS or XPath selectors. Instead, the framework detects
/// UI elements visually and fuses that with accessibility tree data for robustness.
pub mod preprocessing;

/// Represents the location and confidence of a detected UI element.
#[derive(Debug, Clone)]
pub struct DetectedElement {
    /// Bounding box in (x, y, width, height) format.
    pub bbox: (i32, i32, i32, i32),
    /// Confidence score from the model (0.0 to 1.0).
    pub confidence: f32,
    /// Human-readable label (button, input, link, etc.)
    pub element_type: String,
    /// Optional accessibility label or text if available.
    pub label: Option<String>,
}

/// Result of running vision-based UI detection on a screenshot.
#[derive(Debug, Clone)]
pub struct VisionDetectionResult {
    /// Detected UI elements sorted by confidence (highest first).
    pub elements: Vec<DetectedElement>,
    /// Metadata about the detection run.
    pub metadata: DetectionMetadata,
}

/// Metadata about a detection run.
#[derive(Debug, Clone)]
pub struct DetectionMetadata {
    /// Model version or name used.
    pub model_name: String,
    /// Time taken for inference in milliseconds.
    pub inference_time_ms: u64,
    /// Input image dimensions (width x height).
    pub image_size: (u32, u32),
}

/// Placeholder for Phase 3: runs vision-based detection on a screenshot.
///
/// Currently a stub. Will be implemented in Phase 3 to:
/// 1. Preprocess the screenshot
/// 2. Run ONNX model inference
/// 3. Post-process model output into bounding boxes + confidence
/// 4. Return detected UI elements
pub fn detect_ui_elements(
    _screenshot_bytes: &[u8],
) -> crate::error::UtoResult<VisionDetectionResult> {
    Err(crate::error::UtoError::Internal(
        "Vision detection not yet implemented (Phase 3 feature)".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_element_structure() {
        let elem = DetectedElement {
            bbox: (100, 200, 50, 30),
            confidence: 0.95,
            element_type: "button".to_string(),
            label: Some("Submit".to_string()),
        };

        assert_eq!(elem.bbox.0, 100);
        assert!(elem.confidence > 0.9);
    }
}
