# Milestone 5 Engineering Spec: Structural Delta Storage And Branch Delta Layering

> **Status:** Completed
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Closeout:** [milestone-5-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5-closeout.md)
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
> **Concurrent milestone:** Milestone 7 (`Durable Schema, Lineage, Cursor, And Checkpoint Artifacts`)
>
> **Primary architectural driver:** make branch persistence scale with semantic delta while preserving one canonical authority model and while leaving schema, lineage, cursor, and checkpoint durability free to ship concurrently without coupling to backend-local delta layout

## Goal

Make branch persistence scale with semantic delta instead of copied full state
so branch creation, branch-local writes, and replay-safe branch history remain
cheap even under deep branch trees.

## Why This Milestone Exists

Milestone 5 is not "compress some branch data."

It is the milestone that decides whether `worth-store` can stay branch-native
at product scale or whether every branch-capable workflow eventually collapses
into hidden full copies, replay-hostile caches, or backend-local tricks that
cannot be explained back through canonical authority.

Milestone 1 locked canonical durable authority.

Milestone 2 locked operating-mode ownership.

Milestone 3 and Milestone 3.5/3.6 locked the durable crash boundary, media
semantics, and recovery-source precedence.

Milestone 4 locked snapshots as derived immutable restore substrates rather
than shadow truth.

Milestone 5 now has to lock a different physical honesty boundary:

- what exact branch state is shared versus branch-local
- what exact unit of delta layering is allowed to exist
- what exact reads may be served through delta layers without pretending the
  physical stack is semantic authority
- what exact rewrite or flattening work is admissible when read amplification
  rises
- what exact branch-local storage promises stay true while Milestone 7 is built
  concurrently for schema, lineage, cursor, and checkpoint durability

If this milestone is weak, later layout, compaction, replication, bulk ingest,
and analysis programs will all pay for it:

- branch creation becomes copied-base storage in disguise
- deep branch histories become unbounded read-amplification traps
- rewrite helpers become backend-local authority because nothing else can
  explain the stack
- concurrent Milestone 7 artifact families end up coupled to delta layout
  details they should never need to know

This milestone exists to make branch-local storage physically honest before
later optimization programs rely on it.

## Hard Part

The hard part is not "store diffs somehow."

The hard part is preserving one exact separation among four things that naive
designs constantly collapse:

- canonical authoritative commit history
- branch-local physical delta layers derived from that history
- branch read plans that may consult stacked deltas for cost reasons
- rewrite products that exist only to keep the physical stack bounded

The design fails if:

- branch creation cost silently scales with baseline branch size
- delta layers are treated as authoritative state instead of derived storage
- reads stop being replay-safe because they depend on backend-local rewrite
  residue
- rewrite decisions are heuristic but invisible, so later code cannot tell when
  it paid O(depth) versus O(rewritten_layers)
- concurrent Milestone 7 work has to depend on delta stack shape to find schema
  boundaries, lineage meaning, cursor basis, or checkpoint authority

Milestone 5 therefore has to make delta layering cheap enough to matter and
strictly non-authoritative enough that the rest of the store can survive if
every delta stack is rebuilt tomorrow.

## Explicit Assumptions

- Milestone 1 authoritative artifact families remain the only semantic durable
  truth authority.
- Milestone 2 operating-mode boundaries remain unchanged; Milestone 5 is a
  storage-shape milestone, not a new mode program.
- Milestone 3 and Milestone 3.5/3.6 already make durable publication and
  restart exact enough that branch-local delta layers may be treated as derived
  durable artifacts without softening crash semantics.
- Milestone 4 snapshots remain independent derived artifacts; delta layers may
  support them physically later, but Milestone 5 does not redefine snapshot
  basis or restore meaning.
- `worth-relational` still owns branch semantics, commit legality, ordered
  parent meaning, lineage semantics, and canonical replay meaning.
- branch deltas in this milestone are derived from canonical commit envelopes
  and branch ancestry; they are not a second commit language.
- Milestone 7 will be authored and implemented concurrently enough that
  Milestone 5 must preserve stable branch/frontier identity surfaces for schema
  boundaries, lineage events, cursor positions, and embedded checkpoints
  without taking ownership of those artifact families itself.
- aspect-aware physical layout, structural blocks, retention compaction, and
  replication remain later milestones even if this milestone reserves hooks
  they will need.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile structural
  failure before feature convenience spreads. Milestone 5 therefore starts from
  copied-state explosion and deep-stack read amplification, not from "branches
  already kind of work."
- `arch_laws.md`
  The most important thing it protects here is keeping authority, derivation,
  and proof progression separate. Law 33 matters directly: branch deltas must
  remain derived durable artifacts rebuildable from authority. Law 41 matters
  too: branch basis selection, persisted delta layers, rewrite-admitted stacks,
  and replay-verified branch reads must be distinct proof-bearing types.
- `perf_laws.md`
  The most important thing it protects is explicit cost honesty. Milestone 5
  therefore has to name branch-create, delta-read, and delta-rewrite cost bases
  now, with exact counters for traversed layers and rewrite breadth, rather
  than hiding depth costs behind a cheap-looking branch API.
