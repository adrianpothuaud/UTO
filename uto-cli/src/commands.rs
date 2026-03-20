//! CLI command implementations.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::config::{
    UtoProjectConfig, DEFAULT_FRAMEWORK_VERSION, DEFAULT_REPORT_SCHEMA_VERSION,
    PROJECT_SCHEMA_VERSION,
};
use crate::io::write_json;
use crate::templates;

#[derive(Debug, Clone)]
struct DiscoveredTestCase {
    test_bin: String,
    test_name: String,
    target: Option<String>,
    tags: Vec<String>,
    timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Default)]
struct ParsedUtoAttributes {
    target: Option<String>,
    tags: Vec<String>,
    timeout_ms: Option<u64>,
}

fn has_legacy_runner(project: &Path) -> bool {
    project.join("src/bin/uto_project_runner.rs").exists()
}

fn truncate_for_report(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut out = String::new();
    for ch in input.chars().take(max_chars) {
        out.push(ch);
    }
    out.push_str("\n...<truncated>...");
    out
}

fn sanitize_filename_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "test".to_string()
    } else {
        out
    }
}

fn per_test_live_stream_path(report_path: &Path, case: &DiscoveredTestCase) -> PathBuf {
    let file_name = format!(
        "{}__{}.jsonl",
        sanitize_filename_segment(&case.test_bin),
        sanitize_filename_segment(&case.test_name)
    );

    report_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("live-events")
        .join(file_name)
}

fn absolutize_path(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path)
    } else {
        let cwd = std::env::current_dir().map_err(|err| err.to_string())?;
        Ok(cwd.join(path))
    }
}

fn append_live_stream_line(path: &Path, line: &str) -> Result<(), String> {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| {
            format!(
                "Failed to open live event stream '{}': {err}",
                path.display()
            )
        })?;

    file.write_all(line.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|err| {
            format!(
                "Failed to append live event stream '{}': {err}",
                path.display()
            )
        })
}

fn drain_live_event_updates(
    source: &Path,
    sink: &Path,
    consumed_bytes: &mut usize,
    carry: &mut String,
) -> Result<(), String> {
    let data = match fs::read(source) {
        Ok(data) => data,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(format!(
                "Failed to read live event source '{}': {err}",
                source.display()
            ));
        }
    };

    if data.len() <= *consumed_bytes {
        return Ok(());
    }

    let delta = &data[*consumed_bytes..];
    *consumed_bytes = data.len();

    let delta_text = std::str::from_utf8(delta).map_err(|err| {
        format!(
            "Live event source '{}' emitted invalid UTF-8: {err}",
            source.display()
        )
    })?;
    carry.push_str(delta_text);

    while let Some(newline_idx) = carry.find('\n') {
        let line = carry[..newline_idx].trim();
        if !line.is_empty() {
            append_live_stream_line(sink, line)?;
        }
        let remainder = carry[newline_idx + 1..].to_string();
        *carry = remainder;
    }

    Ok(())
}

fn spawn_live_event_forwarder(
    source: PathBuf,
    sink: PathBuf,
    stop_flag: Arc<AtomicBool>,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let mut consumed_bytes = 0usize;
        let mut carry = String::new();

        while !stop_flag.load(Ordering::Relaxed) {
            drain_live_event_updates(&source, &sink, &mut consumed_bytes, &mut carry)?;
            thread::sleep(Duration::from_millis(25));
        }

        drain_live_event_updates(&source, &sink, &mut consumed_bytes, &mut carry)?;

        if !carry.trim().is_empty() {
            append_live_stream_line(&sink, carry.trim())?;
        }

        Ok(())
    })
}

