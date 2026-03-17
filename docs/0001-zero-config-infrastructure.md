# 1. Zero-Config Infrastructure

Date: 2024-05-21

## Status

Accepted

## Context

Modern test automation frameworks often require significant manual setup from the user, including installing specific browser versions, downloading matching WebDriver binaries, and managing their system's `PATH`. This creates a high barrier to entry and is a frequent source of flaky tests and environment-related errors. UTO's goal is to provide a "zero-to-one" experience where the framework handles all environmental dependencies.

## Decision

UTO will implement a "Discover or Deploy" strategy for its environment management (`uto-env`):

1.  **Discovery:** The engine will first attempt to discover existing installations of browsers (Chrome, etc.) and mobile SDKs (Android SDK, Xcode) using a tiered search.
2.  **Deployment:** If discovery fails, UTO will automatically download (provision) a known-good, version-pinned, portable version of the required tool (e.g., Chromium) into a local, managed cache (`.uto/cache`).
3.  **Process Management:** To prevent "zombie" driver processes, UTO will use OS-native process management. On Unix-like systems, all child processes will be spawned in a new **Process Group**. On Windows, a **Job Object** will be used. When the main UTO process terminates, the OS will automatically terminate all processes within that group/job.

## Consequences

**Positive:**
*   **Zero-Config:** Users can run UTO on a fresh machine without any prior setup.
*   **Resiliency:** Tests are more stable as browser/driver versions are perfectly matched.
*   **Cleanliness:** No orphaned processes are left running after tests, improving system stability.

**Negative:**
*   **Initial Overhead:** The first run may be slower as UTO might need to download binaries.
*   **Complexity:** The `uto-env` module requires OS-specific implementation details.

## Implementation Notes

A proof of concept has been successfully implemented for Chrome and ChromeDriver, validating the "Discover or Deploy" strategy for web automation.

The same strategy has been extended to mobile automation with Appium and Android:

1.  **Discovery:**
    *   **Chrome:** The `uto-env` module successfully discovers the installed version of Google Chrome.
    *   **Android SDK:** It discovers the Android SDK by checking the `ANDROID_HOME` environment variable and common installation paths.
    *   **Appium:** It discovers an existing Appium installation by searching the system's `PATH`.
2.  **Deployment:**
    *   **ChromeDriver:** It fetches metadata from the Chrome for Testing JSON endpoints, downloads the correct driver, and extracts it to a local cache.
    *   **Appium:** If Appium is not found, UTO attempts to install it through `npm install -g appium`, then verifies discovery.
    *   **Appium Android Driver:** UTO verifies that `uiautomator2` is installed and runs `appium driver install uiautomator2` when missing.
    *   **Android Runtime Readiness:** UTO starts `adb`, checks for an online device, and can auto-start an available Android emulator AVD when no device is connected.
3.  **Process Management:**
    *   The `uto-driver` module has been refactored to handle multiple driver types.
    *   It can start and manage both `chromedriver` and `appium` processes in isolated process groups.
    *   The `uto-session` module can create both web and mobile sessions with the appropriate capabilities.

This work validates the core principles of UTO for both web and mobile, providing a solid foundation for building a unified testing framework.