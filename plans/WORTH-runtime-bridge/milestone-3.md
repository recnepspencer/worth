# Milestone 3 Engineering Spec: Lineage-Aware Subscription Continuity

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2.md)
>
> **Prior closeout:** [milestone-2-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2-closeout.md)
>
> **Milestone 2 hardening companion:** [milestone-2-envelope-and-planning-hardening.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2-envelope-and-planning-hardening.md)
>
> **Primary architectural driver:** preserve fine-grained bridge subscriptions across truth-side identity evolution without collapsing truth lineage authority into bridge-owned heuristics or signal-owned node identity
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

Milestones 1 and 2 established that:

- committed truth enters the bridge through a compatibility-checked canonical envelope
- truth deltas normalize into canonical fine-grained surfaces
- bridge-owned mapping lowers those surfaces into canonical subscription slices
- signal evaluation stays pinned to stable snapshots and replay-safe bridge artifacts

That is enough to route precise changes while truth identity stays stable.

It is not enough for the workloads the bridge explicitly claims to support:

- topology edits that replace one entity with another
- split operations where one truth identity becomes several descendants
- merge-like or correspondence-bearing history where continuity is possible only
  through explicit lineage evidence
- branch-local histories where "same latest field" is weaker than "same
  lineage-backed continuity decision"

Milestone 3 exists because fine-grained precision without continuity still
fails under real identity evolution. The bridge must be able to say:

`this prior subscription slice continues to these exact successor slices because canonical truth lineage admitted that continuity under this snapshot and branch`

or:

`continuity is rejected because truth lineage did not prove a unique successor`

The bridge still does not own truth lineage semantics. It owns:

- continuity request shaping
- continuity classification over truth-owned lineage artifacts
- deterministic remapping or rejection of prior subscription slices
- replay-safe diagnostics explaining how continuity was preserved, split, merged,
  or denied

## Goal

Keep bridge subscription continuity deterministic, explicit, and replay-safe
when truth identity evolves through replace, split, merge-like, retire, or
branch-local history transitions.

## Why This Milestone Exists

Milestone 3 belongs immediately after Milestone 2 because continuity depends on
surfaces that only Milestone 2 made explicit:

- canonical truth-delta surfaces
- canonical subscription-slice identity
- planning and lowering provenance
- replay-safe route records
- explicit fallback, suppression, and unsupported-path classification

Without Milestone 2, Milestone 3 would have to remap coarse entities rather
than precise dependency slices. Without Milestone 3, later milestones inherit a
fundamental identity bug:

- historical evaluation can tell you what snapshot ran, but not whether the
  subscription you see is the intended descendant of an earlier one
- branch-aware evaluation can replay branch-local truth, but not preserve the
  subscriber identity story across rewires and replacements
- bulk planning can scale routing breadth, but not continuity breadth
- structural-identity-assisted remapping would have no authoritative continuity
  substrate to subordinate itself to

Milestone 3 therefore earns its place in the roadmap by solving the next real
structural problem after precision: continuity under identity evolution.

## Adversarial Constraint

Milestone 3 must survive the following hostile condition:

> A long-lived system with fine-grained bridge subscriptions, branch-local
> history, replace and split mutations, merge-like correspondence events, and
> replay after restart must resolve the same continuity request into the same
> canonical continuity outcome every time, while consulting only truth-owned
> lineage authority and the planned snapshot context, and while rejecting
> ambiguous or unsupported continuity rather than drifting silently.

Concretely, the design must remain correct when all of the following are true:

- a subscribed truth slice is retired and replaced by one successor
- a subscribed truth slice is split into multiple descendants
- continuity requires historical ID resolution rather than latest-identity lookup
- continuity candidates differ across branches
- multiple successor candidates share structural similarity but only one is
  lineage-admitted
- diagnostics richness changes between environments
- replay occurs from canonical continuity artifacts after restart
- some merge-like or correspondence-bearing histories remain unsupported and
  must fail explicitly rather than heuristically continue

If any supported path:

- preserves continuity by latest-state coincidence instead of truth lineage
- widens to arbitrary descendant sets without a typed continuity class
- lets host iteration order choose successor ordering
- re-queries live mutable truth outside the planned snapshot boundary
- cannot explain why continuity continued, split, merged, or failed

