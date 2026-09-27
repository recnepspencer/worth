# Milestone 3: Profile And Policy Vocabulary

## Goal

Define the shared profile and profile-driven policy vocabulary that lets WORTH
crates describe richness, support, compatibility, admission, retention,
certification posture, and profile narrowing in one canonical language without
smuggling policy execution into `worth-foundational`.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering and mechanical enforcement.
The shaping constraint is that profile semantics must close the hard boundary
problem first: optional descriptive richness has to become centrally
controllable without risking authoritative truth drift.

### `arch_laws.md`

Protects proof-bearing phase boundaries, explicit authority/derivation
separation, and structured non-binary outcomes. The shaping constraint is that
profiles must attach to boundary artifacts without becoming authority and must
preserve category distinctions such as advisory versus denied posture.

### `composition_laws.md`

Protects responsibility-shaped files and named semantic steps. The shaping
constraint is that profile families, profile composition, profile identity,
attachment, and materialization planning must live in separate responsibility
homes rather than one broad `profiles` dump.

### `domain_structure_laws.md`

Protects structure as responsibility topology rather than convenience filing.
The shaping constraint is that profile family vocabulary, attachment contracts,
identity derivation, and materialization/elision planning must be independently
locatable and testable.

### `perf_laws.md`

Protects cost-honest boundaries, explicit policy decisions before execution,
and visible materialization cost. The shaping constraint is that reduced
richness must suppress only optional descriptive work at named seams and must
never hide broad materialization behind cheap-looking APIs.

### `worth_foundational_vision.md`

Protects the thesis that WORTH needs one shared profile language for richness,
support posture, and descriptive elision while preserving crate-local runtime
representation and policy execution. The shaping constraint is that Milestone 3
must make profile semantics canonical without creating a universal policy
engine.

### `worth_foundational_roadmap.md`

Protects the sequencing rule that profiles come after canonical value and
canonical basis law, but before artifact taxonomy, branch/merge/commit
vocabulary, diagnostics ontology, provenance, and migrations. The shaping
constraint is that Milestone 3 must make profile identity and elision basis
available to those later surfaces without stealing their ontology.

### `test-requirements.md`

Protects standalone proof before broad adopting-crate migration. The shaping
constraint is that profile identity, profile composition, reduced-richness
elision, proof-bearing attachment, and readiness posture must all be certified
through hostile local doubles, compile-fail boundaries, blind-consumer tests,
and production-test readiness evidence.

### `milestone-1.md`

Protects the canonical value, contract, mask, identity, locator, and
authoritative-state substrate. The shaping constraint is that profiles may
govern descriptive breadth around those surfaces, but must not reopen or blur
their authority law.

### `milestone-1-closeout.md`

Protects the fact that aspect-native values, authoritative state, patches,
identities, locators, and compatibility lowering are already closed and locally
certified. The shaping constraint is that Milestone 3 must compose with those
surfaces rather than redefining them.

### `milestone-2.md`

Protects canonical basis, equivalence basis, mismatch basis, export fixtures,
digest slots, and readiness artifacts as the semantic evidence substrate. The
shaping constraint is that profile identity and profile attachment must become
canonical-basis participants rather than inventing profile-local digest law.

### `milestone-2-closeout.md`

Protects the completed canonicalization substrate and the rule that digest
values are derived compression, not semantic authority. The shaping constraint
is that Milestone 3 must reuse that basis machinery for profile identity and
profile-sensitive certification.

## Why This Milestone Exists

Milestones 1 and 2 made foundational meaning constructible and reproducible.
They did not yet answer how WORTH centrally describes:

- how rich a boundary surface should be
- which descriptive surfaces are optional versus required
- when a support surface is compatibility-shaped, evidence-backed, or
  production-certified
- how reduced-richness operational boundaries can suppress history, replay,
  lineage, provenance, or forensic detail without changing authoritative truth
- how proof-bearing artifacts from `worth-proof` can carry one shared profile
  language without importing profile execution into the proof kernel

If each crate keeps solving those questions with local strings, booleans, or
ad hoc enums, WORTH will regain ontology drift exactly where support,
diagnostics, certification, and hot-path austerity pressure meet.

Milestone 3 exists to create one shared answer for profile meaning while
keeping execution local:

- typed profile families
- composition rules for compatible and incompatible profile combinations
- requested, admitted, and materialized profile progression
- canonical profile basis and profile identity
- attachment contracts for foundational and proof-bearing artifacts
- explicit materialization and elision planning over optional descriptive
  surfaces
- explicit absence-cause and non-assumption vocabulary for unavailable
  descriptive surfaces
- profile-difference and compatibility classification
- certification posture vocabulary for uncertified, evidence-backed, and
  production-certified support surfaces

## Adversarial Constraint

Several WORTH crates with different support surfaces, diagnostics layouts,
retention mechanics, and proof-bearing artifact boundaries must be able to
attach the same profile meaning to semantically identical artifacts, derive the
same canonical profile identity, explain how requested profile meaning was
narrowed into admitted and materialized profile meaning, and centrally suppress
optional descriptive materialization at named seams, while preserving identical
authoritative outcomes and without depending on producer-private strings,
implicit defaults, or crate-local policy engines.

This milestone fails if:

- profiles are string labels or booleans instead of typed shared meaning
- reduced-richness operational profiles can silently change authoritative
  values, patches, proofs, receipts, or commit outcomes
- profile narrowing happens implicitly with no canonical explanation of what
  changed and why
- profile identity depends on insertion order, display text, or local layout
- proof-bearing artifacts can carry arbitrary descriptive policy with no target
  or category law
- materialization planning leaves omitted descriptive surfaces implicit instead
  of explicit
- unavailable descriptive surfaces do not carry typed absence cause
- compatibility between two profiles is guessed from labels rather than
  classified structurally
- certification posture collapses with support richness or retention posture
- `worth-foundational` starts owning runtime policy execution instead of shared
  policy meaning

## WORTH-Proof Dependency Boundary

Milestone 3 uses `worth-proof` for proof-bearing attachment and progression
surfaces, but not for plain profile vocabulary.

`worth-proof` is mandatory for:

- profile attachment to proof-bearing or foundational boundary artifacts where
  the attachment must itself be an admitted boundary state
- checked transitions when profile attachment, materialization planning, or
  certification posture admission can deny, defer, or reject a request
- profile-sensitive readiness or support artifacts that require proof-bearing
  posture rather than plain labels
- explicit progression surfaces when a requested profile becomes an admitted
  profile-bearing form and when an admitted profile-bearing form becomes a
  materialized profile-bearing form
- trust-boundary weakening and readmission whenever a profiled proof-bearing
  artifact crosses export, persistence, replay, or other boundary seams and
  later needs strong basis restoration
- stronger certification/readiness attachment states when evidence-backed or
  production-certified posture must be more than a plain enum claim

`worth-proof` is forbidden for:

- plain profile family vocabulary
- plain profile composition data
- plain descriptive-surface vocabulary
- plain profile identity basis entries
- replacing artifact-category, diagnostics, provenance, or receipt ontology
  owned by later milestones

The operating rule is:

`worth-foundational` defines what a profile means.
`worth-proof` proves which artifact is allowed to carry that profile and which
stronger posture that attachment has reached.

The implementation should specifically evaluate whether the proof-bearing
surfaces map onto `Artifact<...>`, `TransitionOutcome<...>`,
`AuthorityWitness<...>`, checked transition helpers, and boundary
readmission/freshness forms from `worth-proof` rather than recreating those
concepts locally under new names.

Named `worth-proof` surfaces to evaluate first:

- pleasant lane imports:
  - `use worth_proof::prelude::*;`
  - `recipe(...)`
  - `.resolve_with(...)`
  - `.lower_with(...)`
  - `.admit_with(...)`
  - `.ready_with(...)`
  - `.bridge_trust_boundary()`
  - `.rebind_with(...)`
  - `.readmit_with(...)`
  - `.try_resolve_ready(...)`
  - `.try_lower_ready(...)`
  - `.try_admit_ready(...)`
  - `.try_ready_now(...)`
  - `.try_execute()`
  - `ProofOutcome`
  - `ProofOutcomeKind`
  - `gate_ready(...)`
  - `ready_now(...)`
  - `AuthorityWitness::from_authority_marker(...)`
  - `CapabilityWitness::from_capability_marker(...)`
