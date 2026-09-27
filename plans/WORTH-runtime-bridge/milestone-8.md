# Milestone 8 Engineering Spec: Structural-Identity-Aware Remapping

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-7.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-7.md)
>
> **Prior closeout:** [milestone-7-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-7-closeout.md)
>
> **Bridge certification companion:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
>
> **Primary architectural driver:** make structural fingerprints and structural comparison a first-class bridge proof surface for remapping, reuse, and branch comparison without ever allowing structural similarity to become identity authority
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

Milestones 1 through 7 established that the bridge already has strong
authoritative inputs and strong public read/write protocol boundaries:

- canonical truth changes enter through committed bridge-owned envelopes
- aspect routing, continuity, historical evaluation, and branch-aware truth
  selection are explicit proof chains
- wide bridge workloads lower through planned packet sets and deterministic
  reduction artifacts
- stream consumption is a canonical protocol rather than a host-shaped feed
- truth-backed source reads are a canonical protocol rather than adapter
  folklore

That is enough to make the bridge explicit about identity, routing, history,
streaming, and source access.

It is not enough to make the bridge explicit about structural likeness.

Without Milestone 8, the bridge still risks two equally bad failure modes:

- ignore structural identity entirely and lose safe remapping and comparison
  power the roadmap explicitly wants
- treat structural similarity as a quiet substitute for identity and fabricate
  continuity, reuse, or branch-equality claims the runtimes never authorized

Milestone 8 exists because the bridge now has the exact prerequisites needed to
reason about structure honestly:

- Milestone 3 already made lineage and continuity explicit
- Milestone 4 already made branch and historical truth-view selection explicit
- Milestone 5 already made wide comparison and reduction work explicit
- Milestone 7 already made source-backed structural reads explicit and
  replay-safe

The bridge must now be able to say:

`this exact admitted structural comparison contract, over this exact truth-view basis and this exact structural fingerprint basis, found this exact match set, classified ambiguity this exact way, produced this exact advisory remap or branch-comparison artifact, and remained subordinate to authoritative identity the entire time`

not:

`the bridge found something that looked similar enough and treated it as the same thing`

## Goal

Make structural identity a deterministic, replay-safe, bridge-owned advisory
surface for remapping, reuse, and branch comparison without collapsing truth
identity or signal identity into one fused namespace.

## Why This Milestone Exists

Milestone 8 belongs immediately after Milestone 7 because structural remapping
and branch comparison need a trustworthy source protocol before structural
evidence can be used honestly.

Milestone 7 established:

- canonical source declaration identity
- explicit truth-view authority basis
- capability-explicit historical and branch reads
- packetized source planning and replay-safe source records

Milestone 8 now needs to establish the matching structural truths:

- canonical structural declaration identity
- explicit structural fingerprint vocabulary
- explicit ambiguity and mismatch classification
- explicit advisory remap and branch-comparison artifacts

If Milestone 8 shipped before Milestone 7, structural comparison would still
depend on adapter-shaped read paths and ambient capability folklore. That would
make structural identity impossible to certify.

Milestone 8 also belongs before Milestone 9 because merge-aware bridge
semantics will increase the number of structurally plausible candidate
histories. The bridge needs a canonical way to say "these are structurally
similar but not identity-authoritative" before merge-bearing histories can be
consumed safely.

Milestone 8 therefore earns its place in the roadmap by solving the next real
structural problem after source protocol productization: advisory structural
identity that is useful, explicit, and non-authoritative.

## Adversarial Constraint

Milestone 8 must survive the following hostile condition:

> A long-lived system with branch-local truth histories, historical replay,
> replacement and restore flows, same-shape-different-authority entities, near
> matches, ambiguous matches, oscillating branch drift, wide structural
> comparison sets, and diagnostics tiers that vary by environment must produce
> the same structural match classification, the same ambiguity judgment, the
> same advisory remap or branch-comparison artifact, and the same replay result
> every time, while never allowing structural likeness to override lineage,
> truth identity, source authority basis, or derived node identity ownership.

Concretely, the design must remain correct when all of the following are true:

- several candidates are structurally identical but semantically distinct
- one candidate is an exact structural match but carries a different
  authoritative truth identity
- replacement and restore flows produce highly similar shapes across time
- branch-local divergence creates near-match and oscillating-match histories
- unrelated publication occurs between original comparison and replay
- diagnostics richness changes between environments
- wide structural comparison needs packetization and reduction rather than
  scalar candidate loops
