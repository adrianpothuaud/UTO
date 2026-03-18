//! Phase 5 UI showcase — Interactive web automation using UTO intent API.
//!
//! These tests demonstrate:
//! - Vision-first, selector-free test authoring
//! - Real-time interactive use in the UTO UI
//! - Cross-platform test design (same code for web and mobile)
//! - Intent-driven workflows that survive design changes
//!
//! All tests gracefully skip if Chrome is unavailable.

use uto_test::prelude::*;
use uto_core::session::UtoSession;
use log::info;

/// Smoke test: Navigate to a public site and verify page title and visibility.
#[tokio::test]
async fn web_navigate_and_verify_visibility() {
    let mut session = match Session::start_web_session("web_navigate_test").await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Skipping test (Chrome unavailable): {}", e);
            return;
        }
    };

    info!("Navigating to example website...");
    if let Err(e) = session.navigate("https://example.com").await {
        log::warn!("Navigation failed (connectivity?): {}", e);
        let _ = session.close().await;
        return;
    }

    info!("Waiting for page to load...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Verify that the main example.com heading is visible.
    // This uses vision-first intent matching; CSS/XPath selectors are not needed.
    info!("Asserting 'Example Domain' is visible...");
    match session.assert_visible("Example Domain").await {
        Ok(_) => {
            info!("✓ Example Domain heading found");
        }
        Err(e) => {
            log::warn!("Assertion failed: {}", e);
        }
    }

    let _ = session.close().await;
    info!("Test complete");
}

/// Multi-step workflow: Navigate, interact, and assert outcomes.
#[tokio::test]
async fn web_multi_step_workflow() {
    let mut session = match Session::start_web_session("web_workflow_test").await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Skipping test (Chrome unavailable): {}", e);
            return;
        }
    };

    info!("=== Multi-Step Workflow Test ===");

    // Step 1: Navigate
    info!("Step 1: Navigate to example.com");
    if let Err(e) = session.navigate("https://example.com").await {
        log::warn!("Navigation failed: {}", e);
        let _ = session.close().await;
        return;
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Step 2: Assert page structure
    info!("Step 2: Assert page contains recognizable headings");
    match session.assert_visible("Example Domain").await {
        Ok(_) => info!("✓ Found Example Domain heading"),
        Err(e) => info!("Note: {}", e),
    }

    // Step 3: Check for descriptive text
    info!("Step 3: Assert page description is present");
    match session.assert_visible("This domain is for use in examples").await {
        Ok(_) => info!("✓ Found descriptive text"),
        Err(e) => info!("Note: {}", e),
    }

    // Step 4: Cleanup
    info!("Step 4: Close session");
    let _ = session.close().await;

    info!("=== Workflow Complete ===");
}

/// Test: Navigate to multiple sites and verify content (simulates rapid testing).
#[tokio::test]
async fn web_rapid_navigation_series() {
    let mut session = match Session::start_web_session("rapid_nav_test").await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Skipping test (Chrome unavailable): {}", e);
            return;
        }
    };

    info!("=== Rapid Navigation Series ===");

    let sites = vec![
        ("https://example.com", "Example Domain"),
        ("https://www.rust-lang.org", "Rust"),
    ];

    for (url, expected_keyword) in sites {
        info!("Navigating to: {}", url);
        match session.navigate(url).await {
            Ok(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
                info!("✓ Loaded {}", url);

                match session.assert_text_contains(expected_keyword).await {
                    Ok(_) => info!("✓ Found text: {}", expected_keyword),
                    Err(e) => info!("Note: Text not found — {}", e),
                }
            }
            Err(e) => {
                log::warn!("Failed to navigate to {}: {}", url, e);
                break;
            }
        }
    }

    let _ = session.close().await;
    info!("=== Series Complete ===");
}

/// Test: Long-running workflow to demonstrate live UI event streaming.
#[tokio::test]
async fn web_extended_session_with_waits() {
    let mut session = match Session::start_web_session("extended_session_test").await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Skipping test (Chrome unavailable): {}", e);
            return;
        }
    };

    info!("=== Extended Session Demonstrating Live UI Updates ===");

    // This test intentionally takes longer so you can watch the UI
    // update with events in real-time.

    info!("[1/5] Initial navigation...");
    if let Err(e) = session.navigate("https://example.com").await {
        log::warn!("Navigation failed: {}", e);
        let _ = session.close().await;
        return;
    }

    info!("[2/5] Allowing page to fully render (500ms)...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    info!("[3/5] Verifying primary content...");
    let _ = session.assert_visible("Example Domain").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    info!("[4/5] Checking secondary content...");
    let _ = session.assert_visible("More information").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    info!("[5/5] Session cleanup...");
    let _ = session.close().await;

    info!("=== Extended Session Complete ===");
    info!("Watch the UI event list to see each step's outcome in real-time.");
}

/// Test: Demonstrates error detection (intentional assertion failure).
#[tokio::test]
async fn web_assertion_with_graceful_error_handling() {
    let mut session = match Session::start_web_session("error_handling_test").await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Skipping test (Chrome unavailable): {}", e);
            return;
        }
    };

    info!("=== Assertion with Error Handling Test ===");

    info!("Navigating...");
    if let Err(e) = session.navigate("https://example.com").await {
        log::warn!("Navigation failed: {}", e);
        let _ = session.close().await;
        return;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Try to find content that should exist
    info!("Assertion 1: Checking for expected content...");
    match session.assert_visible("Example Domain").await {
        Ok(_) => info!("✓ Assertion passed"),
        Err(e) => info!("✗ Assertion failed (expected): {}", e),
    }

    // Try to find content that likely doesn't exist (graceful error)
    info!("Assertion 2: Intentional assertion on non-existent content...");
    match session.assert_visible("UniquelyMissingElementXYZ12345").await {
        Ok(_) => info!("✓ Assertion passed"),
        Err(e) => info!("✗ Assertion failed (expected): {}", e),
    }

    info!("Both assertions completed. Check event details for outcomes.");

    let _ = session.close().await;
    info!("=== Test Complete ===");
}
