# Milestone 2 Engineering Spec: Aspect Mapping And Fine-Grained Subscriptions

> **Status:** Closed engineering spec and shipped closeout reference
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-1.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-1.md)
>
> **Prior closeout:** [milestone-1-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-1-closeout.md)
>
> **Shipped closeout:** [milestone-2-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2-closeout.md)
>
> **Hardening companion:** [milestone-2-envelope-and-planning-hardening.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2-envelope-and-planning-hardening.md)
>
> **Primary architectural driver:** make bridge precision first-class without collapsing truth-side aspect semantics into signal-side dependency ownership
>
> **Companion docs:**
> - [worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
> - [worth_signal_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/worth_signal_vision.md)
> - [worth_signals2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/worth_signals2.md)
> - [MENTALITY.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/MENTALITY.md)
> - [architectural_guidelines.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/architectural_guidelines.md)
> - [domain_standards.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/domain_standards.md)
> - [performance_guidelines.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/docs/coding-guidelines/performance_guidelines.md)

## Summary

Milestone 1 proved that:

- committed truth can enter the bridge once through a canonical envelope
- routing can lower once into deterministic invalidation artifacts
- bridge-backed evaluation can stay pinned to a stable truth snapshot

That foundation is necessary, but it is still coarse.

Milestone 2 exists because coarse scope-to-scope routing does not survive the
product pressure that the bridge is explicitly meant to handle:

- large truth objects with small field-level edits
- topology- or facet-shaped data where only part of an entity matters
- subscription sets that must stay precise under replay and diagnostics
- hosts that cannot afford whole-object invalidation inflation on every edit

This milestone is therefore not "add field subscriptions somehow."

It is the capability boundary where the bridge learns how to preserve
truth-side precision across the runtime boundary while still obeying the core
bridge rule:

`truth declares what changed, bridge canonically maps that change into explicit subscription slices, signal invalidates only the derived surfaces admitted by that slice`

The bridge still does not own truth semantics or signal dependency ownership.
It owns:

- canonical aspect mapping
- canonical fine-grained subscription identity
- deterministic matching, reduction, and fallback classification
- replay-safe artifacts explaining why a truth-side fine-grained change reached a specific derived slice

## Goal

Make fine-grained truth changes route into equally fine-grained derived
invalidation without widening to whole-object invalidation, without leaking
truth storage semantics into signal ownership, and without losing replay-safe
diagnosability.

## Why This Milestone Exists

Milestone 2 belongs immediately after Milestone 1 because it builds on the
exact artifacts Milestone 1 established:

- canonical committed patch envelopes
- stable snapshot-backed read contexts
- frozen bridge-owned mapping registration
- deterministic route planning and lowering
- typed bridge failures and replay records

Without Milestone 1, fine-grained routing would collapse into ad hoc callback
logic or live-state discovery. Without Milestone 2, every future bridge feature
that depends on precise truth surfaces inherits coarse invalidation debt:

- lineage continuity would have to remap whole objects instead of explicit slices
- historical and branch-aware evaluation would not be able to explain what
  precise truth surface a run depended on
- bulk routing would scale with inflated invalidation breadth rather than
  semantic delta
- structural-identity-aware remapping would not have a stable slice identity to
  remap

Milestone 2 therefore earns its place in the roadmap by solving the next real
structural problem: preserving semantic delta at the bridge boundary.

## Adversarial Constraint

Milestone 2 must survive the following hostile condition:

> A long-lived system with dense entities, overlapping subscriptions, repeated
> field- and aspect-level edits, active truth mutation, and replay after
> restart must route the same canonical fine-grained truth delta into the same
> canonical subscription-slice invalidation artifact every time, while widening
> only when an explicitly typed fallback class permits it, and while every
> affected derived evaluation still reads a stable snapshot view rather than
> drifting live truth.

Concretely, the design must remain correct when all of the following are true:

- one committed patchset carries multiple entity surfaces, field surfaces, and
  aspect surfaces
- multiple mapping registrations overlap by entity but differ by aspect or lens
- the same truth entity participates in both coarse and fine-grained bridge mappings
- subscription slices are registered in different host orderings across runs
- diagnostics richness changes between environments
- replay occurs from canonical bridge artifacts after restart
- some truth surfaces are intentionally unsupported and must fail or widen explicitly

If any supported path:

- widens to whole-entity invalidation without an explicit typed reason
- depends on host iteration order for aspect matching
- invents signal dependency meaning from truth internals inside the hot path
- requires live truth inspection outside the pinned snapshot to decide mapping
- cannot explain why a specific truth surface hit a specific dependency slice

then Milestone 2 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- aspect mapping is a first-class bridge subsystem, not an annotation hidden in
  route planning
- fine-grained subscription slices are bridge-owned canonical artifacts
- mapping registration remains explicit and frozen at build time
- unsupported or ambiguous fine-grained routing must fail explicitly or use a
  typed admitted fallback class
- snapshot-backed reads remain mandatory for all bridge-driven evaluation,
  including fine-grained subscriptions
- Milestone 2 extends Milestone 1 artifacts and proof chains; it does not
  replace them with a new routing protocol

Normative consequence:

- callback-shaped mapping rules are out of spec
- host-local "if field X changed, invalidate Y" conditionals are out of spec
- hidden widening from fine-grained to coarse routing is out of spec
- aspect routing that can only be understood by reading relational internals or
  signal internals is out of spec

## Scope

### In Scope

- canonical bridge-owned aspect vocabulary surfaces and mapping declarations
- field, lens, region, partition, facet, or similarly shaped subscription
  slices where the bridge explicitly admits them
- deterministic lowering from canonical truth-side delta surfaces into
  canonical subscription-slice invalidation targets
- typed ambiguity, suppression, unsupported-surface, and admitted-fallback handling
- diagnostics and replay artifacts showing why truth-side fine-grained surfaces
  matched specific derived slices
- harness scenarios proving fine-grained precision, coarse/fine parity safety,
  replayability, and explicit fallback behavior

### Explicitly Out Of Scope

- lineage-aware continuity across replace, split, or merge-like identity
  evolution
- historical or branch-aware evaluation beyond the snapshot contract already
  inherited from Milestone 1
- speculative preview flows
- generalized reactive source productization
- structural-identity-assisted remapping
- merge-aware bridge semantics

Milestone 2 must leave clean extension points for those later milestones
without pretending to ship them now.

## Governing Design Rules

### 1. Aspect Meaning Stays Split Across The Boundary

Truth runtime aspects and signal runtime aspects are not one shared ontology.

The bridge must own a mapping contract between them, but it must not:

- make truth runtime aspect labels the public signal dependency model
- make signal dependency slices the authority for truth change meaning
- erase the distinction between "truth-side changed surface" and
  "signal-side affected dependency slice"

The bridge answers:

- what truth-side surface changed?
- what bridge mapping rule admitted it?
- what canonical subscription slices did that rule lower to?

It must not answer:

- what the truth runtime should consider an aspect in general
- how signal internally stores or schedules its dependencies

### 2. Fine-Grained Subscription Identity Must Be Canonical

Milestone 2 must introduce bridge-owned subscription-slice identity that is:

- independent of host registration order
- independent of diagnostics richness
- independent of transient object addresses or in-memory handles
- durable enough for replay, route explanation, and future remapping work

Admitted Milestone 2 truth delta surface categories:

- entity field surface
- entity relation-endpoint surface
- entity region surface
- entity partition surface
- entity facet surface

Admitted Milestone 2 subscription-slice categories:

- signal field slice
- signal lens slice
- signal region slice
- signal partition slice
- signal facet slice
- registered coarse fallback slice

Explicit Milestone 2 restriction:

- no user-defined or callback-defined slice categories
- no free-form path strings as the public slice authority
- no host-local extension point that silently invents a new surface category
- structural-identity slices are out of scope until the structural-identity milestone

Each admitted slice category must define:

- identity-bearing fields
- canonical ordering key
- deduplication basis
- digest basis
- explanatory-only fields excluded from identity

### 3. Widening Must Be Typed, Bounded, And Honest

Milestone 2 is allowed to widen only through explicit admitted fallback classes.

Fallback classes must be:

- typed
- canonical
- bounded by the canonical patch scope
- visible in counters, route records, and explanations

Forbidden fallback classes:

- whole-truth scans
- implicit "invalidate the whole object because matching is hard"
- host callbacks that expand the route behind the bridge's back
- live snapshot probing to discover a wider target set after planning

Admitted Milestone 2 fallback classes are intentionally narrow:

- `RegisteredEntityCoarseFallback`
- `RegisteredPartitionFallback`

Both classes must:

- be declared in mapping registration rather than discovered during routing
- preserve deterministic ordering and digest basis
- identify the exact normalized truth delta surfaces that forced widening
- remain visible in canonical route records and explanation reconstruction

### 4. Plan / Match / Lower / Deliver Separation Is Mandatory

Milestone 2 extends the Milestone 1 proof chain rather than shortcutting it.

Fine-grained routing must remain a pipeline:

- validated committed envelope
- normalized truth delta surface set
- eligible subscription-slice request
- planned aspect matches
- lowered fine-grained invalidation artifact
- delivered invalidation result

Matching must not be rediscovered inside delivery. Delivery consumes lowered
bridge truth only.

### 5. Slice Matching Must Be Proof-Carrying

Milestone 2 must continue satisfying Architectural Laws 30 and 41.

Representative progression:

```rust
pub struct NormalizedTruthDeltaSurfaceSet { ... }
pub struct EligibleSliceRouteRequest { ... }
pub struct PlannedAspectMatchSet { ... }
pub struct LoweredSubscriptionSliceArtifact { ... }
pub struct DeliveredSliceInvalidationResult { ... }
```

Rules:

- constructors for proof-bearing route packets remain sealed
- later phases consume the exact proof type produced upstream
- fallback admission, ambiguity classification, and suppression classification
  become part of the proof chain, not side comments
- delivery cannot accept a weaker route packet than lowering produced

### 6. Canonicality Must Be Mechanically Declared

For every Milestone 2 artifact, the spec must define:

- ordered input set
- ordering key
- deduplication rule
- digest basis
- identity-bearing versus explanatory-only fields

Canonicality must cover at least:

- truth delta surface normalization
- aspect registration ordering
- subscription-slice registration ordering
- planned match ordering
- lowered invalidation target ordering
- fallback ordering
- route record slice-entry ordering

If any artifact can vary because a host uses a different map implementation or
registration order, the design is out of spec.

### 7. Snapshot Read Breadth Must Follow The Slice Plan

Milestone 2 may require richer read packets than Milestone 1, but it still may
not widen read breadth opportunistically.

Rules:

- slice-level read packets are derived during planning, not ad hoc during delivery
- later evaluation phases consume the planned packet shape
- packet breadth must be explainable from the lowered slice artifact
- scalar convenience reads remain layered on packetized reads, never the reverse

### 8. Diagnostics Are Derived From Canonical Route Truth

Operational truth for Milestone 2 is:

- canonical truth delta surface set identity
- planned match set identity
- lowered slice artifact identity
- typed fallback/suppression/ambiguity classifications
- counters

Rich explanation remains derived and policy-shaped.
Changing diagnostics richness must not change route truth.

## Target Runtime Model

### 1. Public Surface Extension

Milestone 2 should extend `worth-runtime-bridge` rather than creating a second
bridge surface.

Expected public ownership growth:

- aspect registration declarations
- subscription-slice declarations
- slice-route planning packets
- lowered slice invalidation artifacts
- slice-level diagnostics and replay artifacts

Expected non-ownership remains unchanged:

- relational aspect authority
- relational storage semantics
- signal dependency graph ownership
- signal scheduler internals

### 2. Representative Public Facade Surface

The bridge facade should grow concepts shaped like:

```rust
pub struct BridgeAspectRegistration { ... }
pub struct BridgeSliceRouteRequest { ... }
pub struct BridgePlannedSliceRoute { ... }
pub struct BridgeSliceRouteResult { ... }
pub struct LoweredSubscriptionSliceArtifact { ... }
pub struct SubscriptionSliceReadPacket { ... }

impl RuntimeBridge {
    pub fn plan_slice_route(
        &self,
        request: BridgeSliceRouteRequest,
    ) -> Result<BridgePlannedSliceRoute, BridgeSliceRouteError>;

    pub fn deliver_slice_invalidation(
        &self,
        route: BridgePlannedSliceRoute,
    ) -> Result<BridgeSliceRouteResult, BridgeSliceRouteDeliveryError>;
}
```

Design rules:

- planning and delivery remain separate public boundary crossings
- slice-route requests consume canonical committed-patch truth, not ad hoc host hints
- the bridge facade exposes bridge nouns only
- slice planning must remain an extension of Milestone 1 proof-carrying routing,
  not an alternate hidden orchestration path

### 3. Slice Input Contract

Milestone 2 needs an explicit normalized truth-delta surface contract built on
Milestone 1 envelopes.

Representative shape:

```rust
pub struct TruthDeltaSurface {
    surface_identity: TruthDeltaSurfaceIdentity,
    entity_identity: TruthEntityIdentity,
    aspect_kind: TruthAspectKind,
    surface_kind: TruthDeltaSurfaceKind,
    branch_identity: TruthBranchIdentity,
}

pub struct NormalizedTruthDeltaSurfaceSet {
    source_commit: TruthCommitIdentity,
    source_patch: TruthPatchIdentity,
    source_snapshot: TruthSnapshotIdentity,
    surfaces: Vec<TruthDeltaSurface>,
    digest: TruthDeltaSurfaceSetDigest,
}
```

Rules:

- the normalized surface set is derived exactly once from the validated
  committed patch envelope
- duplicate surfaces must collapse canonically before matching
- each surface must carry enough identity to support replay and explanation
- explanatory-only labels must not participate in digest identity

### 4. Mapping And Subscription Registration Contract

Milestone 2 needs a stricter registration surface than the coarse Milestone 1
mapping declarations.

Representative shape:

```rust
pub struct BridgeAspectRegistration {
    pub registration_id: BridgeAspectRegistrationId,
    pub truth_surface_kind: TruthDeltaSurfaceKind,
    pub truth_aspect_kind: TruthAspectKind,
    pub subscription_slice_kind: SubscriptionSliceKind,
    pub fallback_policy: SliceFallbackPolicy,
}
```

Rules:

- registrations remain explicit and frozen at build time
- a registration must name one admitted truth surface category and one admitted
  subscription-slice category
- registrations may declare only bridge-owned fallback policies
- overlap that would produce ambiguous canonical matching must fail during
  registry freeze rather than later at route time

### 5. Slice Routing Contract

Milestone 2 routing must lower normalized truth delta surfaces into canonical
subscription-slice invalidation artifacts.

Representative shape:

```rust
pub struct EligibleSliceRouteRequest { ... }

pub struct PlannedAspectMatchSet {
    route_identity: BridgeRouteIdentity,
    source_commit: TruthCommitIdentity,
    source_patch: TruthPatchIdentity,
    source_snapshot: TruthSnapshotIdentity,
    normalized_surface_set: NormalizedTruthDeltaSurfaceSet,
    read_packet: SubscriptionSliceReadPacket,
    counters: BridgeSliceRoutingCounters,
}

pub struct LoweredSubscriptionSliceArtifact {
    route_identity: BridgeRouteIdentity,
    subscription_slices: CanonicalSubscriptionSlices,
    fallbacks: CanonicalSliceFallbacks,
    snapshot_token: BridgeSnapshotToken,
    counters: BridgeSliceRoutingCounters,
}
```

Rules:

- canonical ordering is mandatory across surfaces, matches, slices, and fallbacks
- route planning derives the read packet once and passes it forward immutably
- delivery consumes lowered slice truth only
- slice artifacts must remain durable enough for replay, diagnostics, and later
  continuity/history work

### 6. Diagnostics And Replay Surface

Representative shapes:

```rust
pub struct BridgeSliceRouteRecord {
    route_identity: BridgeRouteIdentity,
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    normalized_surface_set_identity: TruthDeltaSurfaceSetDigest,
    slice_artifact_identity: BridgeInvalidationIdentity,
    counters: BridgeSliceRoutingCounters,
    digest: BridgeSliceRouteRecordDigest,
}

pub struct BridgeSliceRoutingExplanation {
    route_identity: BridgeRouteIdentity,
    entries: Vec<BridgeSliceRoutingEntryExplanation>,
}
```

Rules:

- canonical route records remain the replay authority
- explanations remain reconstructable derived richness
- canonical records must distinguish matched, suppressed, ambiguous, and
  fallback-routed surfaces

### 7. Bridge-Owned Adapter Traits

Milestone 2 should continue depending on narrow bridge-owned contracts rather
than wider runtime facades.

Representative shape:

```rust
pub trait RelationalAspectBridgeAdapter {
    type Error;

    fn normalize_truth_delta_surfaces(
        &self,
        envelope: &ValidatedCommittedPatchEnvelope,
    ) -> Result<NormalizedTruthDeltaSurfaceSet, Self::Error>;
}

pub trait SignalSliceBridgeAdapter {
    type Error;

    fn deliver_slice_invalidation(
        &self,
        artifact: LoweredSubscriptionSliceArtifact,
    ) -> Result<DeliveredSliceInvalidationResult, Self::Error>;
}
```

Rules:

- the bridge owns the slice-routing adapter contracts
- parent runtimes implement only the narrow contracts required to satisfy the
  bridge proof chain
- no broad facade reach-through is allowed just because finer precision exists

## Phases

### Phase 1: Canonical Slice Vocabulary And Mapping Authority

Phase 1 exists to make fine-grained precision structurally representable.

Milestone 2 must first define:

- bridge-owned truth delta surface categories admitted for fine-grained routing
- bridge-owned subscription-slice categories admitted on the signal side
- mapping registration declarations connecting admitted truth surfaces to
  admitted subscription slices
- canonical registration freezing, overlap validation, and ambiguity rejection

This phase leaves the system in a coherent state where:

- bridge mappings can name fine-grained surfaces explicitly
- unsupported fine-grained categories fail at build time or registration validation
- no route planning depends on host-local conditionals to interpret slice meaning

### Phase 2: Deterministic Match Planning And Lowered Slice Artifacts

Phase 2 exists to turn the new vocabulary into deterministic bridge truth.

Milestone 2 must then implement:

- normalization of committed truth patch surfaces into canonical fine-grained
  delta surface sets
- eligibility classification for supported, suppressed, ambiguous, and
  fallback-admitted surfaces
- deterministic planned match sets from normalized surfaces plus frozen mapping truth
- lowered invalidation artifacts carrying canonical subscription slices, typed
  fallback classes, counters, and planned read packets

This phase leaves the system in a coherent state where:

- the same canonical truth delta lowers to the same slice artifact every time
- coarse and fine-grained mappings can coexist without hidden drift
- the bridge can explain route precision or intentional widening mechanically

### Phase 3: Replay, Diagnostics, And Precision Certification

Phase 3 exists to prove that the new precision is trustworthy.

Milestone 2 must finally ship:

- canonical route and replay records for slice-level routing
- explanation reconstruction showing why specific truth surfaces matched
  specific subscription slices
- hostile harness suites covering fine-grained precision, diagnostics-tier
  invariance, replay parity, and explicit ambiguity/fallback behavior
- exact counter assertions for representative precision and fallback scenarios

This phase leaves the system in a coherent state where:

- fine-grained routing is certifiable rather than plausible
- replay after restart can validate slice-level parity
- later continuity/history/scale milestones can build on stable slice artifacts

## Must Ship

- bridge-owned aspect mapping vocabulary and registration surfaces
- canonical fine-grained subscription-slice identity types
- deterministic normalization from committed truth patch surfaces into
  canonical fine-grained delta surface sets
- deterministic planned aspect-match artifacts
- lowered slice invalidation artifacts carrying canonical subscription slices,
  planned read packets, typed fallback classes, and counters
- typed failures for missing mapping, ambiguous slice match, unsupported slice
  category, invalid fallback admission, and replay mismatch
- diagnostics and replay artifacts for slice-level routing explanation
- harness certification lanes for fine-grained precision and coarse/fine parity safety

## Must Preserve

- truth runtime remains the only authority for truth semantics and snapshots
- signal runtime remains the owner of dependency ownership and execution
- no live mutable truth reads during bridge-driven evaluation
- no hidden widening from fine-grained routing to coarse routing
- canonical ordering and replay-safe identities
- builder-time freezing of mapping declarations
- clean facade boundaries rather than re-exported parent-runtime internals

## Acceptance Evidence

Milestone 2 is complete only when the bridge harness can prove:

- identical canonical fine-grained truth deltas lower to identical slice
  invalidation artifacts
- field-, lens-, region-, or facet-scoped truth changes invalidate only the
  intended admitted derived slices
- coarse and fine subscription paths remain parity-safe where both are admitted
- diagnostics richness changes explanation only, not slice-routing truth
- explicit fallback classes route deterministically and remain visible in route artifacts
- ambiguous or unsupported fine-grained routing fails explicitly and typed
- replay from canonical slice-level artifacts matches original slice-routing semantics

## Architectural Notes

### Representative Public Surface Growth

Milestone 2 should extend the existing bridge facade with concepts shaped like:

```rust
pub struct BridgeAspectRegistration { ... }
pub struct TruthDeltaSurface { ... }
pub struct SubscriptionSlice { ... }
pub struct PlannedAspectMatchSet { ... }
pub struct LoweredSubscriptionSliceArtifact { ... }

pub enum SliceFallbackClass {
    RegisteredCoarseWidening,
    RegisteredPartitionWidening,
}

pub enum SliceRouteFailure {
    MissingSliceMapping { ... },
    AmbiguousSliceMapping { ... },
    UnsupportedTruthDeltaSurface { ... },
    UnsupportedSubscriptionSlice { ... },
    InvalidFallbackAdmission { ... },
}
```

The important rule is structural, not nominal:

- aspect registrations remain bridge-owned
- slice route failures remain typed and bridge-native
- the bridge facade exposes bridge concepts, not direct relational or signal internals

### Expected Internal Subdomains

Milestone 2 should extend the bridge crate with subdomains such as:

- `mapping/aspects/`
- `mapping/subscriptions/`
- `routing/surfaces/`
- `routing/matching/`
- `routing/fallbacks/`
- `diagnostics/slices/`
- `harness/fixtures/fine_grained_precision.rs`
- `harness/fixtures/ambiguity_failures.rs`
- `harness/fixtures/fallback_routes.rs`

This follows the workspace domain standards:

- aspect vocabulary is not the same responsibility as subscription-slice vocabulary
- matching is not the same responsibility as fallback classification
- slice explanations are not the same responsibility as canonical records

### Minimum Counter Floor

Milestone 2 must add counters such as:

- `truth_delta_surface_count`
- `normalized_truth_delta_surface_count`
- `planned_slice_match_count`
- `lowered_subscription_slice_count`
- `slice_fallback_count`
- `slice_suppression_count`
- `slice_ambiguity_count`
- `slice_snapshot_read_packet_count`
- `slice_replay_mismatch_count`

Exact names may refine during implementation, but the structural floor is not optional.

### Explicit Ambiguity And Suppression Policy

Milestone 2 must classify non-direct matches structurally rather than narratively.

Required classifications:

- `Matched`
- `SuppressedByRegistrationPolicy`
- `AmbiguousRegistration`
- `UnsupportedSurfaceCategory`
- `FallbackAdmitted`

Rules:

- each normalized truth delta surface must resolve to exactly one classification
- ambiguity and suppression must be carried in route artifacts, not only in logs
- unsupported categories must fail explicitly unless the registration policy
  admits a typed fallback class
- a surface may not be both ambiguous and fallback-admitted

## Test And Harness Model

Milestone 2 must follow the same structural testing discipline as Milestone 1.

Expected first-class test surfaces:

- fine-grained precision scenarios
- coarse/fine parity scenarios
- ambiguity failure scenarios
- fallback route scenarios
- diagnostics-tier invariance scenarios
- replay parity and replay drift scenarios
- counter certification scenarios

Minimum representative test names:

- `tests::routing::field_surface_invalidates_only_registered_field_slice`
- `tests::routing::region_surface_invalidates_only_registered_region_slice`
- `tests::routing::coarse_and_fine_routes_remain_parity_safe_for_shared_scope`
- `tests::routing::ambiguous_slice_registration_fails_explicitly`
- `tests::routing::registered_partition_fallback_routes_deterministically`
- `tests::routing::replayed_slice_route_matches_original_canonical_slice_artifact`

## Target API And Module Plan

### New Files Expected

- `crates/worth-runtime-bridge/src/mapping/aspects/mod.rs`
- `crates/worth-runtime-bridge/src/mapping/aspects/registration.rs`
- `crates/worth-runtime-bridge/src/mapping/aspects/freezing.rs`
- `crates/worth-runtime-bridge/src/mapping/aspects/ambiguity.rs`
- `crates/worth-runtime-bridge/src/mapping/subscriptions/mod.rs`
- `crates/worth-runtime-bridge/src/mapping/subscriptions/slices.rs`
- `crates/worth-runtime-bridge/src/mapping/subscriptions/fallback_policy.rs`
- `crates/worth-runtime-bridge/src/routing/surfaces.rs`
- `crates/worth-runtime-bridge/src/routing/matching.rs`
- `crates/worth-runtime-bridge/src/routing/suppression.rs`
- `crates/worth-runtime-bridge/src/routing/fallbacks.rs`
- `crates/worth-runtime-bridge/src/routing/slice_packet.rs`
- `crates/worth-runtime-bridge/src/diagnostics/slices.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/fine_grained_precision.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/ambiguity_failures.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/fallback_routes.rs`
- `crates/worth-runtime-bridge/src/tests/routing/fine_grained_precision.rs`
- `crates/worth-runtime-bridge/src/tests/routing/coarse_fine_parity.rs`
- `crates/worth-runtime-bridge/src/tests/routing/ambiguity_failures.rs`
- `crates/worth-runtime-bridge/src/tests/routing/fallback_routes.rs`

### Existing Files Expected To Change

- [facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade.rs)
- [registration.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/mapping/registration.rs)
- [lookup.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/mapping/lookup.rs)
- [planning.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/routing/planning.rs)
- [lowering.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/routing/lowering.rs)
- [records.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/diagnostics/records.rs)
- [adapter.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/harness/adapter.rs)

## Implementation Phases

Milestone 2 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished precision foundations with ad hoc widening.

### Phase M2.0 - Slice Taxonomy And Boundary Lock

Purpose:

- define the admitted truth delta surface categories and subscription-slice categories
- lock what precision the bridge owns versus what remains parent-runtime authority
- define the explicit out-of-scope slice categories that must fail rather than drift

Required work:

- inventory truth-side aspect surfaces currently exposed or realistically derivable
- inventory signal-side slice shapes that can be admitted without leaking ownership
- define the first closed Milestone 2 slice taxonomy
- define the fallback classes and rejection policy for non-admitted surfaces

Exit criteria:

- the admitted slice taxonomy is closed and explicit
- unsupported categories are named rather than left to future interpretation
- no unresolved ambiguity remains about whether Milestone 2 owns slice identity

### Phase M2.1 - Registration And Freeze Semantics

Purpose:

- make fine-grained mapping truth explicit before route planning exists

Required work:

- define aspect registration declarations
- define subscription-slice declarations and fallback policy declarations
- implement canonical freeze ordering
- reject ambiguous overlap during freeze

Exit criteria:

- routing cannot occur without frozen fine-grained registration truth
- registration order does not affect later route identity
- ambiguous registration fails before route planning

### Phase M2.2 - Normalized Surface Derivation And Eligibility

Purpose:

- derive canonical truth delta surfaces once from the committed patch envelope

Required work:

- define `TruthDeltaSurface` and `NormalizedTruthDeltaSurfaceSet`
- canonicalize duplicate surface entries
- classify surfaces into matched, suppressed, ambiguous, unsupported, or fallback-admitted eligibility
- define exact digest basis for normalized surface identity

Exit criteria:

- the bridge can produce one canonical normalized surface set per committed patch
- every surface has exactly one eligibility classification
- later phases do not need to rediscover truth-side surface meaning

### Phase M2.3 - Planned Match Sets And Lowered Slice Artifacts

Purpose:

- lower canonical surfaces plus frozen mapping truth into canonical slice artifacts

Required work:

- define `PlannedAspectMatchSet`
- derive `SubscriptionSliceReadPacket`
- define `LoweredSubscriptionSliceArtifact`
- record fallback and suppression outcomes canonically
- add exact route counters

Exit criteria:

- identical canonical surface sets lower to identical slice artifacts
- slice read breadth is fixed during planning
- lowered artifacts remain replay-safe and diagnostics-tier-invariant

### Phase M2.4 - Delivery, Replay, And Certification

Purpose:

- prove that fine-grained bridge precision is stable, explicit, and replayable

Required work:

- integrate slice artifact delivery with the signal sink
- add canonical slice route records and replay records
- add explanation reconstruction
- add hostile harness suites and counter certification

Exit criteria:

- fine-grained routing survives replay after restart
- ambiguity, suppression, and fallback behavior are typed and diagnosable
- all roadmap acceptance evidence is covered by bridge-native harness scenarios

## Explicit Failure Taxonomy For Milestone 2

Milestone 2 must ship typed bridge failures for at least:

- missing slice mapping registration
- ambiguous slice mapping registration
- unsupported truth delta surface category
- unsupported subscription-slice category
- invalid fallback admission
- inconsistent normalized surface digest
- slice read packet construction failure
- slice delivery rejection
- slice replay mismatch
- slice canonical artifact decode or compatibility failure

These are bridge failures, not raw string bubbles from parent runtimes.

## Anti-Patterns Explicitly Rejected

- treating field names, lens names, or region names as untyped free-form public authority
- host-local callback routing for fine-grained precision
- silently widening from fine-grained slice routing to whole-entity invalidation
- discovering fallback behavior during delivery instead of declaring it during registration
- re-reading live truth to decide slice matching after planning completed
- burying ambiguity or suppression in explanations without carrying it in canonical route truth
- exposing parent-runtime internals as the slice-routing API

## Sequencing Notes

Milestone 2 must land before:

- lineage-aware subscription continuity, because continuity without explicit
  slice identity is structurally under-specified
- historical and branch-aware evaluation, because replayable history is weaker
  if the bridge cannot state what precise surface was consumed
- bulk routing and scale-path planning, because breadth optimization is dishonest
  until precision semantics are explicit

Milestone 2 must not attempt to pre-solve:

- identity evolution and continuity rules
- historical routing semantics
- branch coordination
- speculative preview flows
- merge-aware routing

Those become stronger because Milestone 2 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it defines the first canonical precision boundary between
truth deltas and derived dependency slices.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of silent widening, iteration-order drift, and live-state rediscovery.

The milestone preserves authority boundaries because truth still owns truth
meaning and snapshots, signal still owns dependency execution, and the bridge
owns only the mapping contract and canonical route artifacts between them.

The milestone defines proof obligations rather than implementation chores
because replay parity, diagnostics-tier invariance, explicit fallback typing,
and exact precision certification are required for closeout.

A competent engineer should be able to map this spec into honest types,
subsystems, route packets, counters, and harness suites without inventing the
architecture during implementation.
