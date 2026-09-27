# Milestone 7 Engineering Spec: Durable Schema, Lineage, Cursor, And Checkpoint Artifacts

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
>
> **Impacted later milestone:** `Milestone 8: Live-Query Substrate And Durable Sync Basis`
>
> **Primary architectural driver:** make schema-boundary truth, lineage truth,
> durable cursor position, and embedded checkpoint basis artifacts survive
> restart as explicit durable families instead of leaking through commit blobs,
> ad hoc metadata, or transport-local state

## Goal

Make schema-boundary artifacts, lineage artifacts, durable cursor positions,
subscriber checkpoints, and embedded-mode checkpoint artifacts first-class
durable store families so restart, replay, identity resolution, and
basis-pinned continuation remain exact without turning these support artifacts
into shadow truth.

## Why This Milestone Exists

Milestone 7 is not "persist some metadata."

It is the milestone that decides whether `worth-store` can carry the durable
supporting truth that later replay, live-query continuation, and embedded-mode
re-entry actually depend on, or whether those systems will quietly smuggle
their meaning through:

- commit-envelope internals that are expensive to rediscover later
- transport-local cursor state that disappears on restart
- ad hoc lineage caches that forget they are derived
- checkpoints whose basis meaning is only implicit in caller memory

Milestone 1 froze the canonical authoritative commit model.

Milestone 2 froze operating-mode ownership and established that embedded-mode
checkpoints are allowed as persisted artifacts without becoming authority.

Milestone 3 and Milestone 3.5/3.6 froze the durable publication and recovery
vocabulary that any later durable family must obey.

Milestone 4 proved one derived family, snapshots, can be basis-explicit and
non-authoritative.

Milestone 7 now has to do the same kind of structural work for a different
class of artifacts:

- schema-boundary meaning that replay and compatibility need
- lineage continuity that historical identity resolution needs
- durable cursor positions that live-query continuation and sync need
- subscriber checkpoints that durable continuation needs
- embedded checkpoints that external runtimes need for re-entry and restore

If this milestone is weak, Milestone 8 will fake "read current truth and stay
synced" with ambient conventions, and later replication, compatibility, and
repair tooling will inherit that ambiguity.

## Hard Part

The hard part is not serializing a few extra records.

The hard part is holding apart four things that naive storage layers constantly
collapse into one:

- authoritative commit truth
- support artifacts that are authoritative for their own declared role
- derived accelerators that may depend on those support artifacts
- caller-local session state that must not accidentally become durable truth

The design fails if:

- schema boundaries survive only as fields buried inside commit envelopes and
  cannot be fetched, compared, or resumed as explicit durable artifacts
- lineage continuity requires replaying arbitrary history because the store
  never made lineage events durably queryable as their own family
- cursor resume depends on subscriber memory, transport offsets, or
  best-effort "continue from latest" heuristics
- cursor advancement can drift away from the canonical commit frontier it
  claims to acknowledge
- embedded checkpoints can be persisted without exact basis identity,
  contained-commit linkage, or restart-visible classification
- restart can reconstruct commit truth but cannot reconstruct the schema,
  lineage, cursor, or checkpoint support artifacts required to continue
  honestly from that truth

Milestone 7 therefore has to make support artifacts durable and queryable
without letting them outrank the canonical commit model they support.

## Explicit Assumptions

- Milestone 1 authoritative commit envelopes, parent records, branch records,
  branch heads, and digest records remain the semantic truth authority for
  committed history.
- Milestone 2's operating-mode boundary remains unchanged: embedded mode may
  persist external commits and checkpoints, but embedded checkpoints do not
  become authoritative commits themselves.
- Milestone 3 and Milestone 3.5/3.6 publication, durability-barrier, and
  recovery-source rules already govern any new durable family this milestone
  adds.
- Milestone 4 snapshot basis and restore rules remain intact; this milestone
  may reference checkpoints and cursors to snapshot or commit frontiers, but it
  does not renegotiate snapshot authority or restore semantics.
- `worth-relational` still owns schema semantics, lineage semantics, historical
  identity semantics, replay legality, and CDC meaning.
- cursor meaning remains above the store: the store persists durable cursor
  position and checkpoint artifacts, but higher layers still own delivery
  policy, subscription semantics, and query narrowing meaning.
- schema, lineage, cursor, and checkpoint artifacts may be authoritative for
  their declared support role while still remaining subordinate to canonical
  commit truth.
- Milestone 5 is proceeding concurrently as the branch-delta physical program;
  Milestone 7 must therefore avoid depending on branch-delta layout, delta
  rewrite policy, or read-amplification internals to define support-artifact
  meaning.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the structural failure before
  it becomes a convenience dependency. Milestone 7 therefore starts from
  restart, resume, and historical identity exactness, not from "better sync
  ergonomics."
- `arch_laws.md`
  The most important thing it protects here is explicit authority separation
  and proof-carrying progression. Law 33 and Law 41 are load-bearing: canonical
  commits, support-authority artifacts, derived cursor accelerators, and
  session-local caller state must be distinct categories, and schema/lineage/
  cursor/checkpoint phases must progress through sealed proof types rather than
  loose records.
- `perf_laws.md`
  The most important thing it protects is honest continuation cost. Cursor
  resume, lineage lookup, schema-boundary fetch, and checkpoint restore basis
  resolution must expose exact counters and admitted cost bases instead of
  hiding replay breadth behind cheap-looking APIs.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Schema boundaries, lineage persistence, cursor checkpoints, embedded
  checkpoint intake, restart reconstruction, and certification evidence must be
  separate subdomains rather than one generic "metadata" module.
