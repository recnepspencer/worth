# Milestone 9 Engineering Spec: Deterministic Bulk Ingest And Bulk Transform Paths

> **Status:** Closed via [milestone-9-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9-closeout.md)
>
> **Closeout:** [milestone-9-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9-closeout.md)
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-1.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-1.md)
> - [milestone-2.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-2.md)
> - [milestone-3.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-3.md)
> - [milestone-3.5-3.6.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-3.5-3.6.md)
> - [milestone-4.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-4.md)
> - [milestone-5.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5.md)
>
> **Concurrent milestone:** Milestone 6 (`Aspect-Aware Physical Layout And Content-Addressed Structural Blocks`)
>
> **Primary architectural driver:** make large ingest and rewrite programs resume
> through one deterministic chunk plan that still commits through the canonical
> store append path, while allowing Milestone 6 to evolve the physical chunk
> substrate concurrently without letting bulk execution depend on backend-local
> layout tricks

## Goal

Make large ingest, migration, and rewrite programs first-class store programs
that remain resumable, bounded, and replay-honest instead of living as ad hoc
utility flows outside the durable authority model.

## Implementation Snapshot

The codebase has moved beyond the speculative shape of this spec.

Implemented now in `worth-store`:

- proof-bearing ingest and transform planning surfaces
- frozen ingest manifests, frozen transform bases, and frozen transform target
  partitions
- deterministic chunk plans and budget-admitted chunk execution
- persisted bulk support artifacts:
  - program identity
  - manifests, bases, and partitions
  - deterministic plans
  - chunk witnesses
  - progress checkpoints
  - per-program witness indexes
- durable bulk execution through canonical append and WAL-bound publication
  phases
- bulk-specific recovery identity, reporting, and recovered-resume admission
- restart-path reconstruction of witnesses and checkpoints from published truth
- reopen-time integrity checks for checkpoint families, witness indexes,
  transform artifacts, and deterministic plan payload drift

The named Milestone 9 certification lane now exists, the machine-checkable
bundle is emitted from `crates/worth-store/src/evidence/milestone_9.rs`, and
the authoritative closeout mapping is recorded in
`milestone-9-closeout.md`.

This means Phases 1 through 5 are now implemented and closed for Milestone 9.

## Why This Milestone Exists

Milestone 9 is not "add a fast import path."

It is the milestone that decides whether `worth-store` can absorb large
operational workloads without inventing:

- a second commit model for bulk jobs
- backend-local resumability checkpoints that are not replay-safe
- chunk heuristics that change final truth depending on interruption timing
- memory-hungry staging pipelines that only work for one dataset shape

Milestone 1 froze canonical commit authority.

Milestone 2 froze operating-mode ownership.

Milestone 3 and Milestone 3.5/3.6 froze durable publication and recovery-source
precedence.

Milestone 4 proved that a derived family can accelerate restore without
becoming truth.

Milestone 5 proved that branch-local physical work can be derived, rebuildable,
and replay-parity safe.

Milestone 6 is being built concurrently to make the physical chunk substrate
honest enough for aspect-aware layout and structural block reuse.

Milestone 9 now has to build the program-level bulk contract on top of those
foundations without taking ownership of physical chunk layout:

- what a bulk program is allowed to plan once and resume later
- what chunk identity means at the canonical execution boundary
- what progress checkpoints are allowed to remember
- what must be recomputed from authority instead of trusted from interrupted
  local state
- what parity must hold between bulk execution and the logically serial control
  lane

If this milestone is weak, later retention, replication, certification, and
platform operations will inherit a dangerous split:

- ordinary writes will be canonical, but bulk writes will be "special"
- recovery will be exact for transactions but heuristic for imports
- progress checkpoints will look durable while secretly depending on local temp
  state
- chunking will become a hidden semantic decision instead of a declared cost
  surface

This milestone exists to make bulk operations honest before anyone depends on
them for real migrations, bootstrap imports, branch rewrites, or operator-scale
backfills.

## Hard Part

The hard part is not batching rows.

The hard part is keeping five things separate that naive bulk systems collapse
into one blurred pipeline:

- canonical commit authority
- deterministic bulk program planning
- physical chunk materialization
- resumable execution checkpoints
- backend-specific throughput optimizations

The design fails if:

- a resumed run can reach a different final history than a non-interrupted run
- the same source payload or transform target can be enumerated in different
  orders on different hosts and still claim to have the same plan identity
