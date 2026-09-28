# Admission-Local Support Reports

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Feature Is

Admission-local support reports let a domain express support posture that
belongs to an admitted intent plan and materialize that support as a Query
support traceability report rather than an admission decision.

## Why You Use It

- you need runtime-facing support context without pretending it is an advisory
  or violation
- you want admitted-plan support to stay distinct from declaration-scoped
  support
- you need support rows that survive certification and debugging

## Stable Entry Points

Proof-facing authoring:

- `WorthQueryAdmissionContributionAuthoring::support_only(...)`

Raw-facing materializers:

- `materialize_runtime_admission_support_traceability_row(...)`
- `materialize_runtime_admission_support_traceability_report(...)`

The public ordinary lane for admission-local support is the admitted-plan
support category, not the ordinary admission verbs themselves.

## Core Mental Model

Admission-local support is still admission-adjacent, but it does not pretend to
be a decision outcome.

Use it when the domain wants to say:

- this admitted plan is supported by a specific runtime fact
- this admitted plan needs explicit support reporting

without collapsing that support into advisory or violation posture.

## How It Executes

1. author a support-only admission contribution against an admitted plan
2. progress it through the ordinary proof-bearing lifecycle
3. materialize a support row or support report
4. keep category identity distinct from declaration support

## Small Example

```rust
let requested = WorthQueryAdmissionContributionAuthoring::support_only(
    "topology.authority_floor",
    "the admitted split remains valid but still needs authority-floor traceability",
);
```

This is the smallest honest example because admission-local support currently
lands through a proof authoring step and the support materializer family rather
than a dedicated common-lane convenience verb.

## Real Example

```rust
let report = materialize_runtime_admission_support_traceability_report(
    ready_support_only_contribution,
)?;

let digest = report.decision_support_traceability_digest();
let rows = report.rows();
```

In a geometry kernel, this is the right surface when an admitted edge or face
operation remains valid but still needs support evidence about authority floor,
naming continuity, or topology substrate posture.

## How It Relates To Other Features

- Declaration-Scoped Support And Traceability
  is for declaration-bound support.
- Advisory And Violation Contributions
  is for decision posture.
- Continuity Contributions And Authoritative Successors
  often pairs with admitted-plan support when successor truth needs both
  support and continuity explanation.

## Inspection And Debugging

- check the support traceability digest to keep admission-local support distinct
  from declaration support
- inspect row posture rather than assuming support implies admission success

## Anti-Patterns

- encoding admission-local support as advisory just because it is easier to
  reach a common lane
- mixing declaration support and admitted-plan support in one artifact story
- teaching this feature as a substitute for admission decisions

## Current Limits

- the ordinary lane here is thinner than declaration support
- the first explicit step is still `support_only(...)`, not an ordinary
  admission convenience verb
- this is a support artifact family, not a canonical admission artifact family
- if you need a full decision, use advisory or violation instead

## Related Docs

- Declaration-Scoped Support And Traceability
- Advisory And Violation Contributions
- [Intent Admission](../../execution/intent-admission.md)
