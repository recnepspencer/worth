# Milestones 1-3 DX Alignment Plan

## Goal

Define the DX hardening work that was required before crate-facing
documentation for Milestones 1, 2, and 3 could honestly meet the standard now
set by Milestones 4, 5, and 6.

This document is not a substitute for the milestone specs. Those specs already
close semantic law. This document answers a different question:

> What should the Milestone 1-3 public code feel like when an engineer is
> actually using the finished surfaces, and what must change before we can
> document those surfaces without making the docs compensate for weak API
> shape?

## Current Status

- Milestone 1 DX hardening is now materially implemented and QA-hardened.
- Milestone 2 DX hardening is now materially implemented and QA-hardened.
- Milestone 3 DX hardening is now materially implemented and QA-hardened.
- The remaining work in this document is no longer milestone DX design or
  implementation. It is:
  - preserve the completion record for Milestones 1, 2, and 3
  - use that completion record as the gating artifact for the later docs
    backfill

## Why This Exists

Milestones 1, 2, and 3 are semantically strong and structurally honest. They
already preserve the foundational boundary laws the crate roadmap required.

What they did not consistently do at the start of this work was teach their
common path through the API surface itself.

Right now:

- the code is stronger than the call-site experience
- the facade exports are denser than the intended user journeys
- the milestone specs are clearer about progression than the public API is
- the upcoming crate-facing docs for Milestones 1-3 would have to recover too
  much structure instead of teaching what the code already makes obvious

That is a DX problem, not a semantic-correctness problem.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first design and mechanical enforcement over
convention. The shaping constraint for this plan is that DX hardening must solve
the real failure mode first: engineers must be guided into the correct common
lane by the API itself rather than by prose memory or lucky file discovery.

### `arch_laws.md`

Protects facade authority, proof-bearing progression, explicit boundary
crossings, and category separation. The shaping constraint is that DX cleanup
must not flatten stronger lanes into friendly convenience mush or make proof
boundaries disappear.

### `composition_laws.md`

Protects semantic files, named orchestration, and predictable logic hierarchy.
The shaping constraint is that the public path through a milestone must read
like named semantic progression rather than a bag of exports.

### `domain_structure_laws.md`

Protects structural topology as real responsibility ownership. The shaping
constraint is that any DX hardening must reinforce, not blur, the existing
module boundaries. The solution is clearer front doors and progression seams,
not reintroducing buckets.

### `perf_laws.md`

Protects cost-visible boundaries and explicit materialization/planning seams.
The shaping constraint is that friendlier APIs must still make expensive
boundaries look expensive and must not disguise broad canonicalization,
materialization, or proof work as cheap getters.

### `worth_foundational_vision.md`

Protects the thesis that `worth-foundational` standardizes shared meaning
without forcing one runtime representation. The shaping constraint is that DX
must make shared meaning easier to use while still preserving representation
freedom and boundary honesty.

### `worth_foundational_roadmap.md`

Protects the sequence and capability boundaries of Milestones 1-3. The shaping
constraint is that this plan must not reopen milestone order or re-scope the
crate; it should make the already-shipped milestones easier to consume and
document honestly.

### `test-requirements.md`

Protects the hostile local proof bar before downstream runtime adoption. The
shaping constraint is that every DX change must preserve compile-fail
boundaries, parity tests, misuse-pressure tests, and readiness evidence rather
than weakening them for authoring convenience.

### `milestone-1.md`

Protects the canonical value, aspect, contract, mask, state, patch, identity,
locator, and compatibility substrate. The shaping constraint is that Milestone
1 DX must expose the real aspect workflow clearly and must not quietly center
only the scalar happy path.

### `milestone-1-closeout.md`

Protects the fact that Milestone 1 is already semantically closed. The shaping
constraint is that this plan must harden API shape and discoverability rather
than revisiting milestone law.

### `milestone-2.md`

Protects canonical basis as semantic authority and digest as derived
compression. The shaping constraint is that Milestone 2 DX must make basis,
comparison, export, and digest lanes easier to navigate without weakening that
authority rule.

### `milestone-2-closeout.md`

Protects the completed canonicalization substrate and its exact proof lanes.
The shaping constraint is that DX improvements must preserve the explicit
readiness and comparison grammar already shipped.