- `domain_laws.md`
  The most important thing it protects is responsibility-shaped decomposition.
  Delta basis selection, branch-base identity, layer persistence, read
  planning, stack rewrite, and certification evidence must live in separate
  subdomains instead of one generic branch-storage helper.
- `worth_store_vision.md`
  The most important thing it protects is that store persists truth without
  making it dumber. Milestone 5 must therefore keep canonical commit artifacts
  authoritative while making branch-local physical storage proportional to
  change instead of copied full state.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 5 belongs here
  because later aspect-aware layout, compaction, replication, and bulk chunking
  need an already-honest branch-delta substrate.
- `worth-store/test-requirements.md`
  The most important thing it protects is certification-grade proof.
  Milestone 5 is not closed until the `Branch Delta Proportionality And Replay
  Parity Test` proves no-edit branches stay near-free, growth tracks delta, and
  replay stays parity-safe.
- `milestone-3.md`
  The most important thing it protects is exact durable publication and replay
  parity. Milestone 5 must build on that durable authority path instead of
  creating a second branch-truth representation inside physical layering.
- `milestone-3.5-3.6.md`
  The most important thing it protects is declared media barriers and recovery
  source precedence. Delta layers and rewrite products must therefore
  participate as derived families that can be quarantined, rebuilt, or ignored
  under precedence rules rather than outranking authority because they look
  newer or cheaper.
- `milestone-4.md`
  The most important thing it protects is snapshot non-authority. Milestone 5
  must preserve the same pattern: branch delta layers may accelerate reads and
  rebuilds later, but snapshots and delta layers must both remain subordinate
  to canonical commit history.

## Adversarial Constraint

Milestone 5 must survive this hostile condition:

> A store with deep branch trees, many near-empty speculative branches, long
> branch-local edit chains, occasional delta-stack rewrites, deleted and
> rebuilt derived branch layers, and concurrent Milestone 7 durable artifact
> development must preserve the same branch-visible truth, replay conclusions,
> and branch-head meaning as a control lane that ignores derived delta stacks
> and reconstructs branch state from canonical authoritative commit history
> alone.

## Product Decision Lock

- branch deltas are always classified as derived durable artifacts
- branch creation must default to shared-base semantics rather than copied-base
  persistence
- one branch-local physical layer corresponds to a declared delta basis over
  canonical commits and branch ancestry, not an implicit backend snapshot
- rewrite or flattening of delta stacks is allowed only as a derived storage
  optimization with explicit replay-parity obligations
- branch reads may use delta stacks or rewritten stack products for cost, but
  the resulting truth must always be restatable through canonical branch
  frontier plus authoritative history
- branch-head authority remains owned by canonical authoritative artifacts; no
  delta stack tip, rewrite product, or cached merged image may act as the
  authoritative branch head
- Milestone 5 must publish stable branch-scope and frontier vocabulary that
  concurrent Milestone 7 work can reference, but Milestone 5 must not absorb
  schema-boundary, lineage, cursor, or checkpoint meaning into delta-layer
  metadata
- deleting all delta layers must leave canonical replay and authoritative
  rebuild intact, even if that fallback is slower

Normative consequence:

- any implementation that materializes a fresh full branch copy on normal
  branch creation is out of spec
- any implementation that makes branch-visible truth depend on backend-local
  rewrite residue that cannot be reconstructed from authority is out of spec
- any implementation that requires Milestone 7 artifact families to understand
  delta stack shape in order to remain meaningful is out of spec

## Scope

### In Scope

- branch-local structural delta storage derived from canonical commit history
- shared-base branch creation with explicit branch-base identity
- explicit delta-layer identity, ancestry, and basis records
- branch reads through stacked delta layers with replay-parity verification
- deterministic delta-stack rewrite or flattening rules
- read-amplification accounting and typed stack-management policy surfaces
- rebuild of delta layers from canonical authoritative artifacts
- stable branch/frontier identity surfaces that concurrent Milestone 7 durable
  artifacts can reference without learning delta-layout internals
- counters, diagnostics, and certification bundles for branch creation,
  delta-read breadth, rewrite breadth, and replay parity

### Explicitly Out Of Scope

- schema-boundary persistence, lineage-event persistence, cursor durability, and
  embedded checkpoint durability as first-class artifact families
- aspect-aware physical layout and structural block deduplication
- retention, compaction, reclaim, or replication policy beyond the minimum
  needed to keep delta-derived status honest
- live-query continuation over durable cursors
- bulk-ingest chunking strategy beyond the branch-delta substrate later bulk
  work will need
- snapshot semantics or snapshot-plus-tail restore semantics already defined in
  Milestone 4

## Delta Authority Model

### Admitted Delta Ontology Rule

Milestone 5 must freeze the admitted delta ontology instead of leaving
"structural delta" as a backend-defined slogan.

Every published branch delta layer in this milestone must be composed only from
declared delta fragment families such as:

- `EntityIntroductionDelta`
- `EntityRemovalDelta`
- `AspectValueDelta`
- `RelationEdgeDelta`
- `BranchScopeTombstoneDelta`

The exact family names may change, but the rules may not:

- each fragment family must represent one singular kind of branch-visible
  change
- fragment families must be canonical durable data, not opaque backend patch
  blobs
- fragment-family equality must be defined by a declared equivalence contract,
  not by backend serialization byte equality alone
