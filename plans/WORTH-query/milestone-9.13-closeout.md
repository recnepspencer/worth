# Milestone 9.13 Closeout: Declarative Query Experience

Core Phases 1-12 closed on 2026-07-14 for the runtime-backed declarative
product boundary. Add-on Phases 13-20 closed on 2026-07-15 under the amendment
below. Add-on Phases 21-30 closed on 2026-07-15 with Foundational-native
aspect values preserved through Relational, Query, Worth UI, and Hadwiger.

## Closed Boundary

Ordinary Query usage is capability-oriented and declarative across ten
namespaces:

- `facade::read`
- `facade::aggregate`
- `facade::live`
- `facade::history`
- `facade::comparison`
- `facade::preview`
- `facade::mutation`
- `facade::workflow`
- `facade::inspection`
- `facade::domain`

Consumers declare desired meaning, attach typed context, and run or open the
capability. Query owns canonicalization, admission, planning, lower-runtime
routing, execution, managed lifecycle, receipts, and typed outcomes. Internal
phase artifacts remain observable only where required for evidence or
certification; ordinary consumers cannot construct or independently advance
them.

Workspace-owned runtime evidence is inspected through the dedicated
`workspace.inspections()` capability lane. Its unified `inspect(...)` and
admitted `inspect_intent(...)` paths keep runtime ownership intact without
reopening a general workspace forwarding API. External trybuild transcripts
prove that downstream consumers can use this lane after the runtime has moved
into its workspace.

The aggregate family is a separate ordinary namespace. Its current shipped
operation is count over an admitted collection declaration, producing
`WorthQueryCountOutcome` and a receipt-backed `WorthQueryCountResult`.

## Closure Evidence

The original Phase 12 closeout used a source-backed product-boundary bundle.
Milestone 9.13.1 retired that bundle, its golden source digests, and its test-
source registry because production code must not certify that tests or source
tokens exist. The retained ordinary tests cover the product behavior directly:

- complete public capability journeys across all ten namespaces
- equivalent declaration convergence and ordinary/internal oracle parity
- cross-capability, cross-basis, stale-context, and receipt-promotion denial
- one-shot/live parity, historical ambiguity, preview/workflow denial, and
  diagnostic-policy equivalence
- managed live open/close lifecycle evidence
- exact-zero planning/runtime work for invalid context and bounded ergonomic
  lowering under unrelated workspace growth
- hard prohibitions, residue audits, sabotage cases, and consumer-owned
  adoption

The documentation closeout additionally proves that all ten ordinary
namespaces are discoverable, the read/projection/inspection and aggregate
examples execute, ordinary collection guidance does not expose internal
planner functions, ordinary projection guidance does not require manual
authority assembly, and relative Markdown links under `crates/worth-query/docs`
resolve.

## Verification Run

The closeout was rechecked on 2026-07-14 with:

- `cargo fmt -p worth-query -- --check`
- `cargo test -p worth-query declarative_facade_docs --lib -- --nocapture`
  (5 passed)
- `cargo test -p worth-query declarative_product_boundary_certification --lib -- --quiet`
  (29 passed)
- `cargo test -p worth-query declarative_product_boundary_compile_fail --lib -- --nocapture`
  (2 passed)
- `cargo test -p worth-query intent_admission_doc --lib -- --nocapture`
  (2 passed)
- `cargo test -p worth-query --lib --no-run`
  (library and test target compiled)

The crate still emits pre-existing unused/dead-code warnings during these
builds. They are not represented as documentation or product-boundary
failures.

## Add-on Phases 13-20 Closeout

Runtime-installed domain authority is the only supported route from domain
package setup to handle-bound execution, contributions, projection, lifecycle,
and inspection. Query validates and canonicalizes the package, compiles every
substrate before publication, mints runtime-affine handles, derives indexed
execution state, and rejects foreign-runtime or stale-generation authority
before planning or lower-runtime work.

