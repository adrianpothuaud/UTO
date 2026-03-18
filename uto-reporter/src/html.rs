//! Deterministic HTML rendering for uto-report/v1 artifacts.

use std::path::Path;

use crate::schema::UtoReportV1;

/// Renders an offline-readable HTML document for a uto-report/v1 payload.
pub fn render_report_html(report: &UtoReportV1) -> String {
    let status_class = status_class(&report.status);
    let error_block = report
        .error
        .as_ref()
        .map(|err| {
            format!(
                "<section class=\"panel panel-error\"><h2>Error</h2><pre>{}</pre></section>",
                escape_html(err)
            )
        })
        .unwrap_or_default();

    let mut event_rows = String::new();
    for (index, event) in report.events.iter().enumerate() {
        let detail_json = serde_json::to_string_pretty(&event.detail)
            .unwrap_or_else(|_| "\"<invalid detail json>\"".to_string());
        event_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td><pre>{}</pre></td></tr>",
            index + 1,
            escape_html(&event.stage),
            escape_html(&event.status),
            escape_html(&detail_json)
        ));
    }

    if event_rows.is_empty() {
        event_rows.push_str(
            "<tr><td colspan=\"4\" class=\"empty\">No events recorded for this run.</td></tr>",
        );
    }

    let schema = escape_html(&report.schema_version);
    let framework = escape_html(&report.framework);
    let run_id = escape_html(&report.run_id);
    let mode = escape_html(&report.mode);
    let status = escape_html(&report.status);
    let started = report.timeline.started_at_unix_ms;
    let finished = optional_u64(report.timeline.finished_at_unix_ms);
    let duration = optional_u64(report.timeline.duration_ms);
    let event_count = report.events.len();

    format!(
        r#"<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <title>UTO Report {run_id}</title>
  <style>
    :root {{
      --bg: #f5f3ef;
      --panel: #ffffff;
      --ink: #1f2429;
      --muted: #5a646e;
      --line: #d7d2ca;
      --ok: #1d7f4f;
      --warn: #af702d;
      --bad: #a93030;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      background: radial-gradient(circle at 15% 0, #fdfbf7, var(--bg));
      color: var(--ink);
      font-family: \"Iowan Old Style\", \"Georgia\", serif;
      line-height: 1.45;
    }}
    .wrap {{ max-width: 1100px; margin: 32px auto; padding: 0 18px 32px; }}
    h1, h2 {{ margin: 0 0 12px; letter-spacing: 0.2px; }}
    h1 {{ font-size: 2rem; }}
    h2 {{ font-size: 1.2rem; }}
    .panel {{
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 16px;
      margin: 14px 0;
      box-shadow: 0 2px 10px rgba(31, 36, 41, 0.04);
    }}
    .meta {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 10px 16px;
      margin-top: 10px;
    }}
    .meta-item {{ border-top: 1px solid var(--line); padding-top: 8px; }}
    .label {{ color: var(--muted); font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.08em; }}
    .value {{ font-size: 1.03rem; margin-top: 2px; word-break: break-word; }}
    .status {{
      display: inline-block;
      border-radius: 999px;
      padding: 3px 10px;
      font-size: 0.85rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }}
    .status-ok {{ background: #deefe6; color: var(--ok); }}
    .status-running {{ background: #f4e8d5; color: var(--warn); }}
    .status-fail {{ background: #f4dddd; color: var(--bad); }}
    .panel-error {{ border-color: #e5b5b5; background: #fff9f9; }}
    table {{ width: 100%; border-collapse: collapse; }}
    th, td {{ border-top: 1px solid var(--line); padding: 10px 8px; vertical-align: top; text-align: left; }}
    th {{ border-top: none; font-size: 0.85rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.08em; }}
    td.empty {{ color: var(--muted); text-align: center; padding: 18px 10px; }}
    pre {{
      margin: 0;
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", \"Courier New\", monospace;
      font-size: 0.83rem;
      white-space: pre-wrap;
      overflow-wrap: anywhere;
    }}
  </style>
</head>
<body>
  <div class=\"wrap\">
    <section class=\"panel\">
      <h1>UTO Execution Report</h1>
      <span class=\"status {status_class}\">{status}</span>
      <div class=\"meta\">
        <div class=\"meta-item\"><div class=\"label\">Schema</div><div class=\"value\">{schema}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Framework</div><div class=\"value\">{framework}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Run ID</div><div class=\"value\">{run_id}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Mode</div><div class=\"value\">{mode}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Started (unix ms)</div><div class=\"value\">{started}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Finished (unix ms)</div><div class=\"value\">{finished}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Duration (ms)</div><div class=\"value\">{duration}</div></div>
        <div class=\"meta-item\"><div class=\"label\">Event Count</div><div class=\"value\">{event_count}</div></div>
      </div>
    </section>
    {error_block}
    <section class=\"panel\">
      <h2>Events</h2>
      <table>
        <thead>
          <tr><th>#</th><th>Stage</th><th>Status</th><th>Detail</th></tr>
        </thead>
        <tbody>
          {event_rows}
        </tbody>
      </table>
    </section>
  </div>
</body>
</html>"#,
        run_id = run_id,
        schema = schema,
        framework = framework,
        mode = mode,
        status = status,
        status_class = status_class,
        started = started,
        finished = finished,
        duration = duration,
        event_count = event_count,
        error_block = error_block,
        event_rows = event_rows
    )
}

/// Writes an HTML report artifact to the requested file path.
pub fn write_report_html(report: &UtoReportV1, file_path: &Path) -> Result<(), String> {
    let html = render_report_html(report);
    std::fs::write(file_path, html)
        .map_err(|e| format!("Failed to write HTML report at {}: {e}", file_path.display()))
}

fn status_class(status: &str) -> &'static str {
    if status.eq_ignore_ascii_case("passed") || status.eq_ignore_ascii_case("ok") {
        "status-ok"
    } else if status.eq_ignore_ascii_case("running") {
        "status-running"
    } else {
        "status-fail"
    }
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ReportEvent;

    #[test]
    fn render_report_html_contains_core_fields() {
        let mut report = UtoReportV1::new("run-77".to_string(), "web".to_string(), 1000);
        report.status = "passed".to_string();
        report.timeline.finished_at_unix_ms = Some(1200);
        report.timeline.duration_ms = Some(200);
        report.events.push(ReportEvent {
            stage: "session.goto".to_string(),
            status: "ok".to_string(),
            detail: serde_json::json!({"url": "https://example.com"}),
        });

        let html = render_report_html(&report);
        assert!(html.contains("UTO Execution Report"));
        assert!(html.contains("run-77"));
        assert!(html.contains("session.goto"));
    }

    #[test]
    fn render_report_html_escapes_untrusted_text() {
        let mut report = UtoReportV1::new("run-<1>".to_string(), "web".to_string(), 1000);
        report.status = "failed".to_string();
        report.error = Some("<script>alert('x')</script>".to_string());

        let html = render_report_html(&report);
        assert!(html.contains("run-&lt;1&gt;"));
        assert!(html.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"));
    }
}
