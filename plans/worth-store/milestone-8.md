# Milestone 8 Engineering Spec: Live-Query Substrate And Durable Sync Basis

> **Status:** Draft
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
> - [milestone-6.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-6.md)
> - [milestone-7.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-7.md)
> - [milestone-6-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-6-closeout.md)
> - [milestone-7-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-7-closeout.md)
>
> **Concurrent milestone:** `Milestone 10` (`Retention, Compaction, And Reclamation`)
>
> **Impacted later milestone:** `Milestone 12: Replication, Capsules, And Integrity Verification`
>
> **Primary architectural driver:** make basis-pinned "read current truth and
> stay synced" an honest store substrate built from Milestone 6 physical
> narrowing and Milestone 7 durable support truth, without turning the store
> into a second query runtime and while keeping the later Milestone 10
> retention/compaction program free to evolve concurrently

## Goal

Make stable-basis reads and durable cursor continuation first-class store
capabilities so a consumer can read from one declared truth basis and continue
from durable support artifacts without relying on ambient subscriber memory,
transport-local offsets, or backend-local narrowing shortcuts.

## Why This Milestone Exists

Milestone 8 is not "add streaming reads."

It is the milestone that decides whether `worth-store` can expose a durable
read/continuation substrate honestly, or whether every upper layer that wants
"read now and keep up" will have to improvise with:

- cursors that are durable in name but actually transport-local
- basis tokens that are not tied to exact branch/frontier/schema meaning
- aspect-local continuation claims that still broaden into hidden control-path
  reads
- restart behavior that can resume "close enough" but not to the same truth
  surface as a fresh read
- retention-sensitive continuation that later Milestone 10 cannot reason about
  because the basis contract was never made explicit

Milestone 6 already closed the physical narrowing side of the problem:

- admitted aspect-layout scope vocabulary exists
- narrow-read, fallback, and control-lane posture is explicit
- physical chunk/block families are rebuildable and non-authoritative

Milestone 7 already closed the durable support-truth side:

- durable cursor identity and checkpoint progress are explicit
- schema-boundary and lineage support truth survive restart
- support recovery is typed instead of ambient

Milestone 8 has to join those two foundations into one store-owned substrate:

- one exact stable-basis read contract
- one exact continuation contract from durable cursor identity plus basis
- one exact mismatch surface when schema, lineage, basis scope, or continuation
  shape no longer matches
- one exact narrowing contract that admits common shapes honestly and reports
  fallback when it cannot stay narrow

If this milestone is weak, Milestone 12 replication and later sync/export work
will inherit ambiguity about what a durable read basis even means, and
Milestone 10 retention/compaction will have no honest way to decide whether a
continuation basis is still retained, degraded, or no longer resumable.

## Hard Part

The hard part is not fetching a batch of changes.

The hard part is keeping six things separate that naive systems blur into one
streaming convenience layer:

- canonical authoritative commit truth
- Milestone 7 durable support truth for schema, lineage, cursor, and checkpoint
  meaning
- Milestone 6 derived physical narrowing and chunk/block families
- a stable-basis read contract over one declared truth surface
- a durable continuation contract over one declared cursor identity and basis
- upper-layer query execution, delivery, fanout, and subscription policy

The design fails if:

- a stable basis is just "whatever branch head was current when we started"
  instead of an exact durable basis object
- continuation can resume from durable cursor identity while silently drifting
  to a different schema boundary, lineage neighborhood, or aspect scope
- fetch width changes produce a different final truth-visible result
- narrowing claims stay "fast" only because hidden broad fallback or control
  replay is masked as ordinary success
- Milestone 8 stores query-runtime semantics, subscriber policy, or delivery
  classes that belong above the store
- Milestone 10 later needs to reclaim or compact support families but cannot
  tell which retained ranges or basis records are still required for admitted
  continuation because Milestone 8 never froze that vocabulary

Milestone 8 therefore has to make basis identity, continuation identity,
schema/basis compatibility, and narrowing posture explicit enough that later
sync, retention, and replication work can build on them without renegotiating
what "continue from here" means.

## Explicit Assumptions

- Milestone 1 authoritative commit envelopes, branch heads, and history digests
  remain the only semantic durable truth authority.
- Milestone 2 operating-mode boundaries remain unchanged; Milestone 8 is a
  read/continuation substrate milestone, not a new runtime mode.
- Milestone 3 and Milestone 3.5/3.6 publication and recovery rules already
  govern restart, support recovery, and degraded-state reporting for any live-
  query support family this milestone consumes.
- Milestone 4 snapshot basis and restore rules remain intact; Milestone 8 may
  read from a basis that later maps to snapshot-plus-tail surfaces, but it does
  not redefine snapshot authority.
- Milestone 5 branch/frontier identity remains the branch-visible authority
  basis beneath Milestone 6 physical narrowing and Milestone 7 support truth.
- Milestone 6 is closed and provides explicit admitted scope classes, materialized
  layout-support lanes, deterministic chunk-model export, and explicit fallback
  posture for narrow reads.
- Milestone 7 is closed and provides durable cursor identity, cursor
  advancement, schema-boundary artifacts, lineage support truth, subscriber
  checkpoints, and typed support-gap recovery.
- Milestone 9 is already closed and remains independent bulk orchestration; its
  chunk/witness vocabulary may share physical units with Milestone 8 through
  Milestone 6, but Milestone 8 does not inherit bulk checkpoint semantics.
- Milestone 10 will be built concurrently; Milestone 8 must therefore define
  retention-facing basis and continuation vocabulary clearly enough that
  Milestone 10 can classify retained, degraded, or reclaimed continuation
  surfaces without Milestone 8 assuming retention policy ownership now.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the structural failure before
  convenience APIs spread. Milestone 8 therefore starts from durable basis
  exactness and restart/continuation parity, not from "streaming reads would be
  useful."
