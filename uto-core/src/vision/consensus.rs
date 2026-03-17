//! Weighted consensus resolver for Phase 3.2.

use crate::error::{UtoError, UtoResult};

use super::DetectedElement;

/// Accessibility candidate metadata used to enrich vision detections.
#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    /// Optional accessible name.
    pub label: Option<String>,
    /// Optional semantic role (e.g. button, textbox, link).
    pub role: Option<String>,
    /// Optional screen-space bounds in (x, y, width, height).
    pub bbox: Option<(i32, i32, i32, i32)>,
}

/// Source of the final candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateSource {
    /// Derived from vision only.
    VisionOnly,
    /// Derived from vision and accessibility consensus.
    VisionAndAccessibility,
}

/// Candidate returned by the weighted resolver.
#[derive(Debug, Clone)]
pub struct ResolvedCandidate {
    /// Original detected element.
    pub element: DetectedElement,
    /// Final weighted score used for ranking.
    pub final_score: f32,
    /// Text matching score against requested intent label.
    pub text_score: f32,
    /// Role compatibility score.
    pub role_score: f32,
    /// Vision confidence contribution.
    pub vision_score: f32,
    /// Optional matched accessibility node.
    pub accessibility: Option<AccessibilityNode>,
    /// Source provenance of the candidate.
    pub source: CandidateSource,
}

/// Configuration for weighted consensus scoring.
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Weight for model confidence.
    pub weight_vision: f32,
    /// Weight for label/text match.
    pub weight_text: f32,
    /// Weight for role consistency.
    pub weight_role: f32,
    /// Bonus weight when accessibility corroborates the candidate.
    pub weight_accessibility_bonus: f32,
    /// Minimum final score required for selection.
    pub min_select_score: f32,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            weight_vision: 0.55,
            weight_text: 0.30,
            weight_role: 0.10,
            weight_accessibility_bonus: 0.05,
            min_select_score: 0.52,
        }
    }
}

/// Ranks candidates for an intent label using weighted consensus.
pub fn resolve_candidates(
    elements: &[DetectedElement],
    accessibility_nodes: &[AccessibilityNode],
    intent_label: &str,
    config: &ConsensusConfig,
) -> Vec<ResolvedCandidate> {
    let mut ranked: Vec<ResolvedCandidate> = elements
        .iter()
        .cloned()
        .map(|element| {
            let best_ax = best_accessibility_match(&element, accessibility_nodes);

            let vision_score = element.confidence.clamp(0.0, 1.0);

            let primary_label = best_ax
                .as_ref()
                .and_then(|n| n.label.as_deref())
                .or(element.label.as_deref());
            let text_score = label_similarity(intent_label, primary_label.unwrap_or_default());

            let role_score = role_compatibility_score(
                intent_label,
                &element.element_type,
                best_ax.as_ref().and_then(|n| n.role.as_deref()),
            );

            let has_ax = best_ax.is_some();
            let bonus = if has_ax { 1.0 } else { 0.0 };

            let final_score = (config.weight_vision * vision_score)
                + (config.weight_text * text_score)
                + (config.weight_role * role_score)
                + (config.weight_accessibility_bonus * bonus);

            ResolvedCandidate {
                element,
                final_score,
                text_score,
                role_score,
                vision_score,
                accessibility: best_ax,
                source: if has_ax {
                    CandidateSource::VisionAndAccessibility
                } else {
                    CandidateSource::VisionOnly
                },
            }
        })
        .collect();

    ranked.sort_by(|a, b| b.final_score.total_cmp(&a.final_score));
    ranked
}

/// Selects the highest-ranked candidate for the given label.
pub fn select_by_label(
    elements: &[DetectedElement],
    accessibility_nodes: &[AccessibilityNode],
    label: &str,
) -> UtoResult<ResolvedCandidate> {
    select_by_label_with_config(
        elements,
        accessibility_nodes,
        label,
        &ConsensusConfig::default(),
    )
}

/// Selects the highest-ranked candidate using a custom resolver configuration.
pub fn select_by_label_with_config(
    elements: &[DetectedElement],
    accessibility_nodes: &[AccessibilityNode],
    label: &str,
    config: &ConsensusConfig,
) -> UtoResult<ResolvedCandidate> {
    let ranked = resolve_candidates(elements, accessibility_nodes, label, config);
    if ranked.is_empty() {
        return Err(UtoError::VisionResolutionFailed(format!(
            "No candidates available for intent '{label}'. Ensure vision and/or accessibility extraction produced interactive nodes."
        )));
    }

    let best = ranked[0].clone();
    if best.final_score < config.min_select_score {
        let reasons = resolution_failure_reasons(&best, config)
            .into_iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let score_gap = (config.min_select_score - best.final_score).max(0.0);
        return Err(UtoError::VisionResolutionFailed(format!(
            "No candidate reached minimum score {:.2} for intent '{}'. Best score={:.2} (gap={:.2}), best-components=[vision={:.2}, text={:.2}, role={:.2}], likely-reasons=[{}]. Top candidates: {}",
            config.min_select_score,
            label,
            best.final_score,
            score_gap,
            best.vision_score,
            best.text_score,
            best.role_score,
            reasons,
            summarize_ranked_candidates(&ranked, 3)
        )));
    }

    Ok(best)
}

