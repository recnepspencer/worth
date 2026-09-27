# Phase 8 Plan: Proof-Driven Parallel Preparation and Post-Commit Scaling

## Summary

Implement Phase 8 as a full program for parallel work around serialized authority, with one implementation-ready first slice: deterministic parallel validation reduction. All later Phase 8 workstreams must reuse the same planning substrate: proof-bearing work packets, explicit legality/profitability decisions, worker-local observations, and deterministic reduction into the only authoritative observable outputs.

Runtime execution modes remain separate:
- `SerialAuthority` is the reference path.
- `StagedParallelPreparation` covers planning, validation, diff/index preparation, and import staging over immutable inputs.
- `ParallelPostCommitConsumption` covers downstream fanout over immutable published artifacts.

`worth-harness` is part of delivery, not follow-up. Acceptance is parity-first for every Phase 8 slice.

## Recommended Module Layout

Phase 8 should not be implemented as one broad "parallel" surface. The module
shape needs to preserve the domain split between planning, proofs, packets,
reduction, validation execution, and validation reduction.

Recommended new subsystem layout:

```text
authority/
  commit/
    preparation/
      mod.rs
      facade.rs
      planning/
        mod.rs
        context.rs
        strategy.rs
        work_plan.rs
      proofs/
        mod.rs
        kinds.rs
        locality.rs
        validity.rs
      packets/
        mod.rs
        invariant.rs
        diff.rs
        index.rs
        import.rs
        post_commit.rs
      reduction/
        mod.rs
        keys.rs
        identity.rs
        merge.rs
      diagnostics/
        mod.rs
        observations.rs
        counters.rs
        failures.rs

validation/
  execution/
    mod.rs
    planning.rs
    packets.rs
    worker.rs
    envelope.rs
  reduction/
    mod.rs
    identity.rs
    reducer.rs
    diagnostics.rs
```

Ownership rules:

- `authority/commit/preparation/planning/` owns planning-context identity,
  legality/profitability selection, and packet construction.
- `authority/commit/preparation/proofs/` owns proof kinds, locality semantics,
  and proof validity / reuse boundaries.
- `authority/commit/preparation/packets/` owns packet data shapes only.
- `authority/commit/preparation/reduction/` owns canonical reduction keys,
  reducer identities, and deterministic merge contracts.
- `authority/commit/preparation/diagnostics/` owns Phase 8 diagnostics
  observations, counters, and failure taxonomy.
- `validation/execution/` owns domain-specific validation planning and
  worker-local evaluation.
- `validation/reduction/` owns validation-specific reduction, duplicate
  handling, and final artifact assembly.

Anti-patterns to avoid:

- no `parallel.rs`
- no `phase8.rs`
- no giant `planner.rs`
- do not keep growing existing broad files such as prepare or engine entrypoints
  until they become god modules
- do not mix proof derivation, worker evaluation, diagnostics emission, and
  reduction in one file

## Phase 8 Semantic Guardrails

- Packet proofs are semantic admission artifacts, not scheduling hints.
- Proofs are sufficient conditions for legal parallel admission, not proofs of maximal parallelism.
- Absence of a valid proof requires serial fallback.
- Legality and profitability are distinct decisions and must never be conflated.
- Workers may emit only local observations, counters, and preparation fragments; reducers produce the only authoritative observable preparation outputs.
- Workers may not mutate authoritative runtime state, shared diagnostics state, authoritative identities, publication-visible metadata, patch positions, or index-generation state.
- Post-commit fanout is strictly downstream-only and cannot affect commit visibility, snapshot publication, replay envelope, patch publication, or branch-head movement.
- Canonical reduction keys are stable, subsystem-defined contracts.
- Proof-bearing packets are invalid outside the exact planning context that produced them.

## Implementation Changes

### 1. Add proof-bearing planning and packetization

Introduce a lowered preparation-planning layer after merged-plan construction and structural-summary derivation.

New planning outputs:
- `PreparationWorkPlan` as the parent lowered plan for Phase 8 work
- subsystem packet families:
  - `InvariantWorkPacket`
  - `DiffPreparationPacket`
  - `IndexPreparationPacket`
  - `ImportStagingPacket`
  - `PostCommitConsumerPacket`
- `PreparationStrategy` with explicit fields:
  - `parallel_legality`
  - `parallel_profitability`
  - `selected_mode`
  - `fallback_reason`

Every packet must carry:
- observation scope
- record domain
- partition scope
- invariant-group or fragment scope
- read-set approximation
- write/publication exclusion class
- canonical reduction key
- proof kind

Allowed proof kinds:
- `PartitionDisjoint`
- `InvariantGroupDisjoint`
- `FragmentIdentityDisjoint`
- `ReadOnlyShared`
- `RequiresSerial`