- continuity and structural reuse are both available, but continuity remains
  authoritative and structural reuse remains only advisory

If any supported path:

- silently picks one candidate from an ambiguous structural match set
- lets structural likeness fabricate identity continuity
- lets branch comparison drift because unrelated publication changed host
  iteration order
- fuses truth identity and signal identity through a shared structural handle
- cannot replay the same structural judgment from canonical bridge records
- hides ambiguity or mismatch inside diagnostics-only text rather than typed
  bridge artifacts

then Milestone 8 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- structural identity is a bridge-owned advisory protocol, not a replacement
  for truth identity or signal identity
- structural fingerprints, structural match sets, ambiguity classifications,
  and structural remap artifacts are distinct concepts and must remain distinct
  types
- lineage continuity remains authoritative when lineage exists; structural
  evidence may refine or reject advisory reuse but may not override canonical
  lineage truth
- exact structural matches are still subordinate to authoritative identity
- ambiguity is a first-class outcome, not a fallback to best-score winner
- branch comparison is a bridge artifact with explicit truth-view basis and
  explicit structural diff basis, not a host-local convenience query
- replay must consume canonical structural records rather than re-running
  ad hoc host comparisons against whatever truth happens to be available later
- diagnostics richness may change explanation detail, but not structural match
  meaning, ambiguity classification, or remap outcome
- Milestone 8 productizes structural remapping and branch comparison only; it
  does not productize merge-aware truth consumption, policy propagation, or
  writeback

Normative consequence:

- public APIs that expose "best structural match" without ambiguity typing are
  out of spec
- identity continuity inferred solely from structural fingerprints is out of
  spec
- host adapters that compute their own structural winner outside bridge-owned
  contracts are out of spec
- branch comparison surfaces that depend on ambient latest truth are out of
  spec
- diagnostics-only descriptions of ambiguity without canonical bridge records
  are out of spec
- replay or reuse keyed only by raw digest without structural-semantics version,
  truth-view basis, and branch basis is out of spec

## Configuration And Defaults

Milestone 8 should expose only a small set of explicit structural
configuration surfaces. Structural authority boundaries and ambiguity policy are
not configurable.

### Admitted Configurable Surfaces

- structural fingerprint family selection
  - default: schema-scoped declaration at registration time rather than
    call-site override
- structural comparison mode
  - default: `AdvisoryRemap`
- truth-view basis
  - default for remap: explicit current snapshot
  - default for branch comparison: explicit branch-pair basis
- candidate search scope
  - default: cohort-bounded scope derived from declared structural indexes,
    lineage neighborhoods, or explicit branch-local cohorts
- diagnostics richness
  - default: structured standard diagnostics, not maximum forensic retention
- replay retention richness for structural records
  - default: retain enough canonical structural evidence to replay
    classification and explain ambiguity without ambient truth lookup
- reuse publication policy
  - default: publish advisory reuse only when unambiguous and
    identity-authority-safe

### Non-Configurable Surfaces

- structural identity overriding authoritative truth identity
  - default: never admitted
- ambiguity auto-resolving to a winner
  - default: never admitted
- ambient latest-truth branch comparison
  - default: never admitted
- host-local custom scoring semantics outside bridge-owned contracts
  - default: never admitted

The bridge should therefore feel configurable at the schema and declaration
layer, but closed and fail-safe at the authority boundary.

## Guideline Influence

### 1. `MENTALITY.md`

This document directly shapes the milestone:

- adversarial constraint first:
  the spec starts from ambiguity, same-shape-different-authority collisions,
  drift, and replay hostility rather than from the pleasant feature phrasing of
  "use structural identity to help remap"
- solve the hard problem first:
  structural ambiguity classification and authority boundaries ship before
  convenience reuse or richer comparison ergonomics
- enforce mechanically, not by convention:
  ambiguity, mismatch, reuse, and advisory-remap legality must be represented
  by proof-bearing types and typed failures, not comments
- spec is architecture is code:
  the spec names exact proof chains, subdomains, counters, and failure classes
  that should map directly into the crate
- authority first, derivation second:
  truth identity and lineage remain authoritative; structural fingerprints and
  structural remap records are rebuildable bridge artifacts
- separate what/how/whether:
  structural similarity is the `what`, remap/comparison planning is the `how`,
  and retained diagnostics richness is the `whether`

### 2. `architectural_guidelines.md`

This document determines the structural boundaries:

- Laws 7, 8, and 32:
  every structural comparison boundary must emit self-describing artifacts,
  decision traces, and counters rather than opaque match results
