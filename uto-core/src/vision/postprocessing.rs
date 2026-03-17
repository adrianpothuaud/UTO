//! Deterministic post-processing for model detections.

use super::inference::RawDetection;
use super::DetectedElement;

/// Converts raw detections into sanitized, sorted final elements.
pub fn postprocess_detections(
    raw: Vec<RawDetection>,
    image_size: (u32, u32),
    min_confidence: f32,
    iou_threshold: f32,
) -> Vec<DetectedElement> {
    let mut filtered: Vec<DetectedElement> = raw
        .into_iter()
        .filter(|d| d.confidence >= min_confidence)
        .map(|d| to_detected_element(d, image_size))
        .collect();

    filtered.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    non_max_suppression(filtered, iou_threshold)
}

fn to_detected_element(raw: RawDetection, image_size: (u32, u32)) -> DetectedElement {
    let (max_w, max_h) = image_size;
    let (x, y, w, h) = raw.bbox;

    let x = x.max(0.0).round() as i32;
    let y = y.max(0.0).round() as i32;
    let w = w.max(1.0).round() as i32;
    let h = h.max(1.0).round() as i32;

    let max_w_i = max_w as i32;
    let max_h_i = max_h as i32;

    let x = x.min(max_w_i.saturating_sub(1));
    let y = y.min(max_h_i.saturating_sub(1));
    let w = w.min(max_w_i.saturating_sub(x)).max(1);
    let h = h.min(max_h_i.saturating_sub(y)).max(1);

    DetectedElement {
        bbox: (x, y, w, h),
        confidence: raw.confidence.clamp(0.0, 1.0),
        element_type: raw.element_type,
        label: raw.label,
    }
}

fn non_max_suppression(elements: Vec<DetectedElement>, iou_threshold: f32) -> Vec<DetectedElement> {
    let mut kept: Vec<DetectedElement> = Vec::new();

    'candidate: for element in elements {
        for existing in &kept {
            if intersection_over_union(existing.bbox, element.bbox) > iou_threshold {
                continue 'candidate;
            }
        }
        kept.push(element);
    }

    kept
}

fn intersection_over_union(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> f32 {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;

    let a_right = ax + aw;
    let a_bottom = ay + ah;
    let b_right = bx + bw;
    let b_bottom = by + bh;

    let inter_left = ax.max(bx);
    let inter_top = ay.max(by);
    let inter_right = a_right.min(b_right);
    let inter_bottom = a_bottom.min(b_bottom);

    let inter_w = (inter_right - inter_left).max(0);
    let inter_h = (inter_bottom - inter_top).max(0);
    let inter_area = (inter_w * inter_h) as f32;

    if inter_area <= 0.0 {
        return 0.0;
    }

    let area_a = (aw.max(1) * ah.max(1)) as f32;
    let area_b = (bw.max(1) * bh.max(1)) as f32;
    inter_area / (area_a + area_b - inter_area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postprocessing_filters_low_confidence_and_sorts() {
        let raw = vec![
            RawDetection {
                bbox: (100.0, 100.0, 40.0, 20.0),
                confidence: 0.40,
                element_type: "button".to_string(),
                label: None,
            },
            RawDetection {
                bbox: (120.0, 120.0, 50.0, 25.0),
                confidence: 0.90,
                element_type: "input".to_string(),
                label: Some("Email".to_string()),
            },
        ];

        let result = postprocess_detections(raw, (300, 300), 0.50, 0.45);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].element_type, "input");
        assert!(result[0].confidence >= 0.9);
    }

    #[test]
    fn postprocessing_applies_nms() {
        let raw = vec![
            RawDetection {
                bbox: (10.0, 10.0, 100.0, 40.0),
                confidence: 0.95,
                element_type: "button".to_string(),
                label: Some("Buy".to_string()),
            },
            RawDetection {
                bbox: (12.0, 12.0, 96.0, 36.0),
                confidence: 0.80,
                element_type: "button".to_string(),
                label: Some("Buy".to_string()),
            },
            RawDetection {
                bbox: (200.0, 200.0, 60.0, 30.0),
                confidence: 0.70,
                element_type: "link".to_string(),
                label: Some("Details".to_string()),
            },
        ];

        let result = postprocess_detections(raw, (400, 400), 0.50, 0.45);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label.as_deref(), Some("Buy"));
        assert_eq!(result[1].label.as_deref(), Some("Details"));
    }

    #[test]
    fn postprocessing_clamps_bounding_boxes_to_image_bounds() {
        let raw = vec![RawDetection {
            bbox: (-20.0, -10.0, 500.0, 500.0),
            confidence: 0.88,
            element_type: "container".to_string(),
            label: None,
        }];

        let result = postprocess_detections(raw, (320, 240), 0.50, 0.45);
        assert_eq!(result.len(), 1);

        let (x, y, w, h) = result[0].bbox;
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert!(w <= 320);
        assert!(h <= 240);
    }
}
