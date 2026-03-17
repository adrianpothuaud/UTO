/// Unified error type for all UTO operations.
pub mod error;

/// Platform discovery (browsers, mobile SDKs) and driver provisioning.
pub mod env;

/// Lifecycle management of WebDriver-compatible server processes.
pub mod driver;

/// Communication layer: session creation and interaction API for web and mobile.
pub mod session;
