# Milestone 11 Engineering Spec: Cross-Runtime Policy Propagation And Clean Configuration Model

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-10.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-10.md)
>
> **Bridge certification companion:** [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
>
> **Primary architectural driver:** make cross-runtime policy declaration, admission, lowering, provenance, and rejection first-class bridge protocol surfaces so deterministic-vs-optimized mode, diagnostics/artifact policy, replay allowance, preview policy, and host/runtime policy inputs remain explicit, replay-safe, and request-scoped instead of drifting into ambient runtime state
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

Milestones 1 through 10 established that the bridge already has strong protocol
surfaces for:

- canonical patch ingestion, routing, and replay-safe reduction
- historical, branch-aware, and merge-aware truth consumption
- protocol-grade source reads and structural advisory remapping
- speculative preview declaration, discard, promotion, and replay
- typed diagnostics, counters, and certification bundles strong enough to
  distinguish authoritative and non-authoritative bridge meaning

That is enough to make the bridge honest about truth authority, history
selection, structural ambiguity, merge pressure, and preview lifecycle.

It is not enough to make the bridge honest about policy.

Without Milestone 11, the bridge still has a dangerous blind spot:

- there is already a runtime-level `BridgeRuntimePolicy`, truth-view policy
  resolution, diagnostics-tier selection, replay allowance, and preview-era
  request-kind separation, but there is not yet one canonical bridge-owned
  policy language that explains how those surfaces compose per request
- identical bridge work can be run under different policy intentions, but the
  system cannot yet produce one typed provenance record saying exactly which
  policy source changed planning, retention, diagnostics, or execution
- host builders can set runtime policy and individual requests can carry local
  policy-like knobs, but the bridge does not yet fully distinguish:
  `runtime baseline`, `request declaration`, `truth-side admissibility`,
  `signal-side execution preference`, `artifact policy`, and `diagnostics
  shaping`
- policy conflicts still risk being handled as local defaults, convenience
  overrides, or builder folklore instead of typed request admission and typed
  rejection

Milestone 10 taught the bridge how to say:

`this exact preview request stayed non-authoritative, crossed this exact discard or promotion boundary, and remained replay-safe`

Milestone 11 must now teach the bridge how to say:

`this exact bridge request, under this exact declared policy bundle, consuming this exact runtime baseline and this exact truth/runtime admissibility basis, lowered into this exact planning mode, diagnostics mode, artifact retention mode, and replay mode, and every policy effect is attributable, replay-safe, and request-scoped`

not:

`the runtime was in some mode, the request asked for something policy-shaped, and the bridge sort of did the reasonable thing`

## Goal

Make policy propagation, policy provenance, and policy rejection a
deterministic, replay-safe, bridge-owned protocol so hosts can vary execution
mode, diagnostics richness, artifact retention, replay allowance, and preview
behavior without turning bridge policy into ambient state, duplicate authority,
or builder-order folklore.

## Why This Milestone Exists

Milestone 11 belongs immediately after Milestone 10 because policy propagation
is only safe once preview-versus-authoritative lifecycle boundaries are already
typed and explicit.

Milestone 10 established:

- explicit request-kind separation
- explicit preview lifecycle and promotion boundaries
- explicit non-authoritative artifact classes and discard rules
- replay-safe preview and authority-crossing records

Milestone 11 now needs to establish the matching policy truths:

- one bridge-owned declaration vocabulary for cross-runtime policy inputs
- one bridge-owned admission and rejection surface for legal and illegal policy
  combinations
- one lowered policy packet or proof bundle that execution consumes without
  policy rediscovery
- one canonical provenance artifact explaining which policy source changed what
  and which policy source was merely present but non-operative

If Milestone 11 shipped before Milestone 10, it would be forced to treat
preview-versus-authoritative semantics as a policy toggle rather than as a
structural boundary. That would be wrong. Request kind and preview lifecycle
are structural truth first, policy refinement second.

Milestone 11 also belongs before Milestone 12 because bridge-mediated writeback
cannot be safe while policy meaning is still ambient. Writeback needs a clean
answer to:

- which requests are deterministic-only
- which requests admit optimized execution
- which requests require replay artifacts
- which diagnostics and artifact policies are permitted for authoritative
  writeback-shaped work
- which policy conflicts fail before execution begins

Milestone 11 therefore earns its place in the roadmap by solving the next real
structural problem after speculative coordination: keeping policy explicit,
bounded, composable, and attributable without promoting the bridge into a
second scheduler or a second policy authority.

## Adversarial Constraint

Milestone 11 must survive the following hostile condition:

> A long-lived system with mixed authoritative and preview requests, historical
> and branch-local truth reads, merge-aware and structural-advisory flows,
> deterministic and optimized execution modes, replay-required and replay-
> disabled environments, diagnostics tiers that vary by deployment, builder-
> provided runtime defaults, per-request policy declarations, host adapters that
> present policy-shaping hints in different orders, and frequent request
> interleaving must produce the same policy admission result, the same lowered
> execution policy, the same rejection class for illegal combinations, the same
> policy provenance artifact, and the same replay result every time, while
> never allowing policy meaning to leak through ambient mutable runtime state or
> host-local convenience defaults.

Concretely, the design must remain correct when all of the following are true:

- one request demands deterministic routing and replay-safe artifacts while the
  next admits optimized execution and reduced richness
- one host sets a runtime baseline through the builder while another host
  declares equivalent request policy explicitly at the call site
- historical and branch-local requests interleave with preview and authoritative
  requests
- diagnostics richness changes between environments
- replay artifacts are disabled for some runtimes but required by specific
  requests
- truth-side admissibility, source capability, and preview lifecycle rules
  constrain which policy combinations are legal
- equivalent policy bundles arrive in different declaration or builder orders
- requests are replayed after restart using only canonical bridge artifacts

If any supported path:

- lets a request inherit policy from the previous request without declaring it
- collapses runtime baseline, request policy, and truth/runtime admissibility
  into one unstructured "effective policy" bag
- treats deterministic-vs-optimized mode as an implementation detail instead of
  an attributable policy outcome
- hides policy rejection behind silent fallback to runtime defaults
- lets diagnostics richness or retention policy change canonical policy meaning
  during replay
- requires the executor to rediscover policy combination legality after
  planning
- makes builder registration order or host call order part of policy meaning
- lets preview/authoritative boundaries be redefined by policy rather than
  consumed as prior structural truth

then Milestone 11 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- request kind remains a structural declaration, not a policy flag
- runtime baseline policy, request policy declaration, truth-view/source
  admissibility, speculation admissibility, lowered execution policy, and
  provenance artifact are distinct concepts and must remain distinct types
- the bridge may propagate and lower policy; it may not invent policy meaning
  that belongs to `worth-relational` or `worth-signal`
- truth/runtime authority still decides source capability, historical
  admissibility, replay compatibility at the truth-view boundary, and branch
  validity
- signal/runtime authority still owns actual scheduling and downstream
  execution semantics
- the bridge owns only policy declaration, combination legality, lowering, and
  provenance
- illegal policy combinations fail during validation or admission, not late in
  the executor
- diagnostics richness may change retained detail, but not the canonical
  identity of the admitted policy bundle or the meaning of a rejection
- builder defaults are baseline inputs only; they are not ambient mutable state
  that later requests may reinterpret
- policy provenance must record not just the final lowered policy, but which
  source supplied each operative policy fact and which declared facts were
  ignored, narrowed, or rejected
- optimized policy must remain subordinate to deterministic canonical meaning;
  it may change admitted execution strategy or richness, but not canonical
  authority boundaries
- Milestone 11 productizes policy propagation only; it does not yet productize
  bridge-mediated writeback or broaden scheduler authority inside the bridge

Normative consequence:

- APIs exposing a loosely mutable global "current bridge mode" are out of spec
- requests that depend on prior request diagnostics or replay settings are out
  of spec
- fallback from illegal policy combination to "closest valid defaults" is out
  of spec
- executor-side strategy branching that recomputes policy legality is out of
  spec
- builder surfaces that flatten unrelated policy concerns into one bag without
  subsystem boundaries are out of spec
- diagnostics-only policy explanation without canonical policy artifacts is out
  of spec

## Configuration And Defaults

Milestone 11 should expose a small set of explicit policy surfaces. Authority
boundaries and legality rules are not configurable.

### Admitted Configurable Surfaces

- runtime baseline policy
  - default: `BridgeRuntimePolicy::development()`
- request execution mode declaration
  - default: `DeterministicCanonical`
- diagnostics policy declaration
  - default: `Standard`
- artifact retention declaration
  - default: sufficient retention for admitted replay and canonical
    diagnostics artifacts at the chosen runtime tier
- replay declaration
  - default: `Enabled` when runtime policy admits replay artifacts
- preview/authority policy refinement
  - default: inherits structural request kind and may only refine allowed
    richness or replay surfaces within that boundary
- provenance publication policy
  - default: publish canonical policy provenance for every admitted request and
    every rejected combination

### Non-Configurable Surfaces

- policy meaning inferred from prior request execution
  - default: never admitted
- request-kind reclassification through policy
  - default: never admitted
- silent fallback from illegal combination to baseline default
  - default: never admitted
- diagnostics retention changing policy identity
  - default: never admitted
- execution-phase policy legality rediscovery
  - default: never admitted
- host-order or builder-order dependence for policy meaning
  - default: never admitted

The bridge should therefore feel configurable at the declaration and richness
layer, but closed and fail-safe at the authority and legality boundary.

## Guideline Influence

### 1. `MENTALITY.md`

This document directly shapes the milestone:

- adversarial constraint first:
  the spec starts from ambient-state leakage, builder-order drift, replay
  ambiguity, and illegal policy fallback rather than from the pleasant feature
  phrase "support policy propagation"
- solve the hard problem first:
  policy vocabulary, policy-source separation, typed rejection, and lowered
  policy proof surfaces ship before ergonomic policy helpers
- enforce mechanically, not by convention:
  legal policy combinations, operative policy source, and lowered policy must
  be represented by proof-bearing types and typed failures
- spec is architecture is code:
  the spec names the declaration, validation, admission, lowering,
  provenance, and diagnostics artifacts that code must map directly
- separate what/how/whether:
  request kind and truth/view basis remain the `what`, policy lowering is the
  `how`, and diagnostics/artifact richness is the `whether`
- authority first, derivation second:
  authoritative runtime semantics stay upstream; policy provenance records and
  lowered bridge policy artifacts remain derived and replayable

### 2. `architectural_guidelines.md`

This document determines the structural boundaries:

- Laws 17, 21, 27, and 30 require policy resolution to happen before
  execution, to stay separate from artifact richness, and to flow as a lowered
  proof chain rather than runtime rediscovery
- Laws 13 and 16 require configuration shape to mirror subsystem architecture
  rather than flattening policy into one bag
- Laws 26, 32, 40, and 41 require explicit equivalence, explicit counters,
  honest naming, and proof-bearing types for policy identity and admissibility
- Law 29 forbids abstractions that hide correctness or cost boundaries, so
  deterministic-vs-optimized mode, replay allowance, diagnostics policy, and
  artifact retention cannot disappear behind one vague "bridge mode" enum

### 3. `domain_standards.md`

This document constrains decomposition and naming:

- policy declaration is not the same responsibility as policy validation
- policy provenance is not the same responsibility as lowered execution policy
- builder baseline registration is not the same responsibility as request-time
  policy admission
- diagnostics explanation is not the same responsibility as canonical policy
  record publication
- names must reflect domain nouns such as policy declaration, policy source,
  lowered execution policy, provenance record, and policy rejection rather than
  `manager`, `resolver`, or `helper`

### 4. `performance_guidelines.md`

This document constrains the cost model:

- policy admission and lowering must be proportional to declared policy width
  and admissibility inputs, not broad scans over runtime history or prior
  requests
- illegal combinations must be rejected before routing, source materialization,
  speculative resource construction, or writeback-shaped preparation
- execution strategy decisions belong before execution and must be consumed from
  a lowered policy packet
- policy APIs must be honest about replay retention, diagnostics richness, and
  artifact policy cost
- counters must explain policy-source count, admitted combination width,
  rejection count, provenance width, and any optimized-path admission decisions

## Scope

In scope for Milestone 11:

- bridge-owned policy declaration and validation surfaces
- separation of runtime baseline policy from request policy declarations
- policy-source provenance for truth/runtime, request, and bridge-lowered
  policy outcomes
- typed legality and rejection for cross-runtime policy combinations
- lowered policy packets or equivalent proof-bearing forms consumed by planning
  and execution
- replay-safe canonical policy records and diagnostics explanation
- builder/configuration restructuring where needed so policy surfaces align
  with subsystem boundaries
- certification satisfying suites 16 through 18 in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)

