# Milestone 5.5 Closeout: Query Workflow Lowering And Writeback Boundaries

## Status

Milestone 5.5 is closed as of 2026-04-19 for the admitted workflow scope in
`worth-query`.

`worth-query` now owns workflow declaration, workflow basis binding, workflow
lowering, conflict inspection, post-merge inspection, authority-outcome
shaping, replay bundling, and milestone-native certification as proof-bearing
query artifacts rather than host glue, raw lower-crate passthrough, or
controller-local orchestration.

The semantic center shipped in this milestone is:

the same canonical query meaning can now bind workflow context, author
mutation/merge/writeback intent, lower that intent into relational and bridge
authorities, inspect merge conflict truth, inspect authoritative post-merge
outcomes, and emit replay-safe workflow bundles without letting `worth-query`
become a second mutation engine or hiding authority transfer behind cheap
application helpers.

## Shipped Scope

Milestone 5.5 delivered:

- query-owned workflow declaration, basis-binding, and admission surfaces in
  [crates/worth-query/src/workflow/foundation.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/foundation.rs)
- query-owned workflow lowering for relational mutation, relational merge, and
  bridge writeback in
  [crates/worth-query/src/workflow/lowering.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/lowering.rs)
- query-shaped conflict inspection, post-merge inspection, authority-outcome
  shaping, and replay-safe bundling in
  [crates/worth-query/src/workflow/inspection.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/inspection.rs)
  and
  [crates/worth-query/src/workflow/inspection_projection.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/inspection_projection.rs)
- dedicated workflow performance and counter surfaces in
  [crates/worth-query/src/workflow/performance.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/performance.rs)
- relational authority-owned merge inspection proof minting in
  [crates/worth-relational/src/merge/data/artifacts.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/merge/data/artifacts.rs)
  and
  [crates/worth-relational/src/merge/logic/mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/merge/logic/mod.rs)
- milestone-native workflow certification in
  [crates/worth-query/src/harness/workflow_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification)
- compile-fail proof boundaries for workflow declarations, lowered artifacts,
  inspection artifacts, authority outcomes, replay bundles, and authority
  override in
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 5.5 is considered closed against
[milestone-5.5.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.5.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required workflow-lowering and workflow-inspection proof surfaces
now exist directly.

### `Query Workflow Lowering And Writeback Boundary Test`

Covered by:

- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification/mod.rs)
- [lane.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification/lane.rs)
- [matrix.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification/matrix.rs)
- [row_catalog.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification/row_catalog.rs)
- [tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification/tests.rs)

What is proven:

- the named certification artifact exists as a first-class Milestone 5.5
  closeout surface
- required canonical rows are present and exercised:
  - `workflow-declaration-family-explicitness`
  - `workflow-basis-family-explicitness`
  - `workflow-authority-target-explicitness`
  - `workflow-preview-foundation-no-rediscovery`
  - `workflow-budget-class-explicitness`
  - `query-authored-mutation-lowering-parity`
  - `query-authored-merge-lowering-parity`
  - `query-triggered-writeback-lowering-parity`
  - `conflict-inspection-explicitness`
  - `unsupported-deletion-topology-merge-class`
  - `post-merge-inspection-explicitness`
  - `workflow-freshness-explicitness`
  - `workflow-prediction-width-explicitness`
  - `workflow-realized-width-explicitness`
  - `workflow-rediscovery-zero-parity`
- required rejection rows are present and exercised:
  - `unsupported-workflow-family`
  - `invalid-basis-pairing`
  - `preview-read-only-authority-request-forbidden`
  - `unsupported-authority-target`
  - `forbidden-workflow-broadening`
  - `unsupported-merge-family`
  - `unsupported-writeback-family`
  - `stale-workflow-denied`
  - `explicit-rebind-required`
  - `post-merge-non-authoritative-outcome-forbidden`
  - `query-workflow-declaration-constructor-private`
  - `workflow-context-binding-constructor-private`
  - `workflow-authority-target-override-forbidden`
  - `workflow-admission-bool-shortcut-forbidden`
  - `raw-preflight-is-not-workflow-declaration`
- the certification suite now asserts exact row coverage, compile-fail binding,
  required verification outputs, adversarial digest distinctness, and
  non-trivial counter distinctness rather than row presence alone

### `Authority-boundary honesty and query-shaped workflow inspection`

Covered by:

- [crates/worth-query/src/workflow/inspection.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/inspection.rs)
- [crates/worth-query/src/workflow/inspection_projection.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/inspection_projection.rs)
- [crates/worth-query/src/workflow/tests/inspection.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/tests/inspection.rs)
- [crates/worth-relational/src/merge/data/artifacts.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/merge/data/artifacts.rs)
- [crates/worth-relational/src/tests/history/milestone_7d_phase_e.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/tests/history/milestone_7d_phase_e.rs)

What is proven:

- conflict inspection now requires an admitted query-owned conflict inspection
  declaration rather than bypassing the declaration boundary through merge
  lowering alone
- conflict inspection consumes a proof-bearing relational inspection artifact,
  not caller-supplied merge evidence
- deletion-denied and topology-region-conflict lanes remain distinct query
  shapes with distinct digests and denied-admission semantics
- post-merge inspection requires an admitted post-merge declaration plus a
  merge/writeback authority outcome and rejects mutation-only outcomes

### `Freshness policy, stale denial, and explicit rebind`

Covered by:

- [crates/worth-query/src/workflow/lowering.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/lowering.rs)
- [crates/worth-query/src/workflow/tests/lowering.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/workflow/tests/lowering.rs)
- [crates/worth-query/src/harness/workflow_certification/matrix.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/workflow_certification/matrix.rs)

What is proven:

- workflow freshness policy is now lowering-significant instead of advisory-only
- preview-origin writeback and merge lowering distinguish:
  - `ExactBasis` -> typed stale denial
  - `AllowExplicitRebind` -> typed explicit rebind requirement
- unsupported merge and unsupported writeback lowering paths now surface
  lowering-specific denial classes rather than collapsing into one generic
  family mismatch
- workflow counters are now lane-explicit as well as phase-explicit:
  mutation lowering, merge lowering, writeback declaration, writeback
  causality binding, conflict inspection, and post-merge inspection each have
  dedicated counters alongside aggregate phase totals

## Final QA Outcome

The last hostile QA loop was run directly against the 5.5 spec, the parity
note, the production workflow modules, the relational authority seam, the
workflow certification matrix, and the unit-test assertions.

The last meaningful findings were:

- conflict inspection was still bypassing its own declaration family
- declaration freshness policy was not affecting lowering behavior
- merge/writeback family-specific denials were still too generic in lowering
- some required compile-fail and stale-denial proof points were not yet part
  of the required certification surface
- workflow accounting was still too aggregate to match the intended per-lane
  counter posture, and replay-bundle counting still relied on in-place counter
  mutation
- query-side inspection still reached into public `MergeExecutionRequest`
  fields directly instead of using a getter surface

Those findings were corrected before this closeout. After the correction pass,
I do not see a remaining meaningful 5.5 spec/code mismatch for the admitted
workflow surface.

## Explicit Deferred Scope

Milestone 5.5 is closed for the admitted workflow-declaration and
workflow-lowering scope only.

The following remain later-milestone work, not implied completeness:

- unified daily-driver application-facade closure, which remains Milestone 5.6
  work
- broader branch/historical/diff basis contexts, which remain Milestone 6 work
- richer workflow-family breadth beyond the currently admitted narrow families
- durable workflow continuation, persisted workflow artifacts, and restart-
  stable workflow resume
- broader bridge and relational family coverage beyond the currently admitted
  workflow matrix

The current 5.5 surface is intentionally narrow in admitted family count and
strict in authority-boundary honesty.

## What Later Milestones May Now Assume

Milestones 5.6 and later may safely assume:

- query-owned workflow declarations already exist as sealed proof-bearing
  artifacts
- conflict inspection and post-merge inspection are already query-shaped,
  basis-explicit, and authority-boundary-honest
- relational merge inspection truth is already sealed behind an
  authority-owned proof-bearing input path
- freshness policy already distinguishes stale denial from explicit rebind on
  preview-origin lowering paths
- workflow certification already proves parity, denial, compile-fail, and
  zero-rediscovery behavior for the admitted families

Those milestones must not assume:

- unsupported workflow families are implicitly beta-supported
- host-local workflow glue is an allowed escape hatch
- ambient basis fallback is available
- authority target override is available after declaration lowering

## Verification Baseline

Milestone 5.5 closeout was verified with:

- `cargo test -p worth-query workflow --quiet`
- `cargo test -p worth-query workflow_certification --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`
- `cargo test -p worth-query --quiet`
- `cargo test -p worth-relational --quiet`

This passes cleanly and includes:

- workflow unit tests
- workflow certification and closeout artifact tests
- trybuild compile-fail proof-boundary tests
- relational merge inspection authority tests
- full regression coverage for both `worth-query` and `worth-relational`

## Operational Conclusion

Milestone 5.5 is now closed at the query-workflow declaration, lowering,
inspection, and authority-boundary layer.

`worth-query` no longer depends on host-local merge/writeback glue, caller-
supplied merge truth, ambient post-merge rereads, or advisory-only freshness
policy interpretation to move from query-native workflow intent into
relational and bridge authorities. It now has sealed workflow declarations,
authority-owned merge inspection proof, query-shaped inspection artifacts,
typed stale-denial versus explicit-rebind behavior, replay-safe outcome
bundles, adversarial milestone certification, and a closeout proof surface that
the unified facade and later basis/policy milestones can build on safely.
