# Milestone 3 Engineering Spec: WAL-Coordinated Durable Mode And Crash Recovery

> **Status:** Closed 2026-04-14
>
> **Roadmap parent:** `worth_store_roadmap.md`
>
> **Vision parent:** [worth_store_vision.md](./worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-1.md](./milestone-1.md)
> - [milestone-1-closeout.md](./milestone-1-closeout.md)
> - [milestone-2.md](./milestone-2.md)
> - [milestone-2-closeout.md](./milestone-2-closeout.md)
>
> **Closeout:** [milestone-3-closeout.md](./milestone-3-closeout.md)
>
> **Primary architectural driver:** make durable mode real without letting the
> WAL, recovery path, or hosted runtime lifecycle become a second authority

## Goal

Make durable mode real: every acknowledged durable-mode commit survives process
failure and recovers to the same committed truth.

## Why This Milestone Exists

Milestone 3 is not "add a journal."

It is the milestone that decides whether `worth-store` can honestly claim to
be a database in durable mode instead of a runtime wrapper with best-effort
persistence.

Milestone 1 locked the authoritative commit artifact model.

Milestone 2 locked who may host the runtime and who may persist external
artifacts.

Milestone 3 now has to lock the crash boundary itself:

- what exactly becomes durable before acknowledgment
- what exactly may be reconstructed after crash
- what exactly the WAL is allowed to prove
- what exactly still has to be derived from canonical authoritative artifacts

If this milestone is weak, later snapshots, delta layering, compaction,
replication, and live-query continuation will all end up depending on a fuzzy
recovery story. That is exactly the failure this milestone exists to prevent.

## Hard Part

The hard part is not appending bytes to a log.

The hard part is keeping three things sharply separate while still making them
compose:

- runtime commit legality and canonical commit production
- WAL durability and acknowledgment safety
- authoritative recovery conclusions after crash

The design fails if the WAL becomes semantic authority, if acknowledged durable
truth can be lost after crash, or if recovery replays backend-local artifacts
that cannot be expressed back through the canonical Milestone 1 authority
model.

## Explicit Assumptions

- Milestone 1 canonical authoritative artifact families remain the only truth
  authority.
- Milestone 2 durable mode remains the only operating mode covered here.
- `worth-relational` still owns transaction semantics, commit legality, replay
  meaning, and canonical commit-envelope production.
- the WAL is an authoritative durability aid for acknowledged durable-mode
  publication, but it is not itself the only semantic truth surface.
- crash recovery and full rebuild are distinct recovery modes and both must
  remain first-class.
- snapshots, delta layering, replication capsules, and compaction are still out
  of scope for this milestone.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is adversarial honesty under pressure.
  Milestone 3 therefore starts from the crash boundary and treats exact
  recovery conclusions as the product, not as backend implementation detail.
- `arch_laws.md`
  The most important thing it protects here is proof-bearing authority
  sequencing. Law 41 matters especially: raw hosted-runtime output, WAL-admitted
  mutation intent, durable acknowledgment eligibility, recovered authoritative
  truth, and rebuilt authoritative truth must all be distinct types with sealed
  transitions. Law 36 also matters: checkpoint or journal recovery may replay
  work, but recovery conclusions must still be reconstructible from canonical
  authority rather than opaque process residue.
- `perf_laws.md`
  The most important thing it protects is truthful cost. WAL append, recovery
  scan, replay breadth, duplicate suppression, and acknowledgment-path work
  must have named complexity bases and exact counters now rather than being
  waved away as "later optimization."
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped decomposition.
  Milestone 3 must separate WAL record authority, durable-mode lifecycle,
  recovery planning, replay execution, and certification evidence rather than
  burying them in one persistence helper.
- `worth_store_vision.md`
  The most important thing it protects is that `worth-store` owns survival
  while `worth-relational` owns semantics. Milestone 3 must therefore host the
  runtime in durable mode without letting the store redefine commit legality or
  replay meaning.
- `worth_store_roadmap.md`
  The most important thing it protects is order. Milestone 3 belongs exactly
  here because snapshots, delta layering, and later recovery accelerators are
  dishonest unless the durable crash boundary is already frozen.
- `worth-store/test-requirements.md`
  The most important thing it protects is exact crash-boundary proof.
  Milestone 3 is not closed until the `WAL Crash Boundary Exactness Test`
  proves acknowledged survival, unacknowledged non-publication, and recovery
  parity with rebuild.
- `milestone-1.md`
  The most important thing it protects is one canonical durable truth model.
  Milestone 3 may widen durable mechanics, but it must still recover and
  publish only Milestone 1 authoritative artifact families.
- `milestone-1-closeout.md`
  The most important thing it protects is that authoritative append, fetch,
  export, and rebuild are already real and certified. Milestone 3 must build on
  that substrate rather than inventing a parallel recovery-only truth path.
