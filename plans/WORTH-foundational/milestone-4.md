# Milestone 4: Boundary Artifact Taxonomy And Materialization Contracts

## Goal

Define the shared boundary artifact categories, role vocabulary, and
materialization contracts that let WORTH crates expose canonical summaries,
reports, artifacts, receipts, and planned/support surfaces without flattening
authority, derivation, or cost boundaries.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering and mechanical enforcement.
The shaping constraint is that Milestone 4 must solve the hard category and
materialization honesty problem first: rich boundary outputs must become
typed, explicit, and cost-visible before later diagnostics, provenance, and
branch/merge/commit work tries to attach to them.

### `arch_laws.md`

Protects explicit authority/derivation separation, self-describing boundary
envelopes, and phase-typed proof progression. The shaping constraint is that
boundary artifact categories must make authoritative, planned, derived,
projected, support-only, and receipt-bearing claims structurally visible
rather than relying on prose or conventions.

### `composition_laws.md`

Protects responsibility-shaped files and named semantic steps. The shaping
constraint is that category vocabulary, role legality, materialization seams,
attachment points, basis lowering, and readiness reporting must live in
separate responsibility homes rather than one broad artifact/materialization
dump.

### `domain_structure_laws.md`

Protects structure as responsibility topology rather than convenience filing.
The shaping constraint is that artifact categories, materialization contracts,
role/authority law, and planned/same-family extension surfaces must be
independently locatable and testable.

### `perf_laws.md`

Protects cost-honest boundaries, explicit policy resolution before execution,
and path separation between authority and descriptive richness. The shaping
constraint is that materialization must be an explicit boundary with visible
cost and explicit profile-driven elision, not a cheap-looking accessor over
authoritative truth.

### `worth_foundational_vision.md`

Protects the thesis that `worth-foundational` owns shared boundary language
for reports, summaries, artifacts, receipts, planned-work descriptions, and
same-family descriptive surfaces while preserving crate-local runtime
representation. The shaping constraint is that Milestone 4 must standardize
boundary meaning without forcing one internal artifact store or one generic
executor.

### `worth_foundational_roadmap.md`

Protects the sequencing rule that artifact taxonomy follows canonicalization
and profiles, but precedes branch/merge/commit vocabulary, diagnostics,
provenance, and migration closure. The shaping constraint is that Milestone 4
must give later milestones a typed boundary language to attach to without
stealing their ontology.

### `test-requirements.md`

Protects standalone proof before adopting-crate migration. The shaping
constraint is that category separation, authority-versus-derived legality,
explicit materialization seams, reduced-richness preservation, and
planned-versus-receipt distinctions must all be certified through hostile
local doubles, compile-fail boundaries, blind-consumer tests, bundle-legality
proof, structured decision-row explanation proof, and a production-test
readiness artifact.

### `milestone-2-closeout.md`

Protects canonical basis as semantic authority and digest values as derived
compression. The shaping constraint is that boundary artifact categories and
materialized outputs must participate in canonical basis law rather than
inventing local digest folklore or transport-shaped identity.

### `milestone-3.md`

Protects profile meaning, target-aware attachment, materialization planning,
and proof-bearing readiness law. The shaping constraint is that Milestone 4
must consume those profile surfaces as already-closed dependencies rather than
reopening profile identity, elision semantics, or proof-lane choices.

### `milestone-3-closeout.md`

Protects the fact that profile families, composition, progression, canonical
identity, optional descriptive surface planning, certification posture, and
readiness evidence are already implemented and locally certified. The shaping
constraint is that Milestone 4 must treat profile-governed elision and
certification posture as real dependency boundaries for boundary artifacts,
not as TODOs to reinterpret later.

## Why This Milestone Exists

Milestones 1 through 3 established:

- aspect-native truth vocabulary
- canonical basis and digest-honest comparison law
- profile meaning, target-aware attachment, and explicit descriptive elision

They did not yet answer the next boundary question:

- when a crate materializes a boundary-facing output, what category is it
- whether it is authoritative, derived, projected, support-only, planned, or
  receipt-bearing
- what the materialization seam is
- what profile-governed descriptive surfaces were attached or elided
- where later diagnostics, provenance, receipts, performance accounting, or
  same-family resolution evidence are allowed to hang

If WORTH keeps solving those questions with local envelope names, free-form
role strings, or crate-private materialization conventions, then the newly
closed profile and canonicalization law will still terminate in category
folklore at the first real boundary crossing.

Milestone 4 exists to create one shared answer for boundary artifact meaning:

- typed category vocabulary for `Summary`, `Report`, `Artifact`, and `Receipt`
- typed role vocabulary for authoritative, derived, projected, support-only,
  planned, and receipt-bearing boundary claims
- explicit materialization seams, sources, and costs
- typed attachment points for profile decisions, canonical basis, diagnostics,
  provenance, and later milestone surfaces
- explicit room for plan-shaped and same-family descriptive outputs without
  pretending those are execution engines or branch/merge/commit authority
  transitions
- proof-bearing readiness that tells later milestones exactly what artifact
  categories and materialization claims they may rely on

## Adversarial Constraint

Several WORTH crates with different internal state layouts, support surfaces,
planned-work shapes, and authority boundaries must be able to materialize
semantically equivalent boundary outputs into the same shared category and role
language, with the same canonical basis participation and the same explicit
profile-driven elision/cost story, while preserving authoritative truth,
without hiding broad materialization behind cheap APIs, and without allowing
descriptive or planned outputs to impersonate authoritative or receipt-bearing
surfaces.

