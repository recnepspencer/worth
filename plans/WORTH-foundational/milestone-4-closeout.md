# Milestone 4 Closeout: Boundary Artifact Taxonomy And Materialization Contracts

Date: 2026-05-13

## Status

Milestone 4 is implementation-complete for `worth-foundational`.

The crate now owns the shared boundary-artifact language for category,
role/authority, materialization seams, bundle emission, canonical basis
participation, proof-bearing current-basis strengthening, planned-work and
same-family descriptive extension room, reserved authority-transition
fail-closed law, and production-test readiness evidence.

Crate-facing boundary-artifact docs now also exist under
[crates/worth-foundational/docs/boundary-artifact-taxonomy-and-materialization-contracts](../../crates/worth-foundational/docs/boundary-artifact-taxonomy-and-materialization-contracts),
with one landing page and one feature doc per shipped boundary-artifact seam.

This milestone is ready for production-shaped testing through `worth-harness`
or adopting-crate migration work. It does not claim that any adopting crate
has already lowered its real runtime envelopes, receipts, or support surfaces
into the foundational boundary-artifact language correctly.

## Completed Surface

- Typed boundary-artifact categories now exist for `Summary`, `Report`,
  `Artifact`, and `Receipt`.
- Category-local construction and denial law now prevent local wrappers or
  neighboring categories from impersonating each other.
- Typed role vocabulary now exists for `AuthoritativeCurrent`,
  `DerivedProjection`, `SupportOnly`, `PlannedWork`, and `ReceiptEvidence`.
- Category-role legality is mechanically enforced where statically knowable and
  explicitly evaluable where reporting is needed.
- Stronger authoritative-current claims now require proof-bearing authority
  admission rather than plain category tags.
- Materialization now has explicit source, seam, delivery class, availability,
  attachment-point, decision-row, plan, denial, and cost surfaces.
- Boundary materialization remains explicit and cost-visible rather than
  ambient accessor behavior.
- Coordinated multi-surface bundle emission now exists as a typed
  artifact-primary bundle lane with legality checks, duplicate-member
  rejection, member-category preservation, and aggregated cost.
- Materialized boundary outputs now lower through the Milestone 2 canonical
  basis lane without rebuilding canonicalization locally.
- Stronger current-basis boundary claims now exist as proof-bearing Phase 4.5
  surfaces with explicit trust-boundary bridging and readmission.
- Planned-work and same-family descriptive outputs now have typed homes that
  remain descriptive and preserve same-family identity.
- Reserved authority-transition law now remains fail-closed in Milestone 4
  rather than leaking branch/merge/commit ontology in early.
- Milestone 4 production-test readiness now exists as a proof-bearing artifact
  with certified-surface inventory, hostile-pressure inventory, compile-fail
  inventory, `worth-proof` appendix, runtime assumptions/non-assumptions, and
  residual debt.

## Phase Crosswalk

### Phase 1: Categories

Shipped homes:

- [categories.rs](../../crates/worth-foundational/src/boundary_artifacts/categories.rs)
- [categories.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/categories.rs)
- [ui/categories](../../crates/worth-foundational/tests/ui/boundary_artifacts/categories)

What closed:

- category vocabulary
- category-local construction law
- blind-consumer category definitions
- category non-substitution proof

### Phase 2: Roles And Authority

Shipped homes:

- [roles.rs](../../crates/worth-foundational/src/boundary_artifacts/roles.rs)
- [authority.rs](../../crates/worth-foundational/src/boundary_artifacts/authority.rs)
- [roles_and_authority.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/roles_and_authority.rs)
- [ui/authority_admission](../../crates/worth-foundational/tests/ui/boundary_artifacts/authority_admission)
- [ui/role_legality](../../crates/worth-foundational/tests/ui/boundary_artifacts/role_legality)

What closed:

- role vocabulary
- category-role legality
- proof-bearing authoritative-current admission
- blind-consumer role interpretation

### Phase 3: Materialization And Bundles

Shipped homes:

- [materialization/mod.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/mod.rs)
- [surface.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/surface.rs)
- [model.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/model.rs)
- [derivation.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/derivation.rs)
- [bundle.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/bundle.rs)
- [bundle_types.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/bundle_types.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/vocabulary.rs)
- [materialization.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/materialization.rs)
- [ui/materialization_contracts](../../crates/worth-foundational/tests/ui/boundary_artifacts/materialization_contracts)
- [ui/bundle_contracts](../../crates/worth-foundational/tests/ui/boundary_artifacts/bundle_contracts)

What closed:

- seam/source/delivery/availability law
- explicit planning and materialization lanes
- decision-row structure
- bundle legality and duplicate-member rejection
- cost-honest artifact/bundle materialization

### Phase 4: Canonical Basis Participation

Shipped homes:

- [basis.rs](../../crates/worth-foundational/src/boundary_artifacts/basis.rs)
- [canonical_basis.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/canonical_basis.rs)
- [ui/basis_boundaries](../../crates/worth-foundational/tests/ui/boundary_artifacts/basis_boundaries)

What closed:

- boundary-artifact canonical basis lowering
- bundle canonical basis participation
- parity across independent producers
- cost exclusion from canonical identity

### Phase 4.5: Current-Basis Proof Lane

Shipped homes:

- [current_basis.rs](../../crates/worth-foundational/src/boundary_artifacts/current_basis.rs)
- [current_basis.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/current_basis.rs)
- [ui/current_basis_boundaries](../../crates/worth-foundational/tests/ui/boundary_artifacts/current_basis_boundaries)

What closed:

- proof-bearing current-basis admission
- explicit trust-boundary bridging and readmission
- Milestone 2 canonicalization reuse for stronger basis claims
- caller-inaccessible stronger authority and readmission witnesses

### Phase 5: Descriptive Extension Law

Shipped homes:

- [planned.rs](../../crates/worth-foundational/src/boundary_artifacts/planned.rs)
- [same_family.rs](../../crates/worth-foundational/src/boundary_artifacts/same_family.rs)
- [reserved_authority_transition.rs](../../crates/worth-foundational/src/boundary_artifacts/reserved_authority_transition.rs)
- [descriptive_extensions.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/descriptive_extensions.rs)
- [ui/descriptive_extensions](../../crates/worth-foundational/tests/ui/boundary_artifacts/descriptive_extensions)

What closed:

- planned-work descriptive wrappers
- same-family descriptive wrappers and identity
- family-scoped canonical identity derivation
- reserved authority-transition fail-closed denials

### Phase 6: Production-Test Readiness

Shipped homes:

- [readiness/mod.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/mod.rs)
- [authority.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/authority.rs)
- [vocabulary.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/vocabulary.rs)
- [inventory.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/inventory.rs)
- [report.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/report.rs)
- [certification.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/certification.rs)
- [readiness.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/readiness.rs)
- [ui/readiness_boundaries](../../crates/worth-foundational/tests/ui/boundary_artifacts/readiness_boundaries)

What closed:

- proof-bearing readiness artifact
- exact certified-surface inventory
- exact `worth-proof` appendix
- hostile-pressure inventory
- compile-fail inventory
- runtime assumptions, non-assumptions, and residual debt

## WORTH-Proof Standardized Lane

Milestone 4 uses `worth-proof` in the places the spec intended, and does not
pull plain boundary vocabulary into the proof kernel.

Proof-bearing surfaces standardized here:

- Phase 2 authority admission through
  [authority.rs](../../crates/worth-foundational/src/boundary_artifacts/authority.rs)
- Phase 4.5 current-basis admission, trust-boundary bridge, and readmission
  through
  [current_basis.rs](../../crates/worth-foundational/src/boundary_artifacts/current_basis.rs)
- Phase 6 production-readiness certification through
  [readiness](../../crates/worth-foundational/src/boundary_artifacts/readiness)

Concrete `worth-proof` APIs the readiness artifact now freezes as the chosen
lane:

- `AuthorityWitness::from_authority_marker`
- `Proof::from_authority_witness`
- `Artifact::with_current_basis`
- `Artifact::with_proofs_and_current_basis`
- `TransitionOutcome`
- `bridge_trust_boundary`
- `readmit_with_authority`

Plain boundary vocabulary deliberately stayed local:

- category nouns
- role nouns
- materialization plans and costs
- bundle membership data
- same-family descriptive nouns

## Test-Requirements Mapping

Milestone 4 now satisfies the Milestone-4-specific bars that were added to
[plans/worth-foundational/test-requirements.md](./test-requirements.md).

### DX-Lane Separation

Evidence:

- [materialization.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/materialization.rs)
- [roles_and_authority.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/roles_and_authority.rs)

What is proved:

- descriptive, authoritative, current-basis, and readiness lanes remain
  distinct
- cheap-looking calls cannot satisfy stronger proof-bearing APIs

### Category Wrapper Collapse Rejection

Evidence:

- [categories.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/categories.rs)
- [local_generic_wrapper_cannot_satisfy_category_surface_trait.rs](../../crates/worth-foundational/tests/ui/boundary_artifacts/categories/local_generic_wrapper_cannot_satisfy_category_surface_trait.rs)

What is proved:

- `Summary`, `Report`, `Artifact`, and `Receipt` are not one shared payload
  wrapper with marker-only differentiation

### Bundle Legality

Evidence:

- [materialization.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/materialization.rs)
- [ui/bundle_contracts](../../crates/worth-foundational/tests/ui/boundary_artifacts/bundle_contracts)

What is proved:

- duplicate members are rejected
- non-artifact primary plans cannot start artifact bundles
- bundle members must match seam, source, and profile

