# Milestone 13 Closeout: Tiering And Durable Working-Set Intelligence

Status: Completed on 2026-04-21

Parent spec: [milestone-13.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.md)

Roadmap: [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)

## Summary

Milestone 13 is closed.

`worth-store` now has explicit hot/warm/cold placement vocabulary, durable
tier-residency and in-flight transfer state, working-set observation and
classification surfaces, placement-bound read handles, cold recall execution,
recall coalescing, crash-safe prepare/transfer/verify/cutover/retire movement,
SQLite restart parity, and machine-checkable certification that tiering changes
cost posture only.

The closure claim is:

- canonical authoritative truth remains the authority regardless of tier
  residence
- derived placement and working-set adaptation remain advisory and cost-shaping,
  not semantic truth
- cold recall is explicit, witnessed, budgeted, and counted rather than hidden
  behind ambient backend fallback
- restart reconstructs tier placement from canonical residency manifests and
  typed in-flight records rather than tier inventory scans
- Milestone 11 may schedule tier work, but scheduler policy does not redefine
  placement meaning

## What Shipped

- public tiering vocabulary and proof-bearing placement types in
  [crates/worth-store/src/tiering](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tiering)
- backend tiering planning, execution, observation, recall, interleaving, and
  recovery logic in
  [crates/worth-store/src/backend/tiering](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/tiering)
- scalar SQLite tiering persistence for residency, transfer, observation, and
  recall records in
  [crates/worth-store/src/backend/sqlite](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/backend/sqlite)
- typed placement, recall, witness-misuse, manifest, and open-failure taxonomy
  in
  [crates/worth-store/src/failure/mod.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/failure/mod.rs)
- milestone-specific counters, complexity surfaces, artifact reports, and
  certification bundles in
  [crates/worth-store/src/evidence/milestone_13.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/evidence/milestone_13.rs)
- public store surfaces for tier planning, tier execution, cold recall,
  working-set observation, placement-bound read resolution, and Milestone 13
  evidence in
  [crates/worth-store/src/facade.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/facade.rs)
- named certification coverage in
  [crates/worth-store/src/tests/milestone_13_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_13_certification.rs)
- phase-scoped hostile coverage in
  [crates/worth-store/src/tests/tiering_phase1.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase1.rs),
  [crates/worth-store/src/tests/tiering_phase2.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase2.rs),
  [crates/worth-store/src/tests/tiering_phase3.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase3.rs),
  [crates/worth-store/src/tests/tiering_phase4_recall_coalescing.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase4_recall_coalescing.rs),
  and
  [crates/worth-store/src/tests/tiering_phase5_interleaving.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase5_interleaving.rs)

## Acceptance Mapping