- `worth_store_vision.md`
  The most important thing it protects is that store owns durable survival
  while runtime owns semantics. Milestone 7 therefore persists runtime-produced
  schema and lineage artifacts faithfully, persists cursor and checkpoint
  support truth explicitly, and refuses to invent storage-local semantic
  substitutes.
- `worth_store_roadmap.md`
  The most important thing it protects is sequence. Milestone 7 belongs before
  Milestone 8 because live-query and durable sync basis are dishonest until
  durable cursor, schema-boundary, lineage, and checkpoint truth already
  exist.
- `worth-store/test-requirements.md`
  The most important thing it protects is certification-grade durability.
  Milestone 7 is not closeable until the `Schema/Lineage/Cursor Durability
  Test` proves restart parity, deterministic cursor resume, durable historical
  identity resolution, and non-drifting embedded checkpoints.
- `milestone-1.md`
  The most important thing it protects is canonical authority and exportable
  durable truth. Milestone 7 must build new durable families around that
  authority rather than creating alternate truth records that replay could
  depend on instead.
- `milestone-3.5-3.6.md`
  The most important thing it protects is that every durable family must obey
  typed publication barriers, source precedence, and degraded-state honesty.
  Milestone 7 must therefore define publication and recovery rules for support
  artifacts instead of treating them as "small enough to ignore."
- `milestone-4.md`
  The most important thing it protects is basis-explicit, non-authoritative
  durable families. Milestone 7 should mirror that discipline for cursor and
  checkpoint basis identity rather than allowing implicit frontier drift.
- `worth_store_dependency_map.md`
  The most important thing it protects is the real unlock shape: Milestone 7 is
  an early foundation that can start before Milestone 5 finishes, but Milestone
  8 should only proceed once both the physical narrowing work and the durable
  support-artifact work are honest.

## Adversarial Constraint

Milestone 7 must survive this hostile condition:

> A store that restarts after schema transitions, lineage-bearing commits,
> subscriber progress, embedded checkpoint intake, snapshot capture, durable
> commits, and partial durable cursor advancement must reconstruct the same
> schema-boundary conclusions, historical identity resolution, cursor-resume
> position truth, and checkpoint basis meaning as a control lane that replayed
> the same canonical history and support artifacts from scratch, without
> trusting caller memory, transport-local offsets, or backend-local shortcuts.

## Product Decision Lock

- schema-boundary artifacts are durable first-class families, not only fields
  buried in commit payloads
- lineage events and their historical-resolution basis are durable first-class
  families, not replay-only byproducts
- durable cursor positions and subscriber checkpoints are explicit persisted
  support artifacts keyed to canonical commit or basis identity
- cursor advancement is transactional with the support-artifact boundary it
  acknowledges; "write the cursor later" is out of spec
- embedded checkpoints remain non-authoritative durable artifacts even when
  they carry contained canonical commits
- every checkpoint must declare an exact checkpoint identity, source runtime
  identity, classification, and basis linkage when such basis exists
- support artifacts added here may be authoritative for their own support role,
  but they may not redefine canonical commit history, branch heads, or replay
  semantics
- restart and rebuild must be able to reconstruct support-artifact conclusions
  from their declared durable families without depending on ambient runtime
  session memory
- Milestone 7 must not depend on Milestone 5 physical delta layout; support
  artifact meaning is branch/frontier based, not delta-stack based

Normative consequence:

- any implementation that resumes a cursor from "latest known subscriber
  position" without a persisted durable cursor family is out of spec
- any implementation that resolves historical identity only by replaying
  arbitrary history because lineage artifacts were never made durable is out of
  spec
- any implementation that lets embedded checkpoints become branch-head or
  commit authority is out of spec
- any implementation that advances a durable cursor outside the same admitted
  durable publication unit as the support truth it acknowledges is out of spec

## Scope

### In Scope

- durable schema evolution boundary artifacts and fetch surfaces
- durable lineage event persistence and historical identity resolution support
- durable cursor-position artifacts for CDC or subscriber continuation
- durable subscriber checkpoint artifacts tied to explicit basis identity
- transactional cursor advancement with canonical commit or basis identity
- embedded-mode checkpoint artifact persistence with basis linkage,
  classification, contained-commit linkage, and fetch surfaces
- restart and rebuild recovery rules for support-artifact families
- counters, diagnostics, and certification bundles for schema, lineage, cursor,
  and checkpoint durability
- public store-owned vocabulary for support-artifact fetch, resume planning,
  and checkpoint inspection

### Explicitly Out Of Scope

- live-query narrowing and continuation execution semantics beyond the cursor
  and basis durability substrate needed for Milestone 8
- branch-delta physical storage, delta-stack rewrite, or structural-block
  layout decisions from Milestone 5 and Milestone 6
- replication capsules, export scope negotiation, and cross-machine cursor or
  checkpoint shipping
- multi-resolution materialization, time-travel diff acceleration, or analysis
  checkpoint lanes beyond the checkpoint basis vocabulary needed here
- query-runtime semantics, delivery policy, or subscription fanout behavior
- schema compatibility policy beyond durable persistence of the artifacts and
  typed support-family version handling required to restart honestly

## Durable Support Artifact Authority Model

### Support-Authority Rule

Milestone 7 introduces durable artifact families that are not canonical commit
records and are not mere derived accelerators.

These families are authoritative only in a subordinate sense: they are
authoritative durable families inside the existing authoritative bucket, but
their authority is scoped to support meaning that is itself subordinate to the
canonical commit model.

These families are authoritative for one narrower role:

- proving what schema boundary applied at a durable frontier
- proving what lineage events and historical identity correspondences were
  committed
- proving what durable cursor position a subscriber or continuation lane had
  acknowledged
- proving what embedded checkpoint identity, classification, and basis were
  durably accepted by the store

They are not allowed to redefine:

- canonical commit meaning
- branch-head truth
- replay legality
- query or delivery semantics above the store

