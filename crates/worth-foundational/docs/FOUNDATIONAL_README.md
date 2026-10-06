# WORTH Foundational Orientation For AI Agents

This document is the orientation map for AI agents working in or around
`worth-foundational`.

It is not an API reference. Its job is to answer three questions:

1. What kind of problem is `worth-foundational` for?
2. When is `worth-foundational` the right crate, and when is a stronger crate
   such as `worth-query` or a domain runtime the right one instead?
3. Which docs should you read next for the real API details?

If you need exact signatures, type names, or step-by-step examples, use the
linked docs. This file is the mental model and routing layer.

## Runtime Stack

WORTH Foundational lives in this stack:

```text
application / downstream crate
-> owning runtime or domain crate
   (worth-query, worth-store, worth-signal, worth-relational, etc.)
-> worth-foundational
-> worth-proof
```

That layering matters.

`worth-proof` owns proof-bearing progression law: witnesses, transitions,
stages, proof carriers, and checked progression topology.

`worth-foundational` owns shared boundary vocabulary: typed values, aspect
state, canonical basis, identity boundaries, diagnostics, profiles, boundary
artifacts, provenance, lineage, support truth, performance claims, and shared
transition nouns.

Owning runtimes and domain crates still own their runtime semantics:

- authoritative truth
- read and write execution
- query planning
- reactive invalidation
- storage layout
- runtime-local receipts
- domain-specific meaning

Foundational is beneath those crates. It is not the ordinary runtime entry
point for application behavior.

## The Core Rule

The governing Foundational rule is:

```text
use the strongest owning type available
lower into foundational when you need shared boundary meaning
do not replace stronger runtime or domain types with foundational
just because the names look similar
```

That one rule prevents most misuse.

If `worth-query` already gives you typed projection-consumption facts,
authority-lane state, read receipts, write receipts, execution artifacts, or
support-bound result surfaces, keep those Query types while the meaning is
still Query-owned.

If a domain runtime already owns the semantics of a hot execution plan, mounted
receipt, scheduling artifact, or mutation result, keep the domain type while
the meaning is still domain-owned.

Reach for `worth-foundational` when the object becomes one of these:

- a shared boundary artifact
- a canonical export
- a trust-boundary identity
- a support-grade diagnostic or evidence surface
- a portable provenance or lineage attachment
- a shared profile or policy posture
- a cross-crate performance or layout claim

Reach for `worth-proof` when the real problem is progression law rather than
boundary vocabulary.

## What This File Is For

Read this file in two passes.

First, find the section that matches the kind of thing you are touching:

- aspect state and typed values
- canonical identity and digest work
- boundary artifacts and receipts
- profiles and support posture
- diagnostics and explanation
- provenance, lineage, and support truth
- performance and layout claims
- transition vocabulary

Second, jump to the linked docs for the real surface details.

If you have no idea where to start, read these first:

- [Docs README](./README.md)
- [Aspect Contracts, Values, And Authoritative State](./aspect-contracts-values-and-authoritative-state/README.md)
- [Canonical Basis And Reproducible Identity](./canonical-basis-and-reproducible-identity/README.md)
- [Profiles And Policy Vocabulary](./profiles-and-policy-vocabulary/README.md)
- [Boundary Artifact Taxonomy And Materialization Contracts](./boundary-artifact-taxonomy-and-materialization-contracts/README.md)

## What WORTH Foundational Is Not

This crate is not:

- the public runtime facade for application reads and writes
- a replacement for `worth-query` projection consumption
- a replacement for Query authority lanes
- a planner
- a storage engine
- a runtime scheduler
- a generic "nice shared types" crate for all internal state
- a proof kernel
- a license to flatten stronger domain semantics into weaker shared vocabulary

The biggest failure mode is bypassing a stronger owning crate because
Foundational has a seemingly similar type.

Do not do that.

## Foundational vs Query vs Proof vs Domain Crates

Use this matrix before writing code.

| Need | Use |
| --- | --- |
| Typed facts from Query materialization | `worth-query` projection consumption |
| Query-owned authoritative or derived state | `worth-query` state, inspection, receipts, bindings |
| Domain-local runtime execution state | the owning runtime or domain crate |
| Shared trust-boundary identity | `worth-foundational` identity boundaries |
| Canonical basis, digest, export parity, mismatch classification | `worth-foundational` canonicalization |
| Shared profile or support posture | `worth-foundational` profiles |
| Shared summary/report/artifact/receipt vocabulary | `worth-foundational` boundary artifacts |
| Shared diagnostics or support reports | `worth-foundational` diagnostics |
| Shared provenance, lineage, or support-grade receipt meaning | `worth-foundational` boundary evidence |
| Shared performance claim or report vocabulary | `worth-foundational` performance |
| Proof-bearing progression law or checked phase transitions | `worth-proof` |

The practical rule is simple:

- if the semantics are still runtime-owned, stay in the runtime crate
- if the semantics are now cross-crate boundary meaning, Foundational is
  probably right
- if the semantics are about proof-bearing progression, `worth-proof` is
  probably right

## Public Surface And Discovery

Start from the public facade and the family READMEs. Do not spelunk internals
first.

Important discovery points include:

- `worth_foundational::aspects()`
- `worth_foundational::compatibility()`
- `worth_foundational::canonicalization()`
- `worth_foundational::boundary_artifact_surface_inventory()`
- `worth_foundational::boundary_evidence()`
- `worth_foundational::foundational_branch_candidate()`
- `worth_foundational::foundational_merge(...)`
- `worth_foundational::canonicalization_api`
- `worth_foundational::profiles_api`
- `worth_foundational::boundary_evidence_api`
- `worth_foundational::performance_api`
- `worth_foundational::expression_api`: the shared typed expression language,
  starting from `expressions().parse(source)?.admit(...)`
- `worth_foundational::foundational_transition_milestone5_readiness_report()`
- the facade exports re-exported from `worth_foundational::*`

Good to know:

- some families lead with common-path helpers
- some families lead with lower-lane types and named surface categories
- stronger lanes exist where the docs say they exist; do not assume every
  surface has a stronger lane just because another family does

When in doubt, start from the family README under `docs/` before reading tests
or internal modules.

## Aspects, Values, And Authoritative State

This family gives WORTH one shared typed vocabulary for aspect keys, contracts,
values, masks, struct carriers, validation, authoritative aspect state, and
patches.

Use this family when you need:

- one cross-crate typed value language
- explicit aspect contracts
- authoritative aspect state at a boundary
- explicit patch meaning
- canonical locator and field-path vocabulary
- projection, mutation, and diagnostic visibility law as part of the contract
- a discoverable common path for native authoring instead of flat type spelunking

Stable entry points start here:

- `worth_foundational::aspects()`
- `worth_foundational::compatibility()`
- [Aspect Contracts, Values, And Authoritative State](./aspect-contracts-values-and-authoritative-state/README.md)