- `milestone-2.md`
  The most important thing it protects is operating-mode ownership.
  Milestone 3 must deepen durable mode only; it may not blur embedded mode,
  absent mode, or checkpoint reception into the same lifecycle contract.
- `milestone-2-closeout.md`
  The most important thing it protects is the proof-bearing mode boundary.
  Durable hosted-runtime ownership is already explicit, so WAL and recovery
  must reuse that ownership proof rather than creating ambient durable helpers.
- `worth_relational_vision.md`
  The most important thing it protects is that replay and truth meaning come
  from canonical runtime commit artifacts, not storage-local reconstruction.
  Milestone 3 must therefore recover by replaying canonical commit meaning,
  not by replaying arbitrary backend write effects.
- `worth_relational_roadmap.md`
  The most important thing it protects is serialized truth publication with
  deterministic replay from canonical commit artifacts. The store WAL must
  preserve that serialized authority instead of rediscovering truth semantics
  during restart.
- `worth_runtime_bridge_vision.md`
  The most important thing it protects is clean runtime ownership boundaries.
  Durable mode may host a runtime, but store recovery still must not leak
  storage-local semantics upward into future bridge or signal consumers.
- `worth_runtime_bridge_roadmap.md`
  The most important thing it protects is that downstream integration consumes
  canonical committed artifacts with replay-safe diagnostics. Milestone 3 must
  keep recovered truth explainable in those same terms.

## Adversarial Constraint

Milestone 3 must survive this hostile condition:

> A crash at any point around the durable commit boundary must not duplicate,
> lose, or partially publish acknowledged truth, and recovery must converge to
> the same committed truth whether it starts from WAL replay or full rebuild
> from canonical authoritative artifacts.

## Product Decision Lock

The following decisions are locked in this milestone:

- durable mode acknowledges success only after the WAL and the authoritative
  publication boundary have both crossed the admitted durability threshold for
  this milestone
- the WAL is append-only and crash-oriented, not a second semantic history
  language
- recovery may consult WAL records, authoritative artifacts, and typed durable
  recovery metadata, but final truth conclusions are always expressed back
  through Milestone 1 authoritative artifact families
- full rebuild from canonical authoritative artifacts remains first-class even
  after WAL recovery ships
- durable-mode hosted runtime restart is explicit and typed; recovery is not an
  ambient side effect of opening a store
- unacknowledged work may be replayed, discarded, or require typed operator
  intervention according to its proof state, but it may not silently publish as
  committed truth

Normative consequence:

- any implementation that treats "record exists in WAL" as equivalent to
  "commit is authoritative truth" is out of spec
- any implementation that can acknowledge a durable-mode commit before the WAL
  and authoritative append unit reach the declared durability threshold is out
  of spec
- any implementation that recovers to backend-local write effects that cannot
  be expressed as canonical authoritative artifacts is out of spec

## Scope

### In Scope

- store-owned hosted runtime lifecycle for durable mode
- append-only WAL for durable-mode commit boundary protection
- typed WAL record families for mutation intent, canonical commit production,
  publication progress, and recovery explanation
- "log before acknowledge" durable-mode contract
- crash recovery from WAL plus canonical authoritative artifacts
- typed distinction between crash recovery and full rebuild recovery
- duplicate-suppression and non-publication rules around crash restart
- durable recovery diagnostics, counters, and machine-checkable certification
  bundles
- backend support sufficient to persist authoritative artifacts together with
  WAL and durable recovery metadata
- explicit durability and publication ordering rules between WAL append,
  canonical authoritative append, and hosted runtime restart
- schema reservations needed for later snapshots and delta-layering recovery
  accelerators, as long as those accelerators do not ship yet

### Explicitly Out Of Scope

- snapshot capture and snapshot-plus-tail restore
- branch delta layering and delta-stack rewrite policy
- compaction and WAL/archive retention policy beyond the minimum needed to keep
  crash recovery honest
- replication, capsule export/import, and offline integrity-audit import
- embedded-mode checkpoint durability changes
- live-query continuation
- bulk-ingest resumability beyond what is needed for one durable commit path
- advanced recovery acceleration structures that bypass ordinary WAL scan and
  authoritative replay

## Durable-Mode Commit And Recovery Model

### Durable-Mode Truth Rule

Durable mode does not create a new truth path.

Durable mode hosts a `worth-relational` runtime instance inside the store and
adds a crash-safe acknowledgment boundary around the existing canonical commit
artifact model.

The truth rule is therefore:

`store admits durable mutation intent into the WAL -> hosted runtime executes the admitted mutation and produces canonical commit meaning -> store records durable publication progress -> store persists canonical authoritative artifacts -> store acknowledges`

The durable-mode contract succeeds only when the resulting committed truth can
still be expressed as:

- canonical commit envelopes
- ordered parent records
- branch records and branch heads
- authoritative artifact digests