- raw lane imports:
  - `use worth_proof::raw::*;`
  - `Artifact<P, T, S, A>`
  - `Recipe<S, T, A>`
  - `ResolveRecipeTransition`
  - `LowerRecipeTransition`
  - `AdmitRecipeTransition`
  - `AdmitExecutionReadyRecipeTransition`
  - `ExecuteReadyRecipeTransition`
  - `CheckedResolveRecipeTransition`
  - `CheckedLowerRecipeTransition`
  - `CheckedAdmitRecipeTransition`
  - `CheckedAdmitExecutionReadyRecipeTransition`
  - `ReadmitLoweredForExecutionReadyTransition`
  - `CheckedReadmitLoweredForExecutionReadyTransition`
  - `RecipeResolutionContext`
  - `ExecutionReadinessContext`
  - `LoweredReadmissionContext`
  - `RecipeResolutionGate`
  - `RecipeLoweringReadiness`
  - `RecipeAdmissionReadiness`
  - `ExecutionReadyAdmissionReadiness`
  - `LoweredReadmissionReadiness`
  - `TransitionOutcome`
  - `PreConstructionGate`
  - `TransitionReadiness`
  - `readmit_ready_and_execute_recipe(...)`
  - `checked_readmit_ready_and_execute_recipe(...)`

## Practical Type Targets

The implementation may choose better names, but these responsibilities must
exist concretely somewhere:

```rust
pub enum DiagnosticRichnessProfile {
    OperationalMinimal,
    Standard,
    Forensic,
}

pub enum SupportPostureProfile {
    InternalOnly,
    SupportReady,
    CertificationReady,
}

pub enum CompatibilityPostureProfile {
    NativeOnly,
    CompatibilityLowered,
    CompatibilityRequired,
}

pub enum AdmissionReadinessProfile {
    CandidateOnly,
    Admitted,
    ProductionGateReady,
}

pub enum RetentionDeliveryProfile {
    Ephemeral,
    Retained,
    Durable,
}

pub enum CertificationPostureProfile {
    Uncertified,
    EvidenceBacked,
    ProductionCertified,
}

pub struct FoundationalProfileSet {
    /* private fields */
    // one required slot per shared family; no missing-family or duplicate-family
    // construction path is public
}

pub struct FoundationalProfileIdentity { /* private fields */ }

pub struct RequestedFoundationalProfileSet { /* private fields */ }

pub struct AdmittedFoundationalProfileSet { /* private fields */ }

pub struct MaterializedFoundationalProfileSet { /* private fields */ }

pub struct FoundationalProfileAttachmentAdmission { /* private fields */ }

pub struct FoundationalProfileMaterializationTransition { /* private fields */ }

pub struct FoundationalProfileNarrowingRecord { /* private fields */ }

pub enum FoundationalProfileNarrowingKind {
    RichnessReduced,
    RetentionNarrowed,
    SupportPostureReduced,
    CertificationPostureReduced,
    CompatibilityRestricted,
}

pub enum FoundationalDescriptiveSurface {
    History,
    Replay,
    Lineage,
    Provenance,
    ForensicDiagnostics,
}

pub enum FoundationalSurfaceAbsenceCause {
    OmittedByActiveRichness,
    DeniedByBudget,
    NotRetained,
    NotReconstructable,
    DeferredBySupportPosture,
    UncertifiedForRequestedPosture,
}

pub struct FoundationalSurfaceAvailabilityDecision { /* private fields */ }

pub struct FoundationalProfileMaterializationPlan { /* private fields */ }

pub trait FoundationalProfileAttachmentTargetMarker: sealed::Sealed {}

pub struct BoundaryArtifactTarget;
pub struct SupportArtifactTarget;
pub struct ProofBearingArtifactTarget;

pub struct FoundationalProfiledArtifact<Target, T> { /* private fields */ }

pub enum FoundationalProfileAttachmentTargetKind {
    BoundaryArtifact,
    SupportArtifact,
    ProofBearingArtifact,
}

pub struct FoundationalProfileApplicability { /* private fields */ }

pub struct FoundationalTargetSurfaceInventory<Target> { /* private fields */ }

pub enum FoundationalProfileCompatibilityClass {
    Exact,
    RichnessOnlyChange,
    RetentionOnlyNarrowing,
    SupportPostureChange,
    CertificationPostureChange,
    Incompatible,
}

pub struct FoundationalProfileDifferenceReport { /* private fields */ }
```

These sketches imply concrete constraints:

- profile families remain distinct typed vocabularies
- composition is explicit, total, and rejects incoherent combinations
- requested, admitted, and materialized profile sets are distinct progression
  surfaces
