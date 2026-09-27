# Milestone 2 Closeout: Schema-Aware Validation, Predicate Legality, And Projection Semantics

## Status

Milestone 2 is closed as of 2026-04-14.

`worth-query` now has a real legality boundary between canonical query meaning
and later planning or execution. Query legality is no longer something the
planner, executor, or host glue "happens to discover later." It is a
proof-bearing phase with sealed artifacts, exact schema-basis identity,
deterministic validation identity, typed rejection, certification artifacts,
and compile-time boundary protection.

The semantic center shipped in this milestone is:

canonical query meaning enters once from Milestone 1, authoritative schema
meaning enters once through a query-owned schema-view boundary, validation owns
projection/predicate/traversal/ordering/result-shape legality exactly once,
legal queries lower into one validated proof chain, illegal queries fail typed
before planning, and the certification matrix proves both parity and rejection
without relying on ambient interpretation.

This is not "filters work now." Milestone 2 made legality itself part of the
type system and the certification surface.

## Shipped Scope

Milestone 2 delivered:

- a query-owned schema-view boundary under
  [crates/worth-query/src/schema_view](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\schema_view)
- proof-bearing validated artifacts under
  [crates/worth-query/src/validation/artifacts](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\artifacts)
- a decomposed validation pipeline under
  [crates/worth-query/src/validation](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation)
  covering:
  - [projection.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\projection.rs)
  - [predicates.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\predicates.rs)
  - [predicate_state](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\predicate_state)
  - [traversal.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\traversal.rs)
  - [ordering.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\ordering.rs)
  - [result_shape.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\result_shape.rs)
  - [pipeline.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation\pipeline.rs)
- canonical predicate and ordering authority threaded through
  [crates/worth-query/src/canonicalization](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\canonicalization)
- schema-derived typed query APIs under
  [crates/worth-query/src/typed](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\typed)
- a schema DSL that derives typed tokens and runtime schema views from one
  declaration in
  [crates/worth-query/src/schema_macro.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\schema_macro.rs)
- Milestone 2 validation certification artifacts under
  [crates/worth-query/src/harness/validation_certification](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_certification)
  and
  [crates/worth-query/src/harness/validation_matrix](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_matrix)
- compile-fail proof-boundary tests under
  [crates/worth-query/tests](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\tests)

## Acceptance Mapping

Milestone 2 is considered closed against
[milestone-2.md](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\plans\worth-query\milestone-2.md)
and
[test-requirements.md](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\plans\worth-query\test-requirements.md)
because the required acceptance surfaces are now covered directly.

### `Schema-Aware Rejection And Projection Legality Test`

Covered by:

- [crates/worth-query/src/harness/validation_certification/mod.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_certification\mod.rs)
- [crates/worth-query/src/harness/validation_certification/tests.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_certification\tests.rs)
- [crates/worth-query/src/harness/validation_matrix](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_matrix)

What is proven:

- the named Milestone 2 certification matrix exists as a first-class closeout
  artifact rather than "coverage somewhere"
- the matrix emits canonical machine-checkable bundles and an aggregate
  `MilestoneTwoValidationCertificationArtifact`
- the required canonical rows and rejection rows exist
- `bundle_completeness_report` proves full Milestone 2 matrix coverage rather
  than partial implementation
- the aggregate certification artifact is deterministic and offline-readable

### `Legal query determinism under exact schema basis`

Covered by:

- `validation_certification` canonical rows:
  - `legal-detail-query-parity`
  - `equivalent-builder-composed-legal-query`
  - `legal-structured-content-query-parity`
  - `legal-workflow-predicate-parity`
  - `schema-basis-variation-boundary`
  - `ordering-only-authority-boundary`
  - `integer-greater-than-predicate-parity`
  - `integer-less-than-predicate-parity`
  - `text-contains-predicate-parity`
  - `scalar-membership-predicate-parity`
  - `presence-predicate-parity`
  - `bounded-range-normalization`
  - `membership-intersection-normalization`
  - `redundant-greater-than-normalization`

What is proven:

- equivalent canonical meaning validates into identical validated identity for
  the same exact schema basis
- exact schema-basis variation changes validated meaning explicitly
- structured-content and workflow legality are admitted only where schema
  authority says they are
- ordering-only authority remains explicit and changes validated meaning
  without smuggling projected data
- predicate normalization is semantic rather than cosmetic

### `Illegal queries fail during validation rather than planning or execution`

Covered by:

- `validation_certification` rejection rows:
  - `unknown-aspect-projection`
  - `incompatible-predicate-family`
  - `illegal-traversal-edge-or-depth`
  - `invalid-result-shape-binding`
  - `structured-content-illegality`
  - `workflow-context-illegality`
  - `forbidden-widening-case`
- direct hostile validation tests under
  [crates/worth-query/src/harness/validation_cases](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_cases)

What is proven:

- unknown aspects and fields fail during validation
- incompatible predicate families fail typed against field-kind authority
- illegal traversal relation/depth requests fail before planning
- invalid result-shape bindings fail before any delivery path
- unsupported structured-content and workflow contexts fail explicitly
- forbidden widening is rejected rather than repaired

### `Validation rejection matrix and exact counters`

Covered by:

- `ValidationFailureArtifact` and `ValidationRejectionMatrix` emission in
  [crates/worth-query/src/validation](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation)
- exact counter and report assertions in
  [crates/worth-query/src/harness/validation_cases](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_cases)
- aggregate certification accounting in
  [crates/worth-query/src/harness/validation_matrix/completeness.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\harness\validation_matrix\completeness.rs)

What is proven:

- rejection bundles emit `failure_digest`, `validation_rejection_matrix`, and
  `counter_snapshot`
- supported lanes preserve zero forbidden fallback
- supported lanes preserve zero forbidden widening residue
- the certification artifact proves exact row and lane completeness rather than
  relying on visual inspection

### `Proof-bearing validated boundary and compile-time protection`

Covered by:

- public sealed validated artifact construction through
  [crates/worth-query/src/validation](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\src\validation)
- compile-fail tests:
  - [private_validated_query_artifact_fields.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\tests\ui\private_validated_query_artifact_fields.rs)
  - [private_validated_query_bundle_fields.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\tests\ui\private_validated_query_bundle_fields.rs)
  - [typed_contains_requires_string_field.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\tests\ui\typed_contains_requires_string_field.rs)
  - [typed_project_rejects_foreign_schema_field.rs](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\tests\ui\typed_project_rejects_foreign_schema_field.rs)

What is proven:

- validated artifacts cannot be field-constructed externally
- weaker artifacts do not publicly manufacture validated proof
- some legality surfaces are compile-time gated through schema-derived typed
  tokens rather than only runtime checks
- typed query APIs lower into the same canonical and validated authority path
  rather than introducing alternate semantics

## Additional Hardening Added Before Close

Milestone 2 closeout includes these extra hardening outcomes beyond the
minimum roadmap labels:

- schema-derived typed query APIs were introduced so capability misuse can fail
  at compile time where Rust can honestly know the answer
- the schema DSL became the single source of truth for typed tokens and runtime
  schema views, reducing drift between compile-time and runtime legality
- identifier categories were hardened with newtypes across the authority path
  so aspect/field/delivered-name/relation identity is materially less stringly
- membership and string-contains normalization were moved into stronger
  invariant-bearing wrappers instead of loose `Vec` discipline
- schema lookups were moved off allocation-heavy string-key rebuilding and into
  typed partitioned lookup structures
- validated artifacts stopped duplicating canonical authority, removing both
  structural waste and semantic drift risk
- the Milestone 2 harness, certification surface, validation matrix, Milestone
  1 adapter, predicate case suite, canonical artifacts, and predicate-state
  modules were all decomposed further before closeout so Milestone 3 does not
  inherit filing-cabinet debt

These changes were made because the closeout bar was not "validation works on
happy paths." The closeout bar was production-grade legality proof with honest
cost surfaces, no silent widening, and no hidden alternate authority.

## Explicit Deferrals

Milestone 2 intentionally does not include:

- planning
- snapshot-backed execution
- live promotion
- diff, historical, lineage, or correspondence query semantics
- policy masking or tenant-schema semantics beyond later extensibility
- saved-query persistence
- durable portability
- full structured-content operator richness
- full schema-generated legality for every future query surface

Those remain later roadmap work and were not faked early here.

## Verification Baseline

At closeout, the operational verification baseline is:

- `cargo fmt --package worth-query --manifest-path C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\crates\worth-query\Cargo.toml`
- isolated standalone `worth-query` verification using a temporary one-crate
  workspace and:
  - `cargo test --manifest-path <temp>\worth-query\Cargo.toml -q`

This passes cleanly and includes:

- 81 unit and harness tests
- 1 trybuild compile-fail suite
- named Milestone 2 validation certification artifact checks
- hostile validation cases across projection, traversal, ordering, predicates,
  workflow, and structured content
- compile-fail proof-boundary tests for validated and typed surfaces

Closeout note:

- root-workspace Cargo verification is currently blocked by an unrelated
  missing-member workspace issue outside `worth-query`
- Milestone 2 verification is therefore recorded against the isolated
  `worth-query` workspace harness rather than the root workspace

## Operational Conclusion

Milestone 2 is now closed at the query-legality level.

`worth-query` no longer depends on planning, execution, host repair, or loose
runtime convention to decide whether a query is legal. It now owns one
schema-aware validation boundary, exact-basis validated identity, typed early
failure, no-silent-widening enforcement, schema-derived typed query surfaces,
and a named certification artifact that proves the closeout claim
mechanically.
