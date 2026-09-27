# Milestone 5.2 Closeout: Preview Session Query Contexts, Preview-Live Composition, And Workflow Foundations

## Status

Milestone 5.2 is closed as of 2026-04-16 for the runtime-backed preview,
preview-live, promotion-parity comparison, and workflow-foundation scope
defined in
[milestone-5.2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.2.md).

`worth-query` now has a real preview-session query substrate layered on top of
the Milestone 3 planning/basis boundary, the Milestone 4 result-family
boundary, and the Milestone 5/5.1 live substrate. Preview basis binding,
lifecycle-explicit execution, promotion-parity comparison, preview-live
admission, drift denial, explicit rebind, workflow-foundation admission, exact
counter surfaces, and milestone-native certification are no longer host glue,
branch aliasing, or bridge-diagnostic folklore. They are crate-owned,
digest-bearing, compile-time hardened, and certification-proven surfaces.

The semantic center shipped in this milestone is:

the same admitted canonical query can bind to an explicit preview session,
preserve preview basis and lifecycle identity through execution, compare
preview results to promoted or authoritative truth through typed query-native
artifacts, and compose with the existing live substrate through one explicit
preview-live proof chain that either remains maintained, denies on lifecycle
drift, or emits one explicit rebind artifact instead of silently retargeting to
authoritative live truth.

## Shipped Scope

Milestone 5.2 delivered:

- preview basis binding, preview execution, promotion-parity comparison,
  preview-live admission/execution/drift, and workflow-foundation artifacts in
  [crates/worth-query/src/preview/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/preview/mod.rs)
- the public preview facade surface in
  [crates/worth-query/src/facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/facade.rs)
- milestone-native preview certification artifacts, completeness reports, row
  catalogs, and matrix assertions under
  [crates/worth-query/src/harness/preview_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/preview_certification)
- shared milestone requirement wiring in
  [crates/worth-query/src/harness/certification/requirements.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/certification/requirements.rs)
  and
  [crates/worth-query/src/harness/certification/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/certification/tests.rs)