The public native common path behind `aspects()` includes:

- `aspects().contract()`
- `aspects().struct_fields()`
- `aspects().mask_contract()`
- `aspects().projection_mask()`
- `aspects().mutation_mask()`
- `aspects().diagnostic_mask()`
- `aspects().validate()`
- `aspects().authoritative_state()`
- `aspects().patch()`
- `aspects().vocabulary()`

The actual capability inventory inside this family is bigger than "typed
values":

- contract authoring for scalar, struct, reference, content, and opaque shapes
- raw value carriers such as `AspectValue` and `StructAspectValue`
- struct field declarations and field-path vocabulary
- projection, mutation, and diagnostic mask contracts plus admitted masks
- validation from raw carriers into validated artifacts
- authoritative state admission from validated artifacts into canonical
  aspect-state maps
- explicit patch construction with separate set and clear semantics
- identity, locator, and digest-preparation inputs for later canonicalization

The important boundary is three-phase, not one-step:

- raw carriers are not validated meaning
- validated artifacts are not yet admitted authoritative state
- admitted authoritative state is the first real authority boundary in this
  family

The important lane split is also real:

- native authoring goes through `aspects()`
- compatibility debt goes through `compatibility()`
- raw values are not authoritative just because they fit in an enum

Reasonable nuance to preserve:

- struct aspects are not just "nested values"; they use their own
  `StructAspectValue` carrier and field-contract surface
- opaque aspects are a contract family, but they do not expose the same raw
  public value-construction surface as ordinary scalar-like carriers
- `Bytes(ContentRefId)` and `ContentRef(ContentRefId)` are not the same
  semantic lane even though they share an underlying id type
- masks are contract law, not presentation sugar; projection, mutation, and
  diagnostic visibility are intentionally separate categories
- patch legality depends on the admitted mutation-mask and contract law, not on
  caller-side field folklore

Do not use this family for:

- Query projection-consumption facts that Query already typed for you
- runtime-local hot state just because it looks aspect-shaped
- replacing domain semantics with generic aspect maps too early
- flattening the validated-state-admitted progression into one helper just to
  reduce ceremony
- hiding compatibility lowering inside a native authoring flow

Read next:

- [Grouped Public Lanes And Common-Path Usage](./aspect-contracts-values-and-authoritative-state/grouped-public-lanes-and-common-path-usage.md)
- [Aspect Shapes And Value Carriers](./aspect-contracts-values-and-authoritative-state/aspect-shapes-and-value-carriers.md)
- [Projection, Mutation, And Diagnostic Masks](./aspect-contracts-values-and-authoritative-state/projection-mutation-and-diagnostic-masks.md)
- [Aspect Keys, Values, And Scalar Contracts](./aspect-contracts-values-and-authoritative-state/aspect-keys-values-and-scalar-contracts.md)
- [Validation And Authoritative State Admission](./aspect-contracts-values-and-authoritative-state/validation-and-authoritative-state-admission.md)
- [Identities, Locators, And Blind-Consumer Addressing](./aspect-contracts-values-and-authoritative-state/identities-locators-and-blind-consumer-addressing.md)

## Compatibility And Explicit Bridge Debt

The compatibility lane exists for transitional input, especially JSON-shaped
input, without letting that transitional input masquerade as native authority.

Use this family when:

- an external boundary still sends JSON
- a migration surface still depends on compatibility lowering
- you need the output to lower into the same native meaning as the real aspect
  lane
- you need explicit typed transition posture instead of pretending the input is
  already native

Do not use it when:

- you are doing native authoring
- you control the boundary and can author native meaning directly
- you want a convenience shortcut around real validation and state admission

The public compatibility common path is intentionally narrow:

- `compatibility().json()`
- `.input(...)`
- `.lower_value(...)`
- `.lower_state(...)`

The lower-lane capability inventory is broader than that entry point:

- `JsonCompatibilityAspectInput`
- `lower_json_aspect_value(...)`
- `lower_json_record_aspect_state(...)`
- `JsonCompatibilityLoweringOutcome<_>`
- `JsonCompatibilityLoweringDenial`
- `JsonCompatibilityLoweringDeferred`
- `JsonCompatibilityLoweringStale`
- `JsonCompatibilityRebindRequired`
- `JsonCompatibilityLoweringFailure`

The important lifecycle is:

1. hold a real native `AspectContract`
2. bind the JSON input to a typed `BoundarySourceLocator`
3. lower one JSON value or a set of JSON-backed inputs through the
   compatibility lane
4. land on the same validated artifact or admitted authoritative state surface
   the native lane would have produced

This is not a parser helper. It is a typed bridge into native meaning.

Reasonable nuance to preserve:

- compatibility lowering still depends on the native contract shape; it does
  not invent meaning from JSON alone
- JSON state lowering still inherits native state-admission law, including
  duplicate-key denial and empty-admission denial
- parity with native state is a real tested expectation for equivalent inputs,
  not a vague design goal
- bytes-family and content-reference-family scalar lowerings remain distinct
  even when both lower from simple numeric JSON forms
- opaque shapes are explicitly not admitted through the JSON bridge in the
  current milestone
- struct lowering has its own denial surfaces for invalid field keys, unknown
  fields, and struct-construction failures
- numeric width ambiguity is a first-class typed denial; the bridge does not
  guess widths when JSON is underspecified
- recursive JSON documents are not a generic fallback lane for scalar aspects

The outcome lattice matters here. Compatibility lowering is not just
success/failure. It can produce:

- success
- denial
- deferred
- stale
- rebind required
- failure

AI consumers should preserve that topology instead of flattening everything to
one `Result` or one denial string.

This is explicit debt on purpose. The docs, readiness surface, and compile-fail
tests all reinforce that:

- raw `serde_json::Value` is supposed to enter through
  `JsonCompatibilityAspectInput`
- raw JSON should not enter native validation directly
- validated native artifacts should not be smuggled back into JSON state
  lowering

Your code should preserve those boundaries too.

Do not use this family for:

- silently routing native authoring through JSON because it feels easier
- teaching app code to start from JSON when native aspect authoring is
  available
- flattening typed compatibility outcomes into one â€œparse failedâ€ branch
- treating compatibility-lowered artifacts as if they erase the fact that a
  compatibility bridge was involved

This family also influences later surfaces indirectly. Compatibility posture is
tracked by later profile, boundary-artifact, canonicalization, and
boundary-evidence families. That means compatibility lowering is not just an
ingest concern; it can remain visible in later support and certification
surfaces.

Read next:

- [Grouped Public Lanes And Common-Path Usage](./aspect-contracts-values-and-authoritative-state/grouped-public-lanes-and-common-path-usage.md)
- [Compatibility Lowering And JSON Bridges](./aspect-contracts-values-and-authoritative-state/compatibility-lowering-and-json-bridges.md)
- [Validation And Authoritative State Admission](./aspect-contracts-values-and-authoritative-state/validation-and-authoritative-state-admission.md)

