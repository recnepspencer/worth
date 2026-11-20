# Grouped Public Lanes And Stronger Readiness

## What This Feature Is

This is the first-contact map for the shipped Milestone 8 public surface. It
shows which entrypoints belong to the common lane, lower lane, and stronger
lane, and what each grouped surface teaches or refuses to hide.

## Why You Use It

- to find the right entry lane without reading internal modules
- to understand which lane owns authoring, lowering, or stronger proof
- to verify that grouped public APIs do not blur authority boundaries
- to inspect the exact public inventory that readiness freezes

## Stable Entry Points

- `worth_foundational::performance_api::common_path`
- `worth_foundational::performance_api::lower_lane`
- `worth_foundational::performance_api::stronger_lane`
- `worth_foundational::performance_api::performance_public_surface_inventory()`

## Core Mental Model

The grouped lanes are intentionally asymmetric.

- `common_path` owns descriptive authoring and primitive legality
- `lower_lane` owns canonical lowering, receipts, comparison, and reports
- `stronger_lane` owns proof-bearing certified bundles and readiness

The grouped API is not a generic convenience umbrella. Each lane tells you what
it owns and what it refuses to hide.

## How It Executes

1. Start in `common_path` when you are authoring meaning.
2. Move to `lower_lane` when you need inspectable accountability surfaces.
3. Move to `stronger_lane::certified` only when a stronger proof claim is real.
4. Move to `stronger_lane::readiness` when you need the machine-checkable
   milestone closure artifact.

## Small Example

```rust
use worth_foundational::performance_api::performance_public_surface_inventory;

let inventory = performance_public_surface_inventory();
```

This is the smallest honest example because it shows the supported grouped map
without implying that all lanes do the same job.

## Real Example

```rust
use worth_foundational::performance_api::{
    common_path,
    lower_lane::{basis, receipts},
    stronger_lane::{certified, readiness},
};

let _authoring = common_path::performance();
let _bundle_builder = basis::performance_bundle(claim);
let _receipt_builder = receipts::counter_backed_performance_receipt(bundle);
let _readiness_report = readiness::foundational_performance_milestone8_readiness_report();
let _certified_authority =
    certified::foundational_performance_certified_attachment_authority();
```

What is authoritative:

- the lane boundary itself

What is derived:

- grouped inventory rows describing what each lane teaches

What gets retained:

- public lane classification and “does not hide” statements

What gets inspected:

- `entry.path()`
- `entry.lane()`
- `entry.teaches()`
- `entry.does_not_hide()`

## How It Relates To Other Features

- Use
  [Common Performance Claims And Layout Intent](./common-performance-claims-and-layout-intent.md)
  for common-path behavior.
- Use
  [Canonical Bundles And Comparison](./canonical-bundles-and-comparison.md),
  [Counter-Backed Performance Receipts](./counter-backed-performance-receipts.md),
  and
  [Performance Report Planning And Materialization](./performance-report-planning-and-materialization.md)
  for lower-lane behavior.
- Use
  [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
  and
  [Performance Production Readiness](./performance-production-readiness.md)
  for stronger-lane behavior.

## Inspection And Debugging

Use `performance_public_surface_inventory()` when:

- you need the exact public lane map
- you want to confirm a seam is grouped intentionally
- you are reviewing readiness or doc coverage drift

The strongest grouped-lane constraints are compile-fail tested: plain
lower-lane artifacts cannot enter grouped stronger certified APIs, and plain
readiness reports cannot stand in for proof-bearing readiness.

## Anti-Patterns

- treating `stronger_lane` as a convenience alias for lower-lane inspection
- skipping the grouped inventory and guessing which public seam owns a job
- documenting grouped lanes as if they were internal module names rather than
  supported public teaching surfaces

## Current Limits

- grouped lanes map the public surface; they do not replace the seam-specific
  docs
- readiness remains a separate stronger seam, not a property of every
  certified bundle
- grouped entrypoints are intentionally narrow and opinionated

## Related Docs

- [Common Performance Claims And Layout Intent](./common-performance-claims-and-layout-intent.md)
- [Certified And Readmitted Performance Bundles](./certified-and-readmitted-performance-bundles.md)
- [Performance Production Readiness](./performance-production-readiness.md)