The durable-mode contract fails if any durable-only metadata becomes necessary
to explain what committed.

### Durable Mutation Identity Rule

Milestone 3 must define one canonical `DurableMutationId`.

This identity exists to anchor:

- WAL intent admission
- publication progress
- duplicate suppression across restart loops
- retry and resubmission diagnostics

Required rules:

- `DurableMutationId` is assigned before hosted-runtime execution begins
- every WAL family in this milestone that refers to one durable mutation must
  reference the same `DurableMutationId`
- `DurableMutationId` is not semantic commit identity; one durable mutation may
  fail to produce any committed truth, and a retried logical request may
  legitimately produce a different `DurableMutationId`
- duplicate suppression during restart is keyed first by `DurableMutationId`
  and then checked against the authoritative commit identity where available
- client or caller resubmission semantics must remain explicit:
  - retrying the same unfinished durable mutation may reuse its
    `DurableMutationId` only through an admitted typed retry path
  - an ordinary new submission gets a new `DurableMutationId`

This is the line that keeps duplicate suppression and semantic commit identity
from collapsing into one overloaded token.

### WAL Role And Non-Authority Rule

The WAL exists to preserve crash-boundary exactness, not to redefine meaning.

The WAL is allowed to prove:

- what durable-mode work was admitted for processing
- what canonical commit artifact identity was produced by the hosted runtime
- what publication step had or had not crossed a durable threshold at crash
  time
- what recovery action is admissible on restart

The WAL is not allowed to become the sole semantic source for:

- branch ancestry
- ordered parent meaning
- branch-head authority
- commit payload meaning
- replay semantics

Normative rule:

- recovery may begin from WAL records
- recovery conclusions must end in canonical authoritative artifacts or typed
  explicit failure
- WAL-only conclusions that cannot be restated through canonical authoritative
  artifacts are forbidden

This is the anti-shadow-authority line for Milestone 3.

### Durable Acknowledgment Boundary

Milestone 3 must define one precise durable acknowledgment boundary.

Required rule:

- a durable-mode mutation may be acknowledged only after:
  - the admitted WAL intent and its required follow-on WAL records for this
    milestone have been durably appended according to the backend contract
  - the canonical authoritative append unit defined in Milestone 1 has been
    durably persisted
  - the publication state is recoverable to the same acknowledged conclusion
    after immediate crash

Milestone 3 must not leave open whether acknowledgment means:

- "intent reached the log"
- "commit envelope was computed"
- "authoritative records were stored"

For this milestone, the answer is:

- acknowledgment means durable WAL evidence and durable authoritative
  publication are both complete enough that crash recovery will still publish
  exactly the acknowledged commit set and no more

Anything weaker is not durable mode.

### Crash Boundary Exactness Rule

Milestone 3 must encode exact outcomes for every crash position around one
durable commit.

Minimum crash classes:

- crash before WAL intent durability
- crash after WAL intent durability but before hosted runtime canonical commit
  result durability
- crash after hosted runtime canonical commit result durability but before
  authoritative append durability
- crash after authoritative append durability but before acknowledgment
- crash after acknowledgment

Required exactness:

- acknowledged commits survive exactly once
- unacknowledged commits may remain unpublished or be replayed to a typed
  admissible conclusion, but they may not surface as partially published truth
- duplicate publication after restart is forbidden even if the same WAL record
  is encountered multiple times across repeated crash-restart loops
- branch-head publication must remain synchronized with commit publication;
  "commit visible but head missing" and "head moved without authoritative
  commit publication" are forbidden states

This rule must be testable mechanically, not described narratively.

### Crash-Class Outcome Matrix

Milestone 3 must make each admitted crash class resolve to one exact recovery
outcome class.

| Crash class | Minimum durable evidence present at crash | Required recovery outcome |
| --- | --- | --- |
| before WAL intent durability | no durable mutation intent | treat as never admitted; no publication |
| after WAL intent durability, before hosted runtime canonical result durability | admitted durable mutation identity only | either typed discard as uncommitted or typed replay from admitted replay basis; no publication may occur without finishing the full durable publication path |
| after hosted runtime canonical result durability, before authoritative append durability | admitted durable mutation identity plus durable canonical result record | restart may replay from the recorded canonical result or restart hosted execution only if the spec-admitted replay basis still holds; partial publication is forbidden |
| after authoritative append durability, before acknowledgment | authoritative publication complete but acknowledgment not yet crossed | recovery must retain the published authoritative truth and suppress duplicate replay; it must not require the commit to vanish merely because acknowledgment was absent |
| after acknowledgment | authoritative publication plus acknowledgment eligibility crossed | recovery must retain the published authoritative truth exactly once |

Normative consequence:

- Milestone 3 may not leave any listed crash class as "implementation choice"
- if a backend cannot distinguish one of these classes durably, it is not an
  admitted Milestone 3 backend for durable mode