- chunk boundaries drift because the backend or host machine chose a different
  buffering strategy on retry
- progress checkpoints remember mutable partial state instead of exact canonical
  next-work identity
- a chunk commits successfully but cannot later be proven to correspond to one
  exact program and chunk ordinal
- a long-running transform resumes after the target branch frontier moved and
  silently rewrites against the new frontier
- bulk ingest bypasses the ordinary commit/WAL path because it is "faster"
- Milestone 6 physical chunk evolution forces Milestone 9 to understand
  structural blocks, aspect pages, or backend-local dedup internals

Milestone 9 therefore has to make chunk planning deterministic enough for
resume parity while remaining abstract enough that Milestone 6 can keep
changing the physical chunk representation underneath the stable chunk contract.

## Explicit Assumptions

- Milestone 1 authoritative commit envelopes remain the only semantic durable
  truth for committed history.
- Milestone 2 operating-mode ownership remains unchanged; Milestone 9 adds bulk
  programs, not a new operating mode.
- Milestone 3 and Milestone 3.5/3.6 publication, WAL, recovery precedence, and
  degraded-state rules already govern any bulk execution path.
- Milestone 4 snapshot basis and restore rules remain intact; bulk programs may
  later consume snapshot or export sources, but this milestone does not
  renegotiate snapshot meaning.
- Milestone 5 branch-delta identity and rebuildability remain the canonical
  branch-local physical substrate for bulk programs that target existing branch
  history.
- Milestone 6 is being authored and implemented concurrently; Milestone 9 may
  depend on a stable chunk identity contract, chunk-width vocabulary, and
  chunk-materialization receipt, but it must not depend on structural-block
  layout, aspect-page shape, or dedup internals.
- `worth-relational` still owns truth semantics, commit legality, branch
  semantics, schema semantics, and replay meaning.
- bulk ingest and bulk transform are execution programs over canonical commits;
  they are not a second durable truth language.
- resumable progress checkpoints are support artifacts for program execution;
  they are never authoritative truth and must remain rebuildable or ignorable
  without changing committed history.
- later retention, replication, and certification milestones may consume bulk
  program evidence, but Milestone 9 must not pre-empt their authority or export
  boundaries.

## Mechanical Enforcement Targets

Milestone 9 must not rely on reviewers remembering the rules.

At minimum, the implementation shape this spec implies should make these states
unrepresentable or uncompilable:

- constructing an executable bulk program without a frozen source or transform
  basis witness
- executing a chunk without a budget-admitted chunk plan
- publishing a progress checkpoint for a chunk that does not have a canonical
  chunk commit witness
- resuming a transform after target drift without first revalidating the locked
  basis and producing an explicit continue-or-reject decision
- reconstructing resume position from host-local staging state instead of
  canonical history plus checkpoint artifacts

Suggested proof-bearing type families:

- `FrozenBulkSourceManifest`
- `FrozenTransformBasis`
- `FrozenTransformTargetPartition`
- `DeterministicChunkPlan`
- `BudgetAdmittedChunkPlan`
- `BulkChunkCommitWitness`
- `PublishedProgressCheckpoint`
- `ResumeReadyBulkProgram`

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile operational
  failure before convenience spreads. Milestone 9 therefore starts from resume
  determinism, bounded memory, and interruption parity, not from "imports
  should be faster."
- `arch_laws.md`
  The most important thing it protects here is authority separation and
  proof-bearing phase boundaries. Canonical commits, bulk plans, chunk receipts,
  progress checkpoints, and execution evidence must be separate types and
  separate phases rather than one mutable job object.
- `perf_laws.md`
  The most important thing it protects is cost honesty. Milestone 9 therefore
  has to name chunk-width, resume breadth, and peak in-flight memory contracts
  explicitly instead of hiding them behind a bulk helper that only looks cheap
  at small scale.
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped decomposition.
  Bulk planning, chunk admission, progress checkpoint persistence, WAL-bound
  execution, resume planning, and certification evidence must stay in separate
  subdomains rather than one catch-all migration utility module.
- `worth_store_vision.md`
  The most important thing it protects is that store makes truth survive
  without making it dumber. Milestone 9 must therefore route bulk work through
  canonical commit truth and durable recovery rules instead of inventing a
  special import truth path.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 9 belongs after
  the branch-delta foundation and alongside late Milestone 6 because bulk
  chunking needs an honest physical chunk contract, but bulk resumability and
  parity should not wait for all layout optimization detail to be complete.