- two layers that claim the same semantic effect over the same basis must
  canonicalize to the same fragment ordering and digest basis or fail
  explicitly

Not admitted in this milestone:

- backend-local row diffs whose semantic meaning cannot be reconstructed
- mixed "catch-all patch items" that collapse unrelated delta categories
- full-state shadow images described as "delta layers"

This is the line that prevents Milestone 5 from turning into "whatever patch
format happened to be convenient."

### Delta Non-Authority Rule

Branch delta layers are derived durable artifacts.

They are allowed to accelerate:

- branch creation
- branch-local reads
- later physical layout and chunked ingest work

They are not allowed to define:

- branch-head authority
- canonical commit ancestry
- ordered parent meaning
- schema-boundary meaning
- lineage identity meaning
- cursor or checkpoint meaning

Normative rule:

- if all delta layers are deleted, the store must still be able to reconstruct
  the same branch-visible truth from canonical authoritative artifacts alone
- if a delta read or rewrite product disagrees with canonical replay, the delta
  family is wrong and must be rejected or rebuilt; canonical replay is not
  allowed to bend toward the derived layer

This is the anti-shadow-authority line for Milestone 5.

### Branch Delta Basis Rule

Every admitted branch delta layer must bind to one exact branch basis.

Minimum basis fields:

- `BranchDeltaLayerId`
- `BranchId`
- `BaseFrontier`
- `TargetFrontier`
- `DerivedFromCommitRange`
- `DeltaFamilyVersion`
- `AuthorityBasisDigest`

Required meaning:

- `BaseFrontier`
  identifies the exact branch-visible frontier the layer starts from
- `TargetFrontier`
  identifies the exact branch-visible frontier reached after applying the layer
- `DerivedFromCommitRange`
  identifies the closed canonical history interval whose effects are encoded by
  the layer
- `AuthorityBasisDigest`
  binds the layer to the authoritative artifact identity it claims to compress

Normative rules:

- one delta basis corresponds to one exact branch-local transition surface
- a layer may not be published against "whatever branch head exists when the
  write finishes"; base and target frontiers must be selected before
  publication
- if branch head advances while a layer is being derived, that later truth
  belongs to a later layer or later replay, not to the current publication unit

### Delta Equivalence Contract Rule

Milestone 5 must define reuse and comparison semantics explicitly for delta
layers and rewrite products.

Required sameness basis:

- branch scope
- base frontier
- target frontier
- canonical delta fragment ordering
- fragment-family comparator rules
- duplicate-fragment handling rules
- digest basis for semantic equivalence

Required rules:

- duplicate fragments must either collapse deterministically or fail typed
- unordered fragment collections must canonicalize through a declared comparator
- rewrite equivalence must compare semantic fragment meaning, not only byte
  equality of replacement storage
- any cache, dedup, suppression, or rewrite reuse surface built later must
  consume this explicit equivalence contract rather than inventing its own

This is the Law 26 protection against equivalence drift.

### Delta Layer Identity Rule

Milestone 5 must define one canonical `BranchDeltaLayerId` distinct from:

- commit identity
- branch identity
- snapshot identity
- durable mutation identity
- checkpoint identity

Required rules:

- `BranchDeltaLayerId` is assigned before delta publication begins
- one complete published layer family corresponds to one
  `BranchDeltaLayerId`
- rewrite products either:
  - produce a new `BranchDeltaLayerId` plus explicit lineage to replaced
    layers, or
  - reuse identity only if the family guarantees bitwise and semantically
    identical republishing under one global rule
- the implementation may not mix those models opportunistically

Milestone 5 should prefer explicit rewrite lineage over identity reuse if there
is any risk that certification or operator tooling would lose track of which
layers were replaced.

### Branch Creation Cost Rule

Milestone 5 must make near-free branch creation mechanical rather than
aspirational.

Required rule:

- creating a branch from an existing branch frontier publishes branch identity
  and base-reference metadata without copying baseline branch state into a fresh
  branch-local full image

Allowed creation work in this milestone:

- persist branch record and branch-base reference
- persist zero or minimal empty-layer metadata when the family requires an
  explicit initial layer
- update authoritative branch-head metadata through already-admitted authority
  paths where needed

Forbidden creation work in this milestone:

- materializing a full copied branch image by default
- replaying the entire base branch into a new branch-local cache as a hidden
  requirement of creation success
- hiding copied-base work inside an initialization helper while still claiming
  near-free branch creation

### Delta Stack Rewrite Rule

Milestone 5 admits deterministic rewrite of delta stacks only as a derived
boundedness program.

Required rewrite model:

- rewrite consumes one declared stack segment of published delta layers
- rewrite emits one replacement derived layer or replacement family with
  explicit replacement lineage
- rewrite preserves the same branch-visible truth, target frontier, and replay
  conclusions as the replaced segment
- rewrite never changes authoritative branch-head or commit history meaning

Required triggers in this milestone:

- explicit typed stack-management request
- or declared policy driven by observable read-amplification counters

Forbidden rewrite behavior:

- opportunistic silent rewrite that changes replay-visible meaning
- treating rewritten products as authoritative because the original stack was
  pruned
- rewrite admission based only on backend convenience rather than a declared
  stack segment and exact replacement lineage