### Recovery Modes

Milestone 3 admits exactly these recovery modes:

- `CrashRecovery`
  Restart from durable-mode WAL records plus already-persisted authoritative
  artifacts after process failure.
- `FullAuthoritativeRebuild`
  Ignore WAL-driven fast recovery conclusions and rebuild from canonical
  authoritative artifacts only.

Milestone 3 may also admit an internal diagnostic comparison mode that runs both
paths for certification, but it must not expose an operator story where "we are
not sure which recovery path is true."

Rules:

- crash recovery is allowed to be faster than full rebuild
- crash recovery is not allowed to produce stronger semantic conclusions than
  full rebuild
- full rebuild remains the semantic control lane for certification
- later milestones may add snapshot-based restore or integrity-audit rebuild,
  but this milestone must not pretend those exist yet

### Admitted Replay Basis Rule

Milestone 3 must define exactly what recovery is replaying for unfinished work.

Admitted replay bases in this milestone:

- `DurableMutationIntentRecord`
  as the request-side basis for typed discard or typed re-execution admission
- `HostedRuntimeCommitResultRecord`
  as the canonical-result-side basis for finishing durable publication when the
  hosted runtime has already produced canonical commit meaning

Not admitted as replay bases:

- raw in-memory hosted-runtime objects from the crashed process
- backend-local partially written rows that are not themselves declared WAL or
  authoritative artifact families
- ambient host callbacks or side effects that are not represented by admitted
  durable artifacts

Required rule:

- every recovery action must name which admitted replay basis justified it
- if no admitted replay basis exists for unfinished work, recovery must fail or
  discard explicitly rather than guessing

## Proof-Carrying Durable Pipeline

Law 41 is load-bearing here. Durable mode must encode the crash boundary as a
proof chain, not as loosely coordinated helpers.

Representative progression:

```rust
pub struct HostedRuntimeMutationRequest { ... }
pub struct WalAdmittedMutationIntent { ... }
pub struct HostedRuntimeCanonicalCommitResult { ... }
pub struct WalRecordedCanonicalCommit { ... }
pub struct DurablyPublishedCommitBoundary { ... }
pub struct AcknowledgedDurableCommit { ... }
pub struct CrashRecoveredAuthoritativeCommit { ... }
pub struct RebuiltAuthoritativeCommitSet { ... }
```

Required meaning:

- `HostedRuntimeMutationRequest`
  proves only that durable-mode hosted execution was requested through the
  admitted durable handle
- `WalAdmittedMutationIntent`
  proves the request has crossed the append-only WAL admission boundary
- `HostedRuntimeCanonicalCommitResult`
  proves the hosted runtime produced canonical commit artifacts, but not yet
  that those artifacts are durably recoverable
- `WalRecordedCanonicalCommit`
  proves the hosted runtime result and publication progress needed for restart
  have durably entered the WAL
- `DurablyPublishedCommitBoundary`
  proves the Milestone 1 authoritative append unit is durably published for the
  commit
- `AcknowledgedDurableCommit`
  proves the commit crossed the exact durable acknowledgment boundary
- `CrashRecoveredAuthoritativeCommit`
  proves restart-specific recovery accepted the commit as published truth
- `RebuiltAuthoritativeCommitSet`
  proves the full authoritative rebuild lane reached its own truth conclusion

Mandatory Law 41 consequences:

- only the durable-mode subsystem may mint the WAL and acknowledgment proof
  types
- restart code may not mint `AcknowledgedDurableCommit` directly; it must pass
  through typed recovery verification
- backend row decoding may not skip the same recovery-verification gateway used
  in ordinary crash recovery
- tests may only bypass phases inside isolated fixture modules
- no public helper may accept a raw hosted-runtime result where a stronger
  durable proof type already exists

## WAL Artifact Model

### Admitted WAL Artifact Families

Milestone 3 must define one store-owned WAL artifact taxonomy.

Required families:

- `DurableMutationIntentRecord`
  Identifies one durable hosted-runtime mutation request and its durable-mode
  context.
- `HostedRuntimeCommitResultRecord`
  Captures the canonical commit identity and canonical authoritative artifact
  references produced by the hosted runtime.
- `DurablePublicationProgressRecord`
  Captures which admitted durable publication phase was completed:
  - intent admitted
  - canonical commit produced
  - authoritative append durably published
  - acknowledgment eligible
- `RecoveryDecisionRecord`
  Captures restart-time typed decisions when the recovery engine determines
  replay, discard, duplicate suppression, or integrity failure outcomes.

Milestone 3 must not hide durable publication state in ad hoc booleans spread
across backend tables. If the recovery engine needs the fact to make a
crash-boundary conclusion, the fact must belong to a declared WAL family or a
declared authoritative artifact family.

### WAL Record Mutability And Truncation Rules

