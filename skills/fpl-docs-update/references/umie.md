# UMIE catalog and model-support documentation

Read UMIE's `docs/reference/model-catalog.md` and applicable repository
instructions first. UMIE owns recipe enumeration and backend resolution in
`crates/umie-spec/examples/model_catalog.rs`; `scripts/catalog-models` invokes it.
The generator produces `docs/reference/model-recipes.json` and
`docs/reference/model-support.md`.

From the discovered UMIE checkout, use the actual site path:

```sh
scripts/catalog-models --site-source /checkout/fungos-web/docs-source/umie.md
scripts/catalog-models --site-source /checkout/fungos-web/docs-source/umie.md --check
```

The public overview has exactly one ordered pair of
`<!-- umie-model-catalog:start -->` and `<!-- umie-model-catalog:end -->` markers.
The generator replaces only that region and rejects invalid markers. Its check
validates both owner outputs and the site projection. It does not update reviewed
metadata. Commit relevant owner outputs and canonical site Markdown before using
the shared publisher. Review branch history before pushing UMIE; unrelated
unpublished implementation work is not part of a documentation task.

## Keep evidence distinct

- **Recipe present:** a specification exists. This alone proves no compatibility.
- **Schema validated:** the existing bounded resolver accepted a typed plan for
  the catalog's stated synthetic host and default inputs. No weights are loaded.
- **Recorded runtime operation:** a checked-in report identifies an operation,
  backend and encoding at its historical provenance. A report's commit date is
  not its measurement date. Timing is not semantic parity or a current rerun.
- **Hardware verified:** requires the owning checkpoint/parity tests on declared
  hardware, weights and encodings. The catalog does not assign this status.

Do not expand family support from one model, text support into vision support,
or primitive kernels into full-model generation. Incubator fixtures establish
metadata/schema consistency. Unresolved context-dependent recipes are not
necessarily unsupported by their subsystem. Cite the actual evidence and retain
limitations for quantization, operators, context, checkpoint admission and memory.

After changing catalog code or recipes, run the relevant existing focused gates:

```sh
cargo check -p umie-spec --example model_catalog
cargo test -p umie-spec --example model_catalog
cargo test -p umie-spec --test incubator_catalog
```

Honor owner-required broader gates; if environment/access/resource limits prevent
one, state that gap and track it through the owner's task system. Do not perform
expensive hardware runs unless the task requires them. Add reviewed evidence to
its owner rather than inserting unsupported claims directly into the website.