### Delta Read Admission Rule

Milestone 5 admits branch reads through delta layers only through explicit
branch-and-frontier vocabulary.

Required read classes:

- `BranchDeltaRead`
  read branch-visible truth at a declared target frontier through an admitted
  stack of delta layers
- `AuthorityReplayControlRead`
  reconstruct the same truth from canonical authoritative history without
  relying on delta layers

Rules:

- `BranchDeltaRead` must declare the target branch scope and target frontier
  explicitly
- the store may widen from one layer to multiple layers or to a rewritten
  segment, but it may not silently widen into backend-local full-state truth
  whose basis is not declared
- every admitted delta read must be parity-comparable with the
  `AuthorityReplayControlRead` for the same frontier

This is the anti-"delta-ish read" rule.

### Target Frontier Legality Rule

Milestone 5 must make read and rewrite target legality exact rather than
convenient.

Allowed target-frontier classes in this milestone:

- `TargetEqualsBaseFrontier`
  zero-delta read for a declared branch frontier
- `TargetDescendsFromBaseFrontierOnSameBranch`
  read or rewrite over a simple same-branch descendant path

Rejected target-frontier classes in this milestone unless a later milestone
widens them explicitly:

- target frontier older than the declared base frontier
- target frontier reachable only through merge-parent ambiguity
- target frontier on a different branch scope
- target frontier whose canonical commit path from base is ambiguous,
  unavailable, or not machine-provable

Required outcomes:

- `BranchDeltaTargetAdmitted`
- `BranchDeltaTargetRejected`
- `BranchDeltaTargetRequiresLaterMergeAwareMilestone`

Normative consequence:

- Milestone 5 may not silently "pick one replay path" through merge topology
- merge-aware branch-layer targeting must remain a later explicit widening if it
  is needed

### Milestone 7 Concurrency Boundary

Milestone 5 and Milestone 7 are allowed to progress concurrently only if their
authority boundaries stay explicit.

Milestone 5 owns:

- branch-base sharing and branch-local delta layering
- delta read planning and rewrite boundedness
- branch/frontier identity surfaces needed by derived physical storage

Milestone 7 owns:

- schema evolution boundary durability
- lineage-event durability and historical identity resolution
- durable cursor and subscriber checkpoint persistence
- embedded-mode checkpoint artifact persistence

Required concurrency rule:

- Milestone 5 must expose stable branch scope, frontier, and derived-basis
  identifiers that Milestone 7 families can reference
- Milestone 7 artifact meaning must not depend on the current delta stack shape
  or rewrite history to remain valid
- Milestone 5 must not smuggle schema, lineage, cursor, or checkpoint meaning
  into delta-family metadata merely because those milestones are being built at
  the same time
- if a concurrent design choice would force one milestone to reinterpret the
  other milestone's authority, the design must be rejected and the boundary
  tightened first

### Compile-Time Boundary Enforcement Rule

Milestone 5 must not leave its most dangerous mistakes as documentation-only
warnings.

Minimum required witness or typestate surfaces:

- `SharedBaseBranchCreationWitness`
  proves the branch-creation path is allowed to publish a shared-base branch
  without copied-base initialization
- `SameBranchDescendantWitness`
  proves a target frontier is an admitted same-branch descendant of the base
  frontier
- `RewriteEligibleDeltaSegment`
  proves a declared delta stack segment is contiguous, published, and eligible
  for rewrite
- `Milestone7IndependentReference`
  proves a schema/lineage/cursor/checkpoint-facing reference uses branch or
  frontier authority vocabulary only, not delta-stack-shape internals

Rules:

- constructors for these witnesses must be sealed to the proving subsystem
- branch creation may not accept a raw branch id plus raw base frontier when a
  `SharedBaseBranchCreationWitness` is required
- rewrite execution may not accept an arbitrary vector of layer ids when a
  `RewriteEligibleDeltaSegment` is required
- Milestone 7-facing adapters must not accept delta-layer handles where a
  `Milestone7IndependentReference` is required
- key misuse cases must be proven by compile-fail tests, not only runtime
  rejection

This is the line that upgrades the spec from "please do it right" to
"the public API makes the wrong thing hard or impossible."

## Proof-Carrying Delta Pipeline

Law 41 is load-bearing here.

Milestone 5 should encode branch-delta work as a proof chain rather than as one
storage helper that "kind of knows" whether a layer is valid.

Minimum intended phase sequence:

- `SelectedBranchDeltaBasis`
- `DeltaPublicationAdmittedPlan`
- `PersistedBranchDeltaLayer`
- `PublishedBranchDeltaLayer`
- `RewriteAdmittedDeltaSegment`
- `PublishedRewrittenDeltaSegment`
- `ReplayVerifiedBranchDeltaRead`

Rules:

- each later type consumes the prior proof-bearing type
- constructors for proof-bearing delta types must be crate-sealed
- fields carrying base frontier, target frontier, commit-range basis, and
  replacement lineage evidence must remain private
- rewrite execution must not accept a weaker type than
  `RewriteAdmittedDeltaSegment`
- branch-visible reads through delta layers must terminate in
  `ReplayVerifiedBranchDeltaRead`, not in raw backend records
