# Retain Query source commands across HTTP replies

## What This Feature Is

`mount_query_source(adapter, limits)` installs bounded, process-local transport
custody for one declared no-output Query action. Server retains the actual
native operation returned by Query. It does not implement another mutation,
recovery, signing or workflow engine.

## Why You Use It

Use it when a caller must inspect or continue the same source command after a
lost reply, a provider refusal or an interrupted publication. A second supplier
implements preparation and installs another adapter, not another queue/worker.

## Stable Entry Points

- `WorthServerQuerySourceAdapter`: typed input/preparation, installed program,
  existing authentication, metadata and safe domain diagnostic correspondence.
- `mount_query_source` returns registration plus `WorthServerSourceContinuation`.
- `WorthServerSourceCommand<Input>` and `WorthServerSourceReply` are the actual
  Serde/JSON-schema protocol, with result version 3.
- `WorthQuerySourceOperationBinding<Schema>` is Query's sealed implementation
  interface. It allows generic hosts to call the ordinary native owner's
  `advance` and `assess_current_authorization` directly. It grants no permission.

The adapter supplies Schema, Program, Binding, Intent, Action, Delivery, Input
and Preparation types. Binding and Action are real Query declarations; Delivery
uses `ApplicationSourceCompletion`. `prepare` returns the actual
`WorthServerSourceOperation<Self>` or the original preparation on refusal.
Server, not a supplier callback, calls native advancement and inspection.

## Core Mental Model

The custody ID addresses a retained owner. It is not authorization, a replay key
or a serialized recovery handle. The independent logical key belongs to the
original request. Neither control commands nor retries can replace its input,
source, key or installed runtime.

The adapter authenticates the existing admitted transport caller at every call.
Query checks current operation permission before retained status is disclosed.
Authorization refusal retains custody and returns `DisclosureRefused`, not an
invented no-effect result. A preparation without a native authorization
observation is also undisclosed. A preparation may expose safe status only
through a genuine Query original-input assessment.

## How It Executes

Create is rejected for duplicate custody/key, quiescence or capacity before
input preparation. The bounded ingress worker prepares once, advances Query,
and stores the returned native owner before replying. Inspect is read-only.
Advance and Close require the exact next sequence; an old sequence runs nothing.
No control automatically retries stale or uncertain business work.

Queue rejection is `NotExecuted`. Once accepted, a caller deadline or lost reply
can produce `OutcomeUnknown`; reconcile the original custody ID. Do not infer
rollback or start a replacement mutation. Native completed/terminal owners
retain their distinct evidence and refuse unlawful cleanup without disposal.

## Small Example

```rust
use worth_server::{
    mount_query_source, WorthServerQuerySourceAdapter, WorthServerSourceCustodyLimits,
    WorthServerSourceCustodyMount,
};
use std::num::NonZeroUsize;

fn install<C: WorthServerQuerySourceAdapter>(adapter: C)
    -> std::io::Result<WorthServerSourceCustodyMount>
{
    mount_query_source(adapter, WorthServerSourceCustodyLimits {
        live_operations: NonZeroUsize::new(32).unwrap(),
        queued_commands: NonZeroUsize::new(8).unwrap(),
    })
}
```

Retain `mount.continuation` before installing `mount.registration` and accepting
requests. The declaration attaches the actual program readiness and native
binding identity; the adapter cannot substitute another operation name.

## Real Example

A browser deliberately creates one command and loses its first HTTP response:

```text
Create { custody_id: "draft-a", logical_key: "request-a", intent: <typed input> }
Inspect { custody_id: "draft-a" }
    -> Held { next_sequence: 2, native: Completed { ... } }
Close { custody_id: "draft-a", sequence: 2 }
    -> Closed
```

Create's input follows the installed adapter's generated schema. Open a fresh
product session and use its actual basis/Server instance for each call. Keep
the source owner until any downstream locator/persistence step has succeeded.
If that step fails, reuse the same completed source rather than creating again.
The source query/receipt establishes what landed; the custody ID alone does not.

If Query currently denies inspection, retain the original command and present
`DisclosureRefused`. Do not expose cached completion or treat it as no-effect.
When lawful again, inspect and use the newly returned next sequence.

## How It Relates To Other Features

Use the existing caller verifier, product admission/sessions and instance
selection. Query owns installed action selection, permission, original intent,
publication, idempotency and recovery. Supplier preparation owns external input
capture/revision checks, never a substitute transaction with that provider.

Required-output, retained-history and asynchronous signing owners are not
selected by this source-completion transport. Do not erase their obligations
to fit this lane. Store persistence remains a distinct integration boundary.

## Inspection And Debugging

Replies preserve source/commit posture, recovery provenance, actual outcome
identity and structured domain/permission/source reasons. Diagnostic members
are contract names, not record values or credentials. Generate clients from
the mounted catalog and actual command/reply schemas; do not parse Debug text.

## Anti-Patterns

- Reissuing an uncertain command with new IDs or a new logical key.
- Treating an external principal/custody match as Query permission.
- Returning cached business evidence after current disclosure was denied.
- Reconstructing native owners or recovery authority from the reply.
- Implementing supplier advancement/authorization callbacks or copied workers.

## Current Limits

One bounded worker owns each installed adapter's move-only native custody.
`try_finish(self)` quiesces creates and succeeds only after custody is empty;
failure returns the same continuation for lawful close/retry. Final Drop is
abandonment, not cleanup, completion or crash durability. This registry does
not survive process loss; old live-instance locators must remain fenced.

## Related Docs

- [Connect another backend](connect-another-backend.md)
- [Ordinary Query owners](../../../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md#owning-fixed-and-source-only-actions)
