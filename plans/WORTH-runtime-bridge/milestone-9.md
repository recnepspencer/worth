# Milestone 9 Engineering Spec: Merge-Aware Bridge Semantics And Multi-Parent History Consumption

> **Status:** Closed engineering spec and shipped closeout reference
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-8.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-8.md)
>
> **Prior closeout:** [milestone-8-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-8-closeout.md)
>
> **Milestone closeout:** [milestone-9-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-9-closeout.md)
>
> **Bridge certification companion:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
>
> **Primary architectural driver:** make ordered multi-parent truth history, merge ontology, causal frontier evidence, and schema-declared merge policy outcomes first-class bridge inputs so invalidation, continuity, remapping, explanation, and replay can remain deterministic without re-inventing merge semantics inside bridge or host code
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

Milestones 1 through 8 established that the bridge already has strong protocol
surfaces for:

- canonical patch ingestion and deterministic invalidation routing
- aspect-aware precision and fine-grained subscriptions
- lineage-aware continuity and branch/historical truth-view selection
- planned bulk routing and replay-safe stream consumption
- protocol-grade source reads and structural-identity-aware advisory remapping

That is enough to make the bridge strong against linear history, branch-local
variation, and structural ambiguity.

It is not enough to make the bridge strong against merge-bearing history.

Without Milestone 9, the bridge still has a dangerous blind spot:

- it can see branch-local truth but not ordered multi-parent truth as a
  canonical protocol surface
- it can consume lineage continuity but not merge ontology as a first-class
  explanation basis
- it can compare structure but not explain whether a merge outcome came from
  parent ordering, causal independence, schema policy, or typed merge denial
- it can replay linear and branch-local history safely, but merge-bearing
  history still risks procedural rediscovery inside adapters or host code

Milestone 9 exists because Milestone 8 deliberately stopped at advisory
structural identity. That milestone taught the bridge how to say:

`these candidates are structurally similar, ambiguous, or advisory-only`

Milestone 9 must now teach the bridge how to say:

`this exact ordered multi-parent truth artifact, carrying this exact merge class, these exact causal frontier facts, and this exact schema-declared merge-policy outcome, produced this exact invalidation, continuity, remap, rejection, and explanation result`

not:

`the bridge saw something merge-like and inferred what probably happened`

## Goal

Make merge-bearing truth history a deterministic, replay-safe, bridge-owned
consumption protocol so the bridge can route, remap, explain, and reject
merge-influenced truth evolution without becoming the authority for merge
meaning or execution.

## Why This Milestone Exists

Milestone 9 belongs immediately after Milestone 8 because merge-bearing history
multiplies structurally plausible candidates and continuity branches, and the
bridge needed a first-class ambiguity model before merge-aware consumption
could be honest.

Milestone 8 established:

- structural equivalence contracts
- explicit ambiguity and identity-conflict outcomes
- replay-safe advisory remap artifacts
- deterministic branch comparison artifacts

Milestone 9 now needs to establish the matching merge-side truths:

- canonical ordered multi-parent history identity
- explicit merge ontology and merge-class admission
- explicit causal-frontier and schema-policy evidence consumption
- explicit merge-aware routing, continuity, remapping, and explanation artifacts

If Milestone 9 shipped before Milestone 8, merge-bearing histories would widen
the search space before the bridge had a trustworthy language for ambiguity,
contradiction, and advisory-only structural evidence.

Milestone 9 also belongs before Milestone 10 because speculative branch
coordination becomes unsafe if the bridge still treats merge-bearing truth
history as an adapter-local special case. Speculative coordination needs a
canonical merge-aware history basis before it can safely model discard,
preview, and commit boundaries.

Milestone 9 therefore earns its place in the roadmap by solving the next real
structural problem after structural remapping: consuming merge-bearing truth
history canonically enough that the bridge no longer assumes history is linear.

## Adversarial Constraint

Milestone 9 must survive the following hostile condition:

> A long-lived system with ordered multi-parent commits, supported and
> unsupported merge classes, causally independent and causally dependent parent
> histories, schema-declared merge policies, merge-driven deletion and topology
> rewiring, structural near-matches, branch-local replay, host-adapter
> ordering variation, and diagnostics tiers that vary by environment must
> produce the same merge-history interpretation, the same invalidation and
> continuity outcome, the same remap and explanation artifacts, and the same
> replay result every time, while never allowing bridge code or host code to
> rediscover merge meaning procedurally or degrade unsupported merge classes
> into heuristic reconciliation.

Concretely, the design must remain correct when all of the following are true:

- two parents are causally independent and policy-admitted
- two parents are causally dependent and require ordered interpretation
- parent ordering differs from lexical, adapter, or storage iteration order
- one merge outcome resolves by declared schema policy while another fails
  closed
- merge history includes deletion or topology rewiring surfaces
- structural similarity suggests continuity but merge ontology rejects it
- replay occurs after unrelated publication and adapter implementation changes
- diagnostics richness changes between environments
- one host adapter exposes merge evidence through a narrow artifact surface
  while another exposes richer internal state the bridge must ignore

If any supported path:

- drops or normalizes parent order as though it were incidental
- infers merge meaning from patch shape alone when canonical merge artifacts
  already exist