- Laws 16, 18, 30, and 41:
  structural comparison must move through a proof chain such as declaration,
  validation, admission, planning, reduction, and artifact publication
- Laws 21 and 33:
  structural diagnostics and advisory remap records are derived artifacts with
  lifecycles separate from authoritative truth and authoritative lineage
- Laws 26 and 40:
  equivalence contracts and naming must be explicit; "same shape" cannot remain
  an ambient intuition
- Law 29:
  abstraction must stop before it hides correctness or cost boundaries, so
  exact match, ambiguous match, mismatch, and branch diff outcomes cannot be
  collapsed into one generic comparison result

### 3. `domain_standards.md`

This document determines crate decomposition:

- structural identity work must live in a dedicated subsystem with
  responsibilities such as `structural/declaration`, `structural/fingerprints`,
  `structural/matching`, `structural/remapping`, and `structural/comparison`
- branch comparison is not the same responsibility as advisory remapping
- ambiguity classification is not the same responsibility as diagnostics
  rendering
- tests must mirror structural responsibilities rather than collapsing all
  cases into one catch-all remap integration file

### 4. `performance_guidelines.md`

This document determines the cost model:

- wide candidate comparison must lower through planned packetized comparison
  work, not scalar candidate loops hidden behind convenience APIs
- ambiguity rejection must precede expensive remap publication or replay-safe
  artifact emission when possible
- structural comparison APIs must reveal breadth and comparison mode honestly
- counters must explain candidate breadth, ambiguity width, exact-match count,
  mismatch count, and branch-diff breadth
- structural reuse without explicit equivalence contracts is forbidden
- structural comparison hot paths must scale with admitted candidate cohort
  width rather than all visible entities in a branch, snapshot, or retained
  history

## Scope

### In Scope

- one bridge-owned declaration surface for structural remapping and branch
  comparison
- canonical structural fingerprint vocabulary and structural match identity
- bridge-owned ambiguity, mismatch, reuse, and advisory-remap classification
- packetized planning for wide structural candidate comparison
- explicit branch comparison artifacts grounded in admitted truth-view basis
- replay-safe structural records, diagnostics, and explanations
- typed failures and counters for ambiguity, mismatch, drift, and
  identity-fusion attempts
- harness certification for suites 7 through 9 in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)

### Explicitly Out Of Scope

- merge-aware structural interpretation across multi-parent histories
- policy provenance across runtimes
- speculative preview lifecycle
- bridge-mediated writeback or commit strategy production
- signal scheduler semantics
- authority transfer from truth/runtime identity into structural fingerprint
  identity

Milestone 8 must stay focused on advisory structural identity and branch
comparison rather than absorb merge, policy, or writeback work early.

## Governing Design Rules

### 1. Authoritative Identity Always Dominates Structural Identity

The truth runtime defines:

- authoritative truth identity
- lineage semantics
- branch and historical truth-view meaning

The signal runtime defines:

- derived node identity
- downstream execution over whatever remap or comparison artifacts the bridge
  publishes

The bridge defines:

- structural fingerprint contracts
- structural match and ambiguity classification
- advisory remap rules
- branch comparison rules
- replay-safe structural records and diagnostics

The bridge must not redefine authoritative identity. Structural identity is a
comparison aid, not identity authority.

### 1.1 Structural Fingerprint Vocabulary Must Be Closed Per Milestone

Milestone 8 must not leave "structure" as an implementation-defined noun.

For this milestone, the bridge must define a closed vocabulary for structural
fingerprint families, such as:

- `TopologyFingerprint`
- `FacetShapeFingerprint`
- `BranchComparisonFingerprint`
- `RestoreCandidateFingerprint`

Rules:

- each fingerprint family defines identity-bearing fields, explanatory-only
  fields, and digest basis
- each fingerprint family also defines a schema-scoped equivalence contract
  including canonical normalization rules, canonical ordering rules,
  omission/null policy, and a `fingerprint_semantics_version`
- fingerprints are bridge artifacts derived from admitted truth views
- fingerprints may be compared only within an admitted comparison family
- host-local "custom structural score" strings are out of spec for Milestone 8
- replay or reuse across differing `fingerprint_semantics_version` values is a
  typed incompatibility, not best-effort compatibility

### 1.2 Structural Families Must Be Schema-Scoped

Milestone 8 must not force one global definition of structural sameness across
all product domains.

Rules:

- structural fingerprint families are declared within schema or domain context
- geometry kernels, chip simulators, and CRUD-style applications may define
  different admitted fingerprint families while still using the same bridge
  protocol shape
- the bridge owns the declaration, versioning, and replay contract
- the schema/domain owns the meaning of the fingerprint family
- cross-schema structural reuse is rejected unless an explicit shared
  equivalence contract exists

### 2. Declaration, Admission, Matching, And Publication Are Distinct Proof Chains

Milestone 8 must not collapse:

- `StructuralIdentityDeclaration`
- `ValidatedStructuralIdentityDeclaration`
- `AdmittedStructuralComparisonContract`
- `PlannedStructuralMatchPacketSet`
- `ReducedStructuralMatchSet`
- `PublishedStructuralRemapArtifact`
- `PublishedBranchComparisonArtifact`

Rules:

- a declaration is not itself permission to compare any candidates
- admission proves the requested structural family, truth-view basis, and
  comparison mode are legal
- planned packets freeze candidate breadth and comparison ordering before match
  execution
- reduced match sets classify exact, ambiguous, and mismatch outcomes before
  publication
- remap artifacts and branch-comparison artifacts are separate outputs, not one
  generic "comparison result"

### 2.1 Truth-View Basis Must Remain Explicit

Every admitted structural comparison contract must carry one explicit authority
basis inherited from bridge-owned source truth.

For Milestone 8 the closed authority basis vocabulary must include:

- `ExplicitSnapshot`
- `ExplicitHistoricalVersion`
- `ExplicitBranchHead`
- `ExplicitBranchPairComparison`

Rules:

- structural comparison may not silently substitute a different truth view
- replay records must preserve the exact authority basis
- branch comparison must name both sides of the comparison explicitly
- any structural reuse or memoization key must include the full truth-view
  authority basis rather than only a structural digest
- additional freshness or merge-aware truth bases are deferred to later
  milestones

### 3. Ambiguity Is A Canonical Outcome, Not A Fallback

Milestone 8 must not allow "best candidate wins" when more than one candidate
matches the admitted structural basis.

The reduced match set must classify candidates into outcomes such as:

- `ExactAdvisoryMatch`
- `AdvisoryReuseCandidate`
- `AmbiguousStructuralMatch`
- `StructuralMismatch`
- `IdentityAuthorityConflict`

Rules:

- ambiguity must retain the full competing candidate set or its canonical
  digest
- ambiguity publication must be replay-safe
- ambiguous outcomes must block advisory remap publication unless a later
  milestone adds a stronger authority basis explicitly
- diagnostics may enrich ambiguity explanation but may not resolve it

### 3.1 Mixed-Evidence Reduction Must Be Explicit

Milestone 8 must not leave the interaction among lineage truth, multiple
fingerprint families, and branch-comparison evidence to ad hoc implementation
precedence.

The bridge must define a closed evidence-reduction topology:

1. authoritative identity and lineage evidence
2. per-family structural comparison evidence
3. contradiction and ambiguity classification
4. admissible publication class

Required rules:

- authoritative lineage continuity dominates any structural continuity signal
- per-family structural evidence may agree, disagree, or be individually
  ambiguous
- disagreement between authoritative lineage and structural evidence must
  produce a typed contradiction artifact rather than disappearing into
  diagnostics
- final publication may only consume reduced evidence outputs, never raw
  family-local scores or host-local precedence rules

### 4. Advisory Reuse Must Never Fabricate Continuity

Structural reuse may improve remapping and comparison only when the bridge can
publish an explicit advisory artifact that remains subordinate to authoritative
identity.

Rules:

- structural reuse may inform candidate narrowing, advisory reuse, or explicit
  comparison reports
- structural reuse may not mint a new truth identity
- structural reuse may not claim subscription continuity where lineage did not
  already authorize it
- restore flows and branch-local history may compare structurally, but they may
  not fabricate continuity records from comparison alone
- reuse is admitted only through explicit advisory artifacts keyed by
  fingerprint family, `fingerprint_semantics_version`, truth-view basis, and
  branch basis

### 5. Branch Comparison Must Be Deterministic, Local, And Explainable

Milestone 8 must define branch comparison as a first-class bridge output rather
than a convenience query.

Branch comparison must define:

- explicit left/right truth-view basis
- explicit structural diff family
- canonical candidate ordering
- canonical drift classification
- canonical output ordering and digest basis

Rules:

- unrelated publication outside the compared branch pair may not alter the
  comparison outcome