## Canonical Basis, Identity, And Digest Boundaries

This family is for canonical basis preparation, comparison, export bundles,
digest derivation, and trust-boundary identity discipline.

Use it when you need:

- reproducible canonical basis
- portable digest evidence
- structured mismatch classification
- explicit authority identity, bridged identity, projection identity, or
  external identity token handling
- export publication shape that can cross a boundary honestly
- readmission vocabulary after a trust boundary
- explicit producer shape and equivalence basis for published canonical outputs

This is the right family when the problem is:

- "what exactly is the canonical meaning of this boundary object?"
- "how do I compare two independently produced surfaces honestly?"
- "how do I keep a value typed as authority identity rather than just a string?"

The grouped public surface here is not one flat lane. It has distinct homes:

- `canonicalization().basis()`
- `canonicalization().compare()`
- `canonicalization().export()`
- `canonicalization().digest()`
- `canonicalization().readiness()`

and the grouped API families:

- `worth_foundational::canonicalization_api::common_path`
- `worth_foundational::canonicalization_api::lower_lane::basis`
- `worth_foundational::canonicalization_api::lower_lane::comparison`
- `worth_foundational::canonicalization_api::lower_lane::export`
- `worth_foundational::canonicalization_api::lower_lane::digest`
- `worth_foundational::canonicalization_api::stronger_lane`
- `worth_foundational::canonicalization_api::stronger_lane::readiness`

The capability inventory is broader than "digests and ids":

- canonical basis sequence and bundle preparation
- canonical comparison with explicit equivalence basis
- structured mismatch and unsupported-comparison outcomes
- export bundle admission with explicit producer shape and export naming
- trust-boundary bridging and explicit readmission of exports
- digest derivation from ready canonical artifacts under explicit algorithm
  slots
- authority identity admission, projection, bridging, digest evidence, and
  readmission

Reasonable nuance to preserve:

- canonical basis is the primary meaning surface; digest is derived compression
  downstream of admitted canonical meaning
- digest output is not semantic authority and is not a substitute for canonical
  comparison
- export is a publication lane, not a source-of-truth construction lane
- producer shape and export manifest mismatches are intentionally distinct from
  semantic canonical mismatches
- comparison outcomes are at least three-way: equivalent, mismatched, and
  unsupported
- trust-boundary bridging does not preserve current authority automatically;
  readmission is explicit
- authority identity is not just a newtype around a value; it includes who
  admitted it and which identity kind it belongs to
- projection labels and digest evidence are useful outputs, but they are not
  authority
- external identity tokens remain external until an owner admits them with an
  authority witness

The identity half of this family is especially easy to underestimate. It
includes distinct categories for:

- current authoritative identity
- admitted-but-not-promoted identity
- boundary-bridged identity
- revalidated identity ready for readmission
- projection identity for logs or display
- digest identity evidence
- external identity tokens before admission

That means AI consumers should not collapse â€œidentityâ€ into one bucket just
because all of the payloads might be strings, integers, or digests underneath.

Do not use it for:

- runtime-local ids that never cross a meaningful boundary
- using digest bytes as if they were authority
- replacing Query-owned identity artifacts just because Foundational identity
  wrappers exist
- comparing digests as a substitute for canonical comparison
- using export bundles as a shortcut around ready canonical basis preparation
- silently treating bridged exports or bridged identities as current authority
- hiding equivalence-basis choice or authority-witness spending in ambient
  helpers

For operational compact identities, use
`canonicalization().digest().for_sequence(...,
CanonicalDigestAlgorithmId::sha256())` and then
`canonicalization().digest().derive(...)`. SHA-256 is the only admitted digest
algorithm; unsupported identifiers deny before derivation. A runtime must not
call a hashing library directly after preparing canonical material.

If Query or another owning crate already gives you a stronger receipt or
identity-bearing artifact, keep that stronger type until you actually need a
shared boundary form.

Read next:

- [Grouped Public Lanes And Front-Door Usage](./canonical-basis-and-reproducible-identity/grouped-public-lanes-and-front-door-usage.md)
- [Equivalence And Mismatch Classification](./canonical-basis-and-reproducible-identity/equivalence-and-mismatch-classification.md)
- [Export Bundles And Producer Shape](./canonical-basis-and-reproducible-identity/export-bundles-and-producer-shape.md)
- [Canonical Basis And Reproducible Identity](./canonical-basis-and-reproducible-identity/README.md)
- [Authority Identity Boundaries](./canonical-basis-and-reproducible-identity/authority-identity-boundaries.md)
- [Digest Derivation And Slot Semantics](./canonical-basis-and-reproducible-identity/digest-derivation-and-slot-semantics.md)

## Profiles, Policy, And Surface Posture

Profiles are how Foundational describes richness, support posture,
compatibility posture, admission posture, certification posture, and related
surface policy without burying those choices in local flags and comments.

Use this family when you need:

- one shared profile vocabulary
- requested, admitted, and materialized posture progression
- target-aware profile attachment
- planned descriptive elision
- proof-bearing certification or readmission of profile-bearing artifacts
- stable identity, difference, and compatibility classification for admitted
  profile meaning
- a grouped public lane for staged profile work instead of ad hoc profile bags

This family is much broader than â€œpick some profile enums.â€ It includes:

- profile-family composition into one coherent profile set
- staged progression from requested -> admitted -> materialized meaning
- explicit narrowing records when meaning is reduced
- target-aware attachment to boundary, support, or proof-bearing artifacts
- descriptive surface materialization planning and elision
- admitted profile identity, difference, and compatibility comparison
- stronger proof-bearing certification and trust-boundary readmission
- grouped stronger readiness closure

The grouped public lane structure matters here:

- `worth_foundational::profiles_api::common_path`
- `worth_foundational::profiles_api::lower_lane::composition`
- `worth_foundational::profiles_api::lower_lane::progression`
- `worth_foundational::profiles_api::lower_lane::attachment`
- `worth_foundational::profiles_api::lower_lane::materialization`
- `worth_foundational::profiles_api::lower_lane::identity`
- `worth_foundational::profiles_api::lower_lane::certification`
- `worth_foundational::profiles_api::stronger_lane`
- `worth_foundational::profiles_api::stronger_lane::readiness`

This is especially useful when you want the same posture language across
multiple crates, but it is not only about shared labels. It is a typed policy
and lifecycle subsystem.

Reasonable nuance to preserve:

- a composed profile set is one complete policy object, not a mutable settings
  bag and not a partial draft
- every profile family must be assigned explicitly; missing or duplicate family
  assignments fail closed
- production-certified or evidence-backed claims are mechanically constrained
  by support, readiness, retention, and certification posture; those strengths
  are not just vibes
