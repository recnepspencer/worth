# Milestone 3.5 And 3.6 Closeout

## Status

Closed.

Milestone `3.5` and Milestone `3.6` are now closed as one combined foundation
program:

- durable media semantics and acknowledgment barriers are explicit
- crash recovery uses typed source precedence instead of backend folklore
- interrupted publication and interrupted maintenance produce typed degraded
  outcomes
- restart becomes quiescent once durable work is already terminal
- certification bundles and named suites now prove the hostile lanes directly

## What Shipped

### 3.5 Durable Media Semantics And Write-Path Certification

Shipped:

- framed durable WAL media with clean, truncated-tail, torn-write, and
  unsupported-version distinction
- backend-family durability barrier vocabulary and explicit acknowledgment
  barriers
- publication-family modeling for:
  - WAL intent
  - WAL canonical result
  - WAL publication progress
  - authoritative append
  - branch-head publication
  - acknowledgment eligibility
  - snapshot basis
  - snapshot image
- typed source-admission separation from raw integrity validation
- machine-checkable write-path evidence bundles with:
  - `write_path_digest`
  - `ack_boundary_report`
  - `media_barrier_matrix`
  - `tail_validation_report`
  - `certification_summary`
  - `observed_failures`

### 3.6 Adversarial Crash Recovery And Recovery Source Precedence

Shipped:

- typed recovery source precedence instead of record-presence heuristics
- typed recovery decisions and degraded outcomes
- explicit quarantine / rebuild / retained-without-acknowledgment surfaces
- interrupted snapshot-publication recovery classification
- scaffolded maintenance-family reporting for non-shipped maintenance families
  without pretending they are present
- backup / restore compatibility reporting with explicit external-restore
  admission boundary
- compile-time-gated authoritative export restore request flow
- operator-facing recovery status reporting with recommended actions
- machine-checkable recovery evidence bundles with:
  - `recovery_source_report`
  - `maintenance_recovery_report`
  - `degraded_state_report`
  - `backup_restore_compatibility_report`
  - `compatibility_digest`
  - `quiescence_report`
  - `recovery_status_report`
  - `certification_summary`
  - `observed_failures`

## Acceptance Mapping

### Spec obligations now materially satisfied

- acknowledgment depends on declared backend-family barriers, not ambient write
  success
- record framing localizes truncated vs torn durable-media failure
- publication-family truth is explicit and typed
- recovery chooses among admitted sources by declared precedence
- incomplete publication families do not bluff retained truth
- interrupted snapshot publication is classified explicitly
- quarantine and retained-without-acknowledgment remain degraded outcomes, not
  silent success
- repeated restart becomes quiescent once terminal recovery evidence exists
- backup / restore compatibility and explicit restore admission are typed rather
  than ambient
- `3.5` and `3.6` certification bundles are machine-checkable, not narrative

### Performance posture

Within the scope of `3.5/3.6`, the write-path proof is now structurally honest:

- hot WAL mutation steps are delta-scoped in memory
- authoritative append is delta-scoped in memory
- branch creation is delta-scoped in memory
- snapshot capture is delta-scoped in memory
- embedded checkpoint persistence is delta-scoped in memory
- targeted snapshot verification replaced hidden whole-family revalidation on
  capture
- exact counters now prove touched breadth and zero clone fallback on the
  admitted hot durable path

Important remaining truth:

- local-file and sqlite persistence still rewrite the whole persisted state
  representation underneath
- true delta-native backend persistence, bulk batching, and larger
  scale-envelope performance programs remain later-roadmap work

That is acceptable for this milestone because the implementation does not claim
that backend physical write breadth is already endgame-grade. It now claims the
truth it can prove.

## Hardening Added Beyond The First Draft

The closeout-worthy hardening that materially improved the milestone includes:

- Law 41 restore-boundary correction from forgeable witness to admitted restore
  request
- typed backup / restore incompatibility records instead of stringly diagnostics
- real publication-family classification rather than backend-local booleans
- recovery source precedence refactor away from ad hoc record walking
- real degraded-state and operator-action surfaces
- harness platform rewrite so certification is scalable rather than milestone-
  local ceremony
- delta-scope counters and proof tests for hot durable mutation paths
- targeted snapshot verification to remove hidden breadth from snapshot capture
- richer `3.5/3.6` evidence bundles carrying typed observed failures and
  certification summaries

## Verification

Verified with:

- `cargo fmt --package worth-store`
- `cargo test -p worth-store`

Current passing result at closeout:

- `78` runtime tests
- compile-fail harness green
- no warning noise

## Remaining Debt

Not closed by this milestone:

- delta-native backend physical persistence
- bulk write batching and amortized publication groups
- broader interrupted-maintenance recovery once compaction / reclaim /
  replication capsule families are actually persisted
- richer salvage execution beyond the typed admitted surface
- later operational envelope and scale-slope benchmarking programs

Those are real future milestones, not hidden incompleteness inside `3.5/3.6`.

## Outcome

`worth-store` now has an honest media and crash-recovery substrate:

- bytes are framed
- barriers are declared
- acknowledgment is typed
- recovery source choice is explicit
- degraded states are visible
- certification proves hostile lanes directly

That closes the combined `Milestone 3.5 / 3.6` foundation program and leaves
later durability work standing on a much harder substrate than the original
Milestone 3 alone.
