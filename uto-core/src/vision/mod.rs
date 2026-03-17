pub mod consensus;
/// Vision-based UI element detection using ONNX models.
///
/// Phase 3 introduces computer vision capabilities to enable intent-driven automation
/// that does not rely on brittle CSS or XPath selectors. Instead, the framework detects
/// UI elements visually and fuses that with accessibility tree data for robustness.
pub mod inference;
pub mod latency;
pub mod postprocessing;
pub mod preprocessing;

use std::time::Instant;

use image::GenericImageView;
use inference::{InferenceEngine, StubOnnxEngine};
use postprocessing::postprocess_detections;

pub use consensus::{
    resolve_candidates, select_by_label, select_by_label_with_config, summarize_ranked_candidates,
    AccessibilityNode, CandidateSource, ConsensusConfig, ResolvedCandidate,
};
pub use latency::{LatencyTracker, ScopedLatency};

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
    screenshot_bytes: &[u8],
) -> crate::error::UtoResult<VisionDetectionResult> {
    let engine = StubOnnxEngine::new();
    detect_ui_elements_with_engine(screenshot_bytes, &engine)
}

/// Runs vision-based detection using an injectable inference engine.
///
/// This is used by tests and future ONNX integration to keep preprocessing,
/// post-processing, and ranking deterministic regardless of inference backend.
pub fn detect_ui_elements_with_engine(
    screenshot_bytes: &[u8],
    engine: &dyn InferenceEngine,
) -> crate::error::UtoResult<VisionDetectionResult> {
    let image = preprocessing::load_screenshot(screenshot_bytes)?;
    let image_size = image.dimensions();

    // Phase 3 baseline input size. Can become model-driven in later iterations.
    let resized = preprocessing::resize_for_inference(&image, (224, 224));
    let normalized = preprocessing::normalize_pixels(&resized);

    let start = Instant::now();
    let raw_detections = engine.infer(&normalized, image_size)?;
    let inference_time_ms = start.elapsed().as_millis() as u64;

    let elements = postprocess_detections(raw_detections, image_size, 0.50, 0.45);

    Ok(VisionDetectionResult {
        elements,
        metadata: DetectionMetadata {
            model_name: engine.model_name().to_string(),
            inference_time_ms,
            image_size,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::UtoResult;
    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};

    struct FakeEngine;

    impl inference::InferenceEngine for FakeEngine {
        fn model_name(&self) -> &'static str {
            "fake-model-v1"
        }

        fn infer(
            &self,
            _normalized_pixels: &[f32],
            _original_image_size: (u32, u32),
        ) -> UtoResult<Vec<inference::RawDetection>> {
            Ok(vec![
                inference::RawDetection {
                    bbox: (10.0, 10.0, 120.0, 30.0),
                    confidence: 0.95,
                    element_type: "button".to_string(),
                    label: Some("Submit".to_string()),
                },
                // Overlaps with first detection and should be suppressed by NMS.
                inference::RawDetection {
                    bbox: (12.0, 12.0, 118.0, 28.0),
                    confidence: 0.86,
                    element_type: "button".to_string(),
                    label: Some("Submit".to_string()),
                },
                // Different area and should be kept.
                inference::RawDetection {
                    bbox: (240.0, 300.0, 140.0, 36.0),
                    confidence: 0.90,
                    element_type: "input".to_string(),
                    label: Some("Email".to_string()),
                },
            ])
        }
    }

    fn sample_png_bytes() -> Vec<u8> {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(400, 600, Rgba([255, 255, 255, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);

        let mut bytes = Vec::new();
        dyn_img
            .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
            .expect("test image should encode");
        bytes
    }

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

    #[test]
    fn test_detect_ui_elements_with_engine_applies_nms() {
        let png = sample_png_bytes();
        let result = detect_ui_elements_with_engine(&png, &FakeEngine)
            .expect("vision detection should succeed with fake engine");

        assert_eq!(result.metadata.model_name, "fake-model-v1");
        assert_eq!(result.metadata.image_size, (400, 600));
        assert_eq!(result.elements.len(), 2);
        assert!(result.elements[0].confidence >= result.elements[1].confidence);
        assert_eq!(result.elements[0].element_type, "button");
        assert_eq!(result.elements[1].element_type, "input");
    }
}