- progression is real staged meaning, not in-place mutation; requested,
  admitted, and materialized artifacts are distinct phases
- narrowing is explicit and typed; if meaning changes across progression, it
  must narrow in a controlled way and may only change one family per step
- target kind matters: boundary, support, and proof-bearing artifacts do not
  admit the same profile claims
- materialization planning is about descriptive surface availability and
  absence causes, not about rendering or UI
- profile identity and compatibility are based on admitted meaning, not on
  construction history or casual field comparison
- certification is a stronger lane for proof-bearing profiled artifacts, and
  trust-boundary crossing drops current proof until explicit readmission

The actual profile families matter enough to name directly:

- `DiagnosticRichnessProfile`
- `SupportPostureProfile`
- `CompatibilityPostureProfile`
- `AdmissionReadinessProfile`
- `RetentionDeliveryProfile`
- `CertificationPostureProfile`

AI consumers should preserve those families as semantic categories, not treat
them as random knobs that can be collapsed into one vague â€œmode.â€

Do not use it as:

- a replacement for runtime behavior policy when the runtime still owns the
  semantics
- a random bag of enums for local UI toggles or feature switches
- a way to hide narrowing or attachment legality
- a mutable effective-config object that silently drifts over time
- a substitute for proof-bearing certification when the stronger lane is what
  the boundary actually needs
- a place to guess which descriptive surfaces are available instead of asking
  the materialization planner

If you only need one crate-local config object and no cross-crate policy
meaning, target legality, narrowing, identity, or certification semantics, this
family is probably too strong for the job.

Read next:

- [Profile Families And Composed Profile Sets](./profiles-and-policy-vocabulary/profile-families-and-composed-profile-sets.md)
- [Requested, Admitted, And Materialized Profile Progression](./profiles-and-policy-vocabulary/requested-admitted-and-materialized-profile-progression.md)
- [Target-Aware Profile Attachment](./profiles-and-policy-vocabulary/target-aware-profile-attachment.md)
- [Descriptive Surface Materialization And Elision](./profiles-and-policy-vocabulary/descriptive-surface-materialization-and-elision.md)
- [Profile Identity, Difference, And Canonical Basis](./profiles-and-policy-vocabulary/profile-identity-difference-and-canonical-basis.md)
- [Proof-Bearing Profile Certification And Readmission](./profiles-and-policy-vocabulary/proof-bearing-profile-certification-and-readmission.md)
- [Profiles And Policy Vocabulary](./profiles-and-policy-vocabulary/README.md)

## Boundary Artifacts And Materialization Contracts

This family gives WORTH one shared language for:

- `Summary`
- `Report`
- `Artifact`
- `Receipt`

and for the materialization plans and bundles that produce those surfaces.

Use this family when the first question is:

- what kind of boundary output is this?
- is this authoritative, derived, support-only, planned, or completed?
- am I producing a report, a payload-shaped artifact, or a receipt?
- how is this boundary output being materialized, bundled, canonicalized, or
  strengthened?

This is the right lane when you want shared boundary nouns that do not drift
from crate to crate.

This family is broader than the four category names. It includes:

- boundary categories
- boundary roles and authority admission
- materialization planning and materialized outputs
- typed multi-output bundles
- descriptive planned-work and same-family wrappers
- canonical basis preparation for materialized boundary outputs
- current-basis strengthening, trust-boundary bridging, and readmission

The category and role split is one of the most important pieces:

- category says what kind of boundary output this is
- role says what kind of claim that boundary output is making

Those are not interchangeable, and not every category-role combination is
legal.

The shipped role families are explicit:

- `AuthoritativeCurrent`
- `DerivedProjection`
- `SupportOnly`
- `PlannedWork`
- `ReceiptEvidence`

The materialization subsystem is also much richer than â€œturn object into
output.â€ It makes these things explicit:

- materialization source
- materialization seam
- delivery class
- availability posture
- attachment inclusion and elision
- decision rows
- materialization cost
- typed bundle membership

Reasonable nuance to preserve:

- `Summary`, `Report`, `Artifact`, and `Receipt` are different output
  categories with different construction rules; they are not one envelope plus
  a tag
- `AuthoritativeCurrent` is a stronger proof-bearing claim than descriptive
  roles and has its own explicit admission lane
- materialization is a boundary crossing, not a cheap getter or view
- bundles are typed coordinated outputs with one primary artifact plus legal
  optional members; they are not arbitrary result bags
- delivery and availability are part of the contract, so â€œdeferred,â€
  â€œreconstructable,â€ and â€œunavailableâ€ are first-class postures rather than
  prose notes
- planned-work and same-family wrappers are intentionally descriptive only;
  they exist to preserve meaning without smuggling branch, merge, or commit
  authority-transition semantics in early
- canonical basis and current-basis are different promises: canonical basis is
  semantic comparability, current-basis is a stronger live-basis claim with
  explicit trust-boundary weakening and readmission
- current-basis strengthening reuses canonicalization and `worth-proof`; it is
  not a second private proof system

There is also an important milestone boundary here:

- Milestone 4 gives descriptive room for planned-work and same-family outputs
- reserved branch / merge / commit authority-transition claims fail closed here
- real branch / merge / commit transition ontology belongs to the later
  transition family, not to these descriptive wrappers

Do not use it to:

- flatten every boundary object into one generic envelope
- fake execution truth with a plan-shaped surface
- upgrade descriptive outputs into authority by convention
- hide category-role illegality behind convenience builders
- treat materialization plans as free accessors
- use planned-work or same-family wrappers as disguised transition engines
- preserve current-basis strength implicitly across trust boundaries

Read next:

- [Boundary Categories](./boundary-artifact-taxonomy-and-materialization-contracts/boundary-categories.md)
- [Boundary Roles And Authority Admission](./boundary-artifact-taxonomy-and-materialization-contracts/boundary-roles-and-authority-admission.md)
- [Boundary Materialization And Bundles](./boundary-artifact-taxonomy-and-materialization-contracts/boundary-materialization-and-bundles.md)
- [Planned Work, Same-Family Outputs, And Reserved Authority Transitions](./boundary-artifact-taxonomy-and-materialization-contracts/planned-work-same-family-and-reserved-authority-transitions.md)
- [Boundary Canonical Basis And Current-Basis](./boundary-artifact-taxonomy-and-materialization-contracts/boundary-canonical-basis-and-current-basis.md)
- [Boundary Artifact Taxonomy And Materialization Contracts](./boundary-artifact-taxonomy-and-materialization-contracts/README.md)

## Diagnostics And Explanation Ontology

This family gives WORTH one shared language for diagnostic primitives,
categories, outcomes, subjects, rows, materialized reports, comparisons, and
certified diagnostic bundles.

Use it when you need:

- structured diagnostic rows instead of strings
- support reports that keep missing or degraded evidence visible
- comparison-ready diagnostic bundles
- shared explanation vocabulary across crates
- canonical parity for diagnostic meaning across producers
- stronger certified coverage and current-basis attachment for diagnostics
- typed partiality, named gaps, fallback debt, and evidence posture

This is the right family when the problem is "how do I explain this outcome
honestly?" rather than "how do I execute it?"

This family is broader than "structured errors." It includes:

- primitive diagnostic vocabulary such as code, scope, severity, denial
  class, breach class, evidence posture, artifact kind, delivery class, and
  availability
- typed diagnostic subjects and semantic locators
- family-distinct row models for decision, failure, comparison, support, and
  provenance-ready rows
- materialization planning and materialized support/explanation surfaces
- partiality, named gaps, gap classes, gap targets, and closure postures
- support-claim strength, counter snapshots, and assembly debt
- canonical basis preparation and semantic comparison for diagnostic bundles
- stronger certified diagnostic bundles with coverage matrices, source
  digests, and trust-boundary readmission

Reasonable nuance to preserve:

- diagnostics are not one generic row bag; row family is semantic truth
- denial class and breach class are intentionally different because a policy
  rejection is not the same thing as an integrity or construction failure
- provenance-ready rows are distinct from ordinary explanation rows; they
  carry evidence-origin meaning without pretending to be receipts or
  authority
- support reports and explanation bundles are planned and materialized
  explicitly, with visible delivery, availability, richness, partiality, and
  fallback-debt decisions
- partial bundles are still honest bundles when named gaps are preserved
  explicitly; missing coverage should not be hidden in prose
- canonical diagnostic comparison compares meaning, not presentation order or
  debug strings
- certified diagnostic bundles are stronger diagnostic claims, not promoted
  transition authority or receipts
- certified coverage is typed and hostile-minded; "happy path plus a badge"
  is not enough

The primitive and row inventory is important enough to name directly.

Primitive families include:

- diagnostic code
- diagnostic scope
- severity
- denial class
- breach class
- evidence posture
- artifact kind
- delivery class
- availability

Row families include:

- `FoundationalDiagnosticDecisionRow`
- `FoundationalDiagnosticFailureRow`
- `FoundationalDiagnosticComparisonRow`
- `FoundationalDiagnosticSupportRow`
- `FoundationalDiagnosticProvenanceReadyRow`

Materialized diagnostic surfaces include:

- `FoundationalDiagnosticSupportReport`
- `FoundationalDiagnosticExplanationBundle`
- `FoundationalDiagnosticComparisonBundle`

Certified stronger-lane surfaces include:

- `FoundationalCertifiedDiagnosticBundle<Source, Bundle>`
- `FoundationalDiagnosticCoverageMatrix`
- `FoundationalDiagnosticCertifiedCoverageClass`
- trust-boundary bridge and readmission helpers

AI consumers should preserve all of those distinctions instead of collapsing
them into "error report," "support report," or "comparison result."

Do not use it as:

- a generic replacement for ordinary error types inside hot local code
- a way to overclaim certainty when evidence is missing or degraded
- a casual string-formatting layer
- a public row union with optional fields standing in for real row families
- a substitute for receipts, authority artifacts, or proof-bearing
  transitions
- a place to hide missing support coverage instead of naming gaps explicitly

Read next:

- [Diagnostics And Explanation Ontology](./diagnostics-and-explanation-ontology/README.md)
- [Diagnostic Primitives And Categories](./diagnostics-and-explanation-ontology/diagnostic-primitives-and-categories.md)
- [Diagnostic Outcomes, Subjects, And Rows](./diagnostics-and-explanation-ontology/diagnostic-outcomes-subjects-and-rows.md)
- [Diagnostic Materialization And Support Reports](./diagnostics-and-explanation-ontology/diagnostic-materialization-and-support-reports.md)
- [Diagnostic Canonical Basis And Comparison](./diagnostics-and-explanation-ontology/diagnostic-canonical-basis-and-comparison.md)
- [Certified Diagnostic Bundles And Attachments](./diagnostics-and-explanation-ontology/certified-diagnostic-bundles-and-attachments.md)

## Lineage, Provenance, Receipts, And Support Truth

This family is where Foundational speaks about:

- provenance layering
- freshness posture
- planning vs executed vs completed receipts
- continuity, divergence, promotion, and replay
- support-grade truth and degraded operation
- attachment materialization and readmission

Use it when you need:

- a typed executed/planned/completed distinction
- shared provenance and support meaning
- support-grade truth that stays honest about degraded posture
- boundary evidence that can cross crates cleanly
- continuity claims that distinguish attested, replay-derived, restored,
  reconstructed, promoted, partial, and divergent outcomes
- a grouped public lane for boundary evidence instead of crate-local evidence
  folklore

This is one of the most important boundaries in the crate.

This family is broader than â€œreceipts and provenance.â€ It includes:

- primitive evidence categories, locality postures, execution postures,
  descriptive-role postures, and freshness postures
- provenance layering over source basis, authority path, profile basis,
  comparison basis, and canonical/digest basis
- typed planning, executed, and completed receipt families
- lineage families for attested continuity, branch-local replacement,
  promotion, replay-derived continuity, restored continuity, reconstructed
  equivalence, and partial lineage
- support-truth families for published support, degraded recovery, transient
  lifecycle evidence, basis disclosure, and residual debt
- attachment materialization that lets provenance, lineage, receipts, support,
  and diagnostics travel together on real targets
- stronger current-basis and support-basis readmission lanes for attached
  evidence bundles

The grouped public lane structure is part of the real feature surface:

- `worth_foundational::boundary_evidence_api::common_path`
- `worth_foundational::boundary_evidence_api::lower_lane::primitives`
- `worth_foundational::boundary_evidence_api::lower_lane::provenance`
- `worth_foundational::boundary_evidence_api::lower_lane::receipts`
- `worth_foundational::boundary_evidence_api::lower_lane::lineage`
- `worth_foundational::boundary_evidence_api::lower_lane::support`
- `worth_foundational::boundary_evidence_api::lower_lane::attachments`
- `worth_foundational::boundary_evidence_api::stronger_lane::readmission`
- `worth_foundational::boundary_evidence_api::stronger_lane::readiness`

Reasonable nuance to preserve:

- category, locality, execution posture, descriptive role, and freshness are
  separate axes; they are not one catch-all â€œevidence statusâ€
- provenance explains basis and freshness, but does not prove execution or
  continuity by itself
- receipts answer â€œwhat completed boundary truth do I have?â€ and still keep
  planned, executed, and completed-closeout families distinct
- completed closeout is not automatically successful execution; blocked or
  denied closeout can still produce real receipt artifacts
- lineage is stronger than provenance and different from receipts; replay-
  derived, restored, reconstructed, branch-local, and promoted continuity are
  intentionally different claims
- support truth can be very important while still being weaker than current-
  basis or proof-bearing authority lanes
