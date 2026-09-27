# Worth Store Milestone 12 Closeout Implementation Plan

## Summary

Finish Milestone 12 honestly.

The compatibility subsystem now has real vocabulary, typed planning, hostile
rejection surfaces, compile-fail proof boundaries, and a deterministic
certification runner. What it does not yet have is the remaining runtime and
persistence work required to make `Artifact Format Evolution And Rolling
Compatibility` true at the real store boundary.

This plan starts from the current runtime-gap labels and burns them down in the
order required for an honest milestone closeout:

1. durable manifest persistence
2. facade read/write/restore integration
3. restore publication execution
4. rolling writer publication execution
5. derived rebuild execution through the already-closed Milestone 11 runtime
6. adapter execution or explicit adapter scope reduction
7. closeout evidence and closeout documentation

The assumption for this plan is explicit: Milestone 11 is already closed
honestly, so Milestone 12 must now consume its maintenance runtime rather than
continue deferring compatibility-triggered rebuild execution.

## Governing Constraint

Milestone 12 is not closeable while compatibility remains mostly a planning
subsystem with certification evidence wrapped around it.

The store must force real runtime behavior:

- persisted manifests must survive restart and reconstruct compatibility
  admission truth without artifact-row folklore
- authoritative read, write, and restore surfaces must require compatibility
  witnesses before semantic meaning is exposed
- restore publication must refuse unsafe truth visibility in the real store,
  not only in planning helpers
- rolling upgrades must constrain real writer publication behavior, not only
  produce admitted/rejected plans
- derived compatibility drift must produce real rebuild execution through
  Milestone 11 scheduler containers or real typed refusal
- adapters must either execute through bounded, parity-proven paths or be
  removed from the claimed first-ship surface

If any of those remain planning-only, Milestone 12 still fails its own
adversarial constraint even if the certification runner is deterministic.

## Current State To Preserve

- Compatibility catalog, manifest digest identities, declared edge registry,
  decode quarantine, authoritative admission, derived planning, rolling
  planning, restore planning, disaster-recovery classification, and
  certification evidence already exist under
  `crates/worth-store/src/compatibility/`.
- The certification runner in
  `crates/worth-store/src/compatibility/certification_runner.rs` is real and
  should remain the named M12 evidence entry point.
- Compile-fail boundaries already prevent external fabrication of proof-bearing
  compatibility, restore, rolling, derived, and certification types.
- Milestone 11 is already closed and provides:
  - scheduler-admitted `DerivedFamilyRebuild` containment
  - restart-honest maintenance admission
  - queue/debt/escalation evidence
  - foreground-safe execution boundaries
- Milestone 13 is already closed and provides tier placement vocabulary that
  compatibility must preserve as non-authority.

## Runtime Gaps To Eliminate

The current certification runner exposes these deferred gaps:

- `durable_manifest_persistence_deferred`
- `facade_read_write_restore_integration_deferred`
- `restore_publication_execution_deferred`
- `rolling_writer_publication_deferred`
- `adapter_execution_deferred`
- `derived_rebuild_execution_deferred`

This plan exists to remove those labels from the runner rather than merely
report them.

## Implementation Order

Implement closeout in this order so each step turns one proof-only surface into
real store behavior and leaves the milestone strictly more honest than before.

1. Durable manifest persistence and restart reconstruction
2. Compatibility-gated authoritative facade reads and writes
3. Restore publication execution and visibility blocking
4. Rolling writer publication and mixed-version operational lanes
5. Derived rebuild execution through Milestone 11
6. Adapter execution decision: ship or cut
7. Named closeout suite hardening, closeout doc, and roadmap/spec status flip

Reason:

- persisted manifests are the basis for honest restart-visible compatibility
- facade enforcement must happen before restore or rolling execution can claim
  real store behavior
- restore and rolling are the operational consequences of compatibility
- derived rebuild execution depends on the already-closed M11 runtime
- adapter execution should be resolved late because the honest answer may be
  "cut adapters from M12 first ship" if execution cannot meet the milestone bar

## Phase 6A: Durable Manifest Persistence And Restart Reconstruction

### Goal

Make compatibility manifests real durable store state rather than in-memory
planning evidence.

### Required work

- add durable backend records for compatibility manifests, publication units,
  recovered frontier summaries, and receipt/registry basis where restart needs
  it
- add SQLite persistence and restart-load support for the compatibility manifest
  ledger
- ensure compatibility index reconstruction on reopen is manifest-backed and
  registry-backed, never artifact-row-scan-backed
- make manifest gap, digest drift, and recovery-window failures surface during
  backend open/recovery rather than only through direct planner invocation
- extend evidence to report durable manifest reads/writes/recovery counts

