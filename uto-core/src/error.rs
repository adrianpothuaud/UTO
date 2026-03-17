/// Unified error type for all UTO operations.
#[derive(Debug, thiserror::Error)]
pub enum UtoError {
    // --- Environment / Provisioning errors ---
    /// The browser could not be found on the system.
    #[error("Browser not found: {0}")]
    BrowserNotFound(String),

    /// The Android SDK could not be located.
    #[error(
        "Android SDK not found. Set the ANDROID_HOME environment variable or install the SDK."
    )]
    AndroidSdkNotFound,

    /// Appium was not found in PATH and could not be provisioned automatically.
    #[error("Appium not found. Install it with: npm install -g appium")]
    AppiumNotFound,

    /// Mobile environment setup failed.
    #[error("Environment setup failed: {0}")]
    EnvironmentSetupFailed(String),

    /// A required system binary (e.g. `adb`) was not found in PATH.
    #[error("Required binary '{0}' not found in PATH")]
    BinaryNotFound(String),

    // --- Driver lifecycle errors ---
    /// The driver process failed to start.
    #[error("Driver failed to start: {0}")]
    DriverStartFailed(String),

    /// The driver process failed to stop cleanly.
    #[error("Driver failed to stop: {0}")]
    DriverStopFailed(String),

    /// No free TCP port could be found for the driver.
    #[error("No free port available for the driver")]
    NoFreePort,

    // --- Session / Communication errors ---
    /// A WebDriver session could not be created.
    #[error("Session creation failed: {0}")]
    SessionCreationFailed(String),

    /// A session command failed.
    #[error("Session command failed: {0}")]
    SessionCommandFailed(String),

    /// Vision candidate resolution failed for the requested intent.
    #[error("Vision resolution failed: {0}")]
    VisionResolutionFailed(String),

    // --- I/O and network errors ---
    /// An HTTP request failed during provisioning.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// A file system operation failed.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// A JSON parse/serialize operation failed.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// An unexpected internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience alias for a `Result` carrying a [`UtoError`].
pub type UtoResult<T> = Result<T, UtoError>;