/// Reads all `ReportEvent` entries from a per-test live-stream JSONL file and
/// injects them into a test handle.  Skips envelope-level events (test_started,
/// test_finished, runner.test_command) that are already recorded by the CLI.
fn inject_live_events_into_handle(handle: &mut uto_reporter::TestRunHandle, jsonl_path: &Path) {
    let content = match fs::read_to_string(jsonl_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let envelope: uto_runner::LiveEventEnvelope = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        match envelope.payload {
            uto_runner::LiveEventPayload::ReportEvent { event } => {
                // Skip runner.test_command — the CLI records its own version.
                if event.stage.starts_with("runner.test_command") {
                    continue;
                }
                handle.event(&event.stage, &event.status, event.detail);
            }
            // TestStarted / TestFinished are envelope-level; skip.
            _ => {}
        }
    }
}

fn extract_fn_name(line: &str) -> Option<String> {
    let candidates = ["pub async fn ", "async fn ", "pub fn ", "fn "];
    let prefix = candidates.into_iter().find(|p| line.starts_with(p))?;
    let rest = &line[prefix.len()..];
    let mut name = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn infer_target_from_name(name: &str) -> Option<String> {
    if name.starts_with("web_") {
        return Some("web".to_string());
    }
    if name.starts_with("mobile_") {
        return Some("mobile".to_string());
    }
    None
}

fn parse_uto_target_attribute(line: &str) -> Option<String> {
    if !line.contains("uto_test") {
        return None;
    }
    if line.contains("target") && line.contains("\"web\"") {
        return Some("web".to_string());
    }
    if line.contains("target") && line.contains("\"mobile\"") {
        return Some("mobile".to_string());
    }
    None
}

fn parse_uto_timeout_attribute(line: &str) -> Option<u64> {
    for key in ["timeout_ms", "timeout"] {
        if let Some(index) = line.find(key) {
            let fragment = &line[index + key.len()..];
            if let Some(eq) = fragment.find('=') {
                let value = fragment[eq + 1..]
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>();
                if !value.is_empty() {
                    if let Ok(parsed) = value.parse::<u64>() {
                        return Some(parsed);
                    }
                }
            }
        }
    }
    None
}

fn parse_quoted_list(fragment: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for ch in fragment.chars() {
        if ch == '"' {
            if in_quote {
                if !current.is_empty() {
                    out.push(current.clone());
                    current.clear();
                }
                in_quote = false;
            } else {
                in_quote = true;
            }
            continue;
        }

        if in_quote {
            current.push(ch);
        }
    }

    out
}

fn parse_uto_tags_attribute(line: &str) -> Vec<String> {
    if let Some(start) = line.find("tags = [") {
        let rest = &line[start + "tags = [".len()..];
        if let Some(end) = rest.find(']') {
            return parse_quoted_list(&rest[..end]);
        }
    }

    if let Some(start) = line.find("tags(") {
        let rest = &line[start + "tags(".len()..];
        if let Some(end) = rest.find(')') {
            return parse_quoted_list(&rest[..end]);
        }
    }

    Vec::new()
}

fn parse_uto_attributes(attr_text: &str) -> ParsedUtoAttributes {
    ParsedUtoAttributes {
        target: parse_uto_target_attribute(attr_text),
        tags: parse_uto_tags_attribute(attr_text),
        timeout_ms: parse_uto_timeout_attribute(attr_text),
    }
}

fn parse_framework_version(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor))
}

fn enforce_legacy_runner_gate(config: &UtoProjectConfig, project: &Path) -> Result<(), String> {
    let version_str = config
        .framework_version
        .as_deref()
        .unwrap_or(DEFAULT_FRAMEWORK_VERSION);
    let (major, minor) = parse_framework_version(version_str).ok_or_else(|| {
        format!(
            "Invalid framework_version '{}' in uto.json. Expected '<major>.<minor>'",
            version_str
        )
    })?;

    let runner = project.join("src/bin/uto_project_runner.rs");

    if major > 4 || (major == 4 && minor >= 8) {
        return Err(format!(
            "Legacy runner mode is disabled for framework_version {}. \
             Remove {} and run with CLI-owned execution only.",
            version_str,
            runner.display()
        ));
    }

    if major == 4 && (minor == 6 || minor == 7) {
        eprintln!(
            "Warning: framework_version {} is in legacy sunset window. \
             {} is still supported, but support ends at 4.8.",
            version_str,
            runner.display()
        );
    }

    Ok(())
}