then Milestone 3 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- continuity is a first-class bridge subsystem, not a helper hidden inside
  routing or diagnostics
- truth runtime remains the authority for lineage events, historical resolution,
  and correspondence evidence
- signal runtime remains the authority for derived node identity and execution
- the bridge owns continuity classification and remapping artifacts between
  those two authorities
- ambiguous continuity must fail explicitly unless a bridge-owned, typed
  continuity class admits a deterministic multi-successor outcome
- historical lookup needed for continuity is a planned bridge read concern, not
  an ad hoc recovery step during delivery
- Milestone 3 extends Milestone 2 route records and slice identity; it does not
  replace them with a new protocol

Normative consequence:

- host-local "just follow the newest ID" logic is out of spec
- structural similarity without truth lineage authority is out of spec
- merge-like continuity inferred from unordered candidate sets is out of spec
- silent subscription drop or silent subscription drift is out of spec

## Scope

### In Scope

- bridge-owned continuity request and continuity outcome artifacts
- truth-owned lineage and historical-resolution adapter surfaces narrow enough
  for bridge planning
- deterministic remapping for replace-style continuity
- deterministic multi-successor remapping for admitted split-style continuity
- explicit handling for merge-like continuity, continuity rejection, and
  unsupported continuity classes
- planned historical ID resolution where a prior subscribed identity must be
  resolved against branch-local lineage
- diagnostics and replay artifacts showing continuity evidence and outcome
- harness certification for continuity preservation, rejection, replay parity,
  and branch-local continuity behavior

### Explicitly Out Of Scope

- branch-aware evaluation as a product surface beyond the continuity-local
  branch context needed for lineage resolution
- full merge-aware bridge semantics across multi-parent truth histories
- structural-identity-assisted remapping
- speculative preview continuity
- bridge-mediated writeback
- generalized host-facing lineage tooling outside the continuity contract

Milestone 3 must remain focused on bridge subscription continuity, not absorb
the later merge or historical product surfaces wholesale.

Explicit Milestone 3 merge-like restriction:

- Milestone 3 does not introduce general merge-aware continuity
- merge-bearing or correspondence-bearing histories may only continue when the
  truth-owned lineage export returns a single canonical successor already
  lowered as an admitted bridge continuity class
- otherwise the bridge must emit `RejectedUnsupportedContinuityClass` or
  `RejectedAmbiguousSuccessor`

## Governing Design Rules

### 1. Truth Owns Lineage Meaning, Bridge Owns Continuity Lowering

The truth runtime defines:

- lineage events
- lineage graph / history access
- correspondence and historical resolution semantics
- branch-local lineage authority

The signal runtime defines:

- node identity
- node lineage diagnostics
- derived recomputation behavior after invalidation arrives

The bridge defines:

- continuity request shape
- continuity classes
- remapped subscription-slice artifacts
- continuity diagnostics and replay records

The bridge must not define its own lineage ontology.

The bridge facade is the only public continuity surface.

External consumers may depend on:

- bridge continuity request types
- bridge continuity result types
- bridge continuity diagnostics and replay records

External consumers may not depend on:

- relational lineage graph internals
- relational historical traversal helpers
- internal bridge continuity classification modules
- internal bridge lowering or diagnostics assembly modules

If a future overhaul of `worth-relational` preserves the narrow bridge-owned
lineage adapter contract, canonical continuity request basis, and returned
continuity authority basis, the bridge should not need architectural surgery.
If the bridge must be rewritten because consumers depended on relational
internals rather than bridge continuity contracts, the facade boundary was too
weak.

### 2. Continuity Must Be Classified, Not Assumed

Each prior subscription slice considered for continuity must resolve to exactly
one bridge-owned continuity outcome.

Required Milestone 3 classifications:

- `ContinuesAsSingleSuccessor`
- `ContinuesAsSplitSuccessors`
- `ContinuesViaTruthLoweredCanonicalMergeSuccessor`
- `RejectedAmbiguousSuccessor`
- `RejectedNoAuthoritativeSuccessor`
- `RejectedUnsupportedContinuityClass`

Rules:

- every successful continuity outcome must identify the exact prior slice and
  exact successor slice set
- every rejection must identify the continuity boundary and typed failure class
- a continuity request may not be simultaneously continued and rejected

### 3. Continuity Identity Must Be Canonical

