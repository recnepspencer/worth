# Diagnostic Canonical Basis And Comparison

## What This Feature Is

This feature gives diagnostic support reports and explanation bundles a
canonical basis and a comparison surface.

It lets independent producers describe the same diagnostic meaning one way and
lets blind consumers compare bundles without relying on input order, debug
strings, or producer-private folklore.

## Why You Use It

Use this surface when you need to:

- prepare an explanation bundle or support report for canonical comparison
- compare two independently produced diagnostic bundles
- preserve explicit mismatch basis instead of flattening comparison to `bool`
- ensure named gaps, evidence posture, and row-family meaning participate in
  canonical identity

If you care about parity, replay, export, or cross-runtime adoption, this
surface matters.

## Stable Entry Points

- `prepare_diagnostic_support_report_for_canonical_basis(...)`
- `prepare_diagnostic_explanation_bundle_for_canonical_basis(...)`
- `compare_diagnostic_support_reports(...)`
- `compare_diagnostic_explanation_bundles(...)`
- `foundational_diagnostic_canonical_basis_entries(...)`

Key types:

- `FoundationalDiagnosticComparisonBundle`
- `FoundationalDiagnosticComparisonDenial`

## Core Mental Model

Canonical comparison does not compare presentation details. It compares meaning.

That means the canonical basis includes:

- row family
- row-specific semantic payload
- locator meaning
- named-gap meaning
- evidence-posture meaning
- fallback-debt meaning where it is part of the bundle truth

If two bundles only differ because the producer inserted rows in a different
order, canonical comparison should still treat them as equal.

## How It Executes

You start from a materialized support report or explanation bundle.

Then you:

1. prepare it for canonical basis
2. compare two canonicalized surfaces
3. inspect the comparison bundle for explicit mismatch meaning

Comparison never needs to guess where the mismatch happened. It carries mismatch
basis explicitly.

## Small Example

```rust
use worth_foundational::{
    compare_diagnostic_support_reports,
    prepare_diagnostic_support_report_for_canonical_basis,
};

let left_ready = prepare_diagnostic_support_report_for_canonical_basis(version.clone(), &left)?;
let right_ready = prepare_diagnostic_support_report_for_canonical_basis(version, &right)?;

let comparison = compare_diagnostic_support_reports(&left_ready, &right_ready)?;
```

## Real Example

Use this lane when you need to compare partial bundles across producers:

```rust
use worth_foundational::{
    compare_diagnostic_explanation_bundles,
    prepare_diagnostic_explanation_bundle_for_canonical_basis,
};

let expected = prepare_diagnostic_explanation_bundle_for_canonical_basis(
    version.clone(),
    &expected_bundle,
)?;
let actual = prepare_diagnostic_explanation_bundle_for_canonical_basis(
    version,
    &actual_bundle,
)?;

let comparison = compare_diagnostic_explanation_bundles(&expected, &actual)?;

if let Some(mismatch) = comparison.mismatch_basis() {
    // Inspect the exact semantic mismatch rather than falling back to
    // "not equal".
    let _ = mismatch;
}
```

## How It Relates To Other Features

- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
  produces the bundles this layer consumes.
- [Certified Diagnostic Bundles And Attachments](./certified-diagnostic-bundles-and-attachments.md)
  depends on canonical bundle truth before it can make stronger claims.

## Inspection And Debugging

When canonical comparison surprises you, check:

- whether the mismatch is row-family-specific instead of just value-specific
- whether named gaps differ
- whether evidence posture differs
- whether the bundles differ semantically even if the rendered rows “look
  similar”

Also check whether the bundle you are comparing is partial. Partial bundles can
still compare meaningfully, but their gaps are part of the comparison story.

## Anti-Patterns

- Do not compare rendered markdown or debug strings.
- Do not compare bundles before canonical preparation.
- Do not drop mismatch basis and replace it with a single equality flag.
- Do not assume two rows with the same common fields are equal if their
  row-family-specific payload differs.

## Current Limits

- This layer compares diagnostic meaning. It does not certify hostile coverage
  or current-basis attachment by itself.
- If you need stronger claims about where a bundle came from, use certified
  attachments on top of this layer instead of overloading comparison.

## Related Docs

- [Diagnostic Materialization And Support Reports](./diagnostic-materialization-and-support-reports.md)
- [Certified Diagnostic Bundles And Attachments](./certified-diagnostic-bundles-and-attachments.md)