fn parse_tests_from_source(test_bin: &str, source: &str) -> Vec<DiscoveredTestCase> {
    let mut out = Vec::new();
    let mut pending_test_attr = false;
    let mut pending_uto_attr = false;
    let mut pending_uto_attr_text: Option<String> = None;
    let mut pending_uto_attributes = ParsedUtoAttributes::default();

    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(current_attr) = pending_uto_attr_text.as_mut() {
            current_attr.push(' ');
            current_attr.push_str(trimmed);
            if trimmed.contains(']') {
                pending_uto_attributes = parse_uto_attributes(current_attr);
                pending_uto_attr_text = None;
            }
            continue;
        }

        if trimmed.starts_with("#[") {
            if trimmed.contains("tokio::test") || trimmed.starts_with("#[test") {
                pending_test_attr = true;
            }
            if trimmed.contains("uto_test") {
                pending_uto_attr = true;
                if trimmed.contains(']') {
                    pending_uto_attributes = parse_uto_attributes(trimmed);
                } else {
                    pending_uto_attr_text = Some(trimmed.to_string());
                }
            }
            continue;
        }

        if let Some(test_name) = extract_fn_name(trimmed) {
            if pending_test_attr || pending_uto_attr {
                let target = pending_uto_attributes
                    .target
                    .clone()
                    .or_else(|| infer_target_from_name(&test_name));
                out.push(DiscoveredTestCase {
                    test_bin: test_bin.to_string(),
                    test_name,
                    target,
                    tags: pending_uto_attributes.tags.clone(),
                    timeout_ms: pending_uto_attributes.timeout_ms,
                });
            }
            pending_test_attr = false;
            pending_uto_attr = false;
            pending_uto_attr_text = None;
            pending_uto_attributes = ParsedUtoAttributes::default();
            continue;
        }

        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            pending_test_attr = false;
        }
    }

    out
}

fn discover_project_tests(
    project: &Path,
    tests_dir: &str,
) -> Result<Vec<DiscoveredTestCase>, String> {
    let tests_root = project.join(tests_dir);
    if !tests_root.exists() {
        return Err(format!("Missing tests directory: {}", tests_root.display()));
    }

    let mut discovered = Vec::new();
    for entry in fs::read_dir(&tests_root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let test_bin = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid test filename: {}", path.display()))?
            .to_string();

        let source = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read test source '{}': {e}", path.display()))?;

        discovered.extend(parse_tests_from_source(&test_bin, &source));
    }

    discovered.sort_by(|a, b| {
        a.test_bin
            .cmp(&b.test_bin)
            .then_with(|| a.test_name.cmp(&b.test_name))
    });

    Ok(discovered)
}

fn select_tests_for_target(
    cases: Vec<DiscoveredTestCase>,
    target: &str,
) -> Vec<DiscoveredTestCase> {
    cases
        .into_iter()
        .filter(|case| match case.target.as_deref() {
            Some(t) => t == target,
            None => true,
        })
        .collect()
}

fn run_legacy_project_runner(
    project: &Path,
    effective_target: &str,
    report_path: &Path,
    driver_trace: bool,
) -> Result<(), String> {
    eprintln!(
        "Warning: legacy project runner detected at {}. \
         This compatibility mode will be removed after two minor releases.",
        project.join("src/bin/uto_project_runner.rs").display()
    );

    let mut cmd = Command::new("cargo");
    cmd.current_dir(project)
        .arg("run")
        .arg("--bin")
        .arg("uto_project_runner")
        .arg("--")
        .arg("--target")
        .arg(effective_target)
        .arg("--json")
        .arg("--report-file")
        .arg(report_path.display().to_string())
        .env("UTO_REPORT_JSON", report_path.display().to_string())
        .env("UTO_REPORT_FILE", report_path.display().to_string())
        .env("RUST_LOG", "info");

    if driver_trace {
        cmd.env("UTO_DRIVER_TRACE", "1");
    }

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to run legacy project runner: {e}"))?;

    if !status.success() {
        return Err("Legacy project runner exited with non-zero status".to_string());
    }

    Ok(())
}