Out of scope for Milestone 11:

- bridge-mediated writeback authority
- new truth-side policy ontology beyond consuming canonical admissibility and
  capability facts already produced by the parent runtime
- replacing signal scheduler semantics with bridge-owned scheduling
- making request kind ambient or reconfigurable through policy
- broad host UX or deployment orchestration around policy editing

## Governing Design Rules

### 1. One Policy Declaration Surface Must Begin Every Policy-Aware Bridge Story

Milestone 11 must not allow hosts to assemble policy behavior from scattered
builder flags, request booleans, and diagnostics knobs.

There must be one bridge-owned policy declaration surface that states:

- request identity and request kind
- runtime baseline policy identity
- requested execution mode
- requested diagnostics richness
- requested artifact retention/replay mode
- any preview-policy refinement admitted by the already-typed request kind
- any truth-view or source-policy inputs consumed from prior admitted artifacts

This declaration surface is the only public starting point for policy-aware
admission.

### 2. Policy Sources Must Remain Structurally Separate

At minimum, Milestone 11 must distinguish these policy-source classes:

- `RuntimeBaseline`
- `RequestDeclared`
- `TruthViewAdmitted`
- `SourceCapabilityAdmitted`
- `SpeculationLifecycleAdmitted`
- `BridgeLowered`