### Rules

- manifest persistence must align with subsystem boundaries; do not bury it in
  a generic persistence helper
- recovered compatibility state must remain proof-bearing and read-only outside
  the compatibility subsystem
- restart reconstruction must consume persisted manifests plus registry
  declarations only
- open-time failures must be typed compatibility failures, not generic backend
  corruption strings

### Tests

- SQLite reopen reconstructs compatibility index from persisted manifests
- local-file and SQLite manifest gap lanes fail typed on open
- manifest digest mismatch survives persistence/reopen and rejects before
  semantic access
- compatibility index rebuild counters prove manifest-bounded reconstruction
  with zero artifact-row scan dependence

### Exit condition

- the runner can remove `durable_manifest_persistence_deferred`

## Phase 6B: Compatibility-Gated Facade Reads, Writes, And Restore Entry

### Goal

Move compatibility from helper calls into the real store entry surfaces.

### Required work

- identify the authoritative store facade read/write/restore entry points that
  currently bypass compatibility admission
- require compatibility-admitted artifact forms before exposing semantic
  authoritative meaning to replay, branch-head, schema, lineage, cursor,
  checkpoint, and restore consumers
- route write publication through writer capability admission and persisted
  manifest publication
- route restore entry through restore compatibility planning rather than raw
  backend intake
- expose real compatibility diagnostics/counters from the facade boundary

### Rules

- facade surfaces must stay the only public boundary; no external caller should
  compose raw compatibility internals directly
- decode success must still terminate in quarantine until the facade-owned
  compatibility proof step completes
- read-path compatibility proof must happen before any support-family meaning is
  exposed
- write-path publication must fail before partial truth publication if the
  compatibility contract is missing

### Tests

- store-level authoritative read lanes reject missing-edge and partial-truth
  cases through the facade
- store-level write lanes reject unsupported writer capability or missing
  manifest publication basis
- schema/lineage/cursor/checkpoint support artifacts cannot be semantically
  accessed from raw decoded forms at the facade boundary
- compatibility counters emitted through the store surface match the planner
  evidence

### Exit condition

- the runner can remove `facade_read_write_restore_integration_deferred`

## Phase 6C: Restore Publication Execution

### Goal

Turn restore publication witnesses into real store-visible restore behavior.

### Required work

- implement restore execution that consumes only typed restore publication
  witnesses
- block truth visibility until all authoritative families required by the
  restore plan pass compatibility and publication conflict checks
- preserve the distinction between authoritative truth restoration and derived
  acceleration restoration
- persist any restore-publication state required for restart-honest execution
  if the restore operation crosses a durable boundary
- emit restore execution counters and failure evidence from the operational
  path, not a synthetic wrapper

### Rules

- planning and execution remain separate subdomains
- restore execution must not widen scan scope beyond backup families and
  declared publication conflicts
- unsafe restore windows must fail before branch heads, cursors, schema support,
  lineage support, tier manifests, or snapshots become visible

### Tests

- real restore execution publishes safe windows only after full compatibility
  admission
- unsafe restore windows remain invisible and fail typed
- scoped restore counters prove unrelated target-store families were not scanned
- restart/reopen after interrupted restore does not invent clean publication

### Exit condition

- the runner can remove `restore_publication_execution_deferred`

## Phase 6D: Rolling Writer Publication And Mixed-Version Operational Closure

### Goal

Make rolling upgrade compatibility affect real writer behavior.

### Required work

- connect rolling admission plans to real writer publication gates
- make mixed-version store posture and mixed-version replica posture visible
  from operational surfaces intended for later Milestone 14 use
- reject unsupported multi-writer, unsupported skew, missing-edge, and
  out-of-policy windows during actual publication attempts
- ensure admitted rolling windows preserve selected relations through the real
  publication path

### Rules

- rolling policy remains explicit first-ship policy, not deployment folklore
- no numeric version proximity inference at runtime
- operational rolling state must remain declarative enough for certification and
  later replication to consume

### Tests

- rolling admitted publication lane writes successfully only inside the declared
  two-capability window
- rolling multi-writer and missing-edge lanes reject through the real writer
  path
- mixed-version posture survives restart if persisted
- operational counters match certification lane evidence for rolling windows

### Exit condition

- the runner can remove `rolling_writer_publication_deferred`

## Phase 6E: Derived Rebuild Execution Through Milestone 11

### Goal

Cash in the Milestone 11 closure by making compatibility drift produce actual
maintenance-executed rebuild work.

### Required work

- map compatibility rebuild-required outcomes onto Milestone 11
  `DerivedFamilyRebuild` scheduler containers
- persist any compatibility-specific rebuild payload required for restart
  readmission
