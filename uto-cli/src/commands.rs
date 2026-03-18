//! CLI command implementations.

use std::fs;
use std::process::Command;

use crate::config::{UtoProjectConfig, PROJECT_SCHEMA_VERSION, REPORT_SCHEMA_VERSION};
use crate::io::write_json;
use crate::templates;

pub mod init {
    use super::*;

    pub fn run(args: &[String]) -> Result<(), String> {
        let parsed = crate::parsing::parse_init_args(
            args,
            &std::env::current_dir().map_err(|e| e.to_string())?,
        )?;

        if !parsed.uto_root.join("uto-core").exists() {
            return Err(format!(
                "Invalid --uto-root '{}': expected uto-core directory at {}",
                parsed.uto_root.display(),
                parsed.uto_root.join("uto-core").display()
            ));
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

        fs::create_dir_all(parsed.project_dir.join("src/bin")).map_err(|e| e.to_string())?;
        fs::create_dir_all(parsed.project_dir.join("tests")).map_err(|e| e.to_string())?;
        fs::create_dir_all(parsed.project_dir.join(".uto/reports")).map_err(|e| e.to_string())?;

        let project_name = parsed
            .project_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("uto-project")
            .to_string();

        let config = UtoProjectConfig {
            schema_version: PROJECT_SCHEMA_VERSION.to_string(),
            project_name: project_name.clone(),
            tests_dir: "tests".to_string(),
            default_target: parsed.template.clone(),
            report_dir: ".uto/reports".to_string(),
            uto_root: parsed.uto_root.display().to_string(),
            report_schema: REPORT_SCHEMA_VERSION.to_string(),
        };

        config.validate()?;

        write_json(parsed.project_dir.join("uto.json"), &config)?;
        fs::write(
            parsed.project_dir.join("Cargo.toml"),
            templates::cargo_toml(&project_name, &parsed.uto_root),
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            parsed.project_dir.join("src/bin/uto_project_runner.rs"),
            templates::project_runner_rs(),
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
        crate::config::validate_project_runner(&parsed.project)?;

        let effective_target = parsed.target.unwrap_or(config.default_target.clone());
        let effective_target = crate::parsing::normalize_target(&effective_target)?;
        let report_path = parsed
            .report_json
            .unwrap_or_else(|| parsed.project.join(config.report_dir).join("last-run.json"));

        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&parsed.project)
            .arg("run")
            .arg("--bin")
            .arg("uto_project_runner")
            .arg("--")
            .arg("--target")
            .arg(effective_target.as_str())
            .arg("--json")
            .arg("--report-file")
            .arg(report_path.display().to_string())
            .env("RUST_LOG", "info");

        if parsed.driver_trace {
            cmd.env("UTO_DRIVER_TRACE", "1");
        }

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to run project runner: {e}"))?;

        if !status.success() {
            return Err("Project runner exited with non-zero status".to_string());
        }

        Ok(())
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

        println!("UTO Report Summary");
        println!("==================");
        println!("Schema: {}", report.schema_version);
        println!("Run ID: {}", report.run_id);
        println!("Mode: {}", report.mode);
        println!("Status: {}", report.status);
        println!(
            "Duration (ms): {}",
            report.timeline.duration_ms.unwrap_or_default()
        );
        println!("Events: {}", report.events.len());
        println!("Report: {}", report_path.display());

        if parsed.html {
            let html_path = parsed
                .html_output
                .unwrap_or_else(|| report_path.with_extension("html"));
            if let Some(parent) = html_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            uto_runner::write_report_html(&report, &html_path)?;
            println!("HTML: {}", html_path.display());
        }

        Ok(())
    }
}