### `milestone-3.md`

Protects typed profile families, composition, progression, attachment,
materialization planning, and certification posture. The shaping constraint is
that Milestone 3 DX must make the requested -> admitted -> materialized journey
obvious without collapsing target legality or stronger proof-bearing lanes.

### `milestone-3-closeout.md`

Protects the fact that Milestone 3 is already implementation-complete. The
shaping constraint is that docs should land only after the public surface is
teachably shaped, not because the internal implementation is already sound.

## Adversarial Constraint

An engineer who has not internalized the milestone prose must still be able to
discover the correct common path for Milestones 1, 2, and 3 directly from the
public crate surface, while hostile misuse cases still fail mechanically and
expensive or stronger boundaries still look expensive or stronger.

This plan fails if:

- the docs have to invent a user journey that the code does not visibly teach
- the public surface keeps reading like a flat export ledger
- the scalar happy path becomes the only obvious Milestone 1 story while
  structs, masks, field paths, and field-level patching become folklore
- Milestone 2 keeps all the right nouns but still gives no obvious answer to
  "what code do I write first?"
- Milestone 3 keeps its proof lanes but still makes the attachment and
  materialization journey inferential instead of obvious
- DX cleanup weakens compile-time boundaries, proof progression, or cost
  honesty in exchange for prettier call sites

## Scope

This plan covers:

- public DX hardening for Milestones 1, 2, and 3
- facade and front-door shape
- common-lane versus lower-lane guidance
- the required pre-doc changes that governed the completed work
- explicit coverage requirements so docs do not accidentally erase important
  capabilities

This plan does not cover:

- semantic redesign of Milestones 1, 2, or 3
- rewriting later milestones
- adopting-crate migration work
- replacing readiness, compile-fail, or certification law with narrative docs

## Cross-Milestone Findings

### What Is Already Strong

- Internal responsibility topology is good across all three milestones.
- The milestone specs and closeouts already encode real progression law.
- Compile-fail and certification proof bars are strong.
- The code is not semantically vague; it is mostly discoverability-vague.

### What Is Still Weak

- [facade.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/facade.rs)
  is too export-dense for Milestones 1-3.
- Common-path entry is not obvious enough from the public surface alone.
- Lower lanes and stronger lanes are visible, but recommended lanes are not
  shaped clearly enough.
- Milestones 1-3 are still easier to understand from the specs than from the
  call sites.

This section is now historically true for the starting point of the work, not
the current state. It no longer describes Milestones 1, 2, or 3 accurately
after the completed DX batches below.

### Global DX Rules

The Milestone 1-3 DX pass must follow these rules:

- Keep the existing responsibility topology unless a real structural defect is
  discovered.
- Prefer clearer semantic front doors over broader flat re-exports.
- Keep stronger proof-bearing lanes visibly stronger than common authoring
  lanes.
- Keep expensive boundaries visible in API shape.
- Do not introduce generic convenience bags, helper buckets, or fake builder
  DSLs that hide milestone law.
- Every common lane must have a corresponding lower lane that remains available
  for exact inspection and stronger proof.

## Desired End State

When this alignment work is finished, each early milestone should have:

- one obvious common path
- one obvious lower/inspectable path
- one obvious stronger/proof-bearing path where relevant
- a facade that groups by milestone responsibility rather than by raw export
  accumulation
- docs that can teach what the code already reveals instead of compensating for
  it

## Milestone 1: Aspects And Authoritative State

### Status

Implemented and hostile-QA'd.

### Current Strengths

- The internal `aspects/` tree is already responsibility-shaped.
- Contract validation, state admission, patch law, masks, structs, and
  compatibility lowering already exist as real semantic surfaces.
- The milestone already has strong compile-fail and canonicalization proof.

### Historical DX Weaknesses

- The common authoring journey is too implicit.
- The surface is noun-rich but workflow-light.
- Struct contracts, field declarations, masks, and field-level patching are
  too easy to mentally downgrade into "advanced details."
- Compatibility lowering is present but not shaped clearly enough as an
  explicit boundary lane.
- Identities and locators are important but currently compete with the primary
  authoring story instead of clearly supporting it.

### Required Coverage

Milestone 1 DX is not acceptable unless the common path explicitly covers all
of the following:

- scalar contract authoring
- struct contract authoring
- field declaration authoring
- field-path authoring and interpretation
- projection mask authoring
- mutation mask authoring
- diagnostic mask authoring
- scalar validation
- struct-valued authoring and validation
- authoritative state admission
- whole-aspect patch authoring
- field-level struct patch authoring
- compatibility lowering
- locator and identity usage

Collection-like or struct-array-shaped support must not disappear into silence.
If the milestone currently rejects or defers those shapes, the API must make
that rejection/debt explicit rather than leaving the user to discover it by
failure archaeology.

### Desired Public Grammar

The Milestone 1 common path should read like this:

1. define an aspect contract
2. define struct fields or masks when the contract needs them
3. validate a value against the contract
4. admit authoritative state from validated values
5. build and apply patches against that state
6. cross the compatibility boundary explicitly when lowering legacy inputs

Representative target shape:

```rust
let contract = aspects::contract()
    .for_key(aspect_key)
    .struct_shape(
        aspects::struct_fields()
            .required("x", ScalarAspectType::Int32)
            .required("y", ScalarAspectType::Int32)
            .optional("label", ScalarAspectType::String)
            .finish()?,
    )
    .with_masks(
        aspects::mask_contract()
            .projection_paths(["x", "y", "label"])
            .mutation_paths(["label"])
            .diagnostic_paths(["x", "y", "label"])
            .finish()?,
    )
    .finish()?;

let validated = aspects::validate()
    .against(&contract)
    .value(
        aspects::struct_value()
            .with_field("x", AspectValue::Int32(3))
            .with_field("y", AspectValue::Int32(4))
            .with_field("label", AspectValue::String("origin".into()))
            .finish()?,
    )?;

let state = aspects::authoritative_state().admit([validated])?;

let patch = aspects::patch()
    .for_contract(&contract)
    .set_field("label", AspectValue::String("moved".into()))
    .finish()?;
```

This is illustrative, not a literal API mandate, but the call-site feel is the
requirement.

### Required Pre-Doc Changes

- Establish explicit semantic front doors under `aspects` for:
  - contract authoring
  - struct field authoring
  - mask authoring
  - value validation
  - authoritative state admission
  - patch authoring
- Make the common path discoverable without opening deep submodules first.
- Make struct and scalar authoring visibly different at the call site.
- Make field-level patching a first-class lane rather than an advanced side
  capability.
- Make compatibility lowering read like a named bridge, not an alternate
  normal authoring path.
- Keep ids and locators grouped as supporting vocabulary rather than letting
  them clutter the primary authoring progression.

### Implemented Surface

Milestone 1 now has a real common lane through:

- [crates/worth-foundational/src/aspects/front_doors/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/mod.rs)
- [crates/worth-foundational/src/aspects/front_doors/contract.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/contract.rs)
- [crates/worth-foundational/src/aspects/front_doors/masks.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/masks.rs)
- [crates/worth-foundational/src/aspects/front_doors/validation.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/validation.rs)
- [crates/worth-foundational/src/aspects/front_doors/state.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/state.rs)
- [crates/worth-foundational/src/aspects/front_doors/patches.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/patches.rs)
- [crates/worth-foundational/src/aspects/front_doors/values.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/aspects/front_doors/values.rs)
- [crates/worth-foundational/src/compatibility/front_doors.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/compatibility/front_doors.rs)

The common lane now honestly covers:

- scalar contract authoring
- struct contract authoring
- explicit scalar and struct contract law when masks/equivalence/evolution must be named
- field declaration authoring
- field-path authoring and interpretation
- projection, mutation, and diagnostic mask authoring
- scalar validation
- struct-valued authoring and validation
- authoritative state admission
- whole-aspect patch authoring
- field-level patch authoring
- patch application back into authoritative state
- explicit compatibility lowering through `compatibility().json()`
- locator and identity usage through the supporting vocabulary lane

The surface is mechanically frozen into the Milestone 1 readiness contract in:

- [crates/worth-foundational/src/boundary/milestone1_readiness.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/boundary/milestone1_readiness.rs)
- [crates/worth-foundational/src/boundary/milestone1_readiness_certification.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/boundary/milestone1_readiness_certification.rs)