WAL records are append-only.

Rules:

- WAL records may not be updated in place to "fix" meaning
- later progress is recorded by later WAL records, not by mutating older ones
- truncation or archival is out of scope except where a backend needs internal
  boundedness for the milestone implementation; even then, truncation may occur
  only after the records it removes are proven semantically redundant for crash
  recovery by the declared Milestone 3 rules
- recovery diagnostics must remain able to distinguish:
  - original intent
  - recorded canonical result
  - publication completion
  - restart-time decision

Milestone 3 does not need final retention policy, but it must forbid silent
WAL mutation.

### WAL Canonicalization And Digest Basis

Milestone 3 must define a canonical digest basis for WAL records too.

Required fields in the digest basis:

- WAL record family
- durable mutation identity
- referenced canonical commit identity where applicable
- durable runtime identity or hosted-session identity where applicable
- publication-phase discriminator
- version field for WAL record interpretation

Excluded from the digest basis unless explicitly declared otherwise:

- explanatory diagnostics strings
- local timing measurements
- non-authoritative backend offsets

Normative rules:

- WAL record digests must be deterministic across backend families
- future-added WAL fields must not change older WAL digest meaning silently
- WAL canonicalization versioning must be explicit from day one

### Backend Durability Capability Rule

Milestone 3 must define the minimum backend durability capability the WAL path
depends on.

Required capability:

- the backend must provide a typed durability primitive that means:
  records written before the durability barrier are recoverably present after
  process crash, and records written after it are not assumed present until a
  later declared barrier completes

The implementation may realize that primitive as:

- fsync-equivalent file durability
- transactional commit durability in an embedded database
- another backend-local mechanism with the same declared crash guarantee

But the spec-level contract must stay backend-independent:

- "written to an in-memory buffer" is not durable
- "driver reported success" is not sufficient unless it is explicitly the
  backend's declared durability barrier
- the backend adapter must expose which writes crossed the durability barrier
  that Milestone 3 acknowledgment relies on

## Hosted Runtime Lifecycle

### Durable Runtime Ownership Rule

Durable mode remains the only mode where `worth-store` owns the live runtime
instance.

Milestone 3 must therefore keep the durable hosted runtime explicit:

- explicit start
- explicit restart
- explicit recovery gating before mutation admission
- explicit shutdown or crash distinction

The store may not reopen into a durable mutation-admitting state until typed
recovery has completed or has explicitly failed.

Forbidden drift:

- ambient auto-recovery that makes the hosted runtime "just available"
- a raw backend-open path that can mutate before crash recovery completes
- embedded-mode or absent-mode handles reusing durable hosted-runtime helpers

Milestone 2 already proved ownership. Milestone 3 must now prove recovery-safe
ownership.

### Durable Runtime Start, Restart, And Recovery Admission

Required lifecycle states:

- `DurableRuntimeCold`
- `RecoveryPending`
- `RecoveryInProgress`
- `RecoveryVerified`
- `DurableRuntimeAdmittingMutations`
- `RecoveryFailed`

Rules:

- durable mutation admission is legal only from
  `DurableRuntimeAdmittingMutations`
- process restart enters `RecoveryPending`, not directly
  `DurableRuntimeAdmittingMutations`
- a failed recovery may expose diagnostics and authoritative read-only
  inspection, but it may not admit new durable-mode writes
- the lifecycle must preserve a typed distinction between:
  - no recovery needed
  - recovery completed
  - recovery failed
  - rebuild required by policy

The lifecycle proof must make "skip recovery and keep writing" impossible
through the normal API.

### Durable Hosted-Mutation Purity Rule

Any durable hosted mutation admitted to replay from `DurableMutationIntentRecord`
must be pure with respect to authoritative truth effects.

Allowed effects inside replay-admitted durable execution:

- `worth-relational` truth mutation planning and commit production
- Milestone 3 WAL and authoritative publication artifacts
- declared diagnostics and counters whose presence does not change truth meaning

Not allowed inside replay-admitted durable execution:

- external network calls
- irreversible host callbacks
- side-channel writes whose duplication would matter semantically outside the
  store
- bridge-origin or subscriber-origin outward effects that are not themselves
  represented as typed durable artifacts

Normative consequence:

- if a durable hosted mutation needs external effects, those effects must cross
  a later explicit boundary outside the replay-admitted truth path
- Milestone 3 recovery may only re-execute durable work whose effect surface is
  safe under crash replay

## Recovery And Replay Contracts

### Crash Recovery Contract

Crash recovery consumes:

- admitted WAL records
- already-persisted authoritative artifacts
- durable recovery metadata admitted by this milestone

Crash recovery must determine, for each durable mutation identity:

- no admitted work exists
- admitted work exists but never crossed a durable publication point
- authoritative publication completed and must be retained
- duplicate replay is being observed and must collapse to the already published
  authoritative result
