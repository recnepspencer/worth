# Milestone 2 Hardening Plan: Envelope And Planning Integrity

> **Status:** Planned hardening companion spec
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Primary milestone:** [milestone-2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2.md)
>
> **Prior milestone:** [milestone-1.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-1.md)
>
> **Prior closeout:** [milestone-1-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-1-closeout.md)
>
> **Companion crate reference:** [worth-relational facade bridge export](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-relational/src/facade.rs)
>
> **Primary architectural driver:** make bridge envelope ingress, route planning, and lowering as proof-carrying and authority-shaped as the rest of WORTH

## Summary

Milestone 2 already established the semantic center of the bridge:

- truth-side fine-grained deltas normalize into canonical truth surfaces
- bridge-owned mapping lowers those surfaces into canonical subscription slices
- signal delivery sees slice-native invalidation truth instead of host-local callback logic

That semantic center is correct.

What remains softer than the rest of WORTH is not the high-level design. It is
the authority posture of the envelope and planning proof chain.

Today, the bridge still performs more repair and canonicalization internally
than a production-grade boundary should need to, and its lowering phase is
cleaner than ad hoc code but weaker than WORTH's strongest lowered-plan
patterns.

This hardening plan exists to close that gap without changing Milestone 2's
semantic ownership model.

The governing hardening rule is:

`producer truth must arrive already authority-shaped enough that bridge normalization narrows, certifies, and proves meaning once; planning must lower once into explicit provenance-bearing artifacts; execution must consume only validated lowered bridge truth`

## Goal

Make bridge envelope ingress and route planning fully proof-carrying,
compatibility-aware, replay-safe, and structurally aligned with WORTH
lowered-plan standards, so no supported path depends on repair-by-convention,
semantic reinterpretation, or hidden plan/execution collapse.

## Why This Hardening Exists

Milestone 2 proved the bridge can route fine-grained truth correctly.

The QA pass showed the next real structural weakness:

- the envelope boundary is still more "raw and repairable" than
  "authority-exported and self-describing"
- planning and lowering still carry less explicit provenance than comparable
  WORTH lowered-plan subsystems
- replay currently certifies final artifact identity more strongly than it
  certifies the lowering contract that produced that identity

That is acceptable for an early foundation. It is not acceptable for the final
production-grade shape this crate is aiming for.

This hardening belongs here in sequence because:

- it preserves the semantic boundary Milestone 2 already established
- it strengthens the proof chain before later milestones depend on it
- it prevents lineage, historical evaluation, and scale-path work from
  inheriting a softer envelope/planning substrate

## Adversarial Constraint

This hardening must survive the following hostile condition:

> A long-lived system receives committed patch exports from multiple producer
> versions, restarts and replays from canonical bridge artifacts, and routes
> semantically identical truth deltas expressed through different raw envelope
> spellings. The bridge must either reject incompatible exports explicitly at
> ingress or produce the exact same normalized truth, planned route, lowered
> slice artifact, read packet, and lowering provenance every time.

Concretely, the design must remain correct when all of the following are true:

- a producer changes bridge-export spelling without changing truth meaning
- equivalent raw surfaces such as `name` and `field:name` arrive across runs
- diagnostics richness changes between environments
- replay occurs from canonical bridge artifacts after restart
- host registration order differs across runs
- future milestones add branch-aware and lineage-aware bridge behavior without
  changing the meaning of already supported envelope/proof artifacts

If any supported path:

- accepts a producer export whose semantics version it does not understand
- lets raw envelope spelling change canonical route truth
- recomputes planning meaning from weaker upstream state during lowering or delivery
- cannot explain what lowering contract produced the final artifact

then this hardening has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this hardening:

- the bridge will continue to own its envelope normalization and planning proof
  chain rather than leaking those responsibilities into the sink or host
- producer compatibility is an ingress concern and must be typed explicitly
- normalized-but-unvalidated bridge envelope state must not remain
  synthesizable by convention
- route planning and lowering must expose provenance and summary artifacts as
  first-class bridge-owned truth
- execution must consume a validated lowered plan, not merely a plausible one
- this hardening strengthens Milestone 2 semantics; it does not change the
  truth/signal authority split

Normative consequence:

- public bag-style construction of semantically authoritative envelope states is
  out of spec
- route identity derived from raw patch spellings after normalized truth exists
  is out of spec
- lowering that lacks explicit provenance or summary truth is out of spec
- delivery that consumes a weaker artifact than the lowered contract is out of
  spec

## Scope

### In Scope

- bridge input schema/version and producer-authority compatibility metadata
- sealed normalized-envelope and validated-envelope proof types
- explicit planning provenance and planning summary artifacts
- explicit lowering provenance and lowering summary artifacts
- validated lowered-plan phase between lowering and delivery
- replay checks over lowering contract truth, not only final output identity
- harness scenarios covering producer compatibility and semantic-equivalent raw
  envelope spellings

### Explicitly Out Of Scope

- changing the admitted fine-grained slice taxonomy
- lineage-aware subscription continuity
- branch-aware or historical semantics beyond compatibility metadata needed at
  ingress
- speculative bridge preview flows
- replacing bridge-owned route/slice/invalidation identities with a different
  public artifact model

This hardening must strengthen the current bridge substrate, not re-open the
Milestone 2 product boundary.

## Governing Design Rules

### 1. Producer Compatibility Must Be Explicit At Ingress

Bridge ingress must know:

- what producer/export authority created the envelope
- what bridge export schema/version it claims
- whether the current bridge runtime supports that export

Bridge ingress must not:

- infer compatibility from field presence alone
- silently accept new export semantics because the shape looks similar
- defer producer compatibility checking until after normalization or planning

### 2. Envelope Phases Must Be Proof-Carrying

