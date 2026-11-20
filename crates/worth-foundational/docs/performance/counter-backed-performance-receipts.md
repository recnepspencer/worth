# Counter-Backed Performance Receipts

## What This Feature Is

This seam records real executed work by attaching exact structural counter rows
to a canonical performance bundle. It is the first lower-lane artifact that can
honestly say execution happened.

## Why You Use It

- to publish exact structural work that really occurred
- to prove a path is stronger than runtime policy admission
- to feed explicit executed evidence into reports or stronger certification
- to fail closed when expected counter rows are missing, duplicated, or wrong

## Stable Entry Points

- `worth_foundational::performance_api::lower_lane::receipts`
- `counter_backed_performance_receipt(bundle)`
- `FoundationalCounterBackedPerformanceReceipt`
- `FoundationalPerformanceCounterRow`

## Core Mental Model

The receipt is the executed-evidence seam.

- the bundle already carries the canonical lower-lane meaning
- the receipt adds exact observed rows for the declared counter specs
- this is still weaker than proof-bearing certification
- this seam does not silently widen into support or report assembly

## How It Executes

1. Build a canonical bundle first.
2. Lower it through `counter_backed_performance_receipt(bundle)`.
3. Attach rows for the declared counter specs.
4. Finish the receipt and inspect real executed evidence.

The receipt builder rejects:

- missing rows
- duplicate rows
- unexpected rows
- mismatched counts

## Small Example

```rust
use worth_foundational::performance_api::lower_lane::receipts;

let receipt = receipts::counter_backed_performance_receipt(bundle)
    .attach_counter_row(counter_row)?
    .finish()?;
```

This is the smallest honest example because the executed evidence arrives only
after bundle lowering and only through declared structural rows.

## Real Example

```rust
use worth_foundational::performance_api::lower_lane::basis;
use worth_foundational::performance_api::lower_lane::receipts;

let bundle = basis::performance_bundle(claim)
    .attach_counter_spec(counter_spec)?
    .finish()?;

let receipt = receipts::counter_backed_performance_receipt(bundle)
    .attach_counter_row(
        worth_foundational::FoundationalPerformanceCounterRow::new(counter_name, 3),
    )
    .finish()?;
```

What is authoritative:

- the executed structural row set

What is derived:

- later reports or stronger proof-bearing bundles

What gets retained:

- exact row values aligned with the counter spec

What gets inspected:

- `receipt.counter_rows()`
- `receipt.counter_specs()`
- the underlying bundle meaning

## How It Relates To Other Features

- Use
  [Canonical Bundles And Comparison](./canonical-bundles-and-comparison.md)
  first to define the shared lower-lane bundle.
- Use
  [Performance Report Planning And Materialization](./performance-report-planning-and-materialization.md)
  if you need broader visibility than direct receipt inspection.
- Use
  [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
  if the receipt needs stronger proof-bearing certification.

## Inspection And Debugging

Check these first:

- `receipt.counter_rows()`
- the bundle’s attached counter specs
- counter row names and expected values

If construction fails, expect a precise receipt denial rather than a vague
execution error.

## Anti-Patterns

- treating a policy receipt as if it were executed evidence
- attaching raw counter bags without bundle lowering
- using this seam as a support/report materialization shortcut
- assuming a receipt is already proof-bearing certification

## Current Limits

- this seam only proves exact structural row attachment, not stronger proof
- it does not include support rows unless you explicitly widen into a report
- it does not replace canonical bundle comparison or digest-ready lowering

## Related Docs

- [Canonical Bundles And Comparison](./canonical-bundles-and-comparison.md)
- [Performance Report Planning And Materialization](./performance-report-planning-and-materialization.md)
- [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