- `worth-store/test-requirements.md`
  The most important thing it protects is certification-grade proof.
  Milestone 9 is not closeable until the `Bulk Ingest And Transform Resume
  Parity Test` proves final-truth parity, deterministic chunk boundaries, WAL
  recovery parity, and machine-checkable boundedness evidence.
- `milestone-5.md`
  The most important thing it protects is derived physical work staying
  subordinate to canonical history. Milestone 9 must preserve that rule for
  branch-targeted bulk rewrites, chunk checkpoints, and rebuild of interrupted
  program residue.
- `milestone-7.md`
  The most important thing it protects is durable support artifacts staying
  explicit and non-ambient. Milestone 9 should mirror that discipline: progress
  checkpoints are support artifacts keyed to exact program and chunk identity,
  not temp files or caller memory.
- `worth_store_dependency_map.md`
  The most important thing it protects is the real unlock shape: Milestone 5
  makes bulk work admissible, Milestone 6 stabilizes the chunk contract, and
  Milestone 9 adds operational bulk capability without unlocking a new core
  foundation milestone by itself.

## Adversarial Constraint

Milestone 9 must survive this hostile condition:

> A store executing long-running ingest and transform programs across deep
> branch history, crash-restart loops, backend variation, partial chunk
> publication, WAL recovery, and concurrent Milestone 6 chunk-substrate
> evolution must resume to the same final canonical truth, commit history, and
> branch-head meaning as a logically serial control lane that executes the same
> admitted source data and transforms through ordinary canonical commits alone,
> while staying within declared chunk-memory budgets.

## Product Decision Lock

- bulk ingest and bulk transform always lower into ordinary canonical commit
  envelopes before truth becomes durable
- no bulk path may invent a second durable append format, branch-head update
  rule, or replay rule
- every bulk program must publish one deterministic program identity and one
  deterministic chunk plan identity for the admitted source and options
- the admitted source for ingest must be frozen as an ordered source manifest;
  "whatever files are present when we retry" is out of spec
- the admitted basis for transform must be frozen as an exact target frontier or
  explicit branch-basis witness before chunk execution begins
- progress checkpoints may record only declared plan identity, completed chunk
  frontier, resume cursor, and machine-checkable evidence needed to continue
- canonical commits emitted by bulk execution must carry enough program/chunk
  provenance to reconstruct completed chunk ordinals during resume planning
- partial chunk work that has not crossed the canonical commit boundary is
  discardable execution residue, not resumable truth
- durable-mode bulk execution must obey the same WAL-before-acknowledgment rule
  as ordinary canonical commits
- resume planning must be able to reconstruct next work from canonical truth
  plus persisted checkpoint artifacts without trusting host-local temp state
- bulk transform resume must revalidate the original transform basis lock rather
  than silently rebasing onto a newer branch frontier
- Milestone 9 may consume the stable chunk contract exposed by concurrent
  Milestone 6, but it may not inspect or depend on backend-local structural
  block composition, dedup tables, or aspect-layout internals
- deleting all progress checkpoints must never change committed truth, even if
  it forces whole-program replay from the original source
- bulk evidence must be machine-checkable enough for later certification,
  retention, and replication work to reason about bulk history honestly

Normative consequence:

- any implementation that writes truth durably before it can restate that truth
  as ordinary canonical commits is out of spec
- any implementation that resumes from host-local mutable staging files without
  an admitted durable checkpoint artifact is out of spec
- any implementation that cannot restate committed bulk progress from canonical
  history as exact program/chunk witnesses is out of spec
- any implementation that lets chunk boundaries vary with retry timing,
  allocator pressure, or backend-local physical packing is out of spec
- any implementation that resumes a transform against a different branch
  frontier without an explicit typed drift decision is out of spec
- any implementation that requires Milestone 6 internal layout details to decide
  whether a chunk is canonical is out of spec

## Scope

### In Scope

- deterministic bulk-program manifest and admitted source classification
- frozen ordered source manifests for ingest and frozen transform-basis locks
  for rewrite programs
- deterministic chunk planning over ingest and transform workloads
- resumable progress checkpoint artifacts keyed to explicit program and chunk
  identity
- canonical chunk commit witnesses sufficient to reconstruct completed ordinals
  from committed history alone
- durable-mode bulk execution through the canonical commit and WAL boundary
- branch-targeted bulk transforms that preserve ordinary branch and replay
  meaning
- resume planning after interruption, restart, or crash using durable
  checkpoints plus canonical truth
- bounded-memory chunk admission, chunk diagnostics, and certification evidence
- parity comparison between bulk execution lanes and logically serial control
  lanes
- public store vocabulary for bulk planning, chunk execution, checkpoint
  inspection, resume planning, and certification output

### Explicitly Out Of Scope

- defining the physical chunk substrate, aspect-page layout, or structural-block
  dedup internals owned by Milestone 6
- retention, compaction, reclaim, or tier movement policy for bulk residue
- replication capsules, import/export protocol negotiation, or cross-machine
  bulk program shipping
- live-query continuation, subscriber delivery, or cursor narrowing semantics
- profitability scheduling, adaptive autotuning, or cluster-scale orchestration
  beyond exact declared budgets and counters
- schema migration meaning beyond executing runtime-legal transforms through the
  canonical append path

## Bulk Program Authority Model

### Admitted Bulk Program Families

Milestone 9 must freeze what kind of program is being resumed.

Every admitted bulk run in this milestone must be classified as exactly one of:

- `BulkIngestProgram`
- `BulkTransformProgram`

Both families must carry:

- `program_id`
- `plan_id`
- `source_identity`
- `target_branch_scope`
- `admitted_chunking_policy`
- `program_version`
- `proof_of_canonical_lowering`

The bulk family controls planning and resume vocabulary only. It does not
change what a committed truth artifact is.

### Frozen Source And Basis Rule

Deterministic planning starts before chunking.

For ingest:

- the store must freeze a `FrozenBulkSourceManifest`
- the manifest must declare the exact admitted source members and their
  canonical enumeration order
- manifest construction must be streaming, externally partitioned, or otherwise
  bounded by declared planning memory contracts; whole-source in-memory
  canonicalization under one `plan_id` is out of spec
- retries may reread those members, but they may not rediscover a different set
  or different ordering under the same `plan_id`

For transform:

- the store must freeze a `FrozenTransformBasis`
- the store must also freeze `FrozenTransformTargetPartition` membership for the
  chunks admitted under that basis
- the basis must identify the exact target branch and frontier the transform
  was planned against
- partition membership must be decided during planning rather than rediscovered
  from the full target frontier during each chunk execution
- retries may continue from that basis, but they may not silently rebase to a
  newer frontier under the same `plan_id`

If either condition fails, the implementation must reject with a typed
determinism or drift failure rather than improvising a new plan.

### Canonical Lowering Rule

Bulk programs may stage, validate, normalize, or partition work however they
want internally, but truth becomes durable only when a chunk lowers into the
same canonical commit boundary ordinary writes use.

That means:

- ingest chunks lower into canonical appendable commits
- transform chunks lower into canonical branch-visible commits
- branch heads advance only through ordinary canonical branch update rules
- replay, export, rebuild, and recovery can explain bulk history entirely
  through canonical commit artifacts plus bulk support evidence

### Canonical Chunk Witness Rule

Every committed bulk chunk must produce a `BulkChunkCommitWitness` derivable
from canonical committed history.

At minimum the witness must preserve:

- `program_id`
- `plan_id`
- `chunk_ordinal`
- `chunk_truth_frontier`
- `canonical_commit_identity`

Resume planning may use published checkpoints as the fast path, but it must be
able to reconstruct completed chunk ordinals from canonical chunk witnesses
alone when checkpoints are missing or damaged.

### Progress Checkpoint Classification

Progress checkpoints are support artifacts for execution continuity.

They must never be treated as:

- authoritative truth
- chunk-local shadow commits
- permission to skip canonical replay verification
- permission to trust mutable host-local staging residue

Deleting or rebuilding progress checkpoints may hurt throughput, but it must
not change final recoverable truth.

## Chunk Model Contract With Milestone 6

Milestone 6 and Milestone 9 are concurrent but not merged.

Milestone 6 owns the physical honesty of chunk materialization:

- aspect-aware physical grouping
- structural block identity
- dedup-admitted chunk reuse
- backend-local decode breadth

Milestone 9 owns the operational honesty of chunk execution:

- deterministic chunk planning for bulk programs
- chunk-order commitments
- bounded-memory admission
- progress checkpoint semantics
- resume parity

The contract between them must therefore be narrow and explicit.

Milestone 9 may depend only on stable surfaces such as:

- `CanonicalChunkPlan`
- `ChunkOrdinal`
- `ChunkWidthBudget`
- `ChunkMaterializationReceipt`
- `ChunkTruthFrontier`

Milestone 9 may not depend on:

- structural block member ordering
- content-address table internals
- aspect-page packing shape
- backend-specific compression or dedup heuristics

Concurrency rule:

- Milestone 9 can begin while Milestone 6 is still in flight, but only once the
  chunk contract above is stable enough that the same admitted source and plan
  identity always produce the same chunk ordinals and chunk truth frontiers
  regardless of physical packing details

If Milestone 6 changes physical storage without changing the chunk contract,
Milestone 9 must not need a spec rewrite.

## Performance Architecture Rules

Milestone 9 must encode performance into architecture, not leave it as an
after-the-fact benchmark concern.

### Admission Before Construction

The system must reject oversized or unsupported work before expensive chunk
materialization begins.

That means:

- raw source discovery must lower into a cheap planning surface first
- plan-time budget admission must happen before building chunk payloads
- a chunk that does not satisfy declared width or memory budgets must fail as
  `BulkChunkWidthBudgetExceeded` before full lower-and-buffer work begins

Suggested enforcement shape:

- `UnadmittedChunkPlan` cannot execute
- only `BudgetAdmittedChunkPlan` may produce `ChunkMaterializationReceipt`
- only `FrozenBulkSourceManifest` or `FrozenTransformTargetPartition` may feed
  executable chunk plans

### One Planned Pass, One Execution Pass

Bulk architecture should prevent repeated rediscovery.

That means:

- canonical source ordering and transform basis partitioning must be decided
  once during planning
- execution consumes pre-decided chunk ordinals rather than re-partitioning on
  the fly
- transform execution may not rediscover target membership by rescanning the
  full branch frontier for each chunk
- resume planning reconstructs position from chunk witnesses and checkpoints
  instead of rescanning raw source to infer prior chunk boundaries

### Resume Lookup Must Be Indexed

Resume planning must not devolve into replaying or scanning all prior bulk work
just to determine the next chunk ordinal.

Milestone 9 should require a per-program witness lookup surface that can answer
at minimum:

- highest committed chunk ordinal for `program_id` plus `plan_id`
- canonical frontier of that committed ordinal
- whether there is a witness gap or duplicate ordinal

Suggested enforcement shape:

- `ProgramChunkWitnessIndex`
- `ResumeBoundaryCandidate`

The point is not the exact names. The point is that resume lookup should be an
architected responsibility, not an ad hoc scan over arbitrary history.

### Chunk Receipts Must Carry Cost Evidence

`ChunkMaterializationReceipt` must be a cost-facing bridge type, not just an
opaque success token.

At minimum it should preserve:

- chunk ordinal
- admitted width units
- decode or materialization breadth
- memory units reserved or consumed within the admitted budget class
- whether the chunk used a verified fast path or an explicit fallback class

This lets later certification assert cost truth from architecture-native
artifacts instead of log scraping.

### Verified Fast Paths And Explicit Debt Paths

Milestone 9 should not blur efficient and fallback execution into one surface.

Every admitted execution path should be classified as one of:

- `VerifiedFastPath`
- `ExplicitFallbackPath`

Fallbacks are allowed only when:

- they are typed and observable
- their broader cost surface is exposed in counters and receipts
- they do not change canonical truth or chunk determinism

### Cost-Honest Public Surfaces

Public bulk results should surface the work performed, not only whether the run
 succeeded.

At minimum, bulk-facing results and evidence bundles should expose:

- chunk count executed
- chunk width units
- checkpoint rebuild count
- witness rebuild count
- peak in-flight memory class
- fallback-path count

If a caller cannot tell whether a run stayed on the verified path or fell back
to a broader one, the API is concealing real operational cost.

## Resume, Recovery, And Progress Checkpoint Rules

### Resume Boundary

The only admissible resume boundary is the last durably published completed
chunk frontier for a declared `program_id` and `plan_id`.

Resume must not trust:

- in-memory iterators
- partial per-record staging output
- backend-local temp files
- allocator-dependent buffered state

### Checkpoint Contents

Each progress checkpoint must declare at minimum:

- `program_id`
- `plan_id`
- `completed_chunk_ordinal`
- `next_chunk_cursor`
- `chunk_truth_frontier`
- `checkpoint_version`
- `checkpoint_digest`
- `last_committed_chunk_witness`

Optional diagnostics may be persisted, but they remain observational and may
not affect resume meaning.

### Crash And Recovery Rules

