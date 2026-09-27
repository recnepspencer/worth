# Milestone 6 Closeout: Branch-Scoped, Historical, And Diff Query Contexts

## Status

Milestone 6 is closed as of 2026-04-19 for the admitted runtime-backed query-
context scope in `worth-query`.

`worth-query` now owns branch basis, historical basis, preview-derived basis,
and typed diff/comparison basis as first-class query-owned artifacts. The same
canonical query shape can now move across current, branch, historical,
preview-derived, and admitted diff contexts without host repair, basis
substitution, raw storage delta leakage, or result-shape drift.

The semantic center shipped in this milestone is:

one canonical query shape can now bind explicit basis contexts, execute through
those admitted contexts, produce basis-explicit result bundles, shape
query-owned diff artifacts across two admitted bases, surface those artifacts
through the unified application facade, and prove the whole boundary through a
named parity suite rather than controller glue or host conventions.

## Shipped Scope

Milestone 6 delivered:

- query-context basis declaration, basis binding, admission, execution,
  metadata shaping, result bundling, and support truth in
  [crates/worth-query/src/query_context](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/query_context)
- unified-facade composition for basis and diff result bundles through
  [crates/worth-query/src/application/capability/witnesses.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/application/capability/witnesses.rs)
  and
  [crates/worth-query/src/application/support/report.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/application/support/report.rs)
- milestone-native certification in
  [crates/worth-query/src/harness/historical_diff_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/historical_diff_certification)
- compile-fail proof boundaries for admitted basis contexts, admitted diff
  contexts, result bundles, metadata artifacts, execution artifacts, and
  query-context support truth in
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 6 is considered closed against
[milestone-6.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-6.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required basis/diff proof surface now exists directly.

### `Historical / Diff / Basis Parity Test`

Covered by:

- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/historical_diff_certification/mod.rs)
- [lane.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/historical_diff_certification/lane.rs)
- [matrix.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/historical_diff_certification/matrix.rs)
- [row_catalog.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/historical_diff_certification/row_catalog.rs)
- [tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/historical_diff_certification/tests.rs)

What is proven:

- the named certification artifact exists as the Milestone 6 closeout suite
- required canonical rows are present and exercised:
  - `current-vs-branch-basis-explicitness`
  - `current-vs-historical-basis-explicitness`
  - `historical-materialization-path-explicitness`
  - `diff-comparison-family-explicitness`
  - `branch-to-branch-diff-shaped`
  - `current-to-historical-diff-shaped`
  - `result-shape-parity-across-basis-variants`
  - `preview-derived-historical-basis-explicitness`
  - `admitted-diff-cost-class-explicitness`
  - `prediction-versus-realization-explicitness`
- required rejection rows are present and exercised:
  - `unsupported-historical-basis`
  - `ambiguous-comparison-basis`
  - `diff-scope-mismatch`
  - `store-backed-historical-deferred-debt`
  - `forbidden-basis-substitution`
  - `raw-storage-delta-leakage-forbidden`
  - `historical-broadening-denied`
  - `broadening-required-comparison-denial`
  - `declared-result-shape-mismatch`
- canonical lanes now carry and assert:
  - `query_digest`
  - `basis_digest`
  - `basis_family`
  - `comparison_basis_family` where relevant
  - `result_shape_digest`
  - `materialization_path_identity` where relevant
  - `preview_provenance_identity` where relevant
  - `result_digest`
  - `replay_digest`
  - `counter_snapshot_digest`
- the suite now proves exact equality, inequality, typed failure, and zero-
  residue requirements rather than row presence alone

### `Query-owned basis and diff artifact honesty`

Covered by:

- [basis.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/query_context/basis.rs)
- [execution.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/query_context/execution.rs)
- [comparison.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/query_context/comparison.rs)
- [metadata.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/query_context/metadata.rs)
- [support.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/query_context/support.rs)

What is proven:

- admitted basis families are sealed and query-owned
- historical execution preserves requested, admitted, resolved, and materialized
  path identity where relevant
- historical reconstruction lanes that would broaden beyond the admitted narrow
  result shape now deny typed and early
- preview-derived execution preserves preview provenance identity rather than
  collapsing into ordinary historical truth
- diff execution requires two admitted bases plus one explicit comparison family
- diff artifacts remain query-shaped and deny broad collection comparison before
  rich artifact shaping
- result bundles are replay-safe proof-bearing artifacts rather than generic
  convenience bags

### `Unified application-facade exposure`

Covered by:

- [witnesses.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/application/capability/witnesses.rs)
- [report.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/application/support/report.rs)
- [matrix.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/unified_facade_certification/matrix.rs)
- [tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/application/tests.rs)

What is proven:

- the normative application-facing surface can execute basis result bundles and
  shape diff result bundles explicitly
- query-context support truth is now domain-owned and consumed by the facade,
  not invented in the facade layer
- broad collection diff remains honestly advertised as deferred scope and also
  denies through the real query-context capability path

