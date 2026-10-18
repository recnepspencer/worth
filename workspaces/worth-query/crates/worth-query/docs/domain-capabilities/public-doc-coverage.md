# Domain Capability Documentation Certification

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

This page records what checks documentation against code today, and what does
not. There is no lane that compiles every documentation example or audits the
guides against the public API. The consumer boundary audit that once did part
of this was removed on 2026-08-23 (see
[Hard Prohibitions](../foundations/hard-prohibitions.md)).

## What Exists Today

| Check | Where | What it proves |
|---|---|---|
| Domain-capability public surface inventory | `worth_query_domain_capability_public_surface_inventory()` in `src/domain_capabilities/certification/surface/mod.rs` | A static list of certified call shapes, one row per category. It is a list of strings; it does not compile them. |
| Certification surface digest | `worth_query_domain_capability_certification_surface()` in `src/domain_capabilities/certification/certification_surface.rs` | Hashes that inventory into `public_surface_digest()` and `certification_surface_digest()` and reports `category_count()` (the number of inventory rows). A changed row changes the digest. |
| Compiled guide examples | `compiled_example_drift.rs` in `worth-query-certification/fixtures/consumer_entry/topology_entry/src/` | The four blocks in [`docs/build-an-application.md`](../../../../../../docs/build-an-application.md) that follow a `<!-- compiled: X -->` marker match, byte for byte, a marked region of a fixture source that compiles. The test also fails if a marker is removed. |
| Facade export snapshot | `tools/boundary-check/snapshots/facades.toml` | The exported names of `worth-query-decl`, `worth-query-host`, and the other snapshotted facades. Boundary-check fails on drift. |
| Rustdoc examples | Doc comments in crate source, for example the `compile_fail` blocks in `worth-query-host/src/facade.rs` | Doctests compile when that crate's doctests run. |
| Front-door guide as rustdoc | `#![doc = include_str!(..)]` in `workspaces/worth-query-bank-world/crates/bank-estate-certification/src/lib.rs` | The [application front door](../foundations/ordinary-application-front-door.md) is that crate's rustdoc, so its `rust` and `rust,no_run` blocks compile when that crate's doctests run. CI does not run them. |

## What Does Not Exist

- `certify_milestone_nine_thirteen_installed_domain(...)` and
  `certify_milestone_nine_thirteen_native_values(...)` are not in the source.
- No CI check compiles the other Rust blocks in the guides. Apart from the
  front door's non-`ignore` blocks above, no check compiles the blocks under
  `workspaces/worth-query/crates/worth-query/docs/`. Those names are checked
  by review only.
- No check compares a guide's prose with the public types it names.

When a guide and the compiled source disagree, the source wins. See
[Which source wins](../../../../../../docs/capability-index.md#which-source-wins).

## Small Example

```rust
let surface = worth_query_domain_capability_certification_surface();
assert!(surface.category_count() > 0);
```

## How It Relates To Other Features

- [Runtime-Installed Domains](./runtime-installed-domains.md) is the canonical
  installed-domain guide.
- [Native Aspect Values](../capabilities/native-aspect-values.md) is the
  canonical value-authority guide.
- [Goldens, Boundaries, And Hostile Certification](./certification/goldens-boundaries-and-hostile-certification.md)
  explains the proof layers.

## Inspection And Debugging

When `compiled_examples_match_build_guide` fails, it names the example and the
first differing line. Change the fixture source and the guide block together.
Do not use certification artifacts as runtime authority.

## Anti-Patterns

- treating file existence or link resolution as semantic documentation proof
- creating a doc-specific test when an existing public integration journey is
  already the stronger evidence
- documenting an internal transition helper as an ordinary consumer entry

## Current Limits

Only the four marked blocks of the build guide are compile-checked in CI. The
inventory digest detects a changed inventory row, not a changed API. Neither is
a substitute for runtime support admission or performance evidence.

## Related Docs

- [Certification Surface And Closeout Bundle](./certification/certification-surface-and-closeout-bundle.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