- transient lifecycle support evidence is first-class and should not be faked
  as durable lineage
- attachment is where these descriptive families finally travel together on a
  target, and target kind plus continuity scope still matter there
- current-basis readmission and support-basis readmission are different stronger
  lanes; they should not be collapsed
- canonical or digest participation can preserve descriptive meaning, but it
  does not strengthen current-basis authority by itself

The receipt and support families are important enough to name directly.

Receipt families include:

- `FoundationalBoundaryEvidencePlanningReceiptArtifact`
- `FoundationalBoundaryEvidenceExecutedReceiptArtifact`
- `FoundationalBoundaryEvidenceCompletedReceiptArtifact`

Support families include:

- published support
- degraded recovery reports
- transient lifecycle evidence
- basis disclosure
- residual debt

Attachment bundle families include:

- `FoundationalMaterializedBoundaryEvidenceAttachmentBundle`
- `CurrentBasisBoundaryEvidenceAttachmentBundle`
- `SupportBasisBoundaryEvidenceAttachmentBundle`

Do not use it to:

- pretend a planning receipt means executed work
- replace a stronger runtime-local receipt while its owning semantics are still
  local
- smuggle support-only truth into authoritative truth
- flatten replay-derived or reconstructed continuity into attested continuity
- treat support-basis readmission as interchangeable with current-basis
  readmission
- treat raw attachment bundles as if they had already crossed the stronger
  readmission boundary
- hide stale, reduced, replayed, or rebuilt posture behind generic â€œavailableâ€
  wording

If the domain crate still owns the true execution semantics, keep the domain
receipt until you actually need a shared support-grade or boundary-evidence
surface.

Read next:

- [Grouped Public Lanes And Stronger Readiness](./lineage-provenance-receipts-and-support-truth/grouped-public-lanes-and-stronger-readiness.md)
- [Primitive Categories, Locality, And Role Postures](./lineage-provenance-receipts-and-support-truth/primitive-categories-locality-and-role-postures.md)
- [Provenance Layering And Freshness](./lineage-provenance-receipts-and-support-truth/provenance-layering-and-freshness.md)
- [Receipts And Closeout Truth](./lineage-provenance-receipts-and-support-truth/receipts-and-closeout-truth.md)
- [Lineage, Continuity, Divergence, And Promotion](./lineage-provenance-receipts-and-support-truth/lineage-continuity-divergence-and-promotion.md)
- [Support Truth, Recovery, And Degraded Operation](./lineage-provenance-receipts-and-support-truth/support-truth-recovery-and-degraded-operation.md)
- [Attachment Materialization, Canonical Participation, And Readmission](./lineage-provenance-receipts-and-support-truth/attachment-materialization-canonical-participation-and-readmission.md)
- [Lineage, Provenance, Receipts, And Support Truth](./lineage-provenance-receipts-and-support-truth/README.md)

## Performance, Layout, And Enforcement Vocabulary

This family gives WORTH one shared way to talk about performance meaning across
authoring, lowering, execution evidence, report widening, stronger proof, and
readiness closure.

Use it when you need:

- a boundary-safe performance claim that names boundary, evidence strength,
  included work, excluded work, temperature, freshness, and fallback debt
- explicit layout intent vocabulary that says what access or allocation posture
  a path assumes without pretending that layout intent alone proves cost
- policy-admission receipts that record pre-execution budget outcomes honestly
- canonical bundle lowering and comparison across independent producers
- counter-backed receipts that say execution really happened and attach exact
  structural rows
- explicit report planning and materialization boundaries for broader support
  output
- proof-bearing certified and readmitted performance bundles when a stronger
  trust-boundary claim is real
- a public lane inventory and readiness closure artifact that freezes what the
  shipped surface really supports

The public lane map is real and should stay visible:

- `worth_foundational::performance_api::common_path`
- `worth_foundational::performance_api::lower_lane`
- `worth_foundational::performance_api::stronger_lane`
- `worth_foundational::performance_api::performance_public_surface_inventory()`

The actual capability inventory is much larger than "claims and reports":

- descriptive claim authoring through
  `worth_foundational::performance_api::common_path::performance()`
- `FoundationalPerformanceClaimAuthoringFrontDoor`
- `FoundationalLayoutIntentClaim`
- policy-admission lowering through
  `worth_foundational::performance_api::lower_lane::policy`
- `FoundationalPolicyAdmissionReceipt`
- `foundational_performance_budget_definitions()`
- canonical bundle lowering and comparison through
  `worth_foundational::performance_api::lower_lane::basis`
- contract names, counter specs, supporting evidence rows, canonical-basis
  preparation, and digest-ready comparison helpers
- executed lower-lane receipts through
  `worth_foundational::performance_api::lower_lane::receipts`
- `FoundationalCounterBackedPerformanceReceipt`
- `FoundationalPerformanceCounterRow`
- planned and materialized reporting through
  `worth_foundational::performance_api::lower_lane::reports`
- `FoundationalPerformanceReportRequest`
- `FoundationalPerformanceReportPlan`
- `FoundationalMaterializedPerformanceReport`
- proof-bearing strengthening through
  `worth_foundational::performance_api::stronger_lane::certified`
- trust-boundary bridging and readmission authorities for certified bundles
- stronger readiness closure through
  `worth_foundational::performance_api::stronger_lane::readiness`

The important strength ladder is the heart of this family:

- a common-path claim is descriptive and legality-checked, but it is still only
  a claim
- a policy-admission receipt is stronger because a runtime budget decision
  really happened, but execution still has not
- a canonical bundle is the shared lower-lane envelope for comparison and
  attachments, not executed proof
- a counter-backed receipt is the first artifact that can honestly say
  execution happened
- a materialized report is a deliberate widening of a lower-lane source, not a
  free accessor
- a certified or readmitted bundle is stronger still, because it carries
  proof-bearing trust-boundary meaning
- readiness is its own stronger closure seam, not a property you get for free
  from every certified artifact

The layout part of this family is easy to underestimate, so be explicit:

- layout intent explains representation family, access posture, and allocation
  posture
- layout intent does not force one storage topology on every crate
- layout intent does not prove equal cost across different representations
- layout intent is part of the claim story, not a substitute for counter-backed
  evidence

The enforcement vocabulary is also a real feature, not just naming polish:

- common-path builders fail closed on contradictory claim shapes
- policy receipts fail closed when widened, denied, deferred, debt, and
  verified stories are mixed dishonestly
- counter-backed receipts fail closed on missing, duplicated, unexpected, or
  mismatched rows
- report planning makes widening visible through explicit materialization
  boundaries such as `ClaimInspectionOnly`, `ReportAssembly`, and
  `SupportExpansion`
- grouped lane APIs and stronger-lane entrypoints are intentionally narrow so
  lower-lane artifacts cannot silently masquerade as stronger proof

