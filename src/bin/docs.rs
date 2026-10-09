//! Canonical Markdown publication with source provenance and explicit code review evidence.
#[allow(dead_code)]
#[path = "../main.rs"]
mod theme;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}
#[derive(Default)]
struct Meta {
    title: Option<String>,
    baseline: Option<String>,
    watches: Vec<String>,
}
fn metadata(source: &str) -> Result<(Meta, &str)> {
    let mut meta = Meta::default();
    let Some(rest) = source.strip_prefix("---\n") else {
        return Ok((meta, source));
    };
    let end = rest
        .find("\n---\n")
        .ok_or("Unclosed Markdown frontmatter")?;
    let mut watches = false;
    for line in rest[..end].lines() {
        if watches && line.trim_start().starts_with("- ") {
            let path = line.trim_start()[2..].trim().trim_matches(['\'', '"']);
            if path.is_empty()
                || Path::new(path).is_absolute()
                || Path::new(path)
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err("Watched paths must be nonempty repository-relative paths".into());
            }
            meta.watches.push(path.into());
            continue;
        }
        watches = false;
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim().trim_matches(['\'', '"']);
            match key.trim() {
                "title" => meta.title = Some(value.into()),
                "source_commit" => meta.baseline = Some(value.into()),
                "watches" => {
                    if !value.is_empty() {
                        return Err("Use a multiline watches list".into());
                    }
                    watches = true;
                }
                _ => {} // Other metadata is owned by the authoring repository, not Wiki admission.
            }
        }
    }
    Ok((meta, &rest[end + 5..]))
}
fn url(value: &str) -> Result<&str> {
    if !value.starts_with("https://")
        || value
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\''))
    {
        return Err("Repository URL must be HTTPS".into());
    }
    Ok(value.trim_end_matches('/'))
}
fn revision(repo: &Path, source: &Path) -> Result<(String, String, String)> {
    let root = PathBuf::from(git(repo, &["rev-parse", "--show-toplevel"])?);
    let relative = source
        .strip_prefix(&root)?
        .to_str()
        .ok_or("Non-UTF8 source")?;
    git(&root, &["ls-files", "--error-unmatch", "--", relative])?;
    if !git(&root, &["status", "--porcelain", "--", relative])?.is_empty() {
        return Err(format!("Commit canonical Markdown before publishing: {relative}").into());
    }
    let log = git(
        &root,
        &["log", "-1", "--format=%H%n%cs%n%s", "--", relative],
    )?;
    let mut lines = log.lines();
    Ok((
        lines.next().ok_or("Missing source commit")?.into(),
        lines.next().ok_or("Missing source date")?.into(),
        lines.collect::<Vec<_>>().join(" "),
    ))
}
fn review(
    meta: &Meta,
    code_repo: Option<&Path>,
    code_url: Option<&str>,
    check: bool,
) -> Result<String> {
    let Some(sha) = &meta.baseline else {
        if !meta.watches.is_empty() {
            return Err("Watched paths require source_commit".into());
        }
        return Ok("<p>Code review baseline not recorded.</p>".into());
    };
    if sha.len() != 40 || !sha.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("source_commit must be a full 40-character commit SHA".into());
    }
    let repo = code_repo.ok_or("source_commit requires explicit --code-repository")?;
    let base_url = url(code_url.ok_or("source_commit requires --code-url")?)?;
    git(repo, &["cat-file", "-e", &format!("{sha}^{{commit}}")])?;
    git(repo, &["merge-base", "--is-ancestor", sha, "HEAD"])?;
    let mut changes = Vec::new();
    for path in &meta.watches {
        if git(repo, &["ls-files", "--", path])?.is_empty() {
            return Err(format!("Watched path has no tracked source: {path}").into());
        }
        if !repo.join(path).exists() {
            return Err(format!("Watched path missing: {path}").into());
        }
        if !git(repo, &["status", "--porcelain", "--", path])?.is_empty() {
            changes.push(format!("{path}: uncommitted changes"));
        }
        let log = git(
            repo,
            &[
                "log",
                "--no-merges",
                "--format=%h %s",
                &format!("{sha}..HEAD"),
                "--",
                path,
            ],
        )?;
        if !log.is_empty() {
            changes.push(format!("{path}: {log}"));
        }
    }
    if check && !changes.is_empty() {
        return Err(format!("Documentation review is stale: {}", changes.join("; ")).into());
    }
    let state = if meta.watches.is_empty() {
        "No watched paths; freshness not assessed."
    } else if changes.is_empty() {
        "No changes to watched paths since review."
    } else {
        "Watched code changed since review; revalidation needed."
    };
    let mut out = format!(
        "<p>Reviewed against <a href=\"{}/commit/{}\"><code>{}</code></a>. {}</p>",
        escape(base_url),
        sha,
        &sha[..7],
        state
    );
    if !meta.watches.is_empty() {
        out += "<details><summary>Watched source paths</summary><ul>";
        for path in &meta.watches {
            out += &format!("<li><code>{}</code></li>", escape(path));
        }
        out += "</ul></details>";
    }
    if !changes.is_empty() {
        out += &format!("<pre>{}</pre>", escape(&changes.join("\n")));
    }
    Ok(out)
}
#[cfg(test)]
fn markdown(body: &str) -> String {
    markdown_at(body, None).expect("Markdown without source links is infallible")
}
fn markdown_at(body: &str, context: Option<(&str, &str, &str)>) -> Result<String> {
    // Raw HTML is treated as text. Project docs do not execute arbitrary embedded HTML.
    let mut events: Vec<_> = Parser::new_ext(
        body,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .map(|event| match event {
        Event::Html(s) | Event::InlineHtml(s) if s.trim_start().starts_with("<!--") => {
            Event::Text("".into())
        }
        Event::Html(s) | Event::InlineHtml(s) => Event::Text(s),
        other => other,
    })
    .collect();
    let mut used = BTreeMap::<String, usize>::new();
    for i in 0..events.len() {
        if let Some((repository, revision, source)) = context {
            if let Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) =
                &mut events[i]
            {
                let destination = dest_url.as_ref();
                if !destination.starts_with('#')
                    && !destination.starts_with('/')
                    && !destination.contains(':')
                {
                    let (path, fragment) = destination
                        .split_once('#')
                        .map_or((destination, ""), |(p, f)| (p, f));
                    let parent = Path::new(source)
                        .parent()
                        .ok_or("Missing canonical source parent")?;
                    let mut normalized = PathBuf::new();
                    for component in parent.join(path).components() {
                        match component {
                            std::path::Component::Normal(value) => normalized.push(value),
                            std::path::Component::CurDir => {}
                            std::path::Component::ParentDir => {
                                if !normalized.pop() {
                                    return Err(
                                        "Relative Markdown link escapes source repository".into()
                                    );
                                }
                            }
                            _ => return Err("Invalid relative Markdown link".into()),
                        }
                    }
                    let fragment = if fragment.is_empty() {
                        String::new()
                    } else {
                        format!("#{}", fragment)
                    };
                    *dest_url = format!(
                        "{repository}/blob/{revision}/{}{fragment}",
                        percent_path(normalized.to_str().ok_or("Non-UTF8 link")?)
                    )
                    .into();
                }
            }
        }
        if matches!(events[i], Event::Start(Tag::Heading { .. })) {
            let text: String = events[i + 1..]
                .iter()
                .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(_))))
                .filter_map(|e| match e {
                    Event::Text(s) | Event::Code(s) => Some(s.as_ref()),
                    _ => None,
                })
                .collect();
            let base = text
                .to_lowercase()
                .chars()
                .filter_map(|c| {
                    if c.is_alphanumeric() || c == '-' {
                        Some(c)
                    } else if c.is_whitespace() {
                        Some('-')
                    } else {
                        None
                    }
                })
                .collect::<String>();
            let count = used.entry(base.clone()).or_default();
            let anchor = if *count == 0 {
                base.clone()
            } else {
                format!("{base}-{count}")
            };
            *count += 1;
            if let Event::Start(Tag::Heading { id, .. }) = &mut events[i] {
                *id = Some(anchor.into());
            }
        }
    }
    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    Ok(out)
}
fn sync(path: &Path, bytes: &[u8], check: bool) -> Result<()> {
    if check {
        if fs::read(path)? != bytes {
            return Err(format!("Documentation drift: {}", path.display()).into());
        }
    } else {
        fs::create_dir_all(path.parent().ok_or("Missing output directory")?)?;
        let temp = path.with_extension("docs-next");
        fs::write(&temp, bytes)?;
        fs::rename(temp, path)?;
    }
    Ok(())
}
fn sidebar(
    article: &str,
    site: &Path,
    output: &Path,
    prefix: &str,
    peers: &[BTreeMap<String, String>],
) -> Result<String> {
    let mut out = String::from(
        "<aside class=\"resources docs-sidebar\" aria-label=\"Help and documentation\">",
    );
    if !peers.is_empty() {
        out += "<nav aria-label=\"Help guides\"><h2>Documentation</h2><ul>";
        for peer in peers {
            let peer_site = fs::canonicalize(peer.get("site").ok_or("Missing peer site")?)?;
            if peer_site != site {
                continue;
            }
            let path = PathBuf::from(peer.get("output").ok_or("Missing peer output")?);
            let relative = path
                .strip_prefix(site)
                .map_err(|_| "Peer output outside site")?;
            if relative
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err("Peer output traverses site".into());
            }
            let source = fs::read_to_string(peer.get("source").ok_or("Missing peer source")?)?;
            let (meta, body) = metadata(&source)?;
            let title = meta
                .title
                .or_else(|| {
                    body.lines()
                        .find_map(|line| line.strip_prefix("# ").map(str::to_owned))
                })
                .ok_or("Guide needs a title")?;
            let current = if path == output {
                " aria-current=\"page\""
            } else {
                ""
            };
            out += &format!(
                "<li><a href=\"{}{}\"{current}>{}</a></li>",
                escape(prefix),
                percent_path(relative.to_str().ok_or("Non-UTF8 peer output")?),
                escape(&title)
            );
        }
        out += "</ul></nav>";
    }
    out += "<nav aria-label=\"On this page\"><h2>On this page</h2><ul>";
    // Read the renderer's emitted heading IDs so duplicate-heading anchors agree.
    for section in article.split("<h2 id=\"").skip(1) {
        let (id, rest) = section
            .split_once("\">")
            .ok_or("Malformed rendered heading")?;
        let label = rest
            .split_once("</h2>")
            .ok_or("Unclosed rendered heading")?
            .0;
        out += &format!("<li><a href=\"#{}\">{label}</a></li>", escape(id));
    }
    out += "</ul></nav><a href=\"#documentation-provenance\">Source and freshness</a></aside>";
    Ok(out)
}
fn generate(
    args: &BTreeMap<String, String>,
    check: bool,
    peers: &[BTreeMap<String, String>],
) -> Result<()> {
    let required = |key: &str| -> Result<&str> {
        args.get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("Missing --{key}").into())
    };
    let source = fs::canonicalize(required("source")?)?;
    let site = fs::canonicalize(required("site")?)?;
    let output_arg = PathBuf::from(required("output")?);
    let output = if output_arg.is_absolute() {
        output_arg
    } else {
        std::env::current_dir()?.join(output_arg)
    };
    let relative = output
        .strip_prefix(&site)
        .map_err(|_| "Output must be inside --site")?;
    if relative
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
        || output.extension().is_none_or(|e| e != "html")
    {
        return Err("Output must be a site-relative HTML file".into());
    }
    let mut candidate = site.clone();
    for component in relative.components() {
        candidate.push(component);
        match fs::symlink_metadata(&candidate) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("Output must not traverse symlinks".into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let export = output.with_extension("md");
    if source == export {
        return Err("Canonical source must be separate from generated Markdown export".into());
    }
    let prefix = "../".repeat(relative.components().count() - 1);
    let original = fs::read_to_string(&source)?;
    let (meta, body) = metadata(&original)?;
    let title = meta
        .title
        .clone()
        .or_else(|| {
            body.lines()
                .find_map(|line| line.strip_prefix("# ").map(str::to_owned))
        })
        .ok_or("Markdown needs title metadata or a level-one heading")?;
    let repo = source.parent().ok_or("Missing source parent")?;
    let (sha, date, subject) = revision(repo, &source)?;
    let repository_url = url(required("repository-url")?)?;
    let code_repo = args.get("code-repository").map(PathBuf::from);
    let evidence = review(
        &meta,
        code_repo.as_deref(),
        args.get("code-url").map(String::as_str),
        args.contains_key("fail-on-stale"),
    )?;
    let active = theme::project(required("project")?)?;
    let source_root = PathBuf::from(git(repo, &["rev-parse", "--show-toplevel"])?);
    let source_path = source
        .strip_prefix(source_root)?
        .to_str()
        .ok_or("Non-UTF8 source")?;
    // Each path component is URL encoded; source URLs remain pinned to the actual docs commit.
    let source_url = format!("{repository_url}/blob/{sha}/{}", percent_path(source_path));
    let file_name = export
        .file_name()
        .ok_or("Invalid export")?
        .to_str()
        .ok_or("Non-UTF8 export")?;
    let article = markdown_at(body, Some((repository_url, &sha, source_path)))?;
    let home = escape(&active.home_url(&prefix));
    let sidebar = sidebar(&article, &site, &output, &prefix, peers)?;
    let html = format!(
        "<!doctype html>\n<html lang=\"en\" data-project=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><link rel=\"stylesheet\" href=\"{prefix}theme/theme.css\"><link rel=\"icon\" href=\"{prefix}theme/{}.svg\"><script defer src=\"{prefix}theme/docs.js\"></script></head><body class=\"manual-page\"><a class=\"skip\" href=\"#content\">Skip to content</a><header class=\"masthead\"><div class=\"wrap\">{}<nav aria-label=\"Documentation navigation\"><a href=\"{home}\">Home</a><a href=\"{}\">Source Markdown</a></nav></div></header><div class=\"site-grid wrap docs-layout\">{sidebar}<main class=\"manual\" id=\"content\"><div class=\"docs-actions\"><button type=\"button\" class=\"button\" data-copy-markdown=\"{}\">Copy for LLM</button><a href=\"{}\" download>Download Markdown</a><span role=\"status\" aria-live=\"polite\" data-copy-status></span></div><article>{}</article><footer class=\"docs-provenance\"><h2 id=\"documentation-provenance\">Documentation provenance</h2><p>Last documentation change: <a href=\"{repository_url}/commit/{sha}\"><code>{}</code></a> · {} · {}</p><p><a href=\"{}\">Canonical source at this revision</a>. Markdown export preserves the canonical source exactly.</p>{}</footer></main></div></body></html>\n",
        escape(required("project")?),
        escape(&title),
        escape(required("project")?),
        theme::dropdown(active, &prefix),
        escape(file_name),
        escape(file_name),
        escape(file_name),
        article,
        &sha[..7],
        escape(&date),
        escape(&subject),
        escape(&source_url),
        evidence
    );
    sync(&export, original.as_bytes(), check)?;
    sync(&output, html.as_bytes(), check)?;
    println!(
        "{} {} from {} ({})",
        if check { "Checked" } else { "Generated" },
        output.display(),
        source.display(),
        &sha[..7]
    );
    Ok(())
}
fn percent_path(path: &str) -> String {
    path.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut values = BTreeMap::new();
    let mut check = false;
    while let Some(arg) = args.next() {
        if arg == "--check" {
            check = true;
        } else if arg == "--fail-on-stale" {
            values.insert("fail-on-stale".into(), String::new());
        } else {
            let key = arg.strip_prefix("--").ok_or("Expected named option")?;
            if ![
                "source",
                "output",
                "site",
                "project",
                "repository-url",
                "code-repository",
                "code-url",
                "manifest",
            ]
            .contains(&key)
            {
                return Err(format!("Unknown option {arg}").into());
            }
            values.insert(key.to_owned(), args.next().ok_or("Missing option value")?);
        }
    }
    if let Some(manifest) = values.get("manifest") {
        if values
            .keys()
            .any(|k| k != "manifest" && k != "fail-on-stale")
        {
            return Err("Use --manifest with only --check/--fail-on-stale".into());
        }
        let manifest = fs::canonicalize(manifest)?;
        let parent = manifest.parent().ok_or("Missing manifest directory")?;
        let mut entries = Vec::new();
        for (i, line) in fs::read_to_string(&manifest)?.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let columns: Vec<_> = line.split('\t').collect();
            if columns.len() != 7 {
                return Err(
                    format!("Manifest line {} needs 7 tab-separated columns", i + 1).into(),
                );
            }
            let mut entry = BTreeMap::new();
            for (key, value) in [
                "source",
                "output",
                "site",
                "project",
                "repository-url",
                "code-repository",
                "code-url",
            ]
            .into_iter()
            .zip(columns)
            {
                if value == "-" {
                    continue;
                }
                entry.insert(
                    key.into(),
                    if ["source", "output", "site", "code-repository"].contains(&key) {
                        parent
                            .join(value)
                            .to_str()
                            .ok_or("Non-UTF8 manifest path")?
                            .into()
                    } else {
                        value.into()
                    },
                );
            }
            if values.contains_key("fail-on-stale") {
                entry.insert("fail-on-stale".into(), String::new());
            }
            entries.push(entry);
        }
        for entry in &entries {
            generate(entry, check, &entries)?;
        }
        Ok(())
    } else {
        generate(&values, check, &[])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn markdown_export_input_is_preserved_and_unsafe_html_is_text() {
        let source = "---\ntitle: Test\nsource_commit: abc\nwatches:\n  - src/lib.rs\n---\n# Test\n\n| A | B |\n|---|---|\n| 1 | 2 |\n<script>alert(1)</script>\n";
        let (meta, body) = metadata(source).unwrap();
        assert_eq!(meta.watches, ["src/lib.rs"]);
        assert!(markdown(body).contains("<table>"));
        assert!(markdown(body).contains("&lt;script&gt;"));
        assert!(source.starts_with("---\n"));
        assert!(metadata("---\nwatches:\n  - ../private\n---\ntext").is_err());
        let rendered = markdown_at("# Model Support\n\n[Recipes](../../recipes/index.md) [Modes](#model-support) [Official](https://example.com)\n", Some(("https://example.com/repo", "abc", "docs-source/projects/umie.md"))).unwrap();
        assert!(rendered.contains("id=\"model-support\""));
        assert!(rendered.contains("https://example.com/repo/blob/abc/recipes/index.md"));
        assert!(rendered.contains("href=\"#model-support\""));
        assert!(
            markdown_at(
                "[Escape](../../../private.md)",
                Some(("https://example.com/repo", "abc", "docs-source/a.md"))
            )
            .is_err()
        );
    }
    #[test]
    fn source_history_and_review_are_independent_of_generated_commits() {
        let dir = std::env::temp_dir().join(format!("theme-docs-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        git(&dir, &["init"]).unwrap();
        git(&dir, &["config", "user.email", "test@example.invalid"]).unwrap();
        git(&dir, &["config", "user.name", "Test"]).unwrap();
        fs::create_dir_all(dir.join("docs-source/projects")).unwrap();
        fs::write(
            dir.join("docs-source/projects/source.md"),
            "# Original\n\n## Models\n\n## Models\n",
        )
        .unwrap();
        fs::write(dir.join("code.rs"), "original").unwrap();
        git(&dir, &["add", "."]).unwrap();
        git(
            &dir,
            &["commit", "-m", "Document <initial> & reviewed code"],
        )
        .unwrap();
        let source = fs::canonicalize(dir.join("docs-source/projects/source.md")).unwrap();
        let first = revision(&dir, &source).unwrap();
        assert_eq!(revision(source.parent().unwrap(), &source).unwrap(), first);
        fs::write(dir.join("generated.html"), "generated").unwrap();
        git(&dir, &["add", "."]).unwrap();
        git(&dir, &["commit", "-m", "Publish rendered page"]).unwrap();
        assert_eq!(revision(&dir, &source).unwrap(), first);
        let args = BTreeMap::from([
            ("source".into(), source.to_str().unwrap().into()),
            (
                "output".into(),
                dir.join("published/page.html").to_str().unwrap().into(),
            ),
            ("site".into(), dir.to_str().unwrap().into()),
            ("project".into(), "holodeck".into()),
            ("repository-url".into(), "https://example.com/repo".into()),
        ]);
        let peers = vec![args.clone()];
        generate(&args, false, &peers).unwrap();
        generate(&args, true, &peers).unwrap();
        assert_eq!(
            fs::read(dir.join("published/page.md")).unwrap(),
            fs::read(&source).unwrap()
        );
        assert!(
            fs::read_to_string(dir.join("published/page.html"))
                .unwrap()
                .contains("Document &lt;initial&gt; &amp; reviewed code")
        );
        let page = fs::read_to_string(dir.join("published/page.html")).unwrap();
        assert!(page.contains("aria-label=\"Help guides\""));
        assert!(page.contains("href=\"#models-1\""));
        assert!(page.contains("href=\"../index.html\">Home</a>"));
        let mut umie = args.clone();
        umie.insert("project".into(), "umie".into());
        umie.insert(
            "output".into(),
            dir.join("projects/umie.html").to_str().unwrap().into(),
        );
        generate(&umie, false, &[umie.clone()]).unwrap();
        let page = fs::read_to_string(dir.join("projects/umie.html")).unwrap();
        assert!(page.contains("href=\"../projects/umie.html\">Home</a>"));
        assert!(page.contains("aria-current=\"page\">Original</a>"));
        assert_eq!(
            theme::project("dcp").unwrap().home_url("../"),
            "https://dcp.fpl.dev/"
        );
        fs::write(dir.join("published/page.md"), "corrupted").unwrap();
        assert!(generate(&args, true, &[]).is_err());
        let meta = Meta {
            baseline: Some(first.0.clone()),
            watches: vec!["code.rs".into()],
            ..Default::default()
        };
        assert!(
            review(&meta, Some(&dir), Some("https://example.com/repo"), true)
                .unwrap()
                .contains("No changes")
        );
        fs::write(dir.join("code.rs"), "changed").unwrap();
        assert!(review(&meta, Some(&dir), Some("https://example.com/repo"), true).is_err());
        git(&dir, &["add", "."]).unwrap();
        git(&dir, &["commit", "-m", "Code changed"]).unwrap();
        assert!(
            review(&meta, Some(&dir), Some("https://example.com/repo"), false)
                .unwrap()
                .contains("revalidation needed")
        );
        assert!(review(&meta, Some(&dir), Some("https://example.com/repo"), true).is_err());
        fs::write(&source, "# Changed\n").unwrap();
        assert!(revision(&dir, &source).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