- target-frontier legality must already be proven before
  `DeltaPublicationAdmittedPlan`, `RewriteAdmittedDeltaSegment`, or
  `ReplayVerifiedBranchDeltaRead` can exist

This is what makes "half-published layer," "rewrite with unknown replacement
scope," and "branch read from unverified stack residue" structurally harder to
express.

## Performance Architecture

Milestone 5 must encode performance into the architecture itself rather than
treating it as counters attached after the fact.

### Strategy-Typed Read Rule

Branch reads must expose the chosen execution strategy as a first-class
architectural decision.

Minimum admitted read strategies in this milestone:

- `DirectDeltaLayerReadPlan`
  read through an admitted bounded layer stack
- `RewrittenSegmentReadPlan`
  read through a replacement segment already proven parity-safe
- `AuthorityReplayControlReadPlan`
  reconstruct truth from canonical authoritative history as the explicit
  control lane

Rules:

- the planner must choose one of these strategies before execution begins
- the executor must consume a lowered strategy-typed plan rather than
  rediscovering the strategy mid-read
- if a requested read cannot stay within an admitted Milestone 5 strategy, it
  must fail typed or enter the explicit control lane; it may not silently drift
  into an unplanned slow path

This is the Law 27 line for branch reads: lowered plans only.

### Strategy-Typed Rewrite Rule

Delta rewrite must also be strategy-typed instead of "rewrite whatever seems
profitable."

Minimum admitted rewrite decisions:

- `RewriteNotNeeded`
- `RewriteAdmittedSegment`
- `RewriteRejectedAsTooBroad`
- `RewriteDeferredAsDebt`

Rules:

- rewrite strategy must be resolved before rewrite execution begins
- the executor may not broaden a bounded rewrite segment into a wider rewrite
  without producing a different typed decision
- any rewrite path that needs full-stack rewrite to succeed must surface that
  fact explicitly as rejection or debt, not as hidden implementation behavior

### Explicit Cost-Regime Rule

Milestone 5 must distinguish sparse and dense branch-read regimes
architecturally.

Minimum regime vocabulary:

- `ShallowLayerTraversal`
- `RewritePreferredTraversal`
- `AuthorityReplayControlRegime`

Rules:

- the result envelope must state which regime executed
- regime changes must be visible to callers and certification bundles
- a path whose cost semantics change across regimes must not pretend to be one
  uniform cheap read surface

This is the anti-cost-dishonest abstraction rule.

### Typed Fallback Rule

Expensive fallback paths must be explicit typed outcomes, not invisible runtime
rescues.

Minimum typed fallback outcomes:

- `RequiresAuthorityReplayControlLane`
- `RequiresRewriteBeforeAdmittedRead`
- `TargetRequiresMergeAwareWidening`
- `RejectedForBranchBaseMaterializationRisk`

Rules:

- execution may not silently enter authoritative replay, broad rewrite, or
  merge-path search from an admitted Milestone 5 delta-read path
- any fallback that is allowed as explicit debt must be represented in the
  result envelope and counter surface as debt, not ordinary success

### Locality Object Rule

Locality and touched scope must be represented as first-class inputs to
planning, not rediscovered heuristically during execution.

Minimum locality objects:

- `BranchDeltaScope`
- `BranchDeltaTarget`
- `RewriteSegmentWidth`
- `TouchedFragmentSet`
- `AuthorityReplayRange`

Rules:

- read planning must consume explicit locality objects
- rewrite planning must consume explicit width and touched-fragment scope
  objects
- later milestones may refine these objects, but Milestone 5 must establish
  them now so aspect-aware layout, structural blocks, and bulk chunking inherit
  real locality vocabulary instead of reverse-engineering it

### Result-Envelope Performance Rule

Performance accounting must be embedded in operation results, not left inside
internal diagnostics.

Minimum required result-envelope fields for admitted reads and rewrites:

- `strategy`
- `regime`
- `complexity_status`
- `layers_traversed`
- `records_decoded`
- `replay_breadth`
- `rewrite_breadth`
- `fallback_class`

Rules:

- `complexity_status` must distinguish at least `Verified` and `Debt`
- `fallback_class` must explicitly report `None` versus the named fallback that
  occurred
- callers must be able to tell from the returned value whether the operation
  remained within the admitted Milestone 5 cost surface

### Budget Contract Rule

Milestone 5 must define local architectural budgets now even though full store
admission control lands later.

Minimum local budget contracts:

- `MaxDirectLayerTraversalDepth`
- `MaxRewriteSegmentWidth`
- `MaxAdmittedReplayBreadthForDeltaReadParity`
- `MaxAdmittedBranchCreationMaterializationBreadth`

Rules:

- exceeding one of these budgets must produce a typed decision, typed reject,
  or explicit `Debt` classification
- budgets may be configuration-backed later, but the architectural meaning of
  each budget must be fixed in this milestone
- no admitted fast path may quietly exceed its local budget and still report as
  ordinary verified success

### Compile-Time Performance Boundary Rule

The highest-risk performance boundaries must be enforced by type construction,
not only by runtime counters.

Required compile-time posture:

- `DirectDeltaLayerReadPlan` may be constructed only from a proof that the
  target is same-branch descendant and within admitted direct-read depth
- `RewriteAdmittedDeltaSegment` may be constructed only from a proof that the
  segment width is within admitted rewrite bounds
- Milestone 7-facing reference types may not carry delta-layer topology fields
  that would force downstream code to branch on performance internals

Required proof surface:

- compile-fail tests for illegal direct-read planning when no descendant or
  bounded-depth proof exists
- compile-fail tests for rewrite execution from raw layer collections
- compile-fail tests for M7-facing references that attempt to encode delta-stack
  shape

## Public Surface

Milestone 5 must keep the public facade explicit and branch/frontier oriented.

Representative surface:

```rust
pub struct BranchCreationRequest { ... }
pub struct BranchDeltaReadRequest { ... }
pub struct DeltaRewriteRequest { ... }

pub struct PublishedBranchDeltaLayerHandle { ... }
pub struct DeltaRewritePlan { ... }
pub struct DeltaRewriteOutcome { ... }

impl WorthStore {
    pub fn create_branch_with_shared_base(
        &mut self,
        request: BranchCreationRequest,
    ) -> Result<BranchHandle, BranchCreationError>;

    pub fn read_branch_via_delta_layers(
        &self,
        request: BranchDeltaReadRequest,
    ) -> Result<BranchReadResult, BranchReadError>;

    pub fn plan_delta_rewrite(
        &self,
        request: DeltaRewriteRequest,
    ) -> Result<DeltaRewritePlan, DeltaRewritePlanningError>;

    pub fn execute_delta_rewrite(
        &mut self,
        plan: DeltaRewritePlan,
    ) -> Result<DeltaRewriteOutcome, DeltaRewriteError>;

    pub fn rebuild_branch_delta_layers(
        &mut self,
        branch_id: BranchId,
    ) -> Result<Vec<PublishedBranchDeltaLayerHandle>, DeltaRebuildError>;
}
```

Surface rules:

- branch and delta APIs must expose branch/base/frontier vocabulary directly
- branch creation must expose shared-base intent, not a generic "clone branch"
  surface that hides copied-state cost
- rewrite planning and rewrite execution should remain distinct public concepts
  if the implementation needs an admissibility boundary
- read and rewrite surfaces must stay in store-owned vocabulary, not raw
  backend file or row vocabulary
- no API may imply that delta layers are authoritative truth rather than
  declared derived branch-storage artifacts

## Required Internal Subsystems

Milestone 5 must decompose by responsibility:

- `delta/basis/`
  branch-base selection, frontier binding, and authority-basis identity
- `delta/layers/`
  immutable layer persistence, lineage, and publication
- `delta/read/`
  branch read planning and replay-parity verification
- `delta/rewrite/`
  stack rewrite planning, execution, and replacement lineage
- `delta/rebuild/`
  rebuild from canonical authoritative artifacts
- `delta/evidence/`
  counters, proportionality bundles, and certification output
- `backend/`
  backend support for delta families without owning delta semantics

This is the `domain_laws.md` line for Milestone 5: basis selection, layer
publication, branch reads, stack rewrite, and rebuild do not change for the
same reasons and must not share one god-module.

## Invariant Allocation Table

| Invariant | Proving Phase | Enforcing Subsystem | Failure Family | Certification Surface |
| --- | --- | --- | --- | --- |
| branch creation does not copy baseline state by default | branch creation admission | `delta/basis/` and `delta/layers/` | `BranchBaseCopyViolation` | `delta_storage_report` |
| each delta layer binds one exact base and target frontier | basis selection and publication | `delta/basis/` and `delta/layers/` | `BranchDeltaBasisAmbiguous` or `BranchDeltaPublicationGap` | `artifact_digest` and `truth_digest` |
| branch delta reads match authoritative replay for the same frontier | read verification | `delta/read/` | `BranchDeltaReplayParityViolation` | `truth_digest` and `history_digest` |
| delta rewrites preserve branch-visible truth and replacement lineage | rewrite planning and execution | `delta/rewrite/` | `BranchDeltaRewriteParityViolation` or `BranchDeltaReplacementGap` | `delta_storage_report` and `counter_snapshot` |
| concurrent Milestone 7 artifacts do not depend on delta stack shape | cross-milestone boundary review and tests | `delta/basis/` plus certification harness | `ConcurrentArtifactBoundaryViolation` | concurrency parity bundle |
| deleted delta layers remain rebuildable from authority | rebuild | `delta/rebuild/` | `BranchDeltaRebuildFailure` | rebuild parity bundle |
| branch-head authority never migrates into delta layers | read and rebuild verification | `delta/read/` and `delta/rebuild/` | `BranchDeltaShadowAuthorityViolation` | control-vs-delta parity bundle |
| same-branch descendant legality is proven before delta read or rewrite | target admission | `delta/basis/` and `delta/read/` | `BranchDeltaTargetIllegal` | target-legality bundle |
| Milestone 7 references do not encode delta-stack shape | cross-milestone witness construction | `delta/basis/` and adapters for M7-facing references | `ConcurrentArtifactBoundaryViolation` | compile-fail boundary proof |

## Failure Taxonomy

Milestone 5 must ship an explicit typed error family matrix at minimum
covering:

- `BranchDeltaBasisAmbiguous`
- `BranchDeltaBasisUnsupported`
- `BranchDeltaPublicationGap`
- `BranchDeltaDigestMismatch`
- `BranchBaseCopyViolation`
- `BranchDeltaReadTargetIllegal`
- `BranchDeltaTargetRequiresMergeAwareWidening`
- `BranchDeltaReplayParityViolation`
- `BranchDeltaRewriteTargetIllegal`
- `BranchDeltaRewriteParityViolation`
- `BranchDeltaReplacementGap`
- `BranchDeltaRebuildFailure`
- `BranchDeltaShadowAuthorityViolation`
- `ConcurrentArtifactBoundaryViolation`
- `BranchDeltaFamilyVersionUnsupported`
- `BranchDeltaIntegrityFailure`

Rules:

- create, read, rewrite, rebuild, and verification paths must map failures
  into these families or explicit refinements of them
- backend-driver failures must not leak as the public semantic error taxonomy
- typed failures must stay stable enough for certification bundles and later
  operator diagnostics

## Complexity Contracts

Milestone 5 must name the hot-path and boundedness cost basis explicitly.

Minimum contracts:

- branch creation cost is proportional to:
  - one branch record publication
  - one shared-base reference publication
  - zero copied baseline records on the admitted near-free path
- branch delta read cost is proportional to:
  - delta layers traversed for the declared frontier
  - records decoded from those layers
  - replay-verification breadth for the same frontier
- delta rewrite cost is proportional to:
  - layers replaced in the declared segment
  - records emitted into the replacement family
  - parity-verification breadth for that replacement
- delta rebuild cost is proportional to:
  - authoritative commit range replayed into the rebuilt layers
  - replacement layer records emitted

Forbidden fallback work that must be made mechanically visible:

- hidden full-base materialization during admitted near-free branch creation
- hidden authoritative replay fallback on every delta read
- hidden full-stack rewrite when only a declared segment was admitted
- hidden merge-path search for targets not covered by the admitted legality
  classes

Minimum counters:

- `branch_create_count`
- `branch_base_reuse_count`
- `branch_base_copy_count`
- `branch_hidden_full_base_materialization_count`
- `branch_delta_read_count`
- `branch_delta_layers_traversed_count`
- `branch_delta_read_record_count`
- `branch_delta_authority_replay_fallback_count`
- `branch_delta_rewrite_count`
- `branch_delta_rewrite_layers_replaced_count`
- `branch_delta_rewrite_record_count`
- `branch_delta_hidden_full_stack_rewrite_count`
- `branch_delta_merge_path_search_count`
- `branch_delta_rebuild_count`
- `branch_delta_rebuild_record_count`
- `branch_delta_integrity_failure_count`
- `concurrent_artifact_boundary_rejection_count`

Milestone 5 may add richer counters, but it may not hide read amplification or
rewrite breadth behind a cheap-looking branch API.

Required counter assertions:

- `branch_base_copy_count` and
  `branch_hidden_full_base_materialization_count` must remain zero on the
  admitted shared-base creation path
- `branch_delta_authority_replay_fallback_count` must remain zero for the
  representative admitted delta-read lanes; any non-zero lane must be named as
  explicit debt or explicit control lane
- `branch_delta_hidden_full_stack_rewrite_count` must remain zero when the plan
  admits only a bounded rewrite segment
- `branch_delta_merge_path_search_count` must remain zero for Milestone 5
  admitted target classes because merge ambiguity is out of scope, not an
  implicit fallback

Debt posture:

- if the first implementation needs authority replay fallback or broad rewrite
  fallback outside the explicit control lane, the contract must be marked
  `Debt` with the exact triggering conditions named
- Milestone 5 may not imply verified proportionality while relying on silent
  fallback paths

## Phases

### Phase 1: Lock Delta Authority Boundaries And Basis Vocabulary

Phase 1 defines what a branch delta layer is allowed to mean before any
backend-specific storage optimization lands.

Required work:

- define branch delta basis fields and identity basis
- define the non-authority rule, branch-creation cost rule, and rewrite rule
- define the proof-bearing delta pipeline
- define read classes and control-lane parity rules
- define strategy types, locality objects, typed fallbacks, and local budget
  contracts
- define the Milestone 7 concurrency boundary in structural terms

Exit condition:

- a branch delta layer has one exact basis vocabulary
- near-free branch creation is defined in terms of shared-base semantics rather
  than aspiration
- concurrent Milestone 7 work has an explicit boundary to build against

### Phase 2: Persist Branch Base And Delta Layer Families

Phase 2 makes branch-base sharing and published delta layers real as derived
artifact families.

Required work:

- implement branch-base reference persistence
- implement delta-layer publication with basis, identity, and integrity
  records
- implement restore-admitted publication boundary for delta families
- expose typed branch-creation and delta-publication failures
- emit exact branch-create and layer-publication counters

Exit condition:

- branch creation no longer depends on copied-state initialization
- delta layers can be durably published as complete derived artifacts
- incomplete or damaged delta families are not admitted as branch-read truth

### Phase 3: Expose Branch Reads And Creation Through Delta Layers

Phase 3 turns delta layering into a real branch surface instead of latent bytes
on disk.

Required work:

- implement shared-base branch creation through the public facade
- implement branch delta reads against explicit target frontiers
- implement strategy-typed read planning and result envelopes
- implement authoritative replay control reads for parity comparison
- expose typed illegal-target and parity failures
- emit exact traversal and decode counters

Exit condition:

- branch creation is physically cheap on the admitted path
- branch delta reads remain branch-and-frontier explicit
- delta reads are mechanically comparable with authoritative replay

### Phase 4: Implement Deterministic Delta Stack Rewrite And Read-Amplification Control

Phase 4 makes stacked delta reads operationally honest instead of leaving deep
stacks to drift forever.

Required work:

- implement rewrite planning over declared stack segments
- implement strategy-typed rewrite decisions and budget-driven reject/debt
  outcomes
- implement rewrite execution with replacement lineage
- implement policy or explicit-request surfaces that trigger rewrite from
  observable read-amplification truth
- verify rewritten segments against authoritative replay parity
- emit rewrite-breadth and replacement counters

Exit condition:

- deep delta stacks can be bounded without redefining branch truth
- rewrite lineage is explicit and machine-checkable
- read amplification becomes visible policy input instead of folklore

### Phase 5: Prove Delta Proportionality And Replay Parity

Phase 5 turns branch delta layering into a certifiable branch-storage substrate
rather than an optimistic optimization.

Required work:

- run the Milestone 5 named suite:
  `Branch Delta Proportionality And Replay Parity Test`
- compare no-edit, small-edit, and deep-branch lanes against authoritative
  replay control lanes
- compare rewritten-stack lanes against unrevised parity lanes
- emit machine-checkable truth, history, delta-storage, and counter bundles

Exit condition:

- no-edit branches remain near-free
- branch-local growth tracks semantic delta rather than copied base size
- delta-read and rewrite lanes match authoritative replay conclusions
- Milestone 5 closeout evidence exists in machine-checkable form

## Must Ship

- shared-base branch creation with explicit branch-base identity
- structural branch-delta layer family with explicit basis and integrity
  records
- branch reads through admitted delta layers with authoritative replay parity
- deterministic delta-stack rewrite with explicit replacement lineage
- rebuild of delta layers from canonical authoritative artifacts
- stable branch/frontier identity surfaces suitable for concurrent Milestone 7
  artifact references
- typed delta create/read/rewrite/rebuild failures
- exact counters and machine-checkable Milestone 5 certification output
- strategy-typed result envelopes and local budget contracts for reads and
  rewrites

## Must Preserve

- canonical commit history remains the only semantic durability authority
- branch heads and branch ancestry remain authoritative and replay-stable
- delta layers remain derived durable artifacts
- branch creation remains proportional to shared-base metadata, not copied
  baseline state
- rewrite products remain subordinate to canonical replay semantics
- concurrent Milestone 7 artifact families remain meaningful without delta
  stack-shape knowledge
- backend variation does not change branch-visible truth meaning

## Acceptance Evidence

Milestone 5 is complete only when the store satisfies the named Milestone 5
suite:

- `Branch Delta Proportionality And Replay Parity Test`

Required machine-checkable outputs:

- `truth_digest`
- `history_digest`
- `delta_storage_report`
- `counter_snapshot`

Milestone-specific proof obligations:

- no-edit branches remain near-free instead of copying base state
- branch-local storage growth tracks semantic delta rather than baseline size
- branch delta reads match authoritative replay for the same frontier
- rewritten delta segments preserve truth-visible meaning
- direct-read, rewrite, and control-lane strategy choices remain explicit and
  cost-honest in the result envelopes
- concurrent Milestone 7 reference surfaces do not force cross-milestone
  authority leakage

Milestone 5 is not closed by "branch reads were faster" tests.

## Architectural Notes

- The smart abstraction is not "branch cache." The smart abstraction is one
  exact branch-basis-and-delta-layer contract with replay-parity verification
  around it.
- Delta layer layout may vary by backend, but branch basis, replacement
  lineage, replay parity, and non-authority rules may not.
- Rewrite planning and rewrite execution should stay separate subdomains even if
  one backend initially implements both.
- Milestone 7 concurrency is a boundary test, not a collaboration slogan. If a
  delta design choice would force schema, lineage, cursor, or checkpoint
  durability to understand stack internals, the delta design is wrong.
- Milestone 6, Milestone 9, and Milestone 10 should inherit this delta basis
  contract rather than renegotiating what a branch layer means.

## Sequencing Notes

This milestone belongs immediately after the durable crash and media substrate
is honest because branch-delta layering is the first major physical branch
storage program that depends on those guarantees.

- `Milestone 4` can proceed in parallel because snapshots are an independent
  derived basis program.
- `Milestone 7` is expected to be built concurrently, but only on the condition
  that Milestone 5 stays confined to branch-delta storage while Milestone 7
  owns schema, lineage, cursor, and checkpoint durability semantics.
- `Milestone 6` depends on this milestone because aspect-aware layout and
  structural dedup need an already-honest branch-delta substrate.
- `Milestone 9` depends on this milestone because deterministic bulk chunking
  needs a real branch-layer model instead of copied full-state branches.
- `Milestone 10` depends on this milestone together with Milestone 4 because
  retention and compaction cannot reason honestly about pruning and rebuild
  until snapshot and branch-delta families are both explicit.
