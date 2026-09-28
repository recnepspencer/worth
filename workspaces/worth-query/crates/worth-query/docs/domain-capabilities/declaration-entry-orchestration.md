# Declaration Entry Orchestration

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration entry orchestration turns one domain declaration into the next
public, proof-bearing artifact without asking the consumer to assemble internal
phase objects. It is available on
`WorthQueryInstalledDomainDeclarationContext<D, C>`.

## Why You Use It

- keep the installed domain, runtime generation, and operating context attached
  to every artifact
- select the narrowest terminal artifact needed by the next consumer
- preserve typed denials instead of flattening readiness, planning, receipt, or
  envelope failures

## Stable Entry Points

- `declare_and_review(...)`
- `declare_review_and_progress(...)`
- `declare_review_progress_describe_and_plan(...)`
- `declare_review_progress_describe_plan_and_receipt(...)`
- `declare_review_progress_describe_plan_receipt_and_envelope(...)`
- `declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(...)`
- `declare_review_progress_describe_plan_receipt_envelope_and_route_bridge_continuation(...)`
- `declare_review_progress_describe_plan_receipt_envelope_and_check_signal_compatibility(...)`

Use the shortest method that returns the artifact the next boundary actually
requires.

## Core Mental Model

The installed declaration context is the authority root. Each public method
advances the same retained declaration story and returns a typed result for one
specific boundary.

```text
installed declaration context
-> declared and reviewed meaning
-> admitted progression
-> route plan
-> boundary receipt
-> boundary envelope
-> optional relational, bridge, or Signal compatibility artifact
```

The intermediate types remain inspectable, but consumers do not reconstruct
the pipeline or call internal orchestration helpers.

## How It Executes

1. The installed context seals the domain package, runtime generation, and
   operating-context identity.
2. Query validates and admits the declaration family.
3. Query derives foundational evidence and a route plan.
4. Query issues a receipt and, when requested, a boundary envelope.
5. A specialized terminal method may route that envelope to relational truth,
   bridge continuation, or Signal compatibility.

Every failure is returned at the boundary that owns it. A successful earlier
artifact never implies that a later boundary admitted.

## Small Example

```rust
let envelope = context
    .declare_review_progress_describe_plan_receipt_and_envelope(request)?;
```

## Real Example

```rust
let routing = context
    .declare_review_progress_describe_plan_receipt_envelope_and_route_relational_truth(
        request,
    )?;

inspect_routing(routing);
```

This path retains installed authority through the terminal routing artifact.

## How It Relates To Other Features

- [Runtime-Installed Domains](./runtime-installed-domains.md) creates the
  installed handle and declaration context.
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md) explains
  receipt authority.
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md) explains
  the retained crossing artifact.
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md) maps typed terminal artifacts to
  a recovery brief.

## Inspection And Debugging

Inspect the returned artifact or typed error at the boundary where execution
stopped. For runtime products, use `workspace.inspections()` and the owning
capability's inspection declaration. Do not treat diagnostic digests as a
replacement for the retained artifact.

## Anti-Patterns

- rebuilding route, receipt, or envelope transitions in a downstream crate
- advancing farther than the next consumer requires
- using a receipt or digest as executable authority
- retrying a later boundary without retaining the earlier typed artifact

## Current Limits

- Specialized terminal methods are available only for declaration families
  whose marker traits admit that boundary.
- Store-backed continuation and durability remain governed by the active
  support profile.

## Related Docs

- [Domain Capabilities](./README.md)
- [Typed Binding Pipeline](./typed-binding-pipeline.md)
- [Lower-Runtime Capability Routing](./lower-runtime-capability-routing.md)
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