- requested, admitted, and materialized profile-set surfaces should be opaque
  wrappers or equivalent sealed stage-specific carriers, not type aliases over
  one mutable record
- proof-bearing progression and checked non-success outcomes reuse
  `worth-proof` transition law rather than inventing local pseudo-proof result
  wrappers
- a composed profile set cannot be an arbitrary bag, map, or list of profile
  entries; it must expose one explicit slot per required family or an
  equivalent sealed total-construction contract
- narrowing is explicit and explainable rather than implicit
- profile identity is derived from canonical basis participation rather than
  raw digest substitution
- attachment requires an explicit target kind
- optional descriptive surfaces are centrally enumerated and planned explicitly
- unavailable surfaces carry typed absence cause
- statically known target legality should be expressed through typed target
  markers and generic inventories/wrappers wherever possible, not only through
  runtime enums
- profile compatibility and profile difference are structural classifications,
  not string heuristics
- materialized profile meaning is derived from admitted profile meaning plus
  target-scoped materialization decisions; it is not a second free-form profile
  dialect

## Naive Traps To Avoid

- Do not model `FoundationalProfileSet` as `Vec<ProfileEntry>`,
  `BTreeMap<Family, Value>`, or any equivalent bag-of-entries structure. The
  milestone wants one total composed profile meaning, not a collection that can
  hide missing or duplicate families.
- Do not let `MaterializedFoundationalProfileSet` become a mutable clone of the
  admitted profile set. Materialized meaning must be derived from admitted
  meaning plus legal target-scoped surface decisions, not rebuilt as a second
  profile dialect.
- Do not let materialization output participate silently in canonical profile
  identity. Identity belongs to canonical profile meaning; target-scoped
  planning output is a separate derivative artifact.
- Do not use `Option<T>` absence alone as the semantics for unavailable
  descriptive surfaces. Unavailability must carry typed absence cause.
- Do not enforce family/target legality only in documentation or runtime logs
  when the target kind is statically known. Use typed target markers, sealed
  constructors, or compile-fail boundaries wherever the crate can enforce them.
- Do not expose `Default` for composed, requested, admitted, or materialized
  profile-set types unless the default is itself a named semantic constructor
  with one canonical meaning and dedicated tests. Silent default profile meaning
  is exactly the kind of folklore this milestone is meant to eliminate.

## Phases

Phase progression gates:

| Phase | Gate before next phase |
| --- | --- |
| Phase 1 | Typed profile families and family-local incoherence law exist before any composed profile set can exist. |
| Phase 2 | One sealed total composed profile-set shape exists before progression, attachment, or identity work begins. |
| Phase 3 | Requested/admitted/materialized progression and target-kind-aware attachment exist before identity derivation. |
| Phase 4 | Canonical profile basis, profile identity, and profile-difference law exist before materialization/elision planning. |
| Phase 5 | Materialization planning, absence-cause law, and target surface inventories are explicit before certification posture can rely on them. |
| Phase 6 | Certification posture and proof-bearing strengthening law exist before milestone readiness closes. |
| Phase 7 | Production-test readiness evidence exists before Milestone 4 consumes profile-sensitive artifact categories. |

### Phase 1: Define Typed Profile Families

Purpose:

Create the shared vocabulary for profile meaning before any attachment or
materialization logic exists.

Engineer order:

Define the family nouns first and freeze their meanings before writing any
composition or attachment API. This phase is complete only when an engineer can
point at each family and say what it means, what it does not mean, and which
neighboring family it must not be confused with.

Must ship:

- typed profile families for diagnostic richness, support posture,
  compatibility posture, admission/readiness posture, retention/delivery
  posture, and certification posture
- explicit ordering and incompatibility rules where profile families have
  meaningful escalation or denial relationships
- typed denial vocabulary for duplicate, conflicting, or incoherent profile
  entries
- facade exports for milestone-owned profile nouns only

Must preserve:

- profile meaning is typed, not stringly
- neighboring profile families remain distinct even when they often travel
  together
- profile law standardizes shared meaning, not policy execution

Acceptance evidence:

- runtime tests proving duplicate and incoherent family combinations deny
- category-adjacency tests proving neighboring families are not substitutable
- compile-fail tests proving raw strings or untyped labels cannot satisfy
  profile-family APIs where the type system can enforce it

### Phase 2: Define The Composed Profile Set