Milestone 3 must introduce bridge-owned continuity identity sufficient for:

- replay
- diagnostics reconstruction
- future historical and branch-aware milestones
- future structural remapping work

Each continuity artifact must define:

- identity-bearing prior slice fields
- identity-bearing successor slice fields
- ordering key for successor sets
- deduplication basis for repeated continuity edges
- digest basis
- explanatory-only fields excluded from identity

Canonicality must cover at least:

- continuity request ordering
- lineage evidence ordering
- continuity outcome ordering
- successor slice ordering
- rejection ordering

### 4. Historical Resolution Must Be Planned

If continuity requires historical ID lookup, the bridge must plan that lookup
before delivery.

Rules:

- continuity planning derives the exact lineage/historical query packet
- later phases consume the planned lineage packet immutably
- delivery must not discover new lineage candidates by widening scope on demand
- the bridge may consult truth-owned lineage access during planning only through
  explicit bridge-owned adapter contracts

### 5. Replace, Split, And Merge-Like Paths Must Stay Distinct

These are not one generic "remap somehow" path.

- replace continuity means one prior slice resolves to one successor slice
- split continuity means one prior slice resolves to an ordered set of admitted
  successor slices
- merge-like continuity in Milestone 3 is closed to one admitted class only:
  `TruthLoweredCanonicalMergeSuccessor`
- that class is admitted only when the truth-owned lineage adapter returns one
  already-reconciled canonical successor with no competing successor set
- every other merge-bearing or correspondence-bearing continuity case is
  rejected in Milestone 3 rather than heuristically continued

If these classes share code, the abstraction must preserve their distinct cost,
failure topology, and correctness rules.

### 6. Plan / Resolve / Lower / Deliver Separation Is Mandatory

Milestone 3 extends the bridge proof chain:

- validated committed envelope
- normalized truth-delta surface set
- planned continuity request set
- resolved lineage continuity set
- lowered continuity remap artifact
- delivered continuity invalidation result

Resolution must not be rediscovered in delivery. Delivery consumes lowered
continuity truth only.

### 7. Continuity Resolution Must Be Proof-Carrying

Milestone 3 must continue satisfying Architectural Laws 30 and 41.

Representative progression:

```rust
pub struct PlannedContinuityRequestSet { ... }
pub struct EligibleContinuityRequestSet { ... }
pub struct ResolvedLineageContinuitySet { ... }
pub struct LoweredContinuityArtifact { ... }
pub struct DeliveredContinuityResult { ... }
```

Rules:

- constructors for proof-bearing continuity packets must be sealed to the
  proving modules
- later phases must consume the exact proof type produced upstream, not a
  weaker bag of prior slices and lineage hints
- continuity rejection, ambiguity classification, and unsupported-class
  classification become part of the proof chain rather than explanation-only
  side channels
- delivery cannot accept a weaker packet than lowering produced
- any runtime check for a property that an earlier continuity phase already
  proved is a design failure

### 8. Canonicality Must Be Mechanically Declared

Milestone 3 must not use "lineage-backed continuity" as a vibe phrase. It must
define continuity authority precisely enough that two independent
implementations could lower the same continuity request into the same artifact.

For every Milestone 3 canonical artifact, the spec must define:

- ordered input set
- ordering key
- deduplication rule
- digest basis
- identity-bearing versus explanatory-only fields

Canonicality must cover at least:

- prior subscription slice ordering
- continuity request-set ordering
- branch identity basis
- historical lineage packet ordering
- lineage evidence ordering
- successor slice ordering
- rejection ordering
- continuity record digest basis

If any continuity artifact can vary because candidate lineage evidence arrives
through map iteration order, host registration order, or unconstrained branch
history traversal order, the design is out of spec.

### 9. Continuity Authority Basis Must Be Explicit

Once a continuity request set exists, continuity truth must be derived only
from:

- canonical prior subscription slice identity
- canonical route identity and snapshot identity
- explicit branch identity
- truth-owned lineage evidence returned through the bridge lineage adapter
- explicit bridge-owned continuity classification rules

Planning and lowering may preserve richer lineage explanation fields, but they
must not allow raw relational history spellings, host-local candidate ordering,
or incidental traversal differences to affect continuity identity after the
canonical continuity request set exists.

At minimum, the continuity digest basis must state:

- whether the prior basis is one slice or an ordered slice set
- what exact branch and snapshot identities are authority-bearing
- what lineage event span or historical-resolution digest participates in the
  identity
- what successor ordering rule applies
- what rejection basis participates in canonical failure identity

### 10. Diagnostics Are Derived From Canonical Continuity Truth

Operational truth for Milestone 3 is:

- canonical continuity request identity
- canonical continuity resolution identity
- canonical remapped slice artifact identity
- typed continuity classifications and rejections
- counters

Rich explanation remains derived and policy-shaped.

### 11. Continuity Breadth And Complexity Must Be Declared

Milestone 3 must make its normal-path breadth and complexity claims explicit.

At minimum, the spec must preserve these structural expectations:

- continuity planning scales with prior subscribed slice count plus planned
  lineage packet width, not whole-branch history size on the normal path
- continuity lowering scales with admitted successor-set width, not all visible
  descendants in storage
- rejection must occur before broad successor materialization when lineage
  authority is absent or unsupported

Forbidden normal-path costs:

- full branch-history scans to discover successors for every prior slice
- broad descendant enumeration before proving the continuity class is admitted
- delivery-time re-traversal of lineage history because planning failed to carry
  enough proof forward

Milestone 3 should eventually register named complexity contracts and proof
tests for continuity planning and lineage resolution. This milestone spec must
at least make those boundaries explicit so later implementation cannot hide
breadth under innocent-looking APIs.

## Target Runtime Model

### 1. Public Surface Growth

Milestone 3 should extend `worth-runtime-bridge` with continuity concepts such
as:

```rust
pub struct BridgeContinuityRequest { ... }
pub struct BridgePlannedContinuityRoute { ... }
pub struct BridgeContinuityResult { ... }
pub struct BridgeContinuityArtifact { ... }
pub struct BridgeHistoricalLineagePacket { ... }

pub enum BridgeContinuityClass {
    SingleSuccessor,
    SplitSuccessors,
    TruthLoweredCanonicalMergeSuccessor,
}

pub enum BridgeContinuityFailure {
    MissingLineageAuthority { ... },
    AmbiguousSuccessor { ... },
    UnsupportedContinuityClass { ... },
    HistoricalResolutionRejected { ... },
    ContinuityReplayMismatch { ... },
}
```

Design rules:

- planning and delivery remain separate public boundary crossings
- continuity requests consume canonical bridge route truth, not host-local
  identity bags
- the bridge facade exposes bridge nouns only
- continuity routing remains an extension of Milestone 2 proof-carrying routing

### 2. Representative Continuity Input Contract

Milestone 3 needs an explicit prior-subscription continuity request built on
Milestone 2 slice identity.

Representative shape:

```rust
pub struct PriorSubscriptionSlice {
    prior_slice_identity: BridgeSubscriptionSliceIdentity,
    truth_entity_identity: TruthEntityIdentity,
    truth_surface_kind: TruthDeltaSurfaceKind,
    source_branch: TruthBranchIdentity,
}

pub struct PlannedContinuityRequestSet {
    route_identity: BridgeRouteIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    prior_slices: Vec<PriorSubscriptionSlice>,
    lineage_packet: BridgeHistoricalLineagePacket,
    digest: BridgeContinuityRequestDigest,
}
```

Rules:

- the continuity request set is derived exactly once from canonical route truth
  plus prior bridge subscription identity
- duplicate prior slices must collapse canonically before lineage resolution
- branch identity is explicit on the request set
- explanatory labels must not participate in digest identity

### 3. Truth-Owned Lineage Adapter Contract

Milestone 3 should depend on narrow bridge-owned lineage contracts rather than
direct relational facade reach-through.

Representative shape:

```rust
pub trait RelationalLineageBridgeAdapter {
    type Error;

    fn resolve_subscription_continuity(
        &self,
        packet: &BridgeHistoricalLineagePacket,
    ) -> Result<ResolvedLineageContinuitySet, Self::Error>;
}
```

The returned truth must be authoritative enough for the bridge to lower
continuity without inventing lineage semantics.

### 3.1 Relational Evolution Tolerance

Milestone 3 must assume `worth-relational` may evolve internally over time.

The bridge should tolerate:

- relational lineage storage rewrites
- lineage graph indexing changes
- historical traversal implementation changes
- correspondence or merge-evidence internal representation changes

The bridge should not depend on:

- direct access to relational lineage storage layout
- direct access to relational graph node internals
- broad relational facade reach-through for convenience
- relational helper APIs whose semantics are not part of the narrow bridge contract

Required implication:

- if relational evolves but still exports the same narrow bridge-owned lineage
  adapter contract with the same canonical authority basis, bridge changes
  should be local adapter maintenance rather than milestone-level redesign

### 4. Continuity Resolution Contract

Representative shape:

```rust
pub struct ResolvedLineageContinuitySet {
    route_identity: BridgeRouteIdentity,
    continuity_digest: BridgeContinuityResolutionDigest,
    continuity_entries: Vec<ResolvedLineageContinuity>,
    counters: BridgeContinuityCounters,
}

pub struct ResolvedLineageContinuity {
    prior_slice_identity: BridgeSubscriptionSliceIdentity,
    continuity_class: BridgeContinuityClass,
    successor_slices: Vec<BridgeSubscriptionSliceIdentity>,
    lineage_digest: LineageResolutionDigest,
}
```

Rules:

- canonical ordering is mandatory across prior slices, lineage evidence, and
  successor slices
- lineage evidence must be carried strongly enough for replay and explanation
- the returned lineage evidence must already be canonicalized by an explicit
  bridge-owned contract rather than left as an unordered host bag
- unsupported continuity classes must fail explicitly rather than degrade into
  best-effort remaps

### 5. Lowered Continuity Artifact

Representative shape:

```rust
pub struct BridgeContinuityArtifact {
    route_identity: BridgeRouteIdentity,
    continuity_identity: BridgeContinuityIdentity,
    remapped_slices: CanonicalSubscriptionSlices,
    continuity_outcomes: CanonicalContinuityOutcomes,
    snapshot_token: BridgeSnapshotToken,
    counters: BridgeContinuityCounters,
}
```

Rules:

- remapped slices are canonical bridge truth
- continuity outcomes remain replay-safe and diagnostics-tier-invariant
- delivery consumes lowered continuity truth only
- signal receives canonical remapped slice invalidation, not relational lineage internals

### 6. Canonical Continuity Replay Record

Milestone 3 should make replay authority as concrete as its routing authority.

Representative shape:

```rust
pub struct BridgeCanonicalContinuityRecord {
    route_identity: BridgeRouteIdentity,
    continuity_request_digest: BridgeContinuityRequestDigest,
    continuity_resolution_digest: BridgeContinuityResolutionDigest,
    continuity_artifact_identity: BridgeContinuityIdentity,
    source_snapshot: TruthSnapshotIdentity,
    source_branch: TruthBranchIdentity,
    counters: BridgeContinuityCounters,
    schema_version: BridgeCanonicalContinuitySchemaVersion,
}
```

Rules:

- replay must proceed from canonical continuity records, not ad hoc in-memory
  lineage state
- continuity replay identity must be derived from canonical request and
  resolution truth, not regenerated from whatever relational history currently
  happens to exist
- compatibility failures and replay mismatches must remain typed and explicit

## Phases

### Phase 1: Continuity Vocabulary And Authority Boundary

Phase 1 exists to make continuity structurally representable without smuggling
truth lineage authority into bridge heuristics.

Milestone 3 must first define:

- bridge-owned continuity classes
- bridge-owned continuity request and rejection vocabulary
- narrow truth-owned lineage adapter surfaces
- explicit unsupported continuity classes for Milestone 3
- exact admitted merge-like continuity restriction
- canonical continuity authority basis and digest basis

This phase leaves the system in a coherent state where:

- continuity can be requested explicitly from prior subscription identity
- unsupported continuity classes fail by design instead of drifting later
- there is no ambiguity about what truth owns versus what the bridge owns

### Phase 2: Deterministic Continuity Resolution And Lowered Remap Artifacts

Phase 2 exists to turn the vocabulary into canonical bridge truth.

Milestone 3 must then implement:

- canonical continuity request-set derivation
- eligibility classification for continuity requests before lineage resolution
- planned historical lineage packet derivation
- deterministic continuity classification over truth-owned lineage results
- lowered remap artifacts for replace, split, and admitted merge-like classes
- exact continuity counters and canonical outcome identity