- a chunk that has not crossed the canonical durable commit boundary is treated
  as not committed
- a chunk that crossed the canonical durable commit boundary but whose progress
  checkpoint was not yet published must still be discoverable from canonical
  truth during resume planning
- recovery may rebuild missing progress conclusions from canonical commit
  history for the affected program, but it must not guess a new chunk plan
- transform resume must revalidate the `FrozenTransformBasis`; if the target
  branch frontier drifted, recovery must fail typed or require an explicit new
  program identity instead of silently rebasing
- WAL recovery remains the mechanism for durable-mode crash exactness; bulk
  logic may not create a second crash recovery lane

### Rebuild Rule

Bulk support artifacts must be rebuildable or safely discardable from:

- original admitted source identity
- deterministic plan identity
- canonical committed history
- persisted progress checkpoints that did publish successfully

If a progress checkpoint family is corrupted or missing, the store must either:

- rebuild the checkpoint conclusion from canonical committed bulk history, or
- fail explicitly and typed if the admitted source identity is unavailable

It may not silently continue from an ambiguous boundary.

## Failure Taxonomy

Milestone 9 must ship an explicit typed error family matrix at minimum
covering:

- `BulkProgramVersionUnsupported`
- `BulkSourceIdentityUnavailable`
- `BulkPlanDeterminismViolation`
- `BulkSourceDriftDetected`
- `BulkTransformBasisDrift`
- `BulkChunkContractUnsupported`
- `BulkChunkWidthBudgetExceeded`
- `BulkCheckpointDigestMismatch`
- `BulkCheckpointPublicationGap`
- `BulkResumeBoundaryAmbiguous`
- `BulkChunkWitnessGap`
- `BulkChunkDuplicateCommit`
- `BulkFallbackPathRequired`
- `BulkCanonicalLoweringViolation`
- `BulkWalParityViolation`
- `BulkTransformTargetIllegal`
- `BulkTruthParityViolation`

Rules:

- planning, chunk admission, execution, checkpoint publication, resume, and
  parity verification must map failures into these families or explicit
  refinements of them
- backend-driver and file-format failures must not leak as the public semantic
  error taxonomy
- typed failures must be stable enough for certification bundles and later
  operator-facing diagnostics

## Complexity Contracts

Milestone 9 must name the admitted cost basis for bulk planning, execution, and
resume explicitly.

Minimum contracts:

- bulk planning cost is proportional to:
  - admitted source partition count
  - frozen source manifest members or transform-basis partitions
  - deterministic chunk boundaries emitted
  - normalization breadth required before canonical lowering
  - manifest or partition index records emitted for later resume lookup
- bulk execution cost is proportional to:
  - chunk width admitted
  - canonical commits emitted
  - chunk materialization breadth returned by the chunk receipt
  - fallback-path breadth when verified fast paths were not admitted
- bulk resume cost is proportional to:
  - published checkpoint count inspected
  - canonical commit history breadth needed to restate the completed frontier
  - remaining chunk count after the resume boundary

Minimum counters:

- `bulk_program_plan_count`
- `bulk_source_manifest_member_count`
- `bulk_source_manifest_stream_pass_count`
- `bulk_transform_partition_count`
- `bulk_chunk_plan_count`
- `bulk_chunk_execute_count`
- `bulk_chunk_resume_count`
- `bulk_chunk_width_units`
- `bulk_completed_checkpoint_count`
- `bulk_rebuilt_checkpoint_count`
- `bulk_chunk_witness_rebuild_count`
- `bulk_resume_index_lookup_count`
- `bulk_resume_index_scan_units`
- `bulk_peak_in_flight_memory_units`
- `bulk_fallback_path_count`
- `bulk_fallback_breadth_units`
- `bulk_canonical_commit_count`
- `bulk_wal_recovery_resume_count`
- `bulk_transform_basis_drift_count`
- `bulk_truth_parity_failure_count`
- `bulk_chunk_determinism_failure_count`

Milestone 9 may add richer counters, but it may not hide chunk width, resume
breadth, or peak in-flight memory behind throughput-only metrics.

## Phases

### Phase 1: Freeze Bulk Program Identity, Chunking, And Non-Authority Boundaries

Phase 1 defines what a resumable bulk program is allowed to mean before any
real execution begins.

Required work:

- define bulk program families, plan identity, and source identity vocabulary
- define frozen source-manifest and frozen transform-basis witness types
- define frozen transform-target partition types and resume-witness index types
- define the canonical lowering rule from bulk chunks into ordinary commits
- define canonical chunk commit witness requirements
- define the support-artifact classification for progress checkpoints
- define the stable chunk contract Milestone 9 is allowed to consume from
  concurrent Milestone 6
- define proof-bearing planning, budget admission, chunk witness, and checkpoint
  identity types

Exit condition:

- a bulk program has one exact identity and one exact plan identity
- source membership/order and transform basis are locked before execution
- progress checkpoints cannot be confused with truth authority
- chunk meaning is stable enough to continue Milestone 6 and Milestone 9 in
  parallel

### Phase 2: Build Deterministic Planning And Progress Checkpoint Surfaces

Phase 2 makes bulk planning and checkpoint publication machine-checkable.

Required work:

- implement deterministic chunk planning for ingest and transform families
- implement source-manifest freezing and transform-basis locking
- implement streaming or equivalently bounded manifest construction
- implement transform target partition freezing during planning
- implement plan-time budget admission before chunk construction
- implement progress checkpoint persistence and digest verification
- implement canonical chunk witness publication and duplicate/gap detection
- implement per-program resume witness indexing
- implement explicit resume-boundary selection and checkpoint inspection
- expose typed planning, checkpoint, and chunk-contract failures
- emit exact planning and checkpoint counters

Exit condition:

- the same admitted source and options always produce the same plan and chunk
  ordinals
- a committed chunk can always be restated as one exact chunk witness
- source freezing does not require whole-source in-memory canonicalization
- transform execution no longer widens to full-frontier rediscovery per chunk
- budget admission happens before expensive chunk construction
- checkpoints can be persisted, fetched, and verified as explicit support
  artifacts
- resume meaning is no longer ambient or host-local

### Phase 3: Execute Bulk Chunks Through Canonical Commit And WAL Boundaries

Phase 3 turns bulk planning into real durable execution without opening a
special append path.

Required work:

- reject chunk execution before expensive construction when the chunk exceeds
  declared memory budget
- execute ingest chunks through canonical append in durable and admitted
  embedded flows
- execute transform chunks through canonical branch-visible commit paths
- ensure durable-mode chunk execution remains WAL-safe
- emit cost-carrying chunk receipts with fast-path versus fallback-path
  classification
- expose typed lowering, target-legality, and WAL-parity failures
- emit exact chunk execution, commit, and in-flight memory counters

Exit condition:

- bulk execution produces only ordinary canonical commit history
- budget violations reject before oversized in-flight chunk construction
- fast-path and fallback-path execution remain mechanically distinguishable
- durable-mode crash semantics remain the same as ordinary writes
- chunk work is bounded and observable instead of throughput-opaque

### Phase 4: Resume, Recover, And Rebuild Bulk Programs Deterministically

Phase 4 makes interruption, restart, and checkpoint damage survivable.

Required work:

- implement resume planning from progress checkpoints plus canonical truth
- implement recovery handling for checkpoint publication gaps and checkpoint
  corruption
- implement canonical chunk witness recovery when checkpoints are missing
- implement indexed resume-boundary lookup from chunk witnesses
- implement typed transform-basis drift revalidation at resume time
- rebuild completed-frontier conclusions from canonical bulk history when
  admitted
- prove restart and WAL recovery remain parity-safe for interrupted runs
- emit exact resume and checkpoint-rebuild counters

Exit condition:

- interrupted runs can continue from one declared boundary
- committed chunks remain discoverable even when fast-path checkpoints are gone
- resume lookup remains bounded and does not require whole-history rescans
- corrupted or missing checkpoint artifacts do not create ambiguous resume
  meaning
- restart does not change final truth or branch-head meaning

### Phase 5: Certify Bulk Resume Parity And Boundedness

Phase 5 turns bulk execution into a certifiable operational surface rather than
an optimistic batch helper.

Required work:

- run the Milestone 9 named suite:
  `Bulk Ingest And Transform Resume Parity Test`
- compare interrupted-and-resumed lanes against logically serial control lanes
- compare WAL-recovered lanes against clean uninterrupted lanes
- emit machine-checkable truth, history, restore, and counter bundles

Exit condition:

- final truth matches the control lane
- chunk boundaries remain deterministic across interruption and recovery
- WAL recovery remains parity-safe for interrupted runs
- Milestone 9 closeout evidence exists in machine-checkable form

Current implementation note:

- the named Milestone 9 suite now exists in
  `crates/worth-store/src/tests/milestone_9_certification.rs`
- the machine-checkable certification bundle now lives in
  `crates/worth-store/src/evidence/milestone_9.rs`
- adversarial reopen and restart-loop coverage in `tests/bulk.rs` and
  `tests/wal_recovery.rs` remains the supporting hostile evidence behind the
  named closeout lane

## Must Ship

- deterministic `BulkIngestProgram` and `BulkTransformProgram` planning surfaces
- explicit bulk plan identity and source identity vocabulary
- resumable progress checkpoint artifacts with typed verification failures
- compile-time-distinct proof types for frozen sources, transform basis locks,
  frozen transform target partitions, budget-admitted chunk plans, canonical
  chunk witnesses, and resume-boundary candidates
- bounded-memory chunk execution through canonical commit boundaries
- cost-carrying chunk materialization receipts and bulk result surfaces
- durable-mode WAL-safe bulk execution
- deterministic resume planning and checkpoint-rebuild rules
- exact bulk counters and machine-checkable Milestone 9 certification output

## Must Preserve

- canonical commits remain the only semantic durable authority
- bulk paths do not invent a second commit, replay, or branch-head model
- progress checkpoints remain support artifacts rather than truth
- source membership/order and transform basis stay frozen under one plan identity
- source freezing and transform partitioning remain bounded planning steps
- chunk determinism remains stable across interruption, restart, and backend
  variation
- verified fast paths remain distinct from explicit fallback paths
- Milestone 6 physical layout internals remain replaceable behind the stable
  chunk contract
- deleting bulk support artifacts never changes committed recoverable truth

## Acceptance Evidence

Milestone 9 is complete only when the store satisfies the named Milestone 9
suite:

- `Bulk Ingest And Transform Resume Parity Test`

Required machine-checkable outputs:

- `truth_digest`
- `history_digest`
- `restore_digest`
- `counter_snapshot`

Implementation note:

- the closeout evidence and explicit obligation-to-test mapping for this
  section is recorded in
  [milestone-9-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-9-closeout.md)

Milestone-specific proof obligations:

- interrupted ingest reaches the same final truth as the logically serial
  control lane
- interrupted transform reaches the same final truth as the logically serial
  control lane
- chunk boundaries remain deterministic for the same admitted source and plan
- missing checkpoints can be recovered from canonical chunk witnesses without
  duplicating chunk execution
- transform basis drift fails explicitly instead of silently rebasing
- resume lookup remains bounded through explicit witness indexing
- WAL recovery plus resume reaches the same canonical history-visible outcome as
  uninterrupted bulk execution
- fast-path versus fallback-path usage remains visible in receipts and counters
- bounded-memory claims remain machine-checkable through exact counters

Milestone 9 is not closed by "the import finished eventually" tests.

## Architectural Notes

- The smart abstraction is not "background job." The smart abstraction is one
  deterministic bulk program contract with support-artifact checkpoints and
  canonical commit lowering.
- Planning, execution, checkpoint publication, resume planning, and parity
  evidence should be separate subdomains even if the first implementation keeps
  them close.
- Milestone 6 may continue changing physical chunk layout underneath the stable
  chunk contract; Milestone 9 should treat chunk materialization receipts as
  bridge artifacts, not semantic truth.
- Bulk profitability, adaptive scheduling, or cluster orchestration later must
  inherit these determinism and support-artifact rules instead of renegotiating
  what a resumable bulk run means.
- Retention, replication, and certification later may consume bulk evidence, but
  only because Milestone 9 already keeps bulk truth inside the canonical commit
  model.

## Sequencing Notes

This milestone belongs after Milestone 5 because bulk work needs an already
honest branch-delta and derived-physical-work boundary before it can claim
resume parity.

It is intentionally concurrent with Milestone 6, not blocked on Milestone 6
fully closing.

What Milestone 9 needs from Milestone 6 is narrower:

- a stable chunk identity contract
- stable chunk width vocabulary
- stable chunk materialization receipts that do not leak backend-local layout

What Milestone 9 must not wait for:

- every aspect-aware optimization path
- every structural dedup acceleration path
- final tuning of physical decode breadth

This milestone therefore belongs in the roadmap as an overlap program:

- it starts after the chunk contract is honest enough for canonical chunking
- it runs concurrently with late Milestone 6 work
- it should close before later retention, replication, and certification work
  begin depending on bulk operational capability as a real platform surface