Required classification rule:

- canonical commit families remain `Authoritative`
- schema-boundary, lineage, durable cursor, subscriber checkpoint, and embedded
  checkpoint families are authoritative durable families with
  `support_authority_kind`, not a fourth top-level durability category
- snapshots and later acceleration families remain `DerivedDurable`
- in-memory session cursors, subscriber transports, and caller-local resume
  hints remain `Ephemeral`

If a new artifact introduced here cannot be placed into one of those buckets
explicitly, the design is incomplete.

### Support-Authority Taxonomy Compatibility Rule

Milestone 7 must not create a second top-level authority taxonomy that fights
the taxonomy already defined in the vision.

Required rule:

- all Milestone 7 durable families still classify at the top level as
  `Authoritative` or `DerivedDurable` or `Ephemeral`
- Milestone 7 adds only a subordinate kind field for authoritative support
  families, such as:
  - `CanonicalCommitAuthority`
  - `SchemaBoundaryAuthority`
  - `LineageAuthority`
  - `CursorAuthority`
  - `EmbeddedCheckpointAuthority`
- retention, rebuild, and recovery precedence continue to use the top-level
  authoritative-versus-derived distinction first
- within the authoritative tier, canonical commit authority outranks support
  authority when semantic truth is disputed

This prevents the naive trap where a future implementation invents a separate
"support truth store" with its own rebuild and retention semantics.

### Commit-Coupled Support Publication Rule

Schema-boundary and lineage support artifacts are not allowed to appear through
background extraction or best-effort post-processing.

Required rule:

- if a canonical commit envelope contains schema-boundary or lineage artifacts,
  the durable support family records derived from those artifacts must publish
  in the same admitted durable publication unit as the canonical commit append
- the commit append and its support-artifact append either become durably
  visible together or remain unpublished together
- restart is not allowed to treat asynchronously backfilled support artifacts
  as ordinary success for that commit

Allowed degraded outcome:

- if Milestone 3.5/3.6 recovery finds canonical commit authority without its
  required support-authority companions, restart must classify this as typed
  degraded support recovery rather than silently reconstructing "something
  close enough" from commit payload scans

This is the anti-"we will extract support artifacts later" rule.

### Exactly-Once Support Publication Rule

Commit-coupled publication is not enough if crash-retry lanes can duplicate or
rename support artifacts while still claiming semantic parity.

Required rule:

- schema-boundary, lineage, cursor-identity, and subscriber-checkpoint artifact
  identities must be deterministic from their admitted basis
- retry, reopen, replay, export/import restore, and recovery rebuild must not
  mint a second durable support artifact for the same admitted support fact
- the same canonical history plus the same admitted support facts must converge
  to the same support artifact identities and digests across equivalent lanes

Disallowed loophole:

- "the artifact content is equivalent even though the persisted support ids are
  different after retry" is out of spec

This is the anti-duplicate-support-publication rule.

### Schema Boundary Rule

Schema-boundary meaning must survive restart as its own durable surface rather
than remaining implicit inside commit payloads.

Milestone 7 therefore requires explicit durable schema-boundary records that
preserve at minimum:

- schema version identity
- transition, continuation, or reconciliation artifact identity when present
- the canonical commit frontier at which the schema boundary became active
- the descriptor semantics version required to interpret the boundary
- the digest or authority basis tying the schema artifact back to canonical
  truth

Rules:

- schema-boundary fetch must not require replaying arbitrary commit history
  just to answer "what schema boundary was active here?"
- schema-boundary artifacts remain runtime-authored; store persists and indexes
  them faithfully
- unsupported schema-boundary family versions must fail explicitly and typed

This is the anti-"schema truth only lives in buried commit JSON" line.

### Lineage Durability Rule

Historical identity resolution must survive restart as a durable family, not as
best-effort replay side effect.

Milestone 7 therefore requires durable persistence of:

- lineage event identity
- lineage event ordering within the canonical commit that produced them
- lineage digest basis and event-batch digest basis
- decision-log digest basis or equivalent identity-resolution support basis
- the canonical commit frontier and branch context that admitted the lineage
  change

Rules:

- historical identity resolution queries must be answerable from durable
  lineage artifacts plus canonical authority, not only by replaying all commits
- lineage persistence may accelerate lookup, but it may not redefine runtime
  lineage semantics
- lineage records must remain replay-consistent with the canonical commit that
  produced them

This is the anti-"lineage cache that forgot its basis" line.

### Cursor Position Rule

Durable cursor position must be an explicit support-authority artifact.

Minimum cursor fields:

- `CursorId`
- `SubscriberId` or equivalent durable consumer identity
- `CursorBasisBranchId`
- `CursorBasisCommitId` or equivalent committed frontier
- `CursorSchemaBoundaryId` or equivalent schema interpretation basis
- `CursorCheckpointSequence` or monotonic advancement token
- `CursorArtifactDigest`

Minimum durable cursor identity basis:

- `CursorId`
- `SubscriberId`
- `DeclaredBranchScope`
- `DeclaredFeedShapeId` or equivalent stable continuation-shape identity
- `DeclaredSchemaInterpretationId`
- `CursorSemanticsVersion`

Rules:

- cursor advancement must bind to one exact canonical frontier or declared
  checkpoint, not to "whatever was latest when the fetch finished"
- cursor resume must return a typed durable basis that later continuation can
  consume; it may not only return raw offsets
- cursor progression must be monotonic under its declared identity basis
- cursor writes must be part of an admitted durable publication unit when the
  store claims the subscriber has durably acknowledged work
- cursor rollback, overwrite, or branch drift must fail explicitly and typed
- changing branch scope, feed shape, schema interpretation basis, or cursor
  semantics version produces a different durable cursor identity rather than a
  mutable in-place reinterpretation of the same cursor