/// Formats the top ranked candidates for logs and diagnostics.
pub fn summarize_ranked_candidates(candidates: &[ResolvedCandidate], max_items: usize) -> String {
    candidates
        .iter()
        .take(max_items)
        .map(|c| {
            let label = c
                .accessibility
                .as_ref()
                .and_then(|n| n.label.as_deref())
                .or(c.element.label.as_deref())
                .unwrap_or("<no-label>");
            format!(
                "{}[{:.2}] type={} source={:?}",
                label, c.final_score, c.element.element_type, c.source
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn best_accessibility_match(
    element: &DetectedElement,
    accessibility_nodes: &[AccessibilityNode],
) -> Option<AccessibilityNode> {
    accessibility_nodes
        .iter()
        .filter_map(|node| {
            let node_bbox = node.bbox?;
            let overlap = intersection_over_union(element.bbox, node_bbox);
            Some((overlap, node.clone()))
        })
        .max_by(|(a, _), (b, _)| a.total_cmp(b))
        .and_then(|(iou, node)| if iou > 0.10 { Some(node) } else { None })
}

fn label_similarity(query: &str, candidate: &str) -> f32 {
    let q = normalize_text(query);
    let c = normalize_text(candidate);

    if q.is_empty() || c.is_empty() {
        return 0.0;
    }
    if q == c {
        return 1.0;
    }
    if c.contains(&q) || q.contains(&c) {
        return 0.85;
    }

    let q_tokens: Vec<&str> = q.split_whitespace().collect();
    let c_tokens: Vec<&str> = c.split_whitespace().collect();
    if q_tokens.is_empty() || c_tokens.is_empty() {
        return 0.0;
    }

    let overlap = q_tokens.iter().filter(|t| c_tokens.contains(t)).count() as f32;
    let total = (q_tokens.len() + c_tokens.len()) as f32 - overlap;

    if total <= 0.0 {
        0.0
    } else {
        overlap / total
    }
}

fn role_compatibility_score(intent_label: &str, element_type: &str, ax_role: Option<&str>) -> f32 {
    let hint = infer_role_hint(intent_label);
    if hint.is_none() {
        return 0.5;
    }
    let hint = hint.unwrap_or_default();

    let elem = normalize_text(element_type);
    let ax = normalize_text(ax_role.unwrap_or_default());

    if elem.contains(&hint) || ax.contains(&hint) {
        1.0
    } else {
        0.2
    }
}

fn infer_role_hint(text: &str) -> Option<String> {
    let t = normalize_text(text);

    if t.contains("button") || t.contains("click") || t.contains("tap") {
        return Some("button".to_string());
    }
    if t.contains("input") || t.contains("field") || t.contains("email") || t.contains("password") {
        return Some("input".to_string());
    }
    if t.contains("link") {
        return Some("link".to_string());
    }

    None
}

fn normalize_text(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolution_failure_reasons(
    best: &ResolvedCandidate,
    config: &ConsensusConfig,
) -> Vec<&'static str> {
    let mut reasons = Vec::new();

    if best.text_score < 0.35 {
        reasons.push("intent-label mismatch");
    }
    if best.role_score < 0.35 {
        reasons.push("role incompatibility");
    }
    if best.vision_score < 0.50 {
        reasons.push("low vision confidence");
    }
    if best.final_score + 0.001 < config.min_select_score && best.accessibility.is_none() {
        reasons.push("no accessibility corroboration");
    }

    if reasons.is_empty() {
        reasons.push("score below threshold");
    }

    reasons
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

    fn element(
        bbox: (i32, i32, i32, i32),
        confidence: f32,
        element_type: &str,
        label: Option<&str>,
    ) -> DetectedElement {
        DetectedElement {
            bbox,
            confidence,
            element_type: element_type.to_string(),
            label: label.map(ToString::to_string),
        }
    }

    fn ax_node(
        bbox: Option<(i32, i32, i32, i32)>,
        label: Option<&str>,
        role: Option<&str>,
    ) -> AccessibilityNode {
        AccessibilityNode {
            label: label.map(ToString::to_string),
            role: role.map(ToString::to_string),
            bbox,
        }
    }

    #[test]
    fn resolver_prefers_label_match_over_raw_confidence() {
        let elements = vec![
            element((10, 10, 120, 32), 0.95, "button", Some("Cancel")),
            element((10, 80, 120, 32), 0.82, "button", Some("Submit")),
        ];

        let selected =
            select_by_label(&elements, &[], "Submit").expect("resolver should find submit element");

        assert_eq!(selected.element.label.as_deref(), Some("Submit"));
    }

    #[test]
    fn resolver_uses_accessibility_to_promote_candidate() {
        let elements = vec![
            element((10, 10, 120, 32), 0.80, "button", Some("Action")),
            element((10, 80, 120, 32), 0.78, "button", Some("Action")),
        ];

        let nodes = vec![
            ax_node(Some((10, 80, 120, 32)), Some("Checkout"), Some("button")),
            ax_node(Some((10, 10, 120, 32)), Some("Cancel"), Some("button")),
        ];

        let selected = select_by_label(&elements, &nodes, "Checkout")
            .expect("resolver should use accessibility label");

        assert_eq!(selected.element.bbox, (10, 80, 120, 32));
        assert_eq!(selected.source, CandidateSource::VisionAndAccessibility);
    }

    #[test]
    fn select_error_contains_top_candidates() {
        let elements = vec![
            element((10, 10, 120, 32), 0.40, "button", Some("Cancel")),
            element((10, 80, 120, 32), 0.42, "button", Some("Help")),
        ];

        let err = select_by_label(&elements, &[], "Confirm")
            .expect_err("selection should fail with low and mismatched candidates");

        let msg = err.to_string();
        assert!(msg.contains("Top candidates"));
        assert!(msg.contains("Cancel") || msg.contains("Help"));
    }

    #[test]
    fn select_error_contains_mismatch_reasons() {
        let elements = vec![
            element((10, 10, 120, 32), 0.30, "button", Some("Cancel")),
            element((10, 80, 120, 32), 0.32, "button", Some("Help")),
        ];

        let err = select_by_label(&elements, &[], "Checkout")
            .expect_err("selection should fail with explicit mismatch diagnostics");

        let msg = err.to_string();
        assert!(msg.contains("likely-reasons"));
        assert!(
            msg.contains("intent-label mismatch") || msg.contains("low vision confidence"),
            "message should include at least one concrete mismatch reason: {msg}"
        );
    }

    #[test]
    fn empty_candidates_error_mentions_extraction_hint() {
        let err = select_by_label(&[], &[], "Submit")
            .expect_err("selection should fail when no candidates exist");
        let msg = err.to_string();
        assert!(msg.contains("Ensure vision and/or accessibility extraction"));
    }

    #[test]
    fn summarize_ranked_candidates_formats_scores_and_sources() {
        let elements = vec![element((10, 10, 120, 32), 0.80, "button", Some("Submit"))];
        let ranked = resolve_candidates(&elements, &[], "Submit", &ConsensusConfig::default());

        let summary = summarize_ranked_candidates(&ranked, 1);
        assert!(summary.contains("Submit"));
        assert!(summary.contains("source=VisionOnly"));
    }

    #[test]
    fn resolution_latency_meets_phase_3_sla() {
        use super::super::LatencyTracker;

        // Phase 3 SLA: median <= 50ms, p95 <= 100ms for resolution
        let mut latency = LatencyTracker::new("select_by_label");

        // Generate a large deterministic fixture: 100 elements
        let mut elements = Vec::new();
        for i in 0..100 {
            elements.push(element(
                (10 + i * 50, 10 + i * 40, 40, 30),
                0.75,
                "button",
                Some(&format!("Element{}", i)),
            ));
        }

        // Perform deterministic resolution operations and measure latency
        for _ in 0..10 {
            let start = std::time::Instant::now();
            let _resolved = select_by_label(&elements, &[], "Element42")
                .expect("resolution should succeed on deterministic fixture");
            let elapsed = start.elapsed().as_millis() as u64;
            latency.record(elapsed);
        }

        // Verify SLA bounds
        assert!(latency.meets_sla(50, 100), "{}", latency.summary());
    }

    #[test]
    fn resolution_with_accessibility_meets_sla() {
        use super::super::LatencyTracker;

        let mut latency = LatencyTracker::new("select_with_accessibility");

        // Generate elements + accessibility nodes
        let mut elements = Vec::new();
        let mut nodes = Vec::new();

        for i in 0..50 {
            elements.push(element(
                (10 + i * 50, 10 + i * 40, 40, 30),
                0.70,
                "button",
                Some(&format!("Element{}", i)),
            ));
            nodes.push(ax_node(
                Some((10 + i * 50, 10 + i * 40, 40, 30)),
                Some(&format!("Label{}", i)),
                Some("button"),
            ));
        }

        // Measure resolution with accessibility fusion
        for _ in 0..10 {
            let start = std::time::Instant::now();
            let _resolved = select_by_label(&elements, &nodes, "Label25")
                .expect("resolution should succeed with accessibility");
            let elapsed = start.elapsed().as_millis() as u64;
            latency.record(elapsed);
        }

        // SLA for accessibility-enriched resolution: median <= 60ms, p95 <= 120ms
        assert!(latency.meets_sla(60, 120), "{}", latency.summary());
    }
}