- compile-fail proof-boundary tests for preview proof types, preview-live
  staging, comparison admission, workflow admission, and forbidden construction
  paths under
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 5.2 is considered closed against
[milestone-5.2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.2.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required preview-session and preview-live acceptance surface now
exists directly.

### `Preview Session Basis And Promotion Parity Test`

Covered by:

- [preview_certification/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/preview_certification/mod.rs)
- [preview_certification/row_catalog.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/preview_certification/row_catalog.rs)
- [preview_certification/model.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/preview_certification/model.rs)

What is proven:

- the named preview certification artifact exists as a first-class Milestone
  5.2 closeout surface
- required canonical rows are present:
  - `preview-basis-execution-parity`
  - `preview-lifecycle-explicitness`
  - `preview-promotion-comparison-parity`
  - `preview-lifecycle-no-rediscovery`
  - `preview-live-admission-parity`
  - `preview-live-drift-explicitness`
  - `preview-comparison-shape-proof-width`
  - `preview-workflow-foundation-admission`
  - `preview-workflow-foundation-no-rescan`
  - `preview-work-avoided-counter-parity`
- required rejection rows are present:
  - `unsupported-preview-family`
  - `invalid-preview-basis`
  - `stale-preview-lifecycle-denied`
  - `discarded-preview-execution-denied`
  - `preview-live-drift-denied`
  - `preview-live-broad-fallback-forbidden`
  - `preview-broad-fallback-forbidden`
  - `unsupported-preview-promotion-comparison`
  - `read-only-preview-denies-promotion-comparison`
  - `raw-branch-alias-preview-forbidden`
  - `preview-promotion-linkage-denied`
  - `preview-replay-linkage-denied`
  - `preview-shape-mismatch-denied`
  - `promotion-eligibility-bool-forbidden`
  - `preview-diagnostics-rescan-forbidden`
  - `fabricated-preview-lifecycle-forbidden`
  - `out-of-scope-workflow-foundation-request`
- the certification artifact is honest about the runtime-backed preview-live
  scope it admits and carries separate binding, execution, comparison, and
  preview-live counter snapshots
- the certification artifact is deterministic and offline-readable

### `Preview basis as a plan-derived proof boundary`

Covered by:

- `preview::bind_preflight_to_preview_session`
- `preview::PreviewSessionBindingTuple`
- `preview::PreviewSessionPlanBinding`
- preview binding tests in
  [crates/worth-query/src/preview/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/preview/mod.rs)

What is proven:

- preview binding consumes admitted preflight artifacts instead of inventing a
  second query language
- preview session identity, declaration identity, lifecycle state,
  execution-record identity, and evaluation class are explicit basis artifacts
  rather than host-side branch aliases
- non-active lifecycle, store-backed preflights, invalid execution-record
  linkage, raw branch aliases, and unsupported preview families fail typed and
  early
- preview proof-bearing types remain externally unconstructable through private
  fields and compile-fail coverage

### `Promotion-parity comparison as a typed query surface`

Covered by:

- `preview::admit_authoritative_preview_comparison_candidate`
- `preview::admit_preview_promotion_parity_comparison`
- `preview::PromotionParityPreviewComparisonAdmission`

What is proven:

- preview-versus-promoted comparison lowers from one canonical query shape plus
  explicit preview and authoritative basis evidence rather than host-side diff
  folklore
- read-only preview execution cannot reach the promotion-parity comparison API
  at compile time
- comparison eligibility proves query digest, result-family, ordering basis,
  and materialization-boundary compatibility before comparison admission
- shape mismatch, promotion-linkage mismatch, and unsupported comparison
  families deny typed and early

### `Preview-live composition, denial, and explicit rebind`

Covered by:

- `preview::admit_preview_live_session_plan`
- `preview::execute_preview_live_session_plan`
- `preview::assess_preview_live_drift`
- `preview::PreviewLiveExecutionEnvelope`
- `preview::PreviewLiveDriftOutcome`
- `preview::PreviewLiveMaintained`
- `preview::PreviewLiveRebindArtifact`

What is proven:

- preview-live reuses the Milestone 5/5.1 live substrate instead of inventing
  a second live model
- preview-live admission requires matching query digest, plan digest,
  collection digest, and authoritative start basis across preview and live
  proofs
- steady-state preview-live maintenance preserves explicit preview basis and
  records one explicit lifecycle check rather than rediscovering lifecycle or
  retargeting basis ambiently
- lifecycle drift has one closed outcome family:
  - `Maintained`
  - `DriftDenied`
  - `ExplicitRebindAvailable`
- invalid rebind basis and broad-fallback pressure are distinct typed drift
  denials rather than one collapsed failure bucket
- explicit rebind is basis-explicit and counter-bearing rather than a disguised
  fallback to ordinary live truth

### `Workflow foundations stop before mutation authority`

Covered by:

- `preview::admit_preview_workflow_foundation_request`
- `preview::admit_preview_workflow_foundation`
- `preview::AdmittedPreviewWorkflowFoundation`
- `preview::PreviewWorkflowFoundationArtifact`

What is proven:

- workflow foundations carry preview/compare basis structure only
- workflow admission is explicit and counter-bearing
- out-of-scope workflow requests deny typed and early
- workflow artifacts remain separate from mutation, merge, and writeback
  lowering, which stay later-milestone work

## Explicit Deferred Scope

Milestone 5.2 is closed for runtime-backed preview-session, preview-live,
promotion-parity comparison, and workflow-foundation semantics only.

The following remain explicit later-milestone work, not implied completeness:

- durable preview replay reload, persisted workflow artifacts, and restart-
  stable continuation
- store-backed preview execution parity
- broader branch/head/historical/diff basis expansion beyond the shipped
  preview-session and promotion-parity surface
- mutation intent lowering, merge execution, conflict classification, and
  writeback lowering
- broader policy-masked or tenant-variant preview combinations than the
  admitted runtime-backed slice

The current Milestone 5.2 surface is intentionally narrow in admitted family
count and broad in semantic honesty.

## What Later Milestones May Now Assume

Milestones 5.3, 5.4, 5.5, and later may safely assume:

- preview session basis is a first-class query proof boundary
- preview lifecycle identity is explicit on bindings, execution artifacts, and
  certification lanes
- promotion-parity comparison is query-native and basis-explicit
- preview-live composition reuses the existing live substrate through an
  explicit proof chain rather than ambient live retargeting
- drift denial and explicit rebind are already typed and counter-proven
- workflow foundations exist as authority-preserving preview/compare artifacts
  instead of host-local bags

Later milestones must not assume:

- durable restart-stable preview continuation
- store-backed preview parity
- preview workflow foundations as permission to imply merge or writeback
  authority

## Verification Baseline

Milestone 5.2 closeout was verified with:

- `cargo test -p worth-query preview --no-fail-fast`
- `cargo test -p worth-query preview_certification --no-fail-fast`
- `cargo test -p worth-query --test phase_boundaries_compile_fail -- --nocapture`
- `cargo test -p worth-query --no-fail-fast`

This passes cleanly and includes:

- unit and harness coverage for preview binding, execution, comparison,
  preview-live admission, preview-live drift, explicit rebind, and workflow
  foundations
- milestone-native preview certification artifact tests
- shared certification-core requirement tests
- trybuild compile-fail tests for preview proof-boundary privacy and forbidden
  raw construction paths

## Operational Conclusion

Milestone 5.2 is now closed at the runtime-backed preview-session and
preview-live level.

`worth-query` no longer depends on branch aliases, ambient preview mode,
host-side preview-versus-promoted diffing, silent preview-live fallback to
authoritative truth, or workflow-shaped bags of bridge metadata to make
preview reads and preview-live composition work. It now has a plan-derived
preview basis boundary, lifecycle-explicit preview execution, typed
promotion-parity comparison, preview-live maintenance with denial and explicit
rebind semantics, workflow-foundation admission, milestone-native
certification, and named Milestone 5.2 acceptance evidence that later
branch-history and workflow milestones can build on safely.
