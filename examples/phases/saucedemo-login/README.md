# saucedemo-login — UTO Example Project

A real-world UTO test project targeting [https://www.saucedemo.com](https://www.saucedemo.com),
a public e-commerce demo app maintained by Sauce Labs for automation testing practice.

## Purpose

This project is the primary validation vehicle for **UTO UI mode** (Phase 5bis), used to
benchmark UTO's interactive test authoring experience against Cypress and Playwright.

## Tests

| Test | Scenario | Expected outcome |
|------|----------|-----------------|
| `login_nominal_valid_credentials_lands_on_products` | Happy path — standard user + valid password | Inventory page is shown |
| `login_error_wrong_password_shows_credential_error` | Wrong password | Error banner: "Username and password do not match" |
| `login_error_locked_out_user_shows_locked_error` | Locked-out account | Error banner: "locked out" |
| `login_form_empty_username_shows_required_error` | Empty username field | Validation: "Username is required" |
| `login_form_empty_password_shows_required_error` | Empty password field | Validation: "Password is required" |
| `login_form_both_empty_shows_username_required_first` | Both fields empty | Validation: "Username is required" |

## Running

```bash
# Run all tests (requires ChromeDriver)
cargo test

# Run via UTO CLI
uto run --project .

# Open in UTO UI mode (run and watch individual tests)
uto ui --project . --port 4010
# Then open http://127.0.0.1:4010
```

## Test Accounts (public, from Sauce Labs)

| Username | Password | Behaviour |
|----------|----------|-----------|
| `standard_user` | `secret_sauce` | Normal login |
| `locked_out_user` | `secret_sauce` | Login blocked |
| `problem_user` | `secret_sauce` | Glitched UI |
| `performance_glitch_user` | `secret_sauce` | Slow login |