- lets host adapters widen or reinterpret merge classes outside bridge-owned
  admission
- fabricates continuity from structural likeness when merge ontology denies it
- treats unsupported merge classes as best-effort branch reconciliation
- cannot replay the same merge-aware routing and explanation result from
  canonical bridge artifacts alone
- hides merge causality or merge-policy outcomes inside diagnostics text rather
  than typed bridge artifacts

then Milestone 9 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- ordered multi-parent history is a first-class bridge input, not a host-local
  special case
- merge class, parent-order basis, causal-frontier evidence, schema-policy
  outcome, and merge-aware bridge result are distinct concepts and must remain
  distinct types
- every bridge-admitted merge class must map losslessly to canonical
  relational merge ontology through either direct wrapper identity or a
  versioned total lowering recorded in canonical artifacts
- the truth runtime remains the authority for merge ontology, causal metadata,
  merge execution, and schema-declared merge policies
- the bridge consumes canonical merge artifacts; it does not infer merge
  semantics from raw branch topology or patch coincidence
- parent order is semantically load-bearing when the truth artifact says it is;
  bridge code may not sort, normalize, or ignore it for convenience
- unsupported merge classes fail explicitly during admission or lowering, not
  late during downstream execution
- structural identity remains advisory under merge pressure and may refine
  candidate classification but may not override canonical merge authority
- merge-aware explanation must be replay-safe and reconstructable from
  canonical bridge records rather than live host queries
- diagnostics richness may change retained detail, but not merge-aware meaning,
  denial class, routing outcome, or replay result
- Milestone 9 productizes merge-aware truth consumption only; it does not
  productize speculative preview lifecycle, cross-runtime policy propagation,
  or bridge-mediated writeback

Normative consequence:

- bridge APIs that expose "merged enough" or "best parent set" without typed
  merge-class and parent-order meaning are out of spec
- host adapters that reinterpret parent order or merge policy outcomes are out
  of spec
- merge-aware continuity inferred solely from structural likeness is out of spec
- replay that re-runs merge reasoning against ambient latest truth is out of
  spec
- diagnostics-only merge explanation without canonical bridge artifacts is out
  of spec
- fallback from unsupported merge class to single-parent or branch-head
  approximation is out of spec

## Configuration And Defaults

Milestone 9 should expose only a small set of explicit merge-consumption
configuration surfaces. Merge authority boundaries are not configurable.

### Admitted Configurable Surfaces

- merge-consumption mode
  - default: `DeterministicCanonical`
- admitted merge class set
  - default: schema/runtime-declared supported classes only
- merge-aware continuity mode
  - default: explicit bridge continuity contract consuming merge authority
- merge-aware remap publication policy
  - default: publish only when merge outcome is canonical and replay-safe
- diagnostics richness
  - default: structured standard diagnostics, not maximum forensic retention
- replay retention richness for merge artifacts
  - default: retain enough canonical merge evidence to replay routing,
    continuity, remap, rejection, and explanation without ambient truth lookup

### Non-Configurable Surfaces

- parent-order normalization for convenience
  - default: never admitted
- heuristic merge reconstruction from patch shape alone
  - default: never admitted
- structural similarity overriding merge authority
  - default: never admitted
- unsupported merge classes degrading into branch reconciliation
  - default: never admitted
- host-local override of schema-declared merge policy outcome
  - default: never admitted

The bridge should therefore feel configurable at the declaration and retention
layer, but closed and fail-safe at the merge authority boundary.

## Guideline Influence

### 1. `MENTALITY.md`

This document directly shapes the milestone:

- adversarial constraint first:
  the spec starts from ordered-parent hostility, unsupported merge classes,
  causal-frontier pressure, topology rewiring, and replay variation rather than
  from the pleasant feature phrase "support merge-aware history"
- solve the hard problem first:
  merge admission, ordered-parent preservation, typed denial, and replay-safe
  explanation ship before convenience APIs for merge-aware consumers
- enforce mechanically, not by convention:
  parent ordering, merge-class support, causal evidence, and merge-policy
  outcomes must travel as proof-bearing types and typed failures
- spec is architecture is code:
  the spec names the merge proof chain, subdomains, counters, and artifact
  types that implementation must map directly
- authority first, derivation second:
  merge ontology stays authoritative in `worth-relational`; bridge merge
  diagnostics and merge-aware routing artifacts are derived and rebuildable
- separate what/how/whether:
  merge truth is the `what`, bridge lowering and routing are the `how`, and
  diagnostics richness is the `whether`

### 2. `architectural_guidelines.md`

This document determines the structural boundaries:

- Laws 7, 8, and 32:
  every merge-aware boundary crossing must emit self-describing envelopes,
  decision traces, and counters rather than opaque "merge happened" results
- Laws 16, 18, 30, and 41:
  merge-aware consumption must move through a proof chain such as declaration,
  validation, admission, lowering, routing, publication, and replay
- Laws 21, 27, and 33:
  merge meaning, execution, and policy authority stay upstream; the executor
  may consume only lowered merge-aware bridge plans and derived artifacts
- Laws 26 and 40:
  equivalence and naming must stay explicit; merge class, parent order,
  causal frontier, and policy outcome cannot collapse into one generic
  "merge metadata" bag
