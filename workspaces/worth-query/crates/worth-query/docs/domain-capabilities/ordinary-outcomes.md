# Ordinary Outcomes

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

`WorthQueryOrdinaryOutcome<T>` is the shared bound/stopped result vocabulary for
domain-capability operations that intentionally expose an outcome lane. It
preserves a successful artifact or a typed stop without inventing a local
status enum.

## Why You Use It

- keep success and typed stop posture in one inspectable result
- project a stopped operation into remediation guidance without flattening it
- share outcome handling across contribution, grouped, continuation, and
  Signal-compatibility orchestration

## Stable Entry Points

- `WorthQueryOrdinaryOutcome::{Bound, Stopped}`
- `WorthQueryOrdinaryOutcome::{bound, stop, into_bound}`
- `WorthQueryInstalledDomainDeclarationContext::recover_from_outcome(...)`
- `orchestrate_declaration_with_contributions_outcome(...)`
- `orchestrate_signal_compatibility_outcome(...)`

Declaration-entry convenience methods that return `Result` should keep their
typed error. Convert to an ordinary outcome only when the owning public API
explicitly exposes that lane.

Despite the method name, `recover_from_outcome(...)` derives descriptive
remediation guidance. It does not mint the receipt-bound
`WorthQueryRecoveryHandle` used by committed application-aftermath recovery.

## Core Mental Model

An ordinary outcome does not erase the boundary. `Bound` retains the public
artifact; `Stopped` retains the stop that explains why no artifact was bound.

## How It Executes

1. Execute through the installed declaration context.
2. Preserve the returned ordinary outcome.
3. Branch on `bound()` or `stop()` when the caller needs immediate handling.
4. Ask `recover_from_outcome(...)` for a typed remediation brief when the
   application needs next-step guidance.

## Small Example

```rust
let outcome = context.orchestrate_signal_compatibility_outcome(input);

if let Some(bound) = outcome.bound() {
    use_compatibility(bound);
}
```

## Real Example

```rust
let outcome = context.orchestrate_declaration_with_contributions_outcome(input);

match (outcome.bound(), context.recover_from_outcome(&outcome)) {
    (Some(bound), _) => publish(bound),
    (None, Some(brief)) => schedule_review(brief.recommended_action()),
    (None, None) => retain_for_inspection(outcome),
}
```

## How It Relates To Other Features

- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) derives descriptive next-step guidance.
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md) owns operational recovery after commit.
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
  exposes an ordinary outcome lane.
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
  exposes an ordinary outcome lane.

## Inspection And Debugging

Inspect the retained stop, linked artifact identities, and installed authority.
Do not infer the stop class from display text.

## Anti-Patterns

- converting the outcome to a boolean
- replacing typed stops with a local retry enum
- manufacturing a bound value from receipt or digest fields

## Current Limits

Not every domain-capability method returns `WorthQueryOrdinaryOutcome`; many
return a boundary-specific `Result` or checked enum. Preserve the type selected
by the owning API.

## Related Docs

- [Domain Capabilities](./README.md)
- [Runtime-Installed Domains](./runtime-installed-domains.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
- [Application Aftermath, External Effects, And Recovery](../execution/application-aftermath-and-recovery.md)
