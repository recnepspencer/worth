# Milestone 5.4 Engineering Spec: Structural Correspondence And Historical Evaluation Contracts

> **Status:** Closed on 2026-04-17 for the runtime-backed structural correspondence and historical materialization-path honesty scope
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-5.3.md](./milestone-5.3.md)
>
> **Adjacent milestone:** [milestone-5.2.md](./milestone-5.2.md) remains authority-distinct; preview-session basis identity and promotion parity are already closed there and must compose with, not redefine, this milestone's correspondence and historical-path semantics.
>
> **Adjacent hardening milestones:** [milestone-5.1.md](./milestone-5.1.md) and [milestone-5.3.md](./milestone-5.3.md)
>
> **Prior closeout:** [milestone-5.3-closeout.md](./milestone-5.3-closeout.md)
>
> **Shipped closeout:** [milestone-5.4-closeout.md](./milestone-5.4-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make advisory structural correspondence and historical materialization-path honesty first-class query artifacts so branch/history/identity-oriented reads can expose how equivalence and historical truth were actually established instead of collapsing ambiguity, disagreement, or reconstruction method into one vague comparison surface
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [domain_laws.md](../../docs/coding-guidelines/domain_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-5.2.md](./milestone-5.2.md)
> - [milestone-5.2-closeout.md](./milestone-5.2-closeout.md)
> - [milestone-5.3.md](./milestone-5.3.md)
> - [milestone-5.3-closeout.md](./milestone-5.3-closeout.md)

## Goal

Strengthen the branch/history/identity story so structural correspondence and
historical materialization-path honesty become explicit query artifacts instead
of implied bridge details, host heuristics, or lower-runtime behavior that
disappears by the time results reach consumers.

## Why This Milestone Exists

Milestone 5 proved that query meaning can survive time under live maintenance.
Milestone 5.1 made locality-bearing live narrowing and stream-contract delivery
explicit. Milestone 5.2 made preview-session basis identity and promotion-parity
comparison explicit. Milestone 5.3 made frontier posture and deterministic
parallel admission explicit.

Those milestones solved different ways query meaning could drift under churn,
workflow, and cost posture. They did not yet solve two closely related branch
and history honesty problems:

- when a query says two things "correspond," is that authoritative lineage or
  only advisory structural resemblance?
- when a query says it returned historical truth, was that truth served from a
  retained snapshot, delta replay, or full reconstruction?

If `worth-query` does not own those distinctions explicitly:

- structural-fingerprint matches will be mistaken for continuity
- lineage-backed and structure-backed correspondences will blur into one
  ambiguous "compare" result
- historical reads will look identical even when they were produced through
  materially different lower-runtime paths
- unsupported history or correspondence lanes will degrade into ambient host
  fallback instead of typed denial
- Milestones 6 and 7 will inherit soft semantics precisely where branch,
  history, and identity become the product

Milestone 5.4 therefore exists to freeze:

- that lineage-backed continuity and structural correspondence are distinct
  query concepts with different semantic authority
- that ambiguity, disagreement, and advisory status are part of the result
  contract rather than diagnostic side notes
- that historical query results carry explicit materialization-path identity
- that path compatibility and historical admission remain lower-runtime
  authority but query-visible
- that later historical, diff, and lineage work builds on explicit basis and
  path contracts rather than reinventing them

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "add correspondence" or "add history
  metadata." It is preserving truth about sameness and truth about historical
  materialization under ambiguity, disagreement, and lower-runtime variation.
  The milestone must solve those honesty boundaries first.
- `arch_laws.md`: Laws 7, 8, 17, 21, 26, 27, 30, 33, 40, and 41 dominate this
  milestone. Correspondence and historical-path semantics must be lowered
  before execution, result envelopes must be self-describing, authority and
  derivation must stay distinct, and proof-bearing types must encode exactly
  what has been established.
- `perf_laws.md`: historical-path and correspondence claims are only honest if
  the query layer makes cost and derivation mode visible. Snapshot reuse, delta
  replay, reconstruction, candidate breadth, and ambiguity denial must be
  mechanically visible rather than hidden behind one generic "comparison"
  surface.
- `domain_laws.md`: correspondence descriptors, historical-path metadata,
  compatibility admission, ambiguity diagnostics, and certification artifacts
  are separate responsibilities and must not collapse into one generic
  "history" or "comparison" module.
- `worth_query_vision.md`: branch-scoped reads, time-travel, diff, lineage, and
  correspondence all assume query-visible basis honesty. Structural
  correspondence and historical materialization-path metadata are the missing
  contract layer that keeps those future features precise.
- `worth_query_roadmap.md`: Milestone 5.4 exists specifically to prove
  structural correspondence and historical materialization-path metadata remain
  explicit and ambiguity-honest before Milestones 6 and 7 broaden branch,
  historical, and lineage surfaces.
- `test-requirements.md`: the `Structural Correspondence And Historical
  Materialization Path Test` is the closeout proof. It requires lineage-backed,
  structural-fingerprint-backed, ambiguous, and historical-path-explicit lanes
  with typed denials for unsupported or dishonest cases.
- `milestone-5.2.md` and `milestone-5.2-closeout.md`: preview basis and
  promotion comparison are already query-native. Milestone 5.4 must reuse that
  basis-explicit discipline for advisory correspondence and historical-path
  identity instead of inventing ambient comparison shortcuts.
- `milestone-5.3.md` and `milestone-5.3-closeout.md`: frontier posture and
  route honesty are already planner-owned. Milestone 5.4 must consume those
  cost and route boundaries when correspondence or historical evaluation rides
  through admitted routes, not replace them with one opaque history lane.

## Adversarial Constraint

Milestone 5.4 must survive the following hostile condition:

> The same canonical query shape is evaluated across branch/history/identity
> surfaces where some correspondences are lineage-backed, some are only
> structural-fingerprint-backed, some are ambiguous or contradictory, and some
> historical reads can be served through retained snapshots, delta replay, or
> full reconstruction; the query layer must preserve explicit advisory versus
> authoritative semantics and explicit materialization-path identity without
> silently collapsing ambiguity, disagreement, or reconstruction method into
> one generic comparison result.

Concretely, the design must remain correct when all of the following are true:

- the same canonical query shape is paired with:
  - authoritative lineage-backed continuity evidence
  - structural-fingerprint-backed advisory correspondence evidence
  - both kinds of evidence at once with disagreement or ambiguity
- structural candidates may be multiple, partial, or tie-ranked rather than
  uniquely resolvable
- the lower runtimes can serve one admitted historical request through:
  - retained snapshot materialization
  - delta replay materialization
  - full reconstruction materialization where explicitly admitted
- historical-path choice may differ even for otherwise similar query shapes
  because the active retained state, replay tail, or lower-runtime capability
  differs
- preview basis, live basis, or frontier posture may already be explicit from
  earlier milestones and must remain preserved rather than erased by
  correspondence/history wrappers
- a naive implementation would be tempted to:
  - report all correspondences as if they were continuity
  - pick one structural candidate silently
  - substitute one historical path for another without saying so
  - hide unsupported paths behind broad fallback or host cache reuse

If any supported path:

- silently upgrades structural correspondence into authoritative continuity
- hides lineage-versus-structural disagreement inside one success result
- returns historical results without exposing how they were materialized
- substitutes one historical path for another without explicit result metadata
  or typed denial
- makes hosts or caches decide whether a historical path was "close enough"
- loses basis identity, route identity, or result-family identity when wrapping
  correspondence or historical metadata

then Milestone 5.4 has failed.

## Product Decision Lock

- lineage remains authoritative continuity and must stay distinct from all
  advisory correspondence surfaces
- structural correspondence is a first-class query artifact, but it is advisory
  unless lower-truth semantics explicitly promote it later
- ambiguity is not an implementation detail; it is part of the query-visible
  contract
- lineage-backed and structural-fingerprint-backed evidence may coexist, and
  disagreement between them must remain explicit
- historical evaluation authority remains in lower runtimes; `worth-query`
  carries admission, compatibility, and materialization-path metadata but does
  not invent historical truth from host caches
- materialization path is result metadata, not a logging detail
- retained snapshot, delta replay, and full reconstruction are distinct
  historical-path classes with distinct compatibility and admission outcomes
- unsupported correspondence families, unsupported historical paths, and
  ambiguous cases must fail typed and early rather than widening into a vague
  "best effort" lane
- Milestone 5.4 does not close full historical query semantics, diff query
  semantics, or lineage traversal semantics; it freezes the correspondence and
  materialization-path contracts those later milestones depend on

Normative consequence:

- any implementation path that exposes "corresponding entity" without
  disclosing whether that claim came from lineage or structure is out of spec
- any implementation path that silently chooses one structural candidate and
  hides ambiguity is out of spec
- any implementation path that reports one historical result without exposing
  retained-snapshot versus replay versus reconstruction identity is out of spec
- any implementation path that treats host cache reuse as historical
  materialization authority is out of spec
- any implementation path that implies structural correspondence is continuity
  by default is out of spec

## Compile-Time Enforcement Policy

Milestone 5.4 must classify which correspondence and historical-path
guarantees become unrepresentable, uncompilable, or construction-time
rejection.

`Unrepresentable` in public types:

- publicly constructible correspondence artifacts that do not carry query
  identity, basis identity, correspondence family identity, and advisory versus
  authoritative classification
- publicly constructible correspondence success artifacts that expose one
  "best match" without also carrying a proved uniqueness or ambiguity state
- publicly constructible historical evaluation envelopes that do not carry
  explicit materialization-path identity and historical compatibility outcome
- publicly constructible historical evaluation envelopes that carry only one
  materialization-path field instead of distinct requested, admitted, and
  resolved path classes
- publicly constructible ambiguity or disagreement outcomes encoded as naked
  booleans, string flags, or optional bags instead of closed families
- publicly constructible historical-path policies as free-form config bags
  rather than closed admitted path classes

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `StructuralCorrespondenceDescriptor`,
  `LineageCorrespondenceDescriptor`, `CorrespondenceAmbiguityEnvelope`,
  `HistoricalMaterializationPathMetadata`, `HistoricalEvaluationAdmission`, or
  materially equivalent proof-bearing types without crate-owned lowering
- public APIs that accept raw lineage events, raw structural fingerprint bags,
  raw cache hits, or host-authored historical path claims as though they were
  admitted query artifacts
- public APIs that let consumers fabricate advisory-versus-authoritative
  outcomes or historical-path compatibility outcomes after planning
- public APIs that return naked correspondence matches or naked historical
  payloads with the honesty metadata stripped away
- public conversion paths that bypass admitted query/basis planning and mint
  correspondence or historical-path artifacts directly from host data

`Construction-time rejection`:

- unsupported correspondence families
- unsupported structural fingerprint classes
- unsupported lineage-plus-structural disagreement handling requests
- unsupported historical materialization-path requests
- invalid historical compatibility pairings between requested basis and lower
  runtime capability
- invalid attempts to request authoritative continuity from structural
  correspondence alone
- invalid attempts to collapse structural candidate sets into one unique
  success without uniqueness proof
- invalid requests that erase materialization-path identity from admitted
  historical result families

Rules:

- the strongest available boundary must be used
- correspondence and historical-path proof types must use sealed constructors
  and private fields
- compile-fail coverage is required for:
  - no fabricated advisory-as-authoritative promotion
  - no raw structural-fingerprint bag as correspondence proof
  - no hidden historical-path erasure
  - no naked best-match accessor without uniqueness proof
  - no naked historical payload accessor without path metadata
  - no external construction of ambiguity or compatibility outcomes
- runtime rejection is allowed only for facts genuinely unavailable until the
  lower runtime reports lineage evidence, structural candidates, or historical
  materialization capability for the requested basis

## Scope

### In Scope

- lineage-backed correspondence descriptors as explicit query-visible artifacts
- structural-fingerprint-backed correspondence descriptors as first-class
  advisory query artifacts
- typed ambiguity and disagreement outcomes for correspondence resolution
- historical evaluation admission and compatibility artifacts for admitted
  runtime-backed historical requests
- explicit historical materialization-path metadata on admitted result
  envelopes
- typed diagnostics, counters, replay bundles, and rejection surfaces for
  correspondence and historical-path semantics
- milestone-native certification for advisory-versus-authoritative honesty,
  ambiguity explicitness, and historical materialization-path visibility

### Explicitly Out Of Scope

- full branch-head and time-travel query-family expansion beyond the admitted
  historical-path metadata surface; that remains Milestone 6 work
- full lineage traversal query semantics and richer correspondence exploration;
  those remain Milestone 7 work
- policy masking, tenant schema variation, or branch access semantics; those
  remain later milestones
- durable historical restore, restart-stable historical replay, or store-backed
  historical parity
- mutation, merge, or writeback workflow lowering
- host cache ownership, external persistence of correspondence artifacts, or
  transport framing

### Initial Admission Matrix

Milestone 5.4 must not leave correspondence or historical-path semantics
ambient.

Initial correspondence-admitted families:

- lineage-backed correspondence where lower-runtime identity continuity is
  already authoritative
- structural-fingerprint-backed correspondence where lower-runtime structural
  evidence is exposed through one explicit advisory artifact
- mixed-evidence correspondence where both lineage and structural evidence are
  present and disagreement or ambiguity can be reported explicitly

Required admitted correspondence outcome classes:

- `LineageContinuity`
- `AdvisoryStructuralUnique`
- `AdvisoryStructuralAmbiguous`
- `LineageStructuralDisagreement`
- `CorrespondenceDenied`

Initial correspondence-denied families:

- any request that attempts to treat structural correspondence as
  authoritative continuity without lower-runtime proof
- any request that requires host-authored fingerprinting or client-local tie
  breaking
- any request that requires `Option<BestMatch>`, implicit uniqueness, or a
  convenience "winner" API without a proved uniqueness state
- any family that cannot expose ambiguity, disagreement, or candidate breadth
  explicitly

Initial historical-path-admitted classes:

- `RequestedRetainedSnapshotPath`
- `RequestedDeltaReplayPath`
- `RequestedFullReconstructionPath`
- retained snapshot path where the lower runtime already has retained
  historical materialization for the requested basis
- delta replay path where the lower runtime can honestly replay from an
  admitted retained or resolved basis into the requested historical result
- full reconstruction path only where the lower runtime admits it explicitly
  and the result envelope can carry that path identity honestly

Required historical path state classes:

- `RequestedHistoricalPathClass`
- `AdmittedHistoricalPathClass`
- `ResolvedHistoricalPathClass`
- `HistoricalPathSubstitutionDenied`

Initial historical-path denials:

- hidden substitution of one path for another
- host-cache-backed reconstruction masquerading as lower-runtime historical
  evaluation
- any admitted result family that cannot carry materialization-path metadata
  end-to-end

Any family not named above is out of scope for Milestone 5.4 and must fail
typed and early rather than becoming implicit beta support.

### Initial Performance Posture Matrix

- lineage-backed correspondence:
  candidate breadth is bounded by admitted lineage evidence and explicit
  continuity classification
- structural-fingerprint-backed correspondence:
  candidate breadth and ambiguity posture must be explicit before execution
- retained snapshot historical evaluation:
  path identity and admission are explicit up front; result envelopes carry the
  retained-snapshot class
- delta replay historical evaluation:
  replay-path admission and replay breadth remain explicit rather than hidden
  in generic historical execution
- full reconstruction historical evaluation:
  admitted only when explicitly lowered and always visible as a reconstruction
  path rather than a generic historical success lane

Rules:

- correspondence family, candidate breadth posture, and ambiguity policy belong
  to lowered query artifacts, not to executor logs
- historical materialization-path identity belongs to result envelopes and
  certification bundles, not to debug text
- any path-compatibility denial must be explicit and counted

### Planner-Owned Performance Artifacts

Milestone 5.4 should encode performance in the plan, not only in realized
counter snapshots.

Required planner-owned performance artifacts:

- `CorrespondenceCostPosture`
- `HistoricalPathCostPosture`
- `StructuralCandidateDiscoveryPlan`
- `StructuralCandidateBudget`
- `StructuralCandidateOrderingContract`
- `HistoricalReplaySpanBudget`
- `HistoricalReconstructionBudget`
- `HistoricalPathReuseDescriptor`
- `RetainedStateReuseEligibility`
- `ReplayTailReuseEligibility`
- `PerformancePredictionDriftOutcome`

Required admitted correspondence cost posture classes:

- `LineageDirect`
- `StructuralCandidateBounded`
- `StructuralAmbiguityBounded`
- `CorrespondenceDeniedByBreadth`

Required admitted historical-path cost posture classes:

- `HistoricalRetainedFastPath`
- `HistoricalReplayBounded`
- `HistoricalReconstructionExpensive`
- `HistoricalPathDeniedByBudget`

Required structural discovery classes:

- `IndexBackedBounded`
- `FingerprintBucketBounded`
- `RequiresBroadScanDenied`

Rules:

- every admitted correspondence lane must lower with one explicit
  `CorrespondenceCostPosture`
- every admitted historical lane must lower with one explicit
  `HistoricalPathCostPosture`
- every structural lane must lower with one explicit candidate discovery plan
  and one explicit candidate budget before execution begins
- every historical lane must lower with explicit replay or reconstruction
  budgets before execution begins
- execution may realize different exact counts, but it may not change cost
  posture class without emitting one typed `PerformancePredictionDriftOutcome`
- the executor may not decide to inspect more structural candidates, widen into
  broad scan, or choose replay versus reconstruction on its own once the plan
  is lowered
- path reuse must be planner-owned; it may not appear only as a retrospective
  "work avoided" anecdote

## Structural Correspondence And Historical Evaluation Architecture

### One History/Identity Honesty Boundary

Milestone 5.4 extends the existing proof chain. It must not create one generic
"history compare" runtime beside the planned query substrate.

The authoritative flow becomes:

`ValidatedQueryBundle`
-> `ExecutionPlanBundle` or admitted later-basis request
-> `CorrespondenceEvaluationRequest`
-> `CorrespondenceEvidenceResolved`
-> `HistoricalPathRequested`
-> `HistoricalPathAdmitted`
-> `HistoricalPathResolved`
-> `CorrespondenceHistoricalEnvelope`
-> `CorrespondenceHistoricalParityBundle`

Correspondence and historical-path artifacts therefore consume already-proven
query meaning and basis meaning. They do not re-author:

- query semantics
- result-family semantics
- preview basis semantics
- live basis semantics
- frontier route posture

Mandatory proof-chain rule:

- each phase output type is the only admitted input type of the next phase
- no public API may lower directly from `CorrespondenceEvaluationRequest` to
  `HistoricalPathResolved` or to a consumer-facing envelope
- no public API may shape a success envelope unless both correspondence
  resolution and historical path resolution have completed through sealed
  proving functions
- route/basis metadata inherited from earlier milestones must be carried
  forward structurally rather than copied into optional diagnostics fields

### Authority Boundaries

`worth-query` owns:

- correspondence descriptor families
- advisory-versus-authoritative classification surfaces
- ambiguity and disagreement result families
- historical evaluation admission wrappers
- historical materialization-path result metadata
- diagnostics, counters, replay bundles, and certification artifacts

Lower runtimes own:

- authoritative lineage continuity semantics
- structural evidence generation where admitted
- historical materialization capability and actual path selection
- retained snapshot, replay, and reconstruction authority

Hosts and delivery glue may own:

- display of correspondence and historical-path metadata
- transport of already-lowered result envelopes

Hosts and delivery glue may not own:

- deciding continuity versus advisory correspondence
- collapsing ambiguity into one chosen candidate
- deciding historical path compatibility
- substituting one historical materialization path for another silently

### Representative Artifact Families

- `LineageCorrespondenceDescriptor`
- `StructuralCorrespondenceDescriptor`
- `CorrespondenceCandidateSet`
- `CorrespondenceEvidenceResolved`
- `CorrespondenceOutcome`
- `UniqueStructuralCorrespondenceWitness`
- `CorrespondenceCostPosture`
- `StructuralCandidateDiscoveryPlan`
- `StructuralCandidateBudget`
- `StructuralCandidateOrderingContract`
- `CorrespondenceAmbiguityEnvelope`
- `CorrespondenceDisagreementEnvelope`
- `HistoricalEvaluationRequest`
- `HistoricalPathRequested`
- `HistoricalEvaluationAdmission`
- `HistoricalPathAdmitted`
- `HistoricalMaterializationPathMetadata`
- `HistoricalPathResolved`
- `HistoricalPathCostPosture`
- `HistoricalReplaySpanBudget`
- `HistoricalReconstructionBudget`
- `HistoricalPathReuseDescriptor`
- `RetainedStateReuseEligibility`
- `ReplayTailReuseEligibility`
- `PerformancePredictionDriftOutcome`
- `HistoricalPathCompatibilityOutcome`
- `MetadataPreservingHistoricalResultView`
- `CorrespondenceHistoricalEnvelope`
- `CorrespondenceHistoricalParityBundle`
- `HistoricalEvaluationCounterSnapshot`
- `CorrespondenceComplexityContract`
- `HistoricalPathComplexityContract`

## Phase Plan

### Phase 1: Freeze Correspondence And Historical-Path Vocabulary

Phase 1 exists to keep Milestone 5.4 from collapsing multiple truths into one
bag-shaped comparison story.

Milestone 5.4 must first implement:

- closed correspondence family vocabulary distinguishing lineage-backed,
  structural-fingerprint-backed, mixed-evidence, ambiguity, and disagreement
  outcomes
- closed historical materialization-path vocabulary distinguishing retained
  snapshot, delta replay, full reconstruction, and typed denial
- explicit advisory-versus-authoritative result classification
- explicit historical compatibility/admission outcomes
- dedicated performance/diagnostics subdomains for correspondence and
  historical-path counters rather than generic comparison telemetry

This phase leaves the system in a coherent state where:

- correspondence can no longer masquerade as continuity accidentally
- historical-path identity can no longer disappear in success envelopes
- later phases have one honest vocabulary to lower into

Phase outputs:

- `CorrespondenceOutcome` as a closed family rather than optional best-match
  convenience access
- `RequestedHistoricalPathClass` as a sealed request vocabulary
- planner-owned performance artifact vocabularies for correspondence and
  historical lanes
- one owning module boundary for correspondence outcomes and one for
  historical-path classes

Phase exit criterion:

- no later phase needs to invent new sameness or historical-path categories to
  proceed

### Phase 2: Lower Lineage And Structural Evidence Into Query-Owned Descriptors

Phase 2 exists to make correspondence query-owned without stealing authority
from lower identity semantics.

Milestone 5.4 must then implement:

- lowering of lower-runtime lineage continuity evidence into one explicit
  authoritative correspondence descriptor
- lowering of lower-runtime structural fingerprint evidence into one explicit
  advisory correspondence descriptor
- candidate-set and ambiguity modeling for multiple structural matches
- explicit disagreement artifacts when lineage and structural evidence do not
  agree
- typed denial for unsupported structural families or unsupported mixed-
  evidence requests

This phase leaves the system in a coherent state where:

- authoritative and advisory sameness remain distinct
- ambiguity and disagreement are explicit and digest-bearing
- unsupported correspondence cases fail before result shaping

Phase outputs:

- `CorrespondenceEvidenceResolved`
- `CorrespondenceCandidateSet`
- `StructuralCandidateDiscoveryPlan`
- `StructuralCandidateBudget`
- `CorrespondenceCostPosture`
- one of:
  - `LineageContinuity`
  - `AdvisoryStructuralUnique`
  - `AdvisoryStructuralAmbiguous`
  - `LineageStructuralDisagreement`
  - `CorrespondenceDenied`

Phase exit criterion:

- no later phase needs to inspect raw lineage facts or raw structural
  fingerprints directly
- no executor lane needs permission to widen candidate discovery beyond the
  planned budget

### Phase 3: Lower Historical Compatibility And Materialization-Path Admission

Phase 3 exists to make historical honesty visible before execution-path details
are lost.

Milestone 5.4 must then implement:

- historical evaluation requests that carry explicit requested basis and
  materialization-path compatibility posture
- lowering from admitted lower-runtime historical capability into typed
  `HistoricalEvaluationAdmission`
- explicit compatibility outcomes for retained snapshot, delta replay, full
  reconstruction, and denial
- typed denial for unsupported path requests or incompatible basis/path pairs
- exact counters for admitted path class, denied path class, and compatibility
  checks

This phase leaves the system in a coherent state where:

- historical-path choice is explicit before result shaping
- unsupported path requests fail closed instead of silently substituting
- later historical milestones can broaden basis classes without redefining path
  honesty

Phase outputs:

- `HistoricalPathRequested`
- `HistoricalPathAdmitted`
- `HistoricalPathResolved`
- `HistoricalPathCostPosture`
- `HistoricalReplaySpanBudget`
- `HistoricalReconstructionBudget`
- `HistoricalPathReuseDescriptor`
- one of:
  - `HistoricalPathCompatibilityOutcome::Admitted`
  - `HistoricalPathCompatibilityOutcome::Denied`
  - `HistoricalPathSubstitutionDenied`

Phase exit criterion:

- no result envelope can exist without distinct requested, admitted, and
  resolved path identity
- no executor lane needs permission to choose replay versus reconstruction on
  its own

### Phase 4: Execute And Shape Correspondence/Historical Result Envelopes

Phase 4 exists to make the consumer boundary honest.

Milestone 5.4 must then implement:

- correspondence-aware result envelopes that preserve result-family identity
  plus correspondence family identity
- historical result envelopes that preserve query/result meaning plus explicit
  materialization-path metadata
- typed envelopes for ambiguity, disagreement, and path-compatibility denial
- exact counter snapshots attached to result or failure bundles
- preservation of preview/live/frontier/basis metadata already established by
  earlier milestones where those routes compose with this milestone

This phase leaves the system in a coherent state where:

- consumers can see whether sameness was authoritative or advisory
- consumers can see how historical truth was materialized
- basis and route honesty remain preserved instead of wrapped away

Phase outputs:

- `CorrespondenceHistoricalEnvelope`
- `MetadataPreservingHistoricalResultView`
- typed denial envelopes for ambiguity, disagreement, and path incompatibility
- metadata-preserving cost-posture and budget snapshots attached to successful
  and denied envelopes

Phase exit criterion:

- public consumers cannot obtain a successful correspondence or historical
  result without the associated honesty metadata still attached

### Phase 5: Replay, Compare, And Certify Honesty

Phase 5 exists to prove these surfaces stay explicit under replay and repeated
evaluation.

Milestone 5.4 must then implement:

- parity bundles comparing lineage-backed, structural-backed, and mixed-
  evidence correspondence outcomes
- historical-path parity bundles comparing retained snapshot, replay, and full
  reconstruction where admitted
- canonical digests for query, lineage evidence, basis, result, failure, and
  counter snapshots
- deterministic reporting of ambiguity, disagreement, and compatibility denials

This phase leaves the system in a coherent state where:

- correspondence ambiguity remains replay-safe
- historical-path identity remains replay-safe
- later milestones inherit certifiable identity/history honesty instead of one
  narrative claim

Phase outputs:

- `CorrespondenceHistoricalParityBundle`
- replay-safe digests for requested, admitted, and resolved path classes
- replay-safe digests for correspondence outcome classes
- replay-safe digests for correspondence and historical cost posture classes
- `PerformancePredictionDriftOutcome`

Phase exit criterion:

- replay can distinguish semantic drift from mere route or path variation
- replay can distinguish cost-posture drift from ordinary realized-count
  variation

### Phase 6: Certification And Boundary Hardening

Phase 6 exists to close the milestone through named proof instead of
"comparison works" demos.

Milestone 5.4 must finally ship:

- the `Structural Correspondence And Historical Materialization Path Test`
- canonical rows proving:
  - lineage-backed correspondence explicitness
  - structural correspondence explicitness
  - ambiguous disagreement explicitness
  - retained snapshot path visibility
  - delta replay path visibility
  - full reconstruction visibility where admitted
- rejection rows proving:
  - structural-as-authoritative-forbidden
  - ambiguous-correspondence-not-collapsed
  - unsupported-correspondence-family
  - unsupported-historical-materialization-path
  - hidden-materialization-path-substitution-forbidden
  - host-cache-history-authority-forbidden
- compile-fail or privacy hardening proving correspondence and historical-path
  proof artifacts cannot be WORTHd externally

This phase leaves the system in a coherent state where:

- Milestone 5.4 is certifiable rather than descriptive
- Milestone 6 can broaden historical basis/query semantics without redefining
  materialization-path honesty
- Milestone 7 can broaden lineage/correspondence semantics without redefining
  advisory-versus-authoritative boundaries

### Representative Scenario Matrix

Milestone 5.4 certification should exercise at minimum:

- `lineage-correspondence-authoritative`:
  one admitted query with authoritative lineage-backed continuity evidence
- `structural-correspondence-advisory`:
  one admitted query with structural-fingerprint-backed advisory match and no
  implied continuity
- `lineage-structural-disagreement-explicit`:
  one admitted query where lineage and structural evidence disagree and the
  disagreement remains typed
- `structural-ambiguity-explicit`:
  one admitted query with multiple structural candidates and no silent tie
  break
- `historical-retained-snapshot-path`:
  one admitted historical query whose result bundle exposes retained snapshot
  materialization
- `historical-delta-replay-path`:
  one admitted historical query whose result bundle exposes replay-based
  materialization
- `historical-full-reconstruction-path`:
  one admitted historical query whose result bundle exposes reconstruction
  materialization where explicitly admitted
- `hidden-materialization-substitution-forbidden`:
  one hostile lane attempting to erase or rewrite historical-path identity

If the harness cannot name concrete lanes at this granularity, the milestone is
still too abstract to close honestly.

## Must Ship

- proof-bearing `LineageCorrespondenceDescriptor`,
  `StructuralCorrespondenceDescriptor`,
  `CorrespondenceOutcome`,
  `CorrespondenceAmbiguityEnvelope`,
  `HistoricalMaterializationPathMetadata`, and
  `HistoricalEvaluationAdmission` families or materially equivalent types
- explicit requested, admitted, and resolved historical-path classes rather
  than one collapsed path field
- structural-fingerprint-based correspondence as a first-class advisory query
  artifact beside lineage-backed correspondence
- explicit mixed-evidence disagreement and ambiguity result families
- query result metadata describing historical materialization path for admitted
  historical reads
- explicit compatibility/admission contracts for historical evaluation where
  the lower runtimes cannot serve a request honestly
- one dedicated correspondence/history performance subdomain owning cost
  contracts, counters, and status rather than generic telemetry-only logging
- planner-owned correspondence and historical cost posture artifacts with
  explicit budgets and drift outcomes
- typed diagnostics, replay bundles, and exact counters for correspondence and
  historical-path semantics
- milestone-native certification proving ambiguity honesty, advisory-versus-
  authoritative boundaries, and historical-path visibility

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- proof-bearing planning and basis identity from Milestone 3 remain
  authoritative
- collection/result-family semantics from Milestone 4 remain authoritative
- live/locality semantics from Milestones 5 and 5.1 remain authoritative where
  those routes compose with this milestone
- preview-session basis identity and promotion-parity surfaces from Milestone
  5.2 remain authoritative where preview-bound historical or correspondence
  lanes later compose with this milestone
- frontier posture and deterministic route honesty from Milestone 5.3 remain
  authoritative where admitted routes compose with this milestone
- lineage remains authoritative continuity; structural correspondence remains
  advisory unless lower-truth semantics say otherwise
- historical evaluation authority remains in lower runtimes, not in hosts,
  caches, or query-local reconstruction folklore
- ambiguity and materialization-path differences remain explicit in results
- public result access may not erase correspondence family or historical-path
  metadata from admitted success envelopes
- performance posture, budget class, and drift outcomes remain planner-owned
  rather than executor-selected

## Complexity / Proof Obligations

Milestone 5.4 must name costs and proofs in terms of:

- lineage evidence lookup count
- structural candidate count
- predicted structural candidate count
- structural ambiguity count
- lineage/structural disagreement count
- predicted correspondence resolution width
- historical compatibility check count
- predicted historical replay span
- predicted historical reconstruction scope
- historical retained-snapshot admission count
- historical delta-replay admission count
- historical full-reconstruction admission count
- historical path denial count
- correspondence replay parity count
- historical-path replay parity count
- work avoided by explicit path reuse versus broad reconstruction
- executor rediscovery avoidance on correspondence/history lanes

Minimum required counters:

- `lineage_evidence_lookup_count`
- `structural_candidate_count`
- `predicted_structural_candidate_count`
- `structural_candidate_rejection_count`
- `structural_ambiguity_count`
- `structural_unique_witness_count`
- `lineage_structural_disagreement_count`
- `structural_authority_promotion_denial_count`
- `predicted_correspondence_resolution_width`
- `historical_requested_path_count`
- `historical_admitted_path_count`
- `historical_resolved_path_count`
- `historical_compatibility_check_count`
- `predicted_historical_replay_span`
- `predicted_historical_reconstruction_scope`
- `historical_retained_snapshot_admission_count`
- `historical_delta_replay_admission_count`
- `historical_full_reconstruction_admission_count`
- `historical_path_denial_count`
- `historical_hidden_path_substitution_denial_count`
- `structural_candidate_prediction_drift_count`
- `historical_replay_span_drift_count`
- `historical_reconstruction_scope_drift_count`
- `historical_result_path_metadata_count`
- `correspondence_replay_parity_count`
- `historical_path_replay_parity_count`
- `history_work_avoided_by_retained_path_count`
- `correspondence_executor_rediscovery_count`
- `historical_executor_rediscovery_count`

Rules:

- counters belong to correspondence result bundles, historical result bundles,
  and certification bundles
- representative certification scenarios must assert exact counts
- `correspondence_executor_rediscovery_count` must be exactly zero on every
  admitted path
- `historical_executor_rediscovery_count` must be exactly zero on every
  admitted path
- every denied attempt to treat structural correspondence as authoritative must
  increment `structural_authority_promotion_denial_count`
- every successful unique structural match must increment
  `structural_unique_witness_count`
- every admitted historical request must record requested, admitted, and
  resolved path counts distinctly
- every admitted lane must record predicted and realized performance posture
  artifacts distinctly rather than inferring them from one combined counter
- every denied or incompatible historical path request must increment
  `historical_path_denial_count`
- every hidden path-substitution denial must increment
  `historical_hidden_path_substitution_denial_count`
- every prediction drift outside the admitted budget class must increment one of
  `structural_candidate_prediction_drift_count`,
  `historical_replay_span_drift_count`, or
  `historical_reconstruction_scope_drift_count`
- no supported path may hide path substitution or ambiguity collapse inside
  generic success counters
- no supported path may hide planner-versus-executor cost posture mutation
  inside generic historical success counters
- "work avoided" counters must make retained-path reuse and explicit replay
  visible rather than anecdotal

Minimum certification rows should include:

- `lineage-correspondence-explicitness`
- `structural-correspondence-explicitness`
- `correspondence-ambiguity-explicitness`
- `retained-snapshot-path-explicitness`
- `delta-replay-path-explicitness`
- `full-reconstruction-path-explicitness`
- `historical-path-no-substitution`
- `correspondence-cost-posture-parity`
- `historical-cost-posture-parity`
- `prediction-drift-explicitness`
- `work-avoided-counter-parity`

Minimum rejection rows should include:

- `structural-as-authoritative-forbidden`
- `ambiguous-correspondence-not-collapsed`
- `unsupported-correspondence-family`
- `unsupported-historical-materialization-path`
- `hidden-materialization-path-substitution-forbidden`
- `host-cache-history-authority-forbidden`
- `raw-ambiguity-bool-forbidden`
- `forbidden-broad-candidate-scan-success`
- `forbidden-executor-path-selection`
- `naked-historical-payload-forbidden`
- `naked-best-match-accessor-forbidden`

## Allowed Debt

- some structural correspondence families may remain unsupported as explicit
  `Debt` while admitted families are fully ambiguity-proven
- some historical materialization paths may remain unsupported as explicit
  `Debt` while admitted paths are explicit and certified
- broader correspondence exploration and lineage traversal remain later work
- durable historical restore and store-backed historical parity remain later
  work
- silent collapse of ambiguity may not exist as debt
- hidden materialization-path substitution may not exist as debt
- host-cache historical authority may not exist as debt
- structural correspondence presented as continuity by default may not exist as
  debt

## Acceptance Evidence

Milestone 5.4 is complete only when `worth-query` can prove:

- the `Structural Correspondence And Historical Materialization Path Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- structural correspondence is explicit and distinct from lineage continuity
- ambiguous correspondence remains explicit and typed
- historical query results expose materialization-path meaning where admitted
- unsupported correspondence or historical-path cases fail typed and early
- correspondence and historical artifacts remain typed, replay-safe, and
  ambiguity-honest

Required verification output must include:

- `query_digest`
- `lineage_digest`
- `basis_digest`
- `result_digest`
- `failure_digest`
- `counter_snapshot`

## Architectural Notes

### Advisory Must Stay Advisory

This milestone only works if the system keeps one hard rule:

- lineage is continuity
- structure is correspondence

Later lower-truth semantics may define promotion rules. Milestone 5.4 may not
smuggle those in early.

The compile-time consequence should be explicit:

- only `LineageContinuity` may expose authoritative-continuity accessors
- `AdvisoryStructuralUnique` may expose a unique advisory candidate, but never
  continuity accessors
- `AdvisoryStructuralAmbiguous` may expose candidate sets, but never one
  convenience winner

### Historical Path Is Part Of Truth Delivery

How historical truth was served is not optional metadata. It changes what the
consumer knows about the result and what later certification can prove.

The required rule is:

- if the lower runtime served retained snapshot truth, say so
- if it served replayed truth, say so
- if it reconstructed truth fully, say so
- if it cannot serve the request honestly, deny it

The type consequence should also be explicit:

- request intent, admission, and final path are three different proof states
- any API that merges them into one enum or one field is architecture debt at
  best and dishonesty at worst

### This Milestone Must Not Steal Milestone 6 Or 7

Milestone 5.4 freezes the contract layer those milestones depend on.

- Milestone 6 will own broader historical reads and diff semantics
- Milestone 7 will own richer lineage traversal and correspondence query
  surfaces
- Milestone 5.4 owns the honesty boundary that keeps both milestones from
  flattening continuity, correspondence, and historical materialization into
  one generic story

### Ambiguity Is A Product Surface

The easiest way to fake correspondence support is to always choose one answer.

Milestone 5.4 must instead treat ambiguity as first-class:

- ambiguity can succeed as an explicit ambiguous outcome
- ambiguity can deny later workflow requests that require stronger proof
- ambiguity may not disappear because the UI would prefer one row

## Sequencing Notes

Milestone 5.4 belongs immediately after Milestone 5.3 because route posture,
basis identity, and preview/live honesty already exist and can now carry
correspondence and historical-path semantics without reopening those earlier
boundaries.

It must land before Milestone 6 because historical reads and diff semantics
need explicit materialization-path contracts instead of one vague historical
success lane.

It must land before Milestone 7 because richer lineage and correspondence work
needs explicit advisory-versus-authoritative and ambiguity semantics rather
than backfilling them later.

## Parallelization Notes

Once the correspondence and historical-path vocabulary is frozen:

- Milestone 5.5 workflow lowering can progress in parallel without redefining
  advisory versus authoritative identity semantics
- Milestone 5.6 facade/configuration work can surface capability metadata
  against explicit correspondence/history contracts rather than implied support
- early Milestone 6 history/diff design can proceed in parallel without
  changing materialization-path honesty
- counter hardening and compile-time tightening can proceed in parallel without
  changing milestone semantics

## Explicit Failure Taxonomy For Milestone 5.4

- unsupported correspondence family
- unsupported structural fingerprint family
- structural-authority promotion denial
- structural ambiguity not surfaced
- lineage/structural disagreement hidden
- invalid historical compatibility request
- unsupported historical materialization path
- hidden historical-path substitution
- host-cache history-authority violation
- correspondence replay divergence
- historical-path replay divergence
- correspondence/history artifact invariant break

## Anti-Patterns Explicitly Rejected

- correspondence APIs that do not disclose whether evidence was lineage or
  structure
- structural matching presented as continuity by default
- host-side tie breaking for ambiguous structural candidates
- historical result envelopes that omit materialization-path identity
- hidden substitution of retained snapshot, replay, and reconstruction paths
- host caches presented as historical authority
- one mega-module mixing correspondence derivation, historical admission,
  diagnostics, replay, and certification
- public construction of correspondence or historical-path proof types without
  the proving path

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes the semantic difference between authoritative
continuity and advisory correspondence, and the semantic difference between
historical result meaning and historical materialization-path meaning, before
later branch/history/identity work multiplies the number of ways those truths
could be flattened.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where structural resemblance quietly becomes continuity and historical
result bundles quietly hide whether they came from retained state, replay, or
reconstruction.

The milestone preserves authority boundaries because lower runtimes still own
lineage, structural evidence, and historical materialization authority, while
`worth-query` owns the typed query-facing descriptors, metadata, denials,
diagnostics, and certification artifacts.

The milestone defines proof obligations rather than implementation chores
because ambiguity explicitness, advisory-versus-authoritative honesty,
historical-path visibility, replay-safe parity, and exact counters are
required for closeout.

A competent engineer should be able to map this spec into honest
correspondence, historical-admission, result-envelope, certification, and
compile-fail modules without inventing the architecture during implementation.

This milestone belongs at 5.4 because it is the contract-hardening layer that
must exist before broader historical and lineage capabilities can ship
honestly.

## Closeout Standard

Milestone 5.4 is complete only when all of the following are true:

- lineage-backed continuity and structural correspondence are distinct,
  explicit query surfaces
- ambiguity and disagreement remain typed rather than silently collapsed
- admitted historical result envelopes expose their materialization path
- unsupported correspondence and historical-path cases fail typed and early
- correspondence and historical-path semantics remain replay-safe and
  machine-checkable
- later milestones can build on these contracts without redefining continuity,
  correspondence, or historical materialization honesty

If code lands but structural matches still masquerade as continuity,
historical-path identity still disappears in successful results, or host caches
still participate as historical authority, Milestone 5.4 is not complete.