Hadwiger Research and Worth UI are real reference consumers of that boundary.
Both install typed packages and obtain handles from the owning workspace.
Hadwiger exercises installed read, workflow, contribution, and research-graph
invariant behavior. Worth UI now exposes measurement vocabulary only as an
extension of `WorthQueryInstalledDomainHandle<WorthUiDomainEntry>`; its former
free read/live declaration route was removed. Its executable consumer proof
covers a package-registered graph read, projection consumption, linked
inspection, installed live lifecycle, workflow promotion, installed invariant
obligation, and invariant-catalog contribution.

The hostile review found and corrected one material Phase 20 omission: Worth
UI was initially present only in the source residue audit. The canonical
evidence manifest now includes both executable Worth UI journey probes, and
the reference-consumer adoption and journey registries point to the installed-
handle extension rather than the deleted free declaration helper.

The add-on closeout was rechecked on 2026-07-15 with:

- `cargo test -p worth-query domain_installation --lib` (40 passed)
- `cargo test -p worth-query domain_authority_inventory --lib` (7 passed)
- `cargo test -p worth-query installed_domain --lib` (8 passed, including 23
  compile-fail boundary cases)
- `cargo test -p worth-query milestone_nine_thirteen_installed_domain_certification --lib`
  (2 passed)
- `cargo test -p worth-ui-query-binding --lib` (10 passed)
- focused Hadwiger installed read, workflow/contribution, and invariant-package
  tests (3 passed)
- `cargo check --workspace` in `workspaces/worth-ui`
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`
- `cargo test -p worth-query --lib -- --test-threads=1` (3,086 passed)
- `cargo test -p worth-query intent_admission_dx_boundaries_hold --lib -- --test-threads=1`
  (the public inspection transcript plus all intent-admission boundaries
  passed)

The installed-domain certification bundle reports closed with zero authority
findings, zero missing compile-fail boundaries, and zero missing consumer
residue classes. No commit, push, or merge is part of this closeout amendment;
the breaking milestone work remains isolated on `ui`.

## Add-on Phases 21-30 Closeout

Foundational owns the native aspect-value grammar, validation, canonical
identity, portable candidates, and authoritative readmission. Relational
accepts native entity and relation patches, applies them under one conflict
and atomic-publication law, and preserves exact typed state through canonical
encoding and checkpoint recovery. Query authors, validates, compares,
projects, retains, and observes the same values without introducing a coarse
scalar union, JSON-shaped authority, or a second identity encoder.

Ordinary projection consumption exposes a borrowed native view with exact
scalar refinements and complete struct values. Worth UI consumes native
measurement facts through its installed domain handle. Hadwiger authors and
reads its installed-domain fields through the same native contract-backed
path.

The hostile review found and corrected material defects and evidence gaps:

- bridge projection split a dotted whole-aspect key as though it were one
  field; whole struct values now remain structurally intact
- live projection accepted only scalar facts; live and retained paths now
  preserve complete struct values
- Hadwiger still modeled a Query-local schema field kind; it now uses the
  Foundational scalar type directly
- the ordinary read facade omitted the native projection view and refinement
  types; the public facade and its authority inventory now agree
- Relational lacked direct native conflict, permutation, mixed entity/relation
  atomicity, and struct/reference/clear recovery proof; those matrices are now
  explicit
- a lower-runtime exact-basis fixture minted a synthetic snapshot identity;
  it now derives the identity from the canonical Relational branch head through
  the Runtime Bridge conversion
- graph-access tests described ordinary collections as unordered even though
  ordinary admission guarantees `identity.id ASC`; the evidence now proves the
  admitted canonical ordering and distinguishes it from explicit ordering
- certification and public Runtime Bridge harnesses authored native fields
  without installing their Foundational contracts; contracts are now installed
  at runtime construction and admission remains strict
- oversized runtime and hostile-certification support files were split by
  subscription, schedule, closure, identity, inventory, schema-fixture, and
  live-inspection responsibility
- product guides still used `serde_json::Value` as the ordinary live/computed
  row marker; they now use `WorthQueryUnrefinedLiveShape`, and the native-value
  certification scans every Worth Query product doc so displaced consumer row
  carriers cannot hide outside the three primary native-value pages

The Phase 21-30 bundle is assembled by
`certify_milestone_nine_thirteen_native_values()` from a source-backed evidence
manifest. Closure requires all ten phase rows, all 26 Foundational value
families, zero authority-inventory findings, zero consumer residue, zero
documentation disagreement, the complete compile-fail fixture set, and
non-empty canonical evidence digests.

The add-on closeout was rechecked on 2026-07-15 with:

- `cargo fmt --all -- --check`
- `cargo test -p worth-foundational portable -- --test-threads=1` (3 library,
  11 certification, and 1 compile-boundary test passed)
- `cargo test -p worth-relational native_ --lib -- --test-threads=1` (20 passed)
- `cargo test -p worth-query native_ --lib -- --test-threads=1` (56 passed,
  including the compile-fail harness)
- `cargo test -p worth-query projection_consumption:: --lib -- --test-threads=1`
  (111 passed)
- `cargo test -p worth-query aspect_native_query_boundaries_are_compile_time_enforced --lib -- --test-threads=1`
  (all 216 compile-fail fixtures passed)
- `cargo test -p worth-query milestone_nine_thirteen_native_value_certification --lib -- --test-threads=1`
  (3 passed)
- `cargo test -p worth-query graph_read_access_shape --lib -- --test-threads=1`
  (8 passed)
- `cargo test -p worth-query lower_runtime_routing::certification::surface --lib -- --test-threads=1`
  (13 passed)
- `cargo test -p worth-query hostile_certification --lib -- --test-threads=1`
  (22 passed)
- `cargo test -p worth-query concurrent_hostile_matrix --lib -- --test-threads=1`
  (3 passed)
- `cargo test -p worth-query declarative_facade_docs --lib -- --test-threads=1`
  (6 passed)
- `cargo test -p hadwiger-research query_entry -- --test-threads=1` (2 installed
  query-entry tests passed)
- `cargo test --manifest-path workspaces/worth-ui/Cargo.toml -p worth-ui-query-binding --lib -- --test-threads=1`
  (10 passed)
- `cargo check --manifest-path workspaces/worth-ui/Cargo.toml --workspace`
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`