- Law 29:
  abstraction must stop before it hides correctness or cost boundaries, so
  supported merge classes, denied merge classes, merge-driven deletion, and
  topology rewiring cannot disappear behind one convenience result type

### 3. `domain_standards.md`

This document determines crate decomposition:

- merge-aware bridge work must live in dedicated subdomains such as
  `merge/declaration`, `merge/contracts`, `merge/history`, `merge/lowering`,
  `merge/routing`, `merge/continuity`, and `merge/explanation`
- merge-class admission is not the same responsibility as explanation
  reconstruction
- causal-frontier interpretation is not the same responsibility as structural
  remap publication
- diagnostics reconstruction is not the same responsibility as replay
  certification
- tests must mirror merge responsibilities rather than collapsing all
  merge-bearing behavior into one large integration file

### 4. `performance_guidelines.md`

This document determines the cost model:

- merge-aware history consumption must scale with admitted merge envelope
  breadth, parent count, touched merge surfaces, and lowered routing packets,
  not with whole-branch or whole-history scans by default
- unsupported merge classes must be rejected before expensive continuity or
  remap materialization
- APIs must reveal when a request implies merge-aware history traversal,
  explanation reconstruction, or replay verification
- counters must explain parent breadth, merge-class incidence, rejection count,
  continuity fanout, remap breadth, and replay equivalence cost
- merge-aware reuse without an explicit equivalence contract is forbidden
- if widened history scans are ever admitted for specific merge classes, they
  must be explicit debt-bearing modes with separate counters and typed plan
  markings

## Scope

### In Scope

- one bridge-owned declaration surface for merge-aware history consumption
- canonical ordered-parent, merge-class, causal-frontier, and merge-policy
  evidence identities
- bridge-owned admission for supported and unsupported merge classes
- merge-aware lowering for invalidation, continuity, advisory remap, and
  explanation
- replay-safe merge records, diagnostics, and explanation surfaces
- typed failures and counters for unsupported merge classes, parent-order drift,
  merge-policy mismatch, merge-aware continuity denial, and replay mismatch
- harness certification for suites 10 through 12 in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)

### Explicitly Out Of Scope

- speculative preview and discard lifecycle
- cross-runtime policy provenance beyond consuming canonical merge-policy
  outcomes already decided by truth authority
- bridge-mediated writeback or merge-producing commit strategies
- scheduler-owned downstream execution semantics inside `worth-signal`
- introducing new merge ontology or new merge execution semantics inside the
  bridge

Milestone 9 must stay focused on merge-aware truth consumption rather than
absorbing speculative, policy, or writeback work early.

## Governing Design Rules

### 1. Truth Owns Merge Meaning, Bridge Owns Merge Consumption

The truth runtime defines:

- merge ontology and merge classes
- ordered parent lists
- causal frontier metadata
- schema-declared merge policies and policy outcomes
- merge execution and authoritative merge artifacts
- deletion, reconciliation, and topology rewiring semantics

The signal runtime defines:

- derived node identity
- downstream execution over invalidations and remap artifacts
- derived explanation consumption after bridge publication

The bridge defines:

- merge-consumption request shape
- merge-aware lowering vocabulary
- merge-aware invalidation, continuity, remap, denial, and explanation artifacts
- merge diagnostics and replay records

The bridge must not define its own merge ontology.

### 1.1 One Declaration Surface Must Begin Every Merge-Aware Read Story

Milestone 9 must not allow hosts or consumers to assemble merge-aware behavior
from scattered flags and adapter-specific helper calls.

There must be one bridge-owned declaration surface that states:

- which merge-bearing truth artifact family is being consumed
- which merge classes are admitted
- what authority basis is being used
- whether continuity, remap, explanation, or replay is being requested
- what diagnostics richness and retention surfaces are attached

This declaration surface is the only public starting point for merge-aware
admission.

### 1.2 Merge Authority Basis Must Remain Explicit

Every admitted merge-aware contract must carry one explicit authority basis
inherited from canonical truth artifacts.

For Milestone 9 the closed authority-basis vocabulary must include:

- `OrderedMergeCommit`
- `HistoricalMergeEnvelope`
- `BranchHeadMergeArtifact`
- `ReplayMergeRecord`

Rules:

- authority basis is part of merge-contract identity
- merge-contract identity must also carry canonical merge ontology version,
  schema merge-policy descriptor version, and parent-order digest/version
- packet lowering and replay records must preserve the exact basis
- host adapters may not substitute, reorder, or widen the basis during
  materialization
- later speculative or policy-rich authority bases may extend this vocabulary,
  but they are not ambiently admitted here

### 2. Merge Class Admission Must Precede Routing

Milestone 9 must reject unsupported merge classes before invalidation routing,
continuity lowering, advisory remap publication, or explanation reconstruction
begins.

The bridge must know, before routing:

- whether the merge class is bridge-admitted
- whether parent ordering is complete and canonical
- whether causal frontier evidence is present at the required level
- whether the schema-policy outcome is canonical and replay-compatible
- whether merge-driven deletion or topology rewiring is supported for the class
  being consumed

This directly follows the rule that rejection must precede expensive
construction.

### 2.1 Merge Class Vocabulary Must Be Closed Per Milestone

Milestone 9 must not leave merge class meaning implementation-defined.