- `arch_laws.md`
  The most important thing it protects here is separation of authority,
  derivation, and proof-bearing phases. Canonical truth, support truth, layout
  acceleration, basis handles, continuation plans, and upper-layer delivery
  mechanics must remain structurally distinct.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. Milestone 8 must
  expose narrowed items, continuation steps, and broadening explicitly instead
  of hiding whole-scope rereads behind a streaming-sounding surface.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Basis admission, stable read planning, cursor continuation, mismatch
  detection, restart reconstruction, and certification evidence must be
  separate subdomains rather than one live-query helper module.
- `worth_store_vision.md`
  The most important thing it protects is that store owns durable survival, not
  runtime semantics. Milestone 8 therefore persists and consumes basis/cursor
  support truth faithfully, but it does not become a second query engine.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 8 belongs after
  Milestone 6 and Milestone 7 because durable sync basis is dishonest until
  both physical narrowing and support-truth durability are explicit.
- `worth-store/test-requirements.md`
  The most important thing it protects is certification-grade parity. Milestone
  8 is not closeable until the `Live-Query Basis Continuation Equivalence Test`
  proves stable-basis reads plus durable continuation converge to the same
  truth as a fresh full read.
- `worth_store_dependency_map.md`
  The most important thing it protects is the unlock boundary. Milestone 6 and
  Milestone 7 together unlock Milestone 8, and Milestone 8 in turn unlocks
  replication/capsule work by freezing durable read/sync basis honestly.
- `milestone-6.md`
  The most important thing it protects is that narrowing stays derived,
  explicit, and non-authoritative. Milestone 8 must consume admitted
  narrow-read surfaces rather than renegotiating what selective reads mean.
- `milestone-6-closeout.md`
  The most important thing it protects is the now-closed three-lane layout
  support model and deterministic chunk/block substrate. Milestone 8 can assume
  materialized narrowing exists, but it must still preserve proof-only versus
  materialized honesty in its evidence and fallback posture.
- `milestone-7.md`
  The most important thing it protects is explicit support authority for
  schema, lineage, cursor, and checkpoint truth. Milestone 8 must build
  continuation on those surfaces instead of smuggling semantics through
  transport offsets or caller memory.
- `milestone-7-closeout.md`
  The most important thing it protects is that durable cursor identity,
  exactly-once support publication, and typed degraded recovery are already
  machine-checked. Milestone 8 should depend on those contracts directly rather
  than restating weaker equivalents.
- `milestone-9.md`
  The most important thing it protects is cross-milestone honesty around shared
  physical units. Milestone 8 may share Milestone 6 chunk/layout vocabulary,
  but it must not absorb Milestone 9 bulk orchestration or checkpoint meaning.
- `milestone-9-closeout.md`
  The most important thing it protects is that bulk resumability is already its
  own closed lane. Milestone 8 therefore only needs to define read/continuation
  parity for live-query basis, not another generalized resume framework.

## Adversarial Constraint

Milestone 8 must survive this hostile condition:

> A consumer that reads from one declared stable branch/frontier basis and then
> continues through durable cursor artifacts across restart points, varying
> fetch widths, schema-boundary changes, lineage-bearing commits, explicit
> narrow-read fallback classes, and in-progress Milestone 10 retention work
> must converge to the same final truth-visible result as a fresh control lane
> that reconstructs the same declared read surface from canonical authority
> plus support truth alone, without transport-local memory, hidden broadening,
> or query-runtime-specific interpretation becoming authority.

## Product Decision Lock

- live-query basis is a store-owned durable substrate, not an ambient token
  minted by upper layers without store-verifiable meaning
- every admitted stable basis must bind to one exact branch/frontier support
  context, declared read scope, and schema-boundary support context
- durable continuation must consume explicit Milestone 7 cursor identity and
  checkpoint truth rather than subscriber-local offsets or "latest seen"
  conventions
- continuation is exact only for one declared basis compatibility class; cross-
  schema, cross-scope, cross-branch, or cross-feed-shape reuse must fail typed
  rather than broadening optimistically
- Milestone 8 may consume Milestone 6 admitted narrowing and materialized
  layout support, but it may not invent a second narrowing meaning, silent
  materialization lane, or backend-private fast path
- stable-basis read and continuation surfaces may return explicit fallback or
  reject outcomes, but they may not conceal fallback broadening behind ordinary
  success
- Milestone 8 remains a substrate for read and sync basis; query planning,
  filter semantics, subscription fanout, delivery policy, and network protocol
  remain above the store
- continuation evidence must be explicit enough that Milestone 10 can later
  determine whether a continuation basis remains fully retained, degraded but
  recoverable, or no longer resumable after retention/reclaim decisions
- deleting derived Milestone 6 layout materializations must not change
  continuation meaning, only cost and fallback posture
- deleting Milestone 7 or Milestone 8 support artifacts beyond retained policy
  must never be treated as ordinary continuation success; such lanes must
  degrade or reject explicitly

Normative consequence:

- any implementation that resumes from a durable cursor without proving basis
  compatibility is out of spec
- any implementation that reports "caught up" after hidden broadening changed
  the truth-visible surface is out of spec
- any implementation that lets a basis token survive retention/reclaim without
  explicit retained-versus-degraded classification is out of spec
- any implementation that requires upper layers to remember schema or frontier
  details not present in store-verifiable basis state is out of spec
- any implementation that pushes query-runtime or subscriber-delivery semantics
  into the store in order to make continuation work is out of spec

## Scope

### In Scope