And it is externally proven by:

- [crates/worth-foundational/tests/certification/aspects/front_doors.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/aspects/front_doors.rs)
- [crates/worth-foundational/tests/certification/compatibility/front_doors.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/compatibility/front_doors.rs)
- [crates/worth-foundational/tests/ui/aspect_front_doors](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/ui/aspect_front_doors)
- [crates/worth-foundational/tests/ui/compatibility_front_doors](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/ui/compatibility_front_doors)

### Non-Goals

- Flattening contract, validation, state, and patch law into one convenience
  surface.
- Hiding proof-bearing validated artifacts behind raw values.
- Treating masks as secondary trivia.
- Teaching only scalar aspects and leaving the rest to docs footnotes.

### Acceptance Evidence

- A new engineer can discover the scalar path and struct path from the public
  `aspects` surface without deep-file archaeology.
- Struct/mask/field-path/field-patch examples are first-class in docs and in
  certification-style common-path examples.
- Compile-fail boundaries remain at least as strong as they are now.
- Compatibility lowering remains visibly separate from native validation.

This acceptance bar is now satisfied by the current implementation and proof
surface.

## Milestone 2: Canonicalization And Digest Basis

### Status

Implemented and hostile-QA'd.

### Current Strengths

- Internal topology is very strong and responsibility-shaped.
- Canonical basis, equivalence, mismatch, export, digest slots, and readiness
  are all real distinct surfaces.
- The milestone is semantically rigorous and already closes the hard problem.

### Historical DX Weaknesses

- The public surface is the least approachable of the three milestones.
- The facade exposes many correct nouns without clearly teaching the main
  journeys.
- The digest slot story is especially dense for first contact.
- The milestone is easy to respect and still hard to start using.

### Desired Public Grammar

The Milestone 2 common path should read like this:

1. prepare canonical basis from a certified foundational surface
2. compare basis under an explicit equivalence basis
3. prepare an export bundle when a basis needs to travel
4. derive digest from admitted canonical basis or export basis
5. inspect readiness and mismatch evidence explicitly

Representative target shape:

```rust
let sequence = canonicalization::basis()
    .for_authoritative_state(&state)
    .prepare()?;

let comparison = canonicalization::compare()
    .left(&left_sequence)
    .right(&right_sequence)
    .under(CanonicalEquivalenceBasis::ExactCanonicalBasis)
    .prepare()?;

let export = canonicalization::export()
    .from_bundle(&bundle)
    .prepare()?;

let digest = canonicalization::digest()
    .from_sequence(&sequence)
    .using(slot)
    .derive()?;
```

Again, the exact names may differ. The required outcome is an API that teaches
these lanes clearly.

### Required Pre-Doc Changes

- Establish obvious front doors for:
  - basis preparation
  - comparison
  - export
  - digest derivation
  - readiness inspection
- Reduce facade-first dependence for discovering the common path.
- Make digest-slot selection feel downstream of basis readiness, not like one
  more adjacent noun in a sea of exports.
- Make the "basis is authority, digest is compression" rule obvious from the
  API story and docs examples.
- Keep mismatch and unsupported comparison visible rather than quietly
  secondary to equality-style flows.

### Implemented Surface

Milestone 2 now has a real grouped public surface through:

- [crates/worth-foundational/src/canonicalization/front_doors](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/canonicalization/front_doors)
- [crates/worth-foundational/src/canonicalization_api/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/canonicalization_api/mod.rs)
- [crates/worth-foundational/src/canonicalization_api/common_path.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/canonicalization_api/common_path.rs)
- [crates/worth-foundational/src/canonicalization_api/lower_lane](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/canonicalization_api/lower_lane)
- [crates/worth-foundational/src/canonicalization_api/stronger_lane](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/canonicalization_api/stronger_lane)
- [crates/worth-foundational/src/canonicalization_api/inventory.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/canonicalization_api/inventory.rs)

The implemented public grammar now provides:

- one obvious staged common path
- one grouped lower lane by real semantic sublane
- one explicitly stronger readiness-owned lane
- direct mismatch and readiness inspection on the common path
- digest derivation that stays downstream of admitted basis/export readiness
- machine-checkable public-surface inventory in the canonical readiness artifact

