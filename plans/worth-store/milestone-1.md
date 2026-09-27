# Milestone 1 Engineering Spec: Canonical Commit Persistence And Artifact Authority

> **Status:** Implemented and closed in
> [milestone-1-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-1-closeout.md)
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Primary architectural driver:** lock one canonical durable artifact model before WAL, snapshots, delta layering, compaction, replication, or derived storage are allowed to exist
>
> **Companion docs:**
> - [MENTALITY.md](/Users/Esther/Documents/Programming/worth_workspace/worth/docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](/Users/Esther/Documents/Programming/worth_workspace/worth/docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](/Users/Esther/Documents/Programming/worth_workspace/worth/docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](/Users/Esther/Documents/Programming/worth_workspace/worth/docs/coding-guidelines/domain_laws.md)
> - [worth_relational_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-relational/worth_relational_vision.md)
> - [worth_relational_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-relational/worth_relational_roadmap.md)
> - [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
> - [milestone-1.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-runtime-bridge/milestone-1.md)
> - [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
> - [worth_signal_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth_signal/worth_signal_vision.md)
> - [worth_signals2.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth_signal/worth_signals2.md)

## Goal

Define one canonical durable artifact model for Worth Store and make it the
only semantically authoritative persistence surface.

## Why This Milestone Exists

Milestone 1 is not "save commits somewhere."

It is the milestone that decides whether `worth-store` becomes:

- a real durability authority layer for the Worth runtime, or
- a generic persistence wrapper that quietly invents a second semantic model

Everything later depends on this boundary being honest:

- WAL must log toward the same canonical artifact model
- snapshots must be provably derived from it
- delta layering must compress it without replacing it
- live-query continuation must resume from durable positions defined against it
- replication and capsules must ship it rather than backend-local layout

If Milestone 1 is vague, every later milestone will end up renegotiating what
"the real store truth" actually is. That is exactly the failure this milestone
exists to prevent.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is structural honesty under pressure.
  The spec therefore starts from the hostile failure mode and treats proof,
  replay, and closeout evidence as the product, not documentation garnish.
- `arch_laws.md`
  The most important thing it protects here is machine-enforced boundaries.
  Law 41 matters especially: every store phase must produce a stronger proof
  type than the one before it, and no later phase may accept a weaker raw
  artifact once a stronger proven artifact exists.
- `perf_laws.md`
  The most important thing it protects is visible cost and bounded hot paths.
  Milestone 1 therefore names append/fetch complexity, canonicalization work,
  and exact counters now instead of pretending cost contracts can wait for
  later scale work.
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped structure.
  The store must be decomposed by authority, canonicalization, branch history,
  and backend responsibilities rather than by generic layers or catch-all
  storage helpers.
- `worth_store_vision.md`
  The most important thing it protects is that the runtime owns semantics while
  store owns survival. Milestone 1 must therefore persist exactly the runtime's
  canonical artifacts and refuse to invent a second truth representation.
- `worth_store_roadmap.md`
  The most important thing it protects is development order. Milestone 1
  belongs first because every later artifact family, recovery path, and
  optimization depends on a frozen authoritative artifact model.
- `worth-store/test-requirements.md`
  The most important thing it protects is beta-grade proof. Milestone 1 is not
  closed until the `Durable Artifact Authority Equivalence Test` proves backend
  parity and rebuild parity with machine-checkable artifact digests.
- `worth_relational_vision.md`
  The most important thing it protects is that canonical commit artifacts come
  from the truth runtime, not the store. Store schema must therefore preserve
  commit identity, ordered parents, branch heads, schema boundaries, lineage,
  and replay surfaces without redefining them.
- `worth_relational_roadmap.md`
  The most important thing it protects is serialized authority and replay from
  canonical commit artifacts. Milestone 1 must treat commit envelopes and
  parent ordering as first-class durable records from day one.
- `worth_runtime_bridge_vision.md`
  The most important thing it protects is decoupled protocol boundaries. Store
  artifacts must be clean enough that bridge and server layers can consume
  canonical history without backend leakage or truth/compute fusion.
- `worth-runtime-bridge/milestone-1.md`
  The most important thing it protects is the proof-carrying envelope pipeline.
  Milestone 1 must mirror that rigor: raw runtime output, canonicalized
  envelope, persisted authoritative record, and fetched verified record must be
  different types with sealed transitions.
- `worth_runtime_bridge_roadmap.md`
  The most important thing it protects is that integration consumes canonical
  artifacts rather than host-local glue. Store must publish canonical commit
  records suitable for replay, CDC, and later bridge consumption.
- `worth_signal_vision.md`
  The most important thing it protects is stable snapshot-backed derived work.
  Store must preserve enough branch and commit identity to support later stable
  basis reads without teaching signal anything about backend storage layout.
- `worth_signals2.md`
  The most important thing it protects is replayable, branchable, diagnosable
  execution over stable truth. Store must therefore keep commit and branch
  artifact identity clean and machine-checkable from the beginning.

## Adversarial Constraint

Milestone 1 must survive this hostile condition:

> Two different backends, one original store, and one rebuild path that reads
> only canonical authoritative artifacts must all converge to the same commit
> history, branch heads, parent ordering, and replay-visible truth without
> consulting backend-local layout or derived storage.

Concretely, the design fails if any supported path:

- stores runtime truth in a backend-shaped schema that cannot be expressed back
  as one canonical commit envelope
- treats branch heads, parent order, or artifact digests as advisory metadata
  instead of authoritative durable records
- allows two envelope shapes for the same committed meaning
- allows fetch or replay paths to read backend-local shortcuts rather than
  canonical authoritative records
- lets an embedded backend and a future backend disagree about authoritative
  artifact identity for the same logical history

The hostile question for this milestone is simple:

`if every derived artifact disappeared tomorrow, what exact durable records would still let us prove what committed and in what order?`

## Product Decision Lock

The following decisions are locked in this milestone:

- canonical commit envelopes are the atomic authoritative truth artifact
- ordered parent lists are persisted explicitly, not reconstructed later
- branch records and branch heads are authoritative durable artifacts
- authoritative artifact identity is digest-backed and canonicalized
- one DRY envelope family is used across durable mode, embedded mode,
  replication, replay, and diagnostics base records
- backend implementations may vary physical layout, but not canonical artifact
  meaning, digest basis, or fetch semantics
- Milestone 1 ships with one production-grade embedded backend baseline and one
  backend abstraction owned by `worth-store`

Normative consequence:

- any implementation that stores "whatever the backend finds convenient" and
  reconstructs canonical envelopes on demand is out of spec
- any implementation that exposes public constructors for persisted-authority
  types is out of spec
- any implementation that puts canonical meaning in diagnostics-rich sidecars
  rather than the authoritative record is out of spec

## Scope

### In Scope

- canonical commit envelope intake from runtime-produced artifacts
- explicit authoritative-versus-derived artifact classification
- persistence of:
  - canonical commit envelopes
  - branch records
  - branch heads
  - ordered commit parent edges
  - authoritative artifact digests and canonicalization version
- one production-grade embedded backend baseline
- one backend abstraction whose contract is defined in store-owned terms
- canonical append, fetch, and verification surfaces
- machine-checkable artifact parity and backend equivalence proof surfaces

### Explicitly Out Of Scope

- WAL durability and crash recovery
- snapshot persistence
- delta layering and branch-local physical compression
- compaction, reclaim, and tiering
- replication capsules and import/export
- blob/object storage
- live-query continuation
- schema/lineage/cursor persistence beyond the fields required to keep commit
  envelopes structurally future-compatible

Milestone 1 may reserve fields and identities for later milestones. It may not
pretend those later capabilities are already shipped.

## Canonical Artifact Model

### DRY Envelope Rule

Milestone 1 adopts a strict DRY envelope rule:

`one committed meaning -> one canonical envelope family -> many consumers`

The store must not create separate semantic envelope shapes for:

- append
- fetch
- replay
- replication
- diagnostics
- embedded-mode checkpoint reception

Instead, it must define one canonical envelope family and allow:

- thinner request wrappers around it
- richer derived explanation around it
- backend-local encoding underneath it

The abstraction stays DRY by sharing the semantic envelope, not by sharing one
giant god-type for every transport concern.

That means:

- the canonical envelope type owns identity-bearing committed meaning
- request/response wrappers add orchestration-only fields
- diagnostics records reference canonical artifact identity instead of
  restating committed meaning in a second schema

This is the smart abstraction line for Milestone 1: reuse the envelope
semantics once, keep wrappers narrow, and do not flatten authority and
orchestration into one bag of fields.

### Authoritative Artifact Families In Milestone 1

Milestone 1 authorizes exactly these durable artifact families:

- `CanonicalCommitEnvelope`
  The canonical truth-bearing commit artifact emitted by the runtime and
  accepted by the store after canonicalization and verification.
- `CommitParentRecord`
  One ordered parent edge per parent position for every persisted commit.
- `BranchRecord`
  Stable branch identity and creation metadata.
- `BranchHeadRecord`
  The current authoritative head pointer for each branch.
- `AuthoritativeArtifactDigestRecord`
  Digest and canonicalization metadata for each authoritative durable artifact.

Milestone 1 must classify all of the above as `Authoritative`.

Milestone 1 must explicitly reject storing any of the following as hidden
authority:

- backend-native row layout as the only source of commit meaning
- denormalized summary tables that are not rebuildable from canonical records
- diagnostics-only records that contain authoritative parent or branch meaning

### Canonicalization Rule

Milestone 1 must define canonicalization mechanically, not rhetorically.

For each canonical commit envelope, the store must specify:

- commit identity basis
- branch identity basis
- ordered parent list basis
- canonical patch/body ordering basis
- canonical digest input fields
- explanatory-only fields excluded from digest
- canonicalization version

Canonicalization must be idempotent:

- canonicalizing an already canonical envelope yields the same digest
- two semantically equivalent envelopes canonicalize to byte-equivalent
  authoritative payloads or an explicitly identical logical digest basis

Milestone 1 must assume future schema evolution and therefore include a
`canonicalization_version` field in authoritative digest metadata from day one.

### Canonicalization Ambiguity Checklist

Milestone 1 code is not implementation-ready until it explicitly resolves every
ambiguity class below.

Required ambiguity decisions:

- exact serialization basis used for authoritative digest computation
- ordering rule and comparator for every unordered map, set, or registry
- duplicate body-item collapse rules
- definition of semantic equivalence for canonicalization
- normalization of absent, default, zero, null, and empty values
- handling of unknown reserved fields
- handling of future-added fields so older digest meaning cannot drift
- field ordering for structured payload serialization
- whether explanatory metadata is stripped before digest or carried outside the
  digest basis entirely
- whether canonicalization is byte-level, logical-structure-level, or a staged
  combination of both

Milestone 1 must fail explicitly if any of these ambiguity classes are still
"to be decided later" at implementation time.

Minimum required resolutions:

- all unordered collections must be sorted by a declared canonical comparator
- duplicate body items must either collapse deterministically or fail
  canonically; silent backend-dependent duplicate retention is forbidden
- unknown fields must either:
  - be forbidden for the active canonicalization version, or
  - be explicitly excluded from digest meaning under a declared compatibility
    rule
- future-added fields must not change the digest meaning of older
  canonicalization versions accidentally
- canonicalization must produce the same digest regardless of backend-local row
  order, map iteration order, or insertion order

### Proof-Carrying Artifact Pipeline

Law 41 is load-bearing here. Milestone 1 must encode authority as a proof chain
rather than a convention-heavy storage helper pipeline.

Representative progression:

```rust
pub struct RawRuntimeCommitEnvelope { ... }
pub struct CanonicalizedCommitEnvelope { ... }
pub struct VerifiedAuthoritativeAppend { ... }
pub struct PersistedAuthoritativeCommit { ... }
pub struct FetchedAuthoritativeCommit { ... }
```

Rules:

- `RawRuntimeCommitEnvelope` may contain runtime-produced committed data but no
  store proof
- `CanonicalizedCommitEnvelope` proves canonical ordering, digest basis, and
  artifact classification
- `VerifiedAuthoritativeAppend` proves the append request is complete,
  well-formed, and admissible for authoritative persistence
- `PersistedAuthoritativeCommit` proves the artifact was durably written through
  a backend contract that preserves canonical identity
- `FetchedAuthoritativeCommit` proves the fetched artifact passed verification
  against its authoritative digest record

Mandatory Law 41 consequences:

- constructors for every proof-bearing type are sealed to the proving module
- proof-bearing fields are private
- later phases consume the immediately prior proof type, not a weaker cousin
- branch-head advancement may not accept a raw commit envelope; it must consume
  a persisted authoritative commit proof
- any append or fetch helper that re-validates what the type already proves is
  dead-code smell and should be deleted or the types tightened

Anti-cheat rules:

- exactly one module may mint each proof-bearing type
- persisted-row decoding must terminate in exactly one verification gateway
  before a fetched authoritative proof type is produced
- test helpers may bypass proof transitions only inside dedicated fixture code
  that cannot leak into production constructors
- no `unsafe`, debug bypass, feature-flag bypass, serde hook, or hidden helper
  may construct proof-bearing authoritative types directly
- rehydration from raw persisted rows must never skip the same verification path
  used by normal fetch

## Store Schema And Physical Baseline

### Database Role In Milestone 1

Milestone 1 does not ship "the final store database."

It ships the first honest authoritative persistence substrate whose job is to
preserve canonical runtime artifacts with minimal backend interpretation.

The database role in Milestone 1 is therefore:

- persist authoritative records exactly once
- preserve canonical identities and parent ordering
- support exact fetch and verification
- stay structurally future-compatible with later WAL, snapshot, delta, and
  replication work

It is not yet responsible for:

- operational boundedness under WAL replay
- branch delta compression
- snapshot serving
- derived acceleration

The physical baseline must bias toward boring correctness:

- append-safe authoritative tables
- explicit foreign keys or equivalent integrity constraints
- backend-owned encoding details hidden behind store-owned record types
- no backend-specific semantic columns leaking into the public model

Milestone 1 must default to a relational embedded backend because the first
job is preserving canonical structure, ordered edges, and verifiable digests,
not proving exotic storage layout.

### Required Authoritative Tables

Milestone 1 must persist at least the following authoritative record groups.

`branch_records`

- `branch_id`
- `branch_name` or stable branch label when applicable
- `created_from_commit_id`
- `branch_status`
- `created_at_commit_sequence` or equivalent canonical creation ordering token

`commit_envelopes`

- `commit_id`
- `branch_id`
- `commit_sequence` or equivalent canonical per-store append ordering token
- `canonicalization_version`
- `runtime_schema_version` or compatible schema-boundary placeholder
- `envelope_payload`
- `envelope_digest`

`commit_parent_records`

- `commit_id`
- `parent_position`
- `parent_commit_id`

`branch_head_records`

- `branch_id`
- `head_commit_id`
- `head_commit_digest`
- `head_update_sequence`

`authoritative_artifact_digests`

- `artifact_family`
- `artifact_id`
- `canonicalization_version`
- `digest_algorithm`
- `artifact_digest`

Schema rules:

- ordered parents belong in their own table, not packed into an opaque blob
- branch heads are authoritative current-state pointers and must not be
  implicit "max commit sequence" guesses
- artifact digests must cover every authoritative family, not only commits

The schema must reserve explicit extension points for later milestones:

- commit-basis linkage for snapshots
- schema-boundary records
- lineage-family records
- cursor-family records

Reserved future families must stay absent or null in Milestone 1, not
half-shipped with fake semantics.

### Required Constraints And Indexes

Milestone 1 must enforce at least:

- unique `branch_id`
- unique `commit_id`
- unique `(commit_id, parent_position)`
- unique `branch_id` in `branch_head_records`
- unique `(artifact_family, artifact_id, canonicalization_version)` in digest
  metadata
- foreign-key or equivalent integrity from:
  - `commit_envelopes.branch_id -> branch_records.branch_id`
  - `commit_parent_records.commit_id -> commit_envelopes.commit_id`
  - `commit_parent_records.parent_commit_id -> commit_envelopes.commit_id`
  - `branch_head_records.branch_id -> branch_records.branch_id`
  - `branch_head_records.head_commit_id -> commit_envelopes.commit_id`

Minimum indexes that must exist:

- fetch commit by `commit_id`
- fetch branch head by `branch_id`
- scan commit history by `(branch_id, commit_sequence)`
- fetch parents by `(commit_id, parent_position)`
- fetch digest by `(artifact_family, artifact_id)`

These indexes are still non-authoritative. Their job is exact fetch and
verification support, not semantic reinterpretation.

### Backend Abstraction Baseline

Milestone 1 must introduce one store-owned backend contract that talks in
canonical store nouns, not backend nouns.

Representative shape:

```rust
pub trait AuthoritativeArtifactBackend {
    type Error;

    fn append_commit(
        &mut self,
        append: VerifiedAuthoritativeAppend,
    ) -> Result<PersistedAuthoritativeCommit, Self::Error>;

    fn fetch_commit(
        &self,
        commit_id: CommitId,
    ) -> Result<FetchedAuthoritativeCommit, Self::Error>;

    fn fetch_branch_head(
        &self,
        branch_id: BranchId,
    ) -> Result<FetchedBranchHeadRecord, Self::Error>;
}
```

Rules:

- the trait belongs to `worth-store`
- backend implementations may encode rows differently, but must return the same
  proof-bearing store record types
- append and fetch semantics are defined in terms of authoritative artifacts,
  not SQL statements, files, or driver-local handles
- later backends must prove parity against the embedded baseline, not invent
  new meaning

The default Milestone 1 backend must be a production-grade embedded backend
with exact constraints and deterministic fetch behavior. "Embedded" does not
mean toy.

### Complexity Contracts

Milestone 1 must declare exact hot-path contracts even before later scale
milestones deepen them.

Minimum contracts:

- canonical append cost is proportional to:
  - committed envelope size
  - parent count
  - authoritative artifact family count written for the commit
- canonical fetch cost is proportional to:
  - one commit envelope fetch
  - ordered parent row count for that commit
  - one branch-head fetch where requested
  - one digest verification pass for the fetched authoritative family
- canonicalization cost is proportional to:
  - canonical body item count
  - ordering/dedup work defined by the canonicalization rules

Minimum counters:

- `authoritative_commit_append_count`
- `authoritative_commit_fetch_count`
- `commit_parent_record_write_count`
- `branch_head_write_count`
- `authoritative_digest_write_count`
- `canonicalization_item_count`
- `canonicalization_duplicate_collapse_count`
- `authoritative_fetch_verification_count`
- `authoritative_fetch_verification_failure_count`

Milestone 1 is not making aggressive performance claims yet, but it must still
make the cost basis observable and testable.

### Commit Sequence Semantics

If Milestone 1 includes `commit_sequence` or an equivalent append-order token,
its semantics must be explicitly constrained.

Rules:

- `commit_sequence` is local store append-order metadata, not semantic history
  authority
- branch ancestry and ordered parent edges remain the semantic history shape
- `commit_sequence` may support fetch locality, diagnostics, and stable local
  append ordering only
- `commit_sequence` must not be required to survive export/import with the same
  value unless a later milestone explicitly upgrades that contract
- replay, replication, and ancestry reasoning must not rely on
  `commit_sequence` as a substitute for parent-edge semantics
- gapless behavior must be declared explicitly; if not declared, callers must
  treat it as non-gapless

Forbidden drift:

- inferring branch heads from max `commit_sequence`
- using `commit_sequence` as the authoritative merge/ancestry ordering surface
- using `commit_sequence` as cross-store semantic identity

### Branch Head Legality Rules

Milestone 1 must make branch-head legality explicit.

Allowed transitions in Milestone 1:

- create branch with an initial head at `created_from_commit_id`
- advance branch head to a newly persisted commit whose branch identity matches
  the target branch
- leave branch head unchanged when append is rejected

Milestone 1 must define whether head advancement is restricted to
parent-consistent forward movement. The default expectation for this milestone
is:

- non-fast-forward branch-head rewrites are forbidden
- detached head states are out of scope
- head rollback is out of scope unless later milestone semantics admit it

Minimum legality requirements:

- target head commit must exist as a persisted authoritative commit
- target head branch identity must match the branch whose head is updated
- parent relations for the target commit must already be persisted before head
  advancement succeeds
- branch-head advancement is part of the same authoritative append unit as the
  commit persistence it publishes

Any future widening of branch-head legality must be explicit and typed rather
than backfilled through loose update helpers.

### Authoritative Append Atomicity Rule

Milestone 1 must treat authoritative append as one atomic unit.

One authoritative append atomically covers:

- commit envelope persistence
- ordered parent-row persistence
- authoritative digest-record persistence
- branch-head update for the admitted branch target

Atomicity rule:

- either the whole authoritative append unit becomes visible, or none of it
  does
- partial parent persistence, partial digest persistence, or head-only
  publication is forbidden
- partial failure must roll back to the pre-append authoritative state

This rule is required even before WAL exists. WAL later protects crash
survivability; Milestone 1 already needs a coherent authoritative transaction
boundary.

### Idempotency And Duplicate Commit Policy

Milestone 1 must define exact behavior for duplicate append attempts.

Required policy:

- re-appending the same canonical commit identity with byte-equivalent or
  digest-equivalent authoritative meaning must be treated as explicit
  idempotency, returning the existing persisted authoritative proof or an
  equivalent idempotent success result
- re-appending the same commit identity with conflicting authoritative meaning
  must fail explicitly and typed
- duplicate detection must compare canonical authoritative meaning, not
  backend-local serialized bytes alone
- idempotent success must not create duplicate parent rows, digest rows, or
  branch-head churn

Milestone 1 must not leave duplicate-append behavior backend-defined.

### Authoritative Record Mutability Rules

Milestone 1 must state mutability explicitly.

Immutable after authoritative write:

- canonical commit envelopes
- commit parent records
- authoritative digest records for a given artifact identity and
  canonicalization version

Conditionally mutable under typed rules:

- branch records may allow non-identity metadata updates only if later
  milestones admit them explicitly; Milestone 1 must treat branch identity
  and branch creation basis as immutable
- branch head records may update only through the typed branch-head legality
  rules defined in this milestone

Forbidden behaviors:

- in-place correction of commit payloads
- in-place parent rewrites
- silent digest repair
- ad hoc branch identity rewrites

## Public Store Surface

Milestone 1 must expose one public facade with explicit authority vocabulary.

Representative surface:

```rust
pub struct WorthStoreBuilder { ... }
pub struct WorthStore { ... }

impl WorthStoreBuilder {
    pub fn new() -> Self;
    pub fn with_backend(self, backend: EmbeddedStoreBackend) -> Self;
    pub fn build(self) -> Result<WorthStore, StoreBuildError>;
}

impl WorthStore {
    pub fn append_canonical_commit(
        &mut self,
        envelope: RawRuntimeCommitEnvelope,
    ) -> Result<PersistedAuthoritativeCommit, StoreAppendError>;

    pub fn fetch_canonical_commit(
        &self,
        commit_id: CommitId,
    ) -> Result<FetchedAuthoritativeCommit, StoreFetchError>;

    pub fn fetch_branch_head(
        &self,
        branch_id: BranchId,
    ) -> Result<FetchedBranchHeadRecord, StoreFetchError>;
}
```

Surface rules:

- public methods expose authoritative store concepts only
- public methods do not expose backend-native handles
- append and fetch remain explicit boundary crossings
- builder surfaces are subsystem-shaped and avoid flat bags of storage flags
- classification, canonicalization, and verification stay internal proving
  phases rather than user-assembled steps

Milestone 1 may add diagnostics and parity helpers, but they must hang off the
store facade as derived surfaces, not hidden alternate APIs.

### Cross-Mode Intake Boundary

Milestone 1 must define the future mode seam now to prevent a second semantic
ingest path from appearing later.

Universal across durable, embedded, replay, and replication modes:

- canonicalization rules
- authoritative artifact classification
- authoritative digest basis
- proof-bearing append verification gateway
- persisted authoritative artifact types

Mode-specific wrapper concerns only:

- who produced the raw runtime artifact
- lifecycle/acknowledgment policy around append
- transport wrapper fields
- future checkpoint or replication packaging

The first universal semantic boundary across all modes is:

- `CanonicalizedCommitEnvelope` for canonical meaning
- `VerifiedAuthoritativeAppend` for append admissibility

No later mode may introduce a parallel "almost canonical" envelope family.

## Required Internal Subsystems

Milestone 1 must decompose by responsibility:

- `facade/`
  Public entrypoint and builder.
- `authority/`
  Authoritative artifact families, classification, and proof-bearing types.
- `canonicalization/`
  Canonical ordering, digest basis, and canonicalization-version enforcement.
- `branches/`
  Branch records, branch-head updates, and parent-order persistence.
- `backend/`
  Store-owned backend traits and the embedded backend baseline.
- `verification/`
  Fetch verification and digest comparison.
- `diagnostics/`
  Counters, parity artifacts, and explanation records.
- `harness/`
  Milestone certification adapters and parity fixtures.

This layout follows `domain_laws.md`: separate by what changes and fails for
different reasons, not by technical layer templates.

## Invariant Allocation Table

Milestone 1 must allocate invariants explicitly so later convenience refactors
cannot move authority checks around carelessly.

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| canonical digest determinism | canonicalization | `canonicalization/` | `NonCanonicalEnvelope` | `Durable Artifact Authority Equivalence Test` |
| duplicate collapse legality | canonicalization | `canonicalization/` | `DuplicateCanonicalBodyItem` or `NonCanonicalEnvelope` | milestone parity bundles |
| parent ordering completeness | append verification | `authority/` or `verification/` | `IncompleteParentSet` | `history_digest` parity |
| branch existence for append | append verification | `branches/` | `UnknownBranch` | append rejection matrix |
| branch-head uniqueness | backend constraint plus append transaction | `backend/` | `BranchHeadUniquenessViolation` | backend parity bundles |
| artifact digest uniqueness | backend constraint plus verification | `backend/` and `verification/` | `DuplicateArtifactIdentity` | `artifact_digest` parity |
| fetched artifact digest equality | fetch verification | `verification/` | `FetchedArtifactDigestMismatch` | fetch verification bundle |
| branch-head legality | append verification plus backend transaction | `branches/` and `backend/` | `IllegalBranchHeadTransition` | branch-head parity bundle |
| authoritative append atomicity | append transaction boundary | `backend/` | `AuthoritativeAppendAtomicityViolation` | append failure certification |

Later implementations may refine module names, but they must not blur which
phase owns which invariant.

## Failure Taxonomy

Milestone 1 must ship an explicit typed error family matrix at minimum
covering:

- `MalformedRuntimeEnvelope`
- `NonCanonicalEnvelope`
- `DuplicateCanonicalBodyItem`
- `UnsupportedCanonicalizationVersion`
- `UnknownReservedField`
- `UnknownBranch`
- `OrphanParentReference`
- `IllegalBranchHeadTransition`
- `DuplicateArtifactIdentity`
- `BranchHeadUniquenessViolation`
- `AuthoritativeAppendAtomicityViolation`
- `FetchedArtifactDigestMismatch`
- `MissingBranchHead`
- `BackendIntegrityViolation`

Rules:

- append, fetch, and verification paths must map failures into these families
  or explicit refinements of them
- backend-specific driver failures must not leak as the public semantic error
  taxonomy
- typed failures must remain stable enough for certification bundles and later
  operator diagnostics

## Version Compatibility Rules

Milestone 1 must declare migration posture now.

Rules:

- `canonicalization_version` is part of authoritative artifact metadata, not an
  optional note
- append must reject unsupported canonicalization versions explicitly
- fetch must either:
  - support older admitted canonicalization versions through an explicit
    compatibility reader, or
  - reject them explicitly with `UnsupportedCanonicalizationVersion`
- future-added fields must not change the digest meaning of older versions
- destructive schema rewrites that change authoritative meaning without an
  explicit compatibility story are forbidden
- any future canonicalization-version bump must declare:
  - old-reader behavior
  - new-reader behavior
  - export/import behavior across versions

Milestone 1 does not need to ship all future compatibility readers, but it
must define that silent drift is forbidden.

## Exportable Canonical Record Basis

Because rebuild and backend parity are already in scope, Milestone 1 must
define a canonical exportable basis for authoritative records independent of
backend schema.

Required rule:

- rebuild-from-authority must operate on a canonical exported record basis made
  of authoritative artifact families and their canonical digest identities, not
  on backend-private table dumps alone

This exportable basis may initially be internal to certification, but it must
exist as a declared concept now so "rebuild from authority" cannot secretly
mean "rebuild from our favorite backend format."

## Phases

### Phase 1: Lock Canonical Envelope Authority

Phase 1 establishes the one true committed artifact model.

Required work:

- define the canonical envelope family and authoritative artifact taxonomy
- define canonicalization rules and canonicalization versioning
- define proof-bearing phase types for append and fetch
- define branch record, branch head, and ordered parent record identities
- freeze the DRY envelope rule so later modes and replicas reuse the same
  semantic envelope family

Exit condition:

- one runtime-produced committed meaning has one canonical store envelope shape
- proof-bearing types make skipped canonicalization or verification
  unrepresentable through the normal API
- branch and parent ordering semantics are explicit and typed

This phase solves the hard semantic problem first and intentionally avoids
storage cleverness.

### Phase 2: Persist Authoritative Artifact Families

Phase 2 maps the locked artifact model into one honest backend baseline.

Required work:

- implement the embedded backend baseline
- create authoritative tables, constraints, and indexes
- persist canonical commit envelopes, parent records, branch records, branch
  heads, and digest records
- implement append and fetch surfaces that only traffic in proof-bearing record
  types
- emit counters for append count, parent-record writes, branch-head writes, and
  authoritative fetch reads

Exit condition:

- store can append and fetch authoritative artifacts without backend leakage
- exact branch heads and ordered parents survive round-trip persistence
- fetch returns verified authoritative records, not unchecked blobs

This phase is intentionally narrow: preserve authoritative artifacts exactly
before introducing WAL, snapshots, or delta storage.

### Phase 3: Prove Backend Parity And Artifact Equivalence

Phase 3 turns the baseline into a certifiable authority surface.

Required work:

- define canonical parity bundles for authoritative artifact comparison
- implement rebuild-from-authority verification lane
- run the Milestone 1 named suite:
  `Durable Artifact Authority Equivalence Test`
- compare at least:
  - original embedded backend store
  - second backend family lane that is structurally distinct from the baseline
    implementation
  - second backend configuration lane for the baseline implementation
  - rebuild lane from authoritative artifacts only
- emit machine-checkable digests for truth, history, branch heads, artifacts,
  and replay-visible outputs

Exit condition:

- backend variation changes layout only, never authoritative meaning
- rebuild from authoritative artifacts produces equivalent truth and history
- milestone closeout evidence exists in machine-checkable form

This phase is what earns the right to build WAL and other derived programs on
top of Milestone 1.

## Must Ship

- one canonical durable envelope family for authoritative commit persistence
- explicit authoritative artifact taxonomy
- proof-bearing append and fetch pipeline types
- persistence of canonical commit envelopes
- persistence of ordered parent records
- persistence of branch records and branch heads
- authoritative artifact digest records with canonicalization version
- one store-owned backend abstraction
- one production-grade embedded backend baseline
- one structurally distinct second backend parity lane
- exact append/fetch counters and parity bundle surfaces
- Milestone 1 certification through the named suite in
  [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)

## Must Preserve

- runtime semantics remain owned by `worth-relational`
- signal and bridge layers consume canonical artifacts without storage leakage
- canonical commit envelopes remain the only semantic durability authority
- branch heads and parent ordering remain authoritative and replay-stable
- backend variation does not change authoritative artifact meaning
- diagnostics richness changes retained detail only, not authoritative truth
- no derived or denormalized store record becomes shadow authority

## Acceptance Evidence

Milestone 1 is complete only when the store satisfies the named Milestone 1
suite:

- `Durable Artifact Authority Equivalence Test`

Required machine-checkable outputs:

- `truth_digest`
- `history_digest`
- `branch_heads_digest`
- `artifact_digest`
- `replay_digest`

Milestone-specific proof obligations:

- exact artifact-digest equality across equivalent backend lanes
- exact branch-head and ordered-parent equality across equivalent backend lanes
- rebuild lane parity using authoritative artifacts only
- exact append/fetch counter assertions for representative adversarial scenarios
- typed rejection when malformed or non-canonical append attempts are presented

Milestone 1 is not closed by happy-path append/fetch tests alone.

## Architectural Notes

- Law 41 must be applied aggressively here because storage authority is the
  worst place to tolerate fake proof types.
- The best abstraction is not "one storage helper for everything." The best
  abstraction is one canonical envelope family plus a small number of sealed
  proof-bearing transitions.
- The schema must stay normalized around authority. If a convenience summary
  is later needed, it should be derived and rebuildable.
- Ordered parent persistence must follow relational's merge-ready history
  stance now, even before merge execution becomes a store milestone.
- The backend abstraction must be narrow enough that later implementations
  cannot widen the semantic contract accidentally.
- Milestone 1 must reserve compatibility fields now rather than forcing a
  later destructive schema rewrite for schema-boundary, lineage, or cursor
  families.

## Sequencing Notes

This milestone belongs first because every later store claim depends on it.

- `Milestone 2` can only define operating-mode lifecycle honestly once the
  authoritative artifact model exists.
- `Milestone 3` WAL durability would be structurally dishonest without a frozen
  authoritative append target.
- `Milestone 7` schema/lineage/cursor artifacts can begin in parallel once this
  milestone freezes authoritative artifact identity.
- snapshots, delta layering, compaction, replication, live-query continuation,
  and blob storage all remain downstream derived or adjacent programs and must
  not backfill authority into their own physical layouts.

If Milestone 1 is weak, the rest of the roadmap becomes an argument about which
optimization is "really authoritative." This spec exists to stop that argument
before it starts.
