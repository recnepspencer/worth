# Policy Admission Receipts

## What This Feature Is

This is the lower-lane seam for runtime policy outcomes before execution. A
policy-admission receipt says what path was admitted, widened, deferred,
rejected, or marked as debt, along with the budget and work disclosure that
made that result true.

## Why You Use It

- to keep runtime policy separate from executed cost truth
- to record width, locality, density, or freshness-sensitive admission budgets
- to show exactly what work was admitted, widened, or denied before execution
- to make “verified”, “widened”, “rejected”, and “deferred” mechanically
  distinct outcomes

## Stable Entry Points

- `worth_foundational::performance_api::lower_lane::policy`
- `foundational_performance_budget_definitions()`
- `policy_admission_receipt(claim)`
- `FoundationalPolicyAdmissionReceipt`

Good to know:

- this seam is lower-lane and shipped
- it requires a policy-admission claim from the common lane
- it still does not claim that execution happened

## Core Mental Model

Authoritative truth here is still the policy result, not execution.

- the claim describes the intended performance boundary and posture
- the receipt adds budget decisions and runtime admission outcome
- the receipt also says what stronger evidence is still missing:
  `CounterBackedExecutionReceipt`

The handle you hold is an inspectable pre-execution artifact. It is stronger
than a compile-time contract and weaker than a counter-backed receipt.

## How It Executes

1. Author a policy-admission claim in the common lane.
2. Lower it through `policy_admission_receipt(claim)`.
3. Attach budget decisions for the inspected kinds.
4. Optionally disclose widened work or denied work when the result requires it.
5. Finish the receipt and inspect the exact policy outcome.

The builder fail-closes if the policy story becomes dishonest.

## Small Example

```rust
use worth_foundational::performance_api::common_path as performance;
use worth_foundational::performance_api::lower_lane::policy;

let claim = performance::performance()
    .claim()
    .policy_admission()
    .boundary_authoritative_execution()
    .runtime_policy_admission()
    .hot_path()
    .delta_bound()
    .exact_basis_current()
    .verified()
    .include_validation_planning_work()
    .exclude_support_report_assembly_work()
    .finish()?;

let receipt = policy::policy_admission_receipt(claim)
    .budget_decision(
        worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
        8,
        8,
    )
    .finish()?;
```

This is the smallest honest example because it shows a policy result without
pretending execution rows already exist.

## Real Example

```rust
use worth_foundational::performance_api::common_path as performance;
use worth_foundational::performance_api::lower_lane::policy;

let claim = performance::performance()
    .claim()
    .policy_admission()
    .boundary_maintenance_planning()
    .runtime_policy_admission()
    .basis_local_batch()
    .density_adaptive()
    .warm_path()
    .historical_retained()
    .widened_with_explicit_disclosure()
    .include_validation_planning_work()
    .exclude_support_report_assembly_work()
    .finish()?;

let receipt = policy::policy_admission_receipt(claim)
    .budget_decision(
        worth_foundational::FoundationalPerformanceBudgetKind::Density,
        4,
        6,
    )
    .widen_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
    .finish()?;
```

What is authoritative:

- the admission outcome and budget disclosure

What is derived:

- nothing replay-like or report-like yet

What gets retained:

- exact budget decisions and widened/denied work disclosure

What gets inspected:

- `receipt.stronger_evidence_still_required()`
- `receipt.budget_decisions()`
- `receipt.widened_work()`
- `receipt.denied_work()`

## How It Relates To Other Features

- Start from
  [Common Performance Claims And Layout Intent](./common-performance-claims-and-layout-intent.md)
  to author the claim honestly.
- Move to
  [Counter-Backed Performance Receipts](./counter-backed-performance-receipts.md)
  only when execution has really happened.
- Use
  [Performance Report Planning And Materialization](./performance-report-planning-and-materialization.md)
  if you need to widen a policy receipt into an explicit report surface.

## Inspection And Debugging

Read these surfaces first:

- `receipt.evidence_strength()`
- `receipt.stronger_evidence_still_required()`
- `receipt.budget_decisions()`
- `receipt.included_work()`
- `receipt.excluded_work()`
- `receipt.widened_work()`
- `receipt.denied_work()`

Common denial signals:

- compile-time contracts cannot lower into policy receipts
- verified, deferred, and debt receipts cannot silently widen budget
- rejected receipts must disclose denied work
- the same work class cannot be both widened and denied

## Anti-Patterns

- treating a policy receipt as executed evidence
- lowering compile-time contracts directly into policy receipts
- widening budget under a “verified” receipt
- hiding denied or widened work behind the same narrow claim surface

## Current Limits

- this seam still does not attach exact execution rows
- policy admission is about runtime permission and budget, not canonical digest
  comparison
- stronger proof-bearing certification starts only after lower-lane execution
  or report artifacts exist

## Related Docs

- [Common Performance Claims And Layout Intent](./common-performance-claims-and-layout-intent.md)
- [Counter-Backed Performance Receipts](./counter-backed-performance-receipts.md)
- [Performance Report Planning And Materialization](./performance-report-planning-and-materialization.md)
