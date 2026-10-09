# Shared publication workflow

Use the `fpl-project-theme` README as the CLI contract. The examples below assume
sibling site checkouts; substitute discovered paths. A manifest can be narrowed
with a temporary file containing only selected existing rows, in the same
manifest directory so relative paths retain their meaning. Do not rewrite the
maintained manifest merely to run a focused update.

From the theme checkout, after committing canonical Markdown:

```sh
cargo run --bin docs -- --manifest ../fungos-web/docs.tsv
cargo run -- --site ../fungos-web/dist --project fungos
cargo run --bin docs -- --manifest ../fungos-web/docs.tsv --check --fail-on-stale
cargo run -- --site ../fungos-web/dist --project fungos --check
```

For Holodeck substitute `../holodeck/website-docs.tsv`,
`../holodeck/website`, and `--project holodeck`.

The renderer accepts a single page's explicit arguments too; use that form when
it avoids rerendering unrelated guides. A row has seven tab-separated columns:
source Markdown, output HTML, site root, project, source repository HTTPS URL,
code checkout and code repository HTTPS URL. A `-` denotes an absent optional
code field. The code checkout is independent from the site repository: using the
site's HEAD as UMIE's review baseline is incorrect.

## Provenance and links

- Latest docs change is the latest commit touching canonical Markdown, including
  its real subject and date. Generated HTML/export commits do not advance it.
- `source_commit` is a full reviewed code SHA; `watches` lists tracked relative
  files/directories. Baselines must resolve and be ancestors of the code checkout.
- Changed commits and dirty watched files are shown as review warnings. Plain
  `--check` checks the rendered warning too; adding `--fail-on-stale` rejects it.
- Rendering pins relative Markdown links to the canonical source repository at
  the docs revision. Use explicit website URLs for website navigation; a source
  link to nonexistent sibling `.html` is not a website link. Check source-link
  targets as well as local output paths and page anchors.
- The generated `.md` preserves the canonical file byte for byte, including
  frontmatter and tables. The renderer escapes raw embedded HTML and hides
  catalog comment markers. Treat exported Markdown as the LLM contract.

## Verification and publication

When publisher code or controls change, use the cheap check while iterating,
then its focused/full relevant tests once complete:

```sh
cargo check
cargo test
node --test tests/docs-copy.cjs
```

For content-only updates, verify generation/drift, meaningful example commands,
local links and source/export byte equality. Serve the site with its existing
preview server. Check HTML and `.md` responses. If a browser is available, verify
Copy for LLM copies the table and frontmatter, succeeds on a secure origin, and
shows a download fallback on permission or fetch errors. Mocked clipboard tests
are not a live browser test; downloading the Markdown remains available.

Run the site's existing static-artifact build gate before authorized publishing.
Commit the generated files and publication runbook changes separately from
canonical source. Inspect remote/branch history before pushing. Verify hosted
HTML, Markdown and `theme/REVISION` through the supported hosting workflow; if
that access is unavailable, report the local/pushed result without claiming a
live deployment. Restore a previous canonical source revision and regenerate
for rollback; do not patch generated HTML manually.