- explicit stable-basis read vocabulary tied to branch/frontier/support truth
- basis identity, compatibility, and mismatch detection surfaces
- durable continuation planning and execution from Milestone 7 cursor identity
  plus declared basis
- storage-visible CDC narrowing for admitted first-ship continuation shapes
- exact fallback, reject, and broadening surfaces for unsupported or widened
  continuation/read shapes
- restart and degraded recovery rules for basis and continuation artifacts
- evidence, counters, and machine-checkable certification for stable-basis read
  and continuation parity
- retention-facing continuation status vocabulary that Milestone 10 can consume
  later without Milestone 8 owning retention policy now

### Explicitly Out Of Scope

- query-language semantics, predicate planning, or application-specific query
  interpretation
- subscriber fanout, delivery classes, network protocol semantics, or sync
  conflict resolution above the store
- branch retention, compaction, reclaim, or admission policy themselves, which
  remain Milestone 10 work
- replication capsules, export scope packaging, and cross-machine basis
  shipping, which remain Milestone 12 work
- bulk ingest or transform resumability, which remain Milestone 9 work
- generalized arbitrary-scope narrowing beyond the admitted first-ship live-
  query shapes frozen here

## Live-Query Authority Model

### Stable Basis Non-Authority Rule

A stable basis is not a second source of truth.

It is a durable support object that binds one read or continuation lane to the
authoritative branch/frontier and support-truth context it claims to observe.

Stable basis artifacts are authoritative only for their support role:

- proving which branch/frontier truth surface a read started from
- proving which declared scope and compatibility class continuation is allowed
  to reuse
- proving which schema-boundary and support-truth context continuation must
  remain compatible with

They are not allowed to redefine:

- canonical commit meaning
- branch-head authority
- query semantics above the store
- delivery policy or subscriber semantics above the store

Required classification rule:

- canonical commit families remain `Authoritative`
- Milestone 7 cursor/schema/lineage/checkpoint families remain authoritative
  support families inside the authoritative tier
- stable basis and continuation families introduced here remain authoritative
  support families subordinate to canonical commit authority
- Milestone 6 layout slices, structural blocks, and chunk families remain
  `DerivedDurable`
- in-memory subscriber cursors, transport offsets, and caller-local "resume
  later" hints remain `Ephemeral`

### Stable Basis Identity Rule

Milestone 8 must freeze one exact basis identity surface.

Every admitted stable basis in this milestone must declare at minimum:

- `StableBasisId`
- `BranchId`
- `BasisFrontier`
- `BasisReadScope`
- `BasisSupportContextDigest`
- `BasisSchemaBoundaryId`
- `BasisLayoutPosture`
- `BasisFamilyVersion`
- `AuthorityBasisDigest`

Required meaning:

- `BasisFrontier`
  identifies the exact branch/frontier truth surface the basis read observed
- `BasisReadScope`
  identifies the admitted first-ship read scope class and its canonicalized
  contents
- `BasisSupportContextDigest`
  binds the basis to the Milestone 7 support-truth context needed for honest
  continuation
- `BasisSchemaBoundaryId`
  identifies the active schema-boundary support artifact for that basis
- `BasisLayoutPosture`
  records whether the basis lane used proof-only, on-demand materialized, or
  policy-eager materialized Milestone 6 layout support
- `AuthorityBasisDigest`
  binds the basis to canonical authoritative truth

Rules:

- two basis artifacts that claim the same semantic start point and scope must
  canonicalize to the same identity inputs or fail typed
- a basis may not be defined as "latest on branch" or "current for subscriber"
  without freezing one exact frontier and support context
- a basis constructed through explicit fallback must still record the fallback
  class it depended on

### First-Ship Product Rule

Milestone 8 must freeze not just scope classes, but the concrete public product
shapes the store admits in its first ship.

Minimum admitted first-ship products:

- `StableBasisProjection`
  one exact basis read result over one admitted scope class
- `ContinuationDeltaBatch`
  one forward-only continuation batch over one compatible stable basis and one
  durable cursor or checkpoint identity

Not admitted in this milestone:

- arbitrary query-language result sets
- server-owned delivery bundles
- fanout-oriented subscriber multiplexing envelopes
- "current truth" surfaces whose contents depend on ambient runtime policy not
  frozen into the basis

This is the anti-"we shipped a generic stream somehow" rule.

### Continuation Batch Envelope Rule

Milestone 8 must define one exact batch envelope shape so continuation remains
derivable from canonical truth rather than ad hoc query reevaluation.

Every admitted `ContinuationDeltaBatch` must declare at minimum:

- `ContinuationBatchId`
- `StableBasisId`
- `CursorIdentity`
- `CoveredCommitRange`
- `FromContinuationFrontier`
- `ToContinuationFrontier`
- `ResolvedScope`
- `ResolvedLayoutPosture`
- `ChangeEnvelopeDigest`
- `NarrowingDisposition`
- `BatchCostSurface`

Required meaning:

- `CoveredCommitRange`
  identifies the exact canonical commits whose admitted change surface is
  represented by the batch
- `FromContinuationFrontier` and `ToContinuationFrontier`
  make the continuation step monotonic and gap-checkable
- `ResolvedScope`
  records the actual admitted or broadened scope the batch used
- `ChangeEnvelopeDigest`
  binds the emitted batch to the exact derived change envelope returned
- `NarrowingDisposition`
  records whether the batch was:
  - `AdmittedNarrow`
  - `ExplicitBroadened`
  - `ControlLane`
- `BatchCostSurface`
  carries counters and strategy fields sufficient to certify the cost path

Rules:

- batch identity must be deterministic from basis identity, cursor identity,
  covered commit range, resolved scope, and batch family version
- the same canonical history plus the same admitted basis and fetch-width lane
  may vary in batch partitioning, but the concatenated truth-visible result
  must converge exactly across equivalent lanes