This milestone fails if:

- `Summary`, `Report`, `Artifact`, and `Receipt` are just local labels over
  one generic envelope
- authoritative, derived, projected, support-only, planned, and receipt-bearing
  boundary claims can be swapped by convention instead of typed law
- expensive materialization is exposed as a cheap-looking getter or view
- profile-driven descriptive elision changes authoritative payload truth
- branch, merge, or commit authority-transition evidence is smuggled through a
  generic artifact category before Milestone 5 exists
- planned-work outputs are confused with execution receipts
- same-family descriptive outputs silently become authority-shaped or runtime
  execution-shaped
- canonical artifact meaning depends on producer-private field layout or local
  envelope naming
- `worth-foundational` starts standardizing one internal artifact store or one
  generic executor instead of boundary meaning

## WORTH-Proof Dependency Boundary

Milestone 4 uses `worth-proof` for proof-bearing authority and readiness
surfaces, but not for plain artifact-category vocabulary.

`worth-proof` is mandatory for:

- boundary artifacts that claim stronger authority or receipt-bearing posture
  when that claim must be more than a plain enum value
- materialized outputs whose current-basis or boundary-readmission status must
  remain proof-bearing rather than ambient
- production-test readiness artifacts for Milestone 4
- authority-gated admission of boundary outputs that claim to be
  authoritative-current rather than descriptive-only, support-only, or planned

`worth-proof` is forbidden for:

- plain artifact-category vocabulary
- plain role/source/seam vocabulary
- plain materialization cost and attachment-point vocabulary
- plain planned-work and same-family descriptive nouns
- replacing branch/merge/commit authority-transition law reserved for
  Milestone 5

The operating rule is:

`worth-foundational` defines what category and role a boundary artifact means.
`worth-proof` proves when a boundary artifact is allowed to carry stronger
authority or readiness claims.

## Practical Type Targets

The implementation may choose better names, but these responsibilities must
exist concretely somewhere:

```rust
pub enum FoundationalBoundaryArtifactCategory {
    Summary,
    Report,
    Artifact,
    Receipt,
}

pub enum FoundationalBoundaryArtifactRole {
    AuthoritativeCurrent,
    DerivedProjection,
    SupportOnly,
    PlannedWork,
    ReceiptEvidence,
}

pub enum FoundationalBoundaryMaterializationSource {
    NativeAuthority,
    CompatibilityLowered,
    DerivedSupport,
}

pub enum FoundationalBoundaryMaterializationSeam {
    BoundaryExchange,
    SupportMaterialization,
    PersistenceExport,
}

pub enum FoundationalBoundaryDeliveryClass {
    MustBeHot,
    CanDefer,
    ReconstructableFromRetainedBasis,
}

pub enum FoundationalBoundaryAvailability {
    Present,
    Deferred,
    Reconstructable,
    Unavailable,
}

pub enum FoundationalBoundaryDecisionSubject {
    CategoryRoleAdmission,
    DeliveryAvailabilityResolution,
    AttachmentInclusion,
    AttachmentElision,
    BundleMembership,
}

pub enum FoundationalBoundaryDecisionCause {
    RequestedAsAdmitted,
    NarrowedByAuthority,
    ElidedByProfile,
    DeniedByBudget,
    UnavailableByRetention,
    ReconstructableFromRetainedBasis,
    DeniedByMilestoneBoundary,
}

pub enum FoundationalBoundaryAttachmentPoint {
    ProfileMeaning,
    ProfileDecisions,
    CanonicalBasis,
    DiagnosticsAttachment,
    ProvenanceAttachment,
    PerformanceAccounting,
    SameFamilyResolutionAttachment,
}

pub struct FoundationalBoundaryMaterializationInput<T> { /* private fields */ }

pub struct FoundationalBoundaryMaterializationPlan { /* private fields */ }

pub struct FoundationalBoundaryMaterializationCost { /* private fields */ }

pub struct FoundationalBoundaryMaterializationDecisionRow { /* private fields */ }

pub struct FoundationalBoundarySurfaceDisposition { /* private fields */ }

pub struct FoundationalBoundaryMaterializationBundle<Primary> { /* private fields */ }

pub trait FoundationalBoundaryCategoryMarker: sealed::Sealed {}
pub struct SummaryCategory;
pub struct ReportCategory;
pub struct ArtifactCategory;
pub struct ReceiptCategory;

pub trait FoundationalBoundaryRoleMarker: sealed::Sealed {}
pub struct AuthoritativeCurrentRole;
pub struct DerivedProjectionRole;
pub struct SupportOnlyRole;
pub struct PlannedWorkRole;
pub struct ReceiptEvidenceRole;

pub struct FoundationalMaterializedBoundaryArtifact<Category, Role, T> {
    /* private fields */
}

pub struct FoundationalBoundarySummarySurface { /* private fields */ }

pub struct FoundationalBoundaryReportSurface<Row> { /* private fields */ }

pub struct FoundationalBoundaryReceiptSurface { /* private fields */ }

pub struct FoundationalAuthoritativeBoundaryArtifact<Category, T> {
    /* private fields */
}

pub struct FoundationalBoundaryAuthorityAdmission { /* private fields */ }

pub enum FoundationalBoundaryMaterializationDenial {
    IllegalCategoryRoleCombination,
    IllegalProfileGovernedAttachment,
    IllegalAuthorityClaim,
    IllegalReceiptClaim,
    Milestone5AuthorityTransitionRequired,
}

pub struct FoundationalPlannedWorkBoundaryArtifact<T> { /* private fields */ }

pub struct FoundationalSameFamilyBoundaryArtifact<T> { /* private fields */ }

pub struct FoundationalSameFamilyBoundaryIdentity { /* private fields */ }
```

