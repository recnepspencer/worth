# Milestone 9.3.2 Closeout: Query Basis Capability Lifecycle

## Status

Milestone 9.3.2 is closed as of 2026-05-13 for the Query-owned basis
capability lifecycle in `worth-query`, with lower-runtime authority preserved
through existing relational, runtime-bridge, and signal facades.

This closeout covers:

- phase-typed basis progression from `RawBasisIntent` through normalized
  intent, eligibility, admitted capability, scoped use, lower-runtime
  readmission, use receipt, self-describing envelope, and certification
- operation-specific admitted capability lanes for observation, mutation
  preparation, replay, inspection, materialization, subscription declaration,
  subscription activation, preview closeout, and certification
- typed denials for stale, inaccessible, incompatible, operation-ineligible,
  policy-masked, tenant/schema-mismatched, preview-drifted, historical replay
  unsupported, missing lower-runtime binding, durable overclaim, temporal
  deferred, async/resource deferred, and missing signal observation cases
- lower-runtime reuse matrix rows for bridge subscription/truth-view/
  continuity/preview/writeback/causal evidence, relational truth/history/
  snapshot evidence, relational bridge adapters, and signal snapshot/replay/
  lineage evidence
- lifecycle adapters for branch/preview admission, read-composition basis
  context, subscription basis posture, causal inspection evidence, and
  historical materialization basis, with zero in-scope compatibility debt
- public-boundary, proof-shape, phase-manifest, performance-slope, DX, support,
  migration, reuse, and certification bundle audits
- compile-fail boundaries preventing external construction of proof-bearing
  lifecycle, adapter, receipt, envelope, lower-runtime, support, audit, and
  certification artifacts

This closeout does not claim durable basis reload, restart-stable basis
envelopes, store-restored snapshot-plus-tail reconstruction, store-backed
historical replay parity, portable basis capability import/export, or persisted
basis use receipt archives. Those remain Milestone 10, Milestone 11, or later
scope exactly as the 9.3.2 spec declared.

## Governing Source Summary

- `MENTALITY.md`: closure required the adversarial path first: raw branch,
  preview, tenant, policy, snapshot, historical, runtime, or lower-runtime
  identifiers cannot act as permission tokens.
- `arch_laws.md`: closure required proof-bearing typestate progression,
  compiler-enforced construction, typed denial, and authority-preserving
  lower-runtime boundaries.
- `composition_laws.md`: closure required basis normalization, eligibility,
  scoped use, lower-runtime readmission, receipt construction, support, and
  certification to stay separately named responsibilities.
- `domain_structure_laws.md`: closure required Query capability state,
  lower-runtime authority evidence, migration/adapters, support metadata, and
  certification artifacts to remain physically locatable.
- `perf_laws.md`: closure required exact counters and slope digests for every
  claimed bounded basis operation.
- `milestone-9.3.2.md`: the shipped surface now satisfies the phase contract,
  acceptance evidence, required verification outputs, and closeout standard.

## Adversarial Constraint Closed

Milestone 9.3.2 had to survive a downstream consumer attempting to observe,
mutate, replay, inspect, materialize, subscribe, close out preview work, or
certify against a basis by passing a raw identifier or stale lower-runtime
token.

The closed surface enforces this through one typed progression:

1. `RawBasisIntent`
2. `NormalizedBasisIntent`
3. `BasisEligibility` or `DeniedBasisCapability`
4. `AdmittedBasisCapability`
5. `ScopedExecutionOrObservationBasis`
6. `LowerRuntimeBoundBasis`
7. `BasisUseReceipt`
8. `SelfDescribingBasisEnvelope`
9. `BasisLifecycleCertificationBundle`

Query owns public basis normalization, admission, capability shaping, receipts,
envelopes, support metadata, and certification. It does not mint relational
truth, runtime-bridge authority basis records, signal scheduling/evaluation
authority, or store-backed durable recovery.

## Shipped Scope

Milestone 9.3.2 delivered:

- basis lifecycle implementation in
  [crates/worth-query/src/basis_lifecycle](../../crates/worth-query/src/basis_lifecycle)
- certification outputs and audits in
  [crates/worth-query/src/basis_lifecycle/certification](../../crates/worth-query/src/basis_lifecycle/certification)
- lower-runtime reuse matrix and adapter shape contract in
  [crates/worth-query/src/basis_lifecycle/reuse.rs](../../crates/worth-query/src/basis_lifecycle/reuse.rs)
- migration audit and zero-debt lifecycle adapters in
  [crates/worth-query/src/basis_lifecycle/migration.rs](../../crates/worth-query/src/basis_lifecycle/migration.rs) and
  [crates/worth-query/src/basis_lifecycle/adapters.rs](../../crates/worth-query/src/basis_lifecycle/adapters.rs)
- DX transcripts and common-path basis API in
  [crates/worth-query/src/basis_lifecycle/dx.rs](../../crates/worth-query/src/basis_lifecycle/dx.rs)
- support metadata and discovery in
  [crates/worth-query/src/basis_lifecycle/support.rs](../../crates/worth-query/src/basis_lifecycle/support.rs)
- facade exports in
  [crates/worth-query/src/facade/exports_foundation.rs](../../crates/worth-query/src/facade/exports_foundation.rs)
