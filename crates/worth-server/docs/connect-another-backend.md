# Connect another backend

## What this feature is

This guide is for a service that wants to call `worth-server` as a downstream
system.

That service might be:

- an app backend
- a worker
- a gateway
- a sidecar
- another Rust service

## Why you use it

A backend should use `worth-server` when it wants a strict execution boundary
without reimplementing:

- validator rules
- basis admission
- idempotent write replay
- multipart upload validation
- binary resume and integrity logic

## Stable entry points

Same-process Rust integration:

- `WorthServer::compat_http()`
- `WorthServerProductOperationDeclaration::with_program_application(...)` for
  an adapter backed by an existing native Query program. This retains its
  authentic allocation for readiness, without extracting a raw runtime.

HTTP-shaped integration:

- the five `/compat/...` route families

If you are already in the same Rust process, use the facade directly. If you
are crossing a network boundary, model requests around the normalized route
families and preserve the same headers and preconditions.

## Core mental model

Another backend should treat `worth-server` as an authority boundary, not as a
thin helper library.

That means:

- do not normalize away denial semantics
- do not rewrite validator or basis rules locally
- do not invent your own resume protocol
- do not silently map upload failures into plain mutation retries

## How it executes

The simplest service-to-service pattern is:

1. Resolve the target operation.
2. Build a read, mutation, upload, stream, or download request in the correct
   route family.
3. Preserve the response semantics exactly.
4. Retry only with the server's supported preconditions and replay keys.

## Small example

```text
Backend A wants to create a task attachment.

1. POST /compat/uploads/files.attachment.upload
2. Include structured command metadata in the manifest.
3. Include the file part separately.
4. If the upload is interrupted, use cleanup or retry honestly instead of
   assuming partial commit succeeded.
```

## Real example

If your backend needs to bridge from OpenAI tools, browser actions, or another
app server into Worth truth:

- map "fetch current state" to `/compat/reads/...`
- map "perform authoritative change" to `/compat/mutations/...`
- map "large result delivery" to `/compat/streams/...`
- map "binary ingress" to `/compat/uploads/...`
- map "binary egress" to `/compat/downloads/...`

The payoff is that your backend does not have to become its own half-correct
query handoff layer.

For a native Query application, keep registration inside its entry/adaptation
boundary. Attach its existing `WorthQueryProgramApplicationRuntime` with
`declaration.with_program_application(program.clone())`, then return the finished
`WorthServerProductApplicationAdapterRegistration` to your server composition
root. That clone retains the same installed allocation; it does not install a
second graph or cache a readiness snapshot. Ordinary operation execution must
still enter Query with fresh authenticated authority and the verifier-bound
request deadline/cancellation scope. Readiness never supplies operation grants.

An adapter implementing only `execute` supports the ordinary scalar execution
path. Its default `execute_with_lease` returns the explicit
`product.checked_execution_unsupported` denial. To support checked shared-read
batches, implement that method and propagate the exact lease into nested work;
never call the unleased method as a fallback.

### Application-owned command retries

For a runtime-backed mutation that retains genuine native operation custody,
declare `WorthServerProductRetryOwner::ApplicationCustody`:

```rust
use worth_server::{WorthServerProductOperationDeclaration, WorthServerProductRetryOwner};
fn owning_command(declaration: WorthServerProductOperationDeclaration)
    -> WorthServerProductOperationDeclaration {
    declaration.with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody)
}
```

The default for eligible runtime-backed mutations is `ServerReceipt`: a matching
idempotency retry returns the prior product response without executing the adapter
again. Reads are never eligible for response replay, even with an idempotency
header; each read reaches fresh adapter authorization. Receipt replay is not appropriate
when the adapter must reauthenticate through Query or advance a live native
owner. Cached responses can also describe Pending or a prior refusal rather than
the owner's present outcome.

With `ApplicationCustody`, Server still performs transport admission, session
checks, readiness and scheduling, then re-enters the adapter for every command.
It does not consult or write its response-retry cache and emits no Server retry
receipt or `PreviouslyCommitted` classification. The adapter's current native
outcome remains in its declared result contract. A successfully published response
is not proof that the business operation committed.

For example, an owning creation adapter reserves bounded capacity, captures the
original input once, and retains the actual native operation before replying.
After a lost reply, a fresh authenticated command addresses that same custody.
The adapter supplies fresh request authority to native advancement, retains any
returned owner before responding, and distinguishes completion from pending,
denial, indeterminate publication and close refusal. It cannot recapture the
clock, replace the original key, infer completion from a current row, or return
protected prior results without fresh native authorization. The declaration flag
does not implement any of those responsibilities.

Only runtime-backed product mutations may use application retry custody. Reads,
session-coordination operations and durable-executor declarations reject it during
registration. Durable mutation replay keeps its separate installed executor
contract. Existing default declarations keep their canonical identity; application
custody contributes distinct canonical meaning. This is process-local ownership,
not Store-backed crash recovery or an atomic whole-graph compare-and-swap promise.

## Inspection and debugging

Log and preserve:

- operation name
- route family
- validator and basis when present
- idempotency-key on writes
- integrity and retry posture on downloads
- exact denial code and detail

Those are the fields that explain behavior. Most generic request logs are too
thin to debug this surface well.

## Anti-patterns

- Flattening every denial into a generic 400 or 500 with no semantic code.
- Mixing binary transfer concerns into ordinary JSON writes.
- Replacing explicit server preconditions with "best effort" local retries.
- Assuming a generic HTTP client abstraction is enough by itself.
- Using Server response replay to bypass fresh native authority or replace a live
  operation owner, or treating `ApplicationCustody` as an idempotency implementation.

## Current limits

- These docs focus on the stable execution surface, not a generated language
  SDK.
- A non-Rust backend will need its own thin request builder layer.
- Durable resume claims remain narrower than ordinary runtime-backed retries.

## Related docs

- [Connect an agent](./connect-an-agent.md)
- [Write data](./write-data.md)
- [Upload files](./upload-files.md)
- [Runtime-backed vs durable](./runtime-backed-vs-durable.md)
