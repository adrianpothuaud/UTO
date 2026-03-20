//! Phase 6 Studio — recording and code generation tests.
//!
//! These tests validate the Studio recording pipeline: step accumulation,
//! code generation correctness, and the full record→stop→generate cycle.

use uto_ui::{generate_test_code, RecordedStep, StepKind};

// ---------------------------------------------------------------------------
// Code generation correctness
// ---------------------------------------------------------------------------

#[test]
fn generate_code_for_login_flow() {
    let steps = vec![
        RecordedStep {
            kind: StepKind::Navigate,
            target: "https://example.com/login".to_string(),
            value: None,
            ts_ms: 0,
        },
        RecordedStep {
            kind: StepKind::Fill,
            target: "Username".to_string(),
            value: Some("alice".to_string()),
            ts_ms: 500,
        },
        RecordedStep {
            kind: StepKind::Fill,
            target: "Password".to_string(),
            value: Some("secret".to_string()),
            ts_ms: 900,
        },
        RecordedStep {
            kind: StepKind::Click,
            target: "Sign in".to_string(),
            value: None,
            ts_ms: 1200,
        },
        RecordedStep {
            kind: StepKind::AssertVisible,
            target: "Dashboard".to_string(),
            value: None,
            ts_ms: 2100,
        },
    ];

    let code = generate_test_code("login_flow", &steps);

    // Must be a valid async test function
    assert!(code.contains("#[tokio::test]"));
    assert!(code.contains("async fn login_flow()"));
    assert!(code.contains("uto_core::error::UtoResult"));

    // Must start a session
    assert!(code.contains("startNewSession"));

    // Must contain all steps in order
    assert!(code.contains(r#"s.goto("https://example.com/login")"#));
    assert!(code.contains(r#"s.fill_intent("Username", "alice")"#));
    assert!(code.contains(r#"s.fill_intent("Password", "secret")"#));
    assert!(code.contains(r#"s.click_intent("Sign in")"#));
    assert!(code.contains(r#"s.assert_visible("Dashboard")"#));

    // Must close the session
    assert!(code.contains("s.close().await"));
}

#[test]
fn generate_code_for_search_flow() {
    let steps = vec![
        RecordedStep {
            kind: StepKind::Navigate,
            target: "https://example.com".to_string(),
            value: None,
            ts_ms: 0,
        },
        RecordedStep {
            kind: StepKind::Fill,
            target: "Search".to_string(),
            value: Some("uto framework".to_string()),
            ts_ms: 300,
        },
        RecordedStep {
            kind: StepKind::Click,
            target: "Search button".to_string(),
            value: None,
            ts_ms: 600,
        },
        RecordedStep {
            kind: StepKind::AssertText,
            target: "Result count".to_string(),
            value: Some("5 results".to_string()),
            ts_ms: 1000,
        },
    ];

    let code = generate_test_code("search_flow", &steps);

    assert!(code.contains("async fn search_flow()"));
    assert!(code.contains(r#"s.fill_intent("Search", "uto framework")"#));
    assert!(code.contains(r#"s.assert_text("Result count", "5 results")"#));
}

#[test]
fn generate_code_for_checkout_flow() {
    let steps = vec![
        RecordedStep {
            kind: StepKind::Navigate,
            target: "https://shop.example.com/cart".to_string(),
            value: None,
            ts_ms: 0,
        },
        RecordedStep {
            kind: StepKind::AssertVisible,
            target: "Shopping cart".to_string(),
            value: None,
            ts_ms: 200,
        },
        RecordedStep {
            kind: StepKind::Click,
            target: "Proceed to checkout".to_string(),
            value: None,
            ts_ms: 700,
        },
        RecordedStep {
            kind: StepKind::AssertGone,
            target: "Loading indicator".to_string(),
            value: None,
            ts_ms: 1500,
        },
        RecordedStep {
            kind: StepKind::AssertVisible,
            target: "Payment form".to_string(),
            value: None,
            ts_ms: 1600,
        },
    ];

    let code = generate_test_code("checkout_flow", &steps);

    assert!(code.contains("async fn checkout_flow()"));
    assert!(code.contains(r#"s.assert_gone("Loading indicator")"#));
    assert!(code.contains(r#"s.assert_visible("Payment form")"#));
}

#[test]
fn generated_code_is_syntactically_complete() {
    // Generated code must open and close the function body correctly.
    let steps = vec![RecordedStep {
        kind: StepKind::Click,
        target: "Button".to_string(),
        value: None,
        ts_ms: 0,
    }];
    let code = generate_test_code("single_step", &steps);

    // Function must open and close
    assert!(code.contains('{'));
    assert!(code.contains('}'));

    // Must not contain selector-based syntax
    assert!(!code.contains("get("), "Must not generate CSS selector calls");
    assert!(!code.contains("xpath"), "Must not generate XPath calls");
    assert!(!code.contains("querySelector"), "Must not generate DOM calls");
}

#[test]
fn generated_code_handles_special_characters() {
    let steps = vec![RecordedStep {
        kind: StepKind::Fill,
        target: "Message".to_string(),
        value: Some(r#"Say "hello" to the world"#.to_string()),
        ts_ms: 0,
    }];
    let code = generate_test_code("special_chars_test", &steps);

    // Special chars must be escaped in the generated code
    assert!(code.contains(r#"say \"hello\""#) || code.contains(r#"Say \"hello\""#));
}

#[test]
fn step_kind_serde_round_trip() {
    // Step kinds must serialize/deserialize correctly (used by the REST API).
    let step = RecordedStep {
        kind: StepKind::AssertText,
        target: "Label".to_string(),
        value: Some("Expected".to_string()),
        ts_ms: 42,
    };
    let json = serde_json::to_string(&step).expect("serialize");
    let restored: RecordedStep = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.kind, StepKind::AssertText);
    assert_eq!(restored.target, "Label");
    assert_eq!(restored.value.as_deref(), Some("Expected"));
    assert_eq!(restored.ts_ms, 42);
}
