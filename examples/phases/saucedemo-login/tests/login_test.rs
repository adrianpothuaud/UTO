//! Login tests for https://www.saucedemo.com
//!
//! Covers the three main flows:
//!   1. Nominal — valid credentials → lands on products page
//!   2. Error — invalid or locked-out credentials → error banner shown
//!   3. Form validation — empty/partial fields → field-level validation messages

#[path = "support/mod.rs"]
mod support;

use uto_test::uto_test;
use saucedemo_login::web::login;

// ---------------------------------------------------------------------------
// 1. Nominal path — successful login
// ---------------------------------------------------------------------------

/// Standard user with valid credentials navigates to the inventory page.
#[uto_test(target = "web")]
#[tokio::test]
async fn login_nominal_valid_credentials_lands_on_products() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    let result = login::scenario_login_success(
        &session,
        login::USER_STANDARD,
        login::PASSWORD_VALID,
    )
    .await;

    saucedemo_login::finish_test(session, result)
        .await
        .expect("login scenario");
}

// ---------------------------------------------------------------------------
// 2. Error paths — authentication failures
// ---------------------------------------------------------------------------

/// Wrong password shows the generic credential error banner.
#[uto_test(target = "web")]
#[tokio::test]
async fn login_error_wrong_password_shows_credential_error() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    let result = login::scenario_login_error(
        &session,
        login::USER_STANDARD,
        login::PASSWORD_INVALID,
        "Username and password do not match",
    )
    .await;

    saucedemo_login::finish_test(session, result)
        .await
        .expect("wrong password scenario");
}

/// The locked-out user account is blocked even with the correct password.
#[uto_test(target = "web")]
#[tokio::test]
async fn login_error_locked_out_user_shows_locked_error() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    let result = login::scenario_login_error(
        &session,
        login::USER_LOCKED_OUT,
        login::PASSWORD_VALID,
        "locked out",
    )
    .await;

    saucedemo_login::finish_test(session, result)
        .await
        .expect("locked-out user scenario");
}

// ---------------------------------------------------------------------------
// 3. Form validation — missing / empty fields
// ---------------------------------------------------------------------------

/// Submitting with no username shows the "Username is required" validation.
#[uto_test(target = "web")]
#[tokio::test]
async fn login_form_empty_username_shows_required_error() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    let result = login::scenario_form_validation(
        &session,
        "",
        login::PASSWORD_VALID,
        "Username is required",
    )
    .await;

    saucedemo_login::finish_test(session, result)
        .await
        .expect("empty username scenario");
}

/// Submitting with no password shows the "Password is required" validation.
#[uto_test(target = "web")]
#[tokio::test]
async fn login_form_empty_password_shows_required_error() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    let result = login::scenario_form_validation(
        &session,
        login::USER_STANDARD,
        "",
        "Password is required",
    )
    .await;

    saucedemo_login::finish_test(session, result)
        .await
        .expect("empty password scenario");
}

/// Submitting completely empty form shows the "Username is required" message
/// (username field is validated first).
#[uto_test(target = "web")]
#[tokio::test]
async fn login_form_both_empty_shows_username_required_first() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    let result = login::scenario_form_validation(
        &session,
        "",
        "",
        "Username is required",
    )
    .await;

    saucedemo_login::finish_test(session, result)
        .await
        .expect("both empty scenario");
}
