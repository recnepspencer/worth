# Single Declaration To Envelope

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Workflow Is

This workflow advances one installed-domain declaration to a retained boundary
envelope through the public declaration context.

## Why You Use It

- obtain one Query-owned crossing artifact from one domain declaration
- keep installed authority, operating context, route evidence, and receipt
  evidence together
- avoid assembling intermediate transition artifacts in the consumer

## Stable Entry Points

- `WorthQueryInstalledDomainDeclarationContext::declare_review_progress_describe_plan_receipt_and_envelope(...)`
- `WorthQueryInstalledDomainDeclarationContext::envelope_routes_from_progressed(...)`

## Core Mental Model

Use the full convenience method when you hold declaration input. Use
`envelope_routes_from_progressed(...)` only when you already hold an admitted
progression from the same installed declaration context.

## How It Executes

```text
installed declaration context
-> declaration input
-> admitted progression
-> route plan
-> boundary receipt
-> boundary envelope
```

## Small Example

```rust
let envelope = context
    .declare_review_progress_describe_plan_receipt_and_envelope(request)?;
```

## Real Example

```rust
let progressed = context.declare_review_and_progress(request)?;
let envelope = context.envelope_routes_from_progressed(progressed)?;
continue_from(envelope);
```

## How It Relates To Other Features

- [Declaration Entry Orchestration](../declaration-entry-orchestration.md)
  describes the complete public ladder.
- [Declaration Boundary Envelopes](../declaration-boundary-envelopes.md)
  describes envelope authority.
- [Envelope To Signal Or Continuation](./envelope-to-signal-or-continuation.md)
  chooses the next boundary.

## Inspection And Debugging

Inspect the typed error at the stage that stopped. A route plan or receipt does
not imply envelope admission.

## Anti-Patterns

- building an envelope from caller-owned ids or digests
- replaying intermediate phases locally
- advancing to a lower-runtime boundary when the caller only needs the envelope

## Current Limits

The declaration family and active support profile decide which terminal
boundaries may consume the envelope.

## Related Docs

- [Runtime-Installed Domains](../runtime-installed-domains.md)
- [Typed Stops And Remediation Guidance](../typed-stops-and-remediation-guidance.md)
