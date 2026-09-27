# Transition Production Readiness

## What This Feature Is

This feature is the machine-checkable closeout surface for transition
vocabulary milestones.

It gives you a proof-bearing readiness artifact and an inspectable report that
freeze the exact closure contract for a named transition scope:

- certified surfaces
- hostile pressures
- compile-fail boundaries
- `worth-proof` appendix
- forbidden `worth-proof` surfaces
- assumptions
- non-assumptions
- residual debt
- linear phase gates

## Why You Use It

Use this surface when you need to:

- verify what Milestone 5 actually certifies
- verify what Milestone 9 scoped merge and cherry-pick vocabulary certifies
- understand what later diagnostics and provenance work may assume
- see the exact hostile pressures that adoption work still has to respect
- confirm the exact `worth-proof` lane that shipped

If you are lowering a real runtime into these transition surfaces, this is the
artifact that tells you what is frozen and what is still deferred.

## Stable Entry Points

- `foundational_transition_milestone5_readiness_report()`
- `certify_foundational_transition_milestone5_production_test_readiness()`
- `require_foundational_transition_milestone5_production_test_readiness(...)`
- `foundational_transition_milestone9_scoped_merge_readiness_report()`
- `certify_foundational_transition_milestone9_scoped_merge_production_test_readiness()`
- `require_foundational_transition_milestone9_scoped_merge_production_test_readiness(...)`

Important types:

- `FoundationalTransitionProductionTestReadyArtifact`
- `FoundationalTransitionProductionReadinessReport`
- `FoundationalTransitionCertifiedSurface`
- `FoundationalTransitionSyntheticRuntimePressure`
- `FoundationalTransitionCompileFailBoundary`
- `FoundationalTransitionWORTHProofSurface`
- `FoundationalTransitionWORTHProofApi`
- `FoundationalTransitionResidualDebt`

## Core Mental Model

Readiness here means exact closure for a named scope, not "good enough
summary."

This report exists so later work does not have to infer:

- which transition lanes are certified
- which hostile pressures are already owned locally
- which compile-fail boundaries are part of the contract
- which `worth-proof` APIs are real Milestone 5 commitments
- which plain transition surfaces are explicitly forbidden from becoming local
  proof substrates
- which downstream gaps are still explicitly deferred

The report carries a `scope()` so consumers can distinguish the original
Milestone 5 transition closeout from the Milestone 9 scoped merge closeout.

## How It Executes

The readiness path:

1. inventories certified surfaces
2. inventories hostile pressures
3. inventories compile-fail boundaries
4. inventories required and forbidden `worth-proof` surfaces
5. inventories assumptions, non-assumptions, residual debt, and phase gates
6. certifies the final artifact through the readiness authority lane

The Milestone 5 frozen certified surfaces are:

- `BranchLocalSeparation`
- `MergeVerdictLaw`
- `CommittedAuthorityTransitions`
- `CommitReceiptsAndBundles`
- `CanonicalBasisAndLocatorIntegration`
- `ProfileRichnessAndCurrentBasisBehavior`

The Milestone 5 frozen hostile pressures are:

- `AuthoritySeparation`
- `MergeTopologyHonesty`
- `NoOpVersusCommitClassification`
- `ReceiptIssuanceBoundary`
- `ReplayInterpretationBoundary`
- `ReducedRichnessPreservation`
- `AmbientBasisChoiceHostility`
- `HiddenStrategyInfluenceHostility`
- `ThinReceiptRejection`
- `GenericTransitionResultBagRejection`
- `CheapConvenienceBypassRejection`

The Milestone 5 frozen compile-fail boundaries are:

- `BranchLocalSurfacesCannotSatisfyAuthorityApis`
- `MergeAdmissionSurfacesRemainNonAuthoritative`
- `CommittedAuthorityRequiresProofBearingAdmission`
- `ReceiptAndCloseoutPreserveAuthoritySeparation`
- `Phase5BasisAndCurrentBasisRequireStrengthenedArtifacts`
- `TransitionReadinessRequiresCertifiedArtifact`
- `TransitionReadinessAuthorityCannotBeMinted`

The Milestone 5 frozen forbidden `worth-proof` surfaces are:

- `PlainBranchLocalVocabulary`
- `PlainMergeVerdictVocabulary`
- `PlainReceiptAndBundleVocabulary`
- `PlainCanonicalBasisAndLocatorVocabulary`

The Milestone 5 linear phase gates are:

1. `BranchLocalSeparation`
2. `MergeVerdictLaw`
3. `CommittedAuthorityTransitionLaw`
4. `CommitReceiptsAndBundles`
5. `CanonicalBasisLocatorAndProfileIntegration`
6. `ProductionReadiness`

The Milestone 9 scoped merge readiness scope additionally certifies:

- `ScopedMergeRequestVocabulary`
- `ScopedMergeAdmissionEvidence`
- `ScopedMergeDenialUnavailableTopology`
- `ScopedMergeCanonicalLocatorDiagnostics`
- `ScopedMergeAdoptionContract`

Its hostile pressures cover category substitution, producer diversity,
denial/unavailable honesty, canonical locator stability, and runtime boundary
honesty. Its residual debt names adopting-crate scoped merge execution, native
cherry-pick execution, and runtime conflict materialization as deferred rather
than silently solved here.

## Small Example

```rust
use worth_foundational::{
    certify_foundational_transition_milestone5_production_test_readiness,
    require_foundational_transition_milestone5_production_test_readiness,
};

let artifact = certify_foundational_transition_milestone5_production_test_readiness();
let report = require_foundational_transition_milestone5_production_test_readiness(&artifact);

assert!(report.passes_readiness_checklist());
```

For scoped merge vocabulary:

```rust
use worth_foundational::{
    certify_foundational_transition_milestone9_scoped_merge_production_test_readiness,
    require_foundational_transition_milestone9_scoped_merge_production_test_readiness,
};

let artifact =
    certify_foundational_transition_milestone9_scoped_merge_production_test_readiness();
let report =
    require_foundational_transition_milestone9_scoped_merge_production_test_readiness(&artifact);

assert!(report.passes_readiness_checklist());
assert_eq!(
    report.scope().milestone(),
    "worth-foundational.milestone-9.scoped-merge",
);
```

## Real Example

Use the report as an adoption handoff:

```rust
use worth_foundational::foundational_transition_milestone5_readiness_report;

let report = foundational_transition_milestone5_readiness_report();

let certified_surfaces = report.certified_surfaces();
let hostile_pressures = report.synthetic_pressures();
let compile_fail_boundaries = report.compile_fail_boundaries();
let worth_proof_api_appendix = report.worth_proof_api_appendix();
let residual_debt = report.residual_debt();

let _ = (
    certified_surfaces,
    hostile_pressures,
    compile_fail_boundaries,
    worth_proof_api_appendix,
    residual_debt,
);
```

Use the scoped merge report before adopting-runtime work:

```rust
use worth_foundational::{
    foundational_transition_milestone9_scoped_merge_readiness_report,
    FoundationalTransitionCertifiedSurface,
};

let report = foundational_transition_milestone9_scoped_merge_readiness_report();

assert!(report
    .certified_surfaces()
    .iter()
    .any(|surface| *surface == FoundationalTransitionCertifiedSurface::ScopedMergeAdoptionContract));
assert!(report.passes_readiness_checklist());
```

## How It Relates To Other Features

- Every other Milestone 5 doc in this folder describes a capability whose exact
  closure status is frozen here.
- The scoped merge doc describes Milestone 9 vocabulary whose exact closure
  status is frozen by the scoped merge readiness report.
- This readiness surface is the handoff target for Milestone 6 diagnostics and
  later provenance work.

## Inspection And Debugging

Check these first:

- `certified_surfaces()`
- `certified_surface_evidence()`
- `synthetic_pressures()`
- `compile_fail_boundaries()`
- `worth_proof_required_surfaces()`
- `worth_proof_api_appendix()`
- `worth_proof_forbidden_surfaces()`
- `assumptions()`
- `non_assumptions()`
- `residual_debt()`
- `phase_gates()`

If a later runtime or milestone depends on something not named in the relevant
readiness scope, that dependency is not part of the frozen contract yet.

## Anti-Patterns

- Do not treat these reports as prose-only closeout replacements.
- Do not use it to imply that downstream runtime lowering parity is already
  solved.
- Do not add inventory entries without matching evidence.
- Do not overstate the `worth-proof` lane beyond what the report names.
- Do not consume a Milestone 9 readiness artifact through a Milestone 5-named
  require helper.

## Current Limits

- These surfaces freeze machine-facing closure contracts.
- They do not replace runtime adoption work or Milestone 6+ ontology work.

The current residual debt is intentionally narrow:

- adopting-runtime parity is still deferred
- later diagnostics and deeper provenance ontology are still deferred
- strategy registries and execution engines are still runtime-owned
- full lineage support beyond transition rows is still deferred

Milestone 9 scoped merge debt is also explicit:

- adopting-crate scoped merge execution is still deferred
- native cherry-pick execution is still deferred
- runtime conflict materialization is still deferred

## Related Docs

- [Committed Authority Transitions](./committed-authority-transitions.md)
- [Scoped Merge And Cherry-Pick Vocabulary](../scoped-merge-adoption.md)
- [plans/worth-foundational/milestone-5-closeout.md](../../../../plans/worth-foundational/milestone-5-closeout.md)
- [plans/worth-foundational/milestone-9.md](../../../../plans/worth-foundational/milestone-9.md)