The grouped surface is externally proven by:

- [crates/worth-foundational/tests/certification/canonicalization/front_doors.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/canonicalization/front_doors.rs)
- [crates/worth-foundational/tests/certification/canonicalization/grouped_surface.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/canonicalization/grouped_surface.rs)
- [crates/worth-foundational/tests/certification/canonicalization/production_readiness/public_surface_inventory.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/canonicalization/production_readiness/public_surface_inventory.rs)
- [crates/worth-foundational/tests/ui/canonicalization/front_doors](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/ui/canonicalization/front_doors)
- [crates/worth-foundational/tests/ui/canonicalization/grouped_surface](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/ui/canonicalization/grouped_surface)

### Non-Goals

- Hiding canonical basis behind direct digest conveniences.
- Replacing typed canonicalization lanes with one generic helper surface.
- Making digest values feel like semantic authority.

### Acceptance Evidence

- An engineer can discover the five real Milestone 2 lanes quickly from the
  public surface.
- Docs can be organized by basis, comparison, export, digest, and readiness
  without inventing a better structure than the code already teaches.
- Compile-fail and readiness proof lanes remain intact and visible.

This acceptance bar is now satisfied by the current implementation and proof
surface.

## Milestone 3: Profiles And Policy Vocabulary

### Status

Implemented and hostile-QA'd.

### Current Strengths

- Public topology is cleaner than Milestones 1 and 2.
- The progression law is real.
- Composition, attachment, identity, materialization, certification, and
  readiness all already exist as named capabilities.

### Historical DX Weaknesses

- The common journey is still more obvious in the spec than in the code.
- Attachment and materialization remain a little more inferential than they
  should be.
- The stronger certification lane is real but not yet shaped quite as clearly
  downstream of plain profile meaning as it should be.

### Desired Public Grammar

The Milestone 3 common path should read like this:

1. compose a total profile set
2. request profile meaning
3. admit profile meaning
4. attach it to a legal target
5. plan or materialize target-scoped descriptive surfaces
6. strengthen into evidence-backed or production-certified form only when
   needed

Representative target shape:

```rust
let requested = profiles::set()
    .diagnostic_richness(DiagnosticRichnessProfile::Standard)
    .support_posture(SupportPostureProfile::SupportReady)
    .compatibility_posture(CompatibilityPostureProfile::NativeOnly)
    .admission_readiness(AdmissionReadinessProfile::Admitted)
    .retention_delivery(RetentionDeliveryProfile::Retained)
    .certification_posture(CertificationPostureProfile::Uncertified)
    .request()?;

let admitted = profiles::admit(requested)?;

let attached = profiles::attach()
    .to_boundary_artifact(&artifact)
    .profile(admitted)?;

let plan = profiles::materialization()
    .for_target(&attached)
    .plan()?;
```

### Required Pre-Doc Changes

- Make the composed profile-set construction lane more obviously the front
  door.
- Make requested -> admitted -> materialized progression visible as the main
  path, not just as a set of adjacent types.
- Make target-aware attachment read like a legal boundary crossing, not like a
  generic wrapper helper.
- Make materialization planning visibly downstream of admitted meaning and
  target legality.
- Make evidence-backed and production-certified strengthening read like a
  clearly stronger lane rather than just one more adjacent export family.

### Implemented Surface

Milestone 3 now has a real common lane through:

- [crates/worth-foundational/src/profiles/front_doors/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/front_doors/mod.rs)
- [crates/worth-foundational/src/profiles/front_doors/set.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/front_doors/set.rs)
- [crates/worth-foundational/src/profiles/front_doors/progression.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/front_doors/progression.rs)
- [crates/worth-foundational/src/profiles/front_doors/attachment.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/front_doors/attachment.rs)
- [crates/worth-foundational/src/profiles/front_doors/materialization.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/front_doors/materialization.rs)
- [crates/worth-foundational/src/profiles/front_doors/certification.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/front_doors/certification.rs)

Milestone 3 now also has a grouped public surface through:

- [crates/worth-foundational/src/profiles_api/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles_api/mod.rs)
- [crates/worth-foundational/src/profiles_api/common_path.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles_api/common_path.rs)
- [crates/worth-foundational/src/profiles_api/lower_lane](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles_api/lower_lane)
- [crates/worth-foundational/src/profiles_api/stronger_lane](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles_api/stronger_lane)
- [crates/worth-foundational/src/profiles_api/inventory.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles_api/inventory.rs)