This phase leaves the system in a coherent state where:

- identical continuity inputs lower to identical continuity artifacts
- replace and split continuity stay distinct and explainable
- rejection is typed and canonical instead of buried in logs

### Phase 3: Replay, Diagnostics, And Continuity Certification

Phase 3 exists to prove the continuity model is trustworthy.

Milestone 3 must finally ship:

- canonical continuity route records and replay records
- explanation reconstruction for continuity continuation, split, merge-like
  continuation, and rejection
- hostile harness suites covering replace, split, branch divergence, ambiguity,
  unsupported merge-like continuity, and replay parity
- exact counter assertions for representative continuity scenarios

This phase leaves the system in a coherent state where:

- continuity survives replay after restart
- branch-local identity evolution remains diagnosable
- later historical, merge-aware, and scale-path milestones can build on stable
  continuity artifacts

## Must Ship

- bridge-owned continuity request, continuity class, and continuity outcome artifacts
- narrow truth-owned lineage resolution adapter surface for bridge planning
- planned historical lineage packet derivation
- deterministic replace-style continuity lowering
- deterministic split-style continuity lowering with canonical successor ordering
- explicit handling for merge-like continuity, ambiguity rejection, and
  unsupported continuity classes
- canonical continuity diagnostics and replay artifacts
- typed failures for missing lineage authority, ambiguous successor sets,
  unsupported continuity class, historical resolution rejection, and replay mismatch
- harness certification lanes for continuity preservation, explicit rejection,
  branch-local parity, and replay parity

## Must Preserve

- truth runtime remains the authority for lineage semantics and historical resolution
- signal runtime remains the authority for node identity and execution
- no live mutable truth reads during continuity resolution
- no hidden widening from one prior slice into arbitrary descendant scans
- canonical ordering and replay-safe continuity identity
- Milestone 2 slice identity remains the continuity substrate
- clean facade boundaries rather than wide parent-runtime reach-through

## Acceptance Evidence

Milestone 3 is complete only when the bridge harness can prove:

- identical canonical continuity requests lower to identical continuity artifacts
- replace-style truth evolution preserves continuity to exactly one successor
  slice when truth lineage admits it
- split-style truth evolution preserves continuity to the canonical successor
  slice set when truth lineage admits it
- ambiguous or unsupported continuity fails explicitly and typed
- branch-local lineage differences produce explicit continuity differences rather
  than accidental cross-branch reuse
- diagnostics richness changes explanation only, not continuity truth
- replay from canonical continuity artifacts matches original continuity behavior

## Architectural Notes

### Expected Internal Subdomains

Milestone 3 should extend the bridge crate with subdomains such as:

- `continuity/requests/`
- `continuity/lineage/`
- `continuity/classification/`
- `continuity/lowering/`
- `diagnostics/continuity/`
- `harness/fixtures/continuity_replace.rs`
- `harness/fixtures/continuity_split.rs`
- `harness/fixtures/continuity_rejections.rs`

This follows workspace domain standards:

- lineage packet planning is not the same responsibility as continuity
  classification
- replace continuity is not the same responsibility as split continuity
- canonical continuity records are not the same responsibility as explanation reconstruction

### Minimum Counter Floor

Milestone 3 must add counters such as:

- `continuity_request_count`
- `continuity_prior_slice_count`
- `lineage_resolution_request_count`
- `lineage_resolution_candidate_count`
- `continuity_single_successor_count`
- `continuity_split_successor_count`
- `continuity_merge_like_successor_count`
- `continuity_rejection_count`
- `continuity_ambiguity_count`
- `continuity_replay_mismatch_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Rejection Policy

Milestone 3 must carry rejection structurally rather than narratively.

Required rejection classes:

- `RejectedNoAuthoritativeSuccessor`
- `RejectedAmbiguousSuccessor`
- `RejectedUnsupportedContinuityClass`
- `RejectedHistoricalResolutionFailure`

Rules:

- every prior slice gets exactly one outcome
- rejection remains visible in canonical route truth
- rejection must include the continuity boundary that failed
- rejection must not degrade into coarse invalidation unless a later milestone
  explicitly introduces and certifies such a policy

## Test And Harness Model

Milestone 3 must follow the same structural testing discipline as Milestones 1
and 2.

Expected first-class test surfaces:

- replace continuity scenarios
- split continuity scenarios
- merge-like continuity and rejection scenarios
- branch divergence continuity scenarios
- diagnostics-tier invariance scenarios
- replay parity and replay drift scenarios
- counter certification scenarios

Milestone 3 is not complete with only direct fixture tests. It must establish a
real continuity certification surface on top of `worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for continuity worlds and identity-evolution cases
- `MutationBatch` for replace, split, retire, and branch-divergence truth mutations
- `ExecutionRequest` for continuity planning, lowering, delivery, replay, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, diagnostics-tier, and branch-divergence sweeps
- `ParitySuite` for profile-to-profile continuity parity
- `CertificationMatrix` for adversarial continuity coverage across multiple
  branch and identity-evolution profiles