fn run_cli_owned_tests(
    project: &Path,
    config: &UtoProjectConfig,
    effective_target: &str,
    report_path: &Path,
    live_events_jsonl: Option<&Path>,
    driver_trace: bool,
    test_bin_filter: Option<&str>,
    test_name_filter: Option<&str>,
) -> Result<(), String> {
    let discovered = discover_project_tests(project, &config.tests_dir)?;
    let selected = select_tests_for_target(discovered, effective_target)
        .into_iter()
        .filter(|case| {
            let bin_ok = match test_bin_filter {
                Some(bin) => case.test_bin == bin,
                None => true,
            };
            let name_ok = match test_name_filter {
                Some(name) => case.test_name == name,
                None => true,
            };
            bin_ok && name_ok
        })
        .collect::<Vec<_>>();

    if selected.is_empty() {
        return Err(format!(
            "No tests discovered for target '{}' in {}. \
             Add #[uto_test(target = \"{}\")] on test functions or use web_/mobile_ naming.",
            effective_target,
            project.join(&config.tests_dir).display(),
            effective_target,
        ));
    }

    let report_file = report_path.display().to_string();
    let mut suite = uto_reporter::SuiteReport::new(true, Some(report_file), effective_target);

    if let Some(path) = live_events_jsonl {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        if path.exists() {
            fs::remove_file(path).map_err(|err| {
                format!(
                    "Failed to reset live event stream '{}': {err}",
                    path.display()
                )
            })?;
        }
    }

    for case in selected {
        let mut handle = suite.begin_test(&case.test_name);
        let live_stream_config = live_events_jsonl.map(|path| uto_runner::LiveEventConfig {
            file_path: path.to_path_buf(),
            test_bin: case.test_bin.clone(),
            test_name: case.test_name.clone(),
            target: Some(effective_target.to_string()),
        });
        let test_command_detail = serde_json::json!({
            "binary": case.test_bin,
            "name": case.test_name,
            "target": effective_target,
            "tags": case.tags,
            "timeout_ms": case.timeout_ms,
        });

        handle.event(
            "runner.test_command",
            "running",
            test_command_detail.clone(),
        );
        if let Some(config) = &live_stream_config {
            uto_runner::append_live_event(
                &config.file_path,
                &uto_runner::LiveEventEnvelope::test_started(config),
            )?;
            uto_runner::append_live_event(
                &config.file_path,
                &uto_runner::LiveEventEnvelope::report_event(
                    config,
                    uto_reporter::ReportEvent {
                        stage: "runner.test_command".to_string(),
                        status: "running".to_string(),
                        detail: test_command_detail,
                    },
                ),
            )?;
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(project)
            .arg("test")
            .arg("--test")
            .arg(&case.test_bin)
            .arg(&case.test_name)
            .arg("--")
            .arg("--nocapture")
            .env("RUST_LOG", "info");

        if driver_trace {
            cmd.env("UTO_DRIVER_TRACE", "1");
        }

        // Always create a per-test JSONL so the subprocess can emit live events
        // and we can inject them into the suite report after the process exits.
        // The forwarder thread (for UI streaming) is only spawned when a sink path
        // (`live_events_jsonl`) is also present.
        let raw_live_stream_path = per_test_live_stream_path(report_path, &case);
        if let Some(parent) = raw_live_stream_path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        if raw_live_stream_path.exists() {
            fs::remove_file(&raw_live_stream_path).map_err(|err| {
                format!(
                    "Failed to reset test live stream '{}': {err}",
                    raw_live_stream_path.display()
                )
            })?;
        }
        cmd.env(
            uto_runner::UTO_LIVE_EVENTS_FILE_ENV,
            raw_live_stream_path.display().to_string(),
        )
        .env(uto_runner::UTO_LIVE_EVENTS_TEST_BIN_ENV, &case.test_bin)
        .env(uto_runner::UTO_LIVE_EVENTS_TEST_NAME_ENV, &case.test_name)
        .env(uto_runner::UTO_LIVE_EVENTS_TARGET_ENV, effective_target);
        let forwarder_stop = Arc::new(AtomicBool::new(false));
        let forwarder = match live_events_jsonl {
            Some(sink) => Some(spawn_live_event_forwarder(
                raw_live_stream_path.clone(),
                sink.to_path_buf(),
                Arc::clone(&forwarder_stop),
            )),
            None => None,
        };

        let started = Instant::now();
        let output = match cmd.output() {
            Ok(output) => output,
            Err(err) => {
                forwarder_stop.store(true, Ordering::Relaxed);
                if let Some(join_handle) = forwarder {
                    match join_handle.join() {
                        Ok(Ok(())) => {}
                        Ok(Err(join_err)) => return Err(join_err),
                        Err(_) => return Err("Live event forwarder thread panicked".to_string()),
                    }
                }
                return Err(format!(
                    "Failed to execute test '{}' from '{}': {err}",
                    case.test_name, case.test_bin
                ));
            }
        };
        let duration_ms = started.elapsed().as_millis() as u64;
        forwarder_stop.store(true, Ordering::Relaxed);
        if let Some(join_handle) = forwarder {
            match join_handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(err)) => return Err(err),
                Err(_) => return Err("Live event forwarder thread panicked".to_string()),
            }
        }

        handle.event(
            "runner.test_command",
            "ok",
            serde_json::json!({
                "binary": case.test_bin,
                "name": case.test_name,
                "target": effective_target,
                "tags": case.tags,
                "timeout_ms": case.timeout_ms,
                "duration_ms": duration_ms,
            }),
        );

        // Merge sub-process events (intent, session, env, driver) from the
        // per-test live JSONL into the suite report handle.
        inject_live_events_into_handle(&mut handle, &raw_live_stream_path);

        if output.status.success() {
            handle.event(
                "test.result",
                "ok",
                serde_json::json!({ "outcome": "passed" }),
            );
            if let Some(config) = &live_stream_config {
                uto_runner::append_live_event(
                    &config.file_path,
                    &uto_runner::LiveEventEnvelope::test_finished(
                        config,
                        "passed",
                        None,
                        duration_ms,
                    ),
                )?;
            }
            suite.record_test(handle, "passed", None);
            continue;
        }

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let excerpt = if !stderr.trim().is_empty() {
            truncate_for_report(&stderr, 4000)
        } else {
            truncate_for_report(&stdout, 4000)
        };

        handle.event(
            "runner.test_command",
            "failed",
            serde_json::json!({
                "binary": case.test_bin,
                "name": case.test_name,
                "target": effective_target,
                "tags": case.tags,
                "timeout_ms": case.timeout_ms,
                "exit_code": output.status.code(),
                "duration_ms": duration_ms,
                "excerpt": excerpt,
            }),
        );

        // Merge sub-process events from the per-test live JSONL (partial run).
        inject_live_events_into_handle(&mut handle, &raw_live_stream_path);

        suite.record_test(
            handle,
            "failed",
            Some(format!(
                "cargo test failed for {}::{}",
                case.test_bin, case.test_name
            )),
        );

        if let Some(config) = &live_stream_config {
            uto_runner::append_live_event(
                &config.file_path,
                &uto_runner::LiveEventEnvelope::test_finished(
                    config,
                    "failed",
                    Some(format!(
                        "cargo test failed for {}::{}",
                        case.test_bin, case.test_name
                    )),
                    duration_ms,
                ),
            )?;
        }
    }

    suite.finish();
    suite.emit();

    let html_path = PathBuf::from(report_path).with_extension("html");
    uto_reporter::write_suite_html(suite.payload(), &html_path)
        .map_err(|e| format!("Failed to write HTML suite report: {e}"))?;

    if suite.payload().summary.failed > 0 {
        return Err("One or more tests failed".to_string());
    }

    Ok(())
}

