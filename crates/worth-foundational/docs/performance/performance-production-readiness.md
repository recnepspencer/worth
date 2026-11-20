# Performance Production Readiness

## What This Feature Is

This is the proof-bearing closure artifact for the shipped Milestone 8
performance surface. It freezes exactly what is certified, what is hostilely
proved, what public lanes exist, what docs cover them, and what explicit
non-goals or runtime assumptions still bound the current closure.

## Why You Use It

- to verify the Milestone 8 surface is frozen enough for migration planning
- to detect drift in certified surfaces, compile-fail boundaries, and docs
- to inspect runtime assumptions, non-assumptions, and adoption-pressure proof
- to distinguish shipped closure from runtime assumptions and non-goals

## Stable Entry Points

- `worth_foundational::performance_api::stronger_lane::readiness`
- `foundational_performance_milestone8_readiness_report()`
- `certify_foundational_performance_milestone8_production_test_readiness()`
- `require_foundational_performance_milestone8_production_test_readiness(...)`
- `FoundationalPerformanceProductionReadinessReport`
- `FoundationalPerformanceProductionTestReadyArtifact`

## Core Mental Model

This seam is not ordinary feature usage. It is the machine-checkable statement
of what the performance feature set really ships today.

- the plain report is the inspectable closure artifact
- the certified artifact is the proof-bearing stronger form
- exact inventory matters more than broad “contains these ideas” compliance
- if a seam is not frozen here, it is not part of the shipped closure claim

## How It Executes

1. Build the readiness report.
2. Inspect certified surfaces, pressures, compile-fail boundaries, public
   inventory, doc inventory, runtime assumptions, and runtime non-assumptions.
3. Certify the production-test readiness artifact when the stronger proof claim
   is required.
4. Require that proof-bearing artifact at the boundary that needs frozen
   closure.

## Small Example

```rust
use worth_foundational::performance_api::stronger_lane::readiness;

let report = readiness::foundational_performance_milestone8_readiness_report();
assert!(report.passes_readiness_checklist());
```

This is the smallest honest example because it inspects closure without yet
claiming stronger proof-bearing readiness.

## Real Example

```rust
use worth_foundational::performance_api::stronger_lane::readiness;

let artifact =
    readiness::certify_foundational_performance_milestone8_production_test_readiness();

let report =
    readiness::require_foundational_performance_milestone8_production_test_readiness(&artifact);

assert!(report.passes_readiness_checklist());
```

What is authoritative:

- the readiness report inventory itself

What is derived:

- the stronger proof-bearing wrapper around that report

What gets retained:

- certified surfaces
- hostile pressures
- compile-fail boundaries
- runtime adoption pressures and evidence
- public surface and documentation coverage
- runtime assumptions and non-assumptions

## How It Relates To Other Features

- Use
  [Grouped Public Lanes And Stronger Readiness](./grouped-public-lanes-and-stronger-readiness.md)
  when you need the first-contact map for the public surface.
- Use
  [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
  for ordinary stronger-lane runtime artifacts; readiness is a separate closure
  seam.
- Use the seam-specific docs when you need how-to guidance rather than closure
  inspection.

## Inspection And Debugging

These are the first surfaces to inspect:

- `report.certified_surfaces()`
- `report.synthetic_pressures()`
- `report.compile_fail_boundaries()`
- `report.runtime_adoption_pressures()`
- `report.documentation_surface_inventory()`
- `report.public_surface_documentation_coverage()`
- `report.assumptions()`
- `report.non_assumptions()`

If the checklist fails, treat that as closure drift, not as an ordinary feature
bug.

## Anti-Patterns

- using readiness as a substitute for seam-specific docs
- assuming “passes checklist” means downstream crates are already migrated
- letting grouped public surfaces drift without updating readiness inventory
  and doc coverage

## Current Limits

- readiness freezes the current shipped surface; it does not promise future
  workspace-wide migration results
- this seam is for closure and proof, not day-to-day claim authoring

## Related Docs

- [Grouped Public Lanes And Stronger Readiness](./grouped-public-lanes-and-stronger-readiness.md)
- [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
- [Performance, Layout, And Enforcement Vocabulary](./README.md)