Rules:

- policy provenance records the source class for every operative policy fact
- a policy fact admitted from one source class cannot masquerade as another
- absence of a policy fact is distinct from presence overridden by stronger
  authority
- the lowered execution policy must preserve enough provenance to explain why a
  request took the path it did during replay

### 2.1 Policy Precedence Must Be Closed Per Field

Milestone 11 must not leave precedence to local judgment.

For every policy field, the spec and implementation must define exactly one of
these outcomes:

- `Rejected`
- `Narrowed`
- `Inherited`
- `AcceptedAsDeclared`

The minimum precedence order is:

1. structural request-kind boundaries
2. authority-admitted truth/source/speculation constraints
3. request-declared policy
4. runtime baseline policy
5. bridge-lowered derived defaults

Rules:

- higher-precedence layers may constrain lower-precedence layers
- lower-precedence layers may never widen higher-precedence constraints
- if a request asks for a value forbidden by a higher-precedence layer, the
  field must either reject or narrow according to an explicit per-field rule
- every policy field must document whether conflict resolves by rejection or
  narrowing; silent inheritance is legal only when the field was never declared
- provenance must record both the declared source and the winning source for
  every field

At minimum, the spec must carry an explicit per-field table for:

- execution mode
- diagnostics richness
- replay allowance
- artifact retention
- preview-policy refinement