Reasonable nuance to preserve:

- this family is about shared performance meaning, not about owning the runtime
  measurement engine or allocator
- elapsed time alone is not the shared meaning; contract names, counter specs,
  row sets, included work, and excluded work matter
- supporting evidence rows on a canonical bundle are not the same thing as
  executed counter rows
- report materialization is intentionally more expensive and wider than direct
  receipt inspection
- hot operational paths and support-expansion paths are meant to stay
  mechanically distinct
- certified support-expansion evidence is not the same claim as current-basis
  hot-path operational truth

Do not use it as:

- a runtime allocator
- a measurement engine by itself
- a shortcut around producing exact execution rows
- a way to claim hot-path proof when you only have policy admission
- a generic report getter that silently widens support work
- proof that a local optimization exists when you have not actually produced
  the supporting evidence

Read next:

- [Performance, Layout, And Enforcement Vocabulary](./performance/README.md)
- [Common Performance Claims And Layout Intent](./performance/common-performance-claims-and-layout-intent.md)
- [Policy Admission Receipts](./performance/policy-admission-receipts.md)
- [Canonical Bundles And Comparison](./performance/canonical-bundles-and-comparison.md)
- [Counter-Backed Performance Receipts](./performance/counter-backed-performance-receipts.md)
- [Performance Report Planning And Materialization](./performance/performance-report-planning-and-materialization.md)
- [Certified And Readmitted Performance Bundles](./performance/certified-and-readmitted-performance-bundles.md)
- [Grouped Public Lanes And Stronger Readiness](./performance/grouped-public-lanes-and-stronger-readiness.md)

## Transition Vocabulary

This family gives WORTH a shared language for transition-like work across
branch-local staging, merge planning, scoped merge/cherry-pick boundary
requests, proof-bearing authority crossing, receipt issuance, canonical
comparison, current-basis strengthening, trust-boundary bridging, and
readiness closure.

Use it when multiple crates need to agree about:

- exact branch-reference observations shared by Relational and Signal without
  moving owner currentness or authority into Foundational
- branch-local candidate meaning without accidentally claiming authority
- staged branch work that is ready for merge planning
- merge planning and typed merge verdict topology
- strategy identity, strategy basis, merge basis, remap basis, and branch-basis
  drift
- scoped merge or cherry-pick request vocabulary before runtime execution
- proof-bearing committed authority transitions
- commit receipts, discard receipts, provenance rows, and coordinated
  transition bundles
- canonical basis preparation, transition locators, current-basis admission,
  and trust-boundary readmission
- milestone closure truth for transition surfaces through readiness artifacts

The canonical exact-reference guide is
[Exact Branch References](./branching-merging-and-commit-vocabulary/branch-references.md).
Its target/identity/generation grammar is descriptive and portable. It neither
reuses nor upgrades Milestone 5's epoch/equivalence candidate bases, and it
never moves component currentness, retention, or authority into Foundational.

The actual capability inventory is much larger than "branch, merge, commit":

- branch-local authoring through `foundational_branch_candidate()`
- `FoundationalBranchCandidateBuilder`
- `FoundationalBranchCandidateArtifact<T>`
- `FoundationalStagedBranchArtifact<T>`
- `FoundationalBranchId`
- `FoundationalBranchCandidateId`
- `FoundationalBranchCandidateForkBasis`
- `FoundationalBranchCandidateObservationBasis`
- `FoundationalBranchCandidateForkObservationBasis`
- `FoundationalBranchCandidateComparisonBasis`
- merge planning through `foundational_merge(...)`
- `FoundationalMergeBuilder<T>`
- `FoundationalMergeCandidate<T>`
- `FoundationalMergeVerdict<T>`
- `FoundationalMergeVerdictKind`
- `FoundationalMergeStructuralSummary`
- `FoundationalMergeConflictLocus`
- typed non-success topology through `worth_proof::TransitionOutcome`
- strategy and basis semantics through `FoundationalTransitionStrategyIdentity`,
  `FoundationalTransitionStrategyDescriptorDigest`,
  `FoundationalTransitionStrategyContractBasis`,
  `FoundationalStrategyBasis`, `FoundationalMergeBasis`,
  `FoundationalMergeBaseSelectionBasis`,
  `FoundationalTransitionCorrespondenceBasis`,
  `FoundationalTransitionRemapBasis`, and `FoundationalBranchBasisDrift`
- proof-bearing authority crossing through
  `foundational_committed_authority_admission()`,
  `FoundationalCommittedAuthorityInput`, and
  `FoundationalCommittedAuthorityArtifact<T>`
- committed transition classes and no-op causes through
  `FoundationalAuthorityTransitionClass`,
  `FoundationalAuthorityTransitionOutcomeKind`, and
  `FoundationalNoOpCause`
- receipt and bundle emission through
  `foundational_commit_receipt_issuance()`,
  `FoundationalCommitReceiptArtifact`, `FoundationalCommitReceiptIdentity`,
  `FoundationalCommitId`, `FoundationalBranchDiscardReceipt`,
  `FoundationalTransitionProvenanceRow`,
  `FoundationalTransitionBundleBuilder<T>`, and
  `FoundationalTransitionBundle<T>`
- canonical preparation, locators, and stronger current-basis behavior through
  `prepare_branch_candidate_for_canonical_basis(...)`,
  `prepare_staged_branch_for_canonical_basis(...)`,
  `prepare_merge_verdict_for_canonical_basis(...)`,
  `prepare_committed_authority_for_canonical_basis(...)`,
  `prepare_commit_receipt_for_canonical_basis(...)`,
  `foundational_transition_canonical_basis_entries(...)`,
  `FoundationalTransitionLocator`, `FoundationalBranchCandidateLocator`,
  `FoundationalMergeConflictLocator`,
  `FoundationalCommitParentageLocator`, and
  `FoundationalCommittedDeltaLocator`
- current-basis admission and trust-boundary readmission through
  `admit_current_basis_committed_authority(...)`,
  `admit_current_basis_commit_receipt(...)`,
  `bridge_current_basis_committed_authority_trust_boundary(...)`,
  `bridge_current_basis_commit_receipt_trust_boundary(...)`,
  `readmit_current_basis_committed_authority_after_boundary(...)`, and
  `readmit_current_basis_commit_receipt_after_boundary(...)`
- scoped merge and cherry-pick boundary vocabulary through
  `FoundationalMergeScope::full_branch()`,
  `FoundationalMergeScope::selected_nodes(...)`,
  `FoundationalMergeScope::selected_aspects(...)`,
  `FoundationalSelectedNodeLocus`, `FoundationalSelectedAspectLocus`,
  `FoundationalSelectedAspectRequestEntry`,
  `FoundationalAdmittedMergeScopeEvidence`,
  `FoundationalSelectedScopeNoOpEvidence`,
  `FoundationalSkippedOutOfScopeEvidence`,
  `FoundationalScopedMergeDenialEvidence`,
  `FoundationalScopedMergeUnavailablePosture`, and scoped canonical/diagnostic
  preparation helpers