- integrity failure or ambiguity requires typed failure

Crash recovery is allowed to replay hosted-runtime commit results only when the
necessary canonical authoritative outcome is already recoverably defined by the
admitted WAL and authoritative artifact model.

Crash recovery is not allowed to:

- guess commit legality from partial side effects
- invent a branch-head advancement not already justified by canonical
  authoritative publication
- treat missing authoritative records as implicitly published because the WAL
  "probably meant to finish"

### Full Rebuild Contract

Full rebuild consumes canonical authoritative artifacts only.

Rules:

- rebuild does not consult WAL semantic conclusions to decide what truth means
- rebuild may ignore WAL entirely except when certification compares the two
  recovery lanes diagnostically
- rebuild must reach branch heads, commit history, and replay-visible truth
  purely from Milestone 1 authoritative artifact families

This lane is what keeps WAL recovery honest.

### Recovery Equivalence Rule

Milestone 3 requires exact equivalence between:

- crash recovery conclusions for acknowledged truth
- full authoritative rebuild conclusions for acknowledged truth

Equivalence must include at minimum:

- committed history
- ordered parents
- branch heads
- replay-visible truth
- durable failure classification where a crash state is intentionally
  non-recoverable

Equivalence does not require the same counters, timings, or diagnostic detail.
It does require the same truth conclusions.

### Acknowledgment Uncertainty And Retry Rule

Milestone 3 must make caller-visible retry semantics explicit for crashes near
the acknowledgment boundary.

Required rule:

- absence of observed acknowledgment by the caller is not proof that the
  authoritative commit vanished
- a retry after acknowledgment uncertainty must enter a typed resolution path,
  not an implicit fresh-mutation path

Minimum admitted retry outcomes:

- `PreviouslyAcknowledgedEquivalentCommit`
  the store proves the uncertain prior attempt already crossed the durable
  publication boundary and resolves the retry to the existing authoritative
  commit
- `NotPreviouslyPublished`
  the store proves the uncertain prior attempt never crossed authoritative
  publication, so a fresh durable mutation may proceed
- `RetryRequiresOperatorOrHigherLevelPolicy`
  the store cannot safely collapse the retry or admit it as fresh without a
  declared higher-level policy

This rule keeps client uncertainty from turning into duplicate authority.

### Recovery Diagnostics And Failure Taxonomy

Milestone 3 must ship typed recovery failures at minimum covering:

- `WalRecordCorruption`
- `WalCanonicalizationVersionUnsupported`
- `WalDigestMismatch`
- `DurablePublicationStateGap`
- `AcknowledgmentBoundaryViolation`
- `RecoveryDuplicateSuppressionFailure`
- `RecoveryAuthoritativeArtifactMissing`
- `RecoveryBranchHeadMismatch`
- `RecoveryReplayParityViolation`
- `RecoveryRequiresFullRebuild`
- `RecoveryIntegrityFailure`
- `HostedRuntimeRestartMisuse`

Rules:

- public recovery errors must be typed in store-owned terms, not backend-driver
  jargon
- diagnostics must localize the failing durable mutation identity or artifact
  family where possible
- recovery diagnostics may be richer than rebuild diagnostics, but they may not
  disagree semantically about retained truth

## Append Atomicity And Publication Rules

Milestone 3 extends the Milestone 1 authoritative append atomicity rule into a
durable-mode publication rule.

One admitted durable commit must have one coherent publication unit covering:

- WAL intent append
- hosted-runtime canonical commit result capture
- authoritative append unit durability
- branch-head publication
- acknowledgment eligibility recording where the design needs it

Required rule:

- either restart can justify the same acknowledged commit exactly once, or it
  cannot justify it at all

Milestone 3 must explicitly forbid:

- authoritative append visible without the matching recovery state needed to
  suppress duplicate replay after crash
- acknowledgment recorded without authoritative append durability
- branch-head publication without the corresponding commit publication
- in-place repair of durable publication state to hide a broken boundary

Idempotent duplicate suppression across repeated crash-restart loops is a
required behavior, not an optimization.

## Public Surface

Milestone 3 must keep the public facade explicit and narrow.

Representative surface:

```rust
pub struct DurableStoreBuilder { ... }
pub struct DurableStoreHandle { ... }
pub struct RecoveryPlan { ... }
pub struct RecoveryOutcome { ... }

impl DurableStoreBuilder {
    pub fn build(self) -> Result<DurableStoreHandle, StoreBuildError>;
}

impl DurableStoreHandle {
    pub fn start_recovery(
        self,
    ) -> Result<RecoveryPlan, RecoveryStartError>;

    pub fn complete_recovery(
        self,
        plan: RecoveryPlan,
    ) -> Result<RecoveredDurableStoreHandle, RecoveryError>;

    pub fn commit_hosted_mutation(
        &mut self,
        request: HostedRuntimeMutationRequest,
    ) -> Result<AcknowledgedDurableCommit, DurableCommitError>;
}
```