The implemented public grammar now provides:

- one obvious common path for composed profile request, admitted progression,
  target-aware attachment, target-scoped materialization, and explicit
  strengthening
- one grouped lower lane for composition, progression, attachment,
  materialization, identity/difference, and certification inspection
- one explicitly stronger readiness-owned lane
- machine-checkable public-surface inventory in the Milestone 3 readiness
  artifact

The shaped surface is mechanically frozen into the Milestone 3 readiness
contract in:

- [crates/worth-foundational/src/profiles/readiness/inventory.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/readiness/inventory.rs)
- [crates/worth-foundational/src/profiles/readiness/report.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/src/profiles/readiness/report.rs)

And it is externally proven by:

- [crates/worth-foundational/tests/certification/profiles/front_doors.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/profiles/front_doors.rs)
- [crates/worth-foundational/tests/certification/profiles/grouped_surface.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/profiles/grouped_surface.rs)
- [crates/worth-foundational/tests/certification/profiles/readiness.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/certification/profiles/readiness.rs)
- [crates/worth-foundational/tests/ui/profiles/front_doors](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/ui/profiles/front_doors)
- [crates/worth-foundational/tests/ui/profiles/grouped_surface](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/crates/worth-foundational/tests/ui/profiles/grouped_surface)

### Non-Goals

- Collapsing requested, admitted, and materialized meaning into one mutable
  effective profile surface.
- Hiding target legality or absence-cause law in the name of convenience.
- Letting stronger certification posture look like a cheap enum swap.

### Acceptance Evidence

- The main profile journey can be taught directly from the code surface.
- Docs do not need to invent a cleaner user flow than the API already exposes.
- Attachment legality, materialization planning, and stronger certification
  remain explicit seams.

This acceptance bar is now satisfied by the current implementation and proof
surface.

## Implementation Order

The DX work should be done in this order:

1. Milestone 2 front-door alignment
2. Milestone 1 common-path hardening
3. Milestone 3 common-path tightening
4. Facade cleanup once the milestone-local front doors are real
5. Milestone 1 docs
6. Milestone 2 docs
7. Milestone 3 docs

Progress update:

- Step 1 is complete.
- Step 2 is complete.
- Step 3 is complete.
- Steps 4 through 7 are now complete.

The reason for this order:

- Milestone 2 currently has the biggest gap between semantic strength and
  public usability.
- Milestone 1 needs the deepest API hardening because the required coverage is
  broad and easy to underspecify.
- Milestone 3 is closest to doc-ready and should benefit from the same
  standard after the earlier alignment work sets the precedent.

## Must Preserve

- existing milestone semantics
- existing proof-bearing progression law
- existing compile-fail and misuse-pressure boundaries
- explicit cost boundaries
- explicit category separation
- responsibility-shaped module topology

## Acceptance Evidence

- a follow-up implementation plan exists per milestone before code changes
  begin
- each milestone has an agreed desired common-path call-site grammar
- Milestone 1 coverage explicitly includes structs, fields, masks, field
  paths, field-level patching, and compatibility lowering
- docs for Milestones 1-3 are deferred until the public surface can honestly
  teach the intended journey

Current read:

- Milestone 1 satisfies this bar.
- Milestone 2 satisfies this bar.
- Milestone 3 satisfies this bar.
- The DX alignment phase for Milestones 1-3 is complete, and the docs backfill
  phase can now honestly begin.

## Self-Check

- Does this solve a real structural problem rather than packaging cosmetic
  cleanup? Yes. It targets the gap between semantically correct infrastructure
  and teachable public API shape.
- Is the adversarial constraint precise and load-bearing? Yes. The constraint
  is that an engineer must be guided into the right lane by the API itself
  without weakening proof or cost boundaries.
- Does this preserve authority boundaries? Yes. The plan hardens front doors
  and progression visibility without flattening stronger lanes.
- Could a competent engineer map this into concrete follow-up work? Yes. The
  plan names the milestone-local target shapes, required coverage, required
  pre-doc changes, and implementation order.