Milestone 13 is considered closed against
[milestone-13.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-13.md)
and
[test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
because the required named suite and supporting phase tests now map directly to
code and machine-checkable evidence.

### `Tiering And Working-Set Non-Authority Test`

Covered by:

- [crates/worth-store/src/tests/milestone_13_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_13_certification.rs)
- [crates/worth-store/src/tests/tiering_phase3.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase3.rs)
- [crates/worth-store/src/tests/tiering_phase4_recall_coalescing.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase4_recall_coalescing.rs)
- [crates/worth-store/src/tests/tiering_phase5_interleaving.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase5_interleaving.rs)

What is proven:

- `truth_digest` remains equal across control, moved/adapted, local-file
  reopened, SQLite, SQLite-reopened, recalled, and interleaved lanes
- `artifact_digest` remains placement-independent by excluding residence class
  while still admitting artifact identities and verification labels
- `diagnostics_digest` diverges when placement, recall, counters, or manifests
  differ, so cost-path changes remain visible without changing truth
- `counter_snapshot` and `Milestone13CounterContract` match exact expected
  movement, recall, coalescing, interleaving, and debt counts
- resident reads and cold-recall reads resolve through typed placement-bound
  handles instead of raw locator fallback
- duplicate recall demand coalesces through explicit in-flight recall records
  and counts suppression separately from completion
- move/read and move/continuation interleavings preserve truth while recording
  the lane-local cost signal

### Durable restart and integrity closure

Covered by:

- [crates/worth-store/src/tests/tiering_phase3.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/tiering_phase3.rs)

What is proven:

- authoritative cutover/retire state survives SQLite reopen with the same
  canonical residency manifest
- prepare-only and verified-before-cutover crash points reopen with singular
  resident truth plus preserved in-flight transfer state
- derived movement plus cold recall survives SQLite reopen without changing
  truth digests
- corrupted SQLite enum labels fail as typed placement/tiering open failures
  rather than generic I/O
- verification-label drift and cutover-completed transfer/residency mismatch are
  rejected on open instead of silently inventing placement meaning

### Compile-time and construction boundary enforcement

Covered by:

- [crates/worth-store/tests/phase_boundaries_compile_fail.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/tests/phase_boundaries_compile_fail.rs)
- [crates/worth-store/tests/ui](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/tests/ui)
- focused unit tests in
  [crates/worth-store/src/tiering](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tiering)

What is proven:

- proof-bearing placement witnesses, transfer/cutover shells, recall witnesses,
  and maintenance tier containers cannot be synthesized through public loose
  constructors
- raw locators and synthetic cross-boundary placement witnesses cannot bypass
  the typed read and movement model
- closed vocabularies parse through owned enum helpers rather than duplicated
  string matching at backend boundaries
- list-bearing proof surfaces normalize deterministically before they can be
  observed

## Acceptance Evidence

The Milestone 13 certification bundle emits:

- `truth_digest`
- `artifact_digest`
- `diagnostics_digest`
- `counter_snapshot`

The bundle also carries:

- `Milestone13ArtifactReport`
- `Milestone13CertificationSummary`
- `Milestone13ComplexitySurface`
- `Milestone13CounterContract`

The certification summary explicitly reports:

- truth matches the control lane
- no tier truth parity failures
- no tier restore parity failures
- no tier recall failures
- no residual residency ambiguity
- verified-path and debt-path counts

## Additional Hardening Added Before Close

The closeout pass intentionally strengthened the tests rather than only
documenting the happy path:

- SQLite load now maps tiering enum corruption to typed placement/tiering
  failures
- crash-point coverage includes prepare-only and verified-before-cutover reopen
  lanes
- certification includes local-file reopen evidence in addition to in-memory
  and SQLite lanes
- certification counter assertions exercise movement, recall, continuation,
  foreground read, and duplicate-demand coalescing rather than checking only
  one isolated field
- adversarial open tests cover residency verification-label drift and completed
  cutover records whose residency row no longer matches
- the certification bundle summary flags are asserted directly so the public
  evidence surface cannot go structurally stale while row digests stay green

## Explicit Deferrals

No in-scope Milestone 13 closeout debt remains for the shipped placement,
recall, persistence, and certification contract.

Future work still exists, but it belongs to later roadmap layers rather than
hidden Milestone 13 incompleteness:

- more ambitious adaptive placement heuristics can remain `Debt` as long as
  their effects continue to flow through the explicit Milestone 13 placement
  surfaces
- Milestone 11 owns pacing, fairness, foreground isolation, and debt-escalation
  policy for tier work; it does not own tier semantics
- replication, blob/object storage, and budget systems should inherit this
  placement vocabulary instead of creating alternate tier authority models
- backend-specific optimization of tier media is future physical work, not a
  change to the closed semantic contract

## Verification

The focused closeout verification run used:

- `cargo test -p worth-store milestone_13_certification -- --test-threads=1`
- `cargo test -p worth-store tiering -- --test-threads=1`
- `cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1`

All passed after the final adversarial test additions.

An earlier full-crate verification run also passed before the final closeout
hardening additions. A later all-in-one rerun hit transient Windows/MSVC build
cache and linker-process issues during the broad dirty-worktree validation
loop; after cleaning the crate target and stopping stale cargo/rustc processes,
the focused Milestone 13 suites and the compile-fail boundary suite passed
cleanly.

## Operational Conclusion

Milestone 13 is now closed at the store level.

`worth-store` no longer treats tiering as an implicit cache policy. Placement,
recall, working-set adaptation, restart reconstruction, and certification are
all explicit, typed, counted, and subordinate to canonical truth. Tier movement
can now change cost and residency without changing replay, restore, or branch
meaning, which is the milestone's core architectural law.