- preserve rebuild debt when rebuild is deferred or blocked
- keep authoritative-versus-derived boundaries strict: rebuilds consume retained
  authority, they do not create new authority
- surface foreground interference, debt, and restart readmission through the
  existing Milestone 11 evidence model where appropriate

### Rules

- do not invent a separate compatibility worker loop
- compatibility owns why rebuild is required; Milestone 11 owns admission,
  pacing, restart, and execution containment
- stale/incompatible derived artifacts must end in one of three states only:
  rebuilt, explicitly invalidated, or typed rejected

### Tests

- compatibility-triggered derived rebuild enters Milestone 11 admission with the
  expected declaration family and counters
- deferred rebuild preserves rebuild debt exactly
- restarted backlog re-enters through the same scheduler path
- derived artifacts cannot continue presenting as exact once rebuild is pending
  or rejected

### Exit condition

- the runner can remove `derived_rebuild_execution_deferred`

## Phase 6F: Adapter Execution Resolution

### Goal

Resolve adapter execution honestly instead of leaving it in ambiguous limbo.

### Required work

Choose one of these paths and record it explicitly in the milestone spec before
closeout:

1. Ship adapter execution:
   - implement bounded execution for admitted adapters
   - keep hot-read rejection for non-admitted cost classes
   - bind parity evidence and adapter digests into the execution result
   - route maintenance-scheduled adapter work through Milestone 11 where needed

2. Cut adapters from first-ship M12 scope:
   - remove adapter execution promises from the milestone spec's must-ship
     wording
   - keep adapter-edge rejection as defensive future-proofing only
   - update the certification runner and closeout doc to state adapters were not
     shipped as an admitted execution path in M12

### Preferred default

Prefer cutting adapters from first-ship closeout unless real execution can be
implemented with parity proof and bounded cost in the same closeout window.
The milestone is about compatibility truth, not about preserving speculative
adapter breadth at any cost.

### Exit condition

- either the runner removes `adapter_execution_deferred`, or the spec/closeout
  explicitly narrows adapters out of the shipped surface

## Phase 7: Closeout Hardening And Documentation

### Goal

Close Milestone 12 the same way Milestones 11 and 13 were closed: code,
machine-checkable evidence, and explicit documentation all agree.

### Required work

- harden the named `Artifact Format Evolution And Rolling Compatibility Test`
  around the real store/runtime surfaces, not only compatibility helpers
- extend the certification runner or named certification tests so the final
  evidence no longer reports runtime-gap labels for shipped surfaces
- write `plans/worth-store/milestone-12-closeout.md`
- update `plans/worth-store/milestone-12.md` from draft to closed status with a
  closeout link
- update `plans/worth-store/worth_store_roadmap.md` to mark Milestone 12 closed
- record exact verification commands and any explicit remaining later-milestone
  debt only if it is truly out of M12 scope

### Closeout acceptance mapping must include

- durable manifest persistence and restart recovery
- real facade compatibility gates
- restore publication execution
- rolling writer publication closure
- derived rebuild execution through Milestone 11
- adapter execution shipped or adapter scope explicitly narrowed
- compile-time witness/privacy coverage
- exact counter and diagnostics evidence

## Tests To Run

Minimum focused verification before claiming closeout:

```text
cargo fmt -p worth-store
cargo test -p worth-store compatibility --lib
cargo test -p worth-store milestone_12 --lib
cargo test -p worth-store artifact_format_evolution --lib
cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1
```

Expected additional focused suites created during closeout:

```text
cargo test -p worth-store compatibility_persistence --lib
cargo test -p worth-store compatibility_facade --lib
cargo test -p worth-store compatibility_restore_execution --lib
cargo test -p worth-store compatibility_rolling_execution --lib
cargo test -p worth-store compatibility_rebuild_execution --lib
```

If the exact test filter names differ, the closeout doc must name the real
ones.

## Explicit Non-Goals

- No new semantic authority beyond compatibility admission, restore safety, and
  derived rebuild honesty
- No Milestone 14 replication semantics beyond mixed-version posture surfaces
- No Milestone 15 extension-family registration work
- No Milestone 20 blob/object compatibility model
- No Milestone 22 operator repair semantics beyond what M12 already needs for
  typed restore/rebuild/rejection behavior

## Exit Condition

Milestone 12 is complete when all current runtime-gap labels are removed or
resolved by explicit scope narrowing, the real store facade and backend runtime
enforce compatibility before semantic truth exposure, compatibility-triggered
derived rebuild executes through Milestone 11, restore and rolling lanes affect
real publication behavior, and `plans/worth-store/milestone-12-closeout.md`
can honestly say the named M12 suite is passing against shipped runtime
behavior rather than planning-only evidence.
