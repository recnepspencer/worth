# Milestone 2 Closeout: Operating Modes And Lifecycle Contracts

> **Note:** The legacy root crate `crates/worth-store` was removed on 2026-09-27. Paths into it below name its files as they were; they are no longer links. The store now lives in [`workspaces/worth-store`](../../workspaces/worth-store/README.md).

## Status

Milestone 2 is closed as of 2026-04-13.

`worth-store` now has explicit operating-mode authority boundaries instead of
ambient deployment conventions.

The semantic center shipped in this milestone is:

durable mode, embedded mode, and absent mode are now distinct proof-bearing
lifecycle shapes; durable and embedded commits converge through the same
canonical append boundary; embedded checkpoints are explicitly non-authoritative;
and absent mode remains a real no-store lane rather than a disabled store
configuration.

This is not "we added a couple builders." The store now owns:

- explicit durable and embedded mode builders and handles
- an explicit absent-mode witness outside the store facade
- mode construction proofs for hosted runtime ownership and external artifact
  intake capability
- mode-specific proof stages for hosted-runtime commit extraction, external
  runtime origin verification, and embedded checkpoint classification
- exact mode counters and machine-checkable Milestone 2 certification bundles
- typed misuse failures plus compile-time phase-boundary rejection for
  non-admitted cross-mode API use

## Shipped Scope

Milestone 2 delivered:

- mode surfaces under
  `crates/worth-store/src/modes`
- a lifecycle proof subdomain under
  `crates/worth-store/src/modes/lifecycle`
- durable-mode hosted-runtime execution and commit extraction in
  `crates/worth-store/src/modes/durable.rs`
- embedded-mode external commit intake, checkpoint classification, and
  checkpoint persistence in
  `crates/worth-store/src/modes/embedded.rs`
- absent-mode semantic witness and certification lane support in
  `crates/worth-store/src/modes/absent.rs`
- Milestone 2 certification evidence in
  `crates/worth-store/src/evidence/milestone_2.rs`
- mode counter extensions in
  `crates/worth-store/src/evidence/counters.rs`
- typed mode failures in
  `crates/worth-store/src/failure/mod.rs`
- runtime-mode scenario coverage in
  `crates/worth-store/src/tests/operating_modes.rs`
  and
  `crates/worth-store/src/tests/mode_certification.rs`
- compile-fail phase-boundary proof in
  `crates/worth-store/tests/phase_boundaries_compile_fail.rs`
  and
  `crates/worth-store/tests/ui`

## Acceptance Mapping

Milestone 2 is considered closed against the roadmap and
[test-requirements.md](./test-requirements.md)
because the required acceptance surfaces are now covered directly.

### `Operating mode contract parity test`

Covered by:

- `tests::operating_modes::durable_and_embedded_modes_persist_equivalent_canonical_artifacts`
- `tests::operating_modes::derived_embedded_checkpoint_persists_without_changing_authority`
- `tests::operating_modes::authoritative_checkpoint_classification_is_rejected`
- `tests::operating_modes::external_commit_requires_non_empty_runtime_identity`
- `tests::operating_modes::embedded_checkpoint_requires_non_empty_identity_fields`
- `tests::operating_modes::absent_runtime_witness_produces_semantic_evidence_without_store`
- `tests::mode_certification::milestone_2_certification_bundle_proves_mode_contract_parity`
- `tests::mode_certification::milestone_2_certification_bundle_captures_typed_mode_failures`

What is proven:

- durable and embedded lanes persist equivalent canonical authoritative
  artifacts for semantically equivalent histories
- durable and embedded lanes converge at the same Milestone 1 append boundary
  instead of taking parallel persistence meanings
- embedded checkpoint persistence does not change authoritative artifact digest
  or branch-head meaning
- authoritative checkpoint classification is rejected explicitly and typed
- external runtime commit and checkpoint intake require explicit non-empty
  runtime identity / checkpoint identity
- absent mode produces semantic evidence without a store facade, store handle,
  or ambient persistence setup
- forbidden cross-mode work remains zero in the representative certification
  lanes where the milestone claims zero work

### `Machine-checkable certification bundle`

Covered by:

- `tests::mode_certification::milestone_2_certification_bundle_proves_mode_contract_parity`
- `tests::mode_certification::milestone_2_certification_bundle_captures_typed_mode_failures`

What is proven:

- Milestone 2 emits `artifact_digest`, `diagnostics_digest`,
  `mode_contract_matrix`, `failure_digest`, `checkpoint_authority_report`, and
  `counter_snapshot`
- the bundle distinguishes semantic parity from lane-local operational counters
- checkpoint evidence is represented as a first-class report rather than buried
  in ad hoc assertions
- absent-mode evidence remains explicit about being a no-store control lane

### `Compile-time phase-boundary enforcement`

Covered by:

- `tests/phase_boundaries_compile_fail.rs`
- `tests/ui/durable_handle_rejects_embedded_checkpoint_intake.rs`
- `tests/ui/embedded_handle_rejects_durable_mutation_execution.rs`
- `tests/ui/store_facade_does_not_construct_absent_mode.rs`

What is proven:

- `DurableStoreHandle` does not expose embedded checkpoint intake APIs
- `EmbeddedStoreHandle` does not expose durable hosted-mutation APIs
- `WorthStoreBuilder` does not expose an absent-mode constructor
- key cross-mode misuse classes are impossible by public API shape rather than
  relying only on runtime rejection

## Additional Hardening Added Before Close

Milestone 2 closeout includes these extra hardening outcomes beyond the minimum
roadmap labels:

- the mode layer was refactored toward Law 41 proof stages instead of leaving
  mode-specific checks buried in handle methods
- durable construction now carries an explicit hosted-runtime ownership proof
  instead of a raw runtime field by convention
- embedded construction now carries an explicit external-artifact intake
  capability proof instead of builder naming alone
- raw mutable escape hatches that weakened mode-handle authority boundaries
  were removed
- the lifecycle layer was split into future-proof subdomains rather than one
  mixed `lifecycle.rs` cabinet
- compile-fail tests were added so the strongest non-admitted mode crossings
  are proven by type surface, not just by runtime tests

These changes were made because the bar for `worth-store` is platform-grade
architectural honesty, not feature-plausible mode naming.

## Explicit Deferrals

Milestone 2 intentionally does not include:

- WAL append and crash-safe acknowledgment
- recovery replay or crash-boundary exactness
- snapshot capture or restore
- embedded checkpoint restore semantics beyond intake/persistence contract
- live-query continuation
- replication capsules
- budget admission control
- any claim that durable mode acknowledgments survive crash before Milestone 3

Those remain later roadmap milestones and were not implied early here.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-store`

This passes cleanly and includes:

- 32 runtime tests
- 1 compile-fail harness
- 3 compile-fail UI boundary cases
- durable vs embedded parity lanes
- absent-mode no-store proof
- checkpoint non-authority proof
- typed misuse and external-origin rejection lanes
- machine-checkable Milestone 2 certification bundles

## Operational Conclusion

Milestone 2 is now closed at the store level.

`worth-store` no longer treats operating mode as an ambient convention or a
boolean hidden inside one universal store object. It now has explicit mode
construction proofs, explicit mode-specific handles, one shared canonical
append boundary across durable and embedded commits, a real absent-mode control
lane, typed misuse failures, compile-time phase-boundary protection, and a
machine-checkable Milestone 2 certification surface.
