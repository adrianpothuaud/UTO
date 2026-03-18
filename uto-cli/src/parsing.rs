//! Command-line argument parsing.

use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub struct InitArgs {
    pub project_dir: PathBuf,
    pub template: String,
    pub uto_root: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunArgs {
    pub project: PathBuf,
    pub target: Option<String>,
    pub report_json: Option<PathBuf>,
    pub driver_trace: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReportArgs {
    pub project: PathBuf,
    pub input: Option<PathBuf>,
    pub html: bool,
    pub html_output: Option<PathBuf>,
}

pub fn parse_init_args(args: &[String], current_dir: &Path) -> Result<InitArgs, String> {
    if args.is_empty() {
        return Err("init requires <project-dir>".to_string());
    }

    let project_token = args[0].as_str();
    if project_token.starts_with('-') {
        return Err("init requires <project-dir> as the first argument".to_string());
    }

    let project_dir = PathBuf::from(project_token);
    let mut template = "web".to_string();
    let mut uto_root = current_dir.to_path_buf();

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => {
                let value = get_flag_value(args, i, "--template")?;
                template = normalize_target(value)?;
                i += 1;
            }
            "--uto-root" => {
                let value = get_flag_value(args, i, "--uto-root")?;
                uto_root = PathBuf::from(value);
                i += 1;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown init option: {flag}"));
            }
            extra => {
                return Err(format!(
                    "Unexpected positional argument for init: {extra}. Expected only <project-dir>"
                ));
            }
        }
        i += 1;
    }

    Ok(InitArgs {
        project_dir,
        template,
        uto_root,
    })
}

pub fn parse_run_args(args: &[String]) -> Result<RunArgs, String> {
    let mut project: Option<PathBuf> = None;
    let mut target: Option<String> = None;
    let mut report_json: Option<PathBuf> = None;
    let mut driver_trace = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                let value = get_flag_value(args, i, "--project")?;
                project = Some(PathBuf::from(value));
                i += 1;
            }
            "--target" => {
                let value = get_flag_value(args, i, "--target")?;
                target = Some(normalize_target(value)?);
                i += 1;
            }
            "--report-json" => {
                let value = get_flag_value(args, i, "--report-json")?;
                report_json = Some(PathBuf::from(value));
                i += 1;
            }
            "--driver-trace" => {
                driver_trace = true;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown run option: {flag}"));
            }
            value => {
                return Err(format!("Unexpected argument for run: {value}"));
            }
        }
        i += 1;
    }

    let project = project.ok_or_else(|| "run requires --project <project-dir>".to_string())?;

    Ok(RunArgs {
        project,
        target,
        report_json,
        driver_trace,
    })
}

pub fn parse_report_args(args: &[String]) -> Result<ReportArgs, String> {
    let mut project: Option<PathBuf> = None;
    let mut input: Option<PathBuf> = None;
    let mut html = false;
    let mut html_output: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                let value = get_flag_value(args, i, "--project")?;
                project = Some(PathBuf::from(value));
                i += 1;
            }
            "--input" => {
                let value = get_flag_value(args, i, "--input")?;
                input = Some(PathBuf::from(value));
                i += 1;
            }
            "--html" => {
                html = true;
            }
            "--html-output" => {
                let value = get_flag_value(args, i, "--html-output")?;
                html = true;
                html_output = Some(PathBuf::from(value));
                i += 1;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown report option: {flag}"));
            }
            value => {
                return Err(format!("Unexpected argument for report: {value}"));
            }
        }
        i += 1;
    }

    let project = project.ok_or_else(|| "report requires --project <project-dir>".to_string())?;
    Ok(ReportArgs {
        project,
        input,
        html,
        html_output,
    })
}

pub fn get_flag_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| format!("{flag} requires a value"))?;
    if value.starts_with('-') {
        return Err(format!("{flag} requires a value"));
    }
    Ok(value)
}

pub fn normalize_target(target: &str) -> Result<String, String> {
    if target.eq_ignore_ascii_case("web") {
        return Ok("web".to_string());
    }
    if target.eq_ignore_ascii_case("mobile") {
        return Ok("mobile".to_string());
    }
    Err(format!(
        "Invalid target/template '{}'. Supported values: web, mobile",
        target
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_target_accepts_web_mobile_case_insensitive() {
        assert_eq!(normalize_target("web").expect("web"), "web");
        assert_eq!(normalize_target("MOBILE").expect("mobile"), "mobile");
    }

    #[test]
    fn normalize_target_rejects_unknown_target() {
        let err = normalize_target("desktop").expect_err("should fail");
        assert!(err.contains("Supported values: web, mobile"));
    }

    #[test]
    fn parse_init_args_defaults_and_flags() {
        let args = vec![
            "sample".to_string(),
            "--template".to_string(),
            "mobile".to_string(),
            "--uto-root".to_string(),
            "/repo".to_string(),
        ];
        let parsed = parse_init_args(&args, Path::new("/cwd")).expect("parse init args");

        assert_eq!(parsed.project_dir, PathBuf::from("sample"));
        assert_eq!(parsed.template, "mobile");
        assert_eq!(parsed.uto_root, PathBuf::from("/repo"));
    }

    #[test]
    fn parse_run_args_requires_project() {
        let err = parse_run_args(&[]).expect_err("should fail");
        assert!(err.contains("run requires --project"));
    }

    #[test]
    fn parse_run_args_rejects_unknown_flag() {
        let args = vec![
            "--project".to_string(),
            "demo".to_string(),
            "--bad".to_string(),
        ];
        let err = parse_run_args(&args).expect_err("should fail");
        assert!(err.contains("Unknown run option"));
    }

    #[test]
    fn parse_init_args_rejects_unknown_option() {
        let args = vec!["sample".to_string(), "--nope".to_string()];
        let err = parse_init_args(&args, Path::new("/cwd")).expect_err("should fail");
        assert!(err.contains("Unknown init option"));
    }

    #[test]
    fn get_flag_value_requires_explicit_value() {
        let args = vec!["--project".to_string(), "--target".to_string()];
        let err = get_flag_value(&args, 0, "--project").expect_err("should fail");
        assert!(err.contains("requires a value"));
    }

    #[test]
    fn parse_report_args_requires_project() {
        let err = parse_report_args(&[]).expect_err("should fail");
        assert!(err.contains("report requires --project"));
    }

    #[test]
    fn parse_report_args_supports_html_flags() {
        let args = vec![
            "--project".to_string(),
            "demo".to_string(),
            "--html".to_string(),
            "--html-output".to_string(),
            "out/report.html".to_string(),
        ];

        let parsed = parse_report_args(&args).expect("parse report args");
        assert_eq!(parsed.project, PathBuf::from("demo"));
        assert!(parsed.html);
        assert_eq!(parsed.html_output, Some(PathBuf::from("out/report.html")));
    }
}