- compile-fail proof boundaries in
  [crates/worth-query/tests/ui/basis_lifecycle](../../crates/worth-query/tests/ui/basis_lifecycle)

## Acceptance Mapping

Milestone 9.3.2 is considered closed against:

- [milestone-9.3.2.md](./milestone-9.3.2.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)
- [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
- [milestone-9.3.1-closeout.md](./milestone-9.3.1-closeout.md)

because Query basis meaning is now represented by a single proof-bearing
capability lifecycle with machine-checkable certification artifacts and
authority-preserving lower-runtime reuse.

### `Query Basis Capability Lifecycle Test`

Covered by:

- [crates/worth-query/src/basis_lifecycle/tests.rs](../../crates/worth-query/src/basis_lifecycle/tests.rs)
- [crates/worth-query/src/basis_lifecycle/certification/tests.rs](../../crates/worth-query/src/basis_lifecycle/certification/tests.rs)
- [crates/worth-query/src/basis_lifecycle/support/tests.rs](../../crates/worth-query/src/basis_lifecycle/support/tests.rs)
- [crates/worth-query/src/basis_lifecycle/reuse/tests.rs](../../crates/worth-query/src/basis_lifecycle/reuse/tests.rs)
- [crates/worth-query/src/basis_lifecycle/adapters/tests.rs](../../crates/worth-query/src/basis_lifecycle/adapters/tests.rs)
- [crates/worth-query/tests/ui/basis_lifecycle](../../crates/worth-query/tests/ui/basis_lifecycle)

What is proven:

- equivalent current-head, branch-head, snapshot, preview, tenant, and policy
  basis intents normalize to stable capability/envelope identity
- intentionally different basis meaning changes digest fields
- denied, advisory, deferred, unsupported, lower-runtime mismatch, and stale
  evidence lanes fail typed and early before operational artifacts exist
- operation lanes are witness types rather than boolean permission flags
- bridge, relational, and signal evidence is readmitted through facade-owned
  authority names and digests rather than reminted as Query facts
- support metadata agrees with executable admission behavior
- every required certification output is present and bound to a concrete audit,
  representative row, support matrix, reuse row, proof-shape audit, phase
  manifest, failure digest, counter snapshot, or performance slope digest
- public callers cannot construct admitted capabilities, scoped bases,
  receipts, envelopes, lower-runtime authority witnesses, support rows,
  migration/adapters, public-boundary rows, proof-shape rows, phase-manifest
  rows, slope rows, or certification bundles directly

## Final Audit Finding Closed

The final closeout audit found one certification-strength gap: three required
verification outputs were still too soft.

- `signal_basis_authority_digest` was a deferred label instead of a digest
  bound to the signal reuse row.
- `typestate_transition_digest` reused the generic certification row summary.
- `phase_artifact_manifest_digest` reused the generic certification row
  summary.

That gap is closed by:

- binding `signal_basis_authority_digest` to the
  `SignalSnapshotReplayLineageBasis` row in the lower-runtime reuse matrix
- adding `BasisLifecyclePhaseManifest`, a sealed phase-manifest audit covering
  every lifecycle artifact from raw intent through certification bundle
- deriving `typestate_transition_digest` from the ordered phase transitions
  rather than the certification row set
- deriving `phase_artifact_manifest_digest` from phase-manifest row digests
- adding tests that reject generic row-summary substitution
- adding compile-fail coverage proving external callers cannot construct phase
  manifest rows

## Closeout Standard

The Milestone 9.3.2 closeout standard is satisfied because:

- the spec phases were implemented in order across the lifecycle, lower-runtime
  reuse, receipts/envelopes, support, migration, and certification slices
- every ordinary Query basis-consumer surface covered by 9.3.2 uses scoped
  capability proof or a zero-debt lifecycle adapter
- every admitted basis family has canonicalization, eligibility, use receipt,
  envelope, support metadata, and certification coverage
- every denied/deferred neighbor has typed denial and zero operational residue
  coverage
- compile-fail boundaries prove public callers cannot construct proof-bearing
  lifecycle artifacts
- performance counters and slope digests are enforced for every claimed bounded
  basis operation
- roadmap and test-requirement references point at the 9.3.2 spec and named
  certification suite
- store-backed and durable claims remain explicit later-milestone scope

## Verification Baseline

The closeout state is verified by:

- `cargo fmt --check -p worth-query`
- `cargo test -p worth-query phase_manifest --quiet`
- `cargo test -p worth-query reuse --quiet`
- `cargo test -p worth-query certification --quiet`
- `cargo test -p worth-query basis_lifecycle --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`
- `cargo test -p worth-query --quiet`
- `cargo test -p worth-runtime-bridge --quiet`
- basis lifecycle line-cap scan
- stale-marker scans for `PendingPhase6`, `reused_or_`, generic output
  substitution, and deferred signal authority labels

## Deferred Scope That Remains Explicitly Deferred

The following are not part of Milestone 9.3.2 closeout:

- durable basis reload
- restart-stable basis envelopes
- store-restored snapshot-plus-tail reconstruction
- store-backed historical replay parity
- portable basis capability import/export
- persisted basis use receipt archives

These remain Milestone 10, Milestone 11, or later lifecycle/certification
scope exactly as the milestone spec declared.