- a batch may not contain changes outside its declared covered commit range

### Canonical Change Envelope Rule

Milestone 8 continuation output must derive from canonical commit/support
summaries plus admitted scope projection, not from an implementation-defined
"current query rerun" path.

Required derivation posture:

- continuation planning identifies the relevant canonical commit range
- Milestone 7 support truth supplies cursor, schema, and support-context facts
- Milestone 6 scope/layout machinery projects admitted affected items for the
  declared scope
- the batch envelope materializes one derived change envelope from those inputs

Forbidden posture:

- recomputing an arbitrary query result set and diffing it against caller memory
- inventing continuation items from transport-local cache state
- silently substituting full query reevaluation while still reporting
  `AdmittedNarrow`

This is the anti-second-query-runtime rule for Milestone 8.

### Continuation Compatibility Rule

Milestone 8 must make continuation admissibility explicit before any batch is
fetched.

Required compatibility dimensions:

- cursor identity equivalence
- branch identity equivalence
- schema-boundary compatibility
- basis read-scope compatibility
- basis support-context compatibility
- continuation shape compatibility for the admitted first-ship narrowing class

Required rule:

- continuation planning consumes one stable basis artifact and one durable
  cursor identity or checkpoint surface
- the store must prove compatibility across all admitted dimensions before
  producing an executable continuation plan
- any dimension that is missing, mismatched, or unsupported must fail typed
  rather than broadening into "best effort continue from latest"

This is the anti-ambient-resume rule for Milestone 8.

### Initial Admitted Read And Continuation Shapes Rule

Milestone 8 must freeze a real first-ship boundary instead of claiming general
live-query continuation for arbitrary scopes.

Minimum admitted first-ship basis/read shapes:

- `SingleEntityAspectBasis`
  one declared entity plus one or more declared aspects
- `EntitySetUniformAspectBasis`
  one declared entity-id set plus one declared aspect set applied uniformly
- `CdcTouchedAspectBasis`
  one continuation basis over the touched entity/aspect pairs already proven by
  canonical commit/support summaries

Minimum admitted first-ship continuation shapes:

- `CursorForwardContinuation`
  continue forward from one durable cursor identity over one compatible stable
  basis
- `CheckpointedCursorContinuation`
  continue forward from one durable subscriber checkpoint over one compatible
  stable basis

Rules:

- any requested shape outside the admitted first-ship catalog must produce an
  explicit typed fallback or typed rejection
- Milestone 8 may widen the admitted catalog later, but it may not claim
  general continuation while routing all complex cases through hidden broad
  control lanes

### Deterministic Item Ordering Rule

Milestone 8 must make batch item ordering explicit so fetch-width variation
cannot silently produce duplicate, skipped, or unstable continuation payloads.

Required ordering basis per admitted batch:

- canonical commit order
- canonical item ordering within each covered commit for the admitted scope
- canonical aspect ordering within each item where multiple aspects are present
- batch family version

Rules:

- two equivalent continuation lanes may partition commits into different batch
  widths, but each batch's internal ordering must follow the same canonical
  ordering rules
- if a caller concatenates batches from a narrow lane and a wider lane over the
  same admitted basis, the final ordered change envelope must converge exactly
- ordering may not depend on backend row order, hash-map iteration order, or
  cache residency

### Retention-Facing Continuation Status Rule

Milestone 8 must define the vocabulary Milestone 10 will later govern.

Required continuation status classes:

- `FullyRetainedContinuationBasis`
- `DegradedButRecoverableContinuationBasis`
- `RejectedContinuationBasis`

Required meaning:

- `FullyRetainedContinuationBasis`
  the declared basis and support context remain retained enough for admitted
  continuation without extra reconstruction
- `DegradedButRecoverableContinuationBasis`
  the declared basis cannot continue on the originally admitted fast path but
  can still be reconstructed or revalidated from retained authority/support
  ranges
- `RejectedContinuationBasis`
  the declared basis can no longer be continued honestly under the retained
  support/authority surface

Milestone 8 does not decide retention policy. It only freezes the continuation
status vocabulary that Milestone 10 will later populate through retention and
reclaim decisions.

### Retention-Facing Basis Descriptor Rule

Milestone 8 must publish the exact facts Milestone 10 will later govern.

Every admitted stable basis must be reducible to one
`ContinuationRetentionDescriptor` containing at minimum:

- `StableBasisId`
- `MinimumRetainedCommitRange`
- `RequiredSupportArtifactSet`
- `SchemaBoundaryDependency`
- `AuthorityReplayFallbackClass`
- `SnapshotTailFallbackClass`
- `RetentionDescriptorVersion`

Required meaning:

- `MinimumRetainedCommitRange`
  identifies the minimum canonical range that must remain available for the
  originally admitted continuation posture
- `RequiredSupportArtifactSet`
  names the Milestone 7 support families continuation depends on
- `AuthorityReplayFallbackClass`
  states whether the basis may degrade to authority replay if the fast-path
  range is no longer retained
- `SnapshotTailFallbackClass`
  states whether the basis may degrade to snapshot-plus-tail revalidation if
  that posture remains admitted later

Rules:

- Milestone 10 may govern whether those requirements remain satisfied
- Milestone 10 may not invent a stronger or weaker meaning for them
- if a basis cannot emit this descriptor, it is not an admitted Milestone 8
  basis

## Physical Narrowing And Support Integration Rules

### Milestone 6 Narrowing Consumption Rule

Milestone 8 may only claim narrowed stable-basis reads or narrowed continuation
when it is using one of the admitted Milestone 6 scope classes and one of the
explicit Milestone 6 layout-support postures.

Required rule:

- stable-basis read planning must declare whether it is using:
  - `ProofOnly`
  - `OnDemandMaterialized`
  - `PolicyEagerMaterializedPublished`
  - `PolicyEagerMaterializedReuseExisting`
- continuation results must preserve the requested and resolved narrowing
  posture in their evidence surface
- if the basis or continuation path widens into a control lane, that widening
  must be explicit in the result envelope and counters

This prevents Milestone 8 from laundering layout fallback through a
live-query-specific success surface.

### Milestone 7 Support Consumption Rule

Milestone 8 continuation planning must consume explicit Milestone 7 support
truth instead of inferring it from raw commit history or caller memory.

Required consumed support families:

- durable cursor identity
- durable cursor checkpoint or subscriber checkpoint when used
- schema-boundary support artifact for the basis and continuation frontier
- support-context digest or equivalent support-truth witness tying basis and
  continuation to the same admitted support context

Rules:

- basis planning may not synthesize schema support context from "current
  branch schema" without a durable support artifact
- continuation may not trust subscriber-local memory to re-establish support
  context after restart
- support gaps remain typed degraded or reject outcomes rather than broadening
  into ordinary continuation

### Basis-To-Cursor Planning Rule

Stable-basis read and durable continuation must remain separate proof phases.

Required proof-bearing type families:

- `StableBasisReadPlan`
- `PublishedStableBasis`
- `ContinuationCompatibilityWitness`
- `AdmittedCursorContinuationPlan`
- `DegradedContinuationPlan`
- `ContinuationBatchReceipt`
- `AcknowledgedContinuationAdvance`

Required rule:

- stable-basis read planning proves what exact truth surface was observed
- continuation planning proves that one durable cursor identity may continue
  from that observed basis
- execution consumes the admitted continuation plan and produces one batch
  receipt that includes truth-visible deltas, cost evidence, and fallback or
  degradation posture

Milestone 8 should not collapse these phases into one mutable "stream session"
object.

### Compile-Time Enforcement Rule

The highest-risk Milestone 8 mistakes must be made unrepresentable or
uncompilable.

Required compile-time posture:

- a raw cursor identity may not execute continuation without a
  `ContinuationCompatibilityWitness`
- a raw stable basis handle may not acknowledge advancement on its own
- only an executed `ContinuationBatchReceipt` may produce an
  `AcknowledgedContinuationAdvance`
- degraded continuation and admitted continuation must be distinct types, not
  one enum that callers can ignore accidentally
- public facade calls may not accept loosely structured "subscriber id plus
  maybe basis token" convenience arguments that force compatibility discovery at
  runtime

Required proof surface:

- compile-fail tests for advancing continuation from a plan that never executed
- compile-fail tests for constructing an admitted continuation plan from raw
  cursor plus raw basis fields
- compile-fail tests for acknowledging a broadened or degraded batch as though
  it were an admitted narrow batch when the policy requires explicit caller
  acknowledgment
- compile-fail tests for Milestone 10-facing retention descriptors built from
  partial basis state

This is the anti-"the type let me do it" rule for Milestone 8.

### Acknowledgment Monotonicity Rule

Continuation acknowledgment must be derived from one executed batch receipt,
not from intent to execute.

Required rule:

- `AcknowledgedContinuationAdvance` may only be produced from:
  - one `ContinuationBatchReceipt`
  - one matching cursor identity
  - one matching stable basis
  - one matching `ToContinuationFrontier`
- acknowledgment must be monotonic in continuation frontier
- duplicate acknowledgment of the same batch must be identity-detectable and
  exactly-once safe

Forbidden posture:

- acknowledging advancement from a planned batch before its change envelope was
  materialized
- acknowledging advancement from caller memory after the batch was delivered
  externally
- acknowledging a different frontier than the executed batch proved

## Failure Taxonomy

Milestone 8 must ship an explicit typed error family matrix at minimum
covering:

- `StableBasisVersionUnsupported`
- `StableBasisScopeUnsupported`
- `StableBasisDigestMismatch`
- `StableBasisSchemaMismatch`
- `StableBasisSupportContextMismatch`
- `StableBasisLayoutPostureUnsupported`
- `StableBasisRetentionStatusUnknown`
- `ContinuationBatchOrderingViolation`
- `ContinuationBatchGap`
- `ContinuationBatchDuplicate`
- `ContinuationCursorIdentityMismatch`
- `ContinuationBranchMismatch`
- `ContinuationSchemaMismatch`
- `ContinuationScopeMismatch`
- `ContinuationCheckpointMissing`
- `ContinuationCheckpointGap`
- `ContinuationBasisNotRetained`
- `ContinuationBasisDegraded`
- `ContinuationAdvanceIllegal`
- `ContinuationRequiresBroadControlLane`
- `ContinuationParityViolation`
- `ContinuationFallbackDebtRequired`
- `ContinuationFamilyVersionUnsupported`

Rules:

- basis admission, continuation planning, execution, restart reconstruction,
  and parity verification must map failures into these families or explicit
  refinements of them
- backend-driver and transport-layer failures must not leak as the public
  semantic taxonomy
- typed degraded outcomes must remain distinguishable from typed hard rejects

## Performance Encoding Rules

Milestone 8 must encode performance into architecture and type boundaries, not
leave it as a later benchmark exercise.

The core rule is:

- continuation cost must be bounded by declared basis scope, declared frontier
  delta, and declared batch budgets
- any widening beyond that must be explicit in planning, result types, and
  counters

### Batch Budget Rule

Continuation planning must consume one explicit batch-budget contract rather
than one unstructured fetch-width integer.

Minimum budget objects:

- `FetchWidth`
- `MaxBatchItems`
- `MaxCoveredCommits`
- `MaxMaterializedBytes`
- `MaxSupportRowsPerBatch`

