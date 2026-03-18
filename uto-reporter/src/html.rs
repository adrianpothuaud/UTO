//! Deterministic HTML rendering for uto-report/v1 and uto-suite/v1 artifacts.

use std::path::Path;

use crate::schema::{ReportEvent, UtoReportV1};
use crate::schema::{TestCaseResult, UtoSuiteReportV1};

// ---------------------------------------------------------------------------
// Embedded CSS — no external dependencies, dark/light themes
// ---------------------------------------------------------------------------

const STYLE: &str = r#"
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
html{scroll-behavior:smooth;-webkit-text-size-adjust:100%}
:root{
    --bg:#f4f8fb;--bg2:#ffffff;--bg3:#edf3f8;--bg4:#e3edf6;
    --bd:#c6d4e1;--bd2:#d7e2eb;
    --tx:#14212e;--tx2:#46586a;--tx3:#6f8091;
    --ok:#0f7b42;--ok-bg:#d8f4e5;--ok-bd:#42b883;
    --fail:#b22b22;--fail-bg:#fde8e6;--fail-bd:#ef6f6a;
    --warn:#946500;--warn-bg:#fff5cf;--warn-bd:#d6a100;
    --skip:#4f6072;--skip-bg:#ecf2f7;
    --acc:#0f5db8;--acc-bg:#ddeeff;--acc-bd:#4c8ed8;
    --acc2:#0f5db81a;
    --radius:10px;
    --radius-sm:7px;
    --shadow:0 9px 22px rgba(26,57,88,.08),0 1px 2px rgba(26,57,88,.07);
    --font:"Avenir Next","Segoe UI",Helvetica,Arial,sans-serif;
    --mono:ui-monospace,SFMono-Regular,"SF Mono",Menlo,Consolas,"Liberation Mono",monospace;
}
[data-theme="dark"]{
    --bg:#0f141b;--bg2:#171f29;--bg3:#1c2632;--bg4:#223142;
    --bd:#2f3e4f;--bd2:#263343;
    --tx:#e6edf5;--tx2:#9ab0c6;--tx3:#71879c;
    --ok:#44c585;--ok-bg:#143326;--ok-bd:#2d9665;
    --fail:#ff7e74;--fail-bg:#381916;--fail-bd:#c34943;
    --warn:#efc14e;--warn-bg:#332712;--warn-bd:#a67b1e;
    --skip:#8ea2b7;--skip-bg:#202a36;
    --acc:#6bb2ff;--acc-bg:#102943;--acc-bd:#2e75be;
    --acc2:#6bb2ff1f;
    --shadow:0 10px 24px rgba(0,0,0,.35),0 1px 2px rgba(0,0,0,.4);
}
body{background:radial-gradient(circle at 12% -5%,var(--bg4),transparent 46%),radial-gradient(circle at 88% -10%,var(--acc2),transparent 40%),var(--bg);color:var(--tx);font-family:var(--font);font-size:14px;line-height:1.5;min-height:100vh}

/* --- Layout --- */
.topbar{position:sticky;top:0;z-index:220;background:color-mix(in srgb,var(--bg2) 88%,transparent);backdrop-filter:blur(8px);border-bottom:1px solid var(--bd);padding:0 20px}
.wrap-h{max-width:1200px;margin:0 auto;display:flex;align-items:center;gap:10px;height:56px;min-width:0}
.brand{font-weight:800;font-size:.72rem;letter-spacing:.16em;text-transform:uppercase;color:var(--tx2);flex-shrink:0}
.sep{width:1px;height:18px;background:var(--bd);flex-shrink:0}
.topbar-title{font-size:.9rem;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;flex:1;min-width:0}
.spacer{flex:1}
.theme-btn{width:32px;height:32px;border:1px solid var(--bd);border-radius:var(--radius-sm);background:var(--bg2);cursor:pointer;display:flex;align-items:center;justify-content:center;font-size:.95rem;color:var(--tx2);flex-shrink:0;transition:all .14s}
.theme-btn:hover{background:var(--bg3);transform:translateY(-1px)}

.main{max-width:1200px;margin:0 auto;padding:20px 20px 44px}

.hero{background:linear-gradient(140deg,var(--bg2),color-mix(in srgb,var(--bg2) 75%,var(--bg3)));border:1px solid var(--bd);border-radius:var(--radius);padding:18px 18px 16px;margin-bottom:14px;box-shadow:var(--shadow);position:relative;overflow:hidden}
.hero::after{content:"";position:absolute;right:-42px;top:-50px;width:170px;height:170px;border-radius:50%;background:var(--acc2)}
.hero-kicker{font-size:.69rem;font-weight:700;letter-spacing:.11em;text-transform:uppercase;color:var(--tx2);margin-bottom:6px}
.hero-title{font-size:1.15rem;font-weight:700;line-height:1.22;margin-bottom:9px;position:relative;z-index:1}
.hero-meta{display:flex;gap:7px;flex-wrap:wrap;position:relative;z-index:1}

/* --- Badges --- */
.badge{display:inline-flex;align-items:center;padding:2px 10px;border-radius:20px;font-size:.69rem;font-weight:700;letter-spacing:.05em;text-transform:uppercase;white-space:nowrap;flex-shrink:0}
.b-ok{background:var(--ok-bg);color:var(--ok);border:1px solid var(--ok-bd)}
.b-fail{background:var(--fail-bg);color:var(--fail);border:1px solid var(--fail-bd)}
.b-warn{background:var(--warn-bg);color:var(--warn);border:1px solid var(--warn-bd)}
.b-skip{background:var(--skip-bg);color:var(--skip);border:1px solid var(--bd)}
.b-run{background:var(--acc-bg);color:var(--acc);border:1px solid var(--acc-bd)}

/* --- Summary cards --- */
.cards{display:flex;gap:10px;flex-wrap:wrap;margin-bottom:16px}
.card{flex:1;min-width:120px;background:var(--bg2);border:1px solid var(--bd);border-radius:var(--radius);padding:14px 16px;box-shadow:var(--shadow)}
.card-lbl{font-size:.7rem;color:var(--tx2);text-transform:uppercase;letter-spacing:.08em;margin-bottom:5px}
.card-val{font-size:1.5rem;font-weight:700;color:var(--tx);font-variant-numeric:tabular-nums;line-height:1.1}
.card-val.ok{color:var(--ok)}.card-val.fail{color:var(--fail)}.card-val.warn{color:var(--warn)}

/* --- Meta grid --- */
.meta-grid{background:var(--bg2);border:1px solid var(--bd);border-radius:var(--radius);display:grid;grid-template-columns:repeat(auto-fill,minmax(190px,1fr));margin-bottom:16px;overflow:hidden;box-shadow:var(--shadow)}
.mc{padding:11px 14px;border-bottom:1px solid var(--bd2);border-right:1px solid var(--bd2)}
.mc:last-child{border-right:none}
.mc-k{font-size:.68rem;text-transform:uppercase;letter-spacing:.08em;color:var(--tx2);margin-bottom:4px}
.mc-v{font-size:.82rem;font-family:var(--mono);word-break:break-all;color:var(--tx)}

/* --- Panel container --- */
.panel{background:var(--bg2);border:1px solid var(--bd);border-radius:var(--radius);margin-bottom:16px;overflow:hidden;box-shadow:var(--shadow)}
.panel-hd{display:flex;align-items:center;gap:8px;padding:10px 14px;border-bottom:1px solid var(--bd2);background:var(--bg3)}
.panel-title{font-weight:700;font-size:.84rem;flex:1;letter-spacing:.02em}
.panel-sub{font-size:.74rem;color:var(--tx2)}

/* --- Error block --- */
.err-block{background:var(--fail-bg);border:1px solid var(--fail-bd);border-radius:var(--radius);padding:12px 14px;margin-bottom:16px;font-family:var(--mono);font-size:.82rem;color:var(--fail);white-space:pre-wrap;word-break:break-word;box-shadow:var(--shadow)}

/* --- Tests accordion (suite view) --- */
.suite-controls{display:flex;align-items:center;gap:8px;flex-wrap:wrap;padding:9px 14px;border-bottom:1px solid var(--bd2);background:var(--bg2)}
.suite-status-filters{display:flex;align-items:center;gap:6px;flex-wrap:wrap}
.suite-actions{display:flex;gap:6px;margin-left:auto}
.btn{padding:5px 10px;border-radius:999px;border:1px solid var(--bd);background:var(--bg3);color:var(--tx2);font:600 .74rem var(--font);cursor:pointer;transition:all .12s;white-space:nowrap}
.btn:hover{background:var(--bg4);color:var(--tx)}

.test-item{border:1px solid var(--bd);border-radius:var(--radius-sm);margin-bottom:8px;overflow:hidden;box-shadow:var(--shadow)}
.test-hd{display:flex;align-items:center;gap:8px;padding:10px 14px;cursor:pointer;background:var(--bg3);user-select:none;transition:background .1s}
.test-hd:hover{background:var(--bg4)}
.test-hd.open{border-bottom:1px solid var(--bd)}
.chevron{width:14px;height:14px;color:var(--tx2);flex-shrink:0;transition:transform .15s}
.test-hd.open .chevron{transform:rotate(90deg)}
.test-name{font-weight:650;font-size:.88rem;flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.test-dur{font-size:.75rem;color:var(--tx2);font-family:var(--mono);flex-shrink:0}
.test-body{display:none}
.test-body.open{display:block}
.test-err{background:var(--fail-bg);border-bottom:1px solid var(--fail-bd);padding:10px 14px;font-family:var(--mono);font-size:.8rem;color:var(--fail);white-space:pre-wrap;word-break:break-word}

/* --- Toolbar (search + filters) --- */
.toolbar{display:flex;align-items:center;gap:8px;padding:8px 14px;border-bottom:1px solid var(--bd2);flex-wrap:wrap;background:var(--bg2)}
.search-inp{flex:1;min-width:150px;padding:6px 11px;border:1px solid var(--bd);border-radius:var(--radius-sm);background:var(--bg3);color:var(--tx);font:inherit;font-size:.82rem;outline:none}
.search-inp:focus{border-color:var(--acc);box-shadow:0 0 0 3px var(--acc2)}
.search-inp::placeholder{color:var(--tx3)}
.filters{display:flex;gap:4px;align-items:center;flex-wrap:wrap}
.fb{padding:4px 11px;border:1px solid var(--bd);border-radius:20px;background:transparent;color:var(--tx2);font:.76rem var(--font);font-weight:600;cursor:pointer;transition:all .1s;white-space:nowrap}
.fb:hover{background:var(--bg3);color:var(--tx)}
.fb.active{background:var(--acc2);color:var(--acc);border-color:var(--acc-bd)}
.fb.active-ok{background:var(--ok-bg);color:var(--ok);border-color:var(--ok-bd)}
.fb.active-fail{background:var(--fail-bg);color:var(--fail);border-color:var(--fail-bd)}

/* --- Events table --- */
.tbl-wrap{overflow-x:auto}
table{width:100%;border-collapse:collapse}
th{padding:8px 12px;text-align:left;font-size:.7rem;text-transform:uppercase;letter-spacing:.08em;color:var(--tx2);font-weight:700;background:var(--bg3);border-bottom:1px solid var(--bd2);white-space:nowrap}
td{padding:8px 12px;border-bottom:1px solid var(--bd2);vertical-align:top;font-size:.83rem}
tr:last-child td{border-bottom:none}
tbody tr:nth-child(even) td{background:color-mix(in srgb,var(--bg2) 80%,var(--bg3))}
tr:hover td{background:var(--bg4)}
.c-idx{color:var(--tx2);font-size:.76rem;width:36px}
.c-stage{font-family:var(--mono);font-size:.8rem;min-width:160px}
.c-status{width:82px}
.c-detail{font-family:var(--mono);font-size:.76rem;color:var(--tx2)}
.c-detail pre{white-space:pre-wrap;word-break:break-word;margin:0}
td.empty{text-align:center;color:var(--tx2);padding:20px;font-size:.85rem}

/* --- Footer --- */
footer{border-top:1px solid var(--bd);padding:14px 20px;color:var(--tx2);font-size:.75rem;text-align:center;margin-top:8px}

@media(max-width:750px){
    .topbar{padding:0 12px}.main{padding:12px 12px 32px}
    .cards{flex-direction:column}.meta-grid{grid-template-columns:1fr 1fr}
    .topbar-title{font-size:.82rem}
    .hero{padding:14px 12px}.hero-title{font-size:1rem}
    .suite-actions{margin-left:0}
}
"#;

// ---------------------------------------------------------------------------
// Embedded JavaScript — theme toggle, live search/filter, timestamp format
// ---------------------------------------------------------------------------

const SCRIPT: &str = r#"
(function(){
    window.toggleTheme=function(){
        var h=document.documentElement;
        var dark=h.dataset.theme==='dark'||(h.dataset.theme!=='light'&&window.matchMedia('(prefers-color-scheme:dark)').matches);
        var next=dark?'light':'dark';
        h.dataset.theme=next;
        try{localStorage.setItem('uto-theme',next)}catch(e){}
        _syncIcon();
    };
    function _syncIcon(){
        var h=document.documentElement;
        var dark=h.dataset.theme==='dark'||(h.dataset.theme!=='light'&&window.matchMedia('(prefers-color-scheme:dark)').matches);
        document.querySelectorAll('.theme-icon').forEach(function(el){el.textContent=dark?'\u2600\uFE0F':'\uD83C\uDF19'});
    }
    window.filterRows=function(tid){
        var tbl=document.getElementById(tid);if(!tbl)return;
        var panel=tbl.closest('.events-panel');
        var q=(panel&&panel.querySelector('.search-inp'))?panel.querySelector('.search-inp').value.toLowerCase():'';
        var ab=panel&&panel.querySelector('.fb.active');
        var af=ab?ab.dataset.f:'all';
        var vis=0;
        tbl.querySelectorAll('tbody tr[data-s]').forEach(function(r){
            var show=(!q||r.textContent.toLowerCase().indexOf(q)>=0)&&(af==='all'||r.dataset.s===af);
            r.style.display=show?'':'none';
            if(show)vis++;
        });
        var tb=tbl.querySelector('tbody'),er=tb.querySelector('.erow');
        if(vis===0){if(!er){var tr=document.createElement('tr');tr.className='erow';tr.innerHTML='<td colspan="4" class="empty">No matching events.</td>';tb.appendChild(tr);}}
        else if(er){er.remove();}
        var mc=panel&&panel.querySelector('.match-count');
        if(mc){
            var total=parseInt(mc.dataset.total||'0',10);
            mc.textContent=vis+' / '+total+' shown';
        }
    };
    window.setFilter=function(btn,tid){
        var panel=btn.closest('.events-panel');
        if(panel)panel.querySelectorAll('.fb').forEach(function(b){b.classList.remove('active','active-ok','active-fail');});
        var f=btn.dataset.f;
        btn.classList.add('active'+(f==='ok'?'-ok':f==='failed'?'-fail':''));
        filterRows(tid);
    };
    window.filterTests=function(){
        var qEl=document.getElementById('suite-test-search');
        var q=(qEl?qEl.value:'').toLowerCase();
        var active=document.querySelector('.suite-status-filters .fb.active');
        var sf=active?active.dataset.f:'all';
        var vis=0,total=0;
        document.querySelectorAll('.test-item[data-test-name]').forEach(function(item){
            total++;
            var name=item.dataset.testName||'';
            var st=item.dataset.testStatus||'';
            var show=(!q||name.indexOf(q)>=0)&&(sf==='all'||st===sf);
            item.style.display=show?'':'none';
            if(show)vis++;
        });
        var c=document.getElementById('suite-test-count');
        if(c){c.textContent=vis+' / '+total+' shown';}
    };
    window.setTestFilter=function(btn){
        var wrap=btn.closest('.suite-status-filters');
        if(wrap)wrap.querySelectorAll('.fb').forEach(function(b){b.classList.remove('active','active-ok','active-fail');});
        var f=btn.dataset.f;
        btn.classList.add('active'+(f==='passed'||f==='ok'?'-ok':f==='failed'?'-fail':''));
        filterTests();
    };
    window.toggleAllTests=function(open){
        document.querySelectorAll('.test-item[data-test-name]').forEach(function(item){
            if(item.style.display==='none')return;
            var body=item.querySelector('.test-body');
            var hd=item.querySelector('.test-hd');
            if(!body||!hd)return;
            body.classList.toggle('open',!!open);
            hd.classList.toggle('open',!!open);
        });
    };
    window.toggleSection=function(id){
        var body=document.getElementById(id);if(!body)return;
        var hd=document.querySelector('[data-toggle="'+id+'"]');
        var open=body.classList.toggle('open');
        if(hd)hd.classList.toggle('open',open);
    };
    function fmtMs(ms){if(isNaN(ms))return'-';if(ms<1000)return ms+'ms';if(ms<60000)return(ms/1000).toFixed(1)+'s';return Math.floor(ms/60000)+'m '+Math.round((ms%60000)/1000)+'s';}
    function fmtTs(ms){if(!ms)return'-';return new Date(ms).toLocaleString();}
    document.addEventListener('keydown',function(e){
        if(e.key==='/'&&e.target&&e.target.tagName!=='INPUT'&&e.target.tagName!=='TEXTAREA'){
            var input=document.querySelector('.search-inp');
            if(input){e.preventDefault();input.focus();input.select();}
        }
    });
    document.addEventListener('DOMContentLoaded',function(){
        try{var s=localStorage.getItem('uto-theme');if(s)document.documentElement.dataset.theme=s;}catch(e){}
        _syncIcon();
        document.querySelectorAll('[data-ts]').forEach(function(el){el.textContent=fmtTs(parseInt(el.dataset.ts,10));});
        document.querySelectorAll('[data-ms]').forEach(function(el){el.textContent=fmtMs(parseInt(el.dataset.ms,10));});
        document.querySelectorAll('table[id]').forEach(function(tbl){filterRows(tbl.id);});
        filterTests();
    });
}());
"#;

// ---------------------------------------------------------------------------
// Public rendering API
// ---------------------------------------------------------------------------

/// Renders a full, offline-readable HTML document for a single `uto-report/v1` run.
pub fn render_report_html(report: &UtoReportV1) -> String {
    let title = format!("UTO Report — {}", report.run_id);
    let bc = badge_class(&report.status);

    let cards = [
        ("Mode", escape_html(&report.mode), ""),
        (
            "Status",
            escape_html(&report.status),
            badge_val_class(&report.status),
        ),
        (
            "Duration",
            report
                .timeline
                .duration_ms
                .map(|d| format!("<span data-ms=\"{d}\">{d}ms</span>"))
                .unwrap_or_else(|| "-".to_string()),
            "",
        ),
        ("Events", report.events.len().to_string(), ""),
    ];

    let meta = [
        ("Schema", escape_html(&report.schema_version)),
        ("Framework", escape_html(&report.framework)),
        ("Run ID", escape_html(&report.run_id)),
        ("Mode", escape_html(&report.mode)),
        ("Status", escape_html(&report.status)),
        (
            "Started",
            format!(
                "<span data-ts=\"{}\">{}</span>",
                report.timeline.started_at_unix_ms, report.timeline.started_at_unix_ms
            ),
        ),
        (
            "Duration",
            report
                .timeline
                .duration_ms
                .map(|d| format!("<span data-ms=\"{d}\">{d}ms</span>"))
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];

    let mut buf = String::with_capacity(32_768);
    buf.push_str(&head_html(&title));
    buf.push_str(&topbar_html(
        "UTO",
        "Execution Report",
        &report.run_id,
        &report.status,
        bc,
    ));
    buf.push_str("<div class=\"main\">");
    buf.push_str(&hero_html(
        "Single Execution",
        &report.run_id,
        &report.status,
        report.timeline.started_at_unix_ms,
        report.timeline.duration_ms,
        bc,
    ));
    buf.push_str(&cards_html(&cards));
    buf.push_str(&meta_grid_html(&meta));
    if let Some(err) = &report.error {
        buf.push_str(&error_block_html(err));
    }
    buf.push_str(&events_panel_html(&report.events, "evt-main"));
    buf.push_str("</div>");
    buf.push_str(FOOTER_HTML);
    buf
}

/// Renders a full, offline-readable HTML document for a `uto-suite/v1` suite run.
pub fn render_suite_html(suite: &UtoSuiteReportV1) -> String {
    let title = format!("UTO Suite — {}", suite.suite_id);
    let bc = badge_class(&suite.status);
    let s = &suite.summary;
    let duration_val = suite
        .timeline
        .duration_ms
        .map(|d| format!("<span data-ms=\"{d}\">{d}ms</span>"))
        .unwrap_or_else(|| "-".to_string());

    let cards = [
        ("Total", s.total.to_string(), ""),
        (
            "Passed",
            s.passed.to_string(),
            if s.passed == s.total && s.total > 0 {
                "ok"
            } else {
                ""
            },
        ),
        (
            "Failed",
            s.failed.to_string(),
            if s.failed > 0 { "fail" } else { "" },
        ),
        ("Skipped", s.skipped.to_string(), ""),
        ("Duration", duration_val, ""),
    ];

    let meta = [
        ("Schema", escape_html(&suite.schema_version)),
        ("Framework", escape_html(&suite.framework)),
        ("Suite ID", escape_html(&suite.suite_id)),
        ("Mode", escape_html(&suite.mode)),
        ("Status", escape_html(&suite.status)),
        (
            "Started",
            format!(
                "<span data-ts=\"{}\">{}</span>",
                suite.timeline.started_at_unix_ms, suite.timeline.started_at_unix_ms
            ),
        ),
        (
            "Duration",
            suite
                .timeline
                .duration_ms
                .map(|d| format!("<span data-ms=\"{d}\">{d}ms</span>"))
                .unwrap_or_else(|| "-".to_string()),
        ),
    ];

    let mut buf = String::with_capacity(65_536);
    buf.push_str(&head_html(&title));
    buf.push_str(&topbar_html(
        "UTO",
        "Suite Report",
        &suite.suite_id,
        &suite.status,
        bc,
    ));
    buf.push_str("<div class=\"main\">");
    buf.push_str(&hero_html(
        "Suite Execution",
        &suite.suite_id,
        &suite.status,
        suite.timeline.started_at_unix_ms,
        suite.timeline.duration_ms,
        bc,
    ));
    buf.push_str(&cards_html(&cards));
    buf.push_str(&meta_grid_html(&meta));
    buf.push_str(&tests_section_html(&suite.tests));
    buf.push_str("</div>");
    buf.push_str(FOOTER_HTML);
    buf
}

/// Writes an HTML report for a single-run artifact to the requested file path.
pub fn write_report_html(report: &UtoReportV1, file_path: &Path) -> Result<(), String> {
    let html = render_report_html(report);
    std::fs::write(file_path, html).map_err(|e| {
        format!(
            "Failed to write HTML report at {}: {e}",
            file_path.display()
        )
    })
}

/// Writes an HTML suite report to the requested file path.
pub fn write_suite_html(suite: &UtoSuiteReportV1, file_path: &Path) -> Result<(), String> {
    let html = render_suite_html(suite);
    std::fs::write(file_path, html).map_err(|e| {
        format!(
            "Failed to write HTML suite report at {}: {e}",
            file_path.display()
        )
    })
}

// ---------------------------------------------------------------------------
// Internal HTML builders
// ---------------------------------------------------------------------------

fn head_html(title: &str) -> String {
    format!(
        concat!(
            "<!doctype html>\n<html>\n<head>\n",
            "<meta charset=\"utf-8\">",
            "<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">",
            "<title>{t}</title>",
            "<style>{css}</style>",
            "<script>{js}</script>",
            "</head><body>\n"
        ),
        t = escape_html(title),
        css = STYLE,
        js = SCRIPT
    )
}

fn topbar_html(brand: &str, label: &str, id: &str, status: &str, badge_cls: &str) -> String {
    format!(
        concat!(
            "<header class=\"topbar\"><div class=\"wrap-h\">",
            "<span class=\"brand\">{brand}</span>",
            "<span class=\"sep\"></span>",
            "<span class=\"topbar-title\">{label}: <code>{id}</code></span>",
            "<span class=\"badge {bc}\">{status}</span>",
            "<span class=\"spacer\"></span>",
            "<button class=\"theme-btn\" onclick=\"toggleTheme()\" title=\"Toggle theme\"><span class=\"theme-icon\">\u{1F319}</span></button>",
            "</div></header>\n"
        ),
        brand = escape_html(brand),
        label = escape_html(label),
        id = escape_html(id),
        bc = badge_cls,
        status = escape_html(status),
    )
}

fn hero_html(
    kicker: &str,
    title: &str,
    status: &str,
    started_ms: u64,
    duration_ms: Option<u64>,
    badge_cls: &str,
) -> String {
    let duration_html = duration_ms
        .map(|d| {
            format!(
                "<span class=\"badge b-run\">duration: <span data-ms=\"{d}\">{d}ms</span></span>"
            )
        })
        .unwrap_or_else(|| "<span class=\"badge b-run\">duration: -</span>".to_string());
    format!(
        concat!(
            "<section class=\"hero\">",
            "<div class=\"hero-kicker\">{k}</div>",
            "<h1 class=\"hero-title\">{t}</h1>",
            "<div class=\"hero-meta\">",
            "<span class=\"badge {bc}\">{s}</span>",
            "<span class=\"badge b-run\">started: <span data-ts=\"{st}\">{st}</span></span>",
            "{dur}",
            "</div></section>"
        ),
        k = escape_html(kicker),
        t = escape_html(title),
        bc = badge_cls,
        s = escape_html(status),
        st = started_ms,
        dur = duration_html,
    )
}

fn cards_html(cards: &[(&str, String, &str)]) -> String {
    let mut buf = String::from("<div class=\"cards\">");
    for (lbl, val, extra_cls) in cards {
        buf.push_str(&format!(
            "<div class=\"card\"><div class=\"card-lbl\">{lbl}</div><div class=\"card-val {cls}\">{val}</div></div>",
            lbl = escape_html(lbl),
            val = val,          // already escaped or dynamic HTML
            cls = extra_cls,
        ));
    }
    buf.push_str("</div>");
    buf
}

fn meta_grid_html(items: &[(&str, String)]) -> String {
    let mut buf = String::from("<div class=\"meta-grid\">");
    for (k, v) in items {
        buf.push_str(&format!(
            "<div class=\"mc\"><div class=\"mc-k\">{k}</div><div class=\"mc-v\">{v}</div></div>",
            k = escape_html(k),
            v = v, // already escaped or dynamic HTML
        ));
    }
    buf.push_str("</div>");
    buf
}

fn error_block_html(err: &str) -> String {
    format!(
        "<div class=\"err-block\"><strong>Error:</strong>\n{}</div>",
        escape_html(err)
    )
}

fn events_panel_html(events: &[ReportEvent], table_id: &str) -> String {
    let count = events.len();
    let rows = event_rows_html(events, table_id);

    format!(
        concat!(
            "<div class=\"panel events-panel\">",
            "<div class=\"panel-hd\">",
            "<span class=\"panel-title\">Events</span>",
            "<span class=\"panel-sub match-count\" data-total=\"{count}\">{count} / {count} shown</span>",
            "<span class=\"badge b-skip\">{count}</span>",
            "</div>",
            "<div class=\"toolbar\">",
            "<input class=\"search-inp\" type=\"search\" placeholder=\"Search events\u{2026}\" ",
            "oninput=\"filterRows('{tid}')\" />",
            "<div class=\"filters\">",
            "<button class=\"fb active\" data-f=\"all\" onclick=\"setFilter(this,'{tid}')\">All</button>",
            "<button class=\"fb\" data-f=\"ok\" onclick=\"setFilter(this,'{tid}')\">OK</button>",
            "<button class=\"fb\" data-f=\"failed\" onclick=\"setFilter(this,'{tid}')\">Failed</button>",
            "</div>",
            "</div>",
            "<div class=\"tbl-wrap\">",
            "<table id=\"{tid}\">",
            "<thead><tr>",
            "<th class=\"c-idx\">#</th>",
            "<th class=\"c-stage\">Stage</th>",
            "<th class=\"c-status\">Status</th>",
            "<th class=\"c-detail\">Detail</th>",
            "</tr></thead>",
            "<tbody>{rows}</tbody>",
            "</table>",
            "</div></div>"
        ),
        count = count,
        tid = table_id,
        rows = rows,
    )
}

fn event_rows_html(events: &[ReportEvent], _table_id: &str) -> String {
    if events.is_empty() {
        return "<tr><td colspan=\"4\" class=\"empty\">No events recorded.</td></tr>".to_string();
    }
    let mut buf = String::new();
    for (i, ev) in events.iter().enumerate() {
        let detail_str = serde_json::to_string_pretty(&ev.detail)
            .unwrap_or_else(|_| "\"<invalid>\"".to_string());
        let status_norm = normalise_status(&ev.status);
        let bc = badge_class(status_norm);
        buf.push_str(&format!(
            "<tr data-s=\"{status_norm}\"><td class=\"c-idx\">{n}</td><td class=\"c-stage\">{stage}</td><td class=\"c-status\"><span class=\"badge {bc}\">{status}</span></td><td class=\"c-detail\"><pre>{detail}</pre></td></tr>",
            n = i + 1,
            stage = escape_html(&ev.stage),
            status_norm = status_norm,
            bc = bc,
            status = escape_html(&ev.status),
            detail = escape_html(&detail_str),
        ));
    }
    buf
}

fn tests_section_html(tests: &[TestCaseResult]) -> String {
    if tests.is_empty() {
        return "<div class=\"panel\"><div class=\"panel-hd\"><span class=\"panel-title\">Tests</span></div><div style=\"padding:20px;color:var(--tx2);text-align:center\">No test cases recorded.</div></div>".to_string();
    }
    let mut buf = format!(
        concat!(
            "<div class=\"panel\">",
            "<div class=\"panel-hd\">",
            "<span class=\"panel-title\">Tests</span>",
            "<span id=\"suite-test-count\" class=\"panel-sub\">{count} / {count} shown</span>",
            "<span class=\"badge b-skip\">{count}</span>",
            "</div>",
            "<div class=\"suite-controls\">",
            "<input id=\"suite-test-search\" class=\"search-inp\" type=\"search\" placeholder=\"Search test names...\" oninput=\"filterTests()\" />",
            "<div class=\"suite-status-filters\">",
            "<button class=\"fb active\" data-f=\"all\" onclick=\"setTestFilter(this)\">All</button>",
            "<button class=\"fb\" data-f=\"passed\" onclick=\"setTestFilter(this)\">Passed</button>",
            "<button class=\"fb\" data-f=\"failed\" onclick=\"setTestFilter(this)\">Failed</button>",
            "<button class=\"fb\" data-f=\"skipped\" onclick=\"setTestFilter(this)\">Skipped</button>",
            "</div>",
            "<div class=\"suite-actions\">",
            "<button class=\"btn\" onclick=\"toggleAllTests(true)\">Expand All</button>",
            "<button class=\"btn\" onclick=\"toggleAllTests(false)\">Collapse All</button>",
            "</div>",
            "</div>",
            "<div id=\"suite-tests-wrap\" style=\"padding:10px\">"
        ),
        count = tests.len(),
    );
    for (idx, tc) in tests.iter().enumerate() {
        buf.push_str(&test_item_html(tc, idx));
    }
    buf.push_str("</div></div>");
    buf
}

fn test_item_html(tc: &TestCaseResult, idx: usize) -> String {
    let body_id = format!("tc-body-{idx}");
    let table_id = format!("tc-tbl-{idx}");
    let bc = badge_class(&tc.status);
    let status_norm = normalise_status(&tc.status).to_ascii_lowercase();
    let search_name = tc.name.to_ascii_lowercase();
    let dur = tc
        .timeline
        .duration_ms
        .map(|d| {
            format!(
                "<span data-ms=\"{d}\" class=\"test-dur\">{d}ms</span>",
                d = d
            )
        })
        .unwrap_or_else(|| "<span class=\"test-dur\">-</span>".to_string());
    let ev_count = tc.events.len();

    // Failed tests open by default for quick triage
    let open_cls = if normalise_status(&tc.status) == "failed" {
        " open"
    } else {
        ""
    };

    let err_html = tc
        .error
        .as_ref()
        .map(|e| format!("<div class=\"test-err\">{}</div>", escape_html(e)))
        .unwrap_or_default();

    let rows = event_rows_html(&tc.events, &table_id);
    let events_html = format!(
        concat!(
            "<div class=\"toolbar\">",
            "<input class=\"search-inp\" type=\"search\" placeholder=\"Search events\u{2026}\" ",
            "oninput=\"filterRows('{tid}')\" />",
            "<div class=\"filters\">",
            "<button class=\"fb active\" data-f=\"all\" onclick=\"setFilter(this,'{tid}')\">All</button>",
            "<button class=\"fb\" data-f=\"ok\" onclick=\"setFilter(this,'{tid}')\">OK</button>",
            "<button class=\"fb\" data-f=\"failed\" onclick=\"setFilter(this,'{tid}')\">Failed</button>",
            "</div></div>",
            "<div class=\"tbl-wrap\"><table id=\"{tid}\">",
            "<thead><tr>",
            "<th class=\"c-idx\">#</th>",
            "<th class=\"c-stage\">Stage</th>",
            "<th class=\"c-status\">Status</th>",
            "<th class=\"c-detail\">Detail</th>",
            "</tr></thead>",
            "<tbody>{rows}</tbody>",
            "</table></div>"
        ),
        tid = table_id,
        rows = rows,
    );

    format!(
        concat!(
            "<div class=\"test-item\" data-test-name=\"{search_name}\" data-test-status=\"{status_norm}\">",
            "<div class=\"test-hd{open}\" data-toggle=\"{bid}\" onclick=\"toggleSection('{bid}')\">",
            "<svg class=\"chevron\" viewBox=\"0 0 16 16\" fill=\"currentColor\">",
            "<path d=\"M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06z\"/>",
            "</svg>",
            "<span class=\"test-name\" title=\"{name}\">{name}</span>",
            "<span class=\"badge {bc}\">{status}</span>",
            "{dur}",
            "<span class=\"badge b-skip\" title=\"{ec} events\">{ec}</span>",
            "</div>",
            "<div class=\"test-body events-panel{open}\" id=\"{bid}\">",
            "{err}{events}",
            "</div>",
            "</div>"
        ),
        open = open_cls,
        search_name = escape_html(&search_name),
        status_norm = escape_html(&status_norm),
        bid = body_id,
        name = escape_html(&tc.name),
        bc = bc,
        status = escape_html(&tc.status),
        dur = dur,
        ec = ev_count,
        err = err_html,
        events = events_html,
    )
}

static FOOTER_HTML: &str = concat!(
    "<footer>Generated by <strong>uto-reporter</strong> &mdash; ",
    "<a href=\"https://github.com/adrianbenavides/uto\" style=\"color:var(--acc)\">UTO</a>",
    "</footer></body></html>"
);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn badge_class(status: &str) -> &'static str {
    match normalise_status(status) {
        "passed" | "ok" => "b-ok",
        "failed" | "fail" | "error" => "b-fail",
        "partial" | "running" => "b-warn",
        "skipped" => "b-skip",
        _ => "b-skip",
    }
}

fn badge_val_class(status: &str) -> &'static str {
    match normalise_status(status) {
        "passed" | "ok" => "ok",
        "failed" | "fail" | "error" => "fail",
        "partial" | "running" => "warn",
        _ => "",
    }
}

fn normalise_status(s: &str) -> &str {
    match s.to_ascii_lowercase().as_str() {
        "passed" | "pass" => "passed",
        "failed" | "fail" | "error" => "failed",
        "ok" => "ok",
        "skipped" | "skip" | "ignored" => "skipped",
        "partial" => "partial",
        "running" => "running",
        _ => s,
    }
}

pub(crate) fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ReportEvent, ReportTimeline, SuiteSummary, TestCaseResult};

    fn make_report(run_id: &str, status: &str) -> UtoReportV1 {
        let mut r = UtoReportV1::new(run_id.to_string(), "web".to_string(), 1000);
        r.status = status.to_string();
        r.timeline.finished_at_unix_ms = Some(1200);
        r.timeline.duration_ms = Some(200);
        r.events.push(ReportEvent {
            stage: "session.goto".to_string(),
            status: "ok".to_string(),
            detail: serde_json::json!({"url": "https://example.com"}),
        });
        r
    }

    #[test]
    fn render_report_html_contains_core_fields() {
        let report = make_report("run-77", "passed");
        let html = render_report_html(&report);
        assert!(html.contains("run-77"));
        assert!(html.contains("session.goto"));
        assert!(html.contains("Execution Report"));
    }

    #[test]
    fn render_report_html_escapes_untrusted_text() {
        let mut report = make_report("run-<1>", "failed");
        report.error = Some("<script>alert('x')</script>".to_string());
        let html = render_report_html(&report);
        assert!(html.contains("run-&lt;1&gt;"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert('x')</script>"));
    }

    #[test]
    fn render_report_html_has_search_filter_js() {
        let report = make_report("run-99", "passed");
        let html = render_report_html(&report);
        assert!(html.contains("filterRows"));
        assert!(html.contains("toggleTheme"));
        assert!(html.contains("setFilter"));
    }

    #[test]
    fn render_suite_html_contains_test_names() {
        use crate::schema::UtoSuiteReportV1;
        let mut suite = UtoSuiteReportV1::new("suite-1".to_string(), "web".to_string(), 1000);
        suite.status = "passed".to_string();
        suite.summary = SuiteSummary {
            total: 2,
            passed: 2,
            failed: 0,
            skipped: 0,
        };
        suite.tests.push(TestCaseResult {
            name: "login flow".to_string(),
            status: "passed".to_string(),
            timeline: ReportTimeline {
                started_at_unix_ms: 1000,
                finished_at_unix_ms: Some(1500),
                duration_ms: Some(500),
            },
            events: vec![],
            error: None,
        });
        suite.tests.push(TestCaseResult {
            name: "checkout flow".to_string(),
            status: "passed".to_string(),
            timeline: ReportTimeline {
                started_at_unix_ms: 1500,
                finished_at_unix_ms: Some(2000),
                duration_ms: Some(500),
            },
            events: vec![],
            error: None,
        });
        let html = render_suite_html(&suite);
        assert!(html.contains("Suite Report"));
        assert!(html.contains("login flow"));
        assert!(html.contains("checkout flow"));
    }

    #[test]
    fn render_suite_html_failed_test_opens_by_default() {
        use crate::schema::UtoSuiteReportV1;
        let mut suite = UtoSuiteReportV1::new("suite-2".to_string(), "web".to_string(), 1000);
        suite.tests.push(TestCaseResult {
            name: "failing test".to_string(),
            status: "failed".to_string(),
            timeline: ReportTimeline {
                started_at_unix_ms: 1000,
                finished_at_unix_ms: Some(1200),
                duration_ms: Some(200),
            },
            events: vec![],
            error: Some("assertion failed".to_string()),
        });
        let html = render_suite_html(&suite);
        // failed test body should have 'open' class by default
        assert!(html.contains("class=\"test-body events-panel open\""));
        assert!(html.contains("assertion failed"));
    }
}