Surface rules:

- durable mutation APIs remain on durable-mode handles only
- hosted-runtime recovery stays explicit on the public surface
- read-only inspection after failed recovery may be admitted explicitly, but it
  must not masquerade as normal durable operation
- the public surface must expose recovery in store-owned vocabulary, not raw
  WAL rows or backend handles

## Required Internal Subsystems

Milestone 3 must decompose by responsibility:

- `modes/durable/`
  hosted-runtime durable-mode lifecycle and mutation admission
- `wal/`
  WAL record families, canonicalization, append contract, and decode
- `recovery/`
  recovery planning, replay, duplicate suppression, and typed decisions
- `publication/`
  durable publication boundary and acknowledgment proofs
- `backend/`
  backend persistence support for WAL plus authoritative artifacts
- `diagnostics/`
  counters, failure-localization records, and certification bundles
- `harness/`
  crash-boundary certification and repeated crash-restart fixtures

This is the `domain_laws.md` line for Milestone 3: separate by what changes
and fails for different reasons, not by generic "storage layer" folders.

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| WAL record canonical determinism | WAL canonicalization | `wal/` | `WalDigestMismatch` or `WalCanonicalizationVersionUnsupported` | `truth_digest` and `failure_digest` |
| durable mutation admitted only through durable hosted handle | lifecycle admission | `modes/durable/` | `HostedRuntimeRestartMisuse` | mode/recovery misuse matrix |
| acknowledgment requires durable WAL plus authoritative publication | publication proof | `publication/` | `AcknowledgmentBoundaryViolation` | crash-boundary certification bundle |
| authoritative append stays branch-head coherent after crash | recovery verification | `recovery/` and `backend/` | `RecoveryBranchHeadMismatch` | `truth_digest` parity |
| duplicate replay suppression across restart loops | recovery verification | `recovery/` | `RecoveryDuplicateSuppressionFailure` | repeated crash-restart bundle |
| crash recovery and full rebuild reach equal truth conclusions | certification comparison | `recovery/` and `harness/` | `RecoveryReplayParityViolation` | `replay_digest` and `restore_digest` |
| unacknowledged work does not partially publish | publication plus recovery | `publication/` and `recovery/` | `DurablePublicationStateGap` | crash-before-ack bundle |
| recovery requires typed failure on irreconcilable damage | recovery verification | `recovery/` | `RecoveryIntegrityFailure` | `failure_digest` |

## Complexity Contracts

Milestone 3 must name the hot-path and recovery-path cost basis explicitly.

Minimum contracts:

- WAL append cost is proportional to:
  - number of WAL records emitted for one durable mutation
  - canonical WAL payload size
  - digest work for admitted WAL families
- durable commit boundary cost is proportional to:
  - hosted-runtime canonical commit artifact size
  - Milestone 1 authoritative append breadth
  - WAL record count for the mutation
- crash recovery scan cost is proportional to:
  - scanned WAL record count
  - number of incomplete or restart-relevant durable mutation identities
  - authoritative artifact verification breadth required by those identities
- full rebuild cost remains proportional to:
  - authoritative commit count
  - ordered parent breadth
  - branch-head verification breadth

Minimum counters:

- `wal_record_append_count`
- `wal_record_scan_count`
- `wal_record_decode_failure_count`
- `durable_mutation_admit_count`
- `durable_commit_acknowledged_count`
- `durable_commit_recovered_count`
- `durable_commit_duplicate_suppression_count`
- `durable_commit_unacknowledged_discard_count`
- `recovery_requires_full_rebuild_count`
- `recovery_failure_count`

Milestone 3 may add richer counters, but it may not hide the actual work basis.

## Phases

### Phase 1: Lock The Durable Commit Boundary

Phase 1 defines the exact durable-mode acknowledgment semantics and freezes the
durable state machine before backend work begins.

Required work:

- define WAL artifact families and canonicalization basis
- define the durable acknowledgment boundary exactly
- define crash classes and exact outcomes
- define proof-bearing durable pipeline types
- define durable lifecycle states around restart and recovery

Exit condition:

- one durable hosted mutation has one explicit crash-boundary state machine
- acknowledgment meaning is no longer ambiguous
- the WAL has a declared role and a declared non-authority boundary

### Phase 2: Persist Admitted WAL Families

Phase 2 makes the append-only WAL real as a backend-supported persistence
surface.

Required work:

- implement append-only WAL persistence through admitted backend families
- implement WAL canonicalization, digesting, and decode verification
- persist durable mutation identity and durable publication progress records
- expose typed WAL append and decode failures
- emit exact WAL append and decode counters

Exit condition:

- WAL records are durably persisted through the declared backend durability
  primitive
- WAL families are canonicalized and verifiable
- restart can load WAL state without yet admitting new durable writes