Required rule:

- an admitted continuation plan must bind to one declared batch-budget object
- widening one budget dimension may not silently widen the others
- a plan that cannot satisfy the requested budget envelope must degrade or
  reject before execution

Forbidden posture:

- treating `fetch_width = 1000` as permission to read any amount of commit
  history, support rows, or payload bytes
- letting allocator pressure or backend packing implicitly decide batch shape
  under one supposedly stable plan

### Pre-Resolved Strategy Rule

Execution must consume one pre-resolved continuation strategy instead of
re-deciding it batch by batch.

Minimum strategy classes:

- `AdmittedLayoutNarrow`
- `ExplicitBroadened`
- `AuthorityReplayControlLane`

Required rule:

- planning resolves the strategy from basis shape, cursor compatibility,
  retention status, and layout posture before batch execution begins
- execution consumes the lowered strategy without reinterpreting eligibility at
  runtime
- changing strategy requires a new plan or an explicit degraded-plan family

This is the anti-plan/execute-conflation rule for Milestone 8.

### Fast-Path Versus Fallback Type Rule

Admitted narrow continuation and broadened continuation must be distinct
runtime objects, not merely one success payload with a bool.

Required posture:

- admitted narrow batches carry one `AdmittedNarrowBatchReceipt`
- broadened batches carry one `BroadenedBatchReceipt`
- control-lane batches carry one `ControlLaneBatchReceipt`
- acknowledgment policy may distinguish among them

Rules:

- callers may not accidentally treat a broadened batch as though it preserved
  the admitted fast-path contract
- evidence and diagnostics may summarize across these families later, but the
  execution boundary must not erase the distinction

### Carried-Proof Rule

Milestone 8 must consume expensive facts already proven by earlier milestones
instead of rediscovering them per batch.

Minimum carried-forward facts:

- touched entity/aspect sets from Milestone 6 control/narrowing summaries
- schema-boundary and support-context facts from Milestone 7 support truth
- retained/degraded/rejected continuation status from the basis descriptor
- canonical frontier linkage already proven by cursor/checkpoint artifacts

Rules:

- continuation planning may consume those proofs
- continuation execution may not rescan broad history merely to rediscover them
  inside the same trust boundary
- if one carried-forward fact is missing, the lane must degrade or reject
  explicitly rather than silently recomputing from arbitrary history

### Cold-Path And Warm-Path Rule

Milestone 8 must say which continuation paths are expected to remain honest
when the process starts cold.

Minimum cold-path expectation:

- stable-basis lookup and continuation planning remain admitted from durable
  basis artifacts, durable support truth, and Milestone 6 durable layout
  families without requiring warm in-memory caches

Minimum warm-path-optional expectation:

- hot scope caches and hot batch assembly caches may improve cost, but they may
  not be required for `Verified` correctness or narrow-path truthfulness

Rules:

- certification evidence should be able to state whether a lane ran cold or
  warm
- if a path is only `Verified` when warm and broadens or rescans when cold,
  that path is `Debt`

### Retention-Driven Strategy Boundary Rule

Retention status must influence the plan family before execution starts.

Required rule:

- `FullyRetainedContinuationBasis` may admit `AdmittedLayoutNarrow`
- `DegradedButRecoverableContinuationBasis` may admit only an explicit
  degraded strategy family
- `RejectedContinuationBasis` may not produce an executable continuation plan

This prevents a naive implementation from discovering mid-batch that the fast
path was never really retained.

### Public Cost Surface Rule

Performance accounting must be embedded in public batch and basis envelopes.

Minimum required public cost fields:

- `covered_commit_count`
- `narrowed_item_count`
- `broadened_item_count`
- `support_rows_read`
- `scope_lookup_count`
- `fallback_class`
- `complexity_status`
- `resolved_strategy`

Rules:

- callers must be able to tell whether a batch stayed within the admitted cost
  contract
- diagnostics may add richer detail, but the public surface may not hide the
  strategy or broadening posture

### Local Budget Rule

Milestone 8 must define local architectural budgets even before Milestone 10
global admission control lands.

Minimum local budgets:

- `MaxAdmittedScopeCardinality`
- `MaxAdmittedContinuationGap`
- `MaxSupportRowsPerBatch`
- `MaxExplicitBroadeningBreadth`

Rules:

- exceeding one of these budgets must produce a typed degrade, reject, or
  explicit `Debt` outcome
- no admitted fast path may quietly exceed its local budget and still report as
  ordinary verified success

### Compile-Time Performance Boundary Rule

The highest-risk performance mistakes must be mechanically blocked.

Required compile-time posture:

- raw fetch-width integers may not construct an admitted continuation plan
  without an explicit batch-budget wrapper
- a broadened or control-lane receipt may not be acknowledged through the same
  API surface as an admitted narrow receipt when explicit caller choice is
  required
- a partial retention descriptor may not be constructed as though it were a
  full `ContinuationRetentionDescriptor`
- public facade calls may not accept convenience arguments that force runtime
  inference of scope, retention posture, or strategy

Required proof surface:

- compile-fail tests for raw integer batch-plan construction
- compile-fail tests for acknowledging broadened receipts through narrow-only
  acknowledgment paths
- compile-fail tests for partial retention descriptor construction

## Complexity Contracts

Milestone 8 must encode cost honesty into the architecture itself.

Minimum contracts:

- stable-basis read cost is proportional to:
  - the admitted Milestone 6 layout scope lookup work for the declared basis
  - support-artifact reads needed to bind schema and support context
  - declared control-lane parity breadth when verification is required