### 3. Validation, Admission, Lowering, And Execution Must Stay Separate

Milestone 11 must move through a real proof chain:

- `BridgePolicyDeclaration`
- `ValidatedBridgePolicyDeclaration`
- `AdmittedBridgePolicyContract`
- `LoweredBridgeExecutionPolicy`
- `BridgePolicyProvenanceRecord`

Rules:

- validation checks declaration shape and source completeness
- admission checks cross-runtime legality and authority compatibility
- lowering computes the monomorphic execution policy consumed by planning and
  execution
- execution consumes lowered policy and must not re-decide legality
- provenance publication is derived from admitted and lowered policy, not from
  host log reconstruction

### 4. Policy Must Refine Structural Boundaries, Not Redefine Them

Policy may refine:

- deterministic versus optimized execution within an admitted structural
  request kind
- diagnostics richness
- replay/retention richness when already permitted by runtime and source
  admissibility
- artifact publication richness

Policy may not redefine:

- preview versus authoritative request kind
- truth-view authority basis
- merge ontology meaning
- source capability truth
- writeback authority

### 4.1 Optimized Mode Must Have Explicit Semantic Boundaries

`Optimized` must not be a semantic loophole.

Milestone 11 must define that `DeterministicCanonical` and `Optimized` may
differ only in explicitly admitted non-canonical surfaces such as:

- packet batching shape
- planning/execution strategy choice
- diagnostics richness
- retained derived artifact richness
- counter profiles expected from different legal strategies

They must not differ in:

- request-kind meaning
- authority basis
- legality or rejection class for the same declared policy bundle
- canonical routing/result bundle meaning
- canonical provenance meaning
- replay-safe policy identity
- whether a flow is preview or authoritative

If an optimization changes any of those semantic surfaces, it is not an
optimization. It is a second semantics and is out of spec.

### 5. Policy Rejection Must Be Typed, Early, And Localized

Milestone 11 must reject illegal combinations before routing, source
materialization, speculative execution, or writeback-shaped preparation begins.

Required rejection families include at minimum:

- `PolicySourceAmbiguity`
- `UnsupportedExecutionMode`
- `ReplayPolicyConflict`
- `DiagnosticsPolicyConflict`
- `ArtifactRetentionConflict`
- `PreviewPolicyBoundaryViolation`
- `TruthViewPolicyConflict`
- `PolicyLoweringMismatch`

Policy rejection must identify:

- the declaration identity
- the relevant request kind
- the conflicting source classes
- the exact policy field that failed
- whether the failure occurred during validation, admission, or lowering

### 5.1 The Policy Legality Matrix Must Be Closed

Milestone 11 must define a closed legality matrix over the admitted policy
fields rather than relying on representative examples.

Rules:

- every validated declaration must reduce to exactly one `Admitted` contract or
  exactly one typed rejection
- there must be no "best effort" or "closest valid" lane
- all unsupported combinations must fail at validation, admission, or lowering
  with a stage-localized rejection class
- the legality matrix must cover at minimum the cross-product of:
  - request kind
  - execution mode
  - replay requirement
  - diagnostics richness
  - artifact retention class
  - preview-policy refinement
  - truth-view/source admissibility classes

Examples of ambiguity that must be resolved explicitly rather than narratively:

- preview request plus replay required plus runtime replay disabled
- authoritative request plus optimized execution plus reduced artifact
  retention
- historical request plus minimal diagnostics plus replay-required provenance
- preview refinement asking for stronger retention than the structural preview
  lifecycle admits

### 6. Provenance Must Explain Both Effective And Non-Effective Policy

Milestone 11 is not complete if provenance only says what won.

For every admitted policy bundle, provenance must make it possible to answer:

- what policy fields were declared
- what policy fields were admitted
- what policy fields were narrowed or ignored
- which source class supplied the operative value
- which source class blocked an alternative value
- whether the final lowered policy was deterministic-safe, replay-safe, and
  request-kind-safe

### 7. Builder Configuration Must Mirror Policy Architecture

The current `BridgeRuntimePolicy` is a useful baseline but too small and too
flat to carry the whole milestone safely on its own.

Milestone 11 should evolve policy configuration toward explicit sections such
as:

- baseline execution policy
- diagnostics baseline
- artifact retention/replay baseline
- preview-policy baseline where admitted

Rules:

- builder order must not change policy meaning
- builder configuration conflicts fail during construction, not later
- runtime baseline policy remains immutable once the bridge is built
- request-time policy can refine only within admitted boundaries

### 8. Replay Must Consume Canonical Policy Records Alone

Replay must not reconstruct policy from ambient runtime state, process
environment, or current builder defaults.

Canonical replay of a policy-aware request must consume:

- the original policy declaration identity
- the admitted contract identity
- the lowered execution policy digest
- the policy provenance record
- any authoritative admissibility digests it depended on

If replay needs current mutable runtime policy to explain what happened, the
milestone has failed.

## Policy Model

Milestone 11 should define a bridge-owned policy model with explicit layers.

### 1. Runtime Baseline Policy

This is the immutable bridge-level default configured at construction time.

It should govern:

- default execution mode class
- default diagnostics tier
- default artifact retention/replay richness
- default operational versus forensic posture

It must not govern:

- request kind identity
- truth-view authority
- source capability truth

### 2. Request Policy Declaration

This is the request-scoped policy ask.

It should declare:

- execution mode ask
- diagnostics richness ask
- artifact retention/replay ask
- any request-scoped policy options admitted for preview or historical work

It must be attached to one specific request identity and one specific request
kind.

### 3. Authority-Admitted Policy Inputs

These are policy-bearing facts derived from already-admitted authoritative
surfaces, such as:

- truth-view replay compatibility
- truth-view retention admission
- source capability admission
- speculation lifecycle admissibility

These are not optional hints. They are hard constraints on legal policy.

### 4. Lowered Execution Policy

This is the one monomorphic policy packet that planning and execution consume.

It should answer at minimum:

- deterministic or optimized execution class
- diagnostics publication class
- replay/retention class
- artifact publication class
- any request-kind-specific policy refinement that survived admission

### 5. Policy Provenance Record

This is the canonical record explaining:

- the declaration digest
- the baseline digest
- the admitted constraint digests
- the lowered policy digest
- the operative source class for each final field
- any narrowed or rejected field classes

### 5.1 Canonical Policy Identity Rules

Milestone 11 must define canonical identity rules for policy declarations,
contracts, lowered policy, and provenance records.

At minimum:

- field ordering must be canonical and independent of builder order or request
  construction order
- omitted fields and explicitly-defaulted fields must have one explicit
  equivalence rule per field
- duplicate declaration of the same semantic field is illegal unless the field
  explicitly admits repeated values with canonical reduction
- source-class ordering inside provenance records must be canonical
- canonical digests must be stable across equivalent hosts and serializers

The spec must say explicitly, for each policy field, whether:

- omission means inheritance
- omission means derived default
- explicit default equals omission
- explicit default differs from omission and therefore changes provenance

Without these rules, replay parity and builder-order parity remain partially
implicit and are out of spec.

## Complexity Contracts

Milestone 11 must name the expected complexity of each hot-path policy
operation and back those claims with counters and proof tests.

At minimum, the spec should treat the following as named contracts:

- policy validation
  - expected bound: proportional to declared policy field count, not prior
    request history
- policy admission
  - expected bound: proportional to declared fields plus admitted authority
    inputs, not broad scans over runtime configuration or request archives
- policy lowering
  - expected bound: proportional to admitted policy width, not downstream
    executor strategy rediscovery
- policy provenance publication
  - expected bound: proportional to admitted fields and source classes, not
    diagnostics retention volume
- policy replay
  - expected bound: proportional to retained canonical policy bundle width, not
    ambient runtime reconstruction

Milestone 11 must add counters sufficient to prove those claims, including at
minimum:

- policy_declaration_field_count
- policy_source_count
- policy_validation_rejection_count
- policy_admission_rejection_count
- policy_lowering_count
- policy_provenance_field_count
- policy_effective_override_count
- policy_ignored_field_count
- policy_replay_request_count
- policy_replay_mismatch_count

The spec must also name bounded richness rules for policy artifacts so
provenance does not become an unbounded hot-path tax.

At minimum:

- canonical policy records must store stable field identities and source-class
  identities rather than repeated free-form explanatory text
- diagnostics explanations may expand canonical policy records, but the
  canonical records themselves must stay bounded by declared field count and
  source count
- richer diagnostics tiers may add derived explanatory detail, but they may not
  widen canonical policy identity or provenance cardinality
- any retained verbose explanation lane beyond the canonical policy record must
  be tier-gated and explicitly excluded from canonical equivalence

Any implementation that scans prior requests, retained diagnostics history, or
ambient mutable runtime state to decide current request policy is out of spec
unless explicitly marked as debt with named proof gaps.

## Phases

### Phase 1: Policy Vocabulary And Authority Lock

Milestone 11 must first define:

- one bridge-owned policy declaration vocabulary
- one bridge-owned policy-source taxonomy
- typed separation between runtime baseline, request declaration, admitted
  authority inputs, lowered execution policy, and provenance record
- typed policy rejection families and rejection-stage identity
- builder-side baseline policy structure that mirrors subsystem boundaries
- the closed legality rules for deterministic-vs-optimized, diagnostics,
  replay, artifact, and preview-policy combinations

This is the hard-foundation workload bucket for the milestone. It should end
with the bridge able to represent policy meaning mechanically, before any
execution-mode-specific convenience APIs or provenance explanation layers are
added.

Phase 1 is complete only when:

- policy source classes are explicit and distinct
- illegal combinations are decidable from typed inputs
- builder baselines and request declarations are structurally separate
- facade growth can occur without inventing new ambient policy state

Phase 1 must not ship:

- lowered execution policy consumed by runtime planning
- provenance publication
- certification suites

### Phase 2: Admission, Lowering, And Provenance

Milestone 11 must then implement:

- validation of request policy declarations
- admission of legal policy bundles against runtime baseline and authority-
  admitted constraints
- lowered execution policy consumed by planning and execution
- canonical policy provenance records for admitted and rejected requests
- bounded counters for declaration width, admission width, provenance width,
  and ignored-versus-operative field counts

This is the policy-execution bridge workload bucket. It should end with the
bridge able to accept one explicit policy declaration, reduce it against real
constraints, and hand one lowered policy packet to planning and execution
without rediscovery.

Phase 2 is complete only when:

- validation, admission, lowering, and provenance are distinct paths
- identical policy inputs lower to identical execution policy digests
- illegal combinations fail typed before execution
- diagnostics richness affects retained detail only, not lowered policy meaning

Phase 2 must not ship:

- milestone closeout certification
- writeback policy behavior
- any executor-side fallback that bypasses typed rejection

### Phase 3: Replay, Configuration Finalization, And Certification

Milestone 11 must finally ship:

- replay-safe canonical policy records and provenance reconstruction
- configuration surfaces that remain explicit and stable under builder-order
  variation
- certification suites 16 through 18 with machine-checkable policy bundles
- proof tests for the named complexity contracts
- diagnostics explanations derived from canonical policy records rather than
  live runtime inspection

This is the closure workload bucket. It should end with policy propagation
being explicit, replay-safe, request-scoped, and certifiable.

Phase 3 is complete only when:

- replay can reconstruct policy provenance from canonical records alone
- builder-order or call-order variation does not change policy meaning
- ambient policy leak scenarios fail the suite unless fixed
- suites 16 through 18 pass with canonical machine-checkable bundles

## Must Ship

- one bridge-owned policy declaration surface
- typed policy-source taxonomy
- typed validation, admission, lowering, and provenance artifacts
- typed rejection classes for illegal policy combinations
- one lowered execution policy artifact consumed by planning and execution
- canonical policy provenance records for admitted and rejected requests
- builder surfaces that keep baseline policy explicit and architecturally
  partitioned
- counters for declaration width, admission width, provenance width, override
  count, ignored-field count, and replay mismatch
- replay-safe policy records and explanations
- certification satisfying Milestone 11 suites 16 through 18

## Must Preserve

- truth runtime remains the authority for truth-view and source admissibility
- signal runtime remains the authority for actual downstream scheduling and
  execution semantics
- bridge policy remains explicit and request-scoped
- no hidden ambient context across request boundaries
- diagnostics richness changes retained detail only, not canonical policy
  meaning
- configuration remains comprehensible at build and call sites

## Acceptance Evidence

Milestone 11 is complete only when the bridge harness can prove:

- deterministic and optimized policy modes can change admitted execution policy
  without ambiguity about which policy surfaces changed
- identical policy inputs produce identical policy digests and provenance
  records across builder-order, call-order, diagnostics-tier, and replay lanes
- illegal policy combinations fail explicitly with typed source-localized
  rejection records before execution begins
- requests do not inherit policy from prior requests or ambient runtime state
- replay reproduces the same lowered policy and provenance from canonical
  policy records alone
- policy admission, lowering, provenance, and replay satisfy the named
  complexity contracts through counter proof tests
- the Milestone 11 certification suites in
  [test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
  pass with canonical machine-checkable bundles

## Architectural Notes

Milestone 11 should extend or restructure the bridge crate with subdomains such
as:

- `policy/`
- `policy/taxonomy.rs`
- `policy/declaration.rs`
- `policy/validation.rs`
- `policy/admission.rs`
- `policy/lowering.rs`
- `policy/provenance.rs`
- `policy/replay.rs`
- `diagnostics/policy.rs`
- `builder/policy.rs`

The current single-file
[policy.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-runtime-bridge/src/policy.rs)
is a valid seed but likely too flat for the completed milestone. The milestone
should preserve the existing public meaning of `BridgeRuntimePolicy` where
possible while decomposing responsibilities beneath it.

Responsibilities should separate as follows:

- `policy/taxonomy.rs`
  - policy-source classes, execution mode classes, rejection kinds, and field
    taxonomies
- `policy/declaration.rs`
  - request-scoped policy declaration and declaration identity
- `policy/validation.rs`
  - declaration-shape validation and source completeness
- `policy/admission.rs`
  - legality checks against runtime baseline and authority-admitted inputs
- `policy/lowering.rs`
  - lowered execution policy construction
- `policy/provenance.rs`
  - canonical policy provenance records and summaries
- `policy/replay.rs`
  - replay-safe policy bundle reconstruction
- `diagnostics/policy.rs`
  - explanations derived from canonical policy artifacts
- `builder/policy.rs`
  - builder-side baseline policy composition and conflict rejection

Phase-local policy types may exist, but only as monotonic projections from
`LoweredBridgeExecutionPolicy`.

Rules:

- a downstream phase may project the lowered policy into a narrower
  phase-specific type
- a downstream phase may not reinterpret legality, widen permissions, or invent
  a new effective policy
- any phase-local projection must be derivable from the lowered policy alone
- if a phase discovers it needs new policy decisions, the lowering phase is
  incomplete and must be expanded rather than patched locally

The bridge facade should expose bridge-owned types such as:

- `BridgePolicyDeclaration`
- `BridgePolicyDeclarationIdentity`
- `ValidatedBridgePolicyDeclaration`
- `AdmittedBridgePolicyContract`
- `BridgePolicySourceClass`
- `BridgeExecutionPolicyClass`
- `LoweredBridgeExecutionPolicy`
- `BridgePolicyProvenanceRecord`
- `BridgePolicyProvenanceEntry`
- `BridgePolicyRejection`
- `BridgePolicyRejectionKind`
- `BridgePolicyCounters`

These names are illustrative, but the separation is mandatory:

- runtime baseline policy is not the same responsibility as request policy
  declaration
- policy admission is not the same responsibility as lowered execution policy
- lowered execution policy is not the same responsibility as provenance
  publication
- canonical policy records are not the same responsibility as diagnostics
  explanation

## Test And Harness Model

Milestone 11 must follow the same structural testing discipline as earlier
bridge milestones and must satisfy the Milestone 11 certification suites in
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md).

Milestone 11 certification must also obey the Milestone 6+ global certification
rules from that document, not just the Milestone 11 suite names.

At minimum, every Milestone 11 certification suite must include:

- `control_lane`
- `hostile_lane`
- `replay_lane`

If the suite is primarily a rejection suite, the hostile lane may terminate in
typed failure, but it must still be compared against a successful or otherwise
semantically equivalent control basis.

Every Milestone 11 certification suite must include all applicable assertion
classes:

- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally different semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden fallback, forbidden ambient policy
  inheritance, or forbidden diagnostics influence

Presence-only bundle checks, non-empty-digest checks, or one-lane-only tests do
not satisfy the milestone.

The harness should expose a request shape able to vary:

- request kind
- runtime baseline policy
- request policy declaration
- diagnostics richness
- replay/retention policy
- truth-view admissibility inputs
- source capability inputs
- preview lifecycle inputs
- builder order and host call order
- pacing and diagnostics-tier perturbations that must preserve canonical policy
  meaning

The harness must encode the mutation-sensitivity rule explicitly:

- at least one perturbation that changes diagnostics richness, pacing, or call
  order without changing canonical policy meaning
- at least one perturbation that changes canonical policy meaning and must
  therefore change a declared digest or structured report
- at least one perturbation that must fail explicitly before semantic drift
  occurs

The certification bundle for Milestone 11 should include at minimum:

- `policy_digest`
- `policy_provenance_report`
- `policy_matrix`
- `request_policy_matrix`
- `routing_digest`
- `replay_digest`
- `diagnostics_digest`
- `counter_snapshot`

These bundles must be offline-sufficient:

- an auditor must be able to determine equality, inequality, typed rejection,
  diagnostics-tier invariance, and counter-contract compliance from the bundle
  alone
- pass/fail judgment must not require host logs, debugger inspection, or live
  runtime state

Counter assertions must be exact for representative scenarios, including
counters that must remain zero. Range assertions are allowed only where the
suite explicitly documents why controlled variability is part of the contract.

The harness must specifically prove:

- policy changes are explicit and attributable rather than ambient
- equivalent policy bundles remain equivalent under builder-order and call-order
  variation
- illegal combinations fail at validation or admission rather than during
  execution
- request interleaving does not leak prior policy
- replay preserves policy-source attribution from canonical records alone

### Suite 16: Policy Provenance Equivalence

Milestone 11 must encode Suite 16 exactly as a certification problem, not just
as a narrative acceptance goal.

Required scenario floor:

- run semantically identical bridge flows under `DeterministicCanonical` and
  `Optimized`
- vary artifact policy explicitly
- vary diagnostics policy explicitly
- replay the admitted policy bundle

Required verification:

- policy provenance artifacts explain exactly which policy surfaces changed
  behavior
- identical policy inputs produce identical `policy_digest` values
- replay preserves policy-source attribution

Required output floor:

- `policy_digest`
- `policy_provenance_report`
- `routing_digest`
- `replay_digest`

### Suite 17: Illegal Policy Combination Rejection

Required scenario floor:

- request incompatible combinations of deterministic mode, optimization mode,
  diagnostics richness, and artifact retention