pub mod init {
    use super::*;

    pub fn run(args: &[String]) -> Result<(), String> {
        let parsed = crate::parsing::parse_init_args(
            args,
            &std::env::current_dir().map_err(|e| e.to_string())?,
        )?;

        // Validate uto_root if provided (development mode)
        if let Some(ref uto_root) = parsed.uto_root {
            if !uto_root.join("uto-core").exists() {
                return Err(format!(
                    "Invalid --uto-root '{}': expected uto-core directory at {}",
                    uto_root.display(),
                    uto_root.join("uto-core").display()
                ));
            }
        }

        if parsed.project_dir.exists() {
            let mut entries = fs::read_dir(&parsed.project_dir).map_err(|e| e.to_string())?;
            if entries.next().is_some() {
                return Err(format!(
                    "Refusing to initialize non-empty directory: {}",
                    parsed.project_dir.display()
                ));
            }
        }

        fs::create_dir_all(parsed.project_dir.join("src")).map_err(|e| e.to_string())?;
        fs::create_dir_all(parsed.project_dir.join("tests")).map_err(|e| e.to_string())?;
        fs::create_dir_all(parsed.project_dir.join(".uto/reports")).map_err(|e| e.to_string())?;

        let project_name = parsed
            .project_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("uto-project")
            .to_string();

        // Generate Cargo.toml based on mode
        let cargo_toml_content = match &parsed.uto_root {
            Some(uto_root) => templates::cargo_toml_path_deps(&project_name, uto_root),
            None => templates::cargo_toml_crates_io(&project_name),
        };

        let config = UtoProjectConfig {
            schema_version: PROJECT_SCHEMA_VERSION.to_string(),
            project_name: project_name.clone(),
            tests_dir: "tests".to_string(),
            default_target: parsed.template.clone(),
            report_dir: ".uto/reports".to_string(),
            uto_root: parsed
                .uto_root
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "crates.io".to_string()),
            report_schema: DEFAULT_REPORT_SCHEMA_VERSION.to_string(),
            framework_version: Some(DEFAULT_FRAMEWORK_VERSION.to_string()),
        };