Representative progression:

```rust
pub struct ProducerBridgePatchEnvelope { ... }
pub struct NormalizedBridgePatchEnvelope { ... }
pub struct ValidatedBridgePatchEnvelope { ... }
```

Rules:

- producer envelope construction stays at the boundary
- normalized envelope construction is sealed to the normalization phase
- validated envelope construction is sealed to the validation phase
- later phases consume the immediately prior proof type, never a weaker bag of
  parts

### 3. Canonical Truth Must Dominate Raw Spelling

Once normalized truth delta surfaces exist, canonical planning identity must be
derived only from normalized truth and resolved mapping proof.

Planning may preserve raw spellings for explanation, but it must not use them
as identity-bearing truth after normalization.

### 4. Lowering Must Emit Provenance, Not Only Output

Bridge lowering must answer:

- what planned route did this lowering consume?
- what canonical lowering summary did it derive?
- what lowering digest/provenance basis explains replay parity?

It must not be merely:

- a packaging helper for already computed values
- the only phase that knows how the final invalidation artifact was assembled

### 5. Execution Must Consume A Validated Lowered Plan

Representative progression:

```rust
pub struct PlannedBridgeRoute { ... }
pub struct BridgeLoweringPlan { ... }
pub struct ValidatedBridgeLoweringPlan { ... }
pub struct DeliveredBridgeResult { ... }
```

Rules:

- lowering resolves structure
- lowered-plan validation certifies delivery-readiness
- delivery consumes only validated lowered truth
- delivery must not rediscover or reinterpret plan semantics

### 6. Canonicality Must Include Planning And Lowering Contracts

For every new hardening artifact, the design must define:

- ordered input set
- ordering key
- deduplication basis
- digest basis
- identity-bearing fields
- explanatory-only fields

Canonicality must cover at least:

- normalized bridge envelope identity
- planned route provenance basis
- planning summary digest basis
- lowering provenance basis
- lowering summary digest basis

## Phases

### Phase 1: Seal Envelope Authority

Build the hardened ingress proof chain.

Must ship:

- producer bridge patch envelope metadata:
  - export schema version
  - producer authority kind
  - optional producer semantics version where required
- sealed `NormalizedBridgePatchEnvelope`
- sealed `ValidatedBridgePatchEnvelope`
- typed compatibility rejection before planning
- envelope compatibility errors with structured context

System state after Phase 1:

- the bridge accepts only explicitly understood producer exports
- normalized-but-unvalidated envelope state is no longer a bag of public fields
- ingress emits proof-bearing envelope truth that later phases can trust

### Phase 2: Make Planning Canonical And Provenance-Bearing

Refactor planning so canonical route truth is fully normalized and explicitly
certified.

Must ship:

- `BridgePlanningProvenance`
- `BridgePlanningSummary`
- route identity derived only from normalized truth + resolved mapping proof
- read-packet derivation folded into planning proof rather than helper-only work
- route records and replay artifacts that reference planning provenance

System state after Phase 2:

- route-planning truth no longer depends on raw envelope spelling
- planning produces explicit identity and summary artifacts
- replay can reason about planning contract parity directly

### Phase 3: Validate Lowered Bridge Truth Before Delivery

Strengthen lowering and execution admission to match WORTH lowered-plan
standards.

Must ship:

- `BridgeLoweringProvenance`
- `BridgeLoweringSummary`
- `ValidatedBridgeLoweringPlan`
- replay-visible digest of lowering summary/provenance
- delivery consuming only validated lowered truth

System state after Phase 3:

- lowering is a named, certifiable bridge phase rather than a packaging step
- execution no longer consumes merely plausible lowered artifacts
- replay certifies not only final identities but also the lowering contract

## Must Ship

- bridge input schema/version metadata
- producer authority metadata at ingress
- sealed normalized and validated envelope proof types
- planning provenance and planning summary artifacts
- lowering provenance and lowering summary artifacts
- validated lowered-plan phase
- replay checks over planning/lowering contract truth
- certification tests for semantic-equivalent raw envelope spellings
- compatibility tests for supported and unsupported producer versions

## Must Preserve

- current Milestone 2 truth/signal ownership boundaries
- slice-native signal delivery contract
- closed truth-surface and subscription-slice taxonomy
- deterministic route, invalidation, and slice identity
- snapshot-pinned evaluation semantics
- explicit typed fallback and suppression behavior

## Acceptance Evidence

This hardening is complete only when the bridge harness can prove:

- supported producer/export versions are accepted and unsupported ones are
  rejected before planning
- semantically equivalent raw envelopes produce identical normalized envelope,
  route identity, planning provenance, lowering provenance, invalidation
  identity, and slice identity
- route planning consumes validated envelope truth only
- delivery consumes validated lowered truth only
- replay detects lowering provenance or lowering-summary drift explicitly
- diagnostics-tier richness changes do not change normalized/planned/lowered
  identities

## Architectural Notes

The strongest pattern to borrow from `worth-relational` is not field count or
artifact richness. It is authority posture:

- export canonical authority early
- lower into explicit provenance-bearing plans
- validate lowered plans before execution

The bridge should remain smaller than relational, but it must not remain softer
at these boundaries.

## Sequencing Notes

This hardening belongs inside Milestone 2 follow-through, before Milestone 3.

Reason:

- Milestone 3 continuity work will depend on stable envelope and route identity
- Milestone 4 historical/branch-aware work will depend on explicit producer and
  lowering compatibility
- Milestone 5 scale-path work will depend on planning/lowering summaries and
  proof-bearing validated lowered plans

If this hardening is deferred until after those milestones, later work will
inherit a weaker substrate and force retrofit changes across replay,
diagnostics, and certification.
