# Milestone 6 Engineering Spec: Branch-Scoped, Historical, And Diff Query Contexts

> **Status:** Draft engineering spec
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-5.6.md](./milestone-5.6.md)
>
> **Adjacent milestones:** [milestone-5.2.md](./milestone-5.2.md),
> [milestone-5.4.md](./milestone-5.4.md), and [milestone-5.5.md](./milestone-5.5.md)
> are already closed and remain authority-distinct inputs for preview-derived
> basis identity, historical materialization-path honesty, and workflow-aware
> branch context composition.
>
> **Prior closeout:** [milestone-5.6-closeout.md](./milestone-5.6-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make branch basis, historical basis, and
> diff/comparison basis first-class query-owned artifacts so the same canonical
> query shape can move across current, branch, historical, and comparison truth
> bases without host repair, basis substitution, or result-shape drift
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
> - [milestone-5.4.md](./milestone-5.4.md)
> - [milestone-5.4-closeout.md](./milestone-5.4-closeout.md)
> - [milestone-5.5.md](./milestone-5.5.md)
> - [milestone-5.5-closeout.md](./milestone-5.5-closeout.md)
> - [milestone-5.6.md](./milestone-5.6.md)
> - [milestone-5.6-closeout.md](./milestone-5.6-closeout.md)

## Goal

Make branch-scoped, historical, and diff query contexts first-class
query-owned artifacts so the same canonical query shape can execute against
current truth, alternate branch truth, admitted historical truth, and typed
comparison truth without changing semantic meaning apart from the explicitly
declared basis.

## Why This Milestone Exists

Milestone 5 made live query meaning survive change. Milestone 5.1 made
region-scoped live narrowing and stream-contract lowering explicit. Milestone
5.2 made preview-session basis identity explicit. Milestone 5.3 made
frontier-aware planning and deterministic parallel admission planner-owned.
Milestone 5.4 made correspondence and historical materialization-path honesty
explicit. Milestone 5.5 made workflow lowering query-native without turning
`worth-query` into a second mutation engine. Milestone 5.6 made the unified
application facade and unified runtime configuration production-honest.

Those milestones solved how query meaning is authored, validated, planned,
maintained, compared in preview/workflow contexts, and surfaced through one
developer-facing facade. They did not yet solve the broader basis problem:

- how the same declared query shape targets current branch head versus another
  branch head
- how the same declared query shape targets historical truth without becoming
  "history mode" host glue
- how a diff/comparison query expresses two explicit truth bases rather than
  one opaque comparison API
- how current, branch, historical, and diff reads preserve the same declared
  result-shape meaning rather than drifting into basis-specific output families

If this milestone does not freeze those contracts now:

- branch reads, time-travel reads, and diff reads will fracture into separate
  helper APIs instead of staying inside one canonical query model
- hosts will start repairing basis mismatches, reconstructing history, or
  substituting branch heads locally precisely where `worth-query` is supposed
  to stay truth-honest
- historical materialization-path metadata from Milestone 5.4 will be present
  but not actually integrated into ordinary query context semantics
- Milestone 7 lineage/correspondence work will inherit soft basis semantics
  instead of building on explicit branch/history/diff contracts
- Milestone 8 view-shape work will be tempted to define branch comparison and
  historical result variation at the presentation layer instead of in the
  query context layer

Milestone 6 therefore exists to freeze:

- that branch basis, historical basis, and comparison basis are query-owned
  artifacts, not host conventions
- that the same canonical query shape preserves the same structural meaning
  across basis variants
- that diff outputs are query-shaped comparison artifacts rather than raw
  storage deltas or arbitrary controller summaries
- that basis identity and materialization-path identity remain explicit
  through execution and result shaping
- that unsupported or store-gated basis families fail typed and early rather
  than widening into "close enough" branch/history behavior

## Governing Summaries

- `MENTALITY.md`: the hard problem is not "support history" or "support diff."
  It is making basis variation survive hostile branch/history/diff pressure
  without letting hosts repair meaning after the fact. The milestone must
  solve basis honesty first.
- `arch_laws.md`: Laws 7, 9, 17, 20, 24, 27, 30, 33, 40, and 41 dominate this
  milestone. Basis identity, historical admission, comparison shaping, and
  execution artifacts must be self-describing, proof-bearing, and planner-owned
  instead of rediscovered by execution or hosts.
- `perf_laws.md`: branch and history support is only honest if basis lookup,
  diff breadth, comparison width, and historical admission breadth are made
  mechanically visible. Cheap-looking basis helpers must not conceal rescans,
  history reconstruction, or raw-storage delta discovery.
- `domain_laws.md`: query-context basis binding, historical admission,
  comparison shaping, metadata/result shaping, execution, and certification are
  separate responsibilities and must not collapse into one "history" or
  "comparison" god module.
- `worth_query_vision.md`: branch-scoped reads, time-travel reads, and diff
  queries are central to the product thesis. Milestone 6 is where those
  capabilities become ordinary query contexts instead of special-case host APIs.
- `worth_query_roadmap.md`: Milestone 6 belongs immediately after the closed
  5.6 facade/configuration milestone and before lineage/view-shape/store parity
  work. It is the contract layer for basis variation, not a catch-all for later
  historical or comparison polish.
- `test-requirements.md`: the `Historical / Diff / Basis Parity Test` is the
  closeout proof. It requires current, branch, historical, and diff lanes to
  preserve the same canonical query meaning apart from their declared basis.
- `milestone-5.2.md` and `milestone-5.2-closeout.md`: preview basis identity is
  already query-native and must compose with, not redefine, the broader basis
  vocabulary for branch and historical contexts.
- `milestone-5.4.md` and `milestone-5.4-closeout.md`: historical
  materialization-path identity and correspondence honesty are already frozen.
  Milestone 6 must consume those explicit lower-runtime/historical proof
  surfaces rather than inventing new hidden history semantics.
- `milestone-5.5.md` and `milestone-5.5-closeout.md`: workflow declarations and
  authority-boundary lowering are already basis-explicit. Milestone 6 must
  provide the basis contexts those workflows can later inspect and compare
  against without turning comparison into workflow-only logic.
- `milestone-5.6.md` and `milestone-5.6-closeout.md`: the unified application
  facade is already closed and production-ready. Milestone 6 must compose
  through that daily-driver surface rather than inventing a second application
  admission layer.

## Adversarial Constraint

Milestone 6 must survive the following hostile condition:

> The same canonical query shape is executed against current branch truth,
> alternate branch truth, admitted historical truth, preview-derived truth,
> and typed diff/comparison truth; every admitted lane must preserve identical
> query meaning apart from its declared basis, explicit materialization-path
> identity, and explicit comparison semantics, without host repair, basis
> substitution, raw storage delta leakage, or result-shape drift.

Concretely, the design must remain correct when all of the following are true:

- a developer uses one application-facing facade and one canonical query shape
  to move among:
  - current branch head
  - alternate branch head
  - historical commit or snapshot basis
  - preview-derived basis where admitted
  - diff/comparison between two already-declared bases
- some historical lanes are runtime-backed and admitted while store-backed
  point-in-time restore remains explicitly deferred
- some basis pairings are valid and some must fail because they would require
  raw storage delta reconstruction, broad rescans, or basis substitution
- result shapes, policy masks, and future view-shape semantics must remain
  stable even when the basis changes
- a naive implementation would be tempted to:
  - add one `history_mode` boolean or one `compare_to` bag on top of ordinary
    query execution
  - treat branch/head/current/historical differences as runtime options rather
    than proof-bearing query contexts
  - reconstruct historical truth through host caches or controller glue
  - surface diff through raw storage deltas or branch summaries instead of
    query-shaped comparison artifacts
  - hide basis changes behind "use latest branch if history unavailable"
    fallback

If any supported path:

- silently substitutes one basis for another
- lets hosts invent or repair history semantics
- lets execution rediscover comparison family or materialization-path choice
  after planning/admission
- returns a branch/historical/diff result without explicit basis metadata
- exposes raw storage deltas as the primary diff artifact
- changes result-shape semantics because basis changed
- implies store-backed or durable history support before those lower-runtime
  guarantees exist

then Milestone 6 has failed.

## Product Decision Lock

- branch basis, historical basis, preview-derived basis, and diff/comparison
  basis are query-owned context artifacts
- `worth-query` owns basis declaration, basis binding, comparison shaping,
  result metadata, and certification for those contexts
- `worth-relational` and later `worth-store` remain authoritative for branch
  head truth, historical truth, snapshot/commit restoration, and comparison
  source data
- diff queries are query-shaped comparison artifacts over two declared bases;
  they are not raw storage delta APIs and not controller-owned summaries
- historical materialization-path identity remains lower-authority truth but
  query-visible result metadata
- the same canonical query shape must preserve the same declared result-shape
  meaning across basis variants
- unsupported basis families, unsupported pairings, and store-gated historical
  capabilities must fail typed and early
- Milestone 6 does not close lineage traversal, correspondence query surfaces,
  branch-comparison view semantics, durable historical parity, or store-backed
  diff execution; those belong to later milestones

Normative consequence:

- any implementation path that exposes branch/head/history through one generic
  options bag is out of spec
- any implementation path that replays host caches or controller memory as
  historical authority is out of spec
- any implementation path that returns diff as raw lower-store or lower-runtime
  delta records is out of spec
- any implementation path that lets result shapes drift by basis family is out
  of spec
- any implementation path that advertises durable/store-backed historical
  execution before the lower runtime can prove it is out of spec

## Compile-Time Enforcement Policy

Milestone 6 must classify which basis and comparison guarantees become
unrepresentable, uncompilable, or construction-time rejection.

`Unrepresentable` in public types:

- publicly constructible admitted basis contexts that do not carry canonical
  query identity, basis-family identity, and explicit basis digest
- publicly constructible diff/comparison artifacts that do not carry both
  declared basis identities and one closed comparison-basis family
- publicly constructible historical result metadata that omits explicit
  materialization-path identity where the path was part of admission
- publicly constructible branch/current/historical option bags that collapse
  basis family into booleans or free-form strings
- publicly constructible diff outputs that expose only raw storage or runtime
  delta payloads with no query-shaped result contract

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `QueryBasisContextBinding`,
  `AdmittedQueryBasisContext`, `AdmittedDiffQueryContext`,
  `QueryContextExecutionArtifact`, `QueryBasisMetadata`,
  `DiffQueryMetadata`, or materially equivalent proof-bearing types without
  crate-owned admission/lowering
- public APIs that accept raw historical lower-runtime artifacts, raw preview
  bags, raw storage deltas, or host-authored branch/history descriptors as if
  they were admitted query contexts
- public APIs that let consumers bypass admitted basis-context binding and call
  diff shaping directly
- public APIs that mutate one admitted basis context into another family after
  admission
- bool-driven basis selection surfaces such as `historical: bool`,
  `compare: bool`, or `use_branch: bool`
- public APIs that expose one generic `execute_with_basis(...)`,
  `compare(...)`, or `capability(mode)` bag where binding, admission,
  execution, and diff shaping are hidden behind one broad call

`Construction-time rejection`:

- unsupported historical basis families
- unsupported preview-derived basis families
- ambiguous or invalid comparison-basis pairings
- diff scope mismatch
- hidden basis substitution requests
- raw storage delta access requests
- store-backed historical or diff requests that remain explicit deferred debt
- historical requests whose admitted path/materialization evidence is missing
  for the requested basis

Rules:

- the strongest available boundary must be used
- basis, diff, and result-metadata proof types must use sealed constructors and
  private fields
- adding a new basis family, comparison family, or historical-admission class
  must force exhaustive compile failures across admission, execution,
  metadata shaping, support reporting, and certification until the new family
  is handled explicitly
- wildcard or catch-all matching over basis family or comparison family is out
  of spec in milestone-owned code paths
- compile-fail coverage is required for:
  - no external construction of admitted basis contexts
  - no raw historical lower-runtime artifact as query-context input
  - no bypass of diff-context admission
  - no bool-driven historical or compare shortcut
  - no post-admission branch/current/historical override
- runtime rejection is allowed only for facts genuinely unavailable until the
  lower runtime reports admitted historical compatibility, preview provenance,
  or comparison-basis admissibility

## Scope

### In Scope

- branch-scoped query contexts for current branch head and alternate branch head
- historical query contexts targeting admitted commit/snapshot/history bases
- preview-derived basis composition where existing preview provenance makes that
  basis identity honest
- query-shaped diff/comparison contexts over two admitted bases
- basis-explicit result metadata for branch, historical, and diff execution
- query-context-owned execution artifacts for admitted basis families
- typed diagnostics, counters, and replay artifacts for basis binding, history
  admission, and diff shaping
- unified-facade composition through the already-closed `QueryContextCapability`
  and related capability/support surfaces
- milestone-native certification for basis parity, historical parity, and
  diff-shaping honesty

### Explicitly Out Of Scope

- lineage traversal query expressions and richer correspondence query families,
  which remain Milestone 7 work
- branch comparison view semantics and presentation-driven diff grouping, which
  remain Milestone 8 work
- policy masking, tenant schema variation, or relationship-proof semantics,
  which remain Milestone 9 work
- store-backed historical parity, durable point-in-time restore, restart-stable
  history parity, and store-backed diff execution, which remain later store-
  gated work
- raw storage delta transport, store inspection helpers, or controller-owned
  comparison summaries
- mutation, merge, or writeback semantics beyond consuming the basis contexts
  already frozen in Milestone 5.5

## Initial Admission Matrix

Milestone 6 must not leave basis behavior ambient.

Initial basis-family-admitted contexts:

- `CurrentBranchHead`
- `BranchHead`
- `HistoricalSnapshot` where the lower runtime already exposes honest admitted
  historical materialization for the requested basis
- `HistoricalCommit` where the lower runtime already exposes honest admitted
  historical materialization for the requested basis
- `PreviewDerivedHistorical` only where an existing preview artifact carries
  enough provenance to bind the basis honestly rather than inventing host glue
- `DiffComparison` only over two already-admitted basis contexts

Required vocabulary artifacts:

- `QueryContextFamily`
- `ComparisonBasisFamily`
- `HistoricalAdmissionClass`
- `QueryContextCostClass`
- `QueryContextBudgetClass`
- `QueryContextDriftOutcome`
- `QueryContextAdmissionFailureClass`

Required basis-state artifacts:

- `QueryBasisContextRequest`
- `QueryBasisContextBinding`
- `AdmittedQueryBasisContext`
- `AdmittedDiffQueryContext`
- `QueryBasisMetadata`
- `DiffQueryMetadata`
- `QueryContextExecutionArtifact`
- `QueryContextPredictionReport`
- `QueryContextPredictionDriftOutcome`

Required basis/result metadata content:

- canonical `query_digest`
- explicit `basis_digest`
- explicit `basis_family`
- explicit `comparison_basis_family` where relevant
- explicit `materialization_path_identity` where historical admission made it
  part of truth delivery
- explicit `preview_provenance_identity` where preview-derived admission made
  it part of truth delivery
- explicit `result_digest`
- explicit denial metadata where the requested lane is rejected rather than
  admitted

Required comparison-basis classes:

- `BranchToBranch`
- `CurrentToHistorical`
- `HistoricalToHistorical`
- `PreviewToAuthoritative` only where the preview provenance artifact is
  already admitted and explicit

Initial admitted historical-admission classes:

- `RuntimeRetained`
- `RuntimeReplay`
- `RuntimeReconstruction` only where the lower runtime explicitly admits it

Required cost posture artifacts:

- `QueryContextCostClass`
- `QueryContextBudgetClass`
- `HistoricalMaterializationCostClass`
- `QueryContextPredictionReport`
- `QueryContextPredictionDriftOutcome`

Initial denied or deferred families:

- raw storage-delta comparison
- hidden branch alias or host-cache history reconstruction
- store-backed historical replay/restore beyond what the lower runtime already
  exposes honestly
- comparison families that require implicit basis discovery or broad
  re-scanning
- any diff/context lane that cannot emit basis-explicit query-shaped output

Any family not named above is out of scope for Milestone 6 and must fail typed
and early rather than becoming implied beta support.

## Initial Performance Posture Matrix

- current branch-head execution:
  basis resolution is O(1) against the admitted runtime-backed current basis
  artifact; no history scan or basis rediscovery is allowed
- alternate branch-head execution:
  branch-basis lookup is explicit and bounded by one admitted branch basis
  artifact; no host fallback to current branch is allowed
- admitted historical execution:
  historical-path compatibility, requested/admitted/resolved path identity, and
  materialization-path posture must be explicit before rich result shaping
- diff/comparison execution:
  comparison shaping operates over two already-admitted basis artifacts and one
  explicit comparison-basis family; raw delta breadth and broad rescans must
  deny rather than widen

## Performance Architecture

Milestone 6 must encode performance as part of basis admission and comparison
architecture, not as passive telemetry attached afterward.

Required performance posture:

- every admitted single-basis context carries one `QueryContextCostClass`
- every admitted single-basis context carries one `QueryContextBudgetClass`
- every admitted diff/comparison context carries one `QueryContextCostClass`
- every admitted diff/comparison context carries one `QueryContextBudgetClass`
- every historical-admission lane carries one
  `HistoricalMaterializationCostClass`
- every executable or comparable context carries one
  `QueryContextPredictionReport` before rich execution or diff shaping
- every executed or compared context carries one
  `QueryContextPredictionDriftOutcome` after realization

Representative cost classes should distinguish at minimum:

- `CurrentHeadNarrow`
- `BranchHeadNarrow`
- `HistoricalRetainedBounded`
- `HistoricalReplayBounded`
- `HistoricalReconstructionBounded` only where the lower runtime explicitly
  admits it
- `DiffComparisonBounded`

Required width surfaces:

- `basis_binding_width`
- `historical_lookup_width`
- `comparison_binding_width`
- `comparison_row_width`
- `metadata_attachment_width`
- `denial_width`

Required drift or denial outcomes should distinguish at minimum:

- `WithinBudget`
- `ExplicitBroadeningDenied`
- `ExplicitRebindRequired`
- `HistoricalPathTooBroadDenied`
- `ComparisonScopeTooBroadDenied`

Rules:

- cost class and budget class belong to admission artifacts, not executor logs
- historical provenance and historical cost posture must remain distinct
- prediction must happen before rich execution or diff row shaping
- denial is cheaper than degraded execution; broadening-required lanes must
  fail before rich artifact shaping and before row materialization
- repeated execution or comparison of the same admitted context must not
  rediscover basis family, comparison family, or materialization-path cost
  posture
- support metadata may later advertise admitted cost posture, but milestone-
  owned execution and certification must already preserve it mechanically
- basis family, comparison family, and materialization-path posture belong to
  admission/lowering artifacts, not executor logs
- cost class, budget class, prediction posture, and drift outcome belong to
  admission/execution artifacts, not executor logs
- the same basis artifact should be executed or compared repeatedly without
  rediscovering its meaning
- unsupported or broadening-required comparisons must deny before building rich
  execution artifacts
- broadening-required historical lanes must deny before rich execution
  artifacts and before metadata row materialization
- diff shaping must stay query-owned and result-shape-aware rather than
  devolving into raw lower-runtime delta narration
- "same query, different basis" must change explicit basis artifacts and
  counter surfaces, not hidden execution heuristics

## Phases

### Phase 1: Basis Taxonomy And Context Binding

Phase 1 exists to freeze the public vocabulary for branch/current/historical/
preview-derived/diff basis families and to bind those families to existing
proof-bearing query or lower-runtime artifacts.

Milestone 6 must first implement:

- one sealed query-context subdomain dedicated to basis contexts
- closed basis-family and comparison-family enums
- proof-bearing basis request, binding, and admitted-context artifacts
- admitted cost class and budget class attachment during context admission
- binding from:
  - runtime current branch-head artifacts
  - runtime alternate branch-head artifacts
  - admitted historical-path artifacts
  - admitted preview-derived provenance artifacts where honest
- typed denial for:
  - unsupported basis family
  - missing or invalid basis provenance
  - basis substitution attempts
  - ambiguous comparison basis

This phase leaves the system in a coherent state where:

- branch/head/history/diff basis families are query vocabulary, not host glue
- later phases can execute or compare only already-admitted basis artifacts
- basis identity is explicit before any historical or diff result shaping

Phase exit criterion:

- no public API still expresses basis as free-form options or ambient host
  state

### Phase 2: Historical Admission And Execution Contracts

Phase 2 exists to make historical execution a real query-context execution path
rather than metadata stapled onto ordinary runtime reads.

Milestone 6 must then implement:

- historical compatibility and admission contracts over admitted historical
  basis artifacts
- historical materialization cost posture alongside historical provenance
- query-context-owned execution artifacts for:
  - branch-head execution
  - admitted historical execution
  - preview-derived execution where admitted
- explicit requested/admitted/resolved materialization-path identity on
  historical execution metadata
- prediction reports before execution and drift outcomes after execution
- typed denial for:
  - unsupported historical basis
  - unsupported historical materialization-path class
  - hidden basis substitution
  - preview-derived basis presented as ordinary authoritative historical truth
    without explicit admitted provenance
  - store-backed or deferred historical request beyond admitted runtime support

This phase leaves the system in a coherent state where:

- historical reads are basis-explicit execution lanes, not host-mode switches
- historical result meaning is aligned with current/branch result meaning
- materialization-path identity remains part of truth delivery rather than
  logging detail

Phase exit criterion:

- admitted historical execution produces query-owned execution artifacts instead
  of ordinary preflight results plus host-added metadata

### Phase 3: Diff Contexts And Query-Shaped Comparison Artifacts

Phase 3 exists to make diff/comparison a real query capability over two
declared bases instead of a post-hoc branch summary or raw lower-delta surface.

Milestone 6 must then implement:

- diff-context binding from two admitted basis contexts
- explicit comparison-basis-family admission
- comparison cost class, budget class, and prediction posture before shaping
- query-shaped diff metadata and one query-shaped change-set artifact
- typed change rows/families aligned with declared projection/scope
- typed denial for:
  - diff scope mismatch
  - raw-storage-delta leakage
  - basis pairings whose declared result shape cannot be produced honestly from
    the admitted basis pair
  - basis pairings that require hidden broadening or reconstruction

This phase leaves the system in a coherent state where:

- diff outputs are comparison artifacts over two query-owned bases
- diff remains aligned to declared query shape and projection semantics
- later lineage or view-shape work can build on explicit comparison contracts
  instead of replacing them

Phase exit criterion:

- no admitted diff lane depends on host comparison summaries or raw delta bags

### Phase 4: Unified Facade Composition And Result-Metadata Integration

Phase 4 exists to make Milestone 6 a real daily-driver query surface through
the already-closed 5.6 application facade instead of one more sidecar module.

Milestone 6 must then implement:

- capability-level composition through the unified application facade
- basis and diff metadata attachment through one application-facing query
  context capability
- support-matrix and capability-admission updates for the new admitted query
  context families
- explicit basis/result metadata on admitted branch, historical, and diff
  execution results
- compile-time witness boundaries proving:
  - query-context witness cannot do ordinary read execution by shortcut
  - historical-only witnesses cannot fabricate basis contexts
  - dynamic basis family routing remains forbidden

This phase leaves the system in a coherent state where:

- Milestone 6 is part of the endorsed application surface
- basis variation is visible through capability/support metadata
- the legacy broad facade does not regain control of branch/history/diff
  semantics

Phase exit criterion:

- all new Milestone 6 capability surfaces are available through the unified
  facade and certified there

### Phase 5: Replay, Counter Proof, And Boundary Certification

Phase 5 exists to close the milestone through proof rather than "history seems
to work" demos.

Milestone 6 must finally ship:

- the `Historical / Diff / Basis Parity Test`
- canonical rows proving:
  - current-vs-branch basis explicitness
  - current-vs-historical basis explicitness
  - historical materialization-path explicitness
  - diff comparison-family explicitness
  - result-shape parity across admitted basis variants
  - preview-derived basis explicitness where admitted
  - admitted cost-class explicitness
  - prediction-versus-realization explicitness
- rejection rows proving:
  - unsupported historical basis
  - ambiguous comparison basis
  - diff scope mismatch
  - store-backed historical deferred debt
  - forbidden basis substitution
  - raw-storage-delta leakage forbidden
  - broadening-required historical denial
  - broadening-required comparison denial
- compile-fail or privacy hardening proving basis and comparison artifacts
  cannot be WORTHd externally

This phase leaves the system in a coherent state where:

- basis variation and diff semantics are replay-safe and machine-checkable
- later lineage, view-shape, and store-backed milestones inherit explicit basis
  contracts instead of soft history APIs

Phase exit criterion:

- the certification suite proves basis parity and denial honesty through
  canonical artifacts rather than row presence alone

## Must Ship

- proof-bearing `QueryBasisContextRequest`, `QueryBasisContextBinding`,
  `AdmittedQueryBasisContext`, `AdmittedDiffQueryContext`,
  `QueryContextExecutionArtifact`, `QueryBasisMetadata`, `DiffQueryMetadata`,
  and `QueryDiffChangeSetArtifact` families or materially equivalent types
- explicit basis-family vocabulary for current branch, alternate branch,
  admitted historical, preview-derived, and diff/comparison contexts
- query-context-owned execution over admitted basis families
- query-shaped diff/comparison artifacts over two admitted bases
- explicit materialization-path-aware metadata for admitted historical results
- one dedicated query-context performance/counter subdomain rather than generic
  telemetry-only logging
- typed diagnostics, replay artifacts, and exact counters for basis binding,
  historical admission, and diff shaping
- milestone-native certification proving basis parity, diff honesty, and
  rejection behavior
- one representative scenario matrix binding admitted basis families to
  concrete current/branch/historical/diff lanes and denial classes

## Must Preserve

- canonical query meaning from Milestone 1 remains authoritative
- validation legality from Milestone 2 remains authoritative
- proof-bearing planning and basis identity from Milestone 3 remain
  authoritative
- collection/result-shape semantics from Milestone 4 remain authoritative
- live/locality semantics from Milestones 5 and 5.1 remain authoritative where
  branch or historical contexts later compose with live-maintained reads
- preview-session basis identity from Milestone 5.2 remains authoritative where
  preview-derived basis families are admitted
- frontier posture from Milestone 5.3 remains authoritative where historical or
  diff execution composes with planned route posture
- historical materialization-path honesty from Milestone 5.4 remains
  authoritative
- workflow basis and lowering honesty from Milestone 5.5 remain authoritative
  where workflow inspection later consumes basis contexts from this milestone
- unified facade/configuration authority and capability discipline from
  Milestone 5.6 remain authoritative
- `worth-relational` and later `worth-store` remain the only authorities for
  branch/head/historical truth
- diff outputs remain query-shaped comparison artifacts rather than raw lower-
  runtime delta passthrough

## Complexity / Proof Obligations

Milestone 6 must name costs and proofs in terms of:

- basis binding count
- historical basis lookup count
- comparison basis lookup count
- basis binding width
- historical lookup width
- comparison binding width
- comparison scope width
- diff input breadth
- metadata attachment width
- denial width
- basis substitution denial count
- unsupported basis denial count
- materialization-path compatibility check count
- query-context execution count
- query-context metadata attachment count
- diff change-set row width
- predicted comparison width
- realized comparison width
- executor rediscovery avoidance on basis and diff lanes

Minimum required counters:

- `query_basis_binding_count`
- `historical_basis_lookup_count`
- `comparison_basis_lookup_count`
- `basis_binding_width`
- `historical_lookup_width`
- `comparison_binding_width`
- `comparison_scope_width`
- `diff_input_breadth`
- `metadata_attachment_width`
- `denial_width`
- `unsupported_basis_denial_count`
- `basis_substitution_denial_count`
- `materialization_path_compatibility_check_count`
- `query_context_execution_count`
- `query_context_metadata_attachment_count`
- `diff_change_set_row_width`
- `predicted_comparison_width`
- `realized_comparison_width`
- `query_context_executor_rediscovery_count`
- `basis_rediscovery_count`
- `historical_path_rediscovery_count`
- `comparison_family_rediscovery_count`

Rules:

- counters belong to admitted basis-context artifacts, diff artifacts, denial
  bundles, and certification bundles
- representative certification scenarios must assert exact counts
- representative certification scenarios must assert exact width surfaces
- `query_context_executor_rediscovery_count` must be exactly zero on every
  admitted lane
- `basis_rediscovery_count` must be exactly zero on every admitted lane
- `historical_path_rediscovery_count` must be exactly zero on every admitted
  lane
- `comparison_family_rediscovery_count` must be exactly zero on every admitted
  lane
- every denied unsupported basis attempt must increment
  `unsupported_basis_denial_count`
- every hidden basis substitution denial must increment
  `basis_substitution_denial_count`
- every historical admission must make compatibility checks and path identity
  mechanically visible rather than implicit
- every diff lane must record both comparison scope width and diff input
  breadth explicitly
- "predicted" versus "realized" comparison width must remain explicit where the
  milestone emits prediction posture rather than inferred from one blended
  counter
- no admitted lane may hide broad comparison rescans or raw delta recovery
  inside generic success counters
- no admitted lane may hide broad historical reconstruction, basis rebinding,
  or comparison-family rediscovery inside generic success counters

Minimum certification rows should include:

- `current-branch-basis-explicitness`
- `current-historical-basis-explicitness`
- `historical-materialization-path-explicitness`
- `diff-comparison-family-explicitness`
- `basis-result-shape-parity`
- `preview-derived-basis-explicitness`
- `query-context-cost-class-explicitness`
- `prediction-realization-drift-explicitness`

Minimum rejection rows should include:

- `unsupported-historical-basis`
- `ambiguous-comparison-basis`
- `diff-scope-mismatch`
- `store-backed-historical-deferred-debt`
- `forbidden-basis-substitution`
- `raw-storage-delta-leakage-forbidden`
- `historical-broadening-denied`
- `comparison-broadening-denied`

## Allowed Debt

- durable store-backed historical execution parity may remain explicit `Debt`
  until `worth-store` can support it honestly
- durable point-in-time restore and restart-stable historical parity may remain
  explicit `Debt`
- store-backed diff execution may remain explicit `Debt`
- richer diff expression families and branch-comparison presentation semantics
  may remain later work
- hidden basis substitution may not exist as debt
- host-cache historical authority may not exist as debt
- raw-storage delta leakage may not exist as debt
- basis-specific result-shape drift may not exist as debt

## Acceptance Evidence

Milestone 6 is complete only when `worth-query` can prove:

- the `Historical / Diff / Basis Parity Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- the same declared query shape can run against current, branch, and admitted
  historical truth bases without changing structural meaning
- diff queries produce structured, typed change sets aligned to declared
  projection and scope
- basis identity and, where applicable, materialization-path identity remain
  explicit in result metadata
- unsupported or deferred basis families fail typed and early
- where store-backed historical execution exists later, it must compare equal
  to runtime-backed historical truth for the same basis

Required verification output must include:

- `query_digest`
- `basis_digest`
- `basis_family`
- `comparison_basis_family` where relevant
- `materialization_path_identity` where relevant
- `preview_provenance_identity` where relevant
- `result_digest`
- `replay_digest`
- `failure_digest`
- `counter_snapshot`

## Representative Scenario Matrix

Milestone 6 must prove the architecture against concrete lanes, not just
abstract capability names.

Minimum representative scenarios:

- `current-to-branch-basis-parity`
  - same canonical query shape executes against current branch head and one
    alternate branch head
  - `query_digest` remains equal while `basis_digest` differs
  - result-shape family remains identical
- `historical-retained-path-visible`
  - admitted historical execution exposes requested, admitted, and resolved
    materialization-path identity explicitly
  - no host-supplied historical reconstruction is involved
- `historical-preview-derived-provenance-explicit`
  - preview-derived historical admission preserves preview provenance identity
    rather than collapsing into ordinary historical truth
  - any attempt to erase that provenance must fail typed
- `branch-to-branch-diff-shaped`
  - diff between two admitted branch bases emits one query-shaped change-set
    artifact rather than raw delta payloads
  - comparison-basis family is explicit and stable
- `current-to-historical-diff-shaped`
  - diff between current and admitted historical basis preserves one declared
    result-shape family and basis-explicit metadata
  - broadening-required comparison must deny before rich artifact shaping
- `forbidden-basis-substitution`
  - a request that would silently swap branch, historical, or preview-derived
    basis must fail typed and increment substitution denial counters
- `raw-storage-delta-forbidden`
  - any attempt to surface raw lower-store or lower-runtime delta records as
    the primary diff artifact must fail typed and early
- `store-backed-history-deferred`
  - store-backed historical or diff request remains explicit deferred debt and
    cannot present as partial support
- `repeated-admitted-basis-no-rediscovery`
  - repeated execution or comparison of the same admitted basis context keeps
    basis, historical-path, and comparison-family rediscovery counters at zero
- `historical-path-too-broad-denied`
  - a historical request that would require broader reconstruction than the
    admitted path allows must deny before rich artifact shaping
- `comparison-scope-too-broad-denied`
  - a comparison request whose declared shape would force broad rescans or raw
    delta expansion must deny before row materialization

## Architectural Notes

### Basis Must Be A First-Class Artifact

This milestone only works if basis is treated as part of query meaning delivery
instead of a runtime option.

The required rule is:

- current branch is a basis
- alternate branch is a basis
- historical truth is a basis
- preview-derived truth is a basis
- diff is two explicit bases plus one comparison family

If code can still say "run this query, maybe historically" without naming a
basis artifact, the milestone is not finished.

### Historical Truth Is Not Host Reconstruction

Milestone 5.4 already froze materialization-path honesty. Milestone 6 must
carry that honesty into ordinary query execution.

The rule is:

- if historical truth is admitted, say how it was materialized
- if the lower runtime cannot admit it honestly, deny it
- if a host cache could approximate it but not prove it, that is not support

### Diff Must Stay Query-Shaped

The easiest way to fake diff support is to return raw lower-runtime delta
records and let hosts interpret them.

Milestone 6 must instead require:

- two explicit admitted bases
- one explicit comparison family
- one query-shaped diff artifact aligned with declared projection and scope

That is the difference between a comparison feature and one more plumbing
escape hatch.

### Result Shape Must Not Drift By Basis

Basis variation is allowed to change truth values and basis metadata. It is not
allowed to change the declared result-shape family for the same canonical query
shape.

The required rule is:

- if the canonical query shape is unchanged, the result-shape family must stay
  unchanged
- basis metadata may change because the basis changed
- result values may change because truth changed
- comparison metadata may change because the comparison family changed
- shape-family drift is only legal when the declared query shape changed

### This Milestone Must Not Steal 7, 8, Or 10

Milestone 6 owns basis variation and diff-context honesty.

- Milestone 7 owns richer lineage and correspondence query surfaces
- Milestone 8 owns branch-comparison view semantics and broader presentation
  shaping
- Milestone 10 owns store-backed execution parity and durable historical/store
  completion

Milestone 6 must therefore stop at:

- basis declaration
- basis admission
- admitted execution
- diff/comparison shaping
- result metadata
- certification

It must not drift into lineage query expansion, presentation-specific branch
views, or speculative store-backed parity claims.

## Sequencing Notes

Milestone 6 belongs immediately after Milestone 5.6 because the unified
application facade is now closed and production-ready, so basis variation can
be exposed through one endorsed surface without reopening application-facade
design.

It belongs after Milestones 5.2 and 5.4 because preview basis identity and
historical materialization-path honesty are already frozen and now need to be
generalized into ordinary basis contexts rather than host-local special cases.

It must land before Milestone 7 because lineage and correspondence queries need
explicit branch/history/diff basis semantics instead of defining their own
parallel basis model.

It must land before Milestone 8 because branch comparison views and
presentation-driven diff semantics should consume explicit diff/basis artifacts
rather than inventing them at the view layer.

## Parallelization Notes

Once basis-family vocabulary and admitted context artifacts are frozen:

- early Milestone 7 lineage/correspondence design can proceed in parallel
  without redefining basis identity
- early Milestone 8 view-shape work can proceed in parallel against explicit
  diff/basis artifacts
- counter hardening and compile-time tightening can proceed in parallel without
  changing milestone semantics
- later store-backed parity work can target the same basis contracts without
  changing runtime-backed Milestone 6 meaning

## Explicit Failure Taxonomy For Milestone 6

- unsupported basis family
- unsupported historical basis
- missing basis provenance
- forbidden basis substitution
- ambiguous comparison basis
- invalid comparison-family pairing
- diff scope mismatch
- raw storage-delta leakage
- deferred store-backed historical request
- deferred store-backed diff request
- historical-path compatibility failure
- query-context replay divergence
- query-context artifact invariant break

## Anti-Patterns Explicitly Rejected

- `history_mode: true` or equivalent bool-driven basis shortcuts
- `compare_to` bags that do not require two admitted bases
- host-side branch aliasing or cache repair that silently changes the basis
- diff implemented as raw storage/runtime delta passthrough
- basis-specific result-shape families that drift from the declared query shape
- one mega-module mixing basis binding, historical execution, diff shaping,
  metadata, diagnostics, replay, and certification
- public construction of basis or diff proof types without the proving path

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it freezes how one canonical query shape moves across
multiple truth bases without changing meaning, which is the contract later
branch/history/lineage/view/store work all depend on.

The adversarial constraint is load-bearing because it forbids the naive failure
mode where current, branch, historical, and diff reads are all "supported"
only because hosts repair basis differences and patch up result meaning after
execution.

The milestone preserves authority boundaries because `worth-query` owns basis
declaration, admission, execution shaping, and comparison shaping while lower
runtime truth and historical authority remain below it.

The milestone defines proof obligations rather than implementation chores
because basis parity, materialization-path visibility, diff honesty, denial
behavior, replay-safe artifacts, and exact counters are required for closeout.

A competent engineer should be able to map this spec into honest query-context,
historical-admission, diff-shaping, facade, certification, and compile-fail
modules without inventing the architecture during implementation.

This milestone belongs at 6 because it is the branch/history/diff contract
layer that must exist before richer lineage, view-shape, and store-backed
work can close honestly.

## Closeout Standard

Milestone 6 is complete only when all of the following are true:

- admitted basis families are explicit, sealed, and query-owned
- current, branch, historical, and diff lanes preserve the same canonical
  query meaning apart from the declared basis
- historical result bundles remain explicit about materialization-path meaning
  where applicable
- diff results are query-shaped comparison artifacts rather than raw deltas
- unsupported or store-gated basis families fail typed and early
- certification bundles prove basis parity and denial honesty through canonical
  machine-checkable artifacts

If code lands but branch/history/diff still depend on ambient basis repair,
host reconstruction of historical truth, raw storage delta exposure, or
basis-specific result-shape drift, Milestone 6 is not complete.