Purpose:

Turn the family vocabulary into one practical, sealed profile-set shape that
every later phase can actually carry, compare, attach, and canonicalize.

Engineer order:

Build the one total composed set next, and do not start progression or target
attachment work until the constructor story is closed. The practical test for
completion is that all required families can be assigned exactly once, no
family can be forgotten, and no caller can smuggle in a partial or duplicate
set through a generic collection.

Must ship:

- `FoundationalProfileSet` or equivalent sealed composed profile surface
- one explicit participation slot for each required profile family or an
  equivalent sealed constructor that guarantees total family coverage
- composition rules that reject incoherent combinations before attachment or
  identity work begins
- builder or constructor APIs that make family assignment explicit rather than
  accepting arbitrary collections of entries
- facade exports for the composed-set surface without exposing public fields

Must preserve:

- a composed profile set cannot be represented as an untyped map, string-keyed
  registry, or append-only list of profile entries
- missing family assignment cannot silently fall back to local defaults
- duplicate family assignment cannot survive construction as "last write wins"
- composition law remains about shared meaning, not target-specific attachment
  or runtime materialization

Acceptance evidence:

- compile-fail tests proving raw collections, string-keyed maps, or duplicate
  family entries cannot satisfy composed profile-set APIs
- compile-fail tests proving callers cannot obtain a silent default composed
  profile set through `Default` or equivalent unnamed construction paths
- construction-path parity tests proving independently assembled total profile
  sets compare equal when family meaning is identical
- hostile tests proving incompatible family combinations deny before
  attachment-specific logic runs

### Phase 3: Define Progression And Attachment Contracts

Purpose:

Take the sealed composed profile set and define how it progresses from
requested meaning to admitted meaning to materialized meaning, and where those
states may legally attach.

Engineer order:

Start by defining the stage carriers for requested, admitted, and materialized
meaning. After that, define legal targets and target evidence, and only then
define the transitions that attach or narrow profile meaning. If an engineer
cannot explain which transitions exist before materialization and which exist
only after a legal target is known, this phase is still too fuzzy.

Must ship:

- distinct requested, admitted, and materialized profile-set surfaces
- proof-bearing requested-to-admitted and admitted-to-materialized transition
  surfaces using `worth-proof` where the attachment itself becomes a stronger
  state
- an implementation decision for whether those transitions are best modeled as
  `Artifact<...>` carriers, `Recipe<...>` carriers, or a narrowly adapted
  foundational wrapper over one of those two existing substrates
- attachment policies describing which target kinds may receive which profile
  sets
- `FoundationalProfiledArtifact<Target, T>` or equivalent profiled boundary
  wrapper
- explicit attachment-target vocabulary and target-kind evidence
- target-scoped legality rules that can reject invalid family/target
  combinations before or during attachment
- explicit narrowing records from requested to admitted and admitted to
  materialized profile meaning
- denial outcomes for invalid target/profile combinations

Must preserve:

- raw payloads cannot receive foundational profiles directly
- attachment does not mutate authoritative payload truth
- target kind remains visible after attachment
- materialized profile meaning cannot exist until an admitted profile meaning
  and a legal attachment target already exist
- narrowing cannot happen silently
- progression denial, deferment, stale inputs, or readmission needs remain
  typed when a proof-bearing attachment path uses `worth-proof`
- one target kind cannot inherit another target kind's admissibility by
  convenience

Acceptance evidence:

- attachment tests proving support-facing and proof-bearing targets can carry
  reduced-richness profiles without mutating payload meaning
- compile-fail tests proving plain payloads and wrong target kinds cannot
  satisfy attachment APIs
- compile-fail tests proving statically known illegal family/target
  combinations cannot enter the attachment path
- narrowing tests proving a stronger requested profile can be reduced only
  through an explicit narrowing record
- checked-transition tests proving proof-bearing attachment progression
  preserves success, denial, deferment, and readmission-required categories
- explicit evaluation notes naming which `worth-proof` lane was chosen for
  Phase 3:
  - prelude verbs such as `.admit_with(...)` and `.try_admit_ready(...)`
  - raw transitions such as `AdmitRecipeTransition`,
    `CheckedAdmitRecipeTransition`, `AdmitExecutionReadyRecipeTransition`, or
    `CheckedAdmitExecutionReadyRecipeTransition`
- blind-consumer tests proving attached profile sets are interpretable without
  producer-private context

