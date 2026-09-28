# Milestone 2 Engineering Spec: Operating Modes And Lifecycle Contracts

> **Status:** Closed via [milestone-2-closeout.md](./milestone-2-closeout.md)
>
> **Roadmap parent:** `worth_store_roadmap.md`
>
> **Vision parent:** [worth_store_vision.md](./worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Prerequisite milestone:** [milestone-1.md](./milestone-1.md)
>
> **Primary architectural driver:** freeze runtime ownership, intake, and persistence contracts for durable mode, embedded mode, and absent mode before WAL, snapshots, and checkpoint flows widen the store surface

## Goal

Make durable mode, embedded mode, and absent mode explicit, proof-bearing
architectural contracts so later durability work cannot blur who owns runtime
execution, who owns canonical artifact production, and where store is allowed
to persist without stealing semantic authority.

## Why This Milestone Exists

Milestone 1 froze authoritative artifact meaning. Milestone 2 freezes who is
allowed to produce, host, and persist those artifacts under each deployment
shape.

This milestone exists because operating modes are one of the easiest places
for a store architecture to become dishonest:

- durable mode can quietly become "store plus a hidden application runtime"
- embedded mode can quietly become "almost durable mode except we skipped WAL"
- absent mode can quietly stop being real because APIs assume persistence
- future checkpoint and replication paths can quietly add a second
  near-canonical intake family

If that ambiguity survives into Milestone 3 and beyond, every later milestone
will be forced to answer the same dangerous question over and over:

`who actually owns the live runtime that produced this persisted truth?`

Milestone 2 answers that once, structurally, before WAL, snapshots, embedded
checkpoints, and live-query basis artifacts widen the system.

## Hard Part

The hard part of Milestone 2 is not naming three modes.

The hard part is freezing one honest ownership model while still admitting two
very different write boundaries:

- a store-hosted runtime that executes mutations internally
- an external runtime that hands the store already-produced artifacts

without:

- creating two canonical commit meanings
- creating two append-verification pipelines
- turning embedded checkpoints into shadow truth snapshots
- making absent mode a fake mode that still requires store-shaped setup

If this milestone is soft here, Milestone 3 will inherit the ambiguity and
WAL/recovery code will end up deciding what the mode model "really meant."

## Explicit Assumptions

Milestone 2 must make the following assumptions explicit instead of ambient:

- Worth runtime can be hosted by store without store gaining semantic authority
- an external runtime can emit canonical commit artifacts that store can admit
  without re-executing commit semantics
- embedded checkpoint artifacts are not authoritative truth commits; they are a
  separate persisted artifact family whose role is lifecycle support
- absent mode may still use shared libraries or helper types from the Worth
  stack, but it must not require a persistence service or store facade
- future durable-mode crash safety will wrap the durable hosted-runtime path
  rather than replace it
- future embedded checkpoint restore paths will build on a checkpoint intake
  contract defined here rather than inventing a new checkpoint meaning later

If any assumption above is false in implementation, the spec must be revised
before code lands.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is adversarial clarity before code or
  features spread. This spec therefore treats mode ambiguity as the hard
  failure to solve first, not as naming polish around a later implementation.
- `arch_laws.md`
  The most important thing it protects here is proof-bearing lifecycle
  boundaries. Law 41 matters directly: a caller must hold different capability
  and ownership proofs for durable hosting, embedded persistence intake, and
  absent-mode operation; mode-specific entrypoints cannot be raw flags over one
  loose store object.
- `perf_laws.md`
  The most important thing it protects is boundary honesty. Milestone 2
  therefore names mode-selection, checkpoint intake, and absent-mode zero-work
  contracts now so later durable-mode hosting does not conceal orchestration
  cost or ambient coupling.
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped structure.
  Mode ownership, runtime hosting, embedded intake, and absent-mode no-store
  operation must live in separate subdomains rather than one convenience file
  with boolean switches.
- `worth_store_vision.md`
  The most important thing it protects is that store always preserves runtime
  truth without replacing runtime semantics. The spec therefore makes mode
  ownership explicit while keeping one canonical artifact family across all
  modes.
- `worth_store_roadmap.md`
  The most important thing it protects is sequence. Milestone 2 belongs
  immediately after artifact authority because WAL, snapshots, schema/cursor
  durability, and live-query continuation all depend on honest lifecycle
  ownership.
- `worth-store/test-requirements.md`
  The most important thing it protects is certification-grade parity.
  Milestone 2 is not closed until the `Operating Mode Contract Parity Test`
  proves that durable and embedded mode preserve equivalent canonical artifact
  meaning while absent mode remains free of ambient store coupling.
- `milestone-1.md`
  The most important thing it protects is one canonical authoritative artifact
  family. Milestone 2 must build on that by ensuring every mode crosses the
  same canonicalization and append-verification boundary rather than creating a
  second semantic intake path.
- `milestone-1-closeout.md`
  The most important thing it protects is that Milestone 1 is already
  certifiably backend- and rebuild-parity-safe. Milestone 2 must preserve that
  authority path while widening lifecycle ownership, not by renegotiating the
  canonical record model.
- `worth_relational_vision.md`
  The most important thing it protects is that `worth-relational` owns truth
  mutation, commit legality, identity, lineage, and replay semantics. Durable
  mode may host a runtime instance; it may not replace or fork relational
  semantics.
- `worth_relational_roadmap.md`
  The most important thing it protects is serialized authority and commit-time
  truth ownership. Milestone 2 must preserve the single-writer truth boundary
  regardless of whether store hosts the runtime or only receives artifacts from
  one.
- `worth_runtime_bridge_vision.md`
  The most important thing it protects is explicit coordination without runtime
  fusion. Milestone 2 therefore keeps "ownership of a runtime instance" and
  "residency of hot data in memory" as separate ideas so store does not become
  a fused truth-plus-compute monolith by accident.
- `worth_runtime_bridge_roadmap.md`
  The most important thing it protects is clean protocol boundaries. Milestone
  2 must make embedded-mode checkpoint and commit reception a named intake
  contract, not a host-specific side channel.

## Adversarial Constraint

Milestone 2 must survive this hostile condition:

> The same logical history is produced once through a store-hosted runtime in
> durable mode, once through an application-hosted runtime in embedded mode,
> and once through an absent-mode control lane with no store present; the
> durable and embedded persisted artifacts must have identical canonical
> meaning, while the absent lane must prove that no persistence lifecycle,
> counter surface, or hidden store dependency is required for truth semantics
> to exist.

Concretely, the design fails if any supported path:

- requires a second semantic envelope family because the runtime owner differs
- forces embedded mode to pass through durable-mode orchestration wrappers
- lets durable mode redefine commit legality or canonicalization because store
  hosts the runtime
- makes absent mode a fake mode whose APIs still require persistence-shaped
  setup or hidden counters
- lets mixed memory residency get confused with mixed authority ownership
- allows later checkpoints or WAL entries to bypass the canonical Milestone 1
  append-verification gateway

## Product Decision Lock

The following decisions are locked in this milestone:

- operating mode describes runtime lifecycle ownership and durability contract,
  not whether data may be hot in memory, cold on disk, prefetched, or lazily
  hydrated
- Worth runtime semantics are always the truth engine in every admitted mode
- `worth-store` only sometimes owns the live runtime instance; when it does,
  that is durable mode
- durable mode and embedded mode must cross the same canonical semantic intake
  boundary from Milestone 1
- embedded mode persists runtime-produced artifacts or checkpoints; it does not
  become a weaker alias for durable mode
- absent mode is a real first-class valid deployment shape and must remain
  buildable and usable without ambient persistence setup
- one logical branch may have hot working state in memory and cold retained
  history in store without implying multiple authorities for that branch
- no mode is allowed to introduce a second canonicalization rule, second digest
  basis, or second branch-head meaning

Normative consequence:

- any implementation that models modes as one object plus booleans like
  `durable=true` or `embedded=true` without ownership-specific proof types is
  out of spec
- any implementation that treats "embedded mode" as "durable mode without WAL"
  is out of spec
- any implementation that requires a store handle merely to run truth semantics
  in absent mode is out of spec

## Scope

### In Scope

- explicit durable-mode, embedded-mode, and absent-mode lifecycle contracts
- proof-bearing mode configuration and construction surfaces
- durable-mode hosted-runtime ownership boundary
- embedded-mode artifact and checkpoint intake boundary
- explicit checkpoint artifact classification and checkpoint-intake legality
- absent-mode no-store contract
- cross-mode canonical semantic boundary reuse from Milestone 1
- typed lifecycle, construction, and misuse failures
- mode-specific diagnostics and exact counters for mode selection and intake
- certification scaffolding for durable/embedded parity and absent-mode
  non-coupling

### Explicitly Out Of Scope

- WAL append and crash recovery implementation
- snapshot materialization and restore
- branch delta layering
- schema/lineage/cursor durable families beyond the mode contracts needed to
  admit their future persistence surfaces honestly
- live-query continuation
- replication capsules
- budget admission control
- any durable-mode guarantee that acknowledged writes survive crash; that
  begins in Milestone 3, not here
- any requirement that embedded mode persist every in-memory mutation rather
  than explicit runtime-produced artifacts

## Operating Mode Model

### Operating Modes Are Ownership Modes, Not Residency Modes

Milestone 2 locks the most important clarification for future readers:

`operating mode != residency mode`

Operating mode answers:

- who owns the live runtime instance?
- who is allowed to drive commit execution?
- who is responsible for persistence lifecycle and acknowledgment semantics?

Residency answers different questions:

- which truth regions are hot in memory?
- which branch slices are loaded?
- which artifacts are cold, tiered, prefetched, or lazily restored?

Milestone 2 must keep these axes separate so the future store can support:

- hot in-memory working sets over cold durable history
- local geometry-kernel runtime sessions with selective persistence
- store-hosted durable truth with partial hydration

without ever implying two different semantic authorities for the same branch
head or commit path.

### Durable Mode

Durable mode is the deployment shape where `worth-store` owns a live internal
Worth runtime instance and owns the durable acknowledgment boundary around that
runtime.

Durable mode in Milestone 2 must define:

- store owns runtime construction, startup, shutdown, and future recovery
  orchestration
- store accepts mutation requests that are executed by the hosted runtime
- the hosted runtime remains the only owner of truth semantics, commit legality,
  branch legality, lineage, and schema semantics
- store persists only the canonical artifacts produced by that hosted runtime
- future WAL and crash-safe acknowledgment will wrap this mode's lifecycle;
  Milestone 2 defines the boundary without yet claiming crash durability

Milestone 2 must not yet fake durable safety by implying that hosted-runtime
execution alone equals crash-safe durability.

Durable mode must also make the execution boundary tangible:

- a durable-mode request enters as a store-owned mutation request type
- the hosted runtime executes it and produces raw runtime commit artifacts
- those artifacts cross the same Milestone 1 canonicalization and append
  verification boundary used everywhere else
- only then may store persist them

### Embedded Mode

Embedded mode is the deployment shape where the application owns the live Worth
runtime instance and `worth-store` acts as a persistence service for artifacts
or checkpoints produced by that external runtime.

Embedded mode in Milestone 2 must define:

- the application owns runtime construction, mutation execution, and
  in-memory lifecycle
- store receives canonical commit artifacts or checkpoint artifacts from an
  external runtime boundary
- store does not reinterpret, replay-for-legality, or re-decide truth
  semantics during normal embedded intake
- store verifies canonicalization, append admissibility, and artifact
  persistence against the same Milestone 1 authority boundary
- unsaved in-memory work may be lost because embedded mode is not claiming
  per-operation durable acknowledgment

Milestone 2 must also define that embedded mode can coexist with mixed
residency:

- the application may keep a hot working set in memory
- the store may hold colder retained authority and checkpoints
- this is still one semantic system because the runtime remains the truth
  authority and store remains the survival layer

What embedded mode must never become:

- a shadow durable mode with hidden hosted runtime behavior
- a second canonical envelope family
- a host-specific shortcut that bypasses append verification

Embedded mode must also make the intake boundary tangible:

- an external runtime artifact enters as an explicitly external artifact wrapper
- store unwraps it only at the shared canonicalization and verification gateway
- the embedded path must not skip directly from "host handed us something" to
  "artifact persisted"

### Absent Mode

Absent mode is the deployment shape where Worth runtime runs without
`worth-store` at all.

Absent mode in Milestone 2 must define:

- no persistence service exists
- no store builder, counter surface, or artifact persistence hook is required
  to execute truth semantics
- any integration helpers that mention store are impossible or inapplicable in
  this mode by type or construction boundary

Absent mode is not a degenerate unsupported case. It is a required proof that
truth semantics are runtime-owned rather than ambiently store-owned.

Normative rule:

- absent mode must not be constructed through a required `worth-store` facade
  or builder
- store may optionally expose adapter helpers that interoperate with an absent
  runtime lane for certification, but the absent runtime itself must remain
  constructible without store participation

This means Milestone 2 must distinguish:

- `store mode handles` for durable and embedded operation
- `absent mode witness or certification adapters` that can observe an
  independent runtime lane without owning it

### Cross-Mode Semantic Boundary

Milestone 2 inherits and reuses the Milestone 1 universal semantic boundary.

Universal across durable mode and embedded mode:

- canonicalization rules
- canonicalization version
- branch and parent ordering meaning
- artifact classification
- digest basis
- proof-bearing authoritative append verification
- persisted authoritative artifact families

Mode-specific concerns only:

- who owns the live runtime instance
- who owns mutation invocation
- whether the caller submits a mutation request or a runtime-produced artifact
- future durability/acknowledgment policy
- future checkpoint transport wrapping

The first common semantic boundary remains:

- `CanonicalizedCommitEnvelope` for canonical committed meaning
- `VerifiedAuthoritativeAppend` for admissible authoritative persistence

Checkpoint artifacts admitted in embedded mode must follow the same rule:

- they may have their own mode-specific wrapper types
- they may not define an alternate semantic meaning for commits, branches, or
  heads that disagrees with the canonical store model

### Embedded Checkpoint Contract

Milestone 2 must make checkpoint meaning explicit enough that Milestone 7 does
not invent it later.

Checkpoint rules for this milestone:

- checkpoint artifacts are not authoritative truth commits
- checkpoint artifacts are not allowed to redefine branch-head authority
- checkpoint artifacts are lifecycle-support artifacts for embedded mode
- checkpoint artifacts must carry:
  - checkpoint identity
  - source runtime identity or capability witness
  - basis commit or branch-head reference
  - classification showing whether the checkpoint is authoritative,
    derived-durable, or ephemeral for its contents
- if a checkpoint contains authoritative commit artifacts, those commits must
  still cross the normal canonical append boundary as commits rather than
  gaining truth status merely by being inside the checkpoint

Milestone 2 does not need to finalize full checkpoint physical schema, but it
must make "what a checkpoint is not allowed to mean" explicit now.

### Mode-Specific Proof Pipelines

Milestone 2 must make the per-mode phase chains tangible enough for code to map
honestly.

Representative durable-mode progression:

```rust
pub struct DurableMutationRequest { ... }
pub struct HostedRuntimeExecutionResult { ... }
pub struct RawHostedRuntimeCommitEnvelope { ... }
pub struct CanonicalizedCommitEnvelope { ... }
pub struct VerifiedAuthoritativeAppend { ... }
pub struct PersistedAuthoritativeCommit { ... }
```

Representative embedded-mode progression:

```rust
pub struct ExternalRuntimeCommitEnvelope { ... }
pub struct VerifiedExternalRuntimeOrigin { ... }
pub struct CanonicalizedCommitEnvelope { ... }
pub struct VerifiedAuthoritativeAppend { ... }
pub struct PersistedAuthoritativeCommit { ... }
```

Representative embedded checkpoint progression:

```rust
pub struct ExternalRuntimeCheckpointEnvelope { ... }
pub struct VerifiedExternalCheckpointOrigin { ... }
pub struct ClassifiedEmbeddedCheckpointArtifact { ... }
pub struct PersistedEmbeddedCheckpointArtifact { ... }
```

Representative absent-mode certification progression:

```rust
pub struct AbsentRuntimeLane { ... }
pub struct AbsentModeSemanticEvidence { ... }
```

Rules:

- durable and embedded commit paths must converge before authoritative append
- checkpoint paths must remain separate from commit append after origin
  verification
- absent mode produces certification evidence, not persisted store artifacts
- no later phase may accept a weaker type once a stronger mode/origin proof
  exists

### Cross-Mode Misuse Boundary

Milestone 2 must make misuse impossible or explicitly typed.

Required misuse rejections:

- using embedded artifact intake APIs from a durable hosted-runtime handle
- invoking durable mutation-execution APIs on an embedded-only persistence
  handle
- constructing an absent-mode runtime through a store-required builder path
- passing non-canonical or non-verified artifacts into any mode-specific
  persistence path
- handing checkpoints to durable-mode commit append APIs as if they were normal
  canonical commits
- asking store to host semantics in embedded mode while also treating the
  external runtime as authoritative for the same write boundary

Where possible, these misuse classes should be made unrepresentable by
construction; where that is not yet possible, they must fail with typed mode
misuse errors.

## Public Lifecycle Surface

Milestone 2 must expose one public facade while still making mode ownership
explicit in the type system.

Representative shape:

```rust
pub struct WorthStoreFacade { ... }

pub struct DurableModeBuilder { ... }
pub struct EmbeddedModeBuilder { ... }

pub struct DurableStoreHandle { ... }
pub struct EmbeddedStoreHandle { ... }

pub struct ExternalRuntimeCommitEnvelope { ... }
pub struct ExternalRuntimeCheckpointEnvelope { ... }
pub struct DurableMutationRequest { ... }

impl WorthStoreFacade {
    pub fn durable_mode(self) -> DurableModeBuilder;
    pub fn embedded_mode(self) -> EmbeddedModeBuilder;
}
```

Surface rules:

- mode selection must not be a flat enum plus optional fields that all
  constructors ignore opportunistically
- durable-mode builders must require hosted-runtime configuration explicitly
- embedded-mode builders must require persistence-intake configuration
  explicitly
- mode-specific handles must expose only the operations admitted for that mode
- the public facade remains the only public subsystem surface; internal mode
  mechanics stay `pub(crate)`

Absent-mode rule:

- the runtime must be constructible without `WorthStoreFacade`
- if store offers certification helpers for absent-mode comparison, those must
  be auxiliary adapters rather than the construction path for the absent lane

Milestone 2 does not need to freeze the exact Rust names above, but it must
freeze the shape:

- explicit mode builders
- explicit mode handles
- explicit mode-specific request envelopes
- no ambient "universal store object" whose methods branch on runtime flags
- no fake absent-mode builder on the required store facade

## Required Internal Subsystems

Milestone 2 must decompose by responsibility:

- `facade/`
  Public mode builders, mode selection entrypoints, and top-level lifecycle
  handles.
- `modes/durable/`
  Hosted-runtime ownership, durable-mode construction, lifecycle state, and
  future WAL-ready hooks.
- `modes/embedded/`
  External-runtime artifact intake, checkpoint intake, and embedded persistence
  contracts.
- `modes/absent/`
  Absent-mode witnesses, certification helpers, and proof that truth semantics
  do not require a persistence dependency.
- `intake/`
  Cross-mode canonicalization and authoritative append-verification gateways.
- `lifecycle/`
  Shared lifecycle typestates, ownership proofs, and construction transitions.
- `diagnostics/`
  Mode counters, mode-choice evidence, and misuse diagnostics.
- `harness/`
  Operating-mode parity fixtures and certification adapters.

This split is load-bearing. Mode ownership, external intake, and no-store
execution change for different reasons and must not be collapsed into one
`mode.rs` convenience cabinet.

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| durable mode owns hosted runtime lifecycle | mode construction | `modes/durable/` | `InvalidRuntimeOwnershipMode` | `mode_contract_matrix` |
| embedded mode never hosts semantics | mode construction and intake | `modes/embedded/` | `EmbeddedModeLifecycleViolation` | `mode_contract_matrix` |
| absent mode has no ambient store dependency | absent-mode construction | `modes/absent/` | `AbsentModeStoreDependencyViolation` | absent-mode control lane |
| durable and embedded use the same canonical append boundary | cross-mode intake verification | `intake/` | `CrossModeCanonicalBoundaryViolation` | `artifact_digest` parity |
| checkpoint intake is distinct from canonical commit append | embedded intake classification | `modes/embedded/` | `CheckpointCommitSurfaceConfusion` | misuse rejection matrix |
| mode-specific handles expose only admitted operations | public construction and visibility | `facade/` and `lifecycle/` | `ModeCapabilityViolation` | compile-time API review plus typed misuse tests |
| one branch has one authority owner per write boundary | request admission | `lifecycle/` and `modes/*` | `ConflictingAuthorityOwner` | durable/embedded hostile lane |
| absent mode does zero persistence work | absent execution | `modes/absent/` and `diagnostics/` | `AbsentModeStoreDependencyViolation` | zero-counter assertions |
| embedded checkpoints cannot become shadow truth authority | checkpoint intake classification | `modes/embedded/` | `EmbeddedCheckpointAuthorityViolation` | checkpoint misuse matrix |

## Failure Taxonomy

Milestone 2 must ship an explicit typed error family matrix at minimum
covering:

- `InvalidRuntimeOwnershipMode`
- `EmbeddedModeLifecycleViolation`
- `AbsentModeStoreDependencyViolation`
- `ModeCapabilityViolation`
- `CrossModeCanonicalBoundaryViolation`
- `CheckpointCommitSurfaceConfusion`
- `EmbeddedCheckpointAuthorityViolation`
- `ConflictingAuthorityOwner`
- `UnsupportedModeConstruction`
- `HostedRuntimeStartupFailure`
- `HostedRuntimeShutdownFailure`
- `ExternalRuntimeArtifactRejection`
- `ExternalRuntimeCheckpointRejection`
- `ModeSelectionContractViolation`

Rules:

- public lifecycle failures must be typed in mode vocabulary, not leaked as
  raw backend or driver errors
- canonicalization and authoritative append failures from Milestone 1 remain
  preserved and may be nested or wrapped, not renamed into vague mode failures
- future WAL or checkpoint-path failures may refine this taxonomy later, but
  Milestone 2 must already make ownership misuse and intake confusion explicit

## Complexity Contracts

Milestone 2 must declare exact hot-path contracts for lifecycle selection and
cross-mode intake even though heavy durability work is still later.

Minimum contracts:

- mode construction cost is proportional to:
  - one mode selection
  - one admitted ownership proof construction
  - hosted-runtime initialization work only in durable mode
- embedded commit-intake cost is proportional to:
  - one external artifact verification pass
  - one authoritative append through the Milestone 1 intake boundary
- embedded checkpoint-intake cost is proportional to:
  - one checkpoint envelope classification
  - one checkpoint artifact persistence path
- absent-mode construction and operation cost is proportional to:
  - one no-store builder path
  - zero persistence-intake work

Minimum counters:

- `durable_mode_selection_count`
- `embedded_mode_selection_count`
- `absent_mode_selection_count`
- `hosted_runtime_start_count`
- `hosted_runtime_stop_count`
- `external_commit_intake_count`
- `external_checkpoint_intake_count`
- `embedded_checkpoint_authority_rejection_count`
- `cross_mode_canonical_boundary_reuse_count`
- `mode_misuse_rejection_count`
- `absent_mode_store_touch_count`

Required zero assertions:

- `absent_mode_store_touch_count` must remain zero in the absent-mode control
  lane
- durable-mode hosted-runtime counters must remain zero in embedded and absent
  lanes
- external-intake counters must remain zero in durable and absent lanes unless
  an explicitly named mixed-certification lane is running

## Phases

### Phase 1: Lock Mode Semantics And Ownership Proofs

Phase 1 defines the meaning of the modes before any implementation convenience
can blur them.

Required work:

- define durable, embedded, and absent mode as separate ownership contracts
- define the distinction between operating mode and residency mode
- define proof-bearing builders and handles for each mode
- define the first universal semantic boundary reused across durable and
  embedded paths
- define the checkpoint artifact contract and checkpoint/commit separation
- define typed misuse boundaries and authority-owner exclusivity rules

Exit condition:

- a competent engineer can tell, from the types alone, who owns runtime
  execution in each mode
- it is impossible or explicitly typed to confuse artifact intake with hosted
  execution
- absent mode is structurally real rather than described only in prose

### Phase 2: Encode Mode-Specific Intake And Lifecycle Paths

Phase 2 maps the locked ownership model into honest subsystem boundaries.

Required work:

- implement durable-mode lifecycle construction and hosted-runtime ownership
  shell
- implement embedded-mode external commit-envelope intake and checkpoint-intake
  shells
- implement absent-mode no-store witness and certification capability surface
- route durable and embedded commit persistence through the same Milestone 1
  canonical append-verification gateway
- route checkpoint intake through a distinct classified checkpoint path
- emit exact mode-selection and intake counters

Exit condition:

- durable mode can host a runtime boundary without yet claiming WAL safety
- embedded mode can persist external runtime artifacts without stealing
  semantics
- absent mode can execute without ambient store coupling
- public APIs expose separate admitted operations per mode instead of
  runtime-flag branches

### Phase 3: Prove Operating-Mode Parity And Misuse Rejection

Phase 3 turns the ownership model into certification evidence.

Required work:

- implement the Milestone 2 named suite:
  `Operating Mode Contract Parity Test`
- compare:
  - durable-mode hosted-runtime persistence lane
  - embedded-mode external-runtime artifact persistence lane
  - absent-mode no-store control lane
- emit machine-checkable parity and misuse bundles
- prove zero-store-touch and zero-hosted-runtime-work assertions where
  required

Exit condition:

- durable and embedded lanes emit equivalent canonical artifact meaning for the
  same logical truth
- absent mode proves truth semantics do not depend on store presence
- checkpoint lanes prove checkpoint classification does not alter commit
  authority meaning
- mode misuse fails explicitly and typed rather than falling through to
  whatever path happens to exist

## Must Ship

- explicit durable and embedded mode builders and handles
- explicit absent-mode no-store witness and certification adapters
- proof-bearing runtime ownership model
- explicit distinction between operating mode and residency mode
- durable-mode hosted-runtime lifecycle shell
- embedded-mode external commit and checkpoint intake shells
- absent-mode no-store witness and certification surface
- one shared canonical semantic boundary reused by durable and embedded modes
- one explicit checkpoint classification contract separate from canonical
  commit append
- typed lifecycle and misuse failures
- exact mode-selection and intake counters
- Milestone 2 certification through the named suite in
  [test-requirements.md](./test-requirements.md)

## Must Preserve

- runtime semantics remain owned by Worth runtime, not by store mode wrappers
- Milestone 1 canonical artifact authority remains singular
- durable mode does not redefine truth because it hosts the runtime
- embedded mode does not become a disguised durable mode
- absent mode remains a valid first-class deployment shape
- operating mode stays distinct from hot/cold residency or tier placement
- later WAL, checkpoint, and snapshot work must build on these contracts rather
  than reinterpret them

## Acceptance Evidence

Milestone 2 is complete only when the store satisfies the named Milestone 2
suite:

- `Operating Mode Contract Parity Test`

Required machine-checkable outputs:

- `artifact_digest`
- `diagnostics_digest`
- `mode_contract_matrix`

Milestone-specific proof obligations:

- exact canonical artifact parity between durable-mode and embedded-mode lanes
  for semantically equivalent histories
- explicit inequality or typed rejection where a mode is asked to do work it
  does not admit
- absent-mode proof that no ambient store dependency exists
- explicit proof that checkpoint intake cannot publish alternate commit or
  branch-head meaning
- exact zero-counter assertions for forbidden cross-mode work
- typed rejection for checkpoint/commit surface confusion and ownership misuse

## Architectural Notes

- The most dangerous naive design here is one `WorthStore` object plus a `mode`
  enum and a long series of `match` statements inside every method. That shape
  hides ownership and invites accidental capability bleed. The correct shape is
  separate proof-bearing handles over one facade.
- Milestone 2 should prefer sealed lifecycle typestates over booleans. "Store
  may host runtime" and "store may receive external commit envelopes" are not
  configuration hints; they are capability boundaries.
- Embedded mode must stay structurally thin. Its job is to persist external
  runtime artifacts faithfully, not to become a second runtime authority that
  silently replays or re-adjudicates host commits.
- Absent mode is a product proof, not a convenience. If absent mode becomes
  awkward or impossible, that is evidence that semantics leaked into store.
- Mixed residency belongs to later tiering and working-set milestones, but the
  conceptual split must be locked here so future readers do not confuse "some
  truth is hot in memory" with "two authorities are active."
- Milestone 3 should be able to add WAL and crash recovery by wrapping the
  durable-mode lifecycle already defined here, not by replacing the mode model.

## Sequencing Notes

This milestone belongs immediately after Milestone 1 because authoritative
artifact meaning must already be frozen before lifecycle ownership can be made
honest.

- `Milestone 3` depends on this milestone because WAL and crash recovery must
  know exactly which mode owns runtime execution and acknowledgment.
- `Milestone 7` depends on this milestone because embedded-mode checkpoint
  artifacts need an explicit non-durable ownership contract before they can be
  persisted honestly.
- `Milestone 8` depends on this milestone because live-query basis and resume
  semantics differ depending on whether store hosted the runtime or only
  persisted external artifacts.
- this milestone deliberately stops before promising crash durability, because
  claiming that before WAL exists would be architectural bluffing.
