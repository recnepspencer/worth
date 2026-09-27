# Milestone 3 Closeout: Assumption, Freshness, Re-Admission, And Downgrade

## Status

Closed.

Milestone 3 now has a machine-checkable named certification surface and an
explicit closure record for what later milestones may assume about basis-scoped
validity, downgrade, and trust-boundary re-admission.

## Implemented Surface

- Assumption-scoped validity carriers:
  - `FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>`
- Typed downgrade carriers:
  - `FreshnessScopedBasis<StaleReadable, AssumptionBasis<B>>`
  - `FreshnessScopedBasis<RebindRequired, AssumptionBasis<B>>`
  - `FreshnessScopedBasis<AuthorityRevalidationRequired, AssumptionBasis<B>>`
- Trust-boundary weakening and re-admission carriers:
  - `BoundaryBridged<...>`
  - `BoundaryBridgedStaleReadableBasis<B>`
  - `BoundaryBridgedRebindRequiredBasis<B>`
  - `BoundaryBridgedAuthorityRevalidationRequiredBasis<B>`
- Representative basis-sensitive recipe progression:
  - `Recipe<Unresolved, ...> -> Recipe<Resolved, ...>`
  - `Recipe<Resolved, ...> -> Recipe<Lowered, ...>`
  - `Recipe<Lowered, ...> -> Recipe<Admitted, ...>`
  - current-validity -> downgrade
  - current-validity -> boundary-bridged -> explicit rebind/readmission
- Representative basis-sensitive artifact progression:
  - current-validity -> downgrade
  - current-validity -> boundary-bridged -> explicit readmission

## Certification Surface

Named suite:

- `Assumption, Freshness, Re-Admission, And Downgrade Test`

Primary test:

- [assumption_freshness_and_downgrade_certification.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/assumption_freshness_and_downgrade_certification.rs)

Supporting evidence module:

- [tests/support/milestone3/mod.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/support/milestone3/mod.rs)

Machine-checkable outputs:

- `basis_digest`
- `failure_digest`
- `compile_fail_bundle`
- `transition_digest`

## Hostile Coverage

The closeout suite now owns the hostile lanes required by the milestone:

- unresolved forms cannot enter trust-boundary progression surfaces
- stale-readable forms cannot consume strong-basis APIs
- rebind-required forms cannot continue lowering progression
- boundary-bridged pre-readmission forms cannot consume strong-basis APIs
- shifted-basis readmission cannot be silently consumed as the original basis
- same-basis readmission can round-trip back into the original strong type
- shifted-basis readmission lands in a different strong type explicitly

Compile-fail fixtures:

- [unresolved_recipe_cannot_bridge_trust_boundary.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/unresolved_recipe_cannot_bridge_trust_boundary.rs)
- [stale_readable_recipe_rejects_strong_basis_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/stale_readable_recipe_rejects_strong_basis_api.rs)
- [rebind_required_recipe_cannot_lower.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/rebind_required_recipe_cannot_lower.rs)
- [boundary_bridged_recipe_rejects_strong_basis_api.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/boundary_bridged_recipe_rejects_strong_basis_api.rs)
- [shifted_basis_readmission_cannot_be_treated_as_original_basis.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/shifted_basis_readmission_cannot_be_treated_as_original_basis.rs)

Compile-pass fixtures:

- [explicit_current_validity_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/explicit_current_validity_progression_compiles.rs)
- [explicit_same_basis_readmission_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/explicit_same_basis_readmission_progression_compiles.rs)
- [explicit_readmission_progression_compiles.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-proof/tests/ui/explicit_readmission_progression_compiles.rs)

## Residual Debt

- Milestone 3 closes the shared substrate with representative same-basis,
  shifted-basis, stale, rebind, and trust-shift lanes, but it does not attempt
  to enumerate every domain-specific basis taxonomy. Domain crates still choose
  their own semantic basis families on top of this substrate.

This is explicit representative-scope debt, not a hidden milestone gap.

## Verification

Verified with:

- `cargo fmt -p worth-proof`
- `cargo test -p worth-proof assumption_freshness_readmission_and_downgrade_certification -- --nocapture`
- `cargo test -p worth-proof`

## What Later Milestones May Assume

Milestone 4, Milestone 5, Milestone 6, and Milestone 7 may now assume:

- stronger proof-bearing validity is basis-scoped, not ambiently global
- stale-readable, rebind-required, and authority-revalidation-required are
  first-class typed downgrade states
- trust-boundary crossings are explicit progression events that suspend strong
  status until rebind or readmission occurs
- same-basis re-admission returns to the original strong type exactly
- shifted-basis re-admission returns to a distinct strong type explicitly
- unresolved forms and pre-readmission forms are compile-time denied from
  stronger basis-sensitive APIs
- Milestone 1 and Milestone 2 remain canonical; Milestone 3 extends them
  without replacing the carrier family or weakening sealed witness authority
