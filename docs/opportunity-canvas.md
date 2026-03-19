Problems
    Per-project runner binaries increase cognitive load and reduce execution transparency
    Users expect framework-style UX (init, run, report) like Playwright and Cypress workflows
    Migration risk exists for older projects that still contain src/bin/uto_project_runner.rs
Users and Customers
    QA engineers authoring UI automation
    SET and SDET teams scaling cross-platform suites
    Developers validating flows locally before CI
    Product and engineering leads needing fast report-driven insight
Solutions Today
    Selenium with custom harness scripts
    WebdriverIO, Cypress, and Playwright managed runner models
    In-house Rust wrappers around cargo test with ad hoc reporting
Solution Ideas
    Make CLI-owned discovery and execution the default (uto run)
    Keep authored tests as standard Rust tests enhanced by #[uto_test(...)] metadata
    Maintain structured JSON and HTML reports and include test metadata (target, tags, timeout_ms)
    Enforce migration with framework_version metadata policy
        4.6 and 4.7: warning for legacy runner usage
        4.8 and later: hard error for legacy runner mode
Business Challenges
    Metadata drift between docs, examples, templates, and generated defaults
    Teams may delay migration and hit the 4.8 hard-stop unexpectedly
How will Users use Solution?
    Developers run uto init to scaffold runnerless projects with explicit framework metadata
    Test authors annotate cases with #[uto_test(...)] while keeping normal Rust test ergonomics
    Teams run uto run and inspect JSON and HTML reports locally, in UI mode, and in CI
    Maintainers rely on warning and hard-stop enforcement tied to framework_version
User Metrics
    Percentage of projects without uto_project_runner.rs
    Percentage of tests annotated with #[uto_test] or covered by fallback inference
    Reduction in setup and support issues around execution transparency
    Stable CI consumption of uto-suite/v1 outputs
Adoption Strategy
    Ship framework_version 4.5 in generated and reference uto.json files
    Migrate committed phase examples immediately to runnerless model
    Keep generated .uto example reports tracked to show expected artifact shape in pull requests
    Communicate sunset window in docs and CLI warning text
Business Benefits and Metrics
    Faster onboarding and lower maintenance overhead
    Better observability and diagnosability for execution failures
    Clear path to Phase 5 interactive UI workflows without duplicated runner logic
Budget
    No dedicated line-item budget defined in this artifact
    Investment focus is engineering time for migration messaging, docs alignment, and compatibility guardrails