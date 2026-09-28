# Milestone 7 Engineering Spec: Lineage, Correspondence, And Identity-Evolution Queries

> **Status:** Completed.
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-6.md](./milestone-6.md)
>
> **Adjacent milestones:** [milestone-5.4.md](./milestone-5.4.md) and
> [milestone-6.md](./milestone-6.md) are already closed and remain
> authority-distinct inputs for advisory-versus-authoritative correspondence
> honesty and basis-explicit branch/historical/diff context semantics.
>
> **Prior closeout:** [milestone-6-closeout.md](./milestone-6-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make lineage traversal, branch-local
> identity evolution, and correspondence-aware identity comparison first-class
> query artifacts so consumers can ask who became whom across current,
> branch-scoped, historical, and comparison truth without exposing raw lineage
> graph internals or silently upgrading advisory correspondence into
> authoritative continuity
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-5.4.md](./milestone-5.4.md)
> - [milestone-5.4-closeout.md](./milestone-5.4-closeout.md)
> - [milestone-6.md](./milestone-6.md)
> - [milestone-6-closeout.md](./milestone-6-closeout.md)

## Goal

Make identity evolution queryable as a first-class query surface so admitted
queries can traverse lineage, inspect replacements and splits, compare
cross-branch correspondences, and report ambiguity or rejection explicitly
without exposing raw lineage internals or forcing hosts to hand-assemble
history walks.

## Why This Milestone Exists

Milestone 5.4 made the honesty boundary between authoritative lineage and
advisory structural correspondence explicit. Milestone 6 made branch,
historical, preview-derived, and diff/comparison basis contexts explicit
query-owned artifacts. Together those milestones established the basis and
evidence vocabulary needed for identity-evolution reads, but they stopped
before identity evolution itself became a query-native capability.

That gap is now load-bearing.

Without Milestone 7:

- lineage remains an internal lower-runtime concept instead of an ordinary
  query surface
- branch comparison can say that two results differ without letting consumers
  ask whether one entity replaced, split from, or merely corresponds to
  another
- hosts will start hand-assembling identity walks from raw lineage data,
  correspondence side channels, or branch-local heuristics
- branch-local divergence will drift into ambient "same enough" identity logic
  precisely where the query layer is supposed to stay continuity-honest
- Milestone 8 view-shape work will be tempted to invent identity grouping or
  diff presentation semantics before the identity-evolution contract itself is
  frozen

Milestone 7 therefore exists to freeze:

- that lineage traversal is a query expression family, not a host utility walk
- that branch-local identity evolution and cross-basis correspondence consume
  the explicit basis/evidence artifacts already frozen in Milestones 5.4 and 6
- that authoritative continuity, advisory correspondence, ambiguity, and
  rejection are distinct result families
- that identity-evolution results stay query-shaped and replay-safe across
  admitted current, branch, historical, and comparison reads
- that unsupported lineage or correspondence classes fail typed and early
  rather than degrading into host-local graph search or continuity folklore

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "add lineage queries." It is keeping
  identity evolution honest under branch-local divergence, historical replay,
  and ambiguous correspondence pressure. The milestone must solve continuity
  honesty first.
- `arch_laws.md`: Laws 2, 5, 7, 8, 17, 21, 26, 27, 30, 33, 40, and 41 dominate
  this milestone. Identity-evolution query meaning must be planner-owned,
  result envelopes must be self-describing, advisory and authoritative
  semantics must remain structurally separate, and proof-bearing types must
  encode exactly what continuity has been proven.
- `perf_laws.md`: lineage support is only honest if traversal breadth,
  candidate breadth, ambiguity posture, branch-crossing scope, and identity
  break posture are mechanically visible. Cheap-looking identity helpers must
  not conceal broad lineage scans or correspondence rediscovery.
- `domain_laws.md`: lineage traversal descriptors, correspondence-aware
  identity comparison, branch-local divergence handling, result bundles,
  denial bundles, performance counters, and certification artifacts are
  separate responsibilities and must not collapse into one `identity.rs` god
  module.
- `worth_query_vision.md`: lineage traversal queries and correspondence queries
  are explicit product pillars. Milestone 7 is where those capabilities become
  ordinary typed query surfaces instead of lower-runtime escape hatches.
- `worth_query_roadmap.md`: Milestone 7 belongs after Milestones 5.4 and 6
  because basis-explicit comparison and advisory-versus-authoritative
  correspondence honesty are already frozen there. This milestone builds on
  those contracts; it does not reopen them.
- `test-requirements.md`: the `Lineage And Correspondence Query Parity Test`
  is the closeout proof. It requires replacement, split, branch-local
  divergence, ambiguous correspondence, and explicit rejection lanes, plus
  replay-safe parity for the same lineage basis.
- `milestone-5.4.md` and `milestone-5.4-closeout.md`: advisory structural
  correspondence, authoritative lineage continuity, ambiguity, disagreement,
  and historical-path honesty are already typed. Milestone 7 must consume that
  vocabulary instead of flattening identity evolution into one generic match
  surface.
- `milestone-6.md` and `milestone-6-closeout.md`: branch/head/historical/diff
  basis contexts are already query-owned and sealed. Milestone 7 must route
  identity-evolution reads through those basis artifacts rather than inventing
  a second basis model for lineage or correspondence.

## Adversarial Constraint

Milestone 7 must survive the following hostile condition:

> The same canonical query shape is executed against current branch truth,
> alternate branch truth, admitted historical truth, preview-derived truth,
> and admitted comparison bases while entities may have been replaced, split,
> merged, or only structurally corresponded; every admitted identity-evolution
> lane must preserve explicit continuity, advisory correspondence, ambiguity,
> branch-local divergence, and rejection semantics without exposing raw lineage
> graph internals, silently crossing branch boundaries, or upgrading advisory
> matches into authoritative identity.

Concretely, the design must remain correct when all of the following are true:

- a consumer asks lineage questions such as:
  - what did this entity used to be
  - what replaced this entity
  - what did this entity split into
  - what in branch B corresponds to this entity in branch A
- some answers are lineage-backed continuity, some are structural
  correspondence, some are ambiguous, and some must be denied
- branch-local lineage evolution exists but has not been promoted or merged
  into another basis
- the same query may be replayed later through the same basis and must emit
  the same identity-evolution meaning
- a naive implementation would be tempted to:
  - expose raw lineage nodes and ask hosts to assemble walks
  - silently follow branch-local lineage across basis boundaries
  - pick one correspondence candidate and call it continuity
  - rebuild lineage meaning from branch diffs or historical helper APIs after
    planning
  - hide unsupported lineage breadth behind broad scans or "best effort"
    identity guesses

If any supported path:

- silently upgrades advisory correspondence into authoritative continuity
- loses the distinction between replacement, split, merge, ambiguity, and
  rejection
- crosses branch or historical boundaries without explicit basis metadata
- exposes raw lineage-graph internals as the primary query contract
- broadens identity traversal after planning
- changes lineage/correspondence meaning under replay for the same lineage
  basis
- implies persisted or store-backed lineage parity before lower-store support
  exists

then Milestone 7 has failed.

## Product Decision Lock

- `worth-query` owns lineage traversal expression types, identity-evolution
  result shaping, correspondence-aware identity comparison, denial surfaces,
  diagnostics, and certification for admitted query families
- `worth-relational` remains authoritative for lineage semantics, lineage
  evidence, correspondence evidence inputs, branch-local identity evolution,
  and any future promotion of advisory correspondence into authoritative truth
- authoritative continuity and advisory correspondence remain different query
  result families even when they point at the same candidate entity
- branch-local identity evolution stays local to the declared basis unless the
  basis itself says otherwise; cross-branch continuity may not be inferred from
  structural resemblance alone
- Milestone 7 lineage traversal is query-shaped and basis-explicit; it is not
  a raw lineage graph viewer and not a convenience wrapper around internal
  lineage events
- Milestone 7 correspondence-aware identity comparison consumes explicit basis
  contexts from Milestone 6 and explicit correspondence vocabulary from
  Milestone 5.4; it must not invent a second correspondence dialect
- identity-evolution result shapes must distinguish at minimum:
  - anchor truth
  - antecedent or successor continuity
  - branch-local replacement/split/merge evolution
  - advisory correspondence candidates
  - ambiguous or rejected identity outcomes
- unsupported lineage breadth, unsupported correspondence families, and
  branch-crossing requests without explicit admitted basis pairings must fail
  typed and early
- Milestone 7 does not close store-backed lineage parity, durable lineage
  artifact reload, broad collection-wide identity discovery, or presentation-
  specific identity grouping; those belong to later milestones

Normative consequence:

- any implementation path that returns raw lineage nodes or events as the
  primary public query artifact is out of spec
- any implementation path that exposes one generic `identity_match` result
  without continuity/advisory/ambiguity/rejection classification is out of spec
- any implementation path that treats branch-local divergence as globally
  authoritative identity is out of spec
- any implementation path that lets execution rediscover traversal breadth or
  correspondence-family choice after planning is out of spec
- any implementation path that implies store-backed durable lineage parity
  before it exists is out of spec

## Compile-Time Enforcement Policy

Milestone 7 must classify which identity-evolution guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible lineage traversal artifacts that do not carry
  canonical query identity, lineage basis identity, traversal family identity,
  and explicit continuity outcome classification
- publicly constructible correspondence-aware identity results that do not
  carry both explicit basis identities and explicit advisory/authoritative/
  ambiguous/rejected outcome class
- publicly constructible identity-evolution success envelopes that expose one
  naked "best match" without uniqueness or continuity proof
- publicly constructible branch-local identity-evolution bundles that omit
  explicit branch-locality metadata
- publicly constructible result families that erase whether an entity was a
  replacement, split successor, merge successor, antecedent, or only an
  advisory correspondence candidate
- publicly constructible result bundles that encode singular continuity,
  plural successor sets, ambiguity, and denial through optional fields on one
  catch-all struct instead of distinct closed families

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `LineageTraversalDescriptor`,
  `IdentityEvolutionQueryContext`, `IdentityEvolutionResultBundle`,
  `CorrespondenceIdentityComparison`, `IdentityEvolutionMetadata`, or
  materially equivalent proof-bearing types without crate-owned lowering
- public APIs that accept raw lineage events, raw lineage graph handles, raw
  branch diff payloads, or host-authored correspondence bags as admitted
  identity-evolution query input
- public APIs that let consumers override lineage basis, branch-locality
  semantics, or correspondence outcome class after admission
- public APIs that expose bool-driven lineage or correspondence routing such as
  `follow_lineage: bool` or `use_correspondence: bool`
- public APIs that return one open-ended `IdentityResult` map whose meaning
  depends on host interpretation rather than closed typed families
- public APIs that expose generic accessors like `best_match()` or
  `successors()` on every identity-evolution result regardless of continuity
  proof or cardinality proof
- public APIs that allow branch-local identity-evolution results to be
  reclassified as cross-branch continuity without promoted or merged authority
  evidence

`Construction-time rejection`:

- unsupported lineage traversal families
- unsupported branch-crossing lineage requests
- unsupported correspondence comparison families
- ambiguous correspondence requests that demand authoritative continuity
- branch-local identity evolution requests whose declared basis does not admit
  that locality
- requests that demand singular continuity when the admitted family can only
  yield plural successor sets or ambiguity
- identity-evolution requests that require broad lineage scans beyond the
  admitted planner-owned width
- store-backed lineage/correspondence requests that remain explicit deferred
  debt

Rules:

- the strongest available boundary must be used
- lineage, correspondence, and result-bundle proof types must use sealed
  constructors and private fields
- adding a new lineage traversal family, identity-evolution outcome family, or
  correspondence family must force exhaustive compile failures across lowering,
  execution, metadata, support reporting, and certification until handled
  explicitly
- wildcard or catch-all matching over identity-evolution family in
  milestone-owned code paths is out of spec
- compile-fail coverage is required for:
  - no external construction of admitted lineage traversal artifacts
  - no raw lineage event or graph handle as query-owned result input
  - no bool-driven lineage/correspondence shortcut
  - no fabricated authoritative continuity from advisory-only evidence
  - no post-admission basis override for branch-local identity evolution
  - no catch-all result bag with optional continuity/candidate/successor fields
  - no cardinality-erasing accessor across singular, plural, ambiguous, and
    denied result families
- runtime rejection is allowed only for facts genuinely unavailable until the
  lower runtime reports lineage evidence, branch-local evolution evidence, or
  correspondence admissibility for the declared basis pair

## Scope

### In Scope

- lineage traversal query expressions over admitted entity anchors and admitted
  bases
- identity-evolution result families for antecedents, successors,
  replacements, splits, merges, and explicit identity breaks where admitted by
  lower truth
- correspondence-aware identity comparison over explicit basis pairs already
  admitted by Milestone 6
- branch-local divergence and branch-local identity evolution as explicit
  result metadata and denial-aware query surfaces
- identity-evolution metadata, diagnostics, replay bundles, and exact counters
- unified-facade composition for admitted lineage/correspondence capability
  surfaces
- milestone-native certification for lineage traversal, correspondence parity,
  ambiguity honesty, and branch-locality honesty

### Explicitly Out Of Scope

- raw lineage graph browsing or host-directed lineage event traversal
- broad collection-wide lineage discovery that would require planner-hidden
  scans
- store-backed durable lineage/correspondence parity or restart-stable lineage
  reload
- view-shape-specific identity grouping, kanban/timeline lineage projections,
  or presentation-first comparison semantics, which remain Milestone 8 work
- policy masking, tenant schema variation, and relationship-proof denial,
  which remain Milestone 9 work
- any lower-runtime promotion rule that would transform advisory
  correspondence into authoritative continuity
- mutation, merge, or writeback semantics beyond consuming the existing basis
  and correspondence artifacts from Milestones 5.4, 5.5, and 6

### Initial Admission Matrix

Milestone 7 must not leave identity-evolution behavior ambient.

Initial lineage-traversal-admitted query families:

- detail-query lineage traversal over one admitted anchor entity
- bounded successor traversal over one admitted current, branch, historical,
  or preview-derived basis
- bounded antecedent traversal over one admitted current, branch, historical,
  or preview-derived basis
- bounded branch-local evolution traversal where the declared basis already
  carries branch-local identity evolution honestly

Initial correspondence-aware identity-comparison-admitted families:

- branch-to-branch identity comparison over two already-admitted basis contexts
- current-to-historical identity comparison over two already-admitted basis
  contexts
- historical-to-historical identity comparison over two already-admitted basis
  contexts
- preview-to-authoritative identity comparison only where the preview
  provenance artifact is already admitted and explicit

Required vocabulary artifacts:

- `IdentityEvolutionQueryFamily`
- `LineageTraversalFamily`
- `IdentityEvolutionOutcomeFamily`
- `IdentityEvolutionAdmissionFailureClass`
- `IdentityEvolutionCostClass`
- `IdentityEvolutionBudgetClass`
- `IdentityEvolutionPredictionDriftOutcome`
- `IdentityEvolutionComplexityContract`
- `IdentityEvolutionComplexityStatus`

Required admitted artifacts:

- `LineageTraversalDescriptor`
- `IdentityEvolutionQueryContext`
- `AdmittedIdentityEvolutionQuery`
- `CorrespondenceIdentityComparison`
- `IdentityEvolutionComparisonBasisFamily`
- `IdentityEvolutionMetadata`
- `IdentityEvolutionResultBundle`
- `IdentityEvolutionPredictionReport`
- `IdentityEvolutionComplexityReport`
- `IdentityEvolutionSupportProfile`

Required cardinality-bearing result artifacts:

- `SingularIdentityContinuityResult`
- `PluralIdentitySuccessorSet`
- `AdvisoryIdentityCandidateSet`
- `IdentityEvolutionAmbiguityBundle`
- `IdentityEvolutionIdentityBreakBundle`
- `IdentityEvolutionDeniedBundle`

Required inspector-consumable identity artifacts:

- `InspectorIdentityArtifact`
- `InspectorIdentityClassification`
- `InspectorIdentityDigest`

Required identity-evolution outcome classes:

- `AuthoritativeAntecedent`
- `AuthoritativeSuccessor`
- `AuthoritativeReplacement`
- `AuthoritativeSplitSuccessors`
- `AuthoritativeMergeSuccessor`
- `AdvisoryCorrespondenceCandidate`
- `AmbiguousCorrespondence`
- `IdentityBreak`
- `IdentityEvolutionDenied`

Initial admitted traversal shapes:

- `DirectPredecessor`
- `DirectSuccessor`
- `DirectReplacement`
- `DirectSplitSuccessors`
- `DirectMergeSuccessor`
- `BranchLocalDirectEvolution`

Initial denied traversal shapes:

- unbounded recursive lineage walks
- host-directed path expansion
- collection-wide lineage discovery without one explicit admitted anchor
- arbitrary `max_depth` or caller-selected breadth parameters that are not part
  of one closed admitted traversal family

Required branch-locality metadata fields:

- `anchor_branch_basis_digest`
- `lineage_origin_branch_digest`
- `branch_divergence_root_digest`
- `branch_locality_class`
- `promotion_or_merge_authority_state`

Required complexity-contract fields:

- `contract_name`
- `declared_big_o`
- `measured_work_basis`
- `verified_or_debt_status`
- `forbidden_broadening_clause`

Initial denied or deferred families:

- raw lineage graph or lineage event passthrough
- global lineage discovery without an admitted anchor
- broad collection correspondence that requires planner-hidden scanning
- identity comparison families that require implicit basis discovery
- any store-backed lineage parity claim not already admitted by lower-runtime
  truth
- any family that cannot emit basis-explicit, query-shaped identity-evolution
  output

Any family not named above is out of scope for Milestone 7 and must fail typed
and early rather than becoming implied beta support.

### Initial Performance Posture Matrix

- admitted anchor-bound lineage traversal:
  lineage anchor resolution is O(1) against the admitted anchor and traversal
  breadth is O(lineage_steps_returned); broad lineage graph scans are forbidden
- admitted successor or antecedent traversal:
  traversal breadth is O(admitted_lineage_width) and must remain bounded by the
  declared traversal family
- admitted correspondence-aware identity comparison:
  candidate breadth is O(admitted_correspondence_candidates) and must stay
  planner-owned rather than executor-expanded
- branch-local identity-evolution traversal:
  branch-locality checks are O(1) per admitted branch boundary and must remain
  explicit in counters and metadata rather than inferred from result absence

Rules:

- lineage family, correspondence family, branch-locality posture, and denied
  breadth posture belong to lowered query artifacts, not executor logs
- every admitted family must carry one named complexity contract before it is
  considered an admitted family rather than an experimental helper
- planner-owned predicted lineage width and realized lineage width must remain
  distinct when the milestone emits prediction posture
- admitted Milestone 7 traversal shapes are direct and anchor-bound; any shape
  that would require recursive or caller-directed path walking is denied in
  this milestone rather than hidden behind one broad lineage API
- complexity status is part of support honesty:
  - `Verified` means the milestone has an exact counter proof row for the
    admitted family
  - `Debt` means the family may exist experimentally but cannot be implied as
    production-safe support
- result bundles and denial bundles must expose the complexity contract used
  for the lane rather than leaving cost interpretation to logs or comments
- no admitted lane may conceal broad lineage scans, raw history walks, or
  candidate rediscovery behind generic success counters
- no admitted lane may broaden from authoritative lineage traversal into
  advisory structural correspondence because one family denied

## Implementation Phases

### Phase 1: Identity-Evolution Vocabulary And Query Surface

Phase 1 exists to freeze the query-owned vocabulary before implementation
pressure collapses lineage, correspondence, and branch-local divergence into
one vague identity helper.

Milestone 7 must first implement:

- one explicit identity-evolution query family taxonomy
- one explicit lineage-traversal family taxonomy
- one explicit identity-evolution outcome-family taxonomy
- anchor-bound lineage traversal descriptors
- correspondence-aware identity-comparison descriptors that consume the
  explicit basis contexts already admitted by Milestone 6
- identity-evolution metadata shells carrying query digest, basis digest,
  lineage digest, and branch-locality posture

This phase leaves the system in a coherent state where:

- lineage traversal is a named query surface rather than a helper convention
- identity-evolution result families are closed and explicit
- later phases can lower, execute, and certify these families without
  re-arguing vocabulary

Phase exit criterion:

- no admitted identity-evolution lane depends on raw lineage internals or one
  generic match result

### Phase 2: Lineage Traversal Admission And Execution Shaping

Phase 2 exists to make lineage traversal planner-owned, basis-explicit, and
bounded before correspondence-aware comparison complicates the surface.

Milestone 7 must then implement:

- admission for bounded antecedent, successor, replacement, split, and merge
  successor traversal where lower truth admits them
- explicit branch-locality checks for branch-bound identity evolution
- one dedicated lineage execution artifact family
- basis-explicit lineage result bundles
- one named complexity contract per admitted traversal family, declared at
  lowering time rather than inferred from execution
- typed denial for unsupported lineage breadth, unsupported branch crossing,
  and missing lineage evidence
- compile-time-distinct result families for:
  - one singular continuity outcome
  - one plural successor outcome
  - one advisory candidate-set outcome
  - one ambiguity outcome
  - one denial outcome

This phase leaves the system in a coherent state where:

- lineage traversal is query-shaped and bounded
- branch-local identity evolution is explicit instead of ambient
- admitted traversal families advertise their cost surface explicitly
- later correspondence work can compose on top of already-honest lineage lanes

Phase exit criterion:

- no admitted lineage lane broadens into planner-hidden lineage graph search

### Phase 3: Correspondence-Aware Identity Comparison

Phase 3 exists to make correspondence-aware identity comparison consume the
frozen lineage and basis contracts instead of redefining them.

Milestone 7 must then implement:

- identity-comparison lowering over two already-admitted bases
- explicit outcome families for:
  - authoritative lineage continuity
  - advisory correspondence candidate
  - ambiguous correspondence
  - explicit identity break
  - typed denial
- branch-scoped comparison semantics that preserve branch-local divergence
  rather than flattening it into global identity
- explicit branch-locality metadata attachment proving whether the outcome is:
  - branch-local only
  - promoted or merged into broader continuity
  - denied from crossing the declared basis boundary
- one named complexity contract per admitted comparison family, including the
  exact candidate-discovery posture the planner is allowed to use
- typed denial for any request that demands authoritative continuity from
  advisory-only evidence

This phase leaves the system in a coherent state where:

- correspondence-aware identity comparison is query-native
- authoritative continuity and advisory correspondence remain distinct
- branch-local divergence remains visible and replay-safe
- candidate discovery cost posture is frozen before execution

Phase exit criterion:

- no admitted comparison lane depends on host tie-breaking or raw diff payloads

### Phase 4: Result Bundles, Metadata, And Unified Facade Composition

Phase 4 exists to make Milestone 7 a real daily-driver surface through the
already-closed unified facade instead of one more sidecar module.

Milestone 7 must then implement:

- identity-evolution result bundles and denial bundles
- explicit metadata attachment for lineage basis, comparison basis,
  branch-locality posture, and identity-evolution outcome family
- support truth that reports admitted traversal families, admitted comparison
  basis families, comparison complexity posture, and inspector-consumable
  identity classifications explicitly rather than only lineage traversal scope
- support-matrix and capability-admission updates for the new admitted
  lineage/correspondence families
- unified-facade exposure for admitted identity-evolution capability surfaces
- compile-time witness boundaries proving identity-evolution capability
  surfaces do not bypass admitted basis or correspondence lowering

This phase leaves the system in a coherent state where:

- Milestone 7 surfaces through the normative `worth-query` application path
- support truth and executable admission stay synchronized
- identity-evolution results are machine-checkable and facade-visible

Phase exit criterion:

- all new Milestone 7 capability surfaces are available through the unified
  facade and certified there

### Phase 5: Replay, Counter Proof, And Boundary Certification

Phase 5 exists to close the milestone through proof rather than "lineage seems
to work" demos.

Milestone 7 must finally ship:

- the `Lineage And Correspondence Query Parity Test`
- canonical rows proving:
  - replacement continuity explicitness
  - split-successor explicitness
  - branch-local divergence explicitness
  - ambiguous correspondence explicitness
  - explicit rejection behavior
  - replay-safe identity-evolution parity for the same lineage basis
- rejection rows proving:
  - unsupported lineage traversal family
  - unsupported correspondence family
  - advisory-as-authoritative forbidden
  - branch-crossing lineage without admitted basis pairing forbidden
  - broad lineage scan forbidden
  - fabricated branch-local continuity forbidden
- compile-fail or privacy hardening proving lineage and correspondence proof
  artifacts cannot be WORTHd externally

This phase leaves the system in a coherent state where:

- lineage and correspondence semantics are replay-safe and machine-checkable
- later view-shape, policy, and store-backed milestones inherit explicit
  identity-evolution contracts instead of soft helper semantics

Phase exit criterion:

- the certification suite proves lineage/correspondence parity and denial
  honesty through canonical artifacts rather than row presence alone

## Must Ship

- proof-bearing `LineageTraversalDescriptor`, `IdentityEvolutionQueryContext`,
  `AdmittedIdentityEvolutionQuery`, `CorrespondenceIdentityComparison`,
  `IdentityEvolutionMetadata`, and `IdentityEvolutionResultBundle` families or
  materially equivalent types
- compile-time-distinct `SingularIdentityContinuityResult`,
  `PluralIdentitySuccessorSet`, `AdvisoryIdentityCandidateSet`,
  `IdentityEvolutionAmbiguityBundle`, `IdentityEvolutionIdentityBreakBundle`,
  and `IdentityEvolutionDeniedBundle` families or materially equivalent
  proof-bearing types
- explicit lineage traversal query expressions for admitted antecedent,
  successor, replacement, split, and merge-successor reads
- correspondence-aware identity-comparison query expressions over two
  admitted bases
- explicit branch-local divergence and branch-local identity-evolution metadata
- identity-evolution result families that distinguish authoritative continuity,
  advisory correspondence, ambiguity, explicit identity break, and typed denial
- one dedicated identity-evolution performance subdomain owning counters and
  contract status rather than generic telemetry-only logging
- typed diagnostics, replay bundles, and exact counters for lineage traversal
  and correspondence-aware identity comparison
- explicit complexity reports embedded in admitted result bundles and denial
  bundles, not only certification harness artifacts
- one inspector-consumable identity wrapper surface that preserves identity
  classification, branch-locality posture, and replay-stable digest without
  exposing raw identity-evolution internals directly to Milestone 8 view code
- unified-facade composition and support-truth updates for admitted
  identity-evolution capability families
- milestone-native certification proving lineage parity, correspondence
  honesty, branch-locality honesty, and rejection behavior

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- proof-bearing planning and basis identity from Milestone 3 remain
  authoritative
- collection/result-shape semantics from Milestone 4 remain authoritative
- advisory-versus-authoritative correspondence honesty from Milestone 5.4
  remains authoritative
- workflow and authority-boundary honesty from Milestone 5.5 remain
  authoritative where identity-evolution results later feed workflow lanes
- branch/historical/diff basis ownership from Milestone 6 remains authoritative
- `worth-relational` remains authoritative for lineage semantics, branch-local
  identity evolution, and correspondence evidence truth
- identity-evolution reads remain query-shaped and replay-honest rather than
  raw lineage graph passthrough
- advisory correspondence must not silently become authoritative continuity
- result cardinality remains compile-time explicit instead of collapsing into
  one optional-field identity result bag
- unsupported lineage and correspondence families fail typed and early rather
  than degrading into host-local graph search or branch heuristics

## Complexity / Proof Obligations

Milestone 7 must name costs and proofs in terms of:

- declared lineage complexity contract
- declared correspondence complexity contract
- lineage anchor lookup count
- lineage step count
- predicted lineage width
- realized lineage width
- lineage path depth class
- split successor fanout width
- branch-local boundary check count
- branch-local divergence count
- promotion-or-merge authority proof check count
- correspondence candidate count
- ambiguous correspondence count
- identity-break count
- lineage_to_correspondence_fallback_count
- advisory-as-authoritative denial count
- unsupported lineage denial count
- unsupported correspondence denial count
- broad lineage scan denial count
- identity-evolution replay parity count
- executor rediscovery avoidance on identity-evolution lanes

Minimum named complexity contracts:

- `direct_predecessor_traversal`
  - declared Big-O: `O(anchor_lookup + lineage_steps_returned)`
  - forbidden broadening clause: no recursive ancestry walk beyond the direct
    admitted family
- `direct_successor_traversal`
  - declared Big-O: `O(anchor_lookup + lineage_steps_returned + split_fanout)`
  - forbidden broadening clause: no descendant expansion beyond the direct
    admitted successor family
- `branch_local_direct_evolution`
  - declared Big-O: `O(anchor_lookup + branch_boundary_checks + lineage_steps_returned)`
  - forbidden broadening clause: no cross-branch continuity discovery without
    explicit promoted/merged authority evidence
- `correspondence_identity_comparison`
  - declared Big-O: `O(basis_binding + admitted_correspondence_candidates)`
  - forbidden broadening clause: no planner-hidden candidate discovery beyond
    the admitted comparison family

Minimum required counters:

- `declared_lineage_complexity_contract_count`
- `declared_correspondence_complexity_contract_count`
- `lineage_anchor_lookup_count`
- `lineage_step_count`
- `predicted_lineage_width`
- `realized_lineage_width`
- `lineage_width_drift_count`
- `lineage_path_depth_class_count`
- `split_successor_fanout_width`
- `branch_local_boundary_check_count`
- `branch_local_divergence_count`
- `promotion_or_merge_authority_proof_check_count`
- `branch_crossing_denial_count`
- `correspondence_candidate_count`
- `ambiguous_correspondence_count`
- `identity_break_count`
- `lineage_to_correspondence_fallback_count`
- `advisory_as_authoritative_denial_count`
- `unsupported_lineage_denial_count`
- `unsupported_correspondence_denial_count`
- `broad_lineage_scan_denial_count`
- `identity_evolution_metadata_attachment_count`
- `identity_evolution_replay_parity_count`
- `identity_evolution_executor_rediscovery_count`
- `identity_evolution_basis_rediscovery_count`
- `complexity_contract_violation_denial_count`
- `complexity_status_debt_count`

Rules:

- counters belong to admitted result bundles, denial bundles, and
  certification bundles
- representative certification scenarios must assert exact counts
- every admitted lane must emit exactly one lineage or correspondence
  complexity contract, and that contract must be included in the result or
  denial bundle
- `identity_evolution_executor_rediscovery_count` must be exactly zero on every
  admitted lane
- `identity_evolution_basis_rediscovery_count` must be exactly zero on every
  admitted lane
- `declared_lineage_complexity_contract_count` or
  `declared_correspondence_complexity_contract_count` must be exactly one per
  admitted lane, never zero and never many
- every lane whose complexity posture remains unproven must increment
  `complexity_status_debt_count` and may not be implied as fully admitted
- every denied attempt to exceed the declared complexity posture must increment
  `complexity_contract_violation_denial_count`
- every denied unsupported lineage request must increment
  `unsupported_lineage_denial_count`
- every denied unsupported correspondence request must increment
  `unsupported_correspondence_denial_count`
- every denied advisory-as-authoritative request must increment
  `advisory_as_authoritative_denial_count`
- every denied broad lineage scan attempt must increment
  `broad_lineage_scan_denial_count`
- every admitted branch-local lane must make promotion-or-merge authority proof
  checks mechanically visible through
  `promotion_or_merge_authority_proof_check_count`
- `lineage_to_correspondence_fallback_count` must be exactly zero on every
  admitted lineage-authoritative lane
- every admitted branch-local lane must make boundary checks and divergence
  posture mechanically visible rather than implicit
- predicted versus realized lineage width must remain explicit where the
  milestone emits prediction posture rather than one blended count
- no admitted lane may hide broad lineage walking or candidate rediscovery
  inside generic success counters
- no admitted lane may silently fall back from denied authoritative lineage
  traversal into advisory structural candidate discovery without the query
  family itself changing explicitly
- elapsed time alone is not acceptable evidence for any Milestone 7
  performance claim; proof must be expressed in structural work counters

Minimum certification rows should include:

- `replacement-continuity-explicitness`
- `split-successor-explicitness`
- `branch-local-divergence-explicitness`
- `ambiguous-correspondence-explicitness`
- `identity-break-explicitness`
- `lineage-versus-structural-disagreement-explicitness`
- `lineage-replay-parity`
- `identity-evolution-width-drift-explicitness`
- `lineage-complexity-contract-parity`
- `correspondence-complexity-contract-parity`
- `complexity-status-honesty`

Minimum rejection rows should include:

- `unsupported-lineage-traversal-family`
- `unsupported-correspondence-family`
- `advisory-as-authoritative-forbidden`
- `lineage-to-correspondence-fallback-forbidden`
- `branch-crossing-lineage-forbidden`
- `broad-lineage-scan-forbidden`
- `fabricated-branch-local-continuity-forbidden`
- `complexity-contract-violation-denied`

## Allowed Debt

- unsupported lineage or correspondence classes may remain explicit `Debt`
  while admitted classes are fully typed and parity-proven
- broader collection-wide identity-evolution discovery may remain explicit
  `Debt`
- restart-stable lineage/correspondence parity may remain explicit `Debt`
  until `worth-store` can support it honestly
- silent continuity through ambiguous correspondence may not exist as debt
- hidden branch crossing or hidden lineage scan broadening may not exist as
  debt

## Acceptance Evidence

Milestone 7 is complete only when `worth-query` can prove:

- the `Lineage And Correspondence Query Parity Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- lineage traversal yields typed, explainable results across admitted identity
  evolution classes
- correspondence-aware identity comparison stays explicit about ambiguity,
  branch-local divergence, and rejection
- explicit identity break remains distinct from generic denial in public result
  families, replay artifacts, and certification evidence
- Milestone 8-facing inspector consumption can preserve identity
  classification without flattening continuity, advisory, ambiguity, identity
  break, or denial into one convenience surface
- replay preserves lineage/correspondence meaning for the same lineage basis
- unsupported lineage or correspondence classes fail typed and early

Required verification output must include:

- `query_digest`
- `basis_digest`
- `lineage_digest`
- `branch_locality_digest`
- `complexity_contract_digest`
- `result_digest`
- `failure_digest`
- `replay_digest`
- `counter_snapshot`

### Representative Scenario Matrix

Milestone 7 must prove the architecture against concrete lanes, not just
capability names.

Minimum representative scenarios:

- `replacement-lineage-traversal`
  - one admitted anchor entity yields one authoritative replacement successor
  - `query_digest` remains equal across replay while `result_digest` and
    `lineage_digest` stay parity-safe
- `split-successor-lineage-traversal`
  - one admitted anchor entity yields multiple authoritative split successors
    through one explicit split-successor result family rather than host
    post-processing
- `branch-local-divergence-stays-local`
  - one branch-local identity evolution lane remains basis-explicit and does
    not silently present as cross-branch global continuity
- `branch-to-branch-correspondence-ambiguous`
  - one branch-to-branch identity comparison produces explicit ambiguity rather
    than one convenience winner
- `lineage-versus-structural-disagreement-explicit`
  - one lane proves an explicit disagreement where lineage says identity break
    or continuity denial while structure still suggests one advisory candidate
  - the result must preserve disagreement rather than collapsing to whichever
    signal is easier for the host
- `preview-to-authoritative-identity-comparison`
  - one preview-derived basis compares to one authoritative basis through one
  admitted explicit correspondence family
- `identity-break-explicit`
  - one lane produces an explicit identity break rather than silent absence
- `identity-aware-inspector-consumption-parity`
  - one admitted identity-evolution artifact lowers into the inspector-facing
    wrapper surface without changing identity classification or
    branch-locality class
- `advisory-as-authoritative-forbidden`
  - one hostile lane tries to demand continuity from advisory-only evidence and
  fails typed and early
- `broad-lineage-scan-forbidden`
  - one hostile lane attempts broad lineage discovery outside the admitted
    anchor-bound width and is denied before rich result shaping
- `complexity-contract-violation-denied`
  - one hostile lane attempts a traversal or comparison posture outside the
    declared contract and fails through one explicit contract-violation denial
- `fabricated-branch-local-continuity-forbidden`
  - one hostile lane tries to synthesize cross-branch continuity from
    branch-local evidence and fails typed
- `lineage-replay-preserves-classification`
  - one replayed lane preserves not only the identity result but the exact
    continuity-versus-advisory-versus-denied classification for the same
    lineage basis

## Architectural Notes

### Lineage Must Stay Query-Shaped

The easiest way to fake lineage support is to expose raw lineage internals and
call that "queryability."

Milestone 7 must instead require:

- one anchor-bound lineage query artifact
- one bounded traversal family
- one typed identity-evolution result bundle

If hosts still have to assemble lineage walks manually, the milestone is not
done.

If an admitted query family can widen from "direct predecessor" to "walk until
you find something useful" without the family or complexity contract changing,
the milestone is also not done.

### Advisory Must Stay Advisory

Milestone 5.4 already froze the rule:

- lineage continuity is authoritative
- structural correspondence is advisory unless lower truth says otherwise

Milestone 7 must preserve that exact boundary even when correspondence-aware
comparison is now part of ordinary query usage.

If an ambiguous or advisory candidate can still look like authoritative
continuity in the public query surface, this milestone has failed.

### Branch-Local Identity Must Not Leak

Branch-local divergence is part of the product story, not a corner case.

The required rule is:

- branch-local identity evolution may be queried inside its declared basis
- cross-branch identity continuity must remain explicit and basis-owned
- no host or helper may infer global continuity from branch-local evidence
- no admitted lane may change complexity class just because branch-local
  evidence was insufficient; broader discovery must deny or require one new
  explicit query family

That is what keeps branch comparison honest instead of "close enough."

### This Milestone Must Not Steal 8, 9, Or 10

Milestone 7 owns:

- lineage traversal
- correspondence-aware identity comparison
- branch-local identity-evolution honesty
- replay-safe identity result bundles

It does not own:

- presentation/view semantics for identity grouping, which remain Milestone 8
- policy and tenant boundaries, which remain Milestone 9
- store-backed lineage parity and durable lineage artifact reload, which remain
  later store-gated work

Milestone 7 must therefore stop at:

- query surface
- admission
- execution/result shaping
- metadata
- certification

## Sequencing Notes

Milestone 7 belongs immediately after Milestone 6 because identity evolution
becomes product-meaningful only once branch, historical, preview-derived, and
comparison basis contexts are already sealed and query-owned.

It also belongs after Milestone 5.4 because advisory-versus-authoritative
correspondence honesty, ambiguity, and historical-path honesty are already
frozen there and should be consumed rather than redefined.

It must land before Milestone 8 because view-shape or presentation semantics
for branch comparison and identity grouping should consume explicit
identity-evolution artifacts rather than invent them in the presentation layer.

## Parallelization Notes

Once the identity-evolution vocabulary and admitted result families are frozen:

- early Milestone 8 scope/template/view work can proceed in parallel against
  explicit identity-evolution artifacts
- counter hardening and compile-time tightening can proceed in parallel without
  changing milestone semantics
- later store-backed parity work can target the same lineage/correspondence
  contracts without changing runtime-backed Milestone 7 meaning

## Store Dependency

- Core lineage traversal, branch-local identity-evolution, and
  correspondence-aware identity comparison semantics are not blocked on
  `worth-store`.
- Restart-stable lineage/correspondence parity across persisted history,
  durable lineage artifact reload, and store-backed lineage execution parity
  remain blocked on `worth-store` durable lineage support and should stay
  explicit completion debt until that support exists.

## Explicit Failure Taxonomy For Milestone 7

- unsupported lineage traversal family
- unsupported correspondence family
- missing lineage anchor
- branch-crossing lineage without admitted basis pairing
- advisory-as-authoritative request
- ambiguous correspondence not surfaced
- explicit identity break erased
- branch-local divergence erased
- broad lineage scan required
- identity-evolution replay divergence
- identity-evolution artifact invariant break

## Anti-Patterns Explicitly Rejected

- raw lineage graph or event browsing exposed as the primary public query
  surface
- one generic `identity_match` result that erases continuity, ambiguity, and
  rejection semantics
- host-side lineage walking or tie-breaking for admitted query families
- branch-local evidence silently presented as cross-branch continuity
- bool-driven lineage or correspondence routing
- broad lineage discovery hidden behind admitted anchor-bound queries
- public construction of lineage or correspondence proof types without the
  proving path

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes how consumers ask "who became whom" across
branch, history, and comparison truth without exposing raw internals or
flattening advisory correspondence into continuity.

The adversarial constraint is load-bearing because it forbids the naive
failure mode where lineage and correspondence appear to work only because hosts
perform graph walks, pick winners, and smooth over branch-local divergence
after execution.

The milestone preserves authority boundaries because `worth-query` owns
lineage/correspondence query expression, result shaping, denials, metadata,
and certification while `worth-relational` remains the authority for lineage
semantics and identity-evolution truth.

The milestone defines proof obligations rather than implementation chores
because lineage parity, ambiguity honesty, branch-locality honesty, replay
parity, and exact counters are all required for closeout.

A competent engineer should be able to map this spec into honest lineage,
identity-comparison, result-bundle, certification, and compile-fail modules
without inventing the architecture during implementation.

This milestone belongs at 7 because it is the identity-evolution contract layer
that must exist before view semantics, policy boundaries, and store-backed
durability can compose honestly.

## Closeout Standard

Milestone 7 is complete only when all of the following are true:

- admitted query families can traverse lineage through explicit, bounded,
  basis-owned query artifacts
- correspondence-aware identity comparison remains explicit about advisory
  status, ambiguity, branch-local divergence, and rejection
- authoritative continuity and advisory correspondence remain distinct in the
  public result surface
- unsupported lineage breadth, unsupported correspondence families, and
  branch-crossing dishonesty fail typed and early
- replay-safe certification bundles prove lineage/correspondence parity and
  denial honesty through canonical machine-checkable artifacts

If code lands but lineage still depends on raw graph access, advisory
correspondence can still masquerade as continuity, branch-local divergence can
still leak across bases, or broad lineage scans still hide behind admitted
queries, Milestone 7 is not complete.