Admitted advancement relation:

- `AdvanceCursorSameIdentity`:
  same durable cursor identity, same branch scope, same feed shape, same schema
  interpretation basis family, later or equal admitted frontier
- `AdvanceCursorToEquivalentSchemaBoundary`:
  same durable cursor identity, same branch scope, same feed shape, explicit
  compatible schema-boundary transition admitted by this milestone

Rejected advancement relation:

- same cursor id but different branch scope
- same cursor id but different feed shape or narrowing basis
- same cursor id but incompatible schema interpretation basis
- frontier regression under the same durable cursor identity

### Cursor Equivalence Contract Rule

Milestone 7 must declare cursor sameness mechanically, not rhetorically.

Required rule:

- every durable cursor family must declare the exact equivalence basis that
  justifies resume:
  - consumer identity
  - branch scope
  - feed shape
  - schema interpretation basis
  - cursor semantics version
- resume is legal only when the requesting continuation shape is equal to the
  persisted cursor's declared equivalence basis
- if a higher layer changes narrowing, delivery semantics, or subscriber shape,
  it must mint a new durable cursor identity rather than mutating the old one

This is the anti-"same subscriber, probably same cursor" trap.

Milestone 7 is not yet the full live-query program, but it must already make
cursor truth exact enough that Milestone 8 can consume it honestly.

### Embedded Checkpoint Rule

Milestone 2 established that embedded checkpoints may be durably persisted
without becoming canonical authority. Milestone 7 hardens that seam into a real
artifact family.

Every embedded checkpoint must durably preserve:

- checkpoint identity
- source runtime identity
- checkpoint classification
- optional basis branch and basis commit identity
- contained canonical commit linkage when included
- metadata identity or digest basis sufficient for operator and certification
  inspection

Rules:

- checkpoint classification remains explicit:
  - `DerivedDurable`
  - `Ephemeral`
- checkpoints may carry canonical commits only through the canonical append
  path; the checkpoint record itself never becomes authoritative commit truth
- fetching a persisted checkpoint must not require caller-side reconstruction
  from ambient runtime memory
- if a checkpoint basis points at a branch or frontier that cannot be
  reconciled with canonical durable truth, restart or fetch must fail
  explicitly and typed

This is the anti-"checkpoint file with vibes" rule.

### Embedded Checkpoint Shape Rule

Milestone 7 must not let embedded checkpoints collapse into one loose record
with runtime-only invariants.

Required type-level distinction:

- basis-bearing checkpoints and basis-free checkpoints are different proof
  types
- `DerivedDurable` checkpoints and `Ephemeral` checkpoints are different proof
  types or phantom-tagged instantiations
- checkpoints that contain canonical commits and checkpoints that contain no
  canonical commits are different proof types or phantom-tagged instantiations

Representative shape:

```rust
pub struct BasisFreeCheckpoint<C, K> { ... }
pub struct BasisBoundCheckpoint<B, C, K> { ... }

pub enum CheckpointKind {
    DerivedDurable,
    Ephemeral,
}
```

The exact encoding can vary, but the compiler must be able to see the semantic
differences that matter.

### Restart Reconstruction Rule

Restart, rebuild, and recovery must preserve support-artifact conclusions as
their own durable program.

Required rule:

- if canonical commit history survives but the store cannot localize the
  schema-boundary, lineage, cursor, or checkpoint support artifacts needed to
  answer support queries honestly, the store must fail typed or mark degraded
  support truth explicitly rather than bluffing a clean resume

Recovery precedence rules from Milestone 3.5 and 3.6 apply here:

- canonical commit families outrank support-authority families for semantic
  truth
- support-authority families outrank derived accelerators for support truth
- derived accelerators may rebuild from canonical plus support-authority bases;
  they may not replace them

This is what makes support-artifact restart a real contract instead of a quiet
best effort.

## Proof-Carrying Artifact Pipeline

Law 41 is load-bearing here.

Minimum intended proof chain:

- `ObservedRuntimeSchemaOrLineageArtifact`
- `CanonicalSupportArtifact`
- `VerifiedSupportArtifactAppend`
- `PersistedSupportArtifact`
- `FetchedSupportArtifact`
- `ResumeAdmittedCursor`
- `ResolvedHistoricalIdentity`
- `VerifiedEmbeddedCheckpoint`

Rules:

- each later phase consumes the immediately prior proof-bearing type
- constructors for proof-bearing support types must be crate-sealed
- fields that encode basis identity, digest basis, and monotonic cursor proofs
  must remain private
- cursor resume planning must not accept raw backend rows or caller-local
  offsets where a `PersistedSupportArtifact` or `ResumeAdmittedCursor` proof
  should exist
- embedded checkpoint inspection must consume a verified durable checkpoint
  proof, not unchecked metadata blobs
- support-artifact append for schema and lineage must consume a
  `CommitCoupledSupportAppendWitness` proving the canonical commit append and
  support append still belong to the same publication unit
- cursor acknowledgment must consume a `ResumeAdmittedCursor` plus an
  `AdvanceCursorWitness` rather than re-deciding sameness from raw request data

Representative witness types:

```rust
pub struct CommitCoupledSupportAppendWitness { ... }
pub struct AdvanceCursorWitness { ... }
pub struct BasisBoundCheckpointWitness { ... }
```

Representative progression:

```rust
pub struct CanonicalSchemaBoundaryArtifact { ... }
pub struct PersistedLineageArtifact { ... }
pub struct PersistedCursorCheckpoint { ... }
pub struct ResumeAdmittedCursor { ... }
pub struct PersistedEmbeddedCheckpoint { ... }
```

The exact type names may evolve, but the proof-carrying shape may not.

## Public Surface

Milestone 7 must expose store-owned vocabulary for support-artifact durability.

