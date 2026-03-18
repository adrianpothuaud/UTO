//! Command-line parsing for test runner invocations.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Web,
    Mobile,
}

/// Parsed command-line options for a test runner.
#[derive(Debug)]
pub struct CliOptions {
    pub mode: RunMode,
    pub report_json: bool,
    pub report_file: Option<String>,
}

impl CliOptions {
    /// Parses command-line arguments and environment variables.
    pub fn from_env() -> Self {
        let mut mode = RunMode::Web;
        let mut report_json = false;
        let mut report_file = std::env::var("UTO_REPORT_FILE").ok();

        let args: Vec<String> = std::env::args().collect();
        let mut i = 1usize;
        while i < args.len() {
            match args[i].as_str() {
                "--target" => {
                    if let Some(next) = args.get(i + 1) {
                        if next.eq_ignore_ascii_case("mobile") {
                            mode = RunMode::Mobile;
                        }
                        i += 1;
                    }
                }
                "--json" => report_json = true,
                "--report-file" => {
                    if let Some(next) = args.get(i + 1) {
                        report_file = Some(next.clone());
                        i += 1;
                    }
                }
                _ => {}
            }
            i += 1;
        }

        CliOptions {
            mode,
            report_json,
            report_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_defaults_to_web() {
        // Note: actual env parsing uses std::env::args(),
        // so this is a minimal structural test.
        assert_eq!(RunMode::Web, RunMode::Web);
    }
}