### Phase 4: Define Canonical Profile Basis And Profile Identity

Purpose:

Make profile meaning reproducible and digest-honest by lowering profile sets
through Milestone 2 canonical basis law.

Engineer order:

Once progression is closed, define canonical basis participation for admitted
profile meaning and derive profile identity from that basis. Only after
identity is stable should difference and compatibility reporting be added,
because those reports must talk about already-canonical meaning rather than
builder-path accidents.

Must ship:

- profile canonical-basis participation through a distinct profile domain or
  equivalent typed basis path
- `FoundationalProfileIdentity` or equivalent identity carrier
- profile-identity derivation that consumes canonical basis readiness rather
  than raw digest bytes
- profile-difference and compatibility classification surfaces
- deterministic ordering for family and token participation in the basis
- explicit residual debt markers if digest algorithm policy remains deferred

Must preserve:

- profile identity is derived from canonical basis, not display labels
- family ordering and composition order cannot drift semantic identity
- raw digest values cannot masquerade as profile identity
- structural profile differences cannot be hidden behind â€œsame enoughâ€ labels

- admitted profile identity and materialization-plan identity remain distinct;
  target-scoped planning output must not silently become the semantic identity
  of the profile set itself

Acceptance evidence:

- order-independence tests proving semantically identical profile sets produce
  identical identity
- hostile tests proving semantic profile changes alter identity while
  preserving authoritative payload truth
- profile-difference tests proving richness-only, retention-only,
  support-posture, certification-posture, and incompatible changes classify
  distinctly
- compile-fail tests proving raw canonical digests and private identity fields
  cannot satisfy profile-identity APIs

### Phase 5: Define Materialization And Elision Planning

Purpose:

Centralize reduced-richness behavior at named boundary seams instead of
scattering profile branches through leaf call sites.

Engineer order:

List the legal optional descriptive surfaces per target first, then define
availability and absence-cause vocabulary, and only then define planning APIs.
The phase is not done when there is a planner function; it is done when an
engineer can enumerate every legal optional surface for a target without using
free-form strings or leaf-call-site folklore.

Must ship:

- typed vocabulary for optional descriptive surfaces such as history, replay,
  lineage, provenance, and forensic diagnostics
- a closed target-surface inventory describing which optional descriptive
  surfaces can ever exist for each legal attachment target
- exhaustive materialization planning over those surfaces
- explicit partial-planning APIs where a caller intentionally requests a subset
- typed availability and absence-cause vocabulary for every optional
  descriptive surface
- target-scoped applicability rules for which profile families and profile
  decisions may govern which artifact kinds
- materialization cost vocabulary for named planning/output breadth
- elision profiles capable of suppressing only optional descriptive surfaces

Must preserve:

- omitted descriptive surfaces are explicit decisions, not silent absence
- reduced richness cannot change authoritative values, proofs, or receipts
- expensive materialization remains a visible boundary
- unavailable surfaces are explainable without producer-private interpretation
- profile applicability is target-scoped rather than ambient
- target-surface inventories remain closed and exhaustive enough that later
  code cannot invent new optional surface names through strings or ad hoc
  option fields

Acceptance evidence:

- exhaustive-versus-selected planning tests
- hostile tests proving replay/history/provenance/forensic suppression removes
  only optional descriptive surfaces
- absence-cause tests proving omitted, denied, not-retained, not-reconstructable,
  deferred, and uncertified surfaces remain distinct
- applicability tests proving a profile family cannot govern an illegal target
  kind
- inventory exhaustiveness tests proving each legal target kind has one closed
  optional-surface inventory and that illegal surface names cannot enter the
  plan vocabulary
- compile-fail tests proving raw strings or ad hoc surface labels cannot stand
  in for descriptive-surface vocabulary
- compile-fail tests proving code cannot construct target-surface inventories
  that mention surfaces illegal for the target when the target is statically
  known

### Phase 6: Define Certification Posture And Proof-Composed Profile Surfaces

Purpose:

Introduce the shared posture vocabulary that later support and certification
artifacts will rely on.

Engineer order:

Define certification posture only after materialization law is already closed,
because stronger posture depends on what evidence and retained surfaces are
available. Then choose the exact `worth-proof` strengthening lane and make that
choice explicit instead of leaving future engineers to infer whether
certification uses plain wrappers, checked outcomes, or readmission-aware
progression.