Representative surface:

```rust
pub struct CursorResumeRequest { ... }
pub struct CursorResumePlan { ... }
pub struct CursorResumeOutcome { ... }
pub struct HistoricalIdentityRequest { ... }
pub struct EmbeddedCheckpointFetchRequest { ... }

impl WorthStore {
    pub fn fetch_schema_boundary(
        &self,
        commit_id: CommitId,
    ) -> Result<FetchedSchemaBoundaryArtifact, StoreError>;

    pub fn fetch_lineage_history(
        &self,
        request: HistoricalIdentityRequest,
    ) -> Result<HistoricalIdentityResolution, StoreError>;

    pub fn plan_cursor_resume(
        &self,
        request: CursorResumeRequest,
    ) -> Result<CursorResumePlan, StoreError>;

    pub fn acknowledge_cursor_progress(
        &mut self,
        progress: CursorProgressAck,
    ) -> Result<PersistedCursorCheckpoint, StoreError>;

    pub fn fetch_embedded_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<PersistedEmbeddedCheckpoint, StoreError>;
}
```

Surface rules:

- support-artifact APIs must expose branch/frontier/schema/lineage/cursor
  vocabulary directly
- no public support-artifact API may traffic in backend-native keys or row
  layouts
- cursor planning and cursor acknowledgment should remain distinct public
  concepts if they need different proof boundaries
- embedded checkpoint fetch should return a store-owned checkpoint proof type,
  not the raw persistence record struct

## Required Internal Subsystems

- `support/schema/`
  schema-boundary record persistence, fetch, version handling, and restart
  reconstruction
- `support/lineage/`
  lineage event persistence, digest basis, and historical identity resolution
- `support/cursors/`
  durable cursor identity, monotonic advancement, resume planning, and
  checkpoint persistence
- `support/checkpoints/`
  embedded checkpoint classification, basis linkage, contained-commit linkage,
  and fetch verification
- `support/restart/`
  support-artifact recovery, precedence, degraded-state classification, and
  restart reconstruction
- `support/evidence/`
  Milestone 7 certification bundles, support-artifact diagnostics, and counter
  reporting
- `backend/`
  support-family persistence mechanics without owning support semantics

This keeps schema, lineage, cursors, and checkpoints from collapsing into one
catch-all persistence helper.

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| support-artifact taxonomy remains subordinate to the existing authoritative-versus-derived taxonomy | support-artifact classification | `support/restart/` and `support/evidence/` | `SupportAuthorityTaxonomyViolation` | `artifact_digest` and `diagnostics_digest` |
| schema and lineage support artifacts publish in the same durable unit as the canonical commit that introduced them | support append publication | `support/schema/`, `support/lineage/`, and publication subsystem | `CommitSupportPublicationGap` | `history_digest`, `artifact_digest`, and `failure_digest` |
| schema-boundary artifact binds to one exact canonical frontier and semantics version | support-artifact append verification | `support/schema/` | `SchemaBoundaryBasisMismatch` or `SchemaBoundaryVersionUnsupported` | `artifact_digest` and `history_digest` |
| lineage artifacts remain replay-consistent with canonical commits | lineage append verification | `support/lineage/` | `LineageArtifactDrift` | `history_digest` and `replay_digest` |
| durable cursor advancement is monotonic and frontier-explicit | cursor acknowledgment | `support/cursors/` | `CursorRegression` or `CursorBasisMismatch` | `artifact_digest` and `diagnostics_digest` |
| durable cursor sameness is defined by identity basis rather than subscriber convention | cursor identity admission | `support/cursors/` | `CursorEquivalenceViolation` | `artifact_digest`, `replay_digest`, and `diagnostics_digest` |
| cursor resume never returns a branch or schema basis different from the persisted durable cursor family | cursor resume planning | `support/cursors/` | `CursorResumeAmbiguous` or `CursorSchemaBasisMismatch` | `replay_digest` |
| embedded checkpoints never become authoritative commits | checkpoint classification | `support/checkpoints/` | `EmbeddedCheckpointAuthorityViolation` | `artifact_digest` and `diagnostics_digest` |
| embedded checkpoint semantic shape is enforced by distinct proof-bearing types rather than optional-field conventions | checkpoint construction | `support/checkpoints/` | `CheckpointShapeViolation` | `artifact_digest` and compile-fail boundary tests |
| checkpoint basis and contained-commit linkage remain durably queryable after restart | checkpoint fetch verification | `support/checkpoints/` | `CheckpointBasisMissing` or `CheckpointContainedCommitMissing` | `artifact_digest` |
| support-artifact restart does not bluff clean resume when required support truth is missing or inconsistent | support restart reconstruction | `support/restart/` | `SupportArtifactRecoveryGap` | `diagnostics_digest` and `failure_digest` |

## Failure Taxonomy

Milestone 7 must ship explicit typed failures at minimum covering:

- `SchemaBoundaryBasisMismatch`
- `SchemaBoundaryVersionUnsupported`
- `SchemaBoundaryArtifactMissing`
- `SupportAuthorityTaxonomyViolation`
- `CommitSupportPublicationGap`
- `LineageArtifactDrift`
- `LineageArtifactMissing`
- `HistoricalIdentityResolutionGap`
- `CursorBasisMismatch`
- `CursorSchemaBasisMismatch`
- `CursorRegression`
- `CursorEquivalenceViolation`
- `CursorResumeAmbiguous`
- `CursorCheckpointMissing`
- `SubscriberCheckpointConflict`
- `EmbeddedCheckpointAuthorityViolation`
- `CheckpointShapeViolation`
- `CheckpointBasisMissing`
- `CheckpointContainedCommitMissing`
- `CheckpointClassificationUnsupported`
- `SupportArtifactRecoveryGap`