These sketches imply concrete constraints:

- category meaning and role meaning remain distinct typed vocabularies
- category legality and role legality are explicit before materialization
  constructors exist
- materialization inputs, plans, and outputs are distinct surfaces rather than
  one catch-all builder object
- category and role should be visible in the materialized type wherever the
  type system can enforce the distinction, not only through runtime enums
- `Summary`, `Report`, `Artifact`, and `Receipt` should not be marker names
  over the same body shape; each category should carry category-appropriate
  construction obligations
- `Summary` should bias toward bounded overview-shaped output, `Report` toward
  explanatory/evidence-bearing rows or sections, `Artifact` toward structured
  materialized payload, and `Receipt` toward completed-boundary attestation
- delivery class and availability remain explicit boundary meaning rather than
  implicit consequences of category or role
- illegal delivery/availability combinations should be mechanically forbidden
  wherever the type system knows enough to reject them
- materialization reports should preserve row-level decision evidence rather
  than collapsing all lowering/admission/exclusion logic into one digest
- decision rows should record subject, cause, and affected boundary seam rather
  than collapsing into free-form explanation text
- one boundary operation may emit a coordinated bundle of summary/report/
  artifact/receipt surfaces, but that bundle must remain typed and category-
  honest rather than becoming a bag-of-everything result
- bundle membership should be explicit and legality-checked so operations do
  not silently attach duplicate or incoherent surfaces
- stronger authority-bearing boundary outputs reuse `worth-proof` rather than
  inventing local pseudo-proof wrappers
- plan-shaped and same-family descriptive outputs must have explicit room
  without collapsing into authority-bearing or receipt-bearing categories
- same-family descriptive outputs should preserve family-scoped identity so
  later milestones can attach lifecycle or resolution evidence without
  reinferring family membership from local names
- branch/merge/commit authority transitions remain fail-closed here and become
  real ontology only in Milestone 5

## Naive Traps To Avoid

- Do not model every boundary output as one generic envelope plus a category
  field. The milestone wants category and role law, not one bag with runtime
  self-restraint.
- Do not implement categories as empty markers over one shared unconstrained
  payload wrapper. That recreates the same envelope problem one layer down.
- Do not let `Receipt` mean both "planned to happen" and "actually happened."
  Planned-work and receipt-bearing outputs must remain structurally distinct.
- Do not let support-only or descriptive-only outputs claim
  `AuthoritativeCurrent` by constructor convenience.
- Do not let delivery class or availability become ambient knowledge inferred
  from category names or local docs. If a surface is hot, deferred,
  reconstructable, or unavailable, that must be visible in the boundary law.
- Do not treat profile elision as a local formatting concern. It must remain a
  visible materialization seam whose decisions and cost can be inspected.
- Do not let canonical basis participation read producer-private layout or
  envelope ordering as semantic meaning.
- Do not sneak branch, merge, or commit authority-transition evidence into
  Milestone 4 by calling it a generic artifact or receipt. That ontology is
  reserved for Milestone 5.
- Do not expose cheap-looking accessors that trigger broad materialization,
  attachment collection, or profile-driven support enrichment behind the
  scenes.
- Do not erase the boundary seam by giving materialized boundary artifacts
  `Deref`, ambient `AsRef`, or trivial payload conversions that make them act
  like raw truth containers.
- Do not force every real operation to choose one output surface when it
  honestly produces a coordinated summary/report/artifact/receipt set. Give
  bundles typed room instead of encouraging local result bags.
- Do not let bundle construction accept arbitrary lists or maps of surfaces.
  If the milestone can know membership legality statically, it should.

## Phases

These phases are implementation order, not topic buckets.

An engineer should be able to start at Phase 1 and move downward without
deciding their own sequencing model. A later phase may consume and extend the
artifacts of an earlier phase, but it may not reopen them casually. If a phase
is not complete enough to support the next phase's code and tests, the
milestone is not ready to advance.

The practical rule is:

- finish the nouns of the current phase
- expose the minimum honest API for those nouns
- prove the boundaries of that phase locally
- only then start the next dependency layer

Phase progression gates:

| Phase | Gate before next phase |
| --- | --- |
| Phase 1 | Category vocabulary and category-local non-substitution law exist before role or materialization APIs exist. |
| Phase 2 | Role/authority law and category-role legality exist before boundary materialization constructors exist. |
| Phase 3 | Materialization seam, source, cost, and attachment-point law exist before canonical basis participation is added. |
| Phase 4 | Canonical basis participation and blind-consumer artifact interpretation exist before any stronger basis-ready boundary lane is standardized. |
| Phase 4.5 | Proof-bearing basis-ready and current-basis boundary surfaces exist, and explicitly reuse Milestone 2 canonicalization rather than rebuilding it, before planned-work and same-family extension surfaces close. |
| Phase 5 | Planned-work and same-family descriptive extension law, plus Milestone 5 fail-closed denials, exist before readiness closes. |
| Phase 6 | Production-test readiness evidence exists before Milestone 5 consumes boundary artifact categories. |

### Phase 1: Define Boundary Artifact Categories

Purpose:

Freeze the shared category nouns before any role, materialization, or
attachment policy exists.

Engineer order:

Define `Summary`, `Report`, `Artifact`, and `Receipt` first and make their
meanings mutually non-substitutable before writing materialization APIs. This
phase is complete only when an engineer can point at each category and explain
what it is for, what it is not for, and why a neighboring category cannot
stand in for it.

Practical implementation order:

1. Create the category home and define only the four category nouns.
2. Give each category its own category-local construction surface or marker
   contract.
3. Add the first deny paths for incoherent category claims.
4. Export only those category nouns through the facade.
5. Write blind-consumer and compile-fail tests before moving on.

What should exist at the end of the phase:

- a caller can name `Summary`, `Report`, `Artifact`, or `Receipt` without
  seeing role, materialization, or authority vocabulary yet
- category-specific surfaces cannot be substituted by one generic payload
  wrapper
- the facade exposes category meaning, not a partially implemented
  artifact-building stack

Must ship:

- typed boundary artifact categories for `Summary`, `Report`, `Artifact`, and
  `Receipt`
- category-specific construction law so summary/report/artifact/receipt shapes
  cannot collapse into one generic payload wrapper
- typed denial vocabulary or equivalent legality law for incoherent category
  claims
- facade exports for milestone-owned boundary category nouns only

Must preserve:

- category meaning is typed, not stringly
- `Summary`, `Report`, `Artifact`, and `Receipt` remain distinct semantic
  categories
- category vocabulary standardizes boundary meaning, not one envelope store

Acceptance evidence:

- runtime tests proving adjacent categories are not substitutable
- compile-fail tests proving plain labels or wrong category markers cannot
  satisfy category-typed APIs
- compile-fail or privacy tests proving category-specific surfaces cannot be
  substituted by one generic payload wrapper
- blind-consumer tests proving category meaning is interpretable without
  producer-private context

### Phase 2: Define Role And Authority Law

Purpose:

Define what kind of truth-claim a boundary artifact is making before the crate
permits materialization into that claim.

Engineer order:

After categories are closed, define role vocabulary and category-role legality.
Only after that should any constructor be allowed to produce a materialized
boundary output. This phase is complete only when authoritative, derived,
support-only, planned, and receipt-bearing claims are mechanically distinct
and illegal combinations fail closed.

Practical implementation order:

1. Create the role/authority home and define role nouns independently from
   materialization.
2. Encode category-role legality while the surface area is still small.
3. Introduce the first sealed authority admission lane for
   authoritative-current claims.
4. Add typed denials for illegal authority and receipt claims.
5. Expose only the role and legality APIs needed for the next phase.
6. Lock the boundaries with compile-fail and authority-hostility tests.

What should exist at the end of the phase:

- an engineer can express what claim a boundary surface is making before they
  can materialize one
- the codebase already refuses support-only-as-authority and
  planned-work-as-receipt mistakes
- authoritative-current is visibly stronger than category naming alone

Must ship:

- typed role vocabulary for authoritative-current, derived/projected,
  support-only, planned-work, and receipt-bearing claims
- explicit category-role legality rules
- proof-bearing authority admission or equivalent sealed authority-gated
  surface for artifacts that claim authoritative-current posture
- typed denials for illegal authority and receipt claims

Must preserve:

- descriptive and support-only outputs cannot impersonate authoritative-current
- planned-work outputs cannot impersonate receipt-bearing outputs
- authority claims remain stronger than category labels alone

Acceptance evidence:

- hostile tests proving support-only and planned-work outputs cannot satisfy
  authority-requiring APIs
- compile-fail tests proving statically known illegal category-role
  substitutions fail closed
- proof-bearing tests proving stronger authority claims require explicit
  admission rather than caller-minted labels

### Phase 3: Define Materialization Contracts

Purpose:

Make boundary materialization an explicit, cost-honest boundary with typed
sources, seams, plans, and attachment points.

Engineer order:

Define materialization source and seam vocabulary first, then define typed
input/plan/output contracts, and only then expose materialization entrypoints.
The phase is not done when a builder exists; it is done when an engineer can
say where the materialization happened, what it cost, and which attachment
points were present or intentionally absent.

Practical implementation order:

1. Define source, seam, delivery-class, and availability nouns before any
  public materialization entrypoint exists.
2. Define the input type that gathers the already-admitted category, role, and
   source information.
3. Define the plan type that exposes seam, cost, availability, attachments,
   and decision rows before execution.
4. Define the output surfaces and only then add `.materialize()`.
5. Add the coordinated bundle lane after single-surface materialization is
   already honest.
6. Lock delivery/availability legality, bundle legality, and no-cheap-
   conversion boundaries with tests before moving forward.

What should exist at the end of the phase:

- the common DX can say `plan()` and `materialize()` honestly
- heavy work is visibly heavy at the call site
- delivery, availability, and decision-row explanation are inspectable before
  canonical basis work begins

Must ship:

- typed materialization source vocabulary
- typed materialization seam vocabulary
- typed delivery-class and availability vocabulary
- explicit legality law for delivery/availability combinations
- typed materialization input, plan, denial, and cost surfaces
- typed materialization decision-row or equivalent provenance-row surface
- typed coordinated bundle surface for operations that emit multiple category-
  distinct boundary outputs together
- typed attachment-point vocabulary for profile meaning, profile decisions,
  canonical basis, diagnostics, provenance, performance accounting, and
  same-family extensions
- explicit materialization APIs that expose cost and seam at the boundary

Must preserve:

- materialization is visible in API shape
- expensive materialization does not masquerade as a cheap getter
- delivery and availability posture remain inspectable without producer-private
  conventions
- decision rows remain structured enough for later support and certification
  surfaces to explain what happened without parsing prose