Rules:
- Proofs are semantic, never lock-based.
- Packet proofs are valid only for the exact planning context:
  - merged plan identity
  - target version / observation boundary
  - structural summary
  - touched-scope derivation
  - schema registry identity
  - invariant catalog identity
  - planning contract identity
- Any change to those inputs invalidates the packet proof and requires replanning.

### 2. Define canonical reduction as an explicit contract

Reduction is not â€œwhatever stable sort each subsystem chooses.â€ It is a declared contract per packet family.

Shared contract:
- each packet has a stable canonical reduction key
- reducers merge by canonical key, never worker completion order
- canonical packet identity is included in diagnostics/harness-visible metadata
- reducers own final observable ordering

Concrete now:
- validation packet key is defined in Phase 8:
  - `(execution_point, observation_kind, partition_scope, invariant_group_scope, packet_index)`
- validation result identity is defined in Phase 8:
  - `(execution_point, failure_effect, invariant_rule_identity, target_scope_identity)`

Deferred but mandatory in follow-on slices:
- diff packet key contract
- index packet key contract
- import packet key contract
- post-commit consumer packet key contract

Those later workstreams must define exact tuple shapes before implementation begins; they may not invent them ad hoc inside execution code.

### 3. Implement deterministic parallel validation reduction first

Replace the current serial invariant execution loop with a proof-driven staged pipeline:

1. Build `PlannedInvariantExecution` from immutable observation plus merged plan.
2. Partition work into `InvariantWorkPacket`s with proof kinds and canonical reduction keys.
3. Evaluate packets independently using immutable observation only.
4. Emit worker-local outputs as:
   - invariant observations
   - worker-local counters
   - local diagnostic observations
5. Reduce all worker-local outputs deterministically into:
   - final invariant execution result
   - final diagnostic artifacts
   - final counters

Validation-specific contracts:
- worker-local duplicates are allowed only if they carry the same stable logical identity
- reducer must define:
  - whether duplicates collapse or preserve
  - which payload wins if duplicates conflict
  - whether same-identity payload mismatch is a reducer error
- reducer output must be byte-for-byte parity-compatible with serial mode on all observable surfaces

Diagnostics contract:
- workers emit diagnostic observations only
- reducer emits diagnostic artifacts only
- no shared mutable diagnostics object is visible to workers

### 4. Reuse the same planning substrate for diff, index, import, and post-commit fanout

These workstreams share packet/proof/reduction architecture, but do not share identical legality criteria, fragment identities, or reduction semantics.

Diff preparation:
- packets prepare worker-local diff fragments over immutable inputs
- fragment identity and canonical ordering basis are declared in planning
- final patch publication remains serialized and canonical

Index preparation:
- packets prepare worker-local derived-index fragments only
- index generation remains non-authoritative and version-bound
- storage-visible fallback semantics remain unchanged

Import staging:
- bulk import inputs are normalized and canonicalized before packetization
- fast rejection happens before expensive staging packet construction
- staging artifacts are deterministic and authority-ready

Post-commit consumption:
- packets consume immutable published artifacts only
- consumer failures are downstream/non-authoritative
- no consumer may retroactively affect commit success or canonical commit artifacts

### 5. Keep execution modes separate and define observability promises per mode

`SerialAuthority`
- reference semantics for all observable surfaces
- no staged preparation parallelism
- no downstream fanout parallelism

`StagedParallelPreparation`
- must preserve identical:
  - commit decision
  - invariant verdicts
  - diagnostics artifacts
  - prepared diff/index/import outputs after reduction
  - replay-visible commit artifacts
- may vary only in internal worker scheduling and internal counters that are explicitly marked non-observable

`ParallelPostCommitConsumption`
- must not affect:
  - commit visibility
  - authoritative patch artifact
  - snapshot artifact
  - replay envelope
  - history/branch authoritative state
- may vary only in downstream consumer scheduling and downstream non-authoritative counters

Combined end-to-end exercise happens in harness scenarios, not by adding a new combined runtime mode in this phase.

### 6. Add diagnostics, counters, and failure taxonomy as first-class feature work

Each Phase 8 subsystem must emit structured, versioned diagnostics and boundary-local counters.

Required counters:
- packet counts
- packet width / covered scope
- selected strategy
- legality result
- profitability result
- fallback reason
- worker-local result counts
- reducer input counts
- dedup or conflict counts
- fragment counts
- downstream consumer packet counts and buffering counts

Minimum failure classes across workstreams:
- `PlanningProofInsufficient`
- `PacketOverlapDetected`
- `ReductionIdentityConflict`
- `FallbackToSerial`
- `WorkerEvaluationFailure`
- `FragmentCanonicalizationFailure`
- `PublicationIsolationViolation`
- `ConsumerFailureNonAuthoritative`