Must ship:

- `CertificationPostureProfile::{Uncertified, EvidenceBacked, ProductionCertified}`
  or equivalent typed posture family
- composition rules requiring stronger posture to carry the necessary support
  and retention commitments
- profile-sensitive support/certification attachment contracts for proof-bearing
  artifacts where posture matters
- explicit proof-bearing strengthening path for evidence-backed and
  production-certified attachment states using `worth-proof` witnesses,
  transitions, or equivalent sealed progression law
- an explicit decision for whether certification/readiness progression is best
  expressed through plain readiness wrappers such as `ExecutionReadyRecipe` and
  `ExecutedRecipe` or through checked transition surfaces that preserve denial,
  deferment, stale, and rebind-required outcomes
- canonical basis participation for certification posture

Must preserve:

- certification posture is not the same thing as diagnostic richness or
  retention breadth
- production-certified posture cannot be claimed without the stronger composed
  profile basis it requires
- stronger certification posture on proof-bearing artifacts cannot survive
  boundary crossing as an unexamined assumption; readmission/revalidation law
  must remain available where later milestones need it
- later artifact, diagnostics, and provenance milestones remain the owners of
  their ontologies

Acceptance evidence:

- posture-ordering and incoherence tests
- profile-identity tests proving certification posture participates in canonical
  identity
- proof-bearing posture-transition tests proving stronger certification states
  require explicit authority/capability progression rather than caller-minted
  labels
- explicit evaluation notes naming the `worth-proof` posture lane chosen for
  Phase 6:
  - `ExecutionReadinessContext`
  - `ExecutionReadyAdmissionReadiness`
  - `AdmitExecutionReadyRecipeTransition`
  - `CheckedAdmitExecutionReadyRecipeTransition`
  - `ProofOutcome` or raw `TransitionOutcome`
- compile-fail or attachment-boundary tests proving caller-minted posture or
  wrong-strength support surfaces cannot satisfy stronger APIs

### Phase 7: Certify Production-Test Readiness

Purpose:

Close Milestone 3 with a proof-bearing readiness artifact that later
artifact-category work can depend on.

Engineer order:

Do not treat this as documentation cleanup. This phase is where the milestone
proves which APIs, compile-time boundaries, hostile cases, and non-assumptions
actually survived implementation. If a surface was important enough to appear
in earlier phases but is absent from the readiness artifact, then the milestone
is not actually closed.

Must ship:

- a Milestone 3 production-test readiness artifact or report
- certified-surface inventory for profile families, composition, attachment,
  identity, materialization planning, and certification posture
- explicit inventory of where Milestone 3 relies on `worth-proof` progression,
  checked outcomes, freshness, and readmission surfaces
- compile-fail boundary inventory
- hostile-pressure inventory for reduced richness, category adjacency, and
  attachment target law
- runtime adoption assumptions, non-assumptions, and residual debt
- a named API appendix or equivalent inventory section listing the chosen
  `worth-proof` prelude verbs, checked helpers, raw transitions, gate types,
  and readmission helpers that Milestone 3 actually standardizes against

Must preserve:

- adopting crates may assume only what the readiness artifact names
- local doubles remain semantic fixtures rather than generic policy engines
- later milestones still own diagnostics, artifact categories, provenance, and
  receipts

Acceptance evidence:

- readiness tests proving every certified surface has hostile evidence,
  compile-fail coverage, and blind-consumer interpretation where required
- exact inventory tests for runtime non-assumptions and residual debt
- topology review proving profile tests live in responsibility-owned homes

## Must Ship

- typed profile family vocabulary for richness, support, compatibility,
  admission/readiness, retention/delivery, and certification posture
- one sealed total composed profile-set surface with explicit family coverage
- explicit profile composition and incoherence law
- requested, admitted, and materialized profile progression surfaces
- explicit profile narrowing records and narrowing kinds
- target-kind-aware profile attachment contracts
- typed target markers and target-parameterized profiled wrappers/inventories
- profiled boundary artifact wrappers for admitted attachment targets
- canonical profile basis participation and profile identity derivation
- profile-difference and compatibility classification
- typed descriptive-surface vocabulary and central materialization/elision
  planning
- closed target-surface inventories for legal attachment targets
- typed surface availability and absence-cause vocabulary
- target-scoped profile applicability rules
- certification posture participation in profile identity
- production-test readiness artifact for Milestone 3