- coordinated bundles preserve category honesty instead of flattening several
  outputs into one generic result object
- profile-driven descriptive elision remains a visible seam consumer rather
  than hidden leaf-call-site behavior

Acceptance evidence:

- materialization honesty tests proving seam and cost remain inspectable
- delivery/availability tests proving hot, deferred, reconstructable, and
  unavailable surfaces stay mechanically distinct
- compile-fail or typed-boundary tests proving statically known illegal
  delivery/availability combinations fail closed
- provenance-row tests proving materialization decisions remain explainable in
  row form and not only by digest comparison
- bundle-legality tests proving duplicate or incoherent category members are
  rejected rather than silently accepted
- API-lane tests proving common-path materialization, advanced plan
  inspection, and authority-shaped admission remain distinct and cannot bypass
  one another
- hostile tests proving reduced-richness profiles remove only optional
  descriptive attachments and not authoritative payload truth
- hostile tests proving materialized boundary artifacts do not expose cheap
  payload-style conversions that bypass the seam
- blind-consumer tests proving attachment points are interpretable without
  producer-private layout knowledge

### Phase 4: Define Canonical Basis Participation

Purpose:

Make boundary artifact meaning basis-honest before planned-work and
same-family descriptive room is closed.

Engineer order:

Once category, role, and materialization contracts are closed, define canonical
basis participation for materialized boundary outputs and add blind-consumer
interpretation tests. Do not add planned-work or same-family extension
surfaces until the artifact categories already lower through canonical basis
law. Do not standardize a stronger basis-ready wrapper in this phase yet;
first close the semantic lowering itself, then add the proof-bearing lane in
Phase 4.5.

Practical implementation order:

1. Read and reuse the existing Milestone 2 canonicalization surfaces before
   creating any boundary-artifact basis helper, because canonical basis
   preparation, readiness, and comparison already exist.
2. Decide exactly which category/role/seam/availability/attachment facts enter
   canonical basis and in what order.
3. Lower each existing materialized surface through that basis path.
4. Prove parity across independent producers and hostile field-order cases.
5. Freeze the basis contract before adding any new descriptive extension room.

What should exist at the end of the phase:

- existing boundary surfaces have stable basis meaning
- materialization cost and other non-semantic counters are already kept out of
  identity
- later phases can attach new descriptive room without re-litigating basis law

Must ship:

- canonical basis participation for materialized boundary artifact categories,
  roles, seams, sources, and attachment-point evidence
- deterministic ordering for basis participation that does not depend on local
  envelope field order
- explicit distinction between semantic basis evidence and materialization cost
  accounting

Must preserve:

- canonical artifact meaning is basis-derived, not layout-derived
- cost counters do not silently become semantic identity
- attachment presence and category meaning remain visible to blind consumers

Acceptance evidence:

- parity tests proving semantically identical boundary outputs from independent
  producers derive identical basis evidence
- hostile tests proving local field/envelope order does not change basis
  meaning
- compile-fail or privacy tests proving raw layout fields cannot satisfy
  basis-ready APIs

### Phase 4.5: Define Proof-Bearing Basis-Ready Boundary Surfaces

Purpose:

Add the stronger proof-bearing boundary lane only after semantic canonical
basis participation is already closed, so Milestone 4 can expose basis-ready
or current-basis boundary artifacts without rebuilding Milestone 2
canonicalization or inventing local pseudo-readiness wrappers.

Engineer order:

Do this immediately after Phase 4 and before any planned-work or same-family
extension room. This phase exists to keep the implementation linear and honest:
first semantic basis lowering, then stronger basis-ready/current-basis
boundary claims, then later descriptive extension work. This phase is complete
only when an engineer can point at the stronger boundary-ready lane and show
that it is reusing Milestone 2 canonicalization and `worth-proof` rather than
shadowing them.

This phase does not reopen Phase 2 as an unfinished milestone step. Phase 2
stays checked off as the first authority-admission lane. Phase 4.5 is a new
later corrective item in the sequence that is allowed to refactor or tighten
the already-shipped Phase 2 authority path where needed so it composes
honestly with Milestone 2 canonicalization and the stronger basis-ready lane.

Practical implementation order:

1. Start from the finished Phase 4 basis entries and identify which existing
   Milestone 2 canonicalization readiness and comparison surfaces boundary
   artifacts must reuse.
2. Define the boundary-artifact stronger lane only as a thin, responsibility-
   honest adaptation over those existing Milestone 2 canonicalization
   surfaces.
3. Make current-basis, basis-ready, or boundary-readmitted claims explicitly
   proof-bearing through `worth-proof` and the existing canonicalization
   readiness/authority lane rather than local booleans or private markers.
4. Refactor the already-completed Phase 2 authority-admission implementation
   only where that is required to make the stronger lane compose honestly;
   treat that as corrective alignment work discovered later, not as new Phase
   2 scope.
5. Keep materialization cost, decision rows, and bundle accounting out of the
   stronger basis-ready claim itself unless Milestone 2 basis law already
   treats them as semantic evidence.
6. Prove that callers cannot satisfy the stronger basis-ready APIs with raw
   materialized artifacts or raw basis sequences.

What should exist at the end of the phase:

- there is an explicit proof-bearing lane for basis-ready or current-basis
  boundary artifacts
- that lane is visibly downstream of Phase 4 basis participation
- that lane reuses Milestone 2 canonicalization and `worth-proof` rather than
  rebuilding either one locally

Must ship:

- explicit proof-bearing basis-ready and/or current-basis boundary artifact
  surfaces, if Milestone 4 exposes stronger basis claims at all
- explicit reuse of Milestone 2 canonicalization readiness, comparison, or
  basis-preparation surfaces rather than boundary-artifact-local substitutes
- typed denials or fail-closed boundaries for callers attempting to bypass the
  stronger basis-ready lane with raw materialized outputs

Must preserve:

- Milestone 2 remains the owner of canonicalization readiness and basis law
- `worth-proof` remains the owner of stronger proof-bearing progression claims
- boundary artifacts do not introduce a second basis-readiness dialect
- semantic basis evidence stays distinct from materialization cost accounting

Acceptance evidence:

- hostile tests proving raw materialized boundary artifacts cannot satisfy the
  stronger basis-ready or current-basis APIs
- proof-lane tests showing the boundary-artifact stronger lane reuses the real
  Milestone 2 canonicalization readiness/basis artifacts
- compile-fail or privacy tests proving callers cannot manufacture local
  pseudo-basis-ready wrappers
- readmission or current-basis tests, if exposed, proving stronger basis
  claims remain proof-bearing rather than ambient

### Phase 5: Define Planned And Same-Family Descriptive Extension Law

Purpose:

Reserve explicit boundary room for planned-work and same-family descriptive
surfaces without accidentally implementing Milestone 5 early.

Engineer order:

After canonical basis participation and the stronger Phase 4.5 basis-ready
lane are closed, define the descriptive room for planned-work and same-family
outputs, plus fail-closed denials for branch, merge, and commit authority
transitions. This phase is complete only when later milestones have a typed
place to attach without Milestone 4 pretending it already owns their
ontology.

Practical implementation order:

1. Create the planned-work and same-family homes after the core category stack
   is already basis-honest.
2. Define the descriptive-only shapes and same-family identity lane.
3. Encode where those surfaces are legal and where they are forbidden,
   especially in bundles and authority-only contexts.
4. Add explicit fail-closed denials for branch/merge/commit authority claims.
5. Prove that these surfaces give later milestones room to attach without
   smuggling Milestone 5 semantics in early.

What should exist at the end of the phase:

- there is a typed home for planned and same-family descriptive outputs
- those outputs are obviously not receipts or authority transitions
- Milestone 5 still owns real branch/merge/commit ontology

Must ship:

- explicit planned-work boundary artifact vocabulary
- explicit same-family descriptive boundary artifact vocabulary
- explicit same-family identity surface for family-scoped descriptive outputs
- typed fail-closed denials for branch/merge/commit authority-transition
  claims that belong to Milestone 5
- attachment-point legality rules for planned-work and same-family descriptive
  outputs

Must preserve:

- planned-work outputs remain descriptive, not authority-bearing
- same-family descriptive outputs remain descriptive, not execution engines
- same-family descriptive outputs remain family-identifiable without local
  naming folklore
- planned-work and same-family surfaces remain illegal bundle members wherever
  a receipt-bearing or authoritative-only bundle is expected
- Milestone 5 authority-transition vocabulary remains out of scope here

Acceptance evidence:

- hostile tests proving planned-work outputs cannot satisfy receipt or
  authoritative APIs
- same-family identity tests proving semantically identical family outputs from
  independent producers preserve the same family-scoped identity
- compile-fail or typed-boundary tests proving planned-work and same-family
  surfaces cannot enter authority-only or receipt-only bundle lanes
- hostile tests proving same-family descriptive outputs cannot satisfy branch,
  merge, or commit authority-transition APIs
- compile-fail or typed-boundary tests proving Milestone 5 claims remain
  fail-closed in Milestone 4

### Phase 6: Certify Production-Test Readiness

Purpose:

Close Milestone 4 with a proof-bearing readiness artifact that later
branch/merge/commit work can depend on.

Engineer order:

Do not treat this as doc cleanup. This phase proves which category, role,
materialization, and basis boundaries actually survived implementation. If a
surface mattered enough to appear in earlier phases but is absent from the
readiness artifact, then the milestone is not actually closed.

Practical implementation order:

1. Inventory the exact surfaces that actually shipped from Phases 1 through
   5, including the stronger Phase 4.5 basis-ready lane where present.
2. Map each certified surface to its runtime tests, compile-fail boundaries,
   and blind-consumer evidence.
3. Record runtime assumptions, non-assumptions, and residual debt while the
   implementation details are still concrete.
4. Refuse closure for any phase surface that cannot be tied to real evidence.
5. Freeze the readiness artifact as the only thing later milestones may assume.

What should exist at the end of the phase:

- a later engineer can tell exactly what Milestone 5 may rely on
- there is no ambiguity about which Phase 3 or Phase 5 surfaces are real and
  which were merely discussed
- the milestone closes as a proved boundary, not as "the code looks about
  right"

Must ship:

- a Milestone 4 production-test readiness artifact or report
- certified-surface inventory for categories, role/authority law,
  materialization contracts, canonical basis participation, and planned/same-family
  extension law
- hostile-pressure inventory for category adjacency, authority/derivation
  separation, materialization honesty, and Milestone 5 fail-closed law
- compile-fail boundary inventory
- runtime assumptions, non-assumptions, and residual debt

Must preserve:

- adopting crates and Milestone 5 may assume only what the readiness artifact
  names
- local doubles remain semantic fixtures, not generic executors or artifact
  stores
- later milestones still own branch/merge/commit ontology, diagnostics,
  provenance, and receipt semantics beyond category law

Acceptance evidence:

- readiness tests proving every certified surface has hostile evidence,
  compile-fail coverage, and blind-consumer interpretation where required
- readiness tests proving every cheap-looking authoring lane has a
  corresponding stronger plan/proof lane and named misuse-pressure coverage
- exact inventory tests for runtime non-assumptions and residual debt
- topology review proving boundary artifact tests live in responsibility-owned
  homes

## Must Ship

- typed boundary artifact categories for `Summary`, `Report`, `Artifact`, and
  `Receipt`
- typed role vocabulary for authoritative-current, derived/projected,
  support-only, planned-work, and receipt-bearing outputs
- explicit category-role legality law
- proof-bearing authority admission for stronger authoritative-current claims
- typed materialization source, seam, plan, denial, and cost surfaces
- typed delivery-class and availability surfaces
- explicit legality law for delivery/availability combinations
- typed materialization decision-row evidence surface
- typed coordinated bundle surface for multi-output materialization
- typed attachment-point vocabulary for profile meaning, profile decisions,
  canonical basis, diagnostics, provenance, performance accounting, and
  same-family resolution attachments
- explicit materialization APIs whose boundary shape exposes seam and cost
- canonical basis participation for materialized boundary outputs
- explicit proof-bearing basis-ready and/or current-basis boundary lane that
  reuses Milestone 2 canonicalization, where stronger basis claims are
  exposed
- explicit planned-work boundary artifact vocabulary
- explicit same-family descriptive boundary artifact vocabulary
- explicit same-family identity surface
- fail-closed Milestone 5 authority-transition denials
- production-test readiness artifact for Milestone 4

## Must Preserve

- category meaning remains shared boundary vocabulary, not one generic
  envelope implementation
- authoritative, derived, projected, support-only, planned, and receipt-bearing
  claims remain structurally distinct
- materialization remains an explicit cost-honest boundary
- delivery/availability meaning remains explicit rather than ambient
- profile-driven descriptive elision removes only optional descriptive
  attachments and does not change authoritative payload truth
- canonical artifact meaning depends on canonical basis participation, not
  producer-private field order or local envelope naming
- materialized boundary artifacts do not silently collapse into raw payload
  views through convenience conversions
- planned-work and same-family descriptive outputs remain descriptive and do
  not become hidden execution or authority-transition engines
- branch/merge/commit authority-transition vocabulary remains owned by
  Milestone 5
- later milestones retain ownership of diagnostics ontology, provenance
  ontology, receipt semantics beyond category law, and migration closure

## Acceptance Evidence

- category-adjacency hostility tests
- category-role legality and authority-admission tests
- compile-fail tests for raw labels, wrong category-role substitutions,
  support-only/planned outputs satisfying authority APIs, and Milestone 5
  authority-transition claims entering Milestone 4 surfaces
- compile-fail tests for category-wrapper collapse, illegal delivery/
  availability combinations, and illegal bundle membership
- materialization honesty tests proving seam and cost remain visible
- delivery/availability distinction tests
- materialization decision-row/provenance-row explanation tests
- API-lane separation tests for common-path, plan-path, and authority-path
  usage
- hostile reduced-richness tests proving optional descriptive attachments can be
  elided without changing authoritative payload truth
- hostile tests proving multi-output bundle surfaces preserve category honesty
  instead of collapsing to one generic envelope
- blind-consumer tests proving category, role, seam, and attachment-point
  meaning are interpretable without producer-private state
- canonical basis parity tests across independent boundary producers
- stronger basis-ready proof-lane tests proving Milestone 2 canonicalization
  and `worth-proof` are reused rather than rebuilt locally
- readiness artifact tests covering certified surfaces, hostile pressures,
  compile-fail boundaries, assumptions, non-assumptions, and debt

## Architectural Notes

The implementation should preserve distinct boundary-artifact responsibility
homes. A likely shape is:

```text
crates/worth-foundational/src/
  boundary_artifacts/
    categories/
    roles/
    materialization/
    basis/
    readiness/
```

Public exports should remain facade-controlled. The root may exist, but it must
not become an unnamed bucket where category nouns, role legality,
materialization plans, basis lowering, and readiness reporting collapse into
one file.

The likely structural split is:

- `categories/` or equivalent owns `Summary` / `Report` / `Artifact` /
  `Receipt` vocabulary
- `roles/` or equivalent owns authoritative-versus-derived legality
- `materialization/` or equivalent owns source/seam/cost/input/output law
- `materialization/` or an adjacent home should also own delivery/
  availability law, decision rows, and coordinated bundle emission surfaces
- `basis/` or equivalent owns canonical basis lowering for materialized
  boundary outputs
- `planned/` and `same_family/` or equivalent own descriptive extension room
  if those surfaces grow large enough to earn dedicated homes
- `readiness/` or equivalent owns the milestone closeout artifact

Materialized outputs should also preserve the split:

- authoritative category/role claims are stronger than descriptive category
  claims and should use proof-bearing admission where the type system can
  enforce it
- profile decisions attach to materialized outputs as derivative boundary
  evidence, not as authority
- multi-output operations may expose bundles, but each member surface should
  still preserve its own category, role, delivery, and availability meaning
- diagnostics/provenance/performance attachments are named boundary hooks here,
  while their deeper ontologies remain later milestone work

## Desired DX End State

Milestone 4 should not finish as "enums plus wrappers." It should finish as a
layered authoring surface where the common call site reads like intent, the
next lower layer exposes planning and cost, and the strongest lane makes
authority/proof requirements explicit.

The finished developer experience should follow these rules:

- common path: ask for the semantic boundary surface directly
- advanced path: lower into an inspectable materialization plan before
  materialization
- expensive path: make materialization, reconstruction, and bundle emission
  look expensive
- strong-claim path: authoritative or receipt-bearing claims must visibly
  require stronger admission/proof than descriptive surfaces
- explanation path: decision rows and availability surfaces must explain why a
  boundary output looks the way it does without producer-private context

The intended common path should look like:

```rust
let summary = boundary_artifacts::summary()
    .from_authoritative(&snapshot)
    .for_exchange()
    .under_profile(profile)
    .materialize()?;
```

```rust
let report = boundary_artifacts::report()
    .from_support(&inspection)
    .for_support()
    .under_profile(profile)
    .materialize()?;
```

The intended advanced path should expose planning and operational footprint
before materialization:

```rust
let plan = boundary_artifacts::artifact()
    .from_authoritative(&publication_bundle)
    .for_persistence_export()
    .under_profile(profile)
    .plan()?;

plan.category();
plan.role();
plan.delivery_class();
plan.availability();
plan.cost();
plan.attachments();
plan.decision_rows();
plan.explain();

let artifact = plan.materialize()?;
```

The intended multi-output path should use a typed bundle rather than a local
result bag:

```rust
let emitted = boundary_artifacts::bundle()
    .primary_artifact()
    .with_summary()
    .with_receipt()
    .from_authoritative(&route_result)
    .for_exchange()
    .under_profile(profile)
    .materialize()?;

let artifact = emitted.primary();
let summary = emitted.summary();
let receipt = emitted.receipt();
```

The intended authority-shaped path should look visibly stronger than ordinary
descriptive materialization:

```rust
let admitted = boundary_artifacts::authoritative_artifact()
    .from_current_basis(&commit_result, authority_witness)
    .for_exchange()
    .under_profile(profile)
    .admit()?;

let artifact = admitted.materialize()?;
```

The intended report path should feel explanatory by default rather than "same
payload, different enum":

```rust
let report = boundary_artifacts::report()
    .from_support(&support_matrix)
    .for_support()
    .under_profile(profile)
    .materialize()?;

for row in report.rows() {
    row.subject();
    row.posture();
    row.cause();
}
```

The intended availability/elision path should keep optional surfaces honest:

```rust
let surface = artifact.diagnostics_surface();

match surface.availability() {
    Present(present) => inspect(present),
    Deferred(handle) => schedule(handle),
    Reconstructable(recipe) => rebuild(recipe)?,
    Unavailable(cause) => explain(cause),
}
```

The intended explanation path should be typed:

```rust
for row in artifact.materialization_decisions() {
    row.subject();
    row.cause();
    row.seam();
    row.attachment_point();
}
```

These examples are not cosmetic. They describe the actual DX target the
implementation should converge toward:

- callers ask for `summary`, `report`, `artifact`, or `receipt` by semantic
  name
- callers name the seam explicitly
- callers attach profile meaning explicitly
- advanced callers can inspect plan, cost, delivery class, availability, and
  decision rows before materialization
- authority-bearing lanes are visibly stronger than descriptive lanes
- multi-surface emission is a typed bundle, not an arbitrary collection
- optional surfaces expose typed availability rather than nullable folklore
- explanation comes from structured decision rows rather than prose recovery

The implementation may choose better names, but if the final call sites do not
feel approximately like these examples, the milestone has likely left DX value
on the table.

## Sequencing Notes

Milestone 4 belongs immediately after profiles because every later
artifact-bearing surface depends on it:

- Milestone 5 needs category, role, and materialization law before branch,
  merge, and commit evidence can stay honest
- Milestone 6 diagnostics need shared report/artifact category law before
  explanation breadth can materialize consistently
- Milestone 7 provenance and receipts need boundary artifact category law
  before their attachments can be interpreted consistently
- Milestone 8 performance vocabulary needs explicit materialization seams and
  cost carriers before layout/performance reporting can attach honestly
- Milestone 11 migrations need one shared boundary artifact language before
  crate-local envelope dialects can retire

This milestone must remain before Milestone 5 because branch/merge/commit
surfaces need somewhere category-safe to land. It must also remain after
Milestone 3 because boundary artifact materialization needs profile-governed
elision and certification posture to already exist.

## Explicit Non-Goals

- branch, merge, or commit authority-transition ontology
- diagnostics or explanation ontology
- provenance or lineage ontology
- full receipt semantics beyond category/role/materialization boundaries
- a universal artifact registry, executor, or storage model
- one internal artifact layout across crates
- replacing `worth-proof` authority/readiness progression law

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes. It standardizes the boundary artifact categories and
  materialization law that later milestones otherwise would attach to through
  incompatible local envelopes.
- Is the adversarial constraint precise and load-bearing? Yes. It attacks
  category collapse, authority impersonation, hidden materialization cost,
  truth-changing elision, premature Milestone 5 leakage, and producer-private
  envelope semantics.
- Does the milestone preserve crate authority boundaries? Yes.
  `worth-foundational` owns shared boundary meaning and materialization
  contracts; domain crates keep runtime execution, storage topology, and real
  authority transitions.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Closure requires category adjacency hostility, authority-admission
  proof, materialization honesty, basis parity, compile-fail boundaries, and
  readiness evidence.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The phases map directly to categories, roles, materialization,
  basis lowering, planned/same-family extension room, and readiness.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. Profiles and canonical basis must exist first, and branch/merge/commit,
  diagnostics, provenance, and migration work must come afterward.
