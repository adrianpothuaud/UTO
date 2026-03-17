# UTO Project - Gemini Agent Prompts

## Run the Appium Demo

**Prompt:**
"Test the Appium demo. This should involve modifying main.rs to run the Appium flow, running the application, and then reverting the changes to main.rs."

## Run the ChromeDriver Demo

**Prompt:**
"Test the ChromeDriver demo. This should involve modifying main.rs to run the ChromeDriver flow, running the application, and then reverting the changes to main.rs."

## Add a new driver

**Prompt:**
"Add support for a new WebDriver, such as SafariDriver. This should include:
1.  Implementing discovery for the new driver in `uto-core/src/env/platform.rs`.
2.  Adding a new variant to the `Driver` enum in `uto-core/src/driver/mod.rs`.
3.  Creating a `start_...` function for the new driver.
4.  Updating the `session` module to handle the new driver's capabilities."