Rules:

- public failures must be store-owned semantic failures, not backend-driver
  jargon
- typed failures must localize the affected cursor, checkpoint, commit,
  branch, schema boundary, or lineage family where possible
- support-summary failures must preserve the affected support family in their
  recovery and evidence surfaces rather than collapsing back to a generic
  summary bucket
- degraded support-artifact outcomes must remain explicit rather than being
  collapsed into clean success

## Complexity Contracts

Milestone 7 must encode performance in the architecture itself, not as a later
storage-engine optimization pass.

That means:

- hot-path identities must be first-class durable keys
- traversal direction must be declared before tables are chosen
- expensive facts proven at append time must be carried forward as summaries
  rather than rediscovered on resume or restart
- APIs must force callers to declare enough basis information that the store
  can stay narrow

### Performance Encoding Rules

#### Identity-Keyed Resume Rule

Cursor resume must be architected as identity lookup, not heuristic search.

Required rule:

- the durable cursor identity basis defined earlier in this spec is also the
  hot-path lookup key for resume
- resume planning must begin from a direct lookup on durable cursor identity,
  not from scanning subscriber history to find something compatible
- any API that asks the store to infer "the right cursor" from partial caller
  input is out of spec

Target architectural consequence:

- admitted cursor resume is `O(1)` or `O(log n)` in cursor identity lookup,
  plus continuation validation work for the declared frontier delta
- admitted cursor resume is not `O(total_subscriber_cursors)` or
  `O(total_branch_history)`

#### Directional Access Rule

Lineage and support-artifact lookup must encode traversal direction
architecturally.

Required directional surfaces:

- schema-boundary lookup:
  - `frontier -> active schema boundary`
  - `schema boundary id -> boundary artifact`
- lineage lookup:
  - `lineage event id -> committed lineage artifact`
  - `identity handle -> lineage history neighborhood`
- cursor lookup:
  - `durable cursor identity -> latest persisted cursor artifact`
  - `(cursor identity, checkpoint sequence) -> historical cursor checkpoint`
- checkpoint lookup:
  - `checkpoint id -> checkpoint artifact`
  - `basis frontier -> checkpoints anchored to that frontier` where admitted

The spec must not allow one normalized generic support table with later
"recovered" access patterns. If the traversal direction is load-bearing, the
storage shape must admit it directly.

#### Commit-Time Summary Rule

Expensive support facts proven during canonical commit append must be carried
forward as immutable summaries.

Milestone 7 therefore requires one append-time summary family:

- `CommitSupportSummary`

Minimum contents:

- whether the commit emitted schema-boundary artifacts
- whether the commit emitted lineage artifacts
- emitted support artifact identities
- affected branch/frontier scope
- cursor-relevant continuation frontier markers when admitted

Rules:

- summary derivation happens once at the commit-coupled support append boundary
- restart, support recovery, and cursor planning may consume the summary
- later phases may not re-scan the full commit payload to rediscover facts the
  summary already proved inside the same trust boundary

This is the anti-repeated-rediscovery rule for Milestone 7.

#### Summary Family Localization Rule

Commit support summaries are allowed to summarize multiple support families, but
recovery and certification are not allowed to collapse family-localized failure
meaning back into one generic summary bucket.

Required rule:

- if summary verification discovers a missing or drifting schema companion, the
  degraded recovery surface must emit a schema-scoped support entry
- if summary verification discovers a missing or drifting lineage companion, the
  degraded recovery surface must emit a lineage-scoped support entry
- if one summary proves multiple family failures, recovery may emit multiple
  support entries for the same commit, one per affected family
- summary-scoped support identities used in recovery and evidence must encode
  both the family and the commit identity, for example:
  - `commit-support-summary:schema:commit:<id>`
  - `commit-support-summary:lineage:commit:<id>`

This prevents the naive trap where commit-coupled support verification passes
through a family-ambiguous "support gap" label that teaches the wrong mental
model to operators and later milestone authors.

#### Narrow API Rule

Public surfaces must force enough caller intent that the store can remain
mechanically narrow.

Required examples:

- prefer `plan_cursor_resume(request_with_declared_identity_and_frontier)` over
  `resume_latest_for_subscriber(subscriber_id)`
- prefer `fetch_schema_boundary(commit_id)` or an equivalent explicit frontier
  surface over `fetch_current_schema(branch_id)`
- prefer `fetch_lineage_history(request_with_identity_scope)` over broad
  "give me lineage for this branch" convenience APIs

If a cheap-looking API would force the store to guess branch scope, frontier,
schema interpretation, or continuation shape, the API is architecturally
dishonest and must not ship.

#### Derived Summary Non-Authority Rule

Milestone 7 may admit derived fast-path summaries, but they must stay derived.

Admitted derived summary families:

- current schema-boundary pointers per branch
- latest durable cursor pointers per cursor identity
- lineage neighborhood summaries

Rules:

- these may accelerate hot reads
- they must rebuild from canonical plus support-authority artifacts alone
- if they drift, the authoritative support artifact wins and the summary is
  rebuilt or rejected

This prevents the performance layer from becoming shadow authority.

### Required Physical Access Structures

Milestone 7 must name the access structures it expects implementations to
provide, even if exact backend syntax varies.

Minimum required access structures:

- cursor identity index over:
  - durable cursor identity tuple
- cursor checkpoint ordering index over:
  - `(cursor identity, checkpoint sequence)`
- schema-boundary frontier index over:
  - `(branch scope, frontier commit id)` or an equivalent exact frontier key
- schema-boundary identity index over:
  - `schema boundary artifact id`
- lineage event identity index over:
  - `lineage event id`
- lineage neighborhood index over:
  - durable identity handle or correspondence handle used for historical
    identity resolution
