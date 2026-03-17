/// Platform-specific discovery of installed browsers and mobile SDKs.
pub mod platform;

/// Android/Appium auto-setup helpers for mobile environment readiness.
pub mod mobile_setup;

/// Automatic provisioning (download + extraction) of missing driver binaries.
pub mod provisioning;
