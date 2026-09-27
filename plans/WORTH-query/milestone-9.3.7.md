# Milestone 9.3.7 Engineering Spec: Domain Capability Contributions And Canonical Runtime Materialization

> **Status:** Closed on 2026-05-22 via
> [milestone-9.3.7-closeout.md](./milestone-9.3.7-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.6.md](./milestone-9.3.6.md)
>
> **Prior closeout:** [milestone-9.3.6-closeout.md](./milestone-9.3.6-closeout.md)
>
> **Next milestone:** [runtime-api-public-stabilization-plan.md](./runtime-api-public-stabilization-plan.md)
> freezes the ordinary public runtime facade only after Query can accept
> domain-authored capability posture across its major runtime artifact classes
> without forcing domains to mint pseudo-Query layers above the public API.
>
> **Shipped closeout:** [milestone-9.3.7-closeout.md](./milestone-9.3.7-closeout.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make domain-authored capability meaning a
> public Query-owned contribution lifecycle so canonical runtime artifacts stay
> Query-owned even when the semantic posture originates in a downstream domain.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [composition_laws.md](../../docs/coding-guidelines/composition_laws.md)
> - [domain_structure_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
> - [milestone-9.3.5.md](./milestone-9.3.5.md)
> - [milestone-9.3.6.md](./milestone-9.3.6.md)
> - [worth_proof_vision.md](../worth-proof/worth_proof_vision.md)
> - [worth_proof_roadmap.md](../worth-proof/worth_proof_roadmap.md)
> - [Boundary Artifact Taxonomy And Materialization Contracts](../../crates/worth-foundational/docs/boundary-artifact-taxonomy-and-materialization-contracts/README.md)
> - [Diagnostic Outcomes, Subjects, And Rows](../../crates/worth-foundational/docs/diagnostics-and-explanation-ontology/diagnostic-outcomes-subjects-and-rows.md)
> - [Requested, Admitted, And Materialized Profile Progression](../../crates/worth-foundational/docs/profiles-and-policy-vocabulary/requested-admitted-and-materialized-profile-progression.md)
> - [Provenance Layering And Freshness](../../crates/worth-foundational/docs/lineage-provenance-receipts-and-support-truth/provenance-layering-and-freshness.md)

## Goal

Give downstream domains one public way to contribute semantic capability truth
to Query across its major runtime artifact categories:

```text
DomainCapabilityContributionRequest
  -> DomainCapabilityContributionEligibility
  -> AdmittedDomainCapabilityContribution
  -> CanonicalRuntimeMaterialization
  -> AdmissionArtifacts
   | SupportArtifacts
   | WorkflowArtifacts
   | ContinuityArtifacts
   | AftermathArtifacts
   | ExplanationArtifacts
```

This milestone does not make canonical Query artifact constructors public.
It makes Query accept domain-authored contribution artifacts and materialize
the canonical public runtime artifacts itself.

## Why This Milestone Exists

Milestone 9.3.5 made intent admission one canonical Query decision lattice.
Milestone 9.3.6 made lower-runtime contact one canonical Query routing and
boundary-envelope model.

What remains open is the general domain contribution seam between those two
truths.

Today Query can own:

- canonical admission decisions
- canonical lower-runtime routing and boundary envelopes
- canonical support inventories and certification traces
- canonical graph-composition capability and invariant-facing runtime artifacts

But a serious downstream domain still cannot publicly contribute its own
semantic runtime posture across several recurring categories without bad
outcomes.

The obvious current gap is admission:

- this declaration is advisory for domain reasons
- this declaration is violating for domain reasons

But the same architectural gap also exists for:

- support and traceability
- invariant and capability posture
- workflow / preview posture
- continuity / lineage posture
- consequence / aftermath posture
- explanation / inspection posture

Without 9.3.7:

- every serious domain runtime will invent its own local Query-shaped adapter
  layer
- each domain will encode capability posture in a different taxonomy
- the public runtime facade will freeze before domains can plug real semantic
  posture into it
- later Query families will rediscover the same domain-contribution problem one
  category at a time

This is a platform defect, not a one-domain defect.

## Governing Summaries

- `MENTALITY.md`: solve the reusable platform problem once. Do not bless local
  folklore layers as permanent architecture.
- `arch_laws.md`: canonical artifacts must stay authority-owned; binary
  success/failure is insufficient; boundary artifacts must be self-describing.
- `composition_laws.md`: authoring, eligibility, materialization, DX, support,
  and certification are distinct responsibilities and must be named separately.
- `domain_structure_laws.md`: the tree must make the difference between
  domain-authored meaning and Query-authored canonical runtime artifacts
  physically obvious.
- `perf_laws.md`: declaration-scoped materialization must scale with
  contribution width, trace width, and evidence width, not unrelated runtime
  breadth.
- `worth_query_vision.md`: Query is the ordinary public runtime layer. Domains
  should not rebuild runtime-facing artifact systems above it.
- `milestone-9.3.5.md`: Query already owns the admission lattice. 9.3.7 adds
  a public contribution seam that feeds that lattice and adjacent artifact
  families rather than replacing them.
- `milestone-9.3.6.md`: Query already owns lower-runtime routing. 9.3.7 must
  feed that routed runtime story rather than inventing a second domain-local
  runtime boundary model.
- `worth_proof_vision.md`: `worth-proof` owns progression law, sealed proof
  minting, typed denial, and lowering-vs-execution boundaries. 9.3.7 must use
  it for the contribution lifecycle instead of inventing another local
  progression substrate inside Query.
- `worth_proof_roadmap.md`: the contribution seam should use real proof-bearing
  requested/admitted/materialization-ready forms, not a generic mutable bag
  plus conventions.
- `Boundary Artifact Taxonomy And Materialization Contracts`: summaries,
  reports, artifacts, and support bundles must stay distinct. 9.3.7 must not
  collapse declaration-scoped support and traceability into one generic
  envelope.
- `Diagnostic Outcomes, Subjects, And Rows`: diagnostic and support
  materialization should use foundational row vocabulary where the new Query
  surfaces emit descriptive boundary meaning.
- `Requested, Admitted, And Materialized Profile Progression`: profile
  narrowing across the new contribution surfaces must be explicit and proof
  bearing where richness, support posture, or delivery posture change.
- `Provenance Layering And Freshness`: materialized contribution artifacts
  must expose honest provenance/freshness posture instead of free-form
  domain-authored explanation strings.

## Adversarial Constraint

Under equivalent domain-authored capability meaning, alternate builder paths,
alternate evidence orderings, alternate declaration construction paths,
alternate richness profiles, and alternate category-specific materialization
paths, Query must materialize the same canonical runtime artifacts.

If a domain can only express its semantic posture by:

- minting local pseudo-Query artifacts,
- calling crate-private Query constructors,
- flattening meaning into free-form strings or JSON,
- mutating global support or certification inventories directly,
- or bypassing the public Query intent/runtime lifecycle,

then this milestone fails.

## Product Decision Lock

- Query remains the sole owner of canonical public runtime artifacts.
- Domains own semantic meaning and contribution evidence, not final artifact
  authority.
- Existing direct constructors for canonical admission decisions remain
  non-public. This milestone does not solve the gap by making canonical
  artifact constructors public.
- The contribution seam is category-general, but the materialization targets
  remain category-specific and canonical.
- Domain contribution posture is declaration-scoped or admitted-family-scoped,
  not a mutation of global support matrices, certification inventories, or
  static capability tables.
- The typed contribution target-binding family introduced here is the first
  shipped slice of a broader Query-owned binding substrate. Later `9.3.8`
  declaration-entry, orchestration, continuation, and ergonomic phases must
  generalize this family rather than introducing a second public binding
  vocabulary beside it.
- The first such generalization is the immediate post-Phase-24 shared
  retained target-binding extraction, so this seam must survive rebasing onto
  that shared core without changing contribution semantics.

## WORTH Proof And Foundational Integration Locked Now

`9.3.7` does not just add a Query-local contribution feature. It also starts
using the shared WORTH substrates where they now clearly belong.

### `worth-proof` is mandatory for contribution phase law

The new contribution lifecycle is a proof-bearing progression surface.

That means `9.3.7` must use `worth-proof` for:

- category-specific requested contribution forms
- contribution eligibility and admission progression
- admitted contribution forms
- materialization-ready progression boundaries
- typed denial / stale / rebind-required / failed progression outcomes
- sealing stronger forms so ordinary callers cannot mint them directly

Concrete locked rule:

- `WORTHQueryDomainCapabilityContributionRequest`
- `WORTHQueryDomainCapabilityContributionEligibility`
- `WORTHQueryAdmittedDomainCapabilityContribution`
- `WORTHQueryCanonicalRuntimeMaterialization`

must be modeled as real proof-bearing progression stages, not just ordinary
payload structs with naming discipline.

Concrete consequence:

- category-specific request types should lower through `worth-proof` phase law
- category-specific admitted contribution types should be stronger than
  request-time forms in the type system
- wrong progression order must be structurally uncallable
- all category materialization outcomes must come back through typed
  progression/transition outcomes, not string errors or booleans

`9.3.7` does **not** require retrofitting every pre-existing `9.3.5` and
`9.3.6` Query API to `worth-proof` in this milestone. It does require that
all new contribution progression surfaces introduced by `9.3.7` use
`worth-proof` from day one.

### `worth-foundational` is mandatory for new descriptive boundary meaning

The new contribution seam also materializes descriptive boundary meaning.

That means `9.3.7` must use `worth-foundational` for:

- boundary artifact taxonomy
- declaration-scoped diagnostic/support rows
- profile progression where the requested/admitted/materialized surface may
  narrow
- provenance and freshness posture on materialized contribution artifacts

Concrete locked rule:

- use foundational boundary category discipline so summaries, reports,
  artifacts, and support bundles remain distinct
- use foundational diagnostic/support row vocabulary for new
  declaration-scoped descriptive outputs
- use foundational profile progression for any new richness/support/delivery
  narrowing across request -> admitted -> materialized phases
- use foundational provenance/freshness vocabulary on materialized
  declaration-scoped support, advisory, capability, and invariant outputs

`9.3.7` does **not** require rewriting every older canonical Query artifact to
be re-founded on `worth-foundational` in this milestone. It does require that
new descriptive surfaces introduced by this contribution seam use
`worth-foundational` instead of inventing another local descriptive taxonomy.

### Division of responsibility is fixed

- `worth-proof` owns progression law
- `worth-foundational` owns descriptive boundary language
- `worth-query` owns canonical runtime artifacts and the public materializers

This milestone fails if any one of those three layers tries to absorb the
others.

## Exact Substrate Usage

This section is intentionally implementation-tight.

It exists so a later engineer does not have to infer what "use
`worth-proof`/`worth-foundational`" means from architecture prose alone.

### Exact `worth-proof` usage

Use the real `worth-proof` substrate:

- [crates/worth-proof/src/artifact/carrier.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-proof/src/artifact/carrier.rs)
  - `Artifact<P, T, S = NoProofs, A = NoAssumptionBasis>`
- [crates/worth-proof/src/transition/outcomes.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-proof/src/transition/outcomes.rs)
  - `TransitionOutcome<S, D, De, St, R, F>`
- [crates/worth-proof/src/proof/witnesses.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-proof/src/proof/witnesses.rs)
  - `AuthorityWitness`
  - `CapabilityWitness`

The contribution lifecycle should be built as proof-bearing artifacts, not as
plain structs that merely sound phased.

Required phase shapes:

- request phase
  - `Artifact<ContributionRequested, ContributionPayload, ...>`
- eligibility/admission-ready phase
  - `Artifact<ContributionEligible, ContributionPayload, ...>`
- admitted phase
  - `Artifact<ContributionAdmitted, ContributionPayload, ...>`
- materialization-ready phase
  - `Artifact<ContributionMaterializationReady, ContributionPayload, ...>`

The exact Rust phase-marker names may differ, but all four phase boundaries
must exist as real proof-bearing forms.

Required transition shape:

- request -> eligible
- eligible -> admitted
- admitted -> materialization-ready
- materialization-ready -> canonical Query artifact materialization

`worth-proof` must own the first three boundaries directly.
The fourth boundary may terminate inside Query-owned canonical materializers,
but the input to that materializer must still be the stronger
materialization-ready proof-bearing form.

Required `TransitionOutcome` mapping:

- `Success`
  - category request admitted correctly
  - canonical materialization completed correctly
- `Denied`
  - malformed contribution
  - illegal category/target combination
  - illegal strengthening or category misuse
  - semantically invalid domain contribution
- `Stale`
  - only when the contribution basis is freshness-sensitive and the basis
    actually drifted
- `RebindRequired`
  - only when declaration/admitted-plan attachment is no longer valid under
    its original binding basis and must be reattached
- `Failed`
  - integrity/construction/runtime-internal failure
  - never ordinary semantic denial

Locked rule:

- no category may use `Deferred` as a substitute for unfinished implementation
- `Stale` and `RebindRequired` are allowed only for target/basis-sensitive
  contribution families
- if a category cannot honestly become stale or rebind-required, its
  contribution lane should not invent those branches

### Exact `worth-foundational` usage

Use the real foundational surfaces that already exist today:

- diagnostics row families from
  [crates/worth-foundational/src/facade.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-foundational/src/facade.rs)
  - `FoundationalDiagnosticDecisionRow`
  - `FoundationalDiagnosticSupportRow`
  - `FoundationalDiagnosticProvenanceReadyRow`
- profile progression from
  [crates/worth-foundational/src/profiles/progression.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-foundational/src/profiles/progression.rs)
  - `request_foundational_profile_set(...)`
  - `admit_requested_foundational_profile(...)`
  - `materialize_admitted_foundational_profile(...)`
- provenance from
  [crates/worth-foundational/src/boundary_evidence/front_doors/mod.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-foundational/src/boundary_evidence/front_doors/mod.rs)
  - `boundary_evidence().provenance()`
  - `FoundationalBoundaryEvidenceSourceBasis`
  - `FoundationalBoundaryEvidenceFreshnessPosture`
- boundary taxonomy from
  [Boundary Artifact Taxonomy And Materialization Contracts](../../crates/worth-foundational/docs/boundary-artifact-taxonomy-and-materialization-contracts/README.md)

Query should compose these pieces. It should not invent a parallel
Query-local substitute vocabulary where these already fit.

Required foundational composition in `9.3.7`:

- declaration-scoped advisory/violation/support outputs
  - use `FoundationalDiagnosticDecisionRow`
- declaration-scoped capability-gap and narrowing outputs
  - use `FoundationalDiagnosticSupportRow`
- declaration-scoped evidence-origin outputs
  - use `FoundationalDiagnosticProvenanceReadyRow`
- descriptive output richness/support narrowing
  - use foundational profile progression
- materialized support/advisory/capability/invariant output basis disclosure
  - use foundational provenance/freshness vocabulary
- output kind separation
  - preserve summary/report/artifact/support bundle distinctions per the
    foundational boundary taxonomy

### Exact profile progression expectations

`9.3.7` does not get to invent arbitrary profile drift.

Required profile families in scope now:

- `DiagnosticRichnessProfile`
- `SupportPostureProfile`

Conditionally allowed only when the artifact is actually retained or delivered:

- `RetentionDeliveryProfile`

Not required in `9.3.7` unless a concrete new materialized surface truly needs
them:

- `CertificationPostureProfile`
- `CompatibilityPostureProfile`
- any other foundational profile family outside the immediate contribution
  descriptive surface

Locked narrowing rule:

- requested -> admitted may narrow at most one family
- admitted -> materialized may narrow at most one family
- narrowing must be explicit and recorded
- silent strengthening is forbidden

### Exact provenance posture expectations

Use foundational provenance/freshness on materialized descriptive outputs with
this default mapping:

- direct domain-authored advisory/support meaning
  - direct/current posture
- inferred or policy-shaped support/advisory meaning
  - derived/current posture
- stale contribution basis
  - stale retained posture
- replay or reconstructed explanatory support
  - reconstructed/replay-derived posture only when that is actually true

Locked rule:

- provenance must describe basis and freshness
- provenance must not pretend to be execution proof
- receipts and closeout truth remain outside this milestone unless a later
  Query family explicitly owns them

### Legacy carve-outs locked now

`9.3.7` starts using the substrates for the new contribution seam.
It does not reopen or rewrite every older Query surface.

Do not retrofit in `9.3.7`:

- existing `9.3.5` canonical decision constructors
- existing `9.3.6` lower-runtime boundary envelope types
- pre-existing Query artifact families that are only being consumed as
  materialization targets

Do require in `9.3.7`:

- every new contribution progression surface uses `worth-proof`
- every new descriptive contribution surface uses `worth-foundational`
- every new ordinary public DX lane lowers honestly into those substrates

## Capability Categories Locked Now

9.3.7 defines one shared contribution pattern across these categories:

1. **Admission**
   - advisory posture
   - violation posture
   - admission-local support posture
2. **Support And Traceability**
   - declaration-scoped support rows
   - declaration-scoped support narrowing / degradation
   - traceability and evidence posture
3. **Invariant And Capability**
   - domain capability gaps
   - domain invariant denials
   - invariant-support summaries
   - ordinary Query-facing registration of custom/core domain invariants that
     lower into relational authority
4. **Workflow / Preview**
   - preview-only
   - promotion-eligible
   - discard-required
   - confirmation or escalation-required
5. **Continuity / Lineage**
   - identity preserved
   - identity split
   - identity replaced
   - semantic correspondence only
6. **Consequence / Aftermath**
   - established fact families
   - target-binding posture
   - aftermath consequence posture
7. **Explanation / Inspection**
   - fallback posture
   - ambiguity posture
   - rejected-alternative posture
   - explanation-only semantic context

Every category listed above must ship as a fully closed member of the shared
Query domain-capability seam in this milestone.

That means each category must receive:

- real public contribution authoring
- real admitted proof types
- real canonical materializers
- real hostile certification
- real runtime-facing artifacts or runtime-facing descriptive outputs where the
  category is descriptive by nature

`9.3.7` is not allowed to close with a split between "real categories now" and
"typed-but-not-finished categories later." If a category is named here, it
must be finished here.

## Locked Scope Decisions

### 1. `WORTHQueryIntentDeclaration` remains the shared runtime declaration carrier

Domains already lower runtime-facing work into `WORTHQueryIntentDeclaration`.
9.3.7 does not replace that carrier. It binds contributions to one declaration
or one already-admitted Query family.

Concrete consequence:

- public contribution authoring must bind through one typed target family,
  not raw strings and not digest-only attachment
- the minimum target shapes are:
  - declaration-bound
  - admitted-plan-bound
  - boundary-envelope-bound only for categories whose semantics honestly occur
    after routing

Digest strings may appear in certification and canonical outputs, but they may
not be the primary authoring/binding contract.

### 2. `WORTHQueryRawIntentAdmissionRequest` remains the generic entry request

9.3.7 does not add a second public runtime front door. The contribution seam
attaches to the existing Query runtime story rather than replacing it.

### 3. Canonical runtime artifacts remain sealed and Query-owned

The current public/private split around canonical artifacts such as:

- `WORTHQueryIntentAdvisoryDecision`
- `WORTHQueryIntentViolationDecision`

is correct and remains locked. 9.3.7 adds public contribution authoring and
public canonical materializers, not public canonical constructors.

Concrete consequence:

- the public contribution boundary must be category-typed
- materializers must consume category-proof types, not one open generic bag
- a public `Other(String)`, `Other(Value)`, or equivalent open escape hatch is
  forbidden at the contribution boundary

### 4. Declaration-scoped support becomes a first-class runtime surface

The existing support matrix and certification traceability reports are not a
substitute for one declaration saying "this exact runtime meaning is advisory
or narrowed for these domain reasons." 9.3.7 adds declaration-scoped runtime
support surfaces rather than overloading inventory or certification surfaces.

### 5. Existing canonical runtime artifact families are materialization targets

Public Query runtime artifacts such as:

- `WORTHQueryIntentAdvisoryDecision`
- `WORTHQueryIntentViolationDecision`
- `WORTHQueryGraphCompositionCapabilitySupportRow`
- `WORTHQueryGraphCompositionDomainInvariantDenial`

already exist as canonical Query-owned runtime artifacts. 9.3.7 uses them as
materialization targets where they fit and introduces new canonical artifact
families where the broader contribution categories require them.

### 6. Query owns the ordinary facade for domain invariant registration

This milestone also claims the ordinary public Query-facing registration lane
for downstream domain invariants that ultimately execute in relational
authority.

The exact lower-level relational surfaces touched today are:

- [crates/worth-relational/src/logic/builder.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-relational/src/logic/builder.rs)
  - `LogicRuntimeBuilder::invariant_catalog(...)`
  - `LogicRuntimeBuilder::custom_invariant(...)`
  - owned data lane:
    - `CustomInvariantRegistration`
    - `InvariantCatalog`
- [crates/worth-relational/src/facade.rs](/C:/Users/shepworth/Documents/programming/WORTH-2/crates/worth-relational/src/facade.rs)
  - exported invariant authoring types:
    - `CustomInvariantRegistration`
    - `CustomInvariantRule`
    - `CustomInvariantDescriptor`
    - `CustomInvariantExecutionContext`
    - `CustomInvariantScopePlanner`
    - `CustomInvariantVerdict`
    - `CustomInvariantRegistrationError`
    - `CustomInvariantRuleId`
    - `CustomInvariantSemanticIdentity`
    - `CustomInvariantSemanticVersion`
    - `CustomInvariantOperationalMetadata`
    - `InvariantExecutionPoint`
    - `InvariantFailureEffect`
    - `InvariantGroup`
    - `InvariantGroupSet`
    - `InvariantCostClass`

This is the real lower-level registration seam.

`9.3.7` does **not** move invariant authority out of `worth-relational`.
It does require Query to own the ordinary public runtime-facing authoring and
registration surface for domains that otherwise would have to import the
relational builder directly.

Concrete consequence:

- ordinary domain authors should not need to call
  `LogicRuntimeBuilder::custom_invariant(...)` directly for Query-integrated
  runtime capability authoring
- Query should expose one typed invariant-registration contribution lane that
  lowers into `CustomInvariantRegistration` / `InvariantCatalog` under the hood
- the relational builder lane remains the contained lower-level surface and
  escape hatch, not the primary Query-integrated DX
- `invariant_access()` / validation inspection is a separate contained lane
  and is not the registration seam this milestone is claiming

## Must Ship

- one public domain capability contribution lifecycle with at least these
  artifact families:
  - `WORTHQueryDomainCapabilityContributionRequest`
  - `WORTHQueryDomainCapabilityContributionEligibility`
  - `WORTHQueryAdmittedDomainCapabilityContribution`
  - `WORTHQueryCanonicalRuntimeMaterialization`
- one real `worth-proof` progression implementation for the new contribution
  lifecycle so request, eligibility, admission, and materialization-ready
  forms are proof-bearing rather than conventional
- typed contribution classes for:
  - admission posture
  - support / traceability posture
  - invariant / capability posture
  - workflow / preview posture
  - continuity / lineage posture
  - consequence / aftermath posture
  - explanation / inspection posture
- sealed category-specific request, admitted, and materializer pairs so:
  - admission contributions can only feed admission materializers
  - support contributions can only feed support materializers
  - invariant / capability contributions can only feed invariant materializers
  - workflow contributions can only feed workflow materializers
  - continuity contributions can only feed continuity materializers
  - aftermath contributions can only feed aftermath materializers
  - explanation contributions can only feed explanation materializers
- canonical materializers that consume admitted domain contributions and
  produce:
  - canonical admission artifacts
  - canonical declaration-scoped support and traceability artifacts
  - canonical capability / invariant artifacts where applicable
  - canonical workflow / preview artifacts
  - canonical continuity / lineage artifacts
  - canonical consequence / aftermath artifacts
  - canonical explanation / inspection artifacts
- foundational-backed descriptive surfaces for all new declaration-scoped
  support, traceability, advisory, capability, and invariant materialization:
  - boundary categories remain explicit
  - row vocabulary remains explicit
  - provenance/freshness remains explicit
  - profile narrowing remains explicit where applicable
- one DX surface for ordinary domain authors so they can:
  - author a typed contribution
  - bind it to a declaration or admitted family
  - request canonical materialization
  - inspect typed denial when the contribution is malformed, illegal for its
    target, stale, or rebind-required
- one ordinary Query-facing invariant registration lane that:
  - authors a domain invariant through Query-facing vocabulary
  - lowers into the relational `CustomInvariantRegistration` /
    `InvariantCatalog` seam
  - keeps relational invariant authority explicit and inspectable
- one certification and compile-boundary story proving domains can contribute
  meaning publicly without minting canonical Query artifacts directly

## Must Preserve

- Query owns canonical artifact construction and digests.
- Domains can contribute semantic meaning without owning artifact authority.
- `worth-proof` remains responsible only for contribution progression law, not
  for canonical runtime artifact vocabulary or descriptive reporting.
- `worth-foundational` remains responsible only for descriptive boundary
  meaning, not for runtime artifact authority or execution routing.
- Equivalent contribution meaning must normalize to the same canonical Query
  artifact regardless of builder path or domain adapter shape.
- One declaration or admitted plan must accumulate contributions exactly once
  into one canonical contribution summary before category-specific
  materialization occurs.
- Declaration-scoped support materialization must not silently mutate the
  global support matrix or certification inventory.
- The contribution seam must compose with 9.3.5 decision traces and 9.3.6
  lower-runtime boundary envelopes instead of creating parallel explanation
  models.

## Must Not Ship

- public `new(...)` constructors for canonical runtime artifacts solely to let
  domains participate
- domain-owned pseudo-Query runtime wrappers presented as the official runtime
  surface
- opaque JSON contribution blobs as the only public authoring format
- one generic admitted contribution type plus runtime switches standing in for
  sealed category-specific proof types
- a new Query-local proof substrate for contribution phase law when
  `worth-proof` already covers the requirement
- global support-matrix mutation as a substitute for declaration-scoped
  materialization
- a generic `Other(String)` catch-all that makes category meaning invisible to
  the type system
- a new Query-local descriptive taxonomy for support/provenance/profile
  progression when `worth-foundational` already covers the requirement

## Phase Contract

```text
WORTHQueryIntentDeclaration | AdmittedIntentFamily
  -> ProofBearingDomainCapabilityContributionRequest
  -> ProofBearingDomainCapabilityContributionEligibility
  -> ProofBearingAdmittedDomainCapabilityContribution
  -> DomainCapabilityContributionSummary
  -> ProofBearingCanonicalRuntimeMaterialization
  -> AdmissionArtifacts
   | SupportArtifacts
   | WorkflowArtifacts
   | ContinuityArtifacts
   | AftermathArtifacts
   | ExplanationArtifacts
  -> RoutedRuntimeBoundaryOrTypedStop
```

The proof-bearing stage names above are architectural obligations even if the
final Rust type names stay shorter. The key rule is that the lifecycle must be
real proof progression backed by `worth-proof`, not ordinary structs with
phase-like names.

## Target DX

The DX laws apply directly to this milestone.

The public surface must not expose â€œdomain capability contributionsâ€ as a bag
of generic builders. It must expose:

1. one obvious common semantic lane
2. one checked / inspectable lane
3. one explicit proof-bearing lane
4. one raw substrate escape hatch

The common lane must read like intent.
The checked lane must expose the next lower semantic layer.
The proof lane must preserve exact category-specific proof progression.
The raw lane must remain available without pretending to be the common path.

### Common Lane

The common lane is the ordinary caller story.

Representative direction:

```rust
let advisory = query
    .domain("worth.spatial")
    .for_intent(&declaration)
    .advises("arbitration.requires_clarification")
    .because("multiple spatial candidates remain admissible")
    .materialize()?;

let denial = query
    .domain("worth.spatial")
    .for_intent(&declaration)
    .violates_invariant("spatial.non_manifold_edge_split")
    .because("result would introduce non-manifold topology")
    .materialize()?;

let support = query
    .domain("worth.spatial")
    .for_intent(&declaration)
    .supports_capability("graph.face_inner_loop_insertion")
    .because("topology substrate is available")
    .materialize()?;

query
    .domain("worth.spatial")
    .register_invariant(
        query
            .invariant("spatial.non_manifold_edge_split")
            .at_commit_boundary()
            .blocks_commit()
            .in_group("topology")
            .with_rule(SpatialInvariantHooks::non_manifold_edge_split),
    )?;
```

Required common-lane properties:

- one obvious entrypoint
- semantic verbs, not generic category bags
- no required exposure of internal proof nouns at the call site
- no stringly binding to declarations as the primary target model
- no forced lower-runtime or certification vocabulary in ordinary authoring

### Checked Lane

The checked lane must make ordinary branching easier without collapsing the
semantic topology into a plain `Result`.

Representative direction:

```rust
let outcome = query
    .domain("worth.spatial")
    .for_intent(&declaration)
    .preserves_continuity("identity.split")
    .because("edge split replaces one edge with two")
    .try_materialize();

match outcome.kind() {
    DomainCapabilityOutcomeKind::Materialized => { /* ... */ }
    DomainCapabilityOutcomeKind::RebindRequired => { /* ... */ }
    DomainCapabilityOutcomeKind::Denied => { /* ... */ }
}
```

Required checked-lane properties:

- preserve typed distinction between materialized, denied, stale,
  rebind-required, and failed posture where those branches honestly exist
- expose category, target, and support posture without re-running authoring
- make every category ergonomic on the ordinary and inspectable lanes
- never collapse the semantic topology into plain `Result<T, E>`
- surface the real proof-bearing progression outcome underneath rather than
  hiding it behind ad hoc Query-local enums where `worth-proof` already gives
  the honest lower lane

### Proof Lane

The proof lane is the precise authoring and progression surface for callers
that need explicit phase control.

Representative direction:

```rust
let authored = WORTHQueryDomainAdvisoryContribution::for_declaration(
    &declaration,
    "arbitration.requires_clarification",
)
.because("multiple spatial candidates remain admissible");

let eligibility = authored.check()?;
let admitted = authored.admit(eligibility)?;
let summary = admitted.summarize();
let artifact = admitted.materialize()?;
```

Required proof-lane properties:

- category-specific request types
- category-specific admitted proof types
- category-specific materializers
- no generic admitted contribution blob plus runtime switches
- compile-time category mismatch when the wrong materializer is applied
- progression implemented through `worth-proof` rather than a Query-local
  imitation typestate substrate

### Raw Lane

The raw lane remains available as the substrate escape hatch.

Representative direction:

```rust
let request = WORTHQueryDomainCapabilityContributionRequest::declaration_bound(
    declaration_binding,
    category_specific_payload,
);
let eligibility = request.evaluate()?;
let admitted = request.admit(eligibility)?;
let materialization = admitted.materialize_with(policy)?;
```

Required raw-lane properties:

- explicit and grepable
- no hidden global state or ambient authority
- visibly weaker and lower-level than the common lane
- suitable for certification, adapters, and advanced integration work

### Degradation Ladder

The public API must degrade one step at a time:

1. common semantic lane
2. checked / inspectable lane
3. proof-bearing lane
4. raw substrate lane

It must not degrade like this:

1. cute common helper
2. immediate fall to raw generic plumbing

### Explanation Surface

Materialized artifacts must expose explanation at their own semantic level.

Representative direction:

```rust
let artifact = query
    .domain("worth.spatial")
    .for_intent(&declaration)
    .violates_invariant("spatial.non_manifold_edge_split")
    .because("result would introduce non-manifold topology")
    .materialize()?;

let explanation = artifact.explain();

explanation.category();
explanation.target();
explanation.trace();
explanation.supporting_capabilities();
explanation.invariant_context();
```

Explanation must not require callers to spelunk raw trace bags or lower-runtime
internals to understand the artifact they just authored.

Where explanation emits descriptive boundary meaning rather than runtime
authority meaning, it should reuse `worth-foundational` diagnostic/support/
provenance vocabulary instead of inventing a Query-only explanation row family.

### Golden Transcript Obligation

This milestone must add compile-checked golden DX transcripts for at least:

- advisory admission materialization
- violation admission materialization
- declaration-scoped support materialization
- invariant / capability materialization
- ordinary Query-facing invariant registration lowering into relational
  registration
- workflow / preview contribution materialization
- continuity / lineage contribution materialization
- consequence / aftermath contribution materialization
- explanation / inspection contribution materialization

Those transcripts are not cosmetic examples. They are part of the proof that
the intended caller ergonomics are real and synchronized with the executable
surface.

## Required Topology

Milestone 9.3.7 should map into responsibility-specific subdomains inside the
Query runtime contribution surface.

Required subdomains:

- `domain_capabilities/authoring`
- `domain_capabilities/eligibility`
- `domain_capabilities/materialization`
- `domain_capabilities/summary`
- `domain_capabilities/trace`
- `domain_capabilities/support`
- `domain_capabilities/workflow`
- `domain_capabilities/continuity`
- `domain_capabilities/aftermath`
- `domain_capabilities/explanation`
- `domain_capabilities/certification`
- `domain_capabilities/proof_integration`
- `domain_capabilities/foundational_integration`

Forbidden topology:

- one giant `domain_capabilities.rs` bag mixing authoring, eligibility,
  materialization, DX, and certification
- one generic runtime-switched contribution blob that carries all categories as
  open data instead of sealed category-specific request and admitted types
- one domain-specific subtree for each client domain
- public surfaces that expose canonical artifact internals more broadly just to
  let domains participate

## Sequencing Notes

This belongs after Milestone 9.3.6 because contributions must feed the
already-canonical lower-runtime routing and boundary-envelope model instead of
inventing parallel domain-local runtime adapters.

It belongs before the Runtime API Public Stabilization Gate because freezing
the runtime facade before this seam exists would force every serious domain to
build its own pseudo-Query capability layer on top of the public API.

It belongs before Milestone 9.4 because temporal and async/runtime expansion
should inherit one public domain capability seam rather than adding
family-specific contribution folklore later.

## Implementation Phases

This milestone is not a buffet.

The engineer must implement it in the phase order below and must not skip
forward just because a later phase looks more immediately useful for one
domain.

Each phase exists because the next phase depends on it structurally.
If a phase is not complete, later work will either re-invent local substrate
or produce public surfaces that must be rewritten.

### Phase 1: Build the proof-bearing contribution core

Start here and finish it completely before any category-specific public
materializer work.

Deliverables:

- create the contribution subtree and phase markers
- implement the proof-bearing request -> eligible -> admitted ->
  materialization-ready lifecycle using the real `worth-proof` substrate
- define category-specific contribution request payload families
- define category-specific admitted contribution families
- define the shared typed target-binding family:
  - declaration-bound
  - admitted-plan-bound
  - boundary-envelope-bound only where allowed by the locked scope decisions
- implement the exact `TransitionOutcome` mapping locked earlier in this spec

Practical rule:

- by the end of this phase, an engineer must be able to author a contribution,
  bind it to the right target family, and move it through proof-bearing
  progression without any canonical Query artifact materialization yet

What must be true before moving on:

- the contribution lifecycle is real `worth-proof`, not local imitation
- the typed target-binding family established here is suitable for later
  promotion into the shared Query binding substrate rather than a one-off
  contribution-only helper seam
- category mismatch is already structurally uncallable
- all categories can reach admitted form through the same proof-bearing
  lifecycle
- compile-fail tests prove callers cannot mint stronger forms directly

Do not begin Phase 2 until this is true.

### Phase 2: Build the foundational descriptive layer across all categories

Only after the proof-bearing core is stable.

Deliverables:

- implement declaration-scoped descriptive materialization across the
  categories that emit descriptive boundary meaning:
  - admission
  - support / traceability
  - invariant / capability
  - workflow / preview
  - continuity / lineage
  - consequence / aftermath
  - explanation / inspection
- compose the real `worth-foundational` pieces:
  - `FoundationalDiagnosticDecisionRow`
  - `FoundationalDiagnosticSupportRow`
  - `FoundationalDiagnosticProvenanceReadyRow`
  - foundational profile progression
  - foundational provenance/freshness
  - foundational boundary category discipline
- define the exact mapping from contribution outcome to:
  - row family
  - profile progression usage
  - provenance/freshness posture

Practical rule:

- by the end of this phase, an engineer must be able to materialize
  declaration-scoped descriptive outputs for every category that needs them
  without inventing any new Query-local row taxonomy

What must be true before moving on:

- descriptive outputs use foundational rows instead of ad hoc structs
- richness/support narrowing is explicit and uses foundational profile
  progression where it actually occurs
- provenance/freshness is explicit on new descriptive outputs
- summary/report/artifact/support bundle distinctions are preserved
- no older Query artifact had to be rewritten just to make this phase pass

Do not begin Phase 3 until this is true.

### Phase 3: Materialize canonical Query artifacts across all categories

Only after the proof-bearing core and foundational descriptive layer are both
stable.

Deliverables:

- build the public canonical materializers for all categories:
  - admission contributions -> canonical admission artifacts
  - support/traceability contributions -> canonical declaration-scoped support
    artifacts
  - invariant/capability contributions -> canonical capability/invariant
    artifacts
  - workflow contributions -> canonical workflow / preview artifacts
  - continuity contributions -> canonical continuity / lineage artifacts
  - aftermath contributions -> canonical consequence / aftermath artifacts
  - explanation contributions -> canonical explanation / inspection artifacts
- wire the contribution seam into existing Query-owned canonical artifact
  families where they already fit
- add any missing canonical artifact family only where the existing runtime
  families are genuinely insufficient

Practical rule:

- by the end of this phase, a domain should be able to use the common Query
  DX lane and obtain real canonical artifacts for every named category without
  touching crate-private constructors

What must be true before moving on:

- canonical artifact constructors remain sealed
- public Query authoring can now reach canonical runtime artifacts honestly
  across all named categories
- equivalent contribution meaning canonicalizes to the same Query artifact
- no domain-authored pseudo-Query wrapper is required for any named category

Do not begin Phase 4 until this is true.

### Phase 4: Ship the ordinary invariant-registration facade

Do this only after the common canonical materialization story is real across
all named contribution categories.

Deliverables:

- add the Query-owned ordinary invariant-registration lane
- lower that lane into the exact relational registration seam already named in
  this spec:
  - `LogicRuntimeBuilder::invariant_catalog(...)`
  - `LogicRuntimeBuilder::custom_invariant(...)`
  - `CustomInvariantRegistration`
  - `InvariantCatalog`
- keep relational authority explicit and inspectable
- make the ordinary Query lane the primary DX path for Query-integrated
  runtime domains

Practical rule:

- by the end of this phase, a domain author using Query should no longer need
  to import the relational builder directly just to register ordinary runtime
  invariants

What must be true before moving on:

- Query owns the public invariant-registration facade
- relational still owns invariant authority
- the DX lane is real and compile-checked
- the old lower-level relational path remains available as the contained escape
  hatch, not the ordinary path

Do not begin Phase 5 until this is true.

### Phase 5: Finish category-complete coverage and invariant-facing integration

Do this only after the common canonical materializers are real.

Deliverables:

- finish any remaining category-specific integration needed so every category
  has a complete public lane:
  - workflow / preview
  - continuity / lineage
  - consequence / aftermath
  - explanation / inspection
- finish category-specific traceability, support, and inspection hooks that
  the generic canonical materializers were not enough to complete by
  themselves
- finish invariant/capability integration against the ordinary Query-facing
  registration and capability surfaces

Practical rule:

- by the end of this phase, every category named by `9.3.7` must feel fully
  finished to a public caller, not merely admitted into shared substrate

What must be true before moving on:

- every category has a complete public lane
- every category has the category-specific traceability/support hooks it needs
- invariant/capability registration and runtime-facing use are both complete
- no category remains in a "typed but not really finished" state

Do not begin Phase 6 until this is true.

### Phase 6: Close the DX and certification gates

This is the final phase, not parallel cleanup.

Deliverables:

- finish the common lane
- finish the checked / inspectable lane
- finish the proof lane
- finish the raw substrate lane
- add the golden transcript suite
- add hostile certification
- add compile-fail boundary coverage
- verify performance and canonicalization obligations from this spec

Practical rule:

- by the end of this phase, an engineer outside the implementation effort
  should be able to follow the intended public lane examples and hit the same
  surfaces that the spec describes

What must be true before moving on:

- all prior phases are complete
- the public API degrades one lane at a time rather than dropping from common
  lane straight to raw plumbing
- compile-fail suites prove illegal progression and category mismatch are
  structurally blocked
- hostile certification proves canonicalization, support posture honesty,
  substrate integration honesty, and no pseudo-Query bypass

Do not begin Phase 7 until this is true.

### Phase 7: Write and install the crate documentation

This is the final closeout phase.

The milestone does not close when the code and tests pass. It closes when the
new public surfaces are documented inside the crate in a way that a later
engineer can actually discover and use.

Deliverables:

- write crate-local documentation for the new `9.3.7` contribution surfaces
- add the docs under the `worth-query` crate documentation tree
- organize the docs in folders by contribution category
- write one feature doc per feature, not one giant omnibus milestone dump
- document both the common lane and the lower/proof lanes where they exist
- document the invariant-registration facade as part of the ordinary public
  Query story

Required documentation topology:

- one folder per category, at minimum:
  - admission
  - support
  - invariants
  - workflow
  - continuity
  - aftermath
  - explanation
- one document per feature within those folders

Examples of the intended shape:

- `crates/worth-query/docs/domain-capabilities/admission/...`
- `crates/worth-query/docs/domain-capabilities/support/...`
- `crates/worth-query/docs/domain-capabilities/invariants/...`

Practical rule:

- documentation must be written as feature documentation, not milestone prose
- the writing pass should explicitly use the `feature-doc-writer` skill
- the docs must explain:
  - what the feature is
  - why you use it
  - the stable entry points
  - the common path
  - the lower/proof lane where relevant
  - the main anti-patterns
  - the current limits

What must be true before the milestone can close:

- all prior phases are complete
- every public `9.3.7` feature has a crate-local feature doc
- docs are organized by category folder rather than one giant page
- each feature has its own document rather than being buried in a mega-doc
- the ordinary path and the lower/proof path are both documented where they
  exist
- the docs are good enough that a later engineer does not have to read the
  milestone spec to use the surfaces correctly

No later phase may be used as an excuse to leave an earlier phase half-done.

## Test Requirements

The named certification suite for this milestone is:

- `9.3.7. Domain Capability Contribution And Canonical Runtime Materialization Test`

That suite must prove:

- equivalent domain admission contributions materialize to the same canonical
  admission artifacts
- equivalent support / traceability contributions materialize to the same
  declaration-scoped support artifacts
- equivalent invariant / capability contributions materialize to the same
  canonical capability or invariant artifacts
- equivalent workflow contributions materialize to the same canonical workflow
  / preview artifacts
- equivalent continuity contributions materialize to the same canonical
  continuity / lineage artifacts
- equivalent aftermath contributions materialize to the same canonical
  consequence / aftermath artifacts
- equivalent explanation contributions materialize to the same canonical
  explanation / inspection artifacts
- illegal direct artifact minting fails at compile-time or typed admission
  boundaries
- category-mismatched materialization fails at compile-time rather than at
  runtime through dynamic switches
- contribution progression is backed by `worth-proof` and rejects out-of-order
  strengthening through compile-fail or typed transition denial
- new declaration-scoped descriptive surfaces use foundational artifact
  taxonomy, provenance/freshness, and profile progression instead of Query-only
  ad hoc vocabularies
- cost scales with contribution width, trace width, category width, and
  support width rather than unrelated runtime breadth

## Required Verification Output

The 9.3.7 certification bundle must emit:

- `query_digest`
- `intent_declaration_digest`
- `domain_capability_contribution_request_digest`
- `domain_capability_contribution_eligibility_digest`
- `admitted_domain_capability_contribution_digest`
- `canonical_runtime_materialization_digest`
- `admission_artifact_digest`
- `support_artifact_digest`
- `workflow_artifact_digest`
- `continuity_artifact_digest`
- `aftermath_artifact_digest`
- `explanation_artifact_digest`
- `capability_support_row_digest`
- `domain_invariant_denial_digest`
- `decision_trace_digest`
- `support_traceability_digest`
- `public_boundary_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `contribution_width`
- `trace_width`
- `category_width`
- `support_width`
- `contribution_materialization_slope_digest`
- `trace_materialization_slope_digest`
- `category_materialization_slope_digest`
- `support_materialization_slope_digest`

## Acceptance Evidence

This milestone is complete only when a hostile certification program can:

- author equivalent domain capability meaning through at least two public
  builder paths and obtain the same canonical Query runtime artifacts
- author intentionally different category posture and obtain predictably
  different canonical artifacts
- prove declaration-scoped support remains declaration-scoped
- prove domains cannot mint canonical Query runtime artifacts directly
- prove every named category closes as a real public runtime or descriptive
  family with one ordinary lane, one inspectable lane, and one proof-bearing
  lane that all hit the same canonical semantics
- prove later runtime routing and public runtime stabilization consume the
  materialized artifacts rather than forcing domains to rebuild the same
  capability layer locally

## Closeout Standard

This milestone may close only when:

- the roadmap points at this spec and the named certification suite
- the ordinary public Query facade exposes one domain capability contribution
  seam
- canonical runtime artifact constructors remain sealed
- category-mismatch materialization is structurally uncallable through sealed
  request/admitted/materializer type pairs
- declaration-scoped support/traceability artifacts are public and typed
- workflow, continuity, aftermath, and explanation categories are fully closed
  public contribution families with no category-local pseudo-Query fallback
- compile-boundary and hostile certification suites prove there is no public
  pseudo-Query bypass
- the Runtime API Public Stabilization Gate can consume this milestone as a
  closed precondition rather than documenting the gap as downstream adapter
  folklore

## Self-Check

- Does the milestone solve a real architectural problem? Yes. It closes the
  public Query gap between generic runtime ownership and serious
  domain-authored capability meaning.
- Is the adversarial constraint precise and load-bearing? Yes. It forbids
  public pseudo-Query layers, crate-private constructor reacharound, and
  stringly capability semantics.
- Does the milestone preserve crate authority boundaries? Yes. Domains own
  semantic meaning, Query owns canonical runtime artifacts.
- Could a competent engineer map this spec into honest modules and tests? Yes.
  The lifecycle, locked decisions, categories, topology, outputs, and named
  suite are explicit.
- Does this belong before the runtime API freeze? Yes. Otherwise the freeze
  would lock in a facade that still cannot honestly accept broad
  domain-authored runtime capability posture.
