# FPL project theme

Shared static project-site styles and an accessible project switcher for Holodeck
and fungOS. Each project keeps its identity through theme tokens and its logo.

- `assets/base.css` owns layout and typography; `assets/switcher.js` owns menu dismissal.
- `src/main.rs` owns the project registry, URLs and color palettes.
- `assets/*.svg` holds the canonical switcher logos with separate notices.
- The Rust installer generates a revision-pinned `theme/` directory and replaces
  the brand component on every HTML page, including nested documentation pages.

No external font, runtime theme service or build dependency is required by a consuming site. Generated files are distribution snapshots;
edit this repository and regenerate rather than maintaining parallel CSS.

## Apply / verify

From a committed checkout:

```sh
cargo check
cargo test
cargo run -- --site ../holodeck/website --project holodeck
cargo run -- --site ../fungos-web/dist --project fungos
cargo run -- --site ../holodeck/website --project holodeck --check
cargo run -- --site ../fungos-web/dist --project fungos --check
```

Consumers import `theme/theme.css` from their local stylesheet. Only site-specific
styles belong in that stylesheet. Their static build publishes the generated
files as-is, including `theme/REVISION`; no private repository clone is needed
on the hosting worker. Check out the revision recorded in that file before
checking drift or deliberately updating a site.

The installer validates all HTML headers before writing. It refuses symlink
traversal and unknown project IDs. Individual writes are atomic, but installation
is not a transactional replacement of an entire site; run in a Git checkout and
review the diff. A missing or changed generated file fails `--check` visibly.

## Publication runbook

1. Edit and commit the theme source, then run its tests.
2. Generate both sites; check both against the committed revision.
3. Review local links and captions, then commit site changes separately.
4. Push consumer sites to their authorized hosting branches.
5. Verify the live stylesheet, switcher and `theme/REVISION` after hosting.

fungOS' website is a published snapshot in `ajmwagar/fungos-web`. If replacing
that snapshot from the distribution source, run the installer again before
publishing; this preserves the single theme owner.

Original theme/tool code: AGPL-3.0-or-later. Logo terms: `ASSET-NOTICES.md`.
The source repository is initially private; the published styles remain readable.

The dropdown groups DCP under **Protocols**, with its official mark, a protocol
subtitle and its own themed card background. DCP links to its
own site; adding it here does not replace or restyle that site.

fungOS appears first, followed by applications and then protocols. UMIE and
Unibus overview pages live in the fungOS consumer at `dist/projects/`. Their
registered paths select their own themes; other pages retain the site theme.
Content belongs to the site repository, while logos, navigation and tokens
belong here. Local project links work on previews as well as the published site.

## Canonical technical documentation

`cargo run --bin docs --` publishes repository-owned CommonMark (tables,
strikethrough, task lists) with real source provenance. It does not crawl websites
or run source examples. Raw embedded HTML renders as text, and headings receive
stable anchors. The source is separate from the generated HTML and adjacent `.md`
export; the latter preserves all original bytes, including author metadata.

Manual runbook (the same commands work in CI):

1. Write/update `docs-source/projects/example.md` in the capability/site owner.
   Validate statements and examples against the owning implementation. Commit the
   canonical Markdown; unrelated dirty repository files do not block publication.
2. Optionally record reviewed code using PedalKernel's convention:

   ```yaml
   ---
   title: Example reference
   source_commit: FULL_40_CHARACTER_REVIEWED_CODE_SHA
   watches:
     - src/interfaces.rs
     - recipes/
   ---
   ```

   The baseline is author evidence, never rewritten by regeneration. Other
   frontmatter belongs to the authoring project; this tool is not Wiki admission
   and does not replace Wiki's visibility or authorization validation.
3. Render from the committed source, identifying the source repository URL and
   the independent code repository explicitly when review metadata is present:

   ```sh
   cargo run --bin docs -- \
     --source /checkout/site/docs-source/projects/example.md \
     --output /checkout/site/dist/projects/example.html \
     --site /checkout/site/dist --project umie \
     --repository-url https://github.com/owner/site \
     --code-repository /checkout/umie \
     --code-url https://github.com/FuturePresentLabs/umie
   ```

4. Run the theme installer for that site's root, which supplies `theme/docs.js`,
   CSS, icons, and consistent navigation. Append `--check` to the docs command
   to reject generated HTML/Markdown drift. Append `--fail-on-stale` to require
   unchanged watched source. Without that flag stale pages render an explicit
   revalidation notice, including changed commits or uncommitted watched changes.
5. Verify published links and examples, commit generated artifacts, and deploy
   through the site's existing hosting pipeline. Roll back via the previous
   canonical source revision and regenerate; do not hand-edit generated output.

Provenance shows the latest commit **affecting that Markdown file**, its subject
and date, and a revision-pinned source link. Commits only affecting generated
output do not advance that provenance. Reviewed code commits must resolve and be
ancestors of the explicit code checkout. Missing/untracked watched paths fail;
changed paths are reported visibly. A directory watch must contain tracked files.
A source file with uncommitted edits or no tracked history fails publication.

Copy for LLM fetches the exact `.md` export on the user's click. Successful copies
are acknowledged; failed requests, permission denial, and insecure contexts
produce a visible error and link to the Markdown download. This is a small browser
UI boundary; all generation/provenance is Rust. No remote requests happen during
reference generation. Tests: `cargo test` and `node --test tests/docs-copy.cjs`.

For repeatable multi-page publication, store a tab-separated manifest in the site
owner. `cargo run --bin docs -- --manifest /site/docs.tsv [--check]
[--fail-on-stale]` accepts seven columns (no header): source Markdown, output HTML,
site root, project ID, source-repository HTTPS URL, code-repository checkout, and
code-repository HTTPS URL. Paths resolve relative to the manifest; `-` means an
absent optional code repository/URL. Lines starting `#` are comments. This avoids
separate generation scripts in each consumer. Each file is written atomically;
run all checks successfully before publishing the complete site.

Rendering rewrites relative Markdown links to their canonical source repository
at the documentation revision; absolute URLs and page anchors remain unchanged.
The `.md` export is never rewritten. Canonical source, not the generated output
folder, defines relative-link meaning. Output traversal through symlinks fails.

## Agent skill

`skills/fpl-docs-update/` packages the documentation maintenance workflow. Install
that directory in your agent's skill location; Codex can invoke `$fpl-docs-update`.
The skill calls the existing publisher and owner catalogs, with separate reviewed
code evidence, generated-file checks and task-authorized publication.