For this milestone the bridge must define a closed vocabulary for merge-aware
consumption classes, such as:

- `AspectReconciliationMerge`
- `DeletionMerge`
- `TopologyRewireMerge`
- `PolicyResolvedConflictMerge`
- `UnsupportedMergeClass`

Rules:

- the bridge merge-class vocabulary is not an independent semantic authority;
  it is a bridge consumption vocabulary that must be losslessly derived from
  canonical relational merge ontology
- each class defines the minimum required truth artifacts, causal metadata, and
  policy evidence
- each class defines which bridge products are legal:
  invalidation only, continuity, advisory remap, explanation, or denial
- each class also defines whether structural evidence may be consulted
  advisory-only after merge authority is known
- every admitted bridge class must record either:
  - the exact canonical relational merge class identifier it wraps, or
  - the exact versioned lowering rule that converted canonical relational merge
    ontology into the bridge consumption class
- many-to-one lowering is allowed only when the lowered artifact records enough
  canonical provenance to reconstruct the original relational class without
  ambiguity during replay and diagnostics
- replay across differing merge-class semantics versions is a typed
  incompatibility, not best-effort compatibility

### 3. Parent Order Is a Proof, Not an Optimization Detail

Milestone 9 must not reduce ordered parent lists to sets.

Rules:

- ordered parents remain in canonical order from truth artifact through bridge
  replay artifact
- any digest involving merge-bearing history must include parent-order basis
- bridge internals may derive indexed lookup structures, but those are derived
  artifacts and may not redefine canonical order
- adapter iteration order is never authority
- if a host cannot provide canonical parent order for a declared authority
  basis, admission fails

### 4. Causal Frontier And Policy Outcome Must Remain Separate From Patch Shape

Milestone 9 must not collapse:

- `MergeHistoryDeclaration`
- `ValidatedMergeHistoryDeclaration`
- `AdmittedMergeHistoryContract`
- `LoweredMergeHistoryPacketSet`
- `ReducedMergeRoutingArtifact`
- `PublishedMergeContinuityArtifact`
- `PublishedMergeRemapArtifact`
- `PublishedMergeExplanationArtifact`

Rules:

- patch shape alone is insufficient to define merge meaning
- causal-frontier evidence remains distinct from merge-policy outcome
- merge-policy outcome remains distinct from resulting patch surfaces
- explanation must name whether an outcome came from parent order, causal
  dependence, policy resolution, deletion semantics, topology rewiring, or
  typed denial
- replay records must preserve these distinctions separately

### 5. Structural Identity Remains Advisory Under Merge Pressure

Milestone 9 must consume Milestone 8 structural artifacts without promoting
them to merge authority.

Rules:

- canonical merge ontology dominates structural likeness
- structural evidence may help explain candidate correspondence after merge
  authority has already admitted the class
- when structural evidence and merge authority disagree, the bridge must emit a
  typed contradiction or denial artifact rather than picking one silently
- merge-aware continuity may only continue when the ordered precedence chain
  admits it

### 5.1 Merge-Aware Continuity Precedence Must Be Explicit

Milestone 9 must not leave merge-aware continuity as a vague combination of
truth signals and bridge judgment.

The bridge must follow one explicit precedence order:

1. merge-class admission
2. authoritative lineage or merge-successor export
3. deletion and topology-rewire denial gates
4. causal-frontier admissibility
5. schema-policy outcome admissibility
6. structural advisory refinement, if still legal
7. continuity or remap publication

Rules:

- a later stage may refine or deny an earlier admissible candidate, but may not
  silently bypass an earlier denial
- structural advisory refinement is never allowed to reopen continuity after an
  authoritative denial from lineage, deletion, topology, causal, or policy
  stages
- every denial artifact must name the precedence stage that blocked
  continuation

### 6. Explanation Must Be Derived From Canonical Merge Records

Milestone 9 explanation is not a logging add-on.

Rules:

- every merge-aware bridge result must be reconstructable from canonical merge
  records alone
- replay must certify equality or typed incompatibility of the full canonical
  merge result bundle, not only explanation parity
- explanation may be omitted by policy, but if materialized it must be derived
  from the same canonical merge record used for replay
- diagnostics tiers may change richness only
- explanation reconstruction may not query ambient latest truth or adapter
  internals

## Phase 1: Merge Taxonomy, Ordered History Basis, And Admission Lock

Phase 1 exists to make merge-bearing history a singular explicit bridge concept
instead of a set of adapter-specific exceptions.

Milestone 9 must first define:

- `MergeHistoryDeclaration`
- `ValidatedMergeHistoryDeclaration`
- the closed merge-class vocabulary for this milestone
- explicit authority-basis vocabulary for ordered merge artifacts
- parent-order proof types
- typed unsupported-merge and parent-order-denial failures

This phase leaves the system in a coherent state where:

- ordered multi-parent history is representable as one bridge concept
- parent order is preserved as canonical proof, not convenience metadata
- unsupported merge classes fail before routing or explanation begins
- the bridge is explicit about what merge meaning it is consuming from truth
  authority and what it is not allowed to invent

## Phase 2: Merge-Aware Lowering, Routing, Continuity, And Advisory Remap

Phase 2 exists to turn merge-aware vocabulary into deterministic bridge work.

