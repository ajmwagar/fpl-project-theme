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
