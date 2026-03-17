Add support for a new WebDriver, such as SafariDriver. This should include:
1.  Implementing discovery for the new driver in `uto-core/src/env/platform.rs`.
2.  Adding a new variant to the `Driver` enum in `uto-core/src/driver/mod.rs`.
3.  Creating a `start_...` function for the new driver.
4.  Updating the `session` module to handle the new driver's capabilities.