The final in-process line-cap audit covered 11,576 tracked, changed, or
untracked Rust files under the workspace crate roots. Every one is at or below
400 lines or is present in the explicit CI allowlist; the audit reported zero
violations. The Bash guard was also repaired to ignore tracked paths deleted by
the current cutover instead of passing them to `wc`.

The documentation gate also exposed a stale executable fixture whose aspect
mappings lacked installed Foundational contracts. The fixture now installs
the same contract-backed schema that the ordinary documentation promises.

## Claims Deliberately Not Made

This closeout does not claim:

- store-backed execution or pushdown parity
- durable restore, saved-query survival, or restart-stable continuation
- a general ordinary collection page-size/cursor continuation API
- durable cursor persistence
- blob-backed delivery
- completion of Milestones 10, 11, 12, or 13

Runtime-backed cursor substrate and graph-access streaming cursors exist, but
the ordinary read/count journey does not currently expose a general paged
collection continuation contract. Store-backed parity belongs to Milestone 10;
durable artifacts and continuations belong to Milestone 11.

## Discovery Contract

Product discovery starts at:

- [`crates/worth-query/docs/AI_README.md`](../../crates/worth-query/docs/AI_README.md)
- [Declarative Query Experience](../../crates/worth-query/docs/capabilities/declarative-query-experience.md)
- [Collections, Ordering, Aggregates, And Cursors](../../crates/worth-query/docs/authoring/collections-cursors-ordering-and-aggregations.md)
- [Native Aspect Values](../../crates/worth-query/docs/capabilities/native-aspect-values.md)
- [Projection Consumption](../../crates/worth-query/docs/capabilities/projection-consumption.md)

These documents teach the current product surface. Historical phase assembly
belongs in milestone and certification records, not ordinary discovery.
