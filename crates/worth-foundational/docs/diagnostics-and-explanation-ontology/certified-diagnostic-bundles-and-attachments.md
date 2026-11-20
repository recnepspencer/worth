# Certified Diagnostic Bundles And Attachments

## What This Feature Is

This feature is the stronger diagnostics lane.

It lets you take a descriptive diagnostic support report or explanation bundle
and attach it to a stronger current-basis source with typed hostile coverage,
typed coverage matrices, and the existing `worth-proof` artifact lane.

Use it only when plain descriptive diagnostics are not enough.

## Why You Use It

Use this surface when you need to:

- claim hostile coverage instead of descriptive-only support
- attach diagnostics to a current-basis transition or boundary-artifact source
- preserve source digest and provenance-hook meaning
- cross a trust boundary and readmit a certified bundle honestly

Do not use this surface just to make ordinary diagnostics look fancier.

## Stable Entry Points

- `certify_current_basis_diagnostic_bundle(...)`
- `certify_diagnostic_bundle_with_source_basis(...)`
- `bridge_certified_diagnostic_bundle_trust_boundary(...)`
- `readmit_certified_diagnostic_bundle_after_boundary(...)`
- `foundational_diagnostic_certified_attachment_authority()`
- `foundational_diagnostic_certified_readmission_authority()`

Important types:

- `FoundationalCertifiedDiagnosticBundle<Source, Bundle>`
- `FoundationalDiagnosticCoverageMatrix`
- `FoundationalDiagnosticCertifiedCoverageClass`
- `FoundationalDiagnosticCoverageFamilyStatus`
- `FoundationalCertifiedDiagnosticSource`
- `FoundationalCertifiedDiagnosticSourceKind`
- `FoundationalCertifiedDiagnosticProvenanceHook`

## Core Mental Model

Certified diagnostics are still diagnostics. They do not become receipts or
authority artifacts.

What changes is the strength of the claim:

- the source is a stronger current-basis or proof-bearing surface
- the source digest is explicit
- the coverage matrix is explicit
- the coverage class is explicit
- the stronger lane reuses `worth-proof` instead of local booleans

That means you can say “this diagnostic bundle is attached to this exact source
under this exact hostile coverage posture” without inventing another proof
system.

## How It Executes

You need:

- a current-basis source that implements the sealed certified-source trait
- a support report or explanation bundle that already has honest descriptive
  meaning
- a coverage matrix that says which row families are hostile-covered, partial,
  absent, or denied
- the attachment authority witness

The certification call then:

1. derives or validates the source digest
2. prepares the bundle for canonical basis
3. validates the coverage matrix against the actual row families and named gaps
4. attaches the bundle through `worth-proof::Artifact`

## Small Example

```rust
use worth_foundational::{
    certify_current_basis_diagnostic_bundle,
    foundational_diagnostic_certified_attachment_authority,
};

let certified = certify_current_basis_diagnostic_bundle(
    version,
    current_basis_receipt,
    support_report,
    coverage_matrix,
    foundational_diagnostic_certified_attachment_authority(),
)?;
```

## Real Example

Use trust-boundary bridge and readmission when the stronger bundle crosses a
boundary:

```rust
use worth_foundational::{
    bridge_certified_diagnostic_bundle_trust_boundary,
    foundational_diagnostic_certified_readmission_authority,
    readmit_certified_diagnostic_bundle_after_boundary,
};

let bridged = bridge_certified_diagnostic_bundle_trust_boundary(certified);

let readmitted = readmit_certified_diagnostic_bundle_after_boundary(
    bridged,
    rebound_basis,
    foundational_diagnostic_certified_readmission_authority(),
);
```

## How It Relates To Other Features

- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
  must already be honest before you certify anything.
- [Diagnostic Canonical Basis And Comparison](./diagnostic-canonical-basis-and-comparison.md)
  provides the canonical bundle truth this layer depends on.
- [Diagnostic Production Readiness](./diagnostic-production-readiness.md)
  freezes the exact coverage, proof-lane, and hostile-pressure contract for
  this stronger lane.

## Inspection And Debugging

Inspect these first:

- `certified.coverage_class()`
- `certified.coverage_matrix()`
- `certified.source_kind()`
- `certified.source_digest()`
- `certified.provenance_hook()`
- `certified.strong_basis()`
- `certified.proofs()`

If certification is denied, the most common reasons are:

- missing source digest
- happy-path-only coverage
- a family marked as covered but absent from the bundle
- partial coverage with a named gap that does not belong to the bundle

## Anti-Patterns

- Do not fabricate source digests.
- Do not certify a bundle with only happy-path rows and call it hostile
  coverage.
- Do not treat partial coverage as hostile coverage when the bundle still has
  named gaps.
- Do not build a second local proof lane for diagnostics.

## Current Limits

- This lane is about stronger diagnostic claims, not about redefining
  transition authority or receipts.
- It only works with approved current-basis sources and bundle surfaces that
  satisfy the sealed attachment contracts.

## Related Docs

- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
- [Diagnostic Production Readiness](./diagnostic-production-readiness.md)