- oscillating near-match histories must remain deterministic across replay
- branch comparison must produce localizable structural diff artifacts rather
  than only one coarse digest

### 6. Replay And Diagnostics Must Consume Canonical Structural Records

Milestone 8 must not allow replay or explanation to depend on re-running a
best-effort host comparison.

Required canonical records must preserve:

- structural declaration identity
- admitted comparison contract identity
- truth-view basis identity
- fingerprint family identity
- candidate set digest
- reduced match-set digest
- published remap or branch-comparison artifact digest
- replay retention anchor or retained structural-evidence basis

Minimum replay-retained structural evidence must include enough information to
reconstruct:

- the admitted fingerprint family and semantics version
- the candidate cohort identity or canonical candidate-set digest
- the reduced ambiguity or mismatch basis
- the truth-view authority basis and retention anchor

Rules:

- digest-only replay records are insufficient if they cannot explain why
  ambiguity or mismatch existed
- retention truncation that invalidates the replay basis must fail typed rather
  than silently recomputing from whatever truth remains available

- replay consumes canonical structural records only
- diagnostics richness changes explanation detail, not classification meaning
- failure and ambiguity records are first-class bridge records, not harness-only
  summaries

## Phases

### Phase 1: Structural Vocabulary And Authority Boundary Lock

Phase 1 exists to make structural identity representable without ambiguity
folklore.

Milestone 8 must first define:

- one structural declaration surface
- closed structural fingerprint families
- explicit structural comparison modes
- exact authority-boundary rules between lineage, authoritative identity, and
  structural identity
- explicit ambiguity, mismatch, and identity-conflict vocabularies

This phase leaves the system in a coherent state where:

- structural identity is discussed through bridge-owned nouns rather than host
  heuristics
- authoritative identity still dominates by construction
- ambiguity is explicit before any remap or comparison publication exists

### Phase 2: Packetized Matching, Reduction, And Advisory Artifact Publication

Phase 2 exists to turn structural vocabulary into deterministic bridge work.

Milestone 8 must then implement:

- admission of structural declarations
- packetized candidate comparison planning
- deterministic reduction into exact, ambiguous, mismatch, and conflict classes
- advisory remap artifact publication
- branch-comparison artifact publication
- exact counters and decision records for candidate breadth and ambiguity width

This phase leaves the system in a coherent state where:

- identical structural inputs lower to identical match packets and reduced match
  sets
- remap and branch-comparison publication consume reduced structural truth only
- wide structural comparison remains cost-honest and replay-safe

Admitted comparison planning in this phase must define a named complexity
contract.

Required complexity rule:

- structural comparison hot paths must scale with admitted candidate cohort
  width, packet width, and reduced ambiguity width
- whole-history, whole-branch, or whole-snapshot scans are not an admitted
  default execution strategy
- widened scans, if ever admitted for a domain, must be explicit debt-bearing
  modes with separate counters and typed plan markings

### Phase 3: Replay, Certification, And Drift Resistance

Phase 3 exists to prove that structural identity is trustworthy instead of
seductive.

Milestone 8 must finally ship:

- replay-safe structural records and explanation reconstruction
- harness certification for structural ambiguity, structural reuse without
  identity fusion, and branch comparison drift
- hostile coverage for same-shape-different-authority collisions, restore
  sequences, and oscillating near-match branches
- exact counter assertions for representative ambiguity, reuse, and drift lanes

This phase leaves the system in a coherent state where:

- structural identity is certifiable as advisory rather than accidental
  authority
- replay validates match classification directly
- Milestone 9 can extend history complexity without reopening structural
  identity rules

## Must Ship

- canonical structural declaration, contract, fingerprint, match-set, remap,
  and branch-comparison artifacts
- schema-scoped structural equivalence contracts with explicit
  `fingerprint_semantics_version`
- typed ambiguity, mismatch, reuse, and identity-conflict outcomes
- typed contradiction artifacts for authoritative-lineage versus structural
  disagreement
- packetized structural comparison planning and deterministic reduction
- replay-safe structural records and explanation surfaces
- counters and decision-log records for candidate breadth, exact-match width,
  ambiguity width, mismatch width, branch-diff breadth, and replay mismatch
- typed failures for unsupported structural family, ambiguous match,
  identity-authority conflict, branch-comparison drift mismatch, replay
  incompatibility, and structural materialization rejection
- harness certification lanes satisfying Milestone 8 suites 7 through 9 in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)

## Must Preserve

- truth runtime remains the authority for truth identity, lineage, and
  truth-view semantics