- continuation planning cost is proportional to:
  - one durable cursor identity lookup
  - one stable basis lookup
  - support-artifact compatibility checks for the declared basis and cursor
  - not total branch history or total subscriber population
- continuation execution cost is proportional to:
  - continuation batches fetched for the declared cursor frontier delta
  - narrowed items admitted for the declared scope
  - explicit fallback breadth when the resolved lane widened

Forbidden fallback work that must be made mechanically visible:

- hidden full-branch rereads during admitted narrowed continuation
- hidden "latest cursor for subscriber" search across unrelated cursor
  identities
- hidden schema or support-context rediscovery by replaying arbitrary history
- hidden control-lane substitution while still reporting ordinary narrowed
  continuation success

Minimum counters:

- `stable_basis_read_count`
- `stable_basis_lookup_count`
- `stable_basis_support_rows_read`
- `stable_basis_scope_lookup_count`
- `stable_basis_fallback_count`
- `stable_basis_broadening_count`
- `continuation_batch_gap_count`
- `continuation_batch_duplicate_count`
- `continuation_plan_count`
- `continuation_cursor_identity_lookup_count`
- `continuation_checkpoint_lookup_count`
- `continuation_support_rows_read`
- `continuation_batch_count`
- `continuation_narrowed_item_count`
- `continuation_broadened_item_count`
- `continuation_step_count`
- `continuation_schema_mismatch_count`
- `continuation_scope_mismatch_count`
- `continuation_degraded_basis_count`
- `continuation_rejected_basis_count`
- `continuation_control_lane_fallback_count`
- `continuation_parity_failure_count`
- `continuation_illegal_acknowledgment_count`

Required counter assertions:

- `continuation_cursor_identity_lookup_count` must remain bounded to the
  declared cursor identity lane for representative admitted continuation cases
- `continuation_control_lane_fallback_count` must remain zero for
  representative admitted narrowed continuation lanes; any non-zero lane must
  be named as explicit fallback or explicit debt
- `continuation_broadened_item_count` must remain zero for representative
  admitted narrowing lanes and non-zero only where the lane explicitly reports
  broadening
- `continuation_parity_failure_count` must remain zero in all equivalent
  certification lanes
- `continuation_illegal_acknowledgment_count` must remain zero in all admitted
  lanes and increment only in explicit hostile proof lanes

Debt posture:

- unsupported continuation shapes may remain `Debt` or explicit reject in this
  milestone
- admitted continuation shapes may not imply verified exactness while relying
  on hidden broad replay or hidden support rediscovery

## Public Surface

Milestone 8 should keep the public facade basis- and cursor-oriented.

Representative surface:

```rust
pub struct StableBasisReadRequest { ... }
pub struct StableBasisHandle { ... }
pub struct CursorContinuationRequest { ... }
pub struct CursorContinuationPlan { ... }
pub struct ContinuationBatchResult { ... }
pub struct ContinuationAdvanceReceipt { ... }

impl WorthStore {
    pub fn read_stable_basis(
        &self,
        request: StableBasisReadRequest,
    ) -> Result<StableBasisHandle, StableBasisError>;

    pub fn plan_cursor_continuation(
        &self,
        request: CursorContinuationRequest,
    ) -> Result<CursorContinuationPlan, CursorContinuationPlanningError>;

    pub fn execute_cursor_continuation(
        &self,
        plan: CursorContinuationPlan,
    ) -> Result<ContinuationBatchResult, CursorContinuationError>;

    pub fn acknowledge_cursor_continuation(
        &mut self,
        receipt: ContinuationAdvanceReceipt,
    ) -> Result<(), CursorContinuationAcknowledgeError>;
}
```

Surface rules:

- stable-basis read must expose branch/frontier/scope vocabulary explicitly
- continuation planning and execution should remain separate concepts if that
  is what keeps compatibility proof boundaries honest
- continuation acknowledgment should remain separate from execution if that is
  what keeps exactly-once and monotonic advancement honest
- result surfaces must report requested-versus-resolved narrowing posture and
  retained-versus-degraded continuation status
- no API may imply that the store owns general query semantics or delivery
  policy

## Required Internal Subsystems

Milestone 8 should decompose by responsibility:

- `live_query/basis/`
  stable-basis identity, scope binding, and basis publication
- `live_query/compatibility/`
  cursor/basis/schema/support-context compatibility proofs
- `live_query/continuation/`
  continuation planning and batch execution
- `live_query/acknowledgment/`
  monotonic advancement and exactly-once batch acknowledgment
- `live_query/restart/`
  degraded basis reconstruction and restart-visible continuation outcomes
- `live_query/retention_descriptor/`
  retention-facing basis descriptor publication
- `live_query/evidence/`
  counters, parity bundles, and certification output
- `backend/`
  backend support for basis and continuation families without owning their
  semantics

## Phases

### Phase 1: Lock Stable Basis Vocabulary And Continuation Compatibility

Required work:

- define stable-basis identity, scope, schema, and support-context vocabulary
- define the admitted first-ship basis and continuation shapes
- define retained/degraded/rejected continuation status vocabulary for later
  Milestone 10 integration
- define continuation batch envelope identity and deterministic ordering rules
- define proof-bearing stable-basis and continuation planning types
- define proof-bearing acknowledgment and retention-descriptor types
- define explicit failure families and counter contracts

Exit condition:

- one exact basis identity model exists
- one exact continuation-compatibility model exists
- Milestone 10-facing retention vocabulary is frozen without Milestone 8 taking
  ownership of retention policy

### Phase 2: Persist Stable Basis Artifacts And Basis Read Surfaces

Required work:

- implement stable-basis read planning and publication
- bind basis publication to explicit Milestone 6 layout posture and Milestone 7
  support context