Each workstream may add domain-specific failure classes, but may not collapse failures into freeform strings.

## Phase 8 Performance Guardrails

- Packet amortization is required. Legal parallelism is not enough; packets must carry enough work to amortize scheduling, handoff, local-envelope construction, and reduction cost.
- No hot-path proof re-derivation. Touched scope, locality, group applicability, fragment identity, and reduction identity are computed once in planning and forwarded.
- Prefer canonical merge of packet-local ordered outputs over global gather-and-sort, unless a subsystem explicitly proves full-sort is acceptable.
- Diagnostics must have subsystem-specific operational ceilings, forensic expansion paths, and bounded fragment counts.
- Parallel admission must consider both semantic disjointness and storage/chunk locality.
- Diff, index, and import preparation must remain proportional to semantic delta; any full rebuild or full scan must be explicit, diagnosable, and policy-visible.
- Worker-local envelopes and fragments must avoid per-record heap churn where practical and preserve a bounded small-work fast path.
- Post-commit fanout must have separate counters, bounded concurrency, and bounded buffering so downstream work does not hide commit cost.
- Profitability must be measured, not assumed. Strategy selection must record why serial or staged-parallel was selected.

## Build Order

1. Add `PreparationWorkPlan`, proof kinds, planning-context identity, packet identities, and `PreparationStrategy`.
2. Implement deterministic parallel validation planning, packetization, worker execution, reduction, diagnostics observations, artifact reduction, and counters.
3. Add `worth-harness` serial-vs-staged-parallel parity suites for validation, including success, advisory, failure, replay parity, and diagnostics parity.
4. Extend the same planning substrate to diff preparation, with explicit fragment identity and reduction-key contracts.
5. Extend it to index preparation, preserving non-authoritative semantics and storage fallback behavior.
6. Extend it to import staging with deterministic packetization and early rejection.
7. Add post-commit consumer packetization and bounded parallel downstream fanout over immutable artifacts.
8. Expand harness matrices to end-to-end combined scenarios using staged preparation plus parallel downstream consumption while serialized authority remains unchanged.

## Implementation Slices

### Slice 1: Planning substrate and validation packetization

Deliver:

- `PreparationWorkPlan`
- planning-context identity
- proof kinds and proof validity contract
- `PreparationStrategy`
- `InvariantWorkPacket`
- canonical validation reduction key and validation result identity contracts

Do not deliver yet:

- worker parallel execution
- diff/index/import packet execution
- post-commit fanout

### Slice 2: Deterministic validation workers and reducer

Deliver:

- `PlannedInvariantExecution`
- worker-local validation envelopes
- diagnostic observations emitted by workers
- validation reducer
- duplicate/conflict handling contract
- serial fallback and legality/profitability instrumentation

Acceptance focus:

- serial-vs-staged-parallel parity
- deterministic reduction under hostile scheduling
- reducer conflict diagnostics

### Slice 3: Harness parity and diagnostics hardening

Deliver:

- `worth-harness` parity matrix for validation
- diagnostics parity comparisons
- replay parity comparisons
- exportable failure artifacts for reducer conflicts and fallback-to-serial

Acceptance focus:

- harness becomes the default acceptance surface for validation parallelism

### Slice 4: Follow-on packet families on the same substrate

Deliver incrementally:

- diff preparation packet family and reducer contract
- index preparation packet family and reducer contract
- import staging packet family and reducer contract
- post-commit consumer packet family and downstream isolation counters

Gate for each follow-on slice:

- exact packet key contract is written before code lands
- reducer identity and duplicate semantics are written before code lands
- authoritative semantics remain unchanged

## Test Plan

### Parity and determinism
- serial vs staged-parallel validation parity for pass, advisory, and blocking failure cases
- diagnostics parity for success and failure paths
- replay parity across serial and staged-parallel preparation modes
- identical outputs under different worker scheduling/interleavings
- canonical reducer ordering independent of packet completion order

### Proof and fallback behavior
- absence of proof causes serial fallback
- invalidated planning context forces replanning
- overlapping packet claims produce `PacketOverlapDetected` or serial fallback according to subsystem contract
- legality true / profitability false produces explicit serial fallback with recorded reason
- no speculative locking or â€œtry parallel then recoverâ€ paths

### Reducer identity
- duplicate worker-local validation observations with same identity reduce correctly
- mismatched payload under same logical identity produces reducer conflict error
- diagnostics observations reduce into canonical artifacts only at the reducer boundary

### Workstream-specific
- diff preparation parity and canonical fragment ordering
- index preparation parity and storage-visible fallback preservation
- import staging parity and deterministic packet ordering
- post-commit consumer parallelism has no effect on authoritative commit artifacts