- signal runtime remains the authority for derived node identity and execution
- structural identity never becomes authoritative identity
- no accidental ID fusion across runtimes
- no silent winner selection from ambiguous structural candidates
- no branch-comparison drift caused by unrelated publication or host ordering
- canonical ordering and replay-safe structural identities
- diagnostics richness changes explanation only, not structural meaning

## Acceptance Evidence

Milestone 8 is complete only when the bridge harness can prove:

- structural identity can assist remapping without overriding authoritative
  truth identity
- exact structural matches remain subordinate to authoritative identity rather
  than replacing it
- ambiguous structural matches are explicit, typed, and replayable
- structural reuse remains advisory and does not fabricate continuity across
  restore or branch-local history
- branch comparison remains deterministic under drift, oscillation, replay, and
  unrelated publication
- explanation surfaces can localize what changed structurally
- the Milestone 8 certification suites in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
  pass with canonical machine-checkable bundles

## Architectural Notes

### Expected Internal Subdomains

Milestone 8 should extend the bridge crate with subdomains such as:

- `structural/declaration/`
- `structural/fingerprints/`
- `structural/contracts/`
- `structural/matching/`
- `structural/reduction/`
- `structural/remapping/`
- `structural/comparison/`
- `diagnostics/structural/`
- `harness/fixtures/structural_ambiguity.rs`
- `harness/fixtures/structural_reuse.rs`
- `harness/fixtures/branch_comparison.rs`

This follows workspace domain standards:

- fingerprint construction is not the same responsibility as candidate matching
- ambiguity reduction is not the same responsibility as remap publication
- branch comparison is not the same responsibility as advisory remap
- diagnostics reconstruction is not the same responsibility as structural replay

### Minimum Counter Floor

Milestone 8 must add counters such as:

- `structural_declaration_count`
- `structural_contract_count`
- `structural_fingerprint_count`
- `structural_match_packet_count`
- `structural_candidate_count`
- `structural_candidate_cohort_count`
- `structural_exact_match_count`
- `structural_ambiguity_count`
- `structural_mismatch_count`
- `structural_identity_conflict_count`
- `structural_lineage_divergence_count`
- `structural_reuse_publication_count`
- `branch_comparison_count`
- `branch_comparison_diff_count`
- `branch_comparison_drift_rejection_count`
- `structural_widened_scan_count`
- `structural_replay_request_count`
- `structural_replay_mismatch_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Structural Failure Policy

Milestone 8 must carry structural failures structurally rather than narratively.

Required failure classes:

- `UnsupportedStructuralFingerprintFamily`
- `StructuralContractMismatch`
- `StructuralAuthorityBasisMismatch`
- `AmbiguousStructuralMatch`
- `IdentityAuthorityConflict`
- `LineageStructuralDivergence`
- `StructuralReplayMismatch`
- `StructuralReplayBasisTruncated`
- `StructuralMaterializationRejected`
- `BranchComparisonDriftDetected`
- `BranchComparisonBasisMismatch`
- `StructuralCandidateOrderingDrift`

Rules:

- every admitted structural request receives exactly one reduced match outcome
  or one typed failure
- failure remains visible in canonical structural truth
- failure must identify the structural boundary that failed
- ambiguity or authority conflict must not degrade into silent winner selection

## Test And Harness Model

Milestone 8 must follow the same structural testing discipline as earlier
bridge milestones and must satisfy the Milestone 8 certification suites in
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md).

Expected first-class test surfaces:

- exact-match, ambiguous-match, and no-match scenarios
- same-shape-different-authority scenarios
- replacement and restore sequence scenarios
- branch drift and oscillation scenarios
- replay parity and replay drift scenarios
- diagnostics-tier invariance scenarios
- counter certification scenarios

Milestone 8 is not complete with only direct fixture tests. It must establish a
real structural-certification surface on top of `worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for structural-candidate matrices
- `ExecutionRequest` for declaration validation, contract admission, packet
  planning, matching, reduction, publication, replay, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, ambiguity-heavy, and drift
  sweeps
- `ParitySuite` for run-to-run and branch-to-branch parity
- `CertificationMatrix` for ambiguous-match hostility, reuse-without-fusion,
  and drift resistance

Required suite alignment:

- Suite 7 must emit `structural_match_digest`, `ambiguity_report`,
  `remap_artifact_digest`, and `failure_digest`
- Suite 8 must emit `structural_reuse_digest`, `identity_separation_report`,
  `replay_digest`, and `diagnostics_digest`