Milestone 9 must then implement:

- admission of merge-aware declarations
- packetized lowering of merge-bearing history into deterministic bridge plans
- merge-aware invalidation routing
- merge-aware continuity classification
- merge-aware advisory remap publication where admitted
- explicit typed denial for unsupported or contradictory merge cases
- exact counters and decision records for parent breadth, merge-class incidence,
  continuity fanout, remap breadth, and rejection width

This phase leaves the system in a coherent state where:

- identical merge-bearing truth artifacts lower to identical bridge packets
- continuity and remap consume canonical merge truth rather than procedural
  host reconstruction
- unsupported merge classes fail closed before downstream semantic drift
- merge-aware bridge behavior stays deterministic under adapter-order
  variation, replay, and unrelated publication

Admitted merge-aware lowering in this phase must define a named complexity
contract.

Required complexity rule:

- merge-aware hot paths must scale with admitted parent count, touched merge
  surfaces, lowered packet count, and reduced contradiction width
- discovery work needed to derive those inputs must also be bounded and
  countered explicitly; implementation may not hide O(history), O(branch), or
  O(candidate cohort) rediscovery before the measured lowering boundary
- the minimum discovery counter floor must include:
  - `merge_history_segment_scan_count`
  - `merge_causal_frontier_lookup_count`
  - `merge_lineage_resolution_width`
  - `merge_structural_consult_width`
  - `merge_candidate_cohort_width`
- whole-history and whole-branch scans are not admitted default execution
  strategies
- widened scans, if ever admitted for specific merge classes, must be explicit
  debt-bearing modes with separate counters and typed plan markings

## Phase 3: Replay, Explanation, And Certification Against Merge Hostility

Phase 3 exists to prove that merge-aware consumption is trustworthy instead of
adapter folklore.

Milestone 9 must finally ship:

- replay-safe merge-aware bridge records
- explanation reconstruction for merge-driven invalidation, continuity,
  deletion, topology rewiring, and denial
- harness certification for merge parent order determinism, unsupported merge
  class denial, and merge replay/explanation parity
- hostile coverage for supported and unsupported merge classes, causal
  dependence and independence, policy-resolved and policy-rejected merges,
  merge-driven deletion, and topology rewiring
- exact counter assertions for representative merge-aware lanes

This phase leaves the system in a coherent state where:

- merge-bearing history is certifiable as a canonical bridge input
- replay validates merge-aware routing and explanation directly
- Milestone 10 can build speculative coordination on a stable merge-aware
  history substrate

## Must Ship

- canonical merge-history declaration, contract, parent-order proof, merge
  lowering, routing, continuity, remap, denial, and explanation artifacts
- a closed bridge-admitted merge-class vocabulary with explicit support and
  denial boundaries
- lossless canonical provenance from bridge merge classes back to relational
  merge ontology
- typed parent-order, causal-frontier, schema-policy, and merge-outcome records
- typed ontology-version, policy-descriptor-version, and parent-order-digest
  identity fields on admitted merge contracts and replay records
- typed contradiction artifacts for merge authority versus structural or
  continuity disagreement
- one canonical merge result bundle containing routing, continuity, remap,
  denial/failure, and explanation digests for replay certification
- packetized merge-aware lowering and deterministic reduction
- replay-safe merge records and explanation surfaces
- counters and decision-log records for parent breadth, merge-class count,
  supported-versus-unsupported class count, continuity fanout, remap breadth,
  deletion/topology merge incidence, and replay mismatch
- counters and decision-log records for discovery work including causal-frontier
  lookups, lineage resolution width, candidate cohort width, structural consult
  width, and history segment scans
- typed failures for unsupported merge class, parent-order drift, merge policy
  evidence mismatch, causal frontier truncation, merge-aware continuity denial,
  merge explanation basis mismatch, and merge replay mismatch
- harness certification satisfying Milestone 9 suites 10 through 12 in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)

## Must Preserve

- truth runtime remains the authority for merge ontology, merge execution,
  causal metadata, and schema-declared merge policies
- signal runtime remains the authority for derived node identity and execution
- bridge does not become merge authority
- parent order remains canonical and replay-safe
- structural identity remains advisory and subordinate to merge authority
- unsupported merge classes fail explicitly rather than degrading heuristically
- diagnostics richness changes explanation only, not merge-aware meaning
- no host-local merge interpretation becomes accidental public contract

## Acceptance Evidence

Milestone 9 is complete only when the bridge harness can prove:

- merge-bearing histories route deterministically through the bridge
- ordered parent lists survive ingestion, lowering, publication, and replay
- every bridge merge class can be traced losslessly back to canonical
  relational merge ontology during replay and diagnostics
- merge-aware continuity and remap behavior consume canonical merge authority
  rather than patch-shape folklore
- unsupported merge classes fail explicitly with typed bridge diagnostics and
  leave no misleading derived artifacts
- replayed merge-aware bridge result bundles match original bundles or fail with
  typed incompatibility
- explanation surfaces can localize exactly which merge inputs and merge
  outcomes influenced invalidation, continuity, remap, deletion, or topology
  rewiring
- representative merge workloads prove bounded discovery cost rather than only
  bounded post-lowering execution cost
