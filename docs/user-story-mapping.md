Create and understand a project
    Scaffold a new UTO project with clear metadata
        As a developer, I want uto init to generate a complete runnerless project so I can run tests without building project-specific binaries
        As a maintainer, I want framework_version written to uto.json so lifecycle and compatibility rules are explicit
Author and organize tests
    Mark framework tests and runtime intent directly in test files
        As a test author, I want #[uto_test(target = "web" | "mobile")] so target routing is explicit and readable
        As a test author, I want optional tags and timeout(_ms) metadata so I can classify and tune tests without custom harnesses
        As a user upgrading old tests, I want fallback target inference from test names so migration is incremental
Execute suites and inspect outcomes
    Run tests from CLI and collect machine-readable artifacts
        As an engineer, I want uto run to discover tests and orchestrate cargo test per case so execution is transparent
        As a reviewer, I want suite-level JSON and HTML reports that include target, tags, and timeout metadata so failures can be diagnosed quickly
        As a team lead, I want generated .uto example reports committed in reference projects so expected output format stays visible in reviews
Collaborate in CI and code review
    Keep behavior stable and observable across environments
        As a CI owner, I want schema and version checks to fail fast when config or report formats drift
        As a documentation reader, I want README, site, and ADRs aligned with CLI behavior so onboarding is consistent
Migrate from legacy runner projects
    Sunset per-project runner binaries safely
        As a maintainer, I want framework-version-gated legacy behavior so teams have a predictable migration window
        As a maintainer, I want warning-only behavior in 4.6 and 4.7 and hard-stop at 4.8 so removal is explicit and enforceable
Release slices
    Release 4.5 (Delivered)
        Runnerless CLI execution path as default
        #[uto_test] introduced and supported in generated and reference projects
        Immediate migration of phase4-framework and ui-showcase examples
    Release 4.6 and 4.7 (Transition)
        Keep legacy runner compatibility for existing projects
        Emit clear sunset warnings when legacy runner path is used
        Encourage metadata cleanup and test annotation adoption
    Release 4.8 (Sunset)
        Hard error on legacy runner mode
        Require CLI-owned execution flow
        Keep structured .uto reporting as first-class, reviewable artifacts