//! `uto-site` — UTO's in-project static website generator.
//!
//! This binary reads Markdown content files from `content/`, renders them
//! through Tera HTML templates from `templates/`, copies static assets from
//! `static/`, and writes the final website to the `dist/` directory ready
//! for serving via GitHub Pages.
//!
//! # Usage
//!
//! ```sh
//! # From the workspace root:
//! cargo run -p uto-site
//!
//! # With a custom output directory:
//! cargo run -p uto-site -- --output /tmp/site
//! ```

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use pulldown_cmark::{html, Options, Parser as MdParser};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use thiserror::Error;
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// UTO static site generator — builds the project landing site.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory containing Markdown content files.
    #[arg(long, default_value = "uto-site/content")]
    content: PathBuf,

    /// Directory containing Tera HTML templates.
    #[arg(long, default_value = "uto-site/templates")]
    templates: PathBuf,

    /// Directory containing static assets (CSS, images, …).
    #[arg(long, default_value = "uto-site/static")]
    r#static: PathBuf,

    /// Output directory where the generated site is written.
    #[arg(long, short, default_value = "uto-site/dist")]
    output: PathBuf,

    /// Base path for static assets and navigation (e.g., "/" for root, "/UTO/" for GitHub Pages).
    /// Defaults to "/" (serves from domain root).
    #[arg(long, default_value = "/")]
    base_path: String,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by the site generator.
#[derive(Debug, Error)]
pub enum SiteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template engine error: {0}")]
    Tera(#[from] tera::Error),

    #[error("TOML parse error in front matter: {0}")]
    Toml(#[from] toml::de::Error),
}

// ---------------------------------------------------------------------------
// Front-matter
// ---------------------------------------------------------------------------

/// TOML front matter found at the top of a Markdown content file between
/// `+++` delimiters.
#[derive(Debug, Deserialize, Serialize)]
struct FrontMatter {
    /// Page title used in `<title>` and the `<h1>`.
    title: String,
    /// Short description used in `<meta name="description">`.
    #[serde(default)]
    description: String,
    /// Template name (without `.html`), defaults to `"page"`.
    #[serde(default = "default_template")]
    template: String,
    /// Output file name, e.g. `index.html`.  Defaults to the slug derived
    /// from the content file stem.
    #[serde(default)]
    slug: String,
}

fn default_template() -> String {
    "page".to_string()
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

/// A fully-parsed content page ready to be rendered.
#[derive(Debug)]
struct Page {
    meta: FrontMatter,
    /// HTML-rendered body (converted from Markdown).
    body_html: String,
    /// Output file path relative to `dist/`.
    output_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Split `+++\n…\n+++\n` TOML front matter from the rest of the file.
///
/// Returns `(raw_toml, markdown_body)`.  If no front matter delimiters are
/// found the whole input is treated as body with no front matter.
fn split_front_matter(source: &str) -> (&str, &str) {
    if !source.starts_with("+++") {
        return ("", source);
    }
    // Skip the opening `+++\n`
    let after_open = &source[3..];
    let after_open = after_open.trim_start_matches('\n');
    if let Some(close_pos) = after_open.find("\n+++") {
        let toml_src = &after_open[..close_pos];
        let body_start = close_pos + 4; // skip `\n+++`
        let body = after_open[body_start..].trim_start_matches('\n');
        (toml_src, body)
    } else {
        ("", source)
    }
}

/// Convert CommonMark Markdown to an HTML string.
fn markdown_to_html(src: &str) -> String {
    let opts = Options::all();
    let parser = MdParser::new_ext(src, opts);
    let mut html_output = String::with_capacity(src.len() * 2);
    html::push_html(&mut html_output, parser);
    html_output
}

/// Parse a single `.md` content file into a [`Page`].
fn parse_page(path: &Path, content_root: &Path) -> Result<Page, SiteError> {
    let source = fs::read_to_string(path)?;
    let (toml_src, markdown_body) = split_front_matter(&source);

    // If no front matter, derive title from the file stem.
    let mut meta: FrontMatter = if toml_src.is_empty() {
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        FrontMatter {
            title: stem.replace(['-', '_'], " "),
            description: String::new(),
            template: default_template(),
            slug: String::new(),
        }
    } else {
        toml::from_str(toml_src)?
    };

    // Derive slug from file stem if not explicitly set.
    if meta.slug.is_empty() {
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        meta.slug = stem;
    }

    let body_html = markdown_to_html(markdown_body);

    // Reconstruct the output path relative to dist/:
    //   content/index.md         → index.html
    //   content/contribute.md    → contribute/index.html  (pretty URLs)
    let relative = path.strip_prefix(content_root).unwrap_or(path);
    let output_path = if meta.slug == "index" {
        PathBuf::from("index.html")
    } else {
        let dir: PathBuf = relative.parent().unwrap_or(Path::new("")).to_path_buf();
        dir.join(&meta.slug).join("index.html")
    };

    Ok(Page {
        meta,
        body_html,
        output_path,
    })
}

///
/// Render a [`Page`] to an HTML string using the Tera template engine.
///
/// The `base_path` parameter is injected into the Tera context so templates
/// can reference it when constructing URLs for static assets and navigation.
fn render_page(page: &Page, tera: &Tera, base_path: &str) -> Result<String, SiteError> {
    let mut ctx = Context::new();
    ctx.insert("title", &page.meta.title);
    ctx.insert("description", &page.meta.description);
    ctx.insert("body", &page.body_html);
    ctx.insert("base_path", base_path);

    let template_name = format!("{}.html", page.meta.template);
    let html = tera.render(&template_name, &ctx)?;
    Ok(html)
}

// ---------------------------------------------------------------------------
// Static asset copy
// ---------------------------------------------------------------------------

/// Recursively copy all files from `src_dir` into `dest_dir`, preserving the
/// relative directory structure.
fn copy_static(src_dir: &Path, dest_dir: &Path) -> Result<(), SiteError> {
    if !src_dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(src_dir).unwrap_or(entry.path());
            let dest = dest_dir.join(relative);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<(), SiteError> {
    let args = Args::parse();

    // ------------------------------------------------------------------
    // 1. Load Tera templates
    // ------------------------------------------------------------------
    let template_glob = format!("{}/**/*.html", args.templates.display());
    let tera = Tera::new(&template_glob).map_err(SiteError::Tera)?;

    // ------------------------------------------------------------------
    // 2. Prepare output directory
    // ------------------------------------------------------------------
    if args.output.exists() {
        fs::remove_dir_all(&args.output)?;
    }
    fs::create_dir_all(&args.output)?;

    // ------------------------------------------------------------------
    // 3. Parse and render content pages
    // ------------------------------------------------------------------
    let mut pages_built = 0usize;
    for entry in WalkDir::new(&args.content)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|e| e.to_str()) == Some("md")
        {
            let page = parse_page(entry.path(), &args.content)?;
            let html = render_page(&page, &tera, &args.base_path)?;

            let dest = args.output.join(&page.output_path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dest, &html)?;
            println!("  ✓  {:?}  →  {:?}", entry.path(), dest);
            pages_built += 1;
        }
    }

    // ------------------------------------------------------------------
    // 4. Copy static assets
    // ------------------------------------------------------------------
    copy_static(&args.r#static, &args.output)?;

    println!(
        "\n🚀  Site built successfully — {pages_built} page(s) → {:?}",
        args.output
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_split_front_matter_with_delimiters() {
        let src = "+++\ntitle = \"Hello\"\n+++\n# Body\n";
        let (toml, body) = split_front_matter(src);
        assert_eq!(toml, "title = \"Hello\"");
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn test_split_front_matter_no_delimiters() {
        let src = "# Just markdown\n";
        let (toml, body) = split_front_matter(src);
        assert!(toml.is_empty());
        assert_eq!(body, "# Just markdown\n");
    }

    #[test]
    fn test_markdown_to_html_heading() {
        let html = markdown_to_html("# Hello World\n");
        assert!(html.contains("<h1>Hello World</h1>"));
    }

    #[test]
    fn test_markdown_to_html_paragraph() {
        let html = markdown_to_html("Some text.\n");
        assert!(html.contains("<p>Some text.</p>"));
    }

    #[test]
    fn test_parse_page_with_front_matter() {
        let content = "+++\ntitle = \"Test Page\"\ndescription = \"A test.\"\n+++\n# Hello\n";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        let tmp_dir = file.path().parent().unwrap().to_path_buf();
        let page = parse_page(file.path(), &tmp_dir).unwrap();
        assert_eq!(page.meta.title, "Test Page");
        assert!(page.body_html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_copy_static_copies_files() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let file = src.path().join("style.css");
        fs::write(&file, "body { margin: 0; }").unwrap();

        copy_static(src.path(), dst.path()).unwrap();

        let dest_file = dst.path().join("style.css");
        assert!(dest_file.exists());
    }
}