- embedded checkpoint identity index over:
  - `checkpoint id`
- optional checkpoint basis index over:
  - `(basis branch, basis commit)` when checkpoint-by-basis lookup is admitted

Rules:

- implementations may add richer indexes
- implementations may not omit the above and replace them with full scans plus
  filters while still claiming the Milestone 7 complexity contracts
- if one backend cannot honestly provide an admitted access structure, the
  complexity contract for that path must be marked `Debt` and surfaced in
  certification output

Required certification consequence:

- Milestone 7 evidence must publish a machine-checkable complexity-status
  surface for every named hot path and backend lane, at minimum:
  - `schema_boundary_fetch`
  - `lineage_lookup`
  - `cursor_resume`
  - `embedded_checkpoint_fetch`
  - `commit_coupled_support_publication`
  - `cursor_identity_admission`
- each path must declare `Verified` or `Debt`
- any path marked `Debt` must name the missing access structure or unresolved
  cost honesty gap

Minimum contracts:

- schema-boundary fetch cost is proportional to:
  - one frontier-index lookup
  - support-artifact rows read for the exact requested frontier
  - compatibility checks required for the admitted support family
- lineage lookup cost is proportional to:
  - one lineage-neighborhood index lookup
  - lineage events examined in the exact requested historical resolution scope
  - digest-basis validations required for the admitted lineage family
- cursor resume cost is proportional to:
  - one durable cursor identity lookup
  - persisted cursor checkpoints consulted for the exact requested cursor
    identity
  - canonical continuation frontier distance needed to validate the resume
    basis
- embedded checkpoint fetch cost is proportional to:
  - one checkpoint identity lookup
  - one checkpoint record read
  - contained-commit linkage reads
  - basis verification work
- commit-coupled support publication cost is proportional to:
  - schema and lineage support artifacts emitted by the canonical commit
  - not total historical support-artifact volume
- cursor identity admission cost is proportional to:
  - one persisted cursor identity read
  - equivalence-basis comparisons
  - not replay breadth for unrelated branches or feeds

Minimum counters:

- `schema_boundary_fetch_count`
- `schema_boundary_index_lookup_count`
- `schema_boundary_rows_read`
- `schema_boundary_resolution_count`
- `commit_support_publication_count`
- `commit_support_publication_gap_count`
- `commit_support_summary_build_count`
- `lineage_lookup_count`
- `lineage_identity_lookup_count`
- `lineage_event_rows_read`
- `lineage_resolution_breadth`
- `cursor_resume_count`
- `cursor_identity_lookup_count`
- `cursor_resume_support_rows_read`
- `cursor_resume_step_count`
- `cursor_ack_count`
- `cursor_equivalence_reject_count`
- `cursor_regression_reject_count`
- `subscriber_checkpoint_write_count`
- `embedded_checkpoint_fetch_count`
- `embedded_checkpoint_index_lookup_count`
- `embedded_checkpoint_basis_read_count`
- `checkpoint_shape_reject_count`
- `support_artifact_recovery_gap_count`

Milestone 7 may add richer counters, but it may not hide resume or historical
resolution breadth.

## Phases

### Phase 1: Lock Support-Artifact Vocabulary And Authority Boundaries

Required work:

- define `SupportAuthority` artifact classification and its relationship to
  canonical authority and derived durable families
- define subordinate support-authority kinds inside the existing top-level
  authoritative taxonomy
- define schema-boundary, lineage, cursor, subscriber checkpoint, and embedded
  checkpoint identity vocabularies
- define basis rules for cursor and checkpoint artifacts
- define cursor equivalence basis and admitted advancement relations
- define proof-bearing support-artifact phase types
- define compile-time witness types for commit-coupled support publication,
  cursor advancement, and basis-bound checkpoint construction
- define support-artifact recovery and degraded-state vocabulary

Exit condition:

- support artifacts have one explicit architectural role
- restart and resume no longer depend on ambient session vocabulary
- Milestone 7 no longer risks collapsing support truth into either canonical
  commits or derived caches

### Phase 2: Persist Schema Boundary And Lineage Artifact Families

Required work:

- persist schema-boundary artifact records and fetch surfaces
- persist lineage event families and their digest basis
- require same-publication-unit persistence with the canonical commit that
  introduced the support artifacts
- verify replay consistency between canonical commits and persisted support
  artifacts
- expose typed schema and lineage failures
- emit schema-boundary and lineage counters

Exit condition:

- schema and lineage support truth is durably queryable after restart
- historical identity resolution no longer depends on replaying arbitrary
  history by default

### Phase 3: Persist Durable Cursor And Subscriber Checkpoint Families

Required work:

- persist cursor identities, basis identity, and advancement sequence
- persist subscriber checkpoint records with explicit durable linkage
- enforce monotonic cursor advancement and basis legality
- enforce cursor identity equivalence mechanically so branch or feed drift
  cannot reuse the same durable cursor
- separate cursor planning from cursor acknowledgment when proof boundaries
  differ
- emit exact resume and acknowledgment counters

Exit condition:

- the store can durably answer "where may this subscriber resume from?"
- cursor truth is frontier-explicit and monotonic instead of ambient

### Phase 4: Harden Embedded Checkpoint Artifact Persistence

Required work:

- upgrade embedded checkpoint persistence from a Milestone 2 mode seam to a
  fully verified support-artifact family
- persist exact basis linkage, classification, and contained-commit linkage
- replace optional-field checkpoint shape conventions with proof-bearing
  classification and basis witnesses
- expose typed checkpoint fetch and mismatch failures
- emit checkpoint read and basis-verification counters

Exit condition:

- checkpoints are durable, inspectable, and restart-safe
- checkpoints remain explicitly non-authoritative

### Phase 5: Integrate Durable Restart, Resume, And Historical Resolution