- vary host-side baseline inputs
- vary truth-side or authority-admitted policy constraints

Required verification:

- invalid combinations are rejected before execution
- failure classes localize `PolicySourceAmbiguity` versus substantive policy
  illegality
- no fallback default hides a bad policy request

Required output floor:

- `policy_matrix`
- `failure_digest`
- `diagnostics_digest`

### Suite 18: Ambient Policy Leak Resistance

Required scenario floor:

- alternate bridge flows with different policy bundles
- interleave branch-local and historical requests
- replay with reordered host call sequences

Required verification:

- each request consumes only its declared policy bundle
- branch-local flows do not inherit stale policy from prior requests
- reordered host execution does not change policy attribution

Required output floor:

- `policy_digest`
- `request_policy_matrix`
- `replay_digest`
- `counter_snapshot`

## Target API And Module Plan

Milestone 11 should add or extend bridge-owned surfaces along these lines:

- facade policy entrypoints
  - validate policy declaration
  - admit policy declaration
  - lower admitted policy
  - replay policy provenance bundle
- policy types
  - declaration
  - admitted contract
  - lowered execution policy
  - provenance record
  - rejection types
  - counters
- builder types
  - baseline execution policy section
  - diagnostics policy section
  - artifact/replay policy section

The facade should not expose:

- ambient mutable policy state
- builder-internal conflict resolution details as the public contract
- executor-internal strategy heuristics as the public policy language
- host-specific deployment flags as bridge-native policy terms

## Anti-Patterns Explicitly Rejected

- policy meaning inferred from prior requests
- one generic "effective policy" bag with no source provenance
- silent fallback from illegal combinations to defaults
- diagnostics-tier changes that alter policy identity
- executor rediscovery of policy legality
- builder-order-sensitive policy meaning
- request kind modeled as just another policy field
- policy explanation that depends on live runtime inspection instead of canonical
  records

## Sequencing Notes

Milestone 11 must land before:

- Milestone 12 bridge-mediated writeback, because writeback requires explicit,
  typed policy legality and replay-safe policy provenance before it can safely
  negotiate deterministic-vs-optimized behavior, artifact retention, and
  authoritative boundaries
- Milestone 13 end-to-end certification, because the bridge is not fully
  certifiable while policy meaning can still leak through builder defaults,
  diagnostics tiers, or request ordering

Milestone 11 builds directly on:

- Milestone 7 source capability and builder-surface groundwork
- Milestone 9 merge-aware authority consumption, which already consumes
  canonical schema-policy outcomes from truth authority
- Milestone 10 preview/request-kind boundaries, which keep policy from having
  to invent structural request classes

Milestone 11 must not attempt to pre-solve:

- writeback strategy authority or idempotence
- new truth-side policy ontology not already exposed as canonical admissibility
  facts
- scheduler replacement inside `worth-signal`
- end-to-end certification bundle unification for every bridge subsystem

Those become stronger because Milestone 11 exists; they do not need to be
productized here.

## Self-Check

- Does the milestone solve a real structural problem or just package work cosmetically?
  - Yes. The bridge already has multiple policy-shaped surfaces; Milestone 11
    turns them into one explicit, typed, replay-safe policy protocol instead of
    leaving them as ambient conventions.
- Is the adversarial constraint precise and load-bearing?
  - Yes. It centers request interleaving, builder-order drift, replay
    sufficiency, and illegal-combination fallback under mixed runtime modes.
- Does the milestone preserve crate authority boundaries?
  - Yes. Truth still owns truth-view and source admissibility; signal still
    owns execution semantics; the bridge owns only policy declaration,
    combination legality, lowering, and provenance.
- Does the milestone define proof obligations, not just implementation tasks?
  - Yes. Policy-source provenance, typed rejection, lowered-policy replay,
    builder-order parity, ambient-leak resistance, and complexity contracts are
    all machine-checkable obligations.
- Could a competent engineer map this spec into honest types, modules, and tests?
  - Yes. The spec names policy declarations, contracts, lowered packets,
    provenance records, rejection classes, counters, and certification bundles
    directly.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  - It belongs here. Preview boundaries had to land first; writeback and full
    certification depend on clean policy propagation landing next.

## Closeout Standard

Milestone 11 is complete only when all of the following are true:

- baseline policy, request policy, admitted authority inputs, lowered execution
  policy, and policy provenance are mechanically distinct
- illegal policy combinations fail typed before execution begins
- lowered execution policy is consumed without executor-side legality
  rediscovery
- identical policy inputs yield identical policy and provenance digests across
  equivalent runs
- requests do not inherit policy from prior requests or ambient runtime state
- replay reconstructs policy provenance from canonical records alone
- diagnostics tiers change retained detail only, not canonical policy meaning
- certification suites 16 through 18 pass with canonical machine-checkable
  outputs

If policy still depends on ambient runtime state, if builder order still
changes meaning, if diagnostics tiers still create a second policy truth, if
illegal combinations still silently fall back, or if replay still needs live
runtime policy to explain what happened, Milestone 11 is not complete.