Minimum certification families:

- fixed deterministic replace-continuity fixtures
- fixed deterministic split-continuity fixtures
- seeded branch-divergence continuity matrices
- replay-after-restart continuity certification from canonical artifacts
- unsupported merge-like continuity rejection certification
- exact counter assertions for named continuity candidate-width and
  successor-width scenarios

Rules:

- continuity tests must describe prior slice identity, truth evolution, and
  expected successor sets through harness fixtures rather than ad hoc local setup
- profile sweeps must use `ExecutionProfile`, not local booleans or enums
- continuity parity checks must use `ParitySuite` where the concern is run-to-run equivalence
- adversarial continuity sweeps must use `CertificationMatrix` where the
  concern is multi-profile hostile coverage

Minimum representative test names:

- `tests::continuity::replace_surface_continues_to_single_successor_slice`
- `tests::continuity::split_surface_continues_to_canonical_successor_slice_set`
- `tests::continuity::ambiguous_successor_set_fails_explicitly`
- `tests::continuity::branch_local_lineage_changes_continuity_outcome_explicitly`
- `tests::continuity::replayed_continuity_route_matches_original_canonical_artifact`

## Target API And Module Plan

### New Files Expected

- `crates/worth-runtime-bridge/src/continuity/mod.rs`
- `crates/worth-runtime-bridge/src/continuity/requests.rs`
- `crates/worth-runtime-bridge/src/continuity/lineage_packet.rs`
- `crates/worth-runtime-bridge/src/continuity/resolution.rs`
- `crates/worth-runtime-bridge/src/continuity/classification.rs`
- `crates/worth-runtime-bridge/src/continuity/lowering.rs`
- `crates/worth-runtime-bridge/src/continuity/counters.rs`
- `crates/worth-runtime-bridge/src/diagnostics/continuity.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/continuity_replace.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/continuity_split.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/continuity_rejections.rs`
- `crates/worth-runtime-bridge/src/tests/continuity/replace.rs`
- `crates/worth-runtime-bridge/src/tests/continuity/split.rs`
- `crates/worth-runtime-bridge/src/tests/continuity/rejections.rs`
- `crates/worth-runtime-bridge/src/tests/continuity/replay.rs`

### Existing Files Expected To Change

- [facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade.rs)
- [planning.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/routing/planning.rs)
- [lowering.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/routing/lowering.rs)
- [records.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/diagnostics/records.rs)
- [adapter.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/harness/adapter.rs)
- [facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)

## Implementation Phases

Milestone 3 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished continuity foundations with host-local ID
glue.

### Phase M3.0 - Continuity Taxonomy And Boundary Lock

Purpose:

- define admitted continuity classes
- lock what truth lineage authority exports versus what the bridge classifies
- define explicit unsupported continuity classes

Required work:

- inventory the truth-owned lineage and historical-resolution surfaces the
  bridge can depend on without reaching through broad relational facades
- define the first closed Milestone 3 continuity taxonomy
- define the exact admitted merge-like continuity class and rejection policy
- define canonical continuity authority basis and digest inputs

Exit criteria:

- continuity vocabulary is closed and explicit
- unsupported classes are named rather than deferred
- there is no unresolved ambiguity about authority boundaries

### Phase M3.1 - Planned Lineage Packet And Historical Resolution Contract

Purpose:

- make continuity depend on planned lineage queries rather than ad hoc lookups

Required work:

- define `PriorSubscriptionSlice` and `PlannedContinuityRequestSet`
- define `EligibleContinuityRequestSet`
- define `BridgeHistoricalLineagePacket`
- define the narrow bridge-owned lineage adapter contract
- define the exact branch and snapshot basis carried into continuity planning

Exit criteria:

- the bridge can derive one canonical continuity request set per route
- historical resolution is explicit and branch-aware
- delivery no longer needs to discover lineage query breadth

### Phase M3.2 - Canonical Continuity Classification And Lowering

Purpose:

- lower planned continuity requests into canonical remap artifacts

Required work:

- define `ResolvedLineageContinuitySet`
- define `BridgeContinuityArtifact`
- classify requests into single-successor, split-successor, admitted
  merge-like successor, ambiguous rejection, no-authority rejection, or
  unsupported-class rejection
- define exact canonical ordering and digest basis for continuity outcomes
- add exact continuity counters

Exit criteria:

- replace and split continuity lower deterministically
- ambiguity and unsupported continuity are typed and canonical
- continuity counters and digest bases are specified and test-covered

### Phase M3.3 - Replay, Diagnostics, And Certification

Purpose:

- make continuity certifiable rather than plausible

Required work:

- add canonical continuity route records and replay records
- add explanation reconstruction over canonical continuity truth
- add `worth-harness` continuity fixtures, parity suites, and certification matrices
- add hostile replay, branch-divergence, and unsupported-merge rejection lanes

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- replay validates continuity parity directly
- diagnostics-tier changes richness only, not continuity truth

## Explicit Failure Taxonomy For Milestone 3

Milestone 3 must ship typed bridge failures for at least:

- missing lineage authority export
- unsupported continuity class
- ambiguous successor set
- rejected historical lineage resolution
- branch mismatch for continuity request
- continuity artifact decode or compatibility failure
- continuity replay mismatch
- continuity delivery rejection

These are bridge failures, not raw parent-runtime strings.

## Anti-Patterns Explicitly Rejected

- treating "latest truth identity" as continuity authority
- remapping subscription slices from structural similarity alone
- collapsing replace, split, and merge-like continuity into one generic path
- discovering continuity breadth during delivery
- hiding ambiguity or rejection inside explanations only
- exposing relational lineage internals as the bridge continuity API
- silently dropping subscriptions when continuity is unresolved

## Sequencing Notes

Milestone 3 must land before:

- historical and branch-aware evaluation, because those surfaces are weaker if
  subscription continuity across identity evolution is undefined
- bulk routing and scale-path planning, because continuity breadth must be made
  explicit before it can be optimized honestly
- structural-identity-assisted remapping, because structural hints must be
  subordinate to lineage-backed continuity rather than replace it

Milestone 3 must not attempt to pre-solve:

- full merge-aware bridge semantics
- general historical evaluation as a user-facing surface
- speculative preview continuity
- structural-identity policy

Those become stronger because Milestone 3 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the bridge cannot claim stable cross-runtime subscription
behavior until identity evolution is handled explicitly.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of latest-ID reuse, heuristic remapping, and branch-insensitive continuity.

The milestone preserves authority boundaries because truth still owns lineage
meaning, signal still owns derived node identity, and the bridge owns only the
continuity contract between them.

The milestone defines proof obligations rather than implementation chores
because deterministic replace/split handling, explicit ambiguity rejection,
replay parity, and branch-local continuity certification are required for
closeout.

A competent engineer should be able to map this spec into honest types,
subsystems, continuity packets, counters, and harness suites without inventing
the architecture during implementation.

## Closeout Standard

Milestone 3 is complete only when all of the following are true:

- prior bridge subscription slices can be lowered into one canonical continuity request set
- continuity planning consults only planned, explicit lineage authority surfaces
- replace continuity and admitted split continuity lower deterministically into
  canonical remap artifacts
- unsupported or ambiguous continuity fails explicitly and typed
- merge-bearing histories outside the one admitted Milestone 3 merge-like class
  are rejected explicitly rather than heuristically continued
- continuity truth is replay-safe and diagnostics-tier-invariant
- harness certification proves replace continuity, split continuity, branch-local
  divergence behavior, replay parity, and explicit rejection behavior under
  hostile identity evolution pressure

If code lands but continuity still depends on latest-ID coincidence, unordered
lineage candidate sets, host-local remap glue, or explanation-only rejection,
Milestone 3 is not complete.
