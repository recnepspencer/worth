# Milestone 1 Engineering Spec: Patch-To-Invalidation And Snapshot Evaluation

> **Status:** Closed engineering spec and shipped closeout reference
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Shipped closeout:** [milestone-1-closeout.md](./milestone-1-closeout.md)
>
> **Primary architectural driver:** establish the first real causal protocol boundary between `worth-relational` and `worth-signal`
>
> **Companion docs:**
> - [worth_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
> - [worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
> - [worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
> - [architectural_guidelines.md](../../docs/coding-guidelines/arch_laws.md)
> - [performance_guidelines.md](../../docs/coding-guidelines/perf_laws.md)
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)

## Summary

Milestone 1 is not "make relational changes wake up signal somehow."

It is the first production-grade bridge foundation:

- committed truth changes become canonical bridge routing input
- bridge routing lowers those committed changes into deterministic invalidation artifacts
- signal evaluation reads stable truth snapshots rather than live mutable state
- bridge diagnostics explain patch-to-invalidation mapping without owning truth logic or scheduling logic
- replay of the same truth-side commit artifacts produces the same bridge artifacts

The governing rule is:

`truth commits once, bridge lowers once, signal evaluates against a stable snapshot`

The bridge's job in Milestone 1 is wiring only. It does not decide truth
semantics. It does not decide scheduling semantics. It coordinates a truthful
handoff between the two.

Milestone 1 should therefore be read as a proof-carrying bridge spec, not a
list of integration goals. If a later implementation can satisfy the prose
while still allowing raw unsafely-constructed packets, canonicality by
convention, or fallback-by-scan behavior, then this spec is incomplete and must
be tightened before code lands.

## 1. Adversarial Constraint

Milestone 1 must survive the following hostile condition:

> A long-lived system with active truth mutation, retained history, and large
> derived graphs must route the same committed truth patchset into the same
> invalidation artifact every time, while every participating signal evaluation
> reads from a stable truth snapshot rather than drifting live state, and while
> the bridge remains explainable and replayable under mutation churn.

Concretely, the design must remain correct when all of the following are true:

- truth is mutating continuously while reads and recomputation requests occur
- patchsets contain multiple entities, aspects, and relation surfaces
- invalidation targets are derived from declared bridge mapping, not ad hoc code
- signal evaluation may happen later than routing, but must still see the same
  intended truth view
- diagnostics tier changes between environments
- replay occurs after process restart from canonical truth artifacts
- future milestones will add merge, preview, policy propagation, and writeback,
  but Milestone 1 must not fake those concepts early

If any supported path:

- reads live mutable truth during evaluation
- routes identical committed patchsets differently across runs
- requires host-specific glue to interpret bridge artifacts
- lets the bridge invent truth semantics or scheduling semantics

then Milestone 1 has failed.

## 2. Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- the bridge will be a dedicated subsystem, not hidden inside either parent runtime
- Milestone 1 introduces a first-class bridge crate, expected to be named
  `worth-runtime-bridge`
- the bridge consumes canonical committed truth artifacts; it does not consume
  ad hoc mutation callbacks
- signal evaluation through the bridge is snapshot-backed only
- bridge artifacts are canonical and replay-safe even if diagnostics richness is
  reduced
- future bridge features must build on Milestone 1 artifacts rather than
  replacing the core patch-to-invalidation and snapshot-read contract

Normative consequence:

- any implementation path that wires relational internals directly into signal
  internals without a bridge facade is out of spec
- any implementation path that exposes live truth handles inside signal
  evaluation is out of spec
- any implementation path that treats bridge diagnostics as optional debug-only
  sugar is out of spec

## 3. Scope

### 3.1 In Scope

- a dedicated bridge facade and crate boundary
- canonical bridge input derived from committed truth patch artifacts
- deterministic patch-to-invalidation routing
- bridge-side snapshot acquisition and stable snapshot handle propagation
- bridge-side mapping diagnostics and replay artifacts
- harness scenarios proving routing determinism and snapshot stability under
  active mutation
- clean builder/configuration surfaces for bridge setup

### 3.2 Explicitly Out Of Scope

- merge-aware bridge semantics
- speculative or preview bridge flows
- bridge-mediated writeback into truth
- branch-aware policy propagation beyond the minimum snapshot identity needed for
  replay and diagnostics
- parallel execution of bridge work beyond preserving future-compatible planning
  boundaries
- fine-grained field/lens subscriptions beyond what Milestone 1 minimally needs
- reactive source protocol as a general-purpose product surface

Milestone 1 must leave room for these later without pretending to ship them now.

## 4. Governing Design Rules

### 4.1 The Bridge Owns Translation, Not Semantics

Milestone 1 must be able to answer:

- what committed truth artifact entered the bridge?
- what invalidation artifact left the bridge?
- what stable truth snapshot backed the derived read?

It must not answer:

- what the truth mutation meant semantically
- how signal scheduling should execute beyond the invalidation request it receives
- whether a truth-side rule was legal

Those remain owned by the parent runtimes.

### 4.2 One Canonical Input Artifact

Bridge routing must consume one canonical truth-side commit-derived artifact.

Milestone 1 must not support:

- parallel bridge-specific patch formats
- host-computed sideband invalidation hints as authority
- per-consumer reinterpretation of truth commit meaning

Truth emits one canonical committed artifact. The bridge derives its routing
view from that artifact.

### 4.3 Snapshot Stability Is A Structural Contract

Signal evaluation through the bridge must receive a stable truth view with a
durable identity.

The bridge may hold:

- a snapshot handle
- a snapshot identity
- snapshot-visible read accessors

The bridge may not hold:

- mutable truth authority
- direct write-path access
- ambient access to "latest truth" during evaluation

### 4.4 Plan / Lower / Execute Separation Is Required From Day One

Even though Milestone 1 is not the bridge parallelism milestone, it must still
separate:

- ingestion of canonical truth patch artifacts
- lowering into deterministic bridge routing artifacts
- execution of invalidation delivery into signal

If routing meaning is rediscovered inside the delivery path, Milestone 1 will
block later work on replay, parallel-ready planning, and certification.

### 4.5 Proof-Carrying Phase Types Are Mandatory

Milestone 1 must explicitly satisfy Architectural Laws 30 and 41.

The bridge pipeline is not:

- raw envelope in
- some internal work
- artifact out

It is a proof chain:

- raw committed envelope
- validated committed envelope
- eligible routed request
- planned route
- lowered invalidation artifact
- delivered invalidation result

Each phase output must be a distinct type carrying exactly the proof established
by that phase.

Representative progression:

```rust
pub struct RawCommittedPatchEnvelope { ... }
pub struct ValidatedCommittedPatchEnvelope { ... }
pub struct EligibleRouteRequest { ... }
pub struct PlannedBridgeRoute { ... }
pub struct LoweredInvalidationArtifact { ... }
pub struct DeliveredInvalidationResult { ... }
```

Rules:

- constructors for proof-bearing types must be sealed to the owning module
- fields that encode proof-bearing transitions must not be publicly writable
- later phases must consume the immediately prior proof type, not a weaker raw type
- execution must not accept raw collections when a proof-bearing packet exists
- any runtime check for a property that a prior phase already proved is a design failure

### 4.6 Canonicality Must Be Mechanically Specified

Milestone 1 must not use "canonical" as a vibe word. It must define canonical
bases precisely enough that two independent implementations could produce the
same artifacts.

For every canonical artifact in this milestone, the spec must define:

- the exact ordered input set
- the sort key
- the deduplication rule
- the digest basis
- what fields are identity-bearing versus explanatory-only

Milestone 1 canonicality must cover at least:

- committed truth envelope normalization
- mapping registration ordering
- route plan ordering
- invalidation target ordering
- route record digest basis

If an artifact can be built from an unordered map, insertion-order vector, or
host iteration order without an explicit canonicalization pass, the design is
out of spec.

### 4.7 Diagnostics Must Be Derived, Not Interleaved With Truth

Operational truth for Milestone 1 is:

- canonical truth artifact identity
- bridge route summary
- snapshot identity
- invalidation artifact identity
- counters

Rich bridge explanation remains derived under diagnostics policy. Changing
diagnostics richness must not change routing truth.

## 5. Target Runtime Model

### 5.1 New Crate Boundary

Milestone 1 should introduce:

- `crates/worth-runtime-bridge/`

Expected public ownership:

- bridge facade
- bridge builder / registration surface
- bridge routing data types
- bridge snapshot contract
- bridge diagnostics / artifacts facade
- bridge harness adapter

Expected non-ownership:

- relational storage internals
- relational commit authority
- signal graph ownership
- signal scheduler internals

### 5.2 Public Facade Surface

The bridge must expose one public facade, expected in:

- `crates/worth-runtime-bridge/src/facade.rs`

Representative surface:

```rust
pub struct RuntimeBridgeBuilder { ... }
pub struct RuntimeBridge { ... }

pub struct BridgeRouteRequest { ... }
pub struct BridgePlannedRoute { ... }
pub struct BridgeRouteResult { ... }
pub struct BridgeInvalidationArtifact { ... }
pub struct BridgeSnapshotToken { ... }
pub struct BridgeDiagnosticsHandle { ... }

impl RuntimeBridgeBuilder {
    pub fn new() -> Self;
    pub fn with_relational_source(self, source: RelationalBridgeSource) -> Self;
    pub fn with_signal_sink(self, sink: SignalBridgeSink) -> Self;
    pub fn with_policy(self, policy: BridgeRuntimePolicy) -> Self;
    pub fn build(self) -> Result<RuntimeBridge, BridgeBuildError>;
}

impl RuntimeBridge {
    pub fn plan_committed_patch(
        &self,
        request: BridgeRouteRequest,
    ) -> Result<BridgePlannedRoute, BridgeRouteError>;

    pub fn deliver_invalidation(
        &self,
        route: BridgePlannedRoute,
    ) -> Result<BridgeRouteResult, BridgeDeliveryError>;

    pub fn diagnostics(&self) -> &BridgeDiagnosticsFacade;
}
```

Design rules:

- the facade exposes bridge concepts only
- it does not re-export parent-runtime internals as the main contract
- builder configuration is subsystem-shaped, not a flat option bag
- planning and delivery are separate public boundary crossings
- callers must not be able to trigger hidden multi-phase orchestration through an innocent-looking getter-shaped API

### 5.3 Input Contract

Milestone 1 needs an explicit truth-to-bridge input envelope.

Representative shape:

```rust
pub struct RawCommittedPatchEnvelope { ... }
pub struct ValidatedCommittedPatchEnvelope {
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
    patch_summary: TruthPatchSummary,
    patch_body: CanonicalTruthPatchBody,
    digest: CanonicalTruthPatchDigest,
}
```

Rules:

- the bridge consumes committed patch truth only
- snapshot identity must be explicit in the envelope or derivable from it
- the bridge may depend on truth-side aspect labels already emitted by the
  truth runtime, but may not infer new truth semantics on its own
- the envelope must be durable enough for replay and diagnostics
- raw envelopes must be normalized and validated into a canonical form before planning
- canonical truth envelope normalization must define:
  - patch item ordering key
  - duplicate patch item collapse rules
  - identity-bearing fields included in digest computation
  - explanatory-only fields excluded from digest computation

### 5.4 Snapshot Contract

Milestone 1 must define a dedicated bridge-side snapshot read contract.

Representative shape:

```rust
pub trait TruthSnapshotReader {
    type Error;

    fn snapshot_identity(&self) -> TruthSnapshotIdentity;
    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, Self::Error>;
}

pub struct BridgeSnapshotContext<R: TruthSnapshotReader> {
    snapshot: R,
    snapshot_identity: TruthSnapshotIdentity,
}

pub struct SnapshotReadPacket { ... }
pub struct SnapshotReadPacketResult { ... }
```

Rules:

- bridge-side signal evaluation consumes `BridgeSnapshotContext`, not raw truth runtime access
- the snapshot reader is read-only and phase-limited
- snapshot identity is carried through bridge diagnostics and replay artifacts
- all later source-protocol work must refine this contract, not replace it
- Milestone 1 must not rely on scalar N+1 snapshot reads as the default bridge execution shape
- any scalar convenience read surface must be implemented on top of packetized reads, not the other way around
- snapshot read packets must be derived once from route planning inputs and reused across later phases where possible

### 5.5 Routing Contract

Milestone 1 routing must lower from committed patch surfaces into deterministic
bridge invalidation artifacts.

Representative shapes:

```rust
pub struct EligibleRouteRequest { ... }

pub struct PlannedBridgeRoute {
    route_identity: BridgeRouteIdentity,
    source_commit: TruthCommitIdentity,
    source_patch: TruthPatchIdentity,
    source_snapshot: TruthSnapshotIdentity,
    routing_summary: BridgeRoutingSummary,
    read_packet: SnapshotReadPacket,
    counters: BridgeRoutingCounters,
}

pub struct LoweredInvalidationArtifact {
    route_identity: BridgeRouteIdentity,
    invalidation_targets: CanonicalInvalidationTargets,
    snapshot_token: BridgeSnapshotToken,
    counters: BridgeRoutingCounters,
}
```

Rules:

- canonical ordering is mandatory
- route planning must derive the batch summary exactly once, then pass it forward immutably
- execution/delivery must consume lowered route truth only
- invalidation artifacts must be durable enough for replay
- signal receives invalidation meaning, not truth internals
- `PlannedBridgeRoute` and `LoweredInvalidationArtifact` constructors must be private to the proving modules
- route identity and digest basis must not depend on diagnostics-only richness

### 5.6 Mapping Registration

Milestone 1 needs an explicit mapping registration surface, but only for the
coarse bridge foundation.

Representative shape:

```rust
pub struct BridgeMappingRegistration {
    pub mapping_id: BridgeMappingId,
    pub truth_scope: TruthPatchScope,
    pub signal_scope: SignalInvalidationScope,
    pub routing_mode: CoarseRoutingMode,
}
```

Rules:

- Milestone 1 mapping is explicit registration, not inferred convention
- mapping registration must be frozen at build time
- routing declarations remain coarse in Milestone 1
- Milestone 1 routing declarations are scope-to-scope wiring declarations, not arbitrary host logic
- mapping registration must normalize into canonical registration order before freeze
- later aspect/fine-grained milestones extend this surface rather than bypass it

Explicit Milestone 1 restriction:

- `CoarseRoutingMode` must be a closed vocabulary owned by the bridge, not a callback or expression language

### 5.7 Diagnostics And Artifact Surface

Representative shapes:

```rust
pub struct BridgeRouteRecord {
    route_identity: BridgeRouteIdentity,
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    invalidation_artifact_identity: BridgeInvalidationIdentity,
    counters: BridgeRoutingCounters,
    digest: BridgeRouteRecordDigest,
}

pub struct BridgeRoutingExplanation {
    route_identity: BridgeRouteIdentity,
    routing_entries: Vec<BridgeRoutingEntryExplanation>,
}
```

Rules:

- route record is canonical
- explanation is derived richness
- all machine-checkable replay uses canonical route record and invalidation artifact, not free-form text
- canonical route record fields must be distinguishable from explanation-only additions in the type surface

### 5.8 Bridge-Owned Adapter Traits

> **Superseded for Relational by [Milestone 20](./milestone-20.md).** The
> Bridge depends on Relational and Signal; Relational does not implement
> Bridge traits. Signal already has that shape.

The bridge must depend on narrow bridge-owned contracts implemented by the
parent runtimes, not broad parent facades.

Representative shape:

```rust
pub trait RelationalBridgeAdapter {
    type Snapshot: TruthSnapshotReader;
    type Error;

    fn validate_committed_patch(
        &self,
        raw: RawCommittedPatchEnvelope,
    ) -> Result<ValidatedCommittedPatchEnvelope, Self::Error>;

    fn open_snapshot(
        &self,
        identity: TruthSnapshotIdentity,
    ) -> Result<Self::Snapshot, Self::Error>;
}

pub trait SignalBridgeAdapter {
    type Error;

    fn deliver_invalidation(
        &self,
        artifact: LoweredInvalidationArtifact,
    ) -> Result<DeliveredInvalidationResult, Self::Error>;
}
```

Rules:

- these adapter traits are owned by `worth-runtime-bridge`
- parent runtimes implement them through narrow integration modules
- the bridge must not depend on wide parent facade access when a narrower adapter contract exists

## 6. Required Internal Subsystems

Milestone 1 should decompose into these internal subsystems:

- `input/`
  canonical truth envelope ingestion
- `snapshot/`
  snapshot token creation and read context
- `mapping/`
  frozen mapping registration and lookup
- `routing/`
  validating eligibility, deriving batch summaries, and lowering route plans into invalidation artifacts
- `delivery/`
  sending invalidation artifacts into signal
- `diagnostics/`
  route records, counters, and explanation reconstruction
- `harness/`
  bridge certification adapter

This keeps read-path, route-path, and delivery-path separate.

### 6.1 Domain-Standards Compliance

Milestone 1 must follow the workspace domain standards from day one.

This means:

- organize by bridge subdomains, not file type
- default to more decomposition when responsibilities may grow apart later
- treat folders as architectural boundaries
- expose one root `facade.rs` only
- avoid flat catch-all files such as `helpers.rs`, `utils.rs`, `bridge.rs`, or
  one giant `routing.rs` that mixes planning, lowering, matching, diagnostics,
  and replay

The bridge is a long-term subsystem. The directory plan must assume growth now,
not "we will split it later if it gets large."

### 6.2 Expected Subdomain Layout

Milestone 1 should begin with a layout shaped like this:

```text
crates/worth-runtime-bridge/src/
  facade.rs
  lib.rs
  input/
    mod.rs
    envelope.rs
    normalization.rs
    validation.rs
  snapshot/
    mod.rs
    token.rs
    context.rs
    packet.rs
  mapping/
    mod.rs
    registration.rs
    freezing.rs
    lookup.rs
    fallback.rs
  routing/
    mod.rs
    eligibility.rs
    planning.rs
    lowering.rs
    canonicalization.rs
    counters.rs
  delivery/
    mod.rs
    invalidation.rs
    result.rs
  diagnostics/
    mod.rs
    records.rs
    explanation.rs
    replay.rs
  harness/
    mod.rs
    adapter.rs
    fixtures/
      mod.rs
      route_parity.rs
      snapshot_stability.rs
      mapping_failures.rs
    profiles.rs
    matrices.rs
```

Rules:

- planning and lowering must not live in the same file just because they are
  both "routing"
- normalization and validation must not live in the same file just because they
  are both "input"
- fallback classification must not disappear inside lookup logic
- replay-facing diagnostics must not be buried inside generic diagnostics helpers
- harness fixtures must be organized by bridge certification concern, not by
  generic categories like `helpers`, `world`, or `actions`

### 6.3 Test-Domain Structure Must Follow Production-Domain Structure

Bridge tests are not exempt from structural standards.

Milestone 1 must avoid test layouts like:

```text
src/tests/
  helpers.rs
  fixtures.rs
  routing.rs
  workflows.rs
```

Instead, test support should be split by responsibility:

- harness adapter support
- fixture definitions
- parity scenarios
- snapshot-stability scenarios
- failure scenarios
- replay scenarios
- counter-certification scenarios

If setup, mutation batches, assertions, and scenario definitions change for
different reasons, they must live in different files or folders.

## 7. Acceptance-Critical Counters

Milestone 1 must declare structural counters now.

Minimum required counters:

- `patch_item_count`
- `normalized_patch_item_count`
- `routing_entry_count`
- `invalidation_target_count`
- `mapping_lookup_count`
- `mapping_fallback_count`
- `snapshot_read_count`
- `snapshot_read_packet_count`
- `snapshot_identity_mismatch_count`
- `route_replay_mismatch_count`

Rules:

- counters belong to canonical route results
- tests must assert exact values in certification scenarios where possible
- later milestones may add counters, but these become part of the floor
- any fallback counter that remains non-zero in a supported path must correspond to an explicitly admitted fallback class

### 7.1 Explicit Fallback Policy

Milestone 1 must not normalize "fallback" into "scan more until something
works."

Admitted fallback classes, if any exist in Milestone 1, must be:

- deterministic
- storage-visible
- bounded by canonical patch scope
- represented as typed fallback classes in route artifacts and diagnostics

Forbidden fallback classes:

- whole-truth scans
- live-state discovery outside the pinned snapshot
- host callback heuristics
- runtime-only hidden retries that change route breadth

## 8. Harness And Certification Model

Milestone 1 is not complete without bridge-native certification.

Milestone 1 must start its testing foundation on top of `worth-harness`, not on
crate-local helpers that later need to be reorganized into a real testing
domain.

`worth-harness` is already the workspace-owned substrate for:

- scenario fixtures
- mutation batches
- execution requests and execution profiles
- run records and snapshot records
- diagnostics, explanations, provenance, and replay captures
- parity suites
- certification matrices

The bridge milestone must treat that as the testing architecture from day one.
If a proposed test helper duplicates harness responsibilities such as scenario
description, profile sweeps, parity comparison, or certification reporting, the
helper is presumptively out of spec.

Expected additions:

- `crates/worth-runtime-bridge/src/tests/`
- bridge harness adapter using `worth-harness`
- fixture builders for truth commit artifacts, mapping registration, and signal invalidation expectations
- named profile catalogs and certification matrices rooted in `worth-harness` nouns rather than bespoke local test runners

Minimum certification lanes:

- deterministic routing parity
- snapshot stability under hot truth mutation
- diagnostics-tier invariance of routing truth
- replay parity from canonical artifacts
- explicit failure on invalid mapping or snapshot identity mismatch

Representative test names:

- `tests::routing::identical_committed_patchsets_lower_to_identical_invalidation_artifacts`
- `tests::routing::bridge_evaluation_reads_snapshot_stable_truth_under_hot_mutation`
- `tests::routing::routing_truth_is_invariant_across_diagnostics_tiers`
- `tests::routing::replayed_route_matches_original_route_from_canonical_artifacts`
- `tests::routing::snapshot_identity_mismatch_fails_explicitly`

### 8.1 Harness Foundation Requirements

Milestone 1 must establish a clean bridge testing domain with these first-class
surfaces:

- `ScenarioPlan` and `ScenarioFixture` for bridge worlds and route scenarios
- `MutationBatch` for truth-side committed patch inputs or hostile change batches
- `ExecutionRequest` for route planning, delivery, replay, and capture requests
- `ExecutionProfile` for deterministic, replay, diagnostics-tier, and future
  parallel-admission profile sweeps
- `HarnessAdapter` implementation for the bridge runtime
- `ParitySuite` for profile-to-profile route parity
- `CertificationMatrix` for adversarial route-certification sweeps

Rules:

- bridge tests must describe scenario truth through harness fixtures, not
  through ad hoc setup helpers hidden in individual tests
- bridge profile sweeps must use `ExecutionProfile`, not local enums or boolean
  flags
- bridge parity checks must use `ParitySuite` where the concern is run-to-run
  equivalence
- bridge certification sweeps must use `CertificationMatrix` where the concern
  is matrix-style hostile coverage across multiple profiles
- bridge-specific helper code must exist to adapt bridge concepts into harness
  concepts, not to replace harness concepts

### 8.2 Bridge Harness Adapter Scope

Milestone 1 should introduce a bridge adapter that truthfully implements the
relevant `worth-harness` contracts.

Expected adapter shape:

- base `HarnessAdapter`
- `DiagnosticsHarnessAdapter`
- `ReplayHarnessAdapter`
- additional capture adapters only where bridge-native artifacts exist and can
  be reported truthfully

The bridge adapter owns:

- creating a bridge runtime/session
- loading a bridge scenario fixture
- applying a truth-side mutation batch or committed patch batch
- executing plan and delivery requests through the bridge facade
- capturing route records, snapshots, diagnostics summaries, and replay records

The bridge adapter must not:

- leak relational or signal internals through harness records
- claim capture surfaces the bridge cannot actually provide
- define a parallel test framework outside `worth-harness`

### 8.3 Adversarial Certification Requirement

Milestone 1 tests must be hostile enough to block the usual trivial fallback of:

- one happy-path fixture
- one mutation
- one assertion on output length

Minimum required certification families:

- fixed deterministic fixtures proving canonical route parity
- seeded hostile mutation matrices over varying patch widths, mapping shapes,
  and diagnostics tiers
- replay-after-restart route certification from canonical artifacts
- mapping ambiguity and unsupported-scope failure certification
- snapshot-stability certification under active mutation pressure
- exact counter assertions for named adversarial scenarios

This milestone is not complete if only fixture-style unit tests exist.
Certification matrices and parity suites are part of the product-grade testing
surface, not optional hardening.

### 8.4 Workflow Certification Forward-Compatibility

Milestone 1 does not need to implement the full workflow-certification layer in
`worth-harness`, but it must remain compatible with that future.

Required implication:

- bridge fixtures, mutation batches, run records, and replay artifacts must be
  shaped so they can later participate in workflow-style certification without
  replacement
- bridge tests must avoid inventing local step, checkpoint, or artifact
  taxonomies that conflict with `worth-harness` workflow certification design
- any bridge-local regression target handling should anticipate eventual
  promotion into harness-visible regression/certification lanes rather than
  `#[ignore]`-style hiding

## 9. Target API And Module Plan

### 9.1 New Files Expected

- `crates/worth-runtime-bridge/src/lib.rs`
- `crates/worth-runtime-bridge/src/facade.rs`
- `crates/worth-runtime-bridge/src/input/mod.rs`
- `crates/worth-runtime-bridge/src/input/envelope.rs`
- `crates/worth-runtime-bridge/src/input/normalization.rs`
- `crates/worth-runtime-bridge/src/input/validation.rs`
- `crates/worth-runtime-bridge/src/snapshot/mod.rs`
- `crates/worth-runtime-bridge/src/snapshot/token.rs`
- `crates/worth-runtime-bridge/src/snapshot/context.rs`
- `crates/worth-runtime-bridge/src/snapshot/packet.rs`
- `crates/worth-runtime-bridge/src/mapping/mod.rs`
- `crates/worth-runtime-bridge/src/mapping/registration.rs`
- `crates/worth-runtime-bridge/src/mapping/freezing.rs`
- `crates/worth-runtime-bridge/src/mapping/lookup.rs`
- `crates/worth-runtime-bridge/src/mapping/fallback.rs`
- `crates/worth-runtime-bridge/src/routing/mod.rs`
- `crates/worth-runtime-bridge/src/routing/eligibility.rs`
- `crates/worth-runtime-bridge/src/routing/planning.rs`
- `crates/worth-runtime-bridge/src/routing/lowering.rs`
- `crates/worth-runtime-bridge/src/routing/canonicalization.rs`
- `crates/worth-runtime-bridge/src/routing/result.rs`
- `crates/worth-runtime-bridge/src/routing/counters.rs`
- `crates/worth-runtime-bridge/src/delivery/mod.rs`
- `crates/worth-runtime-bridge/src/delivery/invalidation.rs`
- `crates/worth-runtime-bridge/src/delivery/result.rs`
- `crates/worth-runtime-bridge/src/diagnostics/mod.rs`
- `crates/worth-runtime-bridge/src/diagnostics/facade.rs`
- `crates/worth-runtime-bridge/src/diagnostics/records.rs`
- `crates/worth-runtime-bridge/src/diagnostics/explanation.rs`
- `crates/worth-runtime-bridge/src/diagnostics/replay.rs`
- `crates/worth-runtime-bridge/src/harness/mod.rs`
- `crates/worth-runtime-bridge/src/harness/adapter.rs`
- `crates/worth-runtime-bridge/src/harness/profiles.rs`
- `crates/worth-runtime-bridge/src/harness/matrices.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/mod.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/route_parity.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/snapshot_stability.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/mapping_failures.rs`
- `crates/worth-runtime-bridge/src/tests/routing/route_parity.rs`
- `crates/worth-runtime-bridge/src/tests/routing/snapshot_stability.rs`
- `crates/worth-runtime-bridge/src/tests/routing/replay_parity.rs`
- `crates/worth-runtime-bridge/src/tests/routing/failure_taxonomy.rs`
- `crates/worth-runtime-bridge/src/tests/routing/counter_certification.rs`

### 9.2 Existing Files Expected To Change

- [Cargo.toml](../../Cargo.toml)
- [facade.rs](../../crates/worth-relational/src/facade.rs)
- [facade.rs](../../crates/worth-signal/src/facade.rs)
- [bridge.rs](../../crates/worth-signal/src/presentation/harness/bridge.rs)
- [harness_bridge.rs](../../crates/worth-signal/src/tests/harness_bridge.rs)

Expected change types:

- add new workspace crate
- add minimal relational export surface for canonical committed patch envelopes or adapters
- add minimal signal sink surface for bridge-driven invalidation delivery
- add harness integration for bridge certification
- preserve domain-aligned file decomposition instead of introducing flat catch-all modules

## 10. Implementation Phases

Milestone 1 must execute in strict order. Later phases may reopen earlier ones,
but no phase should bypass unfinished foundation with ad hoc glue.

### Phase M1.0 - Inventory And Boundary Lock

Purpose:

- identify current truth and signal surfaces relevant to bridge integration
- lock what the bridge owns versus what it must not own
- define the canonical Milestone 1 replacement cut line

Required work:

- inventory existing relational patch and snapshot-visible surfaces
- inventory existing signal invalidation and evaluation entrypoints
- identify any current pseudo-bridge glue that would violate the new crate boundary
- define the minimal adapter surfaces needed from both parent crates

Exit criteria:

- there is an explicit ownership table for truth, bridge, and compute
- there is an explicit list of parent-runtime adapter surfaces
- there is no unresolved ambiguity about whether Milestone 1 lives in a dedicated crate

### Phase M1.1 - Crate And Facade Foundation

Purpose:

- establish `worth-runtime-bridge` as a first-class subsystem

Required work:

- add the new crate to the workspace
- create one public facade
- create builder, error, and policy surface scaffolding
- define visibility boundaries so internal subsystems are not public by default

Exit criteria:

- the bridge has one facade and a compilable crate boundary
- no external consumer must reach through internal modules

### Phase M1.2 - Canonical Truth Envelope And Snapshot Contract

Purpose:

- formalize the bridge input and stable read surfaces

Required work:

- define `BridgeCommittedPatchEnvelope`
- define `TruthSnapshotReader` and `BridgeSnapshotContext`
- define snapshot identity and token shapes
- add adapter surfaces from relational into the bridge envelope/snapshot model

Exit criteria:

- bridge routing can be driven solely from a canonical truth envelope
- bridge-backed reads use snapshot context only
- snapshot identity is explicit and durable

### Phase M1.3 - Mapping Registration And Frozen Registry

Purpose:

- prevent routing meaning from living in host-local conditionals

Required work:

- define mapping registration types
- define frozen mapping registry
- add build-time validation for duplicate or ambiguous registrations
- expose deterministic lookup surfaces

Exit criteria:

- bridge routing cannot occur without registered mapping truth
- mapping iteration order is deterministic
- duplicate or ambiguous mapping fails at build time

### Phase M1.4 - Routing Plan And Invalidation Artifact

Purpose:

- lower canonical truth envelopes into canonical bridge artifacts

Required work:

- define `BridgeRoutePlan`
- define `BridgeInvalidationArtifact`
- implement canonical ordering and digest basis
- add routing counters and plan/result summaries

Exit criteria:

- route lowering occurs exactly once per request
- invalidation artifacts are replay-safe
- route counters are surfaced on results
- canonical ordering and digest inputs are specified and test-covered

### Phase M1.5 - Signal Delivery Integration

Purpose:

- deliver invalidation artifacts into signal without leaking bridge semantics

Required work:

- define signal sink adapter surface
- connect invalidation delivery to signal entrypoints
- wire snapshot context into bridge-driven evaluation request flow

Exit criteria:

- signal receives invalidation artifacts and snapshot context, not raw truth internals
- delivery path does not rediscover routing semantics
- planning and delivery remain separate public surfaces

### Phase M1.6 - Diagnostics, Replay, And Certification

Purpose:

- turn Milestone 1 from integration code into certifiable infrastructure

Required work:

- add canonical route records
- add explanation reconstruction over route records
- add replay parity checks
- add harness fixtures and adversarial tests

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- diagnostics tier changes richness only, not routing truth
- replay from canonical artifacts reproduces route truth

## 11. Explicit Failure Taxonomy For Milestone 1

Milestone 1 must ship typed bridge failures for at least:

- missing mapping registration
- ambiguous mapping registration
- unsupported truth patch scope
- snapshot acquisition failure
- snapshot identity mismatch
- signal sink rejection
- unsupported fallback class
- replay route mismatch
- canonical artifact decode or compatibility failure

These are bridge failures, not raw string bubbles from parent runtimes.

## 12. Anti-Patterns Explicitly Rejected

- embedding bridge code inside `worth-relational` or `worth-signal` as the primary public surface
- passing live mutable truth references into signal evaluation
- deriving invalidation targets directly from host callbacks instead of canonical committed artifacts
- treating bridge routing diagnostics as optional logging
- exposing parent-runtime internals as the bridge API
- storing routing meaning in unordered maps without canonicalization
- allowing mapping registration after bridge build
- letting the delivery path reinterpret truth semantics or routing meaning
- collapsing bridge configuration into a single flat options struct

## 13. Closeout Standard

Milestone 1 is complete only when all of the following are true:

- a dedicated bridge crate and facade exist
- committed truth patch truth enters the bridge through one canonical envelope
- bridge routing deterministically lowers into canonical invalidation artifacts
- bridge-backed signal evaluation reads stable truth snapshots only
- route truth is replay-safe and diagnostics-tier-invariant
- bridge failures are typed and explicit
- harness certification proves deterministic routing, snapshot stability, replay parity, and explicit failure behavior under hostile mutation pressure

If code lands but the bridge still depends on live truth reads, host-local
mapping glue, or non-canonical routing behavior, Milestone 1 is not complete.