## Must Preserve

- profile meaning remains shared boundary vocabulary, not runtime policy
  execution
- reduced-richness profiles remove only optional descriptive surfaces
- authoritative values, patches, proofs, and receipts remain unchanged under
  profile elision
- requested profile meaning, admitted profile meaning, and materialized profile
  meaning remain distinct
- narrowing and unavailability remain explicitly explainable
- profile identity depends on canonical basis, not display text or builder
  order
- materialization planning remains derivative from admitted profile meaning and
  does not become a second semantic identity channel
- target kinds and category boundaries remain explicit during attachment
- later milestones retain ownership of artifact taxonomy, diagnostics,
  provenance, receipts, and branch/merge/commit ontology

## Acceptance Evidence

- profile composition parity tests across independent construction paths
- compile-fail tests for raw labels, wrong target kinds, plain payload
  attachment, raw digest substitution, private-field synthesis, and missing or
  duplicate family assignment, plus unnamed default profile construction
- requested-to-admitted and admitted-to-materialized narrowing tests
- profile-difference and compatibility classification tests
- hostile reduced-richness tests over optional descriptive surfaces
- absence-cause and availability-decision tests for optional descriptive
  surfaces
- target-surface inventory exhaustiveness tests
- target-applicability hostility tests
- blind-consumer tests proving profiled artifacts and profile identity are
  interpretable without producer-private state
- readiness artifact coverage proving the plan names exactly where
  `worth-proof` is required, where it is forbidden, and which profile-bearing
  surfaces still remain plain foundational vocabulary
- readiness artifact tests covering certified surfaces, hostile pressures,
  compile-fail boundaries, assumptions, non-assumptions, and debt

## Architectural Notes

The implementation should preserve distinct profile responsibility homes. A
likely shape is:

```text
crates/worth-foundational/src/
  profiles/
    families/
    composition/
    attachments/
    identity/
    materialization/
    readiness/
```

Public exports should remain facade-controlled. The `profiles` root may exist,
but it must not become an unnamed bucket where family vocabulary, attachment
contracts, canonical basis lowering, and readiness reporting collapse into one
file. The progression and explanation surfaces should also stay separate:

- `requested/` or equivalent owns caller-requested profile intent
- `admission/` or equivalent owns narrowed/admitted profile meaning
- `materialization/` or equivalent owns target-scoped realized profile meaning
- `difference/` or equivalent owns compatibility and diff classification
- `availability/` or equivalent owns absence-cause and unavailability law

## Sequencing Notes

Milestone 3 belongs immediately after canonicalization because every later
profile-sensitive surface depends on it:

- Milestone 4 needs profile-driven materialization and category-aware support
  posture before artifact taxonomy can stay honest.
- Milestone 5 needs profile-sensitive branch/merge/commit reporting before
  reduced-richness authority-transition reporting can be safe.
- Milestone 6 diagnostics need shared richness posture before explanation
  breadth can be standardized.
- Milestone 7 provenance and receipts need support/certification posture before
  those attachments can be interpreted consistently.
- Milestone 11 migrations need one shared profile language before crate-local
  policy dialects can retire.

## Explicit Non-Goals

- a universal runtime policy engine
- artifact/report/summary/receipt taxonomy
- diagnostics or explanation ontology
- provenance, lineage, or receipt ontology
- branch/merge/commit authority-transition vocabulary
- forcing one retention store, diagnostics store, or support matrix layout
- replacing `worth-proof` progression law

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes. It standardizes profile meaning and central elision where
  several later boundary categories would otherwise drift independently.
- Is the adversarial constraint precise and load-bearing? Yes. It attacks
  string labels, order-dependent identity, hidden materialization, arbitrary
  attachment, silent narrowing, untyped absence, and truth-changing reduced
  richness.
- Does the milestone preserve crate authority boundaries? Yes.
  `worth-foundational` owns profile meaning; domain crates keep policy
  execution and runtime behavior.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Closure requires compile-fail attachment boundaries, canonical profile
  identity evidence, narrowing evidence, hostile reduced-richness tests,
  absence-cause law, compatibility classification, and readiness artifacts.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The phases map directly to family vocabulary, composition,
  attachment, identity, materialization planning, certification posture, and
  readiness.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. Canonical basis must exist first, and profile-sensitive artifact,
  diagnostic, provenance, and migration work must come afterward.
