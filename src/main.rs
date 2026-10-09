//! Source-owned, dependency-free theme installer and drift checker.
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(crate) struct Project {
    id: &'static str,
    pub(crate) name: &'static str,
    protocol: bool,
    subtitle: &'static str,
    page: Option<&'static str>,
    url: &'static str,
    colors: [&'static str; 7],
    accents: &'static [(&'static str, &'static str)],
    icon: &'static str,
}
const PROJECTS: [Project; 5] = [
    Project {
        id: "fungos",
        name: "fungOS",
        protocol: false,
        subtitle: "",
        page: None,
        url: "https://fungos.dev/",
        colors: [
            "#e5e8e2", "#c9cdca", "#202d26", "#46554b", "#a0aaa0", "#28543b", "#283d31",
        ],
        accents: &[
            ("header-rule", "#7e9273"),
            ("header-ink", "#e5eadd"),
            ("focus", "#714b21"),
            ("note-rule", "#73836a"),
            ("button-border", "#6b7e6d"),
            ("button-bg", "#e5e8e2"),
            ("button-fill", "#355a3d"),
            ("button-hover", "#d4ddd0"),
            ("button-active", "#284a31"),
            ("surface", "#d3dace"),
            ("code-bg", "#dce0d7"),
        ],
        icon: include_str!("../assets/fungos.svg"),
    },
    Project {
        id: "holodeck",
        name: "Holodeck",
        protocol: false,
        subtitle: "",
        page: None,
        url: "https://holodeck.fpl.dev/",
        colors: [
            "#fafaf7", "#e9eceb", "#25313a", "#59666d", "#cbd2d4", "#245b81", "#203440",
        ],
        accents: &[
            ("header-rule", "#6c919e"),
            ("header-ink", "#f5f6f2"),
            ("focus", "#ae7222"),
            ("note-rule", "#7896a2"),
            ("button-border", "#7a94a1"),
            ("button-bg", "#edf2f2"),
            ("button-fill", "#245b81"),
            ("button-hover", "#e8edef"),
            ("button-active", "#203440"),
            ("surface", "#e8edef"),
            ("code-bg", "#eef1ef"),
        ],
        icon: include_str!("../assets/holodeck.svg"),
    },
    Project {
        id: "umie",
        name: "UMIE",
        protocol: false,
        subtitle: "Modular inference engine",
        page: Some("projects/umie.html"),
        url: "https://fungos.dev/projects/umie.html",
        colors: [
            "#eeeaf2", "#d7d1df", "#302738", "#64596d", "#b7adbf", "#65487d", "#40304e",
        ],
        accents: &[
            ("header-rule", "#9e85b4"),
            ("header-ink", "#f7f1fc"),
            ("focus", "#986b2c"),
            ("note-rule", "#9e85b4"),
            ("button-border", "#9e85b4"),
            ("button-bg", "#eeeaf2"),
            ("button-fill", "#65487d"),
            ("button-hover", "#e0d7e9"),
            ("button-active", "#40304e"),
            ("surface", "#e0d7e9"),
            ("code-bg", "#e0d7e9"),
        ],
        icon: include_str!("../assets/umie.svg"),
    },
    Project {
        id: "dcp",
        name: "DCP",
        protocol: true,
        subtitle: "Decision Catalog Protocol",
        page: None,
        url: "https://dcp.fpl.dev/",
        colors: [
            "#f2f0e8", "#d9ff43", "#10110f", "#595b50", "#242520", "#0c79d8", "#10110f",
        ],
        accents: &[
            ("header-rule", "#d9ff43"),
            ("header-ink", "#f2f0e8"),
            ("focus", "#0c79d8"),
            ("note-rule", "#d9ff43"),
            ("button-border", "#242520"),
            ("button-bg", "#f2f0e8"),
            ("button-fill", "#10110f"),
            ("button-hover", "#d9ff43"),
            ("button-active", "#242520"),
            ("surface", "#e6e4db"),
            ("code-bg", "#e6e4db"),
        ],
        icon: include_str!("../assets/dcp.svg"),
    },
    Project {
        id: "unibus",
        name: "Unibus",
        protocol: true,
        subtitle: "Granted message transport",
        page: Some("projects/unibus.html"),
        url: "https://fungos.dev/projects/unibus.html",
        colors: [
            "#e7eef0", "#cadadc", "#223b40", "#49666b", "#9bb7bc", "#216671", "#21484f",
        ],
        accents: &[
            ("header-rule", "#76aeb7"),
            ("header-ink", "#eff8f8"),
            ("focus", "#986b2c"),
            ("note-rule", "#76aeb7"),
            ("button-border", "#76aeb7"),
            ("button-bg", "#e7eef0"),
            ("button-fill", "#216671"),
            ("button-hover", "#d7e6e8"),
            ("button-active", "#21484f"),
            ("surface", "#d7e6e8"),
            ("code-bg", "#d7e6e8"),
        ],
        icon: include_str!("../assets/unibus.svg"),
    },
];
pub(crate) fn project(id: &str) -> Result<&'static Project> {
    PROJECTS
        .iter()
        .find(|p| p.id == id)
        .ok_or("Unknown project".into())
}
impl Project {
    pub(crate) fn home_url(&self, prefix: &str) -> String {
        match self.page {
            Some(page) => format!("{prefix}{page}"),
            None if self.protocol => self.url.into(),
            None => format!("{prefix}index.html"),
        }
    }
    fn documentation_page(&self) -> Option<&'static str> {
        if self.id == "dcp" {
            Some("projects/dcp.html")
        } else {
            self.page
        }
    }
}
fn stylesheet() -> String {
    let mut css =
        String::from("/* Generated by fpl-project-theme; edit its source repository. */\n");
    for p in &PROJECTS {
        css += &format!("html[data-project=\"{}\"] {{", p.id);
        for (name, color) in ["paper", "page", "ink", "muted", "line", "link", "header"]
            .iter()
            .zip(p.colors)
        {
            css += &format!("--{name}:{color};");
        }
        for (name, color) in p.accents {
            css += &format!("--{name}:{color};");
        }
        css += "}\n";
        css += &format!(
            ".project-{} {{ --project-header:{};--project-page:{};--project-paper:{};--project-ink:{}; }}\n",
            p.id, p.colors[6], p.colors[1], p.colors[0], p.colors[2]
        );
    }
    css += include_str!("../assets/base.css");
    css
}
pub(crate) fn dropdown(active: &Project, prefix: &str) -> String {
    let mut html = format!(
        "<details class=\"project-switcher\">\n<summary class=\"brand project-trigger\" aria-label=\"{} — switch project\"><img src=\"{prefix}theme/{}.svg\" width=\"30\" height=\"30\" alt=\"\">{}<span class=\"project-chevron\" aria-hidden=\"true\">⌄</span></summary>\n<nav class=\"project-menu\" aria-label=\"Choose an FPL project\">\n",
        active.name, active.id, active.name
    );
    let mut protocol_heading = false;
    for p in &PROJECTS {
        let (url, current, check) = if p.id == active.id {
            (active.home_url(prefix), " aria-current=\"page\"", " ✓")
        } else {
            (
                if active.id == "fungos" || active.page.is_some() {
                    match p.page {
                        Some(page) => format!("{prefix}{page}"),
                        None if p.id == "fungos" => format!("{prefix}index.html"),
                        _ => p.url.into(),
                    }
                } else {
                    p.url.into()
                },
                "",
                "",
            )
        };
        if p.protocol && !protocol_heading {
            html += "<div class=\"project-group\">Protocols</div>\n";
            protocol_heading = true;
        }
        let subtitle = if p.subtitle.is_empty() {
            String::new()
        } else {
            format!("<small class=\"project-kind\">{}</small>", p.subtitle)
        };
        html += &format!(
            "<a class=\"project-option project-{}\" href=\"{url}\"{current}><img src=\"{prefix}theme/{}.svg\" width=\"30\" height=\"30\" alt=\"\"><span>{}{check}{subtitle}</span></a>\n",
            p.id, p.id, p.name
        );
    }
    html += &format!("</nav><script defer src=\"{prefix}theme/switcher.js\"></script>\n</details>");
    html
}
fn render_page(source: &str, active: &Project, prefix: &str) -> Result<String> {
    let (start, end) = if let Some(marker) = source.find("class=\"project-switcher\"") {
        let start = source[..marker]
            .rfind("<details")
            .ok_or("Malformed switcher")?;
        let end = source[marker..]
            .find("</details>")
            .ok_or("Unclosed switcher")?
            + marker
            + 10;
        (start, end)
    } else {
        let start = source
            .find("<a class=\"brand\"")
            .ok_or("Missing project brand")?;
        let end = source[start..].find("</a>").ok_or("Unclosed brand")? + start + 4;
        (start, end)
    };
    let mut result = format!(
        "{}{}{}",
        &source[..start],
        dropdown(active, prefix),
        &source[end..]
    );
    let html = result.find("<html").ok_or("Missing html element")?;
    let close = result[html..].find('>').ok_or("Unclosed html element")? + html;
    if !result[html..close].contains("data-project=") {
        result.insert_str(close, &format!(" data-project=\"{}\"", active.id));
    } else if !result[html..close].contains(&format!("data-project=\"{}\"", active.id)) {
        return Err("Mismatched project theme".into());
    }
    Ok(result)
}
fn pages(root: &Path) -> Result<Vec<PathBuf>> {
    let mut pending = vec![root.to_owned()];
    let mut pages = vec![];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                return Err("Site contains symlinks; refusing traversal".into());
            }
            if kind.is_dir() {
                pending.push(entry.path())
            } else if entry.path().extension().is_some_and(|e| e == "html") {
                pages.push(entry.path())
            }
        }
    }
    pages.sort();
    Ok(pages)
}
// Stable asset paths are served immutable by Fab. A revision in the URL prevents
// new HTML from combining with an old browser/CDN stylesheet or script.
fn version_assets(source: &str, revision: &str) -> String {
    let mut result = source.to_owned();
    for extension in [".css", ".js"] {
        let mut offset = 0;
        while let Some(found) = result[offset..].find(extension) {
            let end = offset + found + extension.len();
            let tail = &result[end..];
            let old_end = if tail.starts_with("?v=") {
                end + tail.find(['\"', '\'']).unwrap_or(tail.len())
            } else { end };
            if result[old_end..].starts_with(['\"', '\'']) {
                let query = format!("?v={revision}");
                result.replace_range(end..old_end, &query);
                offset = end + query.len();
            } else { offset = end; }
        }
    }
    result
}
fn sync(path: &Path, bytes: &[u8], check: bool) -> Result<()> {
    if check {
        if fs::read(path)? != bytes {
            return Err(format!("Theme drift: {}", path.display()).into());
        }
    } else {
        fs::create_dir_all(path.parent().ok_or("Invalid output")?)?;
        let next = path.with_extension("theme-next");
        fs::write(&next, bytes)?;
        fs::rename(next, path)?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut site = None;
    let mut id = None;
    let mut check = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--site" => site = args.next(),
            "--project" => id = args.next(),
            "--check" => check = true,
            _ => {
                return Err(
                    "Usage: fpl-project-theme --site DIRECTORY --project holodeck|fungos [--check]"
                        .into(),
                );
            }
        }
    }
    let site = fs::canonicalize(site.ok_or("Missing --site")?)?;
    let active = project(&id.ok_or("Missing --project")?)?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let revision = Command::new("git")
        .args([
            "-C",
            source.to_str().ok_or("Non-UTF8 theme path")?,
            "rev-parse",
            "HEAD",
        ])
        .output()?;
    if !revision.status.success() {
        return Err("Theme must be a committed checkout".into());
    }
    let clean = Command::new("git")
        .args([
            "-C",
            source.to_str().ok_or("Non-UTF8 theme path")?,
            "diff",
            "HEAD",
            "--exit-code",
            "--quiet",
        ])
        .status()?;
    if !clean.success() {
        return Err("Commit tracked theme changes before installing or checking a revision".into());
    }
    let revision = String::from_utf8(revision.stdout)?.trim().to_owned();

    let pages = pages(&site)?;
    if pages.is_empty() {
        return Err("No HTML pages".into());
    }
    // Validate every page before writing.
    let rendered = pages
        .iter()
        .map(|p| {
            let prefix = "../".repeat(p.strip_prefix(&site)?.components().count() - 1);
            let relative = p
                .strip_prefix(&site)?
                .to_str()
                .ok_or("Non-UTF8 page path")?;
            let page_project = if active.id == "fungos" {
                PROJECTS
                    .iter()
                    .find(|project| project.documentation_page() == Some(relative))
                    .unwrap_or(active)
            } else {
                active
            };
            Ok((
                p,
                version_assets(&render_page(&fs::read_to_string(p)?, page_project, &prefix)?, &revision),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    sync(
        &site.join("theme/theme.css"),
        stylesheet().as_bytes(),
        check,
    )?;
    sync(
        &site.join("theme/switcher.js"),
        include_bytes!("../assets/switcher.js"),
        check,
    )?;
    sync(
        &site.join("theme/docs.js"),
        include_bytes!("../assets/docs.js"),
        check,
    )?;
    for p in &PROJECTS {
        sync(
            &site.join(format!("theme/{}.svg", p.id)),
            p.icon.as_bytes(),
            check,
        )?;
    }
    sync(
        &site.join("theme/LICENSE"),
        include_str!("../LICENSE").as_bytes(),
        check,
    )?;
    sync(
        &site.join("theme/ASSET-NOTICES.md"),
        include_str!("../ASSET-NOTICES.md").as_bytes(),
        check,
    )?;
    sync(
        &site.join("theme/REVISION"),
        format!("ajmwagar/fpl-project-theme\n{revision}\n").as_bytes(),
        check,
    )?;
    for (path, html) in rendered {
        sync(path, html.as_bytes(), check)?;
    }
    // Consumer stylesheets can import the shared theme; version those imports too.
    for name in ["distro.css", "style.css", "styles.css"] {
        let path = site.join(name);
        if path.exists() {
            let css = fs::read_to_string(&path)?;
            sync(&path, version_assets(&css, &revision).as_bytes(), check)?;
        }
    }
    println!(
        "{} {} pages for {} at {}",
        if check { "Checked" } else { "Updated" },
        pages.len(),
        active.id,
        revision
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_projects_are_local_and_peers_are_absolute() {
        let p = project("fungos").unwrap();
        let html = dropdown(p, "../");
        assert!(html.contains("href=\"../index.html\" aria-current"));
        assert!(html.contains("href=\"https://holodeck.fpl.dev/\""));
        assert!(html.contains("../theme/fungos.svg"));
        assert!(html.contains("href=\"https://dcp.fpl.dev/\""));
        assert!(html.contains("project-group\">Protocols"));
        assert!(html.contains("Decision Catalog Protocol"));
        assert!(html.contains("../theme/dcp.svg"));
        assert!(!html.contains("project-palette"));
        assert!(html.find("project-fungos").unwrap() < html.find("project-holodeck").unwrap());
        assert!(html.contains("href=\"../projects/umie.html\""));
        assert!(html.find("Protocols</div>").unwrap() < html.find("project-unibus").unwrap());
        assert!(html.contains("Granted message transport"));
    }
    #[test]
    fn install_is_idempotent() {
        let original = "<html lang=\"en\"><a class=\"brand\" href=\"#\">x</a></html>";
        let p = project("holodeck").unwrap();
        let once = render_page(original, p, "").unwrap();
        assert_eq!(render_page(&once, p, "").unwrap(), once);
        assert!(once.contains("data-project=\"holodeck\""));
    }
    #[test]
    fn immutable_assets_get_current_revision() {
        let source = "<link href=\"../theme/theme.css\"><script src=\"theme/docs.js?v=old\"></script>";
        let once = version_assets(source, "new");
        assert!(once.contains("theme.css?v=new\""));
        assert!(once.contains("docs.js?v=new\""));
        assert_eq!(version_assets(&once, "new"), once);
        assert_eq!(version_assets("@import url(\"theme/theme.css\");", "new"), "@import url(\"theme/theme.css?v=new\");");
    }
    #[test]
    fn project_pages_keep_their_identity_and_local_links() {
        let p = project("umie").unwrap();
        let html = dropdown(p, "../");
        assert!(html.contains("href=\"../projects/umie.html\" aria-current"));
        assert!(html.contains("href=\"../index.html\""));
        assert!(html.contains("href=\"../projects/unibus.html\""));
        let original = "<html data-project=\"umie\"><a class=\"brand\" href=\"#\">UMIE</a></html>";
        assert!(
            render_page(original, p, "../")
                .unwrap()
                .contains("UMIE — switch project")
        );
        assert!(render_page(original, project("fungos").unwrap(), "../").is_err());
        let dcp = project("dcp").unwrap();
        assert_eq!(dcp.documentation_page(), Some("projects/dcp.html"));
        assert!(dropdown(dcp, "../").contains("href=\"https://dcp.fpl.dev/\" aria-current"));
    }
    #[test]
    fn unknown_and_malformed_fail() {
        assert!(project("other").is_err());
        assert!(render_page("<html></html>", project("fungos").unwrap(), "").is_err());
    }
}