### Phase 3: Gate Hosted Runtime Restart And Durable Publication

Phase 3 connects the hosted runtime lifecycle to the persisted durable boundary.

Required work:

- wire durable hosted-runtime lifecycle through recovery-required states
- enforce the durable hosted-mutation purity rule
- implement authoritative append plus publication-progress recording as one
  coherent durable publication path
- implement acknowledgment eligibility only after WAL and authoritative
  publication thresholds are crossed
- expose typed retry-resolution surfaces for acknowledgment uncertainty

Exit condition:

- no durable writes can occur before recovery gating completes
- normal durable commit flow crosses one explicit publication path
- acknowledgment uncertainty no longer falls back to ambiguous caller behavior

### Phase 4: Implement Crash Recovery And Full Rebuild Control Lane

Phase 4 makes restart honest by implementing both recovery lanes as first-class
programs.

Required work:

- implement crash recovery from WAL plus authoritative artifacts
- implement full authoritative rebuild as the control lane
- implement duplicate suppression across repeated crash-restart loops
- implement typed recovery decisions for replay, discard, retain, and fail
- emit counters and diagnostics for admitted recovery work

Exit condition:

- crash recovery reaches typed conclusions from admitted durable artifacts
- full rebuild reaches truth conclusions from authoritative artifacts alone
- repeated restart loops do not duplicate published truth

### Phase 5: Prove Crash Boundary Exactness And Recovery Equivalence

Phase 5 turns the durable path into a certifiable database boundary.

Required work:

- run the Milestone 3 named suite:
  `WAL Crash Boundary Exactness Test`
- compare crash-before-ack, crash-after-ack, and repeated crash-restart lanes
- compare crash recovery and full authoritative rebuild conclusions
- emit machine-checkable bundles for truth, replay, restore, failure, and
  counters

Exit condition:

- acknowledged commits survive exactly once
- unacknowledged commits do not partially publish
- crash recovery and rebuild remain equivalent in truth conclusions
- Milestone 3 closeout evidence exists in machine-checkable form

## Must Ship

- store-owned hosted runtime durable lifecycle with typed recovery gating
- append-only WAL with declared record taxonomy
- "log before acknowledge" durable commit contract
- typed durable publication boundary proofs
- crash recovery from WAL plus authoritative artifacts
- full authoritative rebuild lane retained as a first-class control path
- duplicate suppression across repeated crash-restart loops
- typed durable recovery diagnostics and counters
- Milestone 3 certification through the named suite in
  [test-requirements.md](./test-requirements.md)

## Must Preserve

- runtime semantics and commit legality stay owned by `worth-relational`
- Milestone 1 authoritative artifacts remain the only semantic durable truth
- Milestone 2 mode boundaries remain explicit and typed
- recovery conclusions derive from canonical authoritative truth, not WAL alone
- no partial transaction truth publishes after crash
- backend variation does not change crash-boundary truth conclusions

## Acceptance Evidence

Milestone 3 is complete only when the store satisfies the named Milestone 3
suite:

- `WAL Crash Boundary Exactness Test`

Required machine-checkable outputs:

- `truth_digest`
- `replay_digest`
- `restore_digest`
- `failure_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- exact acknowledged-commit survival after crash-after-ack lanes
- exact non-publication of partial truth after crash-before-ack lanes
- duplicate suppression across repeated crash-restart loops
- crash recovery versus full rebuild equivalence for retained truth
- typed failure and localization for corrupted or irreconcilable WAL/recovery
  state

Milestone 3 is not closed by "store restarted successfully" tests.

## Architectural Notes

- The smart abstraction is not "one recovery helper." The smart abstraction is
  one durable publication state machine plus sealed proof-bearing transitions
  around it.
- The WAL must stay honest by recording recovery-relevant facts, not by
  becoming a richer second history language than the authoritative commit model.
- Recovery diagnostics should be richer than steady-state append diagnostics,
  but they should still talk in canonical artifact terms.
- Durable lifecycle, WAL persistence, and recovery verification should remain
  separate subdomains even if one backend initially implements all of them.
- Later snapshot or delta accelerators must consume this durable boundary; they
  are not allowed to backfill it.

## Sequencing Notes

This milestone belongs immediately after operating-mode closure because it is
the first durable-mode milestone where store-hosted runtime ownership must
survive process death honestly.

- `Milestone 4` snapshot persistence depends on having an already-honest crash
  boundary and recovery control lane.
- `Milestone 5` branch delta layering depends on crash-safe publication and
  restart before it can claim proportional storage honestly.
- `Milestone 7` schema/lineage/cursor durability may integrate
  transaction-coupled durable artifacts after this milestone freezes recovery
  semantics.

If Milestone 3 is weak, every later recovery accelerator turns into a bet that
the durable boundary "probably works." This spec exists to remove that bet.