        config.validate()?;

        write_json(parsed.project_dir.join("uto.json"), &config)?;
        fs::write(parsed.project_dir.join("Cargo.toml"), cargo_toml_content)
            .map_err(|e| e.to_string())?;
        fs::write(
            parsed.project_dir.join("src/lib.rs"),
            templates::project_lib_rs(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            parsed.project_dir.join("tests/web_example.rs"),
            templates::web_test_example(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            parsed.project_dir.join("tests/mobile_example.rs"),
            templates::mobile_test_example(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            parsed.project_dir.join("README.md"),
            templates::project_readme(&config),
        )
        .map_err(|e| e.to_string())?;

        println!(
            "Created UTO template project at {} (template: {})",
            parsed.project_dir.display(),
            parsed.template
        );
        println!("Next: uto run --project {}", parsed.project_dir.display());
        Ok(())
    }
}

pub mod run {
    use super::*;

    pub fn run(args: &[String]) -> Result<(), String> {
        let parsed = crate::parsing::parse_run_args(args)?;
        let config = crate::config::load_project_config(&parsed.project)?;

        let effective_target = parsed.target.unwrap_or(config.default_target.clone());
        let effective_target = crate::parsing::normalize_target(&effective_target)?;
        let report_dir = config.report_dir.clone();
        let report_path = parsed
            .report_json
            .unwrap_or_else(|| parsed.project.join(report_dir).join("last-run.json"));
        let report_path = absolutize_path(report_path)?;
        let live_events_jsonl = match parsed.live_events_jsonl {
            Some(path) => Some(absolutize_path(path)?),
            None => None,
        };

        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        if has_legacy_runner(&parsed.project) {
            if parsed.test_bin.is_some()
                || parsed.test_name.is_some()
                || live_events_jsonl.is_some()
            {
                return Err(
                    "Single-test selection and live event streaming are not supported in legacy runner mode. \
                     Remove src/bin/uto_project_runner.rs to enable CLI-owned execution."
                        .to_string(),
                );
            }
            enforce_legacy_runner_gate(&config, &parsed.project)?;
            return run_legacy_project_runner(
                &parsed.project,
                effective_target.as_str(),
                &report_path,
                parsed.driver_trace,
            );
        }

        run_cli_owned_tests(
            &parsed.project,
            &config,
            effective_target.as_str(),
            &report_path,
            live_events_jsonl.as_deref(),
            parsed.driver_trace,
            parsed.test_bin.as_deref(),
            parsed.test_name.as_deref(),
        )
    }
}

pub mod report {
    use std::fs;

    pub fn run(args: &[String]) -> Result<(), String> {
        let parsed = crate::parsing::parse_report_args(args)?;
        let config = crate::config::load_project_config(&parsed.project)?;

        let report_path = parsed
            .input
            .unwrap_or_else(|| parsed.project.join(config.report_dir).join("last-run.json"));

        if !report_path.exists() {
            return Err(format!(
                "Report file not found: {}. Run `uto run --project {}` first or pass --input <report-path>",
                report_path.display(),
                parsed.project.display()
            ));
        }

        let report_value: serde_json::Value = crate::io::read_json(report_path.clone())?;
        let report = crate::config::parse_report_json(&report_value)?;

        let html_path = if parsed.html {
            let p = parsed
                .html_output
                .unwrap_or_else(|| report_path.with_extension("html"));
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            Some(p)
        } else {
            None
        };

        match report {
            crate::config::ParsedReport::Single(r) => {
                println!("UTO Report Summary");
                println!("==================");
                println!("Schema: {}", r.schema_version);
                println!("Run ID: {}", r.run_id);
                println!("Mode: {}", r.mode);
                println!("Status: {}", r.status);
                println!(
                    "Duration (ms): {}",
                    r.timeline.duration_ms.unwrap_or_default()
                );
                println!("Events: {}", r.events.len());
                println!("Report: {}", report_path.display());
                if let Some(hp) = html_path {
                    uto_reporter::write_report_html(&r, &hp)?;
                    println!("HTML: {}", hp.display());
                }
            }
            crate::config::ParsedReport::Suite(s) => {
                println!("UTO Suite Report Summary");
                println!("========================");
                println!("Schema: {}", s.schema_version);
                println!("Suite ID: {}", s.suite_id);
                println!("Mode: {}", s.mode);
                println!("Status: {}", s.status);
                println!(
                    "Duration (ms): {}",
                    s.timeline.duration_ms.unwrap_or_default()
                );
                println!(
                    "Tests: {} total | {} passed | {} failed | {} skipped",
                    s.summary.total, s.summary.passed, s.summary.failed, s.summary.skipped
                );
                println!("Report: {}", report_path.display());
                if let Some(hp) = html_path {
                    uto_reporter::write_suite_html(&s, &hp)?;
                    println!("HTML: {}", hp.display());
                }
            }
        }

        Ok(())
    }
}

pub mod ui {
    /// Run the `uto ui` interactive server.
    ///
    /// Starts an embedded HTTP + WebSocket server that serves the UTO UI SPA and
    /// optionally replays a saved `uto-suite/v1` or `uto-report/v1` artifact.
    pub fn run(args: &[String]) -> Result<(), String> {
        let parsed = crate::parsing::parse_ui_args(args)?;

        // If the project directory contains uto.json, validate it for early feedback.
        // If not present (e.g. user only passed --report), skip silently and proceed.
        if parsed.project.join("uto.json").exists() {
            crate::config::load_project_config(&parsed.project)?;
        }

        let opts = uto_ui::UiOptions {
            project: parsed.project,
            port: parsed.port,
            open: parsed.open,
            watch: parsed.watch,
            report: parsed.report,
            studio: parsed.studio,
        };

        uto_ui::start_server_sync(opts)
    }
}