### Scale-sensitive checks
- packet counts and covered scope stay proportional to semantic delta
- proof facts are not recomputed in workers or reducers
- no shared mutable authoritative patch/diagnostics object is visible to workers
- downstream fanout cost is separately observable from commit cost

## Assumptions and Defaults

- This is a full Phase 8 program plan, but only deterministic parallel validation reduction is implementation-ready in maximum detail.
- Diff preparation, index preparation, import staging, and post-commit fanout are bounded follow-on slices on the same planning substrate; each must define its exact reduction-key and reducer-identity contracts before implementation.
- Acceptance is parity-first, not benchmark-threshold-first.
- Runtime modes remain separate; combined semantics are exercised by harness matrices, not by adding a new combined runtime mode in this phase.
- If any Phase A-F boundary is too weak to carry proof-bearing packetization cleanly, reopening that boundary is allowed and preferred over weakening the Phase 8 design.

## Current Status

Phase 8 is substantially implemented in runtime code.

Shipped runtime surfaces:

- proof-bearing preparation planning and strategy selection
- deterministic staged-parallel validation reduction
- deterministic diff preparation over packet-local ordered fragments
- deterministic derived-index preparation over packetized immutable inputs
- deterministic import staging for bulk entity and relation create
- bounded parallel post-commit consumer packetization over immutable published artifacts
- canonical merge-based reduction instead of global gather-and-sort on the main Phase 8 hot paths
- harness-aware execution-mode selection for serial, staged-preparation, and post-commit-parallel lanes

Shipped harness acceptance surfaces:

- serial vs staged-preparation parity
- serial vs post-commit-parallel parity
- reusable certification matrix with baseline diagnostics plus per-candidate diagnostics and comparisons
- harness-visible preparation fallback reporting
- harness-visible preparation failure reporting for:
  - `FallbackToSerial`
  - `PlanningProofInsufficient`
  - `PublicationIsolationViolation`
  - `ReductionIdentityConflict`
  - `WorkerEvaluationFailure`
  - `FragmentCanonicalizationFailure`
  - `PacketOverlapDetected`
  - `ConsumerFailureNonAuthoritative`

## Observability Status

The runtime now measures the core Phase 8 signals directly in `RuntimeComplexityCounters`.

Shipped preparation counters:

- `preparation_packet_count`
- `preparation_packet_item_count`
- `preparation_packet_peak_width_total`
- `preparation_scope_unit_count`
- `preparation_parallel_legal_count`
- `preparation_parallel_profitable_count`
- `preparation_serial_strategy_count`
- `preparation_staged_parallel_strategy_count`
- `preparation_reducer_conflict_count`

Shipped post-commit counters:

- `post_commit_consumer_packet_count`
- `post_commit_consumer_item_count`
- `post_commit_consumer_peak_width_total`
- `post_commit_scope_unit_count`
- `post_commit_serial_strategy_count`
- `post_commit_parallel_strategy_count`

Structured validation preparation counters also exist at the subsystem boundary:

- packet count
- worker result count
- reducer input count
- reducer conflict count
- failure count

Packet shape and covered-scope observability now ship as first-class runtime counters rather
than being inferred indirectly from tests or workload shape. The remaining fragment-shape details
for specific subsystems are already exposed through structured diagnostics and packet families, so
there is no known Phase 8 observability gap blocking closeout.

## Harness Evidence

Current harness evidence now proves:

- supported runtime lanes preserve parity against serial baseline:
  - `SerialAuthority`
  - `StagedParallelPreparation`
  - `ParallelPostCommitConsumption`
- diagnostics summaries expose execution mode and selected runtime execution model
- staged preparation fallback remains parity-safe while reporting:
  - fallback reason
  - serial strategy selection
  - structured `PreparationFailure` / `PreparationFallback` artifacts
- injected structured failure classes surface through the real harness path:
  - `planning_proof_insufficient`
  - `publication_isolation_violation`
  - `reduction_identity_conflict`
  - `worker_evaluation_failure`
  - `fragment_canonicalization_failure`
  - `packet_overlap_detected`
  - `consumer_failure_non_authoritative`

The matrix-driven closeout path now certifies the full current Phase 8 failure taxonomy.

## Exit Guidance

Phase 8 is complete.

Completion evidence:

- harness matrix coverage is the primary acceptance surface for supported runtime lanes
- the supported runtime lanes are certified against a serial baseline:
  - `SerialAuthority`
  - `StagedParallelPreparation`
  - `ParallelPostCommitConsumption`
- the current structured preparation failure taxonomy is harness-visible and matrix-certified
- packet-shape and covered-scope observability are emitted as first-class runtime counters
- this document now reflects the exact shipped runtime surfaces and acceptance evidence rather than planned-only intent