- Suite 9 must emit `branch_compare_digest`, `structural_diff_report`,
  `replay_digest`, and `counter_snapshot`

Certification for Milestone 8 must also include:

- typed-failure lanes for `LineageStructuralDivergence` and
  `StructuralReplayBasisTruncated` where applicable
- exact counter assertions proving that default plans do not widen into
  whole-history or whole-branch scans
- parity assertions showing that equivalent schema-scoped declarations preserve
  meaning while differing `fingerprint_semantics_version` values fail
  explicitly

Minimum representative test names:

- `tests::structural::ambiguous_structural_candidates_fail_without_winner_selection`
- `tests::structural::exact_match_remains_subordinate_to_authoritative_identity`
- `tests::structural::structural_reuse_does_not_fabricate_continuity`
- `tests::structural::branch_comparison_remains_deterministic_under_drift`
- `tests::structural::replayed_structural_match_records_preserve_original_classification`

## Target API And Module Plan

### Public Surface Growth

Milestone 8 should extend the facade with bridge-owned structural types such as:

```rust
pub struct StructuralIdentityDeclaration { ... }
pub struct ValidatedStructuralIdentityDeclaration { ... }
pub struct AdmittedStructuralComparisonContract { ... }
pub struct PlannedStructuralMatchPacketSet { ... }
pub struct ReducedStructuralMatchSet { ... }
pub struct PublishedStructuralRemapArtifact { ... }
pub struct PublishedBranchComparisonArtifact { ... }

impl RuntimeBridge {
    pub fn admit_structural_comparison(
        &self,
        declaration: StructuralIdentityDeclaration,
    ) -> Result<AdmittedStructuralComparisonContract, BridgeStructuralError>;

    pub fn compare_structure(
        &self,
        contract: AdmittedStructuralComparisonContract,
    ) -> Result<ReducedStructuralMatchSet, BridgeStructuralError>;
}
```

Design rules:

- the facade exposes bridge structural concepts only
- it does not expose raw host structural heuristics as the public contract
- admission, comparison, and publication remain separate boundary crossings
- callers must not be able to trigger wide candidate comparison through a
  getter-shaped convenience API
- schema/domain registrations choose fingerprint families by default; call-site
  overrides, if admitted at all, must remain explicit and typed

### New Files Expected

- `crates/worth-runtime-bridge/src/structural/mod.rs`
- `crates/worth-runtime-bridge/src/structural/declaration.rs`
- `crates/worth-runtime-bridge/src/structural/fingerprints.rs`
- `crates/worth-runtime-bridge/src/structural/contracts.rs`
- `crates/worth-runtime-bridge/src/structural/matching.rs`
- `crates/worth-runtime-bridge/src/structural/reduction.rs`
- `crates/worth-runtime-bridge/src/structural/remapping.rs`
- `crates/worth-runtime-bridge/src/structural/comparison.rs`
- `crates/worth-runtime-bridge/src/diagnostics/structural.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/structural_ambiguity.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/structural_reuse.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/branch_comparison.rs`
- `crates/worth-runtime-bridge/src/tests/structural/ambiguity.rs`
- `crates/worth-runtime-bridge/src/tests/structural/reuse.rs`
- `crates/worth-runtime-bridge/src/tests/structural/comparison.rs`
- `crates/worth-runtime-bridge/src/tests/structural/replay.rs`

### Existing Files Expected To Change

- [facade.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/facade.rs)
- [lib.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/lib.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/diagnostics/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/harness/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/source/mod.rs)
- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/historical/mod.rs)

## Implementation Phases

Milestone 8 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished structural-authority foundations with
host-local similarity heuristics.

### Phase M8.0 - Structural Taxonomy And Authority Lock

Purpose:

- define the structural declaration surface
- define the closed fingerprint and outcome vocabulary
- lock advisory structural identity beneath authoritative identity

Required work:

- define `StructuralIdentityDeclaration`
- define structural fingerprint family vocabulary
- define explicit ambiguity and identity-conflict outcomes
- define structural versus authoritative identity boundary rules

Exit criteria:

- structural identity is a singular explicit bridge concept
- ambiguity is named rather than deferred
- identity authority remains unambiguous

### Phase M8.1 - Admission, Packet Planning, And Match Reduction

Purpose:

- resolve applicability and candidate breadth before structural publication

Required work:

- define `ValidatedStructuralIdentityDeclaration`
- define `AdmittedStructuralComparisonContract`
- define `PlannedStructuralMatchPacketSet`
- define exact candidate ordering and digest basis
- define `ReducedStructuralMatchSet`
- add exact counters and decision-log records

Exit criteria:

- identical declarations and truth-view inputs lower to identical packet plans
- ambiguity, mismatch, and conflict are reduced canonically
- wide structural comparison remains planned rather than ad hoc

### Phase M8.2 - Advisory Remap And Branch Comparison Publication

Purpose:

- publish structural outcomes without promoting them to authority

Required work:

- define `PublishedStructuralRemapArtifact`
- define `PublishedBranchComparisonArtifact`
- define explicit publication rules for exact advisory match, ambiguity, and
  mismatch
- preserve identity separation in all publication paths

Exit criteria:

- structural publication is explicit and replay-safe
- remap and comparison artifacts remain subordinate to authoritative identity
- restore and branch-local history do not fabricate continuity

### Phase M8.3 - Replay And Certification

Purpose:

- make structural claims certifiable rather than plausible

Required work:

- add canonical structural replay records
- add `worth-harness` fixtures, parity suites, and certification matrices for
  suites 7 through 9
- add hostile same-shape-different-authority and branch-drift lanes
- add exact counter assertions for representative ambiguity and drift scenarios

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- replay validates structural classification parity directly
- ambiguity, reuse, and branch comparison behavior are auditable from canonical
  bundles alone

## Explicit Failure Taxonomy For Milestone 8

Milestone 8 must ship typed bridge failures for at least:

- unsupported structural fingerprint family
- structural contract mismatch
- structural authority basis mismatch
- ambiguous structural match
- identity authority conflict
- structural candidate ordering drift
- branch comparison basis mismatch
- branch comparison drift detected
- structural materialization rejection
- structural replay mismatch

These are bridge failures, not raw parent-runtime strings.

## Anti-Patterns Explicitly Rejected

- treating structural similarity as identity authority
- selecting one ambiguous candidate by score or iteration order
- publishing continuity from structural reuse alone
- comparing branches through ambient latest truth rather than explicit
  truth-view basis
- re-running structural judgment during replay instead of consuming canonical
  records
- hiding wide candidate comparison behind convenience getters
- letting host-specific similarity heuristics become the public bridge contract

## Sequencing Notes

Milestone 8 must land before:

- Milestone 9 merge-aware bridge semantics, because merge-bearing histories will
  multiply structurally plausible candidates and need a strong ambiguity model
- Milestone 13 bridge certification, because the bridge is not certifiable
  while structural remapping and branch comparison remain heuristic or
  host-shaped

Milestone 8 must not attempt to pre-solve:

- merge ontology
- speculative preview coordination
- cross-runtime policy provenance
- writeback or commit strategy surfaces

Those become stronger because Milestone 8 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the roadmap explicitly needs structural remapping, reuse,
and branch comparison, and those capabilities are unsafe unless the bridge can
prove that structural likeness stays advisory.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of turning "looks the same" into quiet identity fusion or drift-prone
comparison folklore.

The milestone preserves authority boundaries because truth still owns identity,
lineage, and truth-view meaning, signal still owns derived identity and
execution, and the bridge owns only structural comparison, advisory remap, and
replay-safe structural artifacts between them.

The milestone defines proof obligations rather than implementation chores
because canonical fingerprint vocabulary, typed ambiguity, replay-safe
structural records, branch-comparison determinism, and certification suites 7
through 9 are all required for closeout.

A competent engineer should be able to map this spec into honest structural
types, matching modules, counters, diagnostics, and harness suites without
inventing the architecture during implementation.

## Closeout Standard

Milestone 8 is complete only when all of the following are true:

- structural identity lowers through one canonical bridge declaration and
  comparison contract surface
- structural fingerprints, match sets, remap artifacts, and branch-comparison
  artifacts remain structurally distinct
- exact structural matches remain subordinate to authoritative identity
- ambiguous matches fail explicitly and replay-safely
- structural reuse remains advisory and never fabricates continuity
- branch comparison remains deterministic, local, and explainable under drift
- structural truth is replay-safe and diagnostics-tier-invariant
- harness certification proves ambiguity handling, reuse without identity
  fusion, and branch-comparison drift resistance under hostile conditions

If code lands but structural similarity still acts as quiet identity authority,
ambiguous candidates still produce winners, branch comparison still drifts under
replay or unrelated publication, or explanations are the only place structural
truth can be understood, Milestone 8 is not complete.
