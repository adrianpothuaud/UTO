//! Page-object helpers for the Saucedemo login page.
//!
//! <https://www.saucedemo.com> is a public demo site by Sauce Labs specifically
//! designed for automation testing practice.
//!
//! All form interactions use UTO **web intents** so the test surface is
//! selector-free and driver-agnostic.  Only page-state assertions (error
//! banner, inventory list) still use a minimal set of CSS selectors because
//! those elements are non-interactive and not reachable via `select()`.

use uto_core::error::{UtoError, UtoResult};
use uto_test::ManagedSession;

pub const URL: &str = "https://www.saucedemo.com";

// ---------------------------------------------------------------------------
// Known test accounts (public, documented by Sauce Labs)
// ---------------------------------------------------------------------------

pub const USER_STANDARD: &str = "standard_user";
pub const USER_LOCKED_OUT: &str = "locked_out_user";
pub const USER_PROBLEM: &str = "problem_user";
pub const PASSWORD_VALID: &str = "secret_sauce";
pub const PASSWORD_INVALID: &str = "wrong_password";

// ---------------------------------------------------------------------------
// Assertion selectors (page-state elements, not form interactions)
// ---------------------------------------------------------------------------

/// Error banner shown on authentication / validation failure.
const SEL_ERROR_MSG: &str = "[data-test='error']";
/// Inventory list uniquely identifies the products page after login.
const SEL_INVENTORY: &str = ".inventory_list";

// ---------------------------------------------------------------------------
// Page actions
// ---------------------------------------------------------------------------

/// Navigates to the Saucedemo login page.
pub async fn open(session: &ManagedSession) -> UtoResult<()> {
    session.goto(URL).await
}

/// Submits the login form using UTO web intents — no CSS selectors required.
///
/// The intent engine resolves "Username", "Password", and "Login" by label
/// against the page's accessibility tree and visual layout.
pub async fn submit_login(
    session: &ManagedSession,
    username: &str,
    password: &str,
) -> UtoResult<()> {
    session.fill_intent("Username", username).await?;
    session.fill_intent("Password", password).await?;
    session.click_intent("Login").await?;
    Ok(())
}

/// Asserts that the products inventory list is visible (success post-login).
pub async fn assert_on_inventory_page(session: &ManagedSession) -> UtoResult<()> {
    session
        .find_element(SEL_INVENTORY)
        .await
        .map(|_| ())
        .map_err(|_| {
            UtoError::SessionCommandFailed(
                "inventory list not found — expected to land on products page".to_string(),
            )
        })
}

/// Reads the current error message banner text, failing if none is visible.
pub async fn read_error_message(session: &ManagedSession) -> UtoResult<String> {
    let el = session.find_element(SEL_ERROR_MSG).await.map_err(|_| {
        UtoError::SessionCommandFailed(
            "error message element not found — expected an error to be visible".to_string(),
        )
    })?;
    session.get_text(&el).await
}

/// Asserts the error message banner contains the given substring.
pub async fn assert_error_contains(session: &ManagedSession, fragment: &str) -> UtoResult<()> {
    let msg = read_error_message(session).await?;
    if !msg.contains(fragment) {
        return Err(UtoError::SessionCommandFailed(format!(
            "error message assertion failed: expected to contain '{fragment}', got '{msg}'"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Composite scenarios (used directly by tests)
// ---------------------------------------------------------------------------

/// Happy-path login: navigate → fill credentials via intent → assert inventory.
pub async fn scenario_login_success(
    session: &ManagedSession,
    username: &str,
    password: &str,
) -> UtoResult<()> {
    open(session).await?;
    submit_login(session, username, password).await?;
    assert_on_inventory_page(session).await
}

/// Error-path login: navigate → fill credentials via intent → assert error fragment.
pub async fn scenario_login_error(
    session: &ManagedSession,
    username: &str,
    password: &str,
    expected_error_fragment: &str,
) -> UtoResult<()> {
    open(session).await?;
    submit_login(session, username, password).await?;
    assert_error_contains(session, expected_error_fragment).await
}

/// Form-validation path: submit with the supplied (possibly empty) values
/// and assert a specific error fragment is shown.
pub async fn scenario_form_validation(
    session: &ManagedSession,
    username: &str,
    password: &str,
    expected_error_fragment: &str,
) -> UtoResult<()> {
    open(session).await?;
    submit_login(session, username, password).await?;
    assert_error_contains(session, expected_error_fragment).await
}
