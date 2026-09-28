# Milestone 4 Engineering Spec: Historical And Branch-Aware Evaluation

> **Status:** Completed. Closeout: [milestone-4-closeout.md](./milestone-4-closeout.md).
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-3.md](./milestone-3.md)
>
> **Prior closeout:** [milestone-3-closeout.md](./milestone-3-closeout.md)
>
> **Primary architectural driver:** make intentional evaluation against retained historical truth and branch-local truth a first-class bridge protocol without letting latest-state convenience, ambient branch context, or signal-owned execution identity become accidental truth authority
>
> **Companion docs:**
> - [worth_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
> - [worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
> - [worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [architectural_guidelines.md](../../docs/coding-guidelines/arch_laws.md)
> - [domain_standards.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [performance_guidelines.md](../../docs/coding-guidelines/perf_laws.md)

## Summary

Milestones 1 through 3 established that:

- committed truth enters the bridge through one canonical committed envelope
- fine-grained bridge routing lowers precise truth surfaces into canonical subscription slices
- bridge evaluation stays pinned to stable snapshots instead of live mutable truth
- continuity across identity evolution is branch-aware, lineage-backed, and replay-safe

That is enough to make present-time invalidation and continuity trustworthy.

It is not enough for the class of workloads the bridge explicitly claims to
support:

- explain why a derived answer changed between two retained historical states
- evaluate the same derived contract against a non-head branch without copying
  host-local branch glue into signal code
- replay a historical or branch-local bridge run after newer truth publication
  exists
- compare two branch-local evaluations without ambiguity about which truth
  authority each one consumed
- keep historical and branch-aware reads precise without silently widening into
  "whatever latest snapshot is nearby"

Milestone 4 exists because stable snapshots alone are weaker than intentional
truth-view selection.

The bridge must be able to say:

`this derived run consumed this exact truth view because the bridge planned and opened this canonical historical-or-branch authority basis under this route, snapshot, and branch contract`

not:

`the bridge happened to read some state that looked equivalent at execution time`

The bridge still does not own truth history, truth retention, branch semantics,
or signal scheduling. It owns:

- one declarative historical-evaluation definition surface
- truth-view request shaping
- resolved truth-view policy and admission
- canonical historical/branch authority basis
- deterministic planning and lowering of historical/branch read packets
- phase-typed observation over materialized truth views
- replay-safe records and diagnostics for which truth view a derived run used

## Goal

Make historical and branch-aware bridge evaluation deterministic, explicit, and
replay-safe so derived execution can intentionally consume retained historical
truth and branch-local truth rather than only the latest committed state.

## Why This Milestone Exists

Milestone 4 belongs immediately after Milestone 3 because Milestone 3 supplied
the missing identity and authority substrate that historical evaluation needs:

- canonical route identity
- snapshot identity
- explicit branch identity in bridge authority
- lineage-backed continuity over identity evolution
- replay-safe continuity and route records that survive newer publication

Without Milestone 3, historical or branch-aware evaluation could open an older
snapshot but still fail the deeper question of whether the subscribed or routed
truth surface is the intended one under branch-local identity evolution.

Without Milestone 4, later roadmap work inherits a weaker read contract than
the product requires:

- bulk routing and scale-path planning would optimize current-state routing
  while leaving historical evaluation as a side path
- reactive source contracts would productize read surfaces before the bridge
  proved its strongest retained-history read mode
- structural-identity-assisted remapping would have no canonical historical
  evaluation substrate to compare against
- speculative truth-branch to signal-branch coordination would be forced to
  build on ambiguous branch read semantics
- end-to-end certification would remain incomplete because "what truth view was
  evaluated?" would still be partly narrative

Milestone 4 therefore earns its place in the roadmap by solving the next real
structural problem after continuity: explicit truth-view authority for retained
history and non-head branches.

## Adversarial Constraint

Milestone 4 must survive the following hostile condition:

> A long-lived system with retained historical truth, multiple truth branches,
> branch-local identity evolution, newer publications arriving after an older
> derived run completed, diagnostics tiers that vary by environment, and replay
> after restart must evaluate the same canonical historical-or-branch bridge
> request against the same exact truth view every time, while consulting only
> truth-owned historical and branch authority plus bridge-planned view
> selection, and while rejecting ambiguous, unavailable, or unsupported truth
> views rather than silently falling back to latest reachable state.

Concretely, the design must remain correct when all of the following are true:

- the requested view is historical rather than head-of-branch
- the requested view is branch-local rather than canonical-main
- the same logical truth surface has different branch-local continuity outcomes
- a newer publication exists by the time replay occurs
- a requested historical commit remains retained while surrounding history has
  advanced
- diagnostics richness changes between environments
- some host integrations can provide retained snapshots directly while others
  derive them through historical lookup packets
- some requested historical or branch view classes remain unsupported and must
  fail explicitly

If any supported path:

- resolves historical evaluation by asking for latest visible truth
- chooses a branch implicitly from host ambient context
- reopens truth-view selection during delivery because planning was underspecified
- lets diagnostics policy alter the chosen truth view
- cannot replay or explain which branch, version, or snapshot was consumed

then Milestone 4 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- historical and branch-aware evaluation is a first-class bridge subsystem, not
  a convenience option hidden inside snapshot reads
- historical and branch-aware evaluation must enter the bridge through one
  declarative definition surface rather than scattered selector, replay,
  diagnostics, and delivery wiring calls
- truth-view authority is bridge-owned as a request and artifact vocabulary,
  but truth runtime remains the authority for branch semantics, retention, and
  historical materialization
- branch identity is always explicit in bridge truth-view authority, even when
  the selected branch is the canonical default branch
- historical selection must be expressed as canonical truth-view selectors, not
  ad hoc timestamp or "nearest available" convenience flags
- selector support, retention admission, source capability admission,
  diagnostics policy, and replay compatibility must be resolved before
  materialization begins
- the bridge must plan historical/branch read packets before delivery
- replay authority for historical and branch-aware evaluation must come from
  canonical bridge records rather than live host state
- Milestone 4 extends Milestones 1 through 3 proof chains; it does not create a
  second evaluation path that bypasses canonical routing, continuity, or replay

Normative consequence:

- caller code that registers selector, replay mode, and diagnostics policy
  through separate coordination calls is out of spec
- host-local "if branch is omitted, just use whatever is current" logic is out
  of spec
- latest-snapshot fallback for missing historical authority is out of spec
- historical comparison by raw host timestamps rather than canonical truth-view
  selectors is out of spec
- signal code inferring truth branch semantics from its own branch model is out
  of spec

## Scope

### In Scope

- one bridge-owned declaration surface for historical/branch evaluation
- bridge-owned truth-view selector, authority basis, and evaluation request
  vocabulary
- bridge-owned policy/admission artifacts for selector support, retention
  availability, source capability, replay mode, and diagnostics mode
- historical snapshot evaluation contracts for retained truth views
- branch-aware bridge semantics for branch-local truth evaluation
- deterministic planning of historical/branch read packets and continuity-aware
  routing context
- canonical lowering of truth-view authority into executable evaluation
  artifacts
- phase-typed observation handles over materialized truth views
- canonical route/evaluation records, replay records, counters, and derived
  explanations for historical and branch-local evaluation
- harness certification for retained-history parity, branch divergence,
  diagnostics-tier invariance, and replay after newer publication

### Explicitly Out Of Scope

- speculative truth-branch to signal-branch coordination beyond the explicit
  truth-branch read contract needed here
- generalized reactive source protocol productization across all host shapes
- full merge-aware bridge semantics across multi-parent histories
- structural-identity-assisted remapping or branch comparison heuristics
- bridge-mediated writeback
- preview or non-authoritative branch flows that create temporary bridge-owned
  lifecycle surfaces

Milestone 4 must remain focused on intentional historical and branch-local
reads, not absorb later speculative or generalized source milestones.

## Governing Design Rules

### 1. Truth Owns View Materialization, Bridge Owns View Authority

The truth runtime defines:

- branch existence and branch identity semantics
- retained historical truth and retention policy
- how a truth-view selector resolves to a materializable historical view
- what continuity and lineage evidence remain valid for that view

The signal runtime defines:

- derived node identity
- derived scheduling and execution semantics
- downstream explanation over the delivered bridge result

The bridge defines:

- declarative historical-evaluation definition surfaces
- truth-view selector vocabulary
- truth-view policy/admission vocabulary
- truth-view authority basis and digest basis
- evaluation request shaping
- planned historical/branch read packets
- phase-typed observation handles over materialized truth views
- replay-safe evaluation and diagnostics records

The bridge must not define its own history or branch ontology.

The bridge facade is the only public historical/branch-aware evaluation
surface.

External consumers may depend on:

- bridge historical-evaluation declarations
- bridge truth-view selectors
- bridge resolved truth-view policy and admitted plan types
- bridge historical/branch evaluation request types
- bridge canonical evaluation records and explanations

External consumers may not depend on:

- relational branch internals
- relational retention traversal helpers
- signal scheduler branch internals
- internal bridge planning or diagnostics assembly modules
- internal source-capability resolution modules

### 2. Truth Views Must Be Selected, Not Discovered

Each historical or branch-aware evaluation must start from one explicit
bridge-owned historical-evaluation declaration containing one explicit
bridge-owned truth-view selector.

Representative admitted selector classes for Milestone 4:

- `CommittedSnapshot`
- `CommitBoundHistoricalView`
- `BranchHeadView`
- `BranchSnapshotView`
- `BranchCommitView`

Rules:

- the declaration surface must bundle selector, replay mode, diagnostics mode,
  and delivery intent into one source of truth
- a selector must identify whether it is historical, branch-head, or
  branch-bound historical
- a selector must carry explicit branch identity even when selecting the
  default branch
- a selector must have one canonical digest basis
- unsupported selector classes must fail explicitly rather than degrade into
  nearest available history

Milestone 4 does not admit:

- wall-clock timestamp selectors as public authority
- host-defined free-form "view mode" strings
- implicit latest fallback when the selector cannot be materialized
- scattered declaration across multiple registries or builder side channels

### 3. Policy Resolution Must Be Complete Before Materialization

Milestone 4 must satisfy Law 17 structurally, not rhetorically.

Before truth-view materialization starts, the bridge must resolve:

- selector support
- retention admission
- source capability admission
- diagnostics policy
- replay compatibility
- branch compatibility

The result must be one bridge-owned lowered policy artifact.

Rules:

- materialization may consume only a resolved admitted plan, never a raw
  selector plus ambient policy context
- delivery may consume only lowered execution truth, never re-decide selector
  applicability or source capability
- unsupported, unavailable, or incompatible policy outcomes must be represented
  in the resolved plan artifact, not inferred later from adapter failure shape
- later milestones may add policy axes, but they must extend the resolved plan
  surface rather than introducing execution-time rediscovery

### 4. Historical Evaluation Authority Must Remain Continuity-Aware

Milestone 4 must not treat historical evaluation as "open old snapshot and
ignore identity evolution."

If the requested truth surface depends on identity continuity across history or
branch-local divergence, evaluation planning must consume the continuity
substrate established by Milestone 3.

Rules:

- truth-view planning may depend on canonical route identity, branch identity,
  snapshot identity, and continuity authority basis
- historical evaluation must preserve the exact truth-view basis that continuity
  planning resolved against
- branch-local evaluation must not collapse divergent continuity outcomes into
  one apparent surface
- historical evaluation records must remain strong enough that later replay and
  diagnostics can explain both the truth view and the identity basis used

### 5. Plan / Materialize / Lower / Deliver Separation Is Mandatory

Milestone 4 extends the bridge proof chain:

- declarative historical-evaluation definition
- validated truth-view selector
- resolved truth-view policy
- planned historical/branch truth-view packet
- materialized historical/branch observation authority
- lowered branch-aware evaluation artifact
- delivered derived evaluation result

Truth-view materialization must not be rediscovered during delivery.
Delivery consumes lowered evaluation truth only.

### 6. View Materialization Must Be Proof-Carrying

Milestone 4 must continue satisfying Architectural Laws 30 and 41.

Representative progression:

```rust
pub struct HistoricalEvaluationDeclaration { ... }
pub struct ValidatedTruthViewSelector { ... }
pub struct ResolvedTruthViewPolicy { ... }
pub struct PlannedTruthViewPacket { ... }
pub struct MaterializedTruthViewObservation { ... }
pub struct LoweredHistoricalEvaluationArtifact { ... }
pub struct DeliveredHistoricalEvaluationResult { ... }
```

Rules:

- every proof-bearing type named in this pipeline must have sealed
  constructors and private fields
- constructors for proof-bearing truth-view packets must be sealed to proving
  modules
- later phases must consume the exact proof-bearing type produced upstream
- unsupported, unavailable, or mismatched truth-view outcomes become part of
  the proof chain rather than explanation-only side channels
- delivery cannot accept a weaker packet than lowering produced
- any runtime check for branch/snapshot/view properties already proved upstream
  is a design failure
- the type signatures of the transitions themselves must enforce the legal
  ordering of the pipeline so skipped phases are uncompilable

### 7. Canonicality Must Be Mechanically Declared

For every Milestone 4 canonical artifact, the spec must define:

- ordered input set
- ordering key
- deduplication rule
- digest basis
- identity-bearing versus explanatory-only fields

Canonicality must cover at least:

- declaration ordering
- truth-view selector ordering
- resolved policy ordering
- truth-view packet ordering
- historical lookup request ordering
- branch identity basis
- materialized truth-view authority ordering
- evaluation artifact ordering
- route/evaluation record ordering

If any historical or branch-aware evaluation artifact can vary because host
branch registration order, retention storage layout, or diagnostics policy
changes, the design is out of spec.

### 8. Observation Must Be Phase-Typed And Scoped

Milestone 4 must satisfy Law 18 structurally.

Historical or branch-aware evaluation may only observe truth through
phase-appropriate bridge-owned observation handles.

Rules:

- materialization must yield a `MaterializedTruthViewObservation`, not a raw
  truth-runtime snapshot handle
- the observation type exposed to delivery must not provide mutation authority,
  retention-widening authority, or branch retargeting authority
- replay and diagnostics reconstruction may consume canonical observation truth,
  not re-open mutable truth access
- if a caller can use a historical observation handle to perform mutation or
  ask for a broader truth view, the design is out of spec

### 9. Branch Identity Must Remain Explicit End-To-End

Milestone 4 must not let branch-local truth appear as anonymous historical
state.

Rules:

- every selector carries branch identity
- every planned and materialized truth-view artifact carries branch identity
- every route/evaluation record carries branch identity
- replay mismatch detection must distinguish branch mismatch from version or
  snapshot mismatch
- branch-local truth must not leak into unrelated evaluations through shared
  default branch handles or mutable ambient state

### 10. Historical Breadth Must Be Planned And Bounded

Milestone 4 must make its normal-path breadth claims explicit.

At minimum, the spec must preserve these structural expectations:

- selector validation scales with requested selector count, not whole retained
  history size
- truth-view planning scales with requested branch/view packet width, not full
  branch-history scans on the normal path
- materialization scales with the exact retained view requested, not nearest
  visible state plus ad hoc correction logic
- replay reuses canonical truth-view authority rather than rediscovering branch
  history breadth

Forbidden normal-path costs:

- scanning full retained branch history for every evaluation request
- reopening whole-branch continuity lookup during delivery
- broad "find some compatible snapshot" searches after selector validation
- replay that depends on latest-publication lookup to recover historical truth

### 11. Decision Logs Are First-Class Historical Evaluation Artifacts

Milestone 4 must produce a canonical decision log for truth-view selection.

At minimum, the decision log must record:

- the declaration identity
- the selector that was requested
- the resolved policy outcome
- the admitted materialization path
- any typed rejection or incompatibility outcome
- the final source branch / commit / snapshot authority basis

Rules:

- the decision log is part of canonical evaluation truth, not debug garnish
- replay and explanation must be reconstructable from canonical records plus
  the decision log without querying the producer for hidden context
- decision lookup for a named historical evaluation record must be O(1) by
  evaluation identity

### 12. Diagnostics Are Derived From Canonical Truth-View Authority

Operational truth for Milestone 4 is:

- canonical truth-view selector identity
- canonical planned truth-view packet identity
- canonical materialized truth-view authority identity
- canonical lowered evaluation artifact identity
- typed unavailability, mismatch, and replay failures
- counters

Rich explanation remains derived and policy-shaped.
Changing diagnostics richness must not change truth-view authority.

### 13. Historical Evaluation Must Stay Read-Only

Milestone 4 is an evaluation milestone, not a writeback milestone.

Rules:

- historical or branch-aware evaluation may materialize retained truth views but
  may not mutate truth
- branch-local truth consumed by evaluation remains truth authority, not bridge
  cache authority
- bridge-created artifacts for evaluation are derived and discardable
- any mutation side effect discovered in historical evaluation code is out of
  spec and must be rejected structurally

## Target Runtime Model

### 1. Public Surface Growth

Milestone 4 should extend `worth-runtime-bridge` with truth-view concepts such
as:

```rust
pub struct HistoricalEvaluationDeclaration { ... }
pub struct BridgeTruthViewSelector { ... }
pub struct ResolvedTruthViewPolicy { ... }
pub struct BridgeHistoricalEvaluationRequest { ... }
pub struct BridgePlannedHistoricalEvaluation { ... }
pub struct BridgeHistoricalEvaluationResult { ... }
pub struct BridgeTruthViewAuthority { ... }

pub enum BridgeTruthViewKind {
    CommittedSnapshot,
    HistoricalCommit,
    BranchHead,
    BranchSnapshot,
    BranchCommit,
}

pub enum BridgeTruthViewFailure {
    UnsupportedTruthViewSelector { ... },
    TruthViewUnavailable { ... },
    BranchMismatch { ... },
    SnapshotMismatch { ... },
    HistoricalResolutionRejected { ... },
    HistoricalReplayMismatch { ... },
}
```

Design rules:

- callers declare one historical evaluation, not multiple disconnected knobs
- planning and delivery remain separate public boundary crossings
- evaluation requests consume bridge-owned truth-view selectors, not host-local
  branch flags
- the bridge facade exposes bridge nouns only
- historical evaluation remains an extension of canonical bridge routing and
  continuity, not an alternate hidden runtime tunnel

### 2. Truth-View Selector Contract

Milestone 4 needs an explicit selector contract for truth-view authority.

Representative shape:

```rust
pub struct HistoricalEvaluationDeclaration {
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    selector: BridgeTruthViewSelector,
    replay_mode: BridgeReplayMode,
    diagnostics_mode: BridgeDiagnosticsMode,
    delivery_intent: BridgeDeliveryIntent,
}

pub struct BridgeTruthViewSelector {
    selector_identity: BridgeTruthViewSelectorIdentity,
    view_kind: BridgeTruthViewKind,
    branch_identity: TruthBranchIdentity,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: Option<TruthSnapshotIdentity>,
}

pub struct ValidatedTruthViewSelectorSet {
    selectors: Vec<BridgeTruthViewSelector>,
    digest: BridgeTruthViewSelectorSetDigest,
}
```

Rules:

- declaration identity must be canonical and independent of caller assembly
- the declaration must be the only public source of truth for selector,
  replay, diagnostics, and delivery intent
- selector identity must be canonical and independent of host argument order
- branch identity is mandatory
- optional commit/snapshot fields must be admitted only for selector kinds that
  prove they are meaningful
- explanatory labels must not participate in selector identity

### 3. Resolved Truth-View Policy

Milestone 4 needs an explicit lowered policy artifact before truth-view
materialization begins.

Representative shape:

```rust
pub struct ResolvedTruthViewPolicy {
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    selector_identity: BridgeTruthViewSelectorIdentity,
    retention_admission: RetentionAdmission,
    source_capability: TruthViewSourceCapability,
    replay_mode: BridgeReplayMode,
    diagnostics_mode: BridgeDiagnosticsMode,
    delivery_intent: BridgeDeliveryIntent,
}
```

Rules:

- this policy is resolved once at the entry boundary
- materialization may not widen or reinterpret this policy
- unsupported or unavailable selector classes must appear here as typed
  resolution outcomes before adapter execution begins
- later execution phases consume the resolved policy as proof, not as advice

### 4. Planned Truth-View Packet

Milestone 4 needs a planned packet that binds route, continuity, branch, and
retained-history authority before delivery.

Representative shape:

```rust
pub struct PlannedTruthViewPacket {
    route_identity: BridgeRouteIdentity,
    continuity_identity: Option<BridgeContinuityIdentity>,
    resolved_policy: ResolvedTruthViewPolicy,
    truth_view_selector: BridgeTruthViewSelector,
    authority_basis: BridgeTruthViewAuthorityBasis,
    read_packet: SnapshotReadPacket,
    counters: BridgeHistoricalEvaluationCounters,
}
```

Rules:

- the packet must be derived during planning, not materialized ad hoc during
  execution
- route and continuity authority remain explicit inputs where relevant
- resolved policy must be embedded directly in the packet so execution does not
  re-decide admission
- the packet must already declare the exact retained branch/view materialization
  the truth runtime is allowed to perform
- packet identity must be replay-safe

### 5. Materialized Truth-View Observation Authority

The truth runtime must return one canonical materialized truth-view authority
surface to the bridge.

Representative shape:

```rust
pub struct MaterializedTruthViewObservation {
    authority_basis: BridgeTruthViewAuthorityBasis,
    branch_identity: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    snapshot_token: BridgeSnapshotToken,
    counters: BridgeHistoricalEvaluationCounters,
}
```

Rules:

- the returned authority must match the planned authority basis exactly
- the returned observation type must expose only phase-appropriate read access
- the bridge must reject returned branch or snapshot drift explicitly
- historical materialization may come from retained snapshots or canonical
  historical lookup, but the returned authority basis must be the same either
  way
- delivery consumes this authority, not the raw truth runtime lookup process

### 5. Lowered Historical Evaluation Artifact

Representative shape:

```rust
pub struct LoweredHistoricalEvaluationArtifact {
    route_identity: BridgeRouteIdentity,
    truth_view_identity: BridgeTruthViewIdentity,
    source_branch: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    snapshot_token: BridgeSnapshotToken,
    subscription_slices: CanonicalSubscriptionSlices,
    counters: BridgeHistoricalEvaluationCounters,
}
```

Rules:

- the artifact must combine canonical truth-view authority with canonical route
  truth
- the artifact must remain durable enough for replay and diagnostics
- signal consumes canonical branch-aware evaluation artifacts, not relational
  history internals
- later milestones may extend this artifact, but they may not weaken its truth
  view identity

### 6. Canonical Historical Evaluation Record

Milestone 4 should make historical replay authority as concrete as route and
continuity replay authority already are.

Representative shape:

```rust
pub struct BridgeCanonicalHistoricalEvaluationRecord {
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    route_identity: BridgeRouteIdentity,
    truth_view_selector_digest: BridgeTruthViewSelectorDigest,
    resolved_policy_digest: ResolvedTruthViewPolicyDigest,
    truth_view_authority_digest: BridgeTruthViewAuthorityDigest,
    historical_evaluation_identity: BridgeTruthViewIdentity,
    source_branch: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    counters: BridgeHistoricalEvaluationCounters,
    schema_version: BridgeCanonicalHistoricalEvaluationSchemaVersion,
}
```

Rules:

- replay must proceed from canonical historical evaluation records, not ambient
  branch or latest-state lookups
- replay identity must derive from canonical selector and authority truth
- replay identity must also preserve declaration identity and resolved policy
  identity
- compatibility failures and replay mismatches must remain typed and explicit

### 7. Bridge-Owned Adapter Traits

Milestone 4 should continue depending on narrow bridge-owned contracts rather
than wide parent-runtime facades.

Representative shape:

```rust
pub trait HistoricalTruthViewBridgeAdapter {
    type Error;

    fn materialize_truth_view(
        &self,
        packet: &PlannedTruthViewPacket,
    ) -> Result<MaterializedTruthViewObservation, Self::Error>;
}

pub trait BranchAwareSignalBridgeAdapter {
    type Error;

    fn deliver_historical_evaluation(
        &self,
        artifact: LoweredHistoricalEvaluationArtifact,
    ) -> Result<DeliveredHistoricalEvaluationResult, Self::Error>;
}
```

Rules:

- the bridge owns the historical/branch evaluation adapter contracts
- parent runtimes implement only the narrow contracts needed to satisfy the
  proof chain
- no broad facade reach-through is allowed because historical evaluation is
  stronger than current-state evaluation

## Phases

### Phase 1: Truth-View Vocabulary And Authority Boundary

Phase 1 exists to make historical and branch-aware evaluation structurally
representable without letting hosts smuggle latest-state shortcuts into the
bridge.

Milestone 4 must first define:

- bridge-owned historical-evaluation declaration vocabulary
- bridge-owned truth-view selector taxonomy
- bridge-owned resolved truth-view policy vocabulary
- bridge-owned truth-view authority basis and digest basis
- explicit admitted versus unsupported historical/branch selector classes
- narrow truth-owned historical materialization adapter surfaces
- exact relationship between route authority, continuity authority, and
  truth-view authority

This phase leaves the system in a coherent state where:

- a historical or branch-aware evaluation can be requested explicitly
- one declaration struct is the only public source of truth for the request
- unsupported truth-view classes fail by design instead of drifting later
- there is no ambiguity about what truth owns versus what the bridge owns

### Phase 2: Deterministic Truth-View Planning And Lowered Evaluation Artifacts

Phase 2 exists to turn the vocabulary into canonical bridge truth.

Milestone 4 must then implement:

- canonical historical-evaluation declaration validation
- resolved truth-view policy and admission before materialization
- canonical truth-view selector validation and freezing
- deterministic planned truth-view packet derivation from route and continuity
  authority
- canonical truth-view materialization and authority-basis verification
- lowered branch-aware historical evaluation artifacts
- exact counters and typed mismatch/unavailability failures

This phase leaves the system in a coherent state where:

- identical historical or branch-aware inputs lower to identical evaluation
  artifacts
- branch identity remains explicit through planning and delivery
- execution no longer re-decides selector support or source capability
- retained-history unavailability or mismatch is typed and canonical rather
  than buried in logs

### Phase 3: Replay, Diagnostics, And Historical/Branch Certification

Phase 3 exists to prove the truth-view model is trustworthy.

Milestone 4 must finally ship:

- canonical historical/branch evaluation records and replay records
- canonical decision-log records for truth-view selection and admission
- explanation reconstruction for selected historical and branch-local truth views
- hostile harness suites covering retained-history replay, branch divergence,
  diagnostics-tier invariance, unavailable truth views, and replay after newer
  publication
- exact counter assertions for representative selector-width and view-width scenarios

This phase leaves the system in a coherent state where:

- historical and branch-local evaluation survives replay after restart
- truth-view choice remains diagnosable even after newer truth exists
- later source-protocol, scale-path, and speculative-branch milestones can
  build on stable truth-view artifacts

## Must Ship

- bridge-owned truth-view selector, authority-basis, and evaluation request
  artifacts
- one bridge-owned declaration surface for historical evaluation
- resolved truth-view policy/admission artifacts
- explicit historical snapshot evaluation contracts
- explicit branch-aware evaluation contracts for branch-local truth
- planned truth-view packet derivation from canonical route and continuity
  authority where relevant
- canonical materialized truth-view authority and lowered evaluation artifacts
- phase-typed observation handles over materialized truth views
- typed failures for unsupported truth-view selector, unavailable truth view,
  branch mismatch, snapshot mismatch, historical resolution rejection, and
  replay mismatch
- canonical route/evaluation records, replay artifacts, and derived explanations
- harness certification lanes for retained-history parity, branch-local
  isolation, diagnostics-tier invariance, and replay after newer publication

## Must Preserve

- truth runtime remains the authority for history retention, branch semantics,
  and historical materialization
- signal runtime remains the authority for node identity and execution
- no live mutable truth reads during historical or branch-aware evaluation
- no implicit latest-state fallback when historical/branch authority is missing
- no scattered declaration or execution-time policy rediscovery
- branch identity remains explicit end-to-end
- canonical ordering and replay-safe truth-view identity
- clean facade boundaries rather than wide parent-runtime reach-through

## Acceptance Evidence

Milestone 4 is complete only when the bridge harness can prove:

- identical canonical historical or branch-aware requests lower to identical
  truth-view evaluation artifacts
- historical evaluation uses the exact retained truth view requested rather than
  latest reachable state
- branch-local truth does not leak into unrelated derived runs
- branch-local continuity differences remain visible in the selected truth view
  rather than being flattened away
- diagnostics richness changes explanation only, not truth-view authority
- declaration identity, resolved policy, and decision-log truth remain replay-safe
- replay from canonical historical evaluation artifacts matches original
  historical or branch-aware behavior even after newer publication arrives
- unavailable or unsupported truth views fail explicitly and typed

## Architectural Notes

### Expected Internal Subdomains

Milestone 4 should extend the bridge crate with subdomains such as:

- `snapshot/context/`
- `snapshot/declaration/`
- `snapshot/selection/`
- `snapshot/policy/`
- `snapshot/materialization/`
- `snapshot/authority/`
- `snapshot/decision_log/`
- `delivery/historical/`
- `diagnostics/history/`
- `harness/fixtures/historical_evaluation.rs`
- `harness/fixtures/branch_local_evaluation.rs`
- `harness/fixtures/historical_replay.rs`

This follows workspace domain standards:

- declaration validation is not the same responsibility as policy resolution
- selector validation is not the same responsibility as truth-view
  materialization
- branch authority is not the same responsibility as replay record construction
- historical explanation reconstruction is not the same responsibility as
  canonical truth-view identity

### Minimum Counter Floor

Milestone 4 must add counters such as:

- `truth_view_selector_count`
- `historical_truth_view_count`
- `branch_truth_view_count`
- `planned_truth_view_packet_count`
- `resolved_truth_view_policy_count`
- `materialized_truth_view_count`
- `truth_view_unavailable_count`
- `truth_view_branch_mismatch_count`
- `truth_view_snapshot_mismatch_count`
- `historical_replay_mismatch_count`
- `branch_local_evaluation_count`
- `truth_view_decision_log_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Truth-View Failure Policy

Milestone 4 must carry unavailable and unsupported truth-view outcomes
structurally rather than narratively.

Required failure classes:

- `UnsupportedTruthViewSelector`
- `TruthViewUnavailable`
- `RejectedBranchMismatch`
- `RejectedSnapshotMismatch`
- `RejectedHistoricalResolutionFailure`
- `HistoricalReplayMismatch`

Rules:

- every requested truth view gets exactly one outcome
- failure remains visible in canonical evaluation truth
- failure must include the truth-view boundary that failed
- failure must not degrade into latest-state evaluation unless a later
  milestone explicitly introduces and certifies such a policy

## Test And Harness Model

Milestone 4 must follow the same structural testing discipline as earlier
bridge milestones.

Expected first-class test surfaces:

- retained-history evaluation scenarios
- branch-local evaluation scenarios
- branch divergence scenarios
- unavailable truth-view failure scenarios
- diagnostics-tier invariance scenarios
- replay parity and replay drift scenarios
- counter certification scenarios

Milestone 4 is not complete with only direct fixture tests. It must establish a
real historical/branch evaluation certification surface on top of
`worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for retained-history and branch-local
  truth worlds
- `MutationBatch` for historical commits, branch divergence, and newer
  publication after source evaluation
- `ExecutionRequest` for selector planning, truth-view materialization,
  delivery, replay, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, diagnostics-tier, and
  branch-divergence sweeps
- `ParitySuite` for profile-to-profile truth-view parity
- `CertificationMatrix` for adversarial retained-history and branch-local
  coverage across multiple profiles

Minimum certification families:

- fixed deterministic historical-view fixtures
- fixed deterministic branch-head and branch-commit fixtures
- seeded branch-divergence matrices
- replay-after-newer-publication certification from canonical historical
  evaluation records
- unavailable truth-view rejection certification
- exact counter assertions for named selector-width and branch-width scenarios
- decision-log certification for selector admission and rejection paths

Rules:

- historical evaluation tests must describe requested truth views, retained
  history, and expected authority basis through harness fixtures rather than ad
  hoc local setup
- profile sweeps must use `ExecutionProfile`, not local booleans or enums
- historical parity checks must use `ParitySuite` where the concern is
  run-to-run equivalence
- adversarial retained-history sweeps must use `CertificationMatrix` where the
  concern is multi-profile hostile coverage

Minimum representative test names:

- `tests::history::historical_commit_view_uses_requested_retained_snapshot`
- `tests::history::branch_head_view_remains_isolated_to_requested_branch`
- `tests::history::branch_divergence_changes_selected_truth_view_explicitly`
- `tests::history::unavailable_historical_view_fails_explicitly`
- `tests::history::replayed_historical_evaluation_matches_original_canonical_artifact`

## Target API And Module Plan

### New Files Expected

- `crates/worth-runtime-bridge/src/snapshot/selection.rs`
- `crates/worth-runtime-bridge/src/snapshot/declaration.rs`
- `crates/worth-runtime-bridge/src/snapshot/policy.rs`
- `crates/worth-runtime-bridge/src/snapshot/authority.rs`
- `crates/worth-runtime-bridge/src/snapshot/materialization.rs`
- `crates/worth-runtime-bridge/src/snapshot/history.rs`
- `crates/worth-runtime-bridge/src/snapshot/decision_log.rs`
- `crates/worth-runtime-bridge/src/delivery/historical.rs`
- `crates/worth-runtime-bridge/src/diagnostics/history.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/historical_evaluation.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/branch_local_evaluation.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/historical_replay.rs`
- `crates/worth-runtime-bridge/src/tests/history/historical_views.rs`
- `crates/worth-runtime-bridge/src/tests/history/branch_views.rs`
- `crates/worth-runtime-bridge/src/tests/history/failures.rs`
- `crates/worth-runtime-bridge/src/tests/history/replay.rs`

### Existing Files Expected To Change

- [mod.rs](../../crates/worth-runtime-bridge/src/snapshot/mod.rs)
- [context.rs](../../crates/worth-runtime-bridge/src/snapshot/context.rs)
- [packet.rs](../../crates/worth-runtime-bridge/src/snapshot/packet.rs)
- [context.rs](../../crates/worth-runtime-bridge/src/routing/context.rs)
- [context.rs](../../crates/worth-runtime-bridge/src/delivery/context.rs)
- [facade.rs](../../crates/worth-runtime-bridge/src/facade.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/harness/mod.rs)
- [facade.rs](../../crates/worth-relational/src/facade.rs)

## Implementation Phases

Milestone 4 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished truth-view authority foundations with
host-local history glue.

### Phase M4.0 - Truth-View Taxonomy And Declaration Boundary Lock

Purpose:

- define the one declarative historical-evaluation surface
- define admitted truth-view selectors
- lock what truth history and branch authority exports versus what the bridge
  classifies
- define explicit unsupported truth-view classes

Required work:

- define `HistoricalEvaluationDeclaration`
- define the exact declaration fields for selector, replay mode, diagnostics
  mode, and delivery intent
- define canonical declaration identity and digest basis
- inventory the retained-history and branch authority surfaces the bridge can
  depend on without reaching through broad relational facades
- define the first closed Milestone 4 truth-view taxonomy
- define the resolved truth-view policy vocabulary
- define canonical truth-view authority basis and digest inputs
- define the explicit relationship between route truth, continuity truth, and
  truth-view truth

Exit criteria:

- the declaration surface is singular and explicit
- callers cannot honestly describe one historical evaluation through scattered
  knobs without violating the spec
- truth-view vocabulary is closed and explicit
- unsupported selector classes are named rather than deferred
- there is no unresolved ambiguity about authority boundaries

### Phase M4.1 - Resolved Policy And Admission Model

Purpose:

- make selector support, retention admission, source capability, replay mode,
  diagnostics mode, and delivery intent resolve once before execution

Required work:

- define `ResolvedTruthViewPolicy`
- define the closed set of policy and admission outcomes for selector support,
  retention availability, source capability, replay compatibility, branch
  compatibility, and diagnostics mode
- define which outcomes are admissible plans versus typed rejections
- define canonical ordering and digest basis for resolved policy
- define the exact seam where raw declaration becomes resolved policy

Exit criteria:

- selector and policy applicability are fully resolved before planning begins
- unsupported or unavailable truth-view requests are represented as typed
  policy outcomes rather than deferred adapter failures
- later phases can consume one monomorphic admitted plan rather than ambient
  context plus conditionals

### Phase M4.2 - Planned Truth-View Packets And Materialization Contract

Purpose:

- make historical and branch-aware evaluation depend on planned truth-view
  packets rather than ad hoc branch or historical lookups

Required work:

- define `BridgeTruthViewSelector` and `ValidatedTruthViewSelectorSet`
- define `PlannedTruthViewPacket`
- define `BridgeTruthViewAuthorityBasis`
- define the narrow bridge-owned historical materialization adapter contract
- define the exact branch, commit, and snapshot basis carried into truth-view
  planning
- define how route truth, continuity truth, and resolved policy are combined
  into one planned packet
- define the exact packet fields that freeze breadth before materialization

Exit criteria:

- the bridge can derive one canonical truth-view packet per requested view
- policy resolution is complete before materialization
- branch and historical resolution are explicit
- delivery no longer needs to discover truth-view breadth

### Phase M4.3 - Canonical Truth-View Materialization, Observation, And Lowering

Purpose:

- lower planned truth views into canonical branch-aware evaluation artifacts
- ensure the materialized read surface is observation-scoped and phase-typed

Required work:

- define `MaterializedTruthViewObservation`
- define `LoweredHistoricalEvaluationArtifact`
- classify requests into materialized, unavailable, unsupported, branch
  mismatch, snapshot mismatch, or historical-resolution rejection
- define exact canonical ordering and digest basis for truth-view authority
- add exact evaluation counters
- define the read-only observation API exposed by
  `MaterializedTruthViewObservation`
- prove that materialization cannot widen retention, retarget branch, or gain
  mutation authority after planning
- define the exact lowering boundary between observation truth and delivered
  bridge artifact truth

Exit criteria:

- historical and branch-local views lower deterministically
- mismatch and unavailability are typed and canonical
- observation handles remain phase-typed and read-only
- lowered artifacts no longer depend on raw truth-runtime handles
- counters and digest bases are specified and test-covered

### Phase M4.4 - Replay Records And Decision-Log Reconstruction

Purpose:

- make historical and branch-aware evaluation reconstructable from canonical
  records rather than ambient runtime context

Required work:

- add canonical historical evaluation records and replay records
- add canonical decision-log records for selector admission and materialization path
- add explanation reconstruction over canonical truth-view authority
- define evaluation identity, declaration identity, resolved policy identity,
  and decision-log identity relationships
- define replay mismatch classification for declaration drift, policy drift,
  authority drift, and schema incompatibility

Exit criteria:

- an older historical evaluation can be reconstructed after newer publication
  without querying ambient latest-state truth
- the decision trail for selector admission and materialization path is
  canonical, queryable, and replay-safe
- replay mismatch classes are closed, typed, and mechanically attributable

### Phase M4.5 - Harness Certification And Hostile Parity Coverage

Purpose:

- make historical and branch-aware evaluation certifiable rather than
  plausible

Required work:

- add `worth-harness` fixtures, parity suites, and certification matrices for
  retained-history and branch-local evaluation
- add hostile replay-after-newer-publication, branch-divergence, and
  unavailable-view failure lanes
- add exact counter assertions for selector width, branch width, and
  materialization path
- add certification coverage for decision-log reconstruction and replay parity

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- replay validates truth-view parity directly
- diagnostics-tier changes richness only, not truth-view truth
- decision-log, policy, and authority truth remain parity-safe across hostile
  profiles

## Explicit Failure Taxonomy For Milestone 4

Milestone 4 must ship typed bridge failures for at least:

- unsupported truth-view selector
- unresolved truth-view policy conflict
- unavailable retained truth view
- branch mismatch against planned authority
- snapshot mismatch against planned authority
- rejected historical resolution
- truth-view authority decode or compatibility failure
- historical evaluation replay mismatch
- historical evaluation delivery rejection

These are bridge failures, not raw parent-runtime strings.

## Anti-Patterns Explicitly Rejected

- treating latest reachable truth as acceptable historical fallback
- declaring one historical evaluation through scattered coordination calls
- choosing the truth branch from ambient host state
- re-deciding source capability or replay admission during materialization
- expressing public historical authority as free-form timestamps or mode
  strings
- rediscovering truth-view breadth during delivery
- burying branch or historical mismatch in explanation-only surfaces
- exposing relational history internals as the bridge historical evaluation API
- letting signal branch semantics substitute for truth branch semantics

## Sequencing Notes

Milestone 4 must land before:

- bulk routing and scale-path planning, because historical/branch breadth must
  be made explicit before it can be optimized honestly
- reactive source protocol productization, because that source contract should
  already include the strongest admitted truth-view modes
- structural-identity-aware remapping, because branch comparison and reuse are
  weaker if canonical historical truth-view authority is still vague
- speculative truth-branch to signal-branch coordination, because speculative
  coordination should build on explicit truth-branch read semantics rather than
  invent them at the same time

Milestone 4 must not attempt to pre-solve:

- generalized host source productization
- speculative preview lifecycle
- multi-parent merge-aware evaluation
- structural-identity comparison policy
- bridge-mediated writeback

Those become stronger because Milestone 4 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the bridge cannot honestly claim historical or branch-aware
evaluation until truth-view authority is explicit, replay-safe, and resistant to
latest-state drift.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of ambient branch selection, latest fallback, and replay that depends on
current truth rather than canonical historical authority.

The milestone preserves authority boundaries because truth still owns retention
and branch semantics, signal still owns execution, and the bridge owns only the
truth-view contract and canonical evaluation artifacts between them.

The milestone defines proof obligations rather than implementation chores
because deterministic truth-view selection, explicit mismatch/unavailability
typing, pre-resolved policy, phase-typed observation, replay parity, and
branch-local certification are required for closeout.

A competent engineer should be able to map this spec into honest types,
subsystems, truth-view packets, counters, and harness suites without inventing
the architecture during implementation.

## Closeout Standard

Milestone 4 is complete only when all of the following are true:

- requested historical and branch-local truth views lower into one canonical
  truth-view packet set
- truth-view planning consults only explicit historical and branch authority
  surfaces
- materialized truth-view authority matches planned branch, commit, and
  snapshot basis exactly
- resolved truth-view policy is complete before materialization starts
- unsupported or unavailable truth views fail explicitly and typed
- branch-local evaluation remains isolated and explicit under divergence
- historical and branch-aware evaluation truth is replay-safe and
  diagnostics-tier-invariant
- harness certification proves retained-history parity, branch isolation,
  replay after newer publication, and explicit failure behavior under hostile
  retained-history pressure

If code lands but historical evaluation still depends on latest-state fallback,
ambient branch selection, delivery-time historical widening, or explanation-only
truth-view identity, Milestone 4 is not complete.