### Decision-Row Structure

Evidence:

- [materialization.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/materialization.rs)

What is proved:

- rows carry subject, cause, seam, and affected category
- bundle-membership rows remain blind-consumer interpretable

### Delivery/Availability Legality

Evidence:

- [materialization.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/materialization.rs)
- [materialization/vocabulary.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/vocabulary.rs)
- [materialization/derivation.rs](../../crates/worth-foundational/src/boundary_artifacts/materialization/derivation.rs)

What is proved:

- illegal delivery/availability combinations fail closed
- support and richness posture affect availability explicitly

### Canonical Basis Parity

Evidence:

- [canonical_basis.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/canonical_basis.rs)
- [basis.rs](../../crates/worth-foundational/src/boundary_artifacts/basis.rs)

What is proved:

- independent producers lower to the same canonical boundary-artifact meaning
- payload layout does not define semantic identity

### Current-Basis Proof Lane

Evidence:

- [current_basis.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/current_basis.rs)
- [ui/current_basis_boundaries](../../crates/worth-foundational/tests/ui/boundary_artifacts/current_basis_boundaries)

What is proved:

- stronger basis claims reuse Milestone 2 canonicalization and `worth-proof`
- raw materialized artifacts cannot bypass stronger proof-bearing lanes

### Descriptive Extension Fail-Closed Law

Evidence:

- [descriptive_extensions.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/descriptive_extensions.rs)
- [ui/descriptive_extensions](../../crates/worth-foundational/tests/ui/boundary_artifacts/descriptive_extensions)

What is proved:

- planned-work and same-family outputs remain descriptive
- reserved authority transitions are not smuggled through Milestone 4

### Readiness Closure

Evidence:

- [readiness.rs test](../../crates/worth-foundational/tests/certification/boundary_artifacts/readiness.rs)
- [readiness/inventory.rs](../../crates/worth-foundational/src/boundary_artifacts/readiness/inventory.rs)

What is proved:

- certified surfaces, pressures, compile-fail boundaries, `worth-proof`
  surfaces, assumptions, non-assumptions, and debt are exact

## Final QA Fixes

- Tightened the stronger authority and current-basis lanes so they now carry
  real proof markers rather than authority-gated `NoProofs` artifacts.
- Renamed the descriptive-extension guard surface from milestone-provenance
  naming into responsibility-owned naming:
  [reserved_authority_transition.rs](../../crates/worth-foundational/src/boundary_artifacts/reserved_authority_transition.rs)
  and
  [descriptive_extensions.rs](../../crates/worth-foundational/tests/certification/boundary_artifacts/descriptive_extensions.rs).
- Corrected the materialization split so model-owned types and plan
  materialization stayed in the model layer instead of leaking through the
  entrypoint layer.
- Removed stale milestone-provenance names from the exported readiness
  vocabulary so the public closure artifact now uses responsibility-based names
  for the reserved-authority-transition boundary as well.

## Proof Evidence

- Certification tests cover categories, role/authority law, materialization
  and bundle emission, canonical basis lowering, current-basis proof lane,
  descriptive extension law, and Phase 6 readiness.
- Compile-fail tests prove raw labels, wrong category-role substitutions,
  plain payload shortcuts, raw materialized outputs, planned-work wrappers, and
  same-family wrappers cannot satisfy stronger APIs.
- Blind-consumer style certification tests prove category, role, seam,
  availability, attachment, decision-row, basis, and readiness meaning remain
  interpretable without producer-private state.
- Topology checks show all touched boundary-artifact production and test files
  remain under the 400-line cap, and all boundary-artifact proof directories
  remain under the 10-direct-file cap through responsibility-shaped
  subdivision.
- Crate-facing feature docs now exist for categories, roles/authority,
  materialization/bundles, canonical/current-basis, descriptive extensions,
  and readiness.

## Verification

The final QA pass ran:

```powershell
cargo fmt -p worth-foundational
cargo test -p worth-foundational --test certification boundary_artifacts::readiness -- --nocapture
cargo test -p worth-foundational
git diff --check
```

All passed.

Result counts from the full suite after Milestone 4 closure:

- `5` unit tests passed.
- `163` certification tests passed.
- `31` compile-time boundary test groups passed.
- `0` doc tests ran.

## Explicit Deferrals

Milestone 4 does not implement:

- real branch/merge/commit authority-transition ontology
- diagnostics ontology
- provenance or lineage ontology
- receipt semantics beyond category/role/materialization boundary law
- real adopting-crate migration parity
- a universal artifact registry, executor, or storage model
- Milestone 5 or any later roadmap milestone

Those remain downstream roadmap work. Milestone 4 closes the shared
boundary-artifact category, role, materialization, basis, descriptive
extension, and readiness language that those later surfaces must consume.