Required work:

- integrate support-artifact recovery and restart reconstruction with Milestone
  3.5/3.6 precedence and degraded-state rules
- classify commit-present but support-family-missing lanes as typed degraded
  support recovery, not ordinary success
- make cursor resume consume persisted support artifacts rather than caller
  memory
- make historical identity resolution consume persisted lineage support truth
- classify missing or inconsistent support artifacts as typed degraded restart
  outcomes where appropriate

Exit condition:

- restart can reconstruct support-artifact conclusions honestly
- missing support truth cannot masquerade as clean continuation

### Phase 6: Prove Schema, Lineage, Cursor, And Checkpoint Durability

Required work:

- run the Milestone 7 named suite:
  `Schema/Lineage/Cursor Durability Test`
- compare restart, replay, and resume lanes against a control reconstruction
  lane
- prove deterministic cursor resume and historical identity resolution
- prove commit-coupled support publication parity and typed degraded recovery
  for support-publication gaps
- prove checkpoint compile-time shape boundaries with UI or compile-fail tests
- emit machine-checkable history, artifact, replay, diagnostics, and counter
  bundles

Exit condition:

- support-artifact restart parity is proven
- cursor resume is deterministic
- embedded checkpoint persistence remains durable and non-authoritative

## Must Ship

- explicit durable schema-boundary artifact families
- explicit durable lineage artifact families and historical identity support
- same-publication-unit persistence rules for commit-coupled schema and lineage
  support artifacts
- explicit durable cursor and subscriber checkpoint families
- monotonic transactional cursor advancement
- explicit cursor equivalence contract and enforcement surfaces
- hardened embedded checkpoint durability with basis and contained-commit
  linkage
- proof-bearing checkpoint shape enforcement
- restart and recovery rules for support-artifact families
- typed schema, lineage, cursor, and checkpoint failures
- machine-checkable Milestone 7 certification output

## Must Preserve

- canonical commit envelopes remain the semantic truth authority
- schema and lineage semantics remain owned by `worth-relational`
- cursor meaning, delivery policy, and query semantics remain above the store
- embedded checkpoints remain non-authoritative support artifacts
- Milestone 5 physical delta work may change cost later, not support-artifact
  meaning now
- derived accelerators remain rebuildable from canonical plus support-artifact
  bases and may not replace them

## Acceptance Evidence

Milestone 7 is complete only when the store satisfies the named Milestone 7
suite:

- `Schema/Lineage/Cursor Durability Test`

Required machine-checkable outputs:

- `history_digest`
- `artifact_digest`
- `replay_digest`
- `support_truth_digest`
- `diagnostics_digest`
- `certification_summary`
- `counter_contract`
- `counter_snapshot`

Evidence separation rule:

- `support_truth_digest` is the canonical parity surface for support-artifact
  truth across equivalent lanes
- `diagnostics_digest` is the telemetry-bearing diagnostics surface and may
  legitimately diverge across equivalent lanes when lane-local work differs
- `counter_contract` must contain the Milestone 7 named hot-path counters
  needed to judge continuation and recovery cost honestly without requiring
  consumers to interpret the full global counter snapshot
- `counter_snapshot` remains the raw store-wide accounting surface and must not
  be used as the sole Milestone 7 cost contract

Milestone-specific proof obligations:

- schema-boundary conclusions survive restart
- historical identity resolution survives restart without semantic drift
- schema and lineage support publication remains commit-coupled rather than
  post-hoc extracted
- support publication remains exactly-once across retry, replay, and equivalent
  restore lanes
- durable cursor resume is deterministic for equivalent lanes
- cross-branch, cross-feed-shape, and incompatible-schema cursor reuse is
  rejected explicitly
- embedded checkpoints persist and reload without becoming authority
- invalid checkpoint semantic shapes are rejected mechanically
- missing or inconsistent support artifacts fail typed rather than broadening
  into ambient continuation
- support-summary failures localize the affected family rather than surfacing as
  family-ambiguous gaps

Milestone 7 is not closed by "cursor resumed once" or "checkpoint fetched
successfully" tests.

## Architectural Notes

- The smart abstraction is not "metadata persistence." The smart abstraction is
  one support-authority layer that durably preserves schema, lineage, cursor,
  and checkpoint truth without challenging canonical commit authority.
- Milestone 7 should lean on the existing embedded checkpoint seam in the code,
  but it must harden that seam into proof-bearing support artifacts rather than
  leaving it as a convenience record path.
- Because canonical commit envelopes already carry schema and lineage material,
  Milestone 7 should prefer faithful durable extraction/indexing of those
  runtime-authored artifacts over store-invented reinterpretation.
- Cursor planning and cursor acknowledgment should remain separate if that is
  what keeps monotonic durable progress mechanically enforceable.
- Milestone 8 should consume the artifact families defined here rather than
  redefining what a basis, cursor, or checkpoint means.

## Sequencing Notes

This milestone belongs early because it defines the durable support truth that
later live-query, sync, compatibility, and repair programs depend on.

- It can begin once Milestone 1 and Milestone 2 freeze canonical authority and
  mode boundaries.
- Its durable restart integration depends on Milestone 3 and Milestone 3.5/3.6
  vocabulary for publication, recovery, and degraded-state honesty.
- It should proceed concurrently with Milestone 5 rather than waiting on
  branch-delta physical storage, because Milestone 7 is about support-artifact
  meaning, not physical delta layout.
- Milestone 8 should treat Milestone 7 as a prerequisite on the support-truth
  side even if some Milestone 8 work overlaps with late Milestone 6 physical
  narrowing.

If Milestone 7 is weak, Milestone 8 will end up faking durable sync from
ambient cursor memory and implicit basis assumptions. This spec exists to stop
that before it starts.
