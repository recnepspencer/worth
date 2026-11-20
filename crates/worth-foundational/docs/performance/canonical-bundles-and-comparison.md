# Canonical Bundles And Comparison

## What This Feature Is

This seam lowers descriptive claims into canonical lower-lane bundles that can
carry contract names, counter specs, supporting evidence rows, canonical-basis
entries, and precise mismatch explanation across independent producers.

## Why You Use It

- to compare performance meaning across producers without sharing storage layout
- to attach named contract and counter-spec semantics before execution rows
- to prepare digest-ready performance meaning through canonical basis lowering
- to fence out producer-private telemetry snapshots from the shared API

## Stable Entry Points

- `worth_foundational::performance_api::lower_lane::basis`
- `performance_bundle(claim)`
- `compare_performance_bundles(...)`
- `prepare_performance_bundle_for_canonical_basis(...)`
- `prepare_counter_backed_performance_receipt_for_canonical_basis(...)`
- `prepare_materialized_performance_report_for_canonical_basis(...)`

## Core Mental Model

A bundle is the shared lower-lane envelope for performance meaning before
executed rows are necessarily present.

- the claim still carries boundary, strength, postures, and work disclosure
- the bundle adds canonical lowering and shared attachment surfaces
- comparison happens on the bundle’s full meaning, not just one or two fields
- digest-ready lowering is about canonical basis participation, not proof
  certification

## How It Executes

1. Start from a finished common-lane claim.
2. Lower it through `performance_bundle(claim)`.
3. Attach zero or more contract names, counter specs, and supporting evidence
   rows.
4. Finish the bundle.
5. Compare bundles directly or prepare them for canonical basis / digest-ready
   lowering.

## Small Example

```rust
use worth_foundational::performance_api::lower_lane::basis;

let bundle = basis::performance_bundle(claim)
    .attach_contract_name(contract_name)?
    .attach_counter_spec(counter_spec)?
    .finish()?;
```

This is the smallest honest example because the claim has become a shared
bundle without pretending execution rows or stronger proof already exist.

## Real Example

```rust
use worth_foundational::performance_api::lower_lane::basis;

let left = basis::performance_bundle(left_claim)
    .attach_contract_name(contract_name.clone())?
    .attach_counter_spec(counter_spec_left)?
    .attach_supporting_evidence_row(support_row.clone())?
    .finish()?;

let right = basis::performance_bundle(right_claim)
    .attach_contract_name(contract_name)?
    .attach_counter_spec(counter_spec_right)?
    .attach_supporting_evidence_row(support_row)?
    .finish()?;

let comparison = basis::compare_performance_bundles(&left, &right);
```

What is authoritative:

- the claim meaning plus shared lower-lane attachments

What is derived:

- mismatch explanation and canonical-basis-ready representation

What gets retained:

- contract names, counter specs, and supporting evidence rows

What gets inspected:

- comparison mismatches
- canonical basis entries
- digest-ready lowering surfaces

## How It Relates To Other Features

- Start with
  [Common Performance Claims And Layout Intent](./common-performance-claims-and-layout-intent.md)
  for descriptive claim authoring.
- Move to
  [Counter-Backed Performance Receipts](./counter-backed-performance-receipts.md)
  when exact execution rows exist.
- Move to
  [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
  only when a stronger proof-bearing claim is real.

## Inspection And Debugging

Use these surfaces:

- `compare_performance_bundles(...)`
- `comparison.is_equivalent()`
- `comparison.mismatches()`
- `foundational_performance_canonical_basis_entries(...)`
- canonical-basis preparation helpers

Expect mismatches to stay precise:

- contract-name mismatches
- counter-spec mismatches
- breadth/locality mismatches
- access-pattern mismatches
- execution-temperature mismatches
- fallback/debt mismatches

## Anti-Patterns

- feeding raw telemetry bags into shared bundle APIs
- assuming canonical equivalence implies equal slope or equal planner quality
- skipping bundle lowering and jumping directly from claim to receipt-like APIs
- treating supporting evidence rows as a substitute for report materialization

## Current Limits

- this seam does not attach executed counter rows by itself
- canonical basis participation is not the same as proof-bearing certification
- elapsed time alone is not sufficient shared performance meaning

## Related Docs

- [Common Performance Claims And Layout Intent](./common-performance-claims-and-layout-intent.md)
- [Counter-Backed Performance Receipts](./counter-backed-performance-receipts.md)
- [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
