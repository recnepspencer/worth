# Milestone 11 Closeout: Background Maintenance Isolation And Scheduling Contracts

> **Note:** The legacy root crate `crates/worth-store` was removed on 2026-09-27. Paths into it below name its files as they were; they are no longer links. The store now lives in [`workspaces/worth-store`](../../workspaces/worth-store/README.md).

Status: Completed on 2026-04-21

Parent spec: [milestone-11.md](./milestone-11.md)

Roadmap: `worth_store_roadmap.md`

## Summary

Milestone 11 is closed.

`worth-store` now has a typed, proof-bearing maintenance scheduler boundary for
retention-adjacent work. Maintenance is admitted through descriptors, lowered
through scheduler-owned summaries, paced through deterministic
multi-dimensional budget grants, reported through operator-visible status, and
recovered through restart readmission rather than ambient queue replay.

The closure claim is:

- background maintenance no longer runs as anonymous "jobs"
- every admitted maintenance unit belongs to one work class, one locality
  scope, one reservation family, and one equivalence key
- foreground reads, writes, cursor acknowledgments, and continuations surface
  maintenance interference instead of inheriting hidden queue cost
- debt, starvation, escalation, coalescing, supersession, and freshness
  decisions are represented as typed scheduler evidence
- recovered backlog re-enters through the same admission path as fresh work
- Milestone 13 tier work, snapshot refresh, replication preparation, derived
  rebuild, and audit work share the maintenance runtime without taking over
  scheduler policy or truth authority

## What Shipped

- maintenance declaration payloads, classes, descriptor identities, and
  crate-private container constructors in
  `crates/worth-store/src/maintenance/declarations`
- scheduler vocabulary, lane keys, locality scopes, reservation families,
  budget grants, queue summaries, debt summaries, coalescing decisions,
  starvation status, escalation verdicts, and descriptor lowering in
  `crates/worth-store/src/maintenance/scheduler.rs`
- lifecycle-typed maintenance status, foreground interference, foreground wait,
  readmission, reservation, completion, cancellation, and failure surfaces in
  `crates/worth-store/src/maintenance/lifecycle.rs`
- backend admission, planning, execution, foreground-guarding, evidence, and
  restart-readmission logic in
  `crates/worth-store/src/backend/maintenance`
- durable maintenance declaration, execution, queue-summary,
  locality-summary, reservation-summary, budget-summary, and debt-summary
  records in
  `crates/worth-store/src/backend/records/maintenance.rs`
- Milestone 11 counter contracts, maintenance reports, topology reports,
  resource-budget reports, interference matrix rows, debt-escalation reports,
  complexity surfaces, and certification bundles in
  `crates/worth-store/src/evidence/milestone_11.rs`
- focused Milestone 11 runtime coverage in
  `crates/worth-store/src/tests/milestone_11_maintenance.rs`
  and
  `crates/worth-store/src/tests/milestone_11_maintenance`
- compile-time phase-boundary coverage in
  `crates/worth-store/tests/phase_boundaries_compile_fail.rs`
  and
  `crates/worth-store/tests/ui`

## Acceptance Mapping

Milestone 11 is considered closed against
[milestone-11.md](./milestone-11.md)
and
[test-requirements.md](./test-requirements.md)
because the required named suite and supporting hostile lanes now map directly
to code and machine-checkable evidence.

### `Background Maintenance Isolation And Scheduling Test`

Covered by:

- `crates/worth-store/src/tests/milestone_11_maintenance.rs`
- `crates/worth-store/src/tests/milestone_11_maintenance/foreground.rs`
- `crates/worth-store/src/tests/milestone_11_maintenance/plan_transitions.rs`
- `crates/worth-store/src/tests/milestone_11_maintenance/restart_status.rs`
- `crates/worth-store/src/tests/milestone_11_maintenance/resume.rs`
- `crates/worth-store/src/tests/milestone_11_maintenance/rebuild.rs`

What is proven:

- retention, compaction, reclaim, authoritative reclaim, retained-range
  rebuild, derived-family rebuild, snapshot refresh, replication preparation,
  maintenance audit, tier-placement proposal, and tier-move execution enter
  the scheduler through typed maintenance declarations
- duplicate and same-equivalence work coalesces or rejects through explicit
  descriptor identity rather than queue folklore
- superseded and stale recovered work cancels before reservation or expensive
  execution
- one-dimensional budget failure defers without leaking reservations in the
  other dimensions
- starvation and deferred lanes publish typed evidence and operator-visible
  status
- explicit global-scope debt and explicit cross-locality tier debt increment
  only through typed proof lanes
- foreground reads, writes, continuations, and cursor acknowledgments expose
  reservation violations or wait dependencies when escalated maintenance would
  interfere
- cold-start and warm-start recovered backlog shapes produce equivalent
  scheduler summaries and truth-visible digests
- queue summaries survive local-file and SQLite restart, and corrupted queue
  summaries are rejected instead of trusted

### Scheduler container coverage

Milestone 11 owns scheduling containment, not every downstream semantic
meaning. The following work families are now concrete scheduler containers:

- `RetentionAudit`
- `CompactionMaintenance`
- `DerivedArtifactReclaim`
- `AuthoritativeReclaim`
- `RetainedRangeRebuild`
- `DerivedFamilyRebuild`
- `SnapshotRefresh`
- `ReplicationPreparation`
- `MaintenanceAudit`
- `TierPlacementProposal`
- `TierMoveExecution`

For first ship, late-family execution means "scheduler-admitted container
executed." Snapshot refresh, replication preparation, derived-family rebuild,
audit, and tier containers deliberately do not absorb later milestone semantics.
They prove admission, pacing, locality, debt posture, counters, and visibility.

### Compile-time and construction boundary enforcement

Covered by:

- `crates/worth-store/tests/phase_boundaries_compile_fail.rs`
- `crates/worth-store/tests/ui/tier_maintenance_container_constructor_private.rs`
- `crates/worth-store/tests/ui/late_maintenance_container_constructor_private.rs`

What is proven:

- external callers cannot synthesize raw maintenance declaration ids,
  admitted-work wrappers, reserved-work wrappers, plan wrappers, completed
  receipts, or maintenance containers directly
- raw compaction plans and raw tier plans cannot bypass maintenance admission
- tier, derived-family rebuild, snapshot refresh, replication preparation, and
  audit maintenance declaration payloads have private constructors
- deserialization cannot create admitted declarations, work descriptors,
  maintenance batches, or lowered plan wrappers outside the scheduler proof
  chain

## Acceptance Evidence

The Milestone 11 certification bundle emits:

- `truth_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`
- `scheduler_topology_report`
- `resource_budget_report`
- `maintenance_interference_matrix`
- `debt_escalation_report`

The bundle also carries:

- `Milestone11MaintenanceReport`
- `Milestone11ComplexitySurface`
- `Milestone11CounterContract`
- `Milestone11CertificationSummary`

The certification summary explicitly reports:

- truth matches the control lane
- hidden foreground broadening is absent from representative isolated lanes
- reservation violations match typed counter evidence
- recovered backlog is reported through restart-intake evidence
- scheduler topology is declared
- debt escalation is reported
- cold/warm scheduler equivalence is reported
- tier pressure is contained
- cross-locality escalation remains explicit
- queue timing preserves truth parity
- verified-path and debt-path counts

The interference matrix includes named rows for:

- `isolated`
- `hostile_backlog`
- `deferred`
- `escalated`
- `recovered`
- `coalesced`
- `freshness_rejected`
- `tier_pressure`
- `explicit_cross_locality_debt`

## Additional Hardening Added Before Close

The closeout-readiness pass intentionally corrected real spec/code drift rather
than documenting it as "close enough":

- tier-placement and tier-move work were added as Milestone 11 scheduling
  containers while leaving placement semantics in Milestone 13
- explicit cross-locality tier debt was added and counted separately from
  implicit global fallback
- certification diagnostics were tightened so truth, failure, counters,
  topology, resource budgets, interference matrix, and debt reports all
  participate in the proof digest
- snapshot refresh and replication-preparation work were added as concrete
  scheduler containers rather than vocabulary-only classes
- the final QA loop found `DerivedFamilyRebuild` and `MaintenanceAudit` were
  still vocabulary-only; both now have crate-private declaration payloads,
  durable persistence, scheduler mappings, execution container phases, tests,
  and compile-fail privacy coverage
- the Milestone 11 stable-basis test helper was corrected to mirror production
  digest normalization and select support summaries by both branch and commit

## Explicit Deferrals

No in-scope Milestone 11 closeout debt remains for the shipped scheduler
boundary, proof chain, foreground-interference reporting, restart readmission,
and maintenance-family containment.

Future work still exists, but it belongs to later roadmap layers rather than
hidden Milestone 11 incompleteness:

- heuristic policy tuning may evolve in Milestone 21 budget work
- adaptive topology reshaping, multi-pool work stealing, and dynamic
  density-regime scheduling remain future scheduler-policy work
- Milestone 12 should inherit this runtime for compatibility rebuilds and
  rolling-format maintenance
- Milestone 14 should inherit this runtime for replication preparation and
  capsule integrity workflows
- Milestone 22 should inherit these debt, restart, and operator-evidence
  surfaces for repair and forensic tooling
- Milestone 13 remains the owner of placement meaning; Milestone 11 owns only
  scheduling, pacing, foreground isolation, and debt visibility for tier work

## Verification

The focused closeout verification run used:

- `cargo test -p worth-store milestone_11_maintenance --lib`
- `cargo test -p worth-store milestone_11_certification_bundle --lib`
- `cargo test -p worth-store --test phase_boundaries_compile_fail -- --exact store_mode_phase_boundaries_are_compile_time_private`

All passed after the final closeout-readiness QA corrections.

## Operational Conclusion

Milestone 11 is now closed at the store level.

`worth-store` no longer treats maintenance as an opportunistic background
worker loop. It has one typed maintenance runtime boundary with explicit work
classes, locality, reservations, budget posture, debt, escalation,
foreground-interference reporting, restart readmission, and certification
evidence. Background maintenance can now change cost posture without becoming
shadow authority for truth visibility, which is the milestone's core
architectural law.
