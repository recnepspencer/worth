# Diagnostic Production Readiness

## What This Feature Is

This feature is the machine-checkable closeout surface for the Milestone 6
diagnostics ontology.

It is not a human summary. It is a proof-bearing readiness artifact that names
exact certified surfaces, exact hostile pressures, exact compile-fail
boundaries, exact golden artifacts, exact property seeds, exact harness
expansion points, and exact downstream adoption pressures.

## Why You Use It

Use this surface when you need to:

- verify what the milestone actually certifies
- inspect the exact proof-lane dependency boundary
- see what downstream runtimes may and may not assume
- understand what hostile pressures still matter during adoption

If you are migrating another crate onto this diagnostics ontology, this is the
artifact that tells you what is already frozen and what is still deferred.

## Stable Entry Points

- `certify_foundational_diagnostic_milestone6_production_test_readiness()`
- `require_foundational_diagnostic_milestone6_production_test_readiness(...)`
- `foundational_diagnostic_milestone6_readiness_report()`

Important types:

- `FoundationalDiagnosticProductionTestReadyArtifact`
- `FoundationalDiagnosticProductionReadinessReport`
- `FoundationalDiagnosticCertifiedSurface`
- `FoundationalDiagnosticSyntheticRuntimePressure`
- `FoundationalDiagnosticCompileFailBoundary`
- `FoundationalDiagnosticCanonicalGoldenArtifact`
- `FoundationalDiagnosticPropertySeed`
- `FoundationalDiagnosticHarnessExpansionPoint`
- `FoundationalDiagnosticRuntimeAdoptionFailurePressure`
- `FoundationalDiagnosticAdoptionShapedFollowthrough`

## Core Mental Model

Readiness is exact or it is not readiness.

This artifact is designed to answer questions like:

- Which surfaces are certified?
- Which hostile pressures are owned locally?
- Which compile-fail boundaries are part of the contract?
- Which golden artifacts and property seeds are frozen?
- Which `worth-proof` APIs are actually part of the shipped stronger lane?
- What remains deferred for adoption or later milestones?

It is meant to stop later work from depending on code archaeology.

## How It Executes

The readiness certification path:

1. builds an exact report inventory
2. binds each certified surface to evidence
3. binds each hostile pressure to evidence
4. binds each compile-fail boundary to evidence
5. binds golden artifacts, property seeds, and harness expansion points to
   evidence
6. certifies the whole closure contract through the readiness authority lane

The result is a proof-bearing artifact plus a report you can inspect directly.

## Small Example

```rust
use worth_foundational::{
    certify_foundational_diagnostic_milestone6_production_test_readiness,
    require_foundational_diagnostic_milestone6_production_test_readiness,
};

let artifact = certify_foundational_diagnostic_milestone6_production_test_readiness();
let report = require_foundational_diagnostic_milestone6_production_test_readiness(&artifact);

assert!(report.passes_readiness_checklist());
```

## Real Example

Use the report to drive adoption planning:

```rust
use worth_foundational::foundational_diagnostic_milestone6_readiness_report;

let report = foundational_diagnostic_milestone6_readiness_report();

let certified_surfaces = report.certified_surfaces();
let hostile_pressures = report.synthetic_pressures();
let compile_fail_boundaries = report.compile_fail_boundaries();
let adoption_pressures = report.runtime_adoption_failure_pressures();
let followthrough = report.adoption_shaped_followthrough();

let _ = (
    certified_surfaces,
    hostile_pressures,
    compile_fail_boundaries,
    adoption_pressures,
    followthrough,
);
```

## How It Relates To Other Features

- [Certified Diagnostic Bundles And Attachments](./certified-diagnostic-bundles-and-attachments.md)
  is the strongest runtime-facing diagnostics lane this report certifies.
- Every other diagnostics feature doc in this folder describes a surface whose
  exact closure status is frozen here.

## Inspection And Debugging

Check these first:

- `report.certified_surfaces()`
- `report.certified_surface_evidence()`
- `report.synthetic_pressures()`
- `report.compile_fail_boundaries()`
- `report.canonical_golden_artifacts()`
- `report.property_seed_inventory()`
- `report.harness_expansion_points()`
- `report.worth_proof_api_appendix()`
- `report.residual_debt()`

If a downstream crate wants to assume something that is not named here, that
assumption is not part of the frozen contract yet.

## Anti-Patterns

- Do not treat the readiness artifact as a prose closeout replacement.
- Do not add loose inventory rows without matching evidence.
- Do not use this artifact to imply that deferred adoption work is already
  done.
- Do not claim a stronger `worth-proof` dependency lane than the report
  actually names.

## Current Limits

- This artifact freezes the machine-facing closure contract.
- It does not replace crate-facing feature docs, runtime adoption work, or
  Milestone 7 provenance and receipt deepening.

## Related Docs

- [Certified Diagnostic Bundles And Attachments](./certified-diagnostic-bundles-and-attachments.md)
- [plans/worth-foundational/milestone-6-closeout.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/worktree_3/plans/worth-foundational/milestone-6-closeout.md)