- emit a retention-facing basis descriptor from admitted basis state
- implement typed basis fetch, scope mismatch, and schema mismatch failures
- emit exact basis-read and broadening counters

Exit condition:

- a stable basis is a real durable support artifact instead of a caller-local
  token
- basis reads are basis-explicit and cost-honest

### Phase 3: Implement Durable Cursor Continuation Over Admitted Shapes

Required work:

- implement cursor-to-basis compatibility planning
- implement continuation execution for admitted first-ship shapes
- emit deterministic continuation batch envelopes with canonical ordering
- preserve requested-versus-resolved narrowing posture in result envelopes
- implement monotonic acknowledgment from executed batch receipts only
- expose typed continuation mismatch, fallback, and degraded-basis outcomes
- emit exact continuation plan, batch, narrowed-item, broadening, and
  acknowledgment counters

Exit condition:

- admitted continuation no longer depends on ambient subscriber memory
- continuation truth and cost posture are explicit per batch

### Phase 4: Harden Restart, Degraded Basis, And Milestone 10 Concurrency Boundaries

Required work:

- implement restart reconstruction for stable basis and continuation surfaces
- classify retained, degraded, and rejected continuation status without owning
  retention policy decisions
- make missing or reclaimed support families produce typed degraded or reject
  outcomes
- make batch gap, duplicate, and illegal acknowledgment outcomes typed and
  machine-checkable
- define the concurrency boundary with Milestone 10 so retention/compaction may
  evolve concurrently without changing basis identity semantics

Exit condition:

- restart cannot bluff clean continuation where basis/support truth is missing
- Milestone 10 can progress concurrently against frozen continuation status
  vocabulary

### Phase 5: Prove Live-Query Basis Continuation Equivalence

Required work:

- run the Milestone 8 named suite:
  `Live-Query Basis Continuation Equivalence Test`
- compare stable-basis-plus-continuation lanes against fresh-read control lanes
- compare varying fetch-width lanes against the same truth-visible result
- compare restart lanes against uninterrupted continuation lanes
- compare duplicate, gap, and illegal-acknowledgment hostile lanes against
  typed failure expectations
- emit machine-checkable truth, restore, failure, and counter bundles

Exit condition:

- continued truth matches fresh truth for equivalent admitted workloads
- basis mismatches fail explicitly
- narrowing acceleration changes cost only, not meaning

## Must Ship

- explicit stable-basis durable support artifacts
- explicit basis identity and compatibility vocabulary
- durable cursor continuation planning and execution for admitted first-ship
  shapes
- deterministic continuation batch envelopes with explicit covered commit range
- monotonic exactly-once continuation acknowledgment derived from executed batch
  receipts
- requested-versus-resolved narrowing posture in read and continuation results
- retained/degraded/rejected continuation status vocabulary
- retention-facing basis descriptors for Milestone 10 consumption
- typed basis and continuation failure taxonomy
- exact counters and machine-checkable Milestone 8 certification output

## Must Preserve

- canonical commit history remains the only semantic durable authority
- Milestone 7 support truth remains the support-authority substrate for cursor,
  schema, lineage, and checkpoint meaning
- Milestone 6 layout narrowing remains derived and non-authoritative
- stable basis and continuation do not become a second query runtime
- Milestone 10 remains free to own retention, compaction, and reclaim policy
  over the vocabulary Milestone 8 freezes here

## Acceptance Evidence

Milestone 8 is complete only when the store satisfies the named Milestone 8
suite:

- `Live-Query Basis Continuation Equivalence Test`

Required machine-checkable outputs:

- `truth_digest`
- `restore_digest`
- `failure_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- stable-basis read plus durable continuation converges to the same truth as a
  fresh control read
- varying fetch widths do not change the final truth-visible result
- duplicate, skipped, or out-of-order continuation batches fail explicitly and
  typed
- continuation advancement may only acknowledge executed batches and remains
  monotonic and exactly-once safe
- basis/schema/scope/support-context mismatches fail explicitly and typed
- requested-versus-resolved narrowing posture remains explicit in evidence
- degraded basis lanes remain distinct from rejected basis lanes
- restart does not change continuation conclusions
- Milestone 10-facing continuation status vocabulary remains stable enough for
  concurrent retention work to consume without redefining basis semantics

Milestone 8 is not closed by "streaming looked correct" or "subscriber caught
up once" tests.

## Architectural Notes

- The smart abstraction is not "subscriptions in the store." The smart
  abstraction is one exact stable-basis plus durable-continuation substrate.
- Milestone 8 should prefer basis/cursor proof objects over loosely structured
  session handles.
- Requested-versus-resolved narrowing posture is a first-class truthfulness
  requirement, not optional diagnostics polish.
- The concurrency note with Milestone 10 matters structurally: Milestone 8
  freezes continuation meaning; Milestone 10 later governs how retention and
  reclaim affect that meaning in practice.

## Sequencing Notes

This milestone belongs after Milestone 6 and Milestone 7 because durable sync
basis is dishonest until both physical narrowing and support-truth durability
are explicit and machine-checked.

- Milestone 8 unlocks Milestone 12 because replication/export work needs one
  honest durable read/sync basis to ship around.
- Milestone 10 can be built concurrently once Milestone 4, Milestone 5, and
  Milestone 6 are already honest, but it must treat Milestone 8 basis identity
  and continuation-status vocabulary as fixed input rather than redefining what
  a stable basis or durable continuation means.
- The two milestones solve different problems in parallel:
  - Milestone 8 freezes durable read/continuation meaning
  - Milestone 10 freezes retention/compaction/reclaim behavior over retained
    authority and derived families
- Milestone 12 still depends on both: it needs Milestone 8's durable read/sync
  basis and Milestone 10's stable retention/rebuild rules.