- readiness closure through
  `foundational_transition_milestone5_readiness_report()`,
  `certify_foundational_transition_milestone5_production_test_readiness()`,
  `require_foundational_transition_milestone5_production_test_readiness(...)`,
  `foundational_transition_milestone9_scoped_merge_readiness_report()`, and
  the matching scoped-merge certification helpers

The authority ladder is the heart of this family:

- branch-local candidate and staged branch surfaces are descriptive only
- merge candidates and merge verdicts are still non-authoritative, even when
  they are fully typed and admitted
- scoped merge request, admitted-scope, denial, unavailable, no-op, and
  skipped evidence stay boundary vocabulary until an adopting runtime executes
  them
- committed authority is the first proof-bearing authority lane
- commit receipts and transition bundles derive from committed authority; they
  do not create a second independent authority lane
- canonical basis and current-basis strengthening make already-honest
  transition artifacts reproducible and stronger; they do not redefine weak
  artifacts into authority
- readiness is a named closure artifact for a milestone scope, not a vague
  quality statement

The basis and strategy part is also easy to underestimate:

- strategy identity and basis are separate axes
- correspondence basis and remap basis are not optional trivia when they shaped
  the result
- branch-basis drift is explicit vocabulary, not ambient "stale merge"
  folklore
- replay, promotion, metadata-only, no-op, and ordinary commit meanings remain
  distinct transition classes

Reasonable nuance to preserve:

- this family is not a generic version-control library; it standardizes shared
  transition boundary meaning
- it depends on `worth-proof` for progression topology instead of inventing a
  second proof kernel
- discard receipts are explicit non-authoritative closeout evidence, not weak
  commits
- report-only or summary-only transition bundles must not fake receipt
  attestation fields
- canonical basis preparation and current-basis admission solve different
  problems
- scoped merge vocabulary names request, admission, denial, and unavailable
  posture before execution; adopting runtimes still own branch mutation,
  cherry-pick materialization, and conflict resolution
- current-basis strength does not silently survive trust boundaries; bridging
  and readmission stay explicit

Do not use it as:

- a generic version-control layer for unrelated domain problems
- a substitute for `worth-proof` progression law
- a way to turn descriptive branch-local or merge-local work into authority too
  early
- a place to hide strategy, basis, or drift meaning in payload metadata
- a shortcut that lets report-only bundles pretend they issued a receipt
- a runtime merge executor, branch mutator, cherry-pick engine, or conflict
  resolver

Read next:

- [Branching, Merging, And Commit Vocabulary](./branching-merging-and-commit-vocabulary/README.md)
- [Branch-Local Candidates And Staged Branches](./branching-merging-and-commit-vocabulary/branch-local-candidates-and-staged-branches.md)
- [Merge Planning And Verdicts](./branching-merging-and-commit-vocabulary/merge-planning-and-verdicts.md)
- [Transition Strategy And Basis Semantics](./branching-merging-and-commit-vocabulary/transition-strategy-and-basis-semantics.md)
- [Committed Authority Transitions](./branching-merging-and-commit-vocabulary/committed-authority-transitions.md)
- [Commit Receipts, Discard, And Transition Bundles](./branching-merging-and-commit-vocabulary/commit-receipts-discard-and-transition-bundles.md)
- [Transition Canonical Basis, Locators, And Current-Basis](./branching-merging-and-commit-vocabulary/transition-canonical-basis-locators-and-current-basis.md)
- [Scoped Merge And Cherry-Pick Vocabulary](./scoped-merge-adoption.md)
- [Transition Production Readiness](./branching-merging-and-commit-vocabulary/transition-production-readiness.md)

## Decision Rules

Use these rules when the right crate still feels ambiguous.

If you need typed facts from Query materialization:

- use Query projection consumption
- do not lower immediately into Foundational aspect state or generic boundary
  artifacts

If you need one portable identity that must remain admitted, bridged,
projected, or readmitted across a trust boundary:

- use Foundational identity boundaries

If you need to compare or export a boundary object across producers:

- use Foundational canonical basis and digest surfaces

If you need one shared explanation, support report, or portable receipt story:

- use Foundational diagnostics, boundary artifacts, or boundary evidence

If you need checked progression law, witnesses, stage distinctions, or
proof-bearing transition topology:

- use `worth-proof`

If you need hot execution, runtime scheduling, domain truth mutation, or
runtime-local receipts:

- stay in the owning runtime or domain crate

## Hard Prohibitions

Do not do these things:

- do not replace `worth-query` typed fact receipts with Foundational types while
  the meaning is still Query-owned
- do not replace Query authority-lane state with Foundational wrappers out of
  convenience
- do not use Foundational compatibility lowering for native authoring
- do not use Foundational digest evidence as if it were authority identity
- do not use Foundational boundary artifacts as a generic envelope escape hatch
- do not use Foundational performance vocabulary as a substitute for real
  measurement or execution evidence
- do not move progression law into Foundational when `worth-proof` already owns
  it
- do not force hot internal runtime state into Foundational shapes unless the
  boundary itself truly needs the shared vocabulary
- do not flatten stronger domain semantics into weaker shared vocabulary just to
  make types look uniform

## AI Checklist Before Editing Code

Before introducing a Foundational type, ask these questions:

1. Is there a stronger owning runtime or domain type already available?
2. Am I still inside Query-owned or domain-owned semantics?
3. Am I crossing a trust boundary, export boundary, or support boundary?
4. Do I need canonical comparison, digest, or readmission?
5. Do I need shared profile, provenance, diagnostic, or receipt vocabulary?
6. Is the real problem progression law instead?
7. Am I about to replace a stronger artifact with a weaker shared boundary type
   for convenience?

If the answer to question 1 is yes, stop and justify the lowering.

If the answer to question 7 is yes, you are probably about to make the design
worse.

## When In Doubt

Start from the strongest owning crate.

- Start from `worth-query` when the semantics are Query-owned.
- Start from the domain runtime when the semantics are runtime-local.
- Start from `worth-proof` when the semantics are progression law.
- Start from `worth-foundational` when the semantics are shared boundary
  meaning.

And if the surface is still ambiguous after that, read these next:

- [Docs README](./README.md)
- [Aspect Contracts, Values, And Authoritative State](./aspect-contracts-values-and-authoritative-state/README.md)
- [Canonical Basis And Reproducible Identity](./canonical-basis-and-reproducible-identity/README.md)
- [Boundary Artifact Taxonomy And Materialization Contracts](./boundary-artifact-taxonomy-and-materialization-contracts/README.md)
- [Lineage, Provenance, Receipts, And Support Truth](./lineage-provenance-receipts-and-support-truth/README.md)