### `Compile-time proof-surface hardening`

Covered by:

- [tests/ui/query_basis_context_binding_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/query_basis_context_binding_constructor_private.rs)
- [tests/ui/admitted_query_basis_context_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/admitted_query_basis_context_constructor_private.rs)
- [tests/ui/admitted_diff_query_context_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/admitted_diff_query_context_constructor_private.rs)
- [tests/ui/query_basis_result_bundle_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/query_basis_result_bundle_constructor_private.rs)
- [tests/ui/query_diff_result_bundle_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/query_diff_result_bundle_constructor_private.rs)
- [tests/ui/query_basis_metadata_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/query_basis_metadata_constructor_private.rs)
- [tests/ui/diff_query_metadata_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/diff_query_metadata_constructor_private.rs)
- [tests/ui/query_context_execution_artifact_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/query_context_execution_artifact_constructor_private.rs)
- [tests/ui/query_context_support_profile_constructor_private.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/query_context_support_profile_constructor_private.rs)
- [tests/ui/historical_materialization_metadata_is_not_query_basis_result_bundle.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/historical_materialization_metadata_is_not_query_basis_result_bundle.rs)
- [tests/ui/preview_workflow_foundation_is_not_query_basis_result_bundle.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui/preview_workflow_foundation_is_not_query_basis_result_bundle.rs)

What is proven:

- basis contexts, diff contexts, metadata artifacts, execution artifacts,
  support-profile artifacts, and result bundles cannot be WORTHd externally
- raw historical materialization metadata and raw preview workflow foundations
  cannot stand in for query-owned result-bundle artifacts
- bool-driven basis and comparison shortcuts remain uncompilable
- direct diff shaping bypasses remain uncompilable

## Final QA Outcome

The final hostile QA loop was run directly against the Milestone 6 spec, the
query-context subdomain, the unified-facade composition layer, the support
profile path, the historical/diff certification harness, and the compile-fail
boundary set.

The last meaningful findings were:

- query-context support truth was still facade-owned instead of query-context-
  owned
- a unified-facade broad-diff rejection row was synthetic rather than derived
  from a real denied capability path
- the historical/diff certification lane was still synthesizing replay for some
  single-basis paths and not carrying the full materialization/provenance closeout
  surface
- the remaining proof artifacts and raw lower-runtime payload seams were not
  yet all explicitly sealed by compile-fail tests

Those findings were corrected before this closeout. After the correction pass,
I do not see a remaining meaningful Milestone 6 spec/code mismatch for the
admitted runtime-backed scope.

## Explicit Deferred Scope

Milestone 6 is closed for the admitted runtime-backed basis and diff scope
only.

The following remain later-milestone work, not implied completeness:

- store-backed historical parity
- store-backed diff parity
- richer lineage and correspondence query semantics, which remain Milestone 7
  work
- broader branch-comparison presentation and view semantics, which remain
  Milestone 8 work
- richer diff expression families beyond the current narrow query-result-shaped
  comparison surface

The current surface is intentionally strict and honest about those limits.

## What Later Milestones May Now Assume

Milestones 7 and later may safely assume:

- basis variation is already sealed and query-owned
- historical materialization-path identity is already integrated into ordinary
  query-context execution
- preview-derived basis provenance is already explicit and preserved
- diff artifacts are already query-shaped, replay-safe, and basis-explicit
- the unified application facade already exposes the normative Milestone 6
  daily-driver surface
- the closeout certification suite already proves exact row coverage, exact
  denial coverage, exact replay posture, exact counter posture, explicit
  result-shape identity, and zero rediscovery on admitted lanes

Those milestones must not assume:

- store-backed historical or diff parity is already available
- broad collection diff is admitted
- raw storage delta escape hatches are tolerated
- basis-specific result-shape drift is available as a convenience

## Verification Baseline

Milestone 6 closeout was verified with:

- `cargo fmt --package worth-query`
- `cargo test -p worth-query query_context --quiet`
- `cargo test -p worth-query historical_diff_certification --quiet`
- `cargo test -p worth-query unified_facade_certification --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`
- `cargo test -p worth-query --quiet`

This passes cleanly and includes:

- query-context unit tests
- historical/diff certification and closeout artifact tests
- unified-facade composition verification
- trybuild compile-fail proof-boundary tests
- full regression coverage for `worth-query`

## Operational Conclusion

Milestone 6 is now closed at the branch-scoped, historical, preview-derived,
and diff query-context layer.

`worth-query` no longer depends on ambient basis repair, host-local historical
reconstruction, raw storage delta exposure, or basis-specific result-shape
drift to move one canonical query shape across current, branch, historical,
preview-derived, and admitted diff contexts. It now has sealed basis and diff
artifacts, explicit materialization/provenance-aware result bundles,
query-context-owned support truth, unified-facade exposure, adversarial
milestone certification, and compile-time proof boundaries that later lineage,
view, and store milestones can build on safely.