- the Milestone 9 certification suites in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
  pass with canonical machine-checkable bundles

## Architectural Notes

### Expected Internal Subdomains

Milestone 9 should extend the bridge crate with subdomains such as:

- `merge/declaration/`
- `merge/contracts/`
- `merge/history/`
- `merge/lowering/`
- `merge/routing/`
- `merge/continuity/`
- `merge/remapping/`
- `merge/explanation/`
- `diagnostics/merge/`
- `harness/fixtures/merge_parent_order.rs`
- `harness/fixtures/unsupported_merge.rs`
- `harness/fixtures/merge_replay.rs`

This follows workspace domain standards:

- merge-class admission is not the same responsibility as merge-aware routing
- continuity lowering is not the same responsibility as advisory remap
  publication
- explanation reconstruction is not the same responsibility as replay
  verification
- unsupported-merge denial is not the same responsibility as malformed-history
  rejection

### Minimum Counter Floor

Milestone 9 must add counters such as:

- `merge_history_declaration_count`
- `merge_history_contract_count`
- `merge_parent_count`
- `merge_supported_class_count`
- `merge_unsupported_class_count`
- `merge_parent_order_rejection_count`
- `merge_history_segment_scan_count`
- `merge_causal_frontier_count`
- `merge_causal_frontier_lookup_count`
- `merge_policy_outcome_count`
- `merge_packet_count`
- `merge_routing_result_count`
- `merge_continuity_count`
- `merge_continuity_denial_count`
- `merge_lineage_resolution_width`
- `merge_candidate_cohort_width`
- `merge_structural_consult_width`
- `merge_remap_publication_count`
- `merge_deletion_class_count`
- `merge_topology_rewire_class_count`
- `merge_structural_contradiction_count`
- `merge_explanation_request_count`
- `merge_replay_request_count`
- `merge_replay_mismatch_count`
- `merge_widened_scan_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Merge Failure Policy

Milestone 9 must carry merge failures structurally rather than narratively.

Required failure classes:

- `UnsupportedMergeClass`
- `MergeAuthorityBasisMismatch`
- `MergeParentOrderDrift`
- `MergeParentOrderMissing`
- `MergeCausalFrontierTruncated`
- `MergePolicyOutcomeMismatch`
- `MergeContinuityDenied`
- `MergeStructuralContradiction`
- `MergeExplanationBasisMismatch`
- `MergeReplayMismatch`
- `MergeMaterializationRejected`
- `MergeOntologyLoweringMismatch`
- `MergeResultBundleMismatch`

Rules:

- every admitted merge-aware request receives exactly one reduced bridge result
  or one typed failure
- failure remains visible in canonical merge-aware bridge truth
- failure must identify the merge boundary that failed
- unsupported merge classes and contradiction cases must not degrade into
  silent single-parent behavior

## Test And Harness Model

Milestone 9 must follow the same structural testing discipline as earlier
bridge milestones and must satisfy the Milestone 9 certification suites in
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md).

Expected first-class test surfaces:

- ordered-parent determinism scenarios
- supported and unsupported merge-class scenarios
- causally independent versus causally dependent merge scenarios
- policy-resolved versus policy-rejected merge scenarios
- merge-driven deletion and topology rewiring scenarios
- structural-contradiction and continuity-denial scenarios
- replay parity and replay drift scenarios
- ontology-lowering parity and version-mismatch scenarios
- diagnostics-tier invariance scenarios
- counter certification scenarios

Milestone 9 is not complete with only direct fixture tests. It must establish a
real merge-aware certification surface on top of `worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for merge-history matrices
- `ExecutionRequest` for declaration validation, contract admission, lowering,
  routing, continuity, remap, explanation, replay, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, unsupported-class, and
  topology-heavy sweeps
- `ParitySuite` for run-to-run and adapter-to-adapter parity
- `CertificationMatrix` for parent-order determinism, unsupported merge denial,
  and explanation parity

Required suite alignment:

- Suite 10 must emit `merge_history_digest`, `parent_order_report`,
  `routing_digest`, and `replay_digest`
- Suite 11 must emit `merge_support_matrix`, `failure_digest`,
  `diagnostics_digest`, and `counter_snapshot`
- Suite 12 must emit `merge_history_digest`, `continuity_digest`,
  `explanation_digest`, and `replay_digest`

Certification for Milestone 9 must also include:

- typed-failure lanes for `MergeParentOrderDrift` and
  `MergeCausalFrontierTruncated` where applicable
- exact counter assertions proving default plans do not widen into whole-branch
  or whole-history scans
- exact counter assertions proving discovery work does not silently widen before
  lowering begins
- parity assertions showing that equivalent canonical merge artifacts preserve
  meaning while differing merge-class semantics versions fail explicitly

Minimum representative test names:

- `tests::merge::ordered_parent_history_remains_deterministic_under_adapter_variation`
- `tests::merge::unsupported_merge_classes_fail_without_branch_reconciliation_fallback`
- `tests::merge::merge_aware_continuity_consumes_canonical_merge_authority`
- `tests::merge::merge_replay_preserves_routing_and_explanation_parity`
- `tests::merge::structural_similarity_cannot_override_merge_denial`

## Target API And Module Plan

### Public Surface Growth

Milestone 9 should extend the facade with bridge-owned merge types such as:

```rust
pub struct MergeHistoryDeclaration { ... }
pub struct ValidatedMergeHistoryDeclaration { ... }
pub struct AdmittedMergeHistoryContract { ... }
pub struct LoweredMergeHistoryPacketSet { ... }
pub struct ReducedMergeRoutingArtifact { ... }
pub struct PublishedMergeContinuityArtifact { ... }
pub struct PublishedMergeRemapArtifact { ... }
pub struct PublishedMergeExplanationArtifact { ... }
pub struct MergeReplayCertificationBundle { ... }

impl RuntimeBridge {
    pub fn admit_merge_history(
        &self,
        declaration: MergeHistoryDeclaration,
    ) -> Result<AdmittedMergeHistoryContract, BridgeMergeError>;

    pub fn lower_merge_history(
        &self,
        contract: AdmittedMergeHistoryContract,
    ) -> Result<LoweredMergeHistoryPacketSet, BridgeMergeError>;

    pub fn replay_merge_history(
        &self,
        contract: AdmittedMergeHistoryContract,
    ) -> Result<MergeReplayCertificationBundle, BridgeMergeError>;
}
```

Design rules:

- the facade exposes bridge merge concepts only
- it does not expose raw relational merge-planning internals as the public
  contract
- admission, lowering, publication, and replay remain separate boundary
  crossings
- callers must not be able to trigger merge-aware whole-history traversal
  through a getter-shaped convenience API
- schema/runtime registrations choose admitted merge classes by default;
  call-site overrides, if admitted at all, must remain explicit and typed

### New Files Expected

- `crates/worth-runtime-bridge/src/merge/mod.rs`
- `crates/worth-runtime-bridge/src/merge/declaration.rs`
- `crates/worth-runtime-bridge/src/merge/contracts.rs`
- `crates/worth-runtime-bridge/src/merge/history.rs`
- `crates/worth-runtime-bridge/src/merge/lowering.rs`
- `crates/worth-runtime-bridge/src/merge/routing.rs`
- `crates/worth-runtime-bridge/src/merge/continuity.rs`
- `crates/worth-runtime-bridge/src/merge/remapping.rs`
- `crates/worth-runtime-bridge/src/merge/explanation.rs`
- `crates/worth-runtime-bridge/src/diagnostics/merge.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/merge_parent_order.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/unsupported_merge.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/merge_replay.rs`
- `crates/worth-runtime-bridge/src/tests/merge/ordering.rs`
- `crates/worth-runtime-bridge/src/tests/merge/denial.rs`
- `crates/worth-runtime-bridge/src/tests/merge/continuity.rs`
- `crates/worth-runtime-bridge/src/tests/merge/replay.rs`

### Existing Files Expected To Change

- [facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade.rs)
- [lib.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/lib.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/diagnostics/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/harness/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/source/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/historical/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/structural/mod.rs)

## Implementation Phases

Milestone 9 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished ordered-history or merge-admission
foundations with host-local merge heuristics.

Implementation order is itself part of the safety contract for this milestone.
The build must not begin with convenience routing or explanation surfaces and
retrofit provenance, precedence, and replay later.

Required implementation order:

1. canonical merge ontology mapping and admitted contract identity
2. merge-aware continuity and denial precedence pipeline
3. canonical merge result bundle and replay certification surfaces
4. routing, remap publication, explanation reconstruction, and harness
   expansion on top of those foundations

If implementation pressure tries to invert this order, the milestone is
drifting toward heuristic behavior.

### Phase M9.0 - Merge Taxonomy And Ordered Parent Authority Lock

Purpose:

- define the merge-history declaration surface
- define the closed merge-class and authority vocabulary
- lock ordered parent meaning beneath truth authority

Required work:

- define `MergeHistoryDeclaration`
- define merge-class vocabulary
- define the one canonical bridge-to-relational ontology mapping surface
- define ontology-version, policy-descriptor-version, and parent-order-digest
  identity fields on admitted contracts
- define explicit parent-order proof surfaces
- define typed unsupported-merge and parent-order failures

Exit criteria:

- merge-aware history is a singular explicit bridge concept
- parent order is named and load-bearing rather than implicit
- merge authority remains unambiguous

### Phase M9.1 - Admission, Lowering, And Merge-Aware Routing

Purpose:

- resolve merge applicability and packet breadth before publication

Required work:

- define `ValidatedMergeHistoryDeclaration`
- define `AdmittedMergeHistoryContract`
- define `LoweredMergeHistoryPacketSet`
- encode the continuity and denial precedence chain as explicit typed stages or
  proof-bearing phase outputs rather than open-ended bridge judgment
- define exact parent-order digest basis
- define `ReducedMergeRoutingArtifact`
- add exact counters and decision-log records

Exit criteria:

- identical merge declarations and truth artifacts lower to identical packet
  plans
- supported, denied, and contradictory cases reduce canonically
- merge-aware routing remains planned rather than ad hoc

### Phase M9.2 - Merge-Aware Continuity, Advisory Remap, And Explanation Publication

Purpose:

- publish merge outcomes without promoting the bridge to merge authority

Required work:

- define `MergeReplayCertificationBundle`
- define `PublishedMergeContinuityArtifact`
- define `PublishedMergeRemapArtifact`
- define `PublishedMergeExplanationArtifact`
- define explicit publication rules for supported, denied, and contradictory
  merge outcomes
- preserve structural advisory-only status in all publication paths
- define canonical merge result bundle contents for replay certification

Exit criteria:

- merge publication is explicit and replay-safe
- continuity and remap remain subordinate to canonical merge authority
- deletion and topology-rewire merges do not fabricate continuity accidentally
- replay can certify the entire merge result bundle rather than explanation
  alone

### Phase M9.3 - Replay And Certification

Purpose:

- make merge-aware claims certifiable rather than plausible

Required work:

- add canonical merge replay records
- add `worth-harness` fixtures, parity suites, and certification matrices for
  suites 10 through 12
- add hostile unsupported-class, parent-order drift, causal truncation, and
  topology-rewire lanes
- add exact counter assertions for representative merge-aware scenarios

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- replay validates merge-aware routing and explanation parity directly
- denial, contradiction, and parent-order behavior are auditable from canonical
  bundles alone

## Anti-Patterns Explicitly Rejected

- treating ordered parent lists as sets or convenience metadata
- reconstructing merge meaning from branch topology or patch coincidence when
  canonical merge artifacts exist
- selecting continuity winners from structural likeness when merge authority
  denied continuity
- degrading unsupported merge classes into single-parent approximation
- re-running merge reasoning during replay instead of consuming canonical
  records
- hiding merge-aware whole-history work behind convenience getters
- letting host-specific merge helpers become the public bridge contract

## Sequencing Notes

Milestone 9 must land before:

- Milestone 10 speculative truth-branch to signal-branch coordination, because
  speculative branch coordination needs canonical merge-aware history
  consumption before it can safely reason about discard and commit
- Milestone 13 bridge certification, because the bridge is not certifiable
  while merge-bearing history interpretation remains heuristic or adapter-shaped

Milestone 9 builds directly on:

- Milestone 3 continuity foundations, which explicitly deferred general
  merge-aware continuity
- Milestone 6 stream protocol foundations, which made multi-parent stream
  material replay-safe
- Milestone 7 source protocol foundations, which made truth-backed history
  reads capability-explicit
- Milestone 8 structural ambiguity foundations, which prevent structural
  similarity from quietly becoming merge authority

Future-proofing rules for implementation:

- do not duplicate merge ontology lowering across routing, continuity, replay,
  and diagnostics modules; one canonical lowering surface must feed all of them
- do not allow explanation-first implementation to become the de facto replay
  model; replay must certify canonical result bundles from the beginning
- do not postpone discovery counters; causal-frontier lookup width, lineage
  resolution width, structural consult width, candidate cohort width, and
  history segment scan count must exist in the first hot-path implementation
- do not broaden the supported merge-class set for convenience; unsupported
  classes should remain fail-closed until their full authority, locality, and
  replay semantics are implemented
- do not let deletion or topology-rewire classes fall back to branch-wide
  invalidation by default; any widened fallback must be explicit debt with
  separate counters and typed markings

Milestone 9 must not attempt to pre-solve:

- speculative preview coordination
- cross-runtime policy provenance
- bridge-mediated writeback
- new merge ontology beyond what truth authority already defines

Those become stronger because Milestone 9 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the roadmap explicitly requires merge-bearing history
consumption, and the bridge is not production-grade while merge meaning still
depends on host-local procedure or single-parent assumptions.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of silently dropping parent order, re-inventing merge meaning from patch
shape, or treating unsupported merge classes as "close enough" reconciliation.

The milestone preserves authority boundaries because truth still owns merge
ontology, merge execution, causal metadata, and schema-declared merge policies;
signal still owns derived identity and execution; and the bridge owns only the
consumption, lowering, explanation, and replay-safe artifacts between them.

The milestone defines proof obligations rather than implementation chores
because canonical ordered-parent identity, typed merge-class denial,
merge-aware continuity/remap artifacts, replay-safe explanation, and
certification suites 10 through 12 are all required for closeout.

A competent engineer should be able to map this spec into honest declaration
types, merge modules, counters, diagnostics, and harness suites without
inventing the architecture during implementation.

## Closeout Standard

Milestone 9 is complete only when all of the following are true:

- merge-bearing history lowers through one canonical bridge declaration and
  contract surface
- ordered parent lists remain canonical through ingestion, lowering,
  publication, and replay
- merge class, causal frontier, schema-policy outcome, and resulting bridge
  artifacts remain structurally distinct
- unsupported merge classes fail explicitly and replay-safely
- structural identity remains advisory and never overrides merge authority
- merge-aware continuity and remap remain deterministic and explainable under
  hostile replay and adapter variation
- explanation surfaces are derived from canonical merge-aware records rather
  than live adapter queries
- harness certification proves parent-order determinism, unsupported merge
  denial, and replay/explanation parity under hostile conditions

If code lands but ordered multi-parent history is still normalized away,
unsupported merge classes still degrade into heuristic reconciliation,
bridge merge classes still drift semantically from relational merge ontology,
merge-aware continuity still depends on structural convenience over canonical
merge authority, replay still re-runs merge reasoning against ambient truth, or
explanations are the only place merge causality can be understood, Milestone 9
is not complete.
