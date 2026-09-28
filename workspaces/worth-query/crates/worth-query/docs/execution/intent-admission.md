# Intent Admission

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Intent admission is the advanced runtime surface for Query operations that
need an inspectable eligibility decision before execution. Ordinary product
journeys begin in a capability namespace such as `facade::read`,
`facade::aggregate`, `facade::live`, or `facade::mutation`. Use the intent
surface when a runtime-owned family specifically exposes it and the consumer
needs its decision trace or sealed handoff.

## Why You Use It

- Execute a supported read, live read, materialization, inspection, probe,
  write, batch write, or effect-follow-on intent.
- Inspect eligibility and the decision trace before committing to execution.
- Preserve one proof chain from the authored operation through the final result
  or receipt.
- Handle support, basis, policy, routing, and stale-state failures as typed
  outcomes instead of rediscovering them from lower-runtime errors.

## Stable Entry Points

Use a family-owned authoring method from `worth_query::facade::runtime`:

- `runtime.intent(declaration)`
- `runtime.write_intent(command)`
- `runtime.write_batch_intent(commands)`
- `runtime.next_effect_write_intent(...)`
- `workspace.read_family_intent(&family)`
- `workspace.read_family_in_basis_context_intent(&family, &context)`
- `workspace.read_live_intent(&view)`
- `workspace.materialize_intent(&view)`
- `workspace.inspections()?.inspect_intent(target)`
- `workspace.inspect_derived_intent(&view)`
- `runtime.probe_existing_intent(request)`
- `workspace.probe_existing_intent(request)`

The ordinary path is `.execute()`. The advanced path is
`.review()?.admit()?.execute()`.

Coverage is family-specific. Check the support matrix before treating a nearby
intent-shaped type as an admitted runtime feature.

Basis admission and projection authority have their own public surfaces. Use
`facade::foundation::basis_lifecycle()` for advanced basis capabilities. For
an ordinary read, consume projection facts from the completed read with
`completion.consume_projection(read::project_facts()...)`.

## Core Mental Model

Intent authoring and intent authority are different things.

The authoring object records what the caller wants. Query derives eligibility,
classifies the decision, and—only for an admitted decision—creates the sealed
handoff and execution binding. The result or receipt retains the same decision
trace so inspection never has to guess how execution became legal.

Some decisions are advisory or require a next action. They are not successful
execution, and they should not be flattened into `true`, `false`, or a message
string.

## How It Executes

```text
family-owned intent authoring
  -> request and eligibility
  -> admitted, advisory, or violation decision
  -> sealed handoff and execution binding, when admitted
  -> runtime execution
  -> result or receipt carrying the decision trace
```

Unsupported combinations stop before construction, lowering, or lower-runtime
contact. The executor consumes the admitted binding; it does not repeat support
or authority decisions.

## Small Example

```rust
let result = workspace
    .read_family_intent(&family)
    .execute()?;

let trace = result.receipt().decision_trace_envelope();
```

This is the normal product path: declare the read family, execute through
admission, and retain the receipt if the application needs evidence.

## Real Example

```rust
use worth_query::facade::runtime::WorthQueryIntentConsumerOutcomeClass;

let review = workspace
    .read_live_intent(&view)
    .review()?;

if review.consumer_inspection().outcome_class()
    != WorthQueryIntentConsumerOutcomeClass::Admitted
{
    return handle_intent_stop(review.consumer_inspection());
}

let admitted = review.admit()?;
let handoff_digest = admitted.handoff().handoff_digest().to_owned();
let binding_digest = admitted.execution_binding().binding_digest().to_owned();
let result = admitted.execute()?;

record_live_read_evidence(
    handoff_digest,
    binding_digest,
    result.receipt().execution_provenance_chain_digest(),
);
```

The workspace owns runtime configuration and the view owns canonical Query
meaning. The review exposes read-only decision evidence. Only `admit()` can
produce the binding consumed by execution, and the result receipt preserves
the provenance chain.

For a write whose operation is already known:

```rust
use worth_query::facade::runtime::WorthQueryIntentConsumerOutcomeClass;

let receipt = runtime
    .write_intent(command)
    .execute()?;

match receipt
    .consumer_inspection()
    .map(|inspection| inspection.outcome_class())
{
    Some(WorthQueryIntentConsumerOutcomeClass::Admitted) => persist_receipt(receipt),
    _ => handle_missing_admission_evidence(receipt),
}
```

Use the typed outcome and stop classification available on the concrete
receipt or error in production code; the string projection above is useful for
logging and examples, not for minting authority.

### Family route reference

The common path stays deliberately uniform across the covered families. These
snippets assume the declarations, handles, contexts, and requests have already
been authored through their owning public facade namespace:

```rust
let receipt = runtime
    .intent(declaration)
    .execute()?;

let consumer = receipt.consumer_inspection();

let outcome = consumer.map(|inspection| inspection.outcome_class());

let scoped_basis = basis_lifecycle()
    .current_head()
    .observe()?;

let read_completion = read_declaration
    .using(read::current())
    .run(&mut workspace)
    .into_result()?;
let projection = read_completion
    .consume_projection(read::project_facts().entity_identities());

let read_result = workspace.read_family_intent(&family).execute()?;
let basis_result = workspace
    .read_family_in_basis_context_intent(&family, &context)
    .execute()?;
let write_receipt = runtime.write_intent(command).execute()?;
let batch_receipt = runtime.write_batch_intent(commands).execute()?;
let live_result = workspace.read_live_intent(&view).execute()?;
let live_rows = workspace.read(&view);
let inspection_result = workspace.inspections()?.inspect_intent(&view).execute()?;
let materialization = workspace.materialize_result(&view)?;
let inspection = workspace.inspections()?.inspect(&view)?;
let probe_result = runtime.probe_existing_intent(request).execute()?;
let workspace_probe = workspace.probe_existing_intent(request).execute()?;
```

When you need to inspect a decision before execution, stop at review or at a
scoped capability instead of reaching into admission internals:

```rust
let review = runtime.intent(declaration).review()?;

let request = review.request();
let admitted = runtime.intent(declaration).review()?.admit()?;
let batch_review = runtime.write_batch_intent(commands).review()?;
let scoped_inspection_basis = basis_lifecycle()
    .current_head()
    .inspect()?;
let (authority, projection_warnings) = projection
    .into_admitted()
    .map_err(handle_projection_stop)?;
let write_review = runtime.write_intent(command).review()?;
let live_review = workspace.read_live_intent(&view).review()?;
let inspection_review = workspace.inspections()?.inspect_intent(&view).review()?;
let probe_review = runtime.probe_existing_intent(request).review()?;
```

The scoped basis and consumed projection authority are neighboring authority
surfaces, not intent receipts. They appear here because intent execution may
consume them; intent admission does not mint or replace them. Advanced
substrate sources such as retained artifacts or write receipts keep their
source-specific projection-consumption operations, but ordinary read code must
not reconstruct result-shape or authorization inputs that its completion
already owns.

## How It Relates To Other Features

- [Writes And Intent Boundaries](./writes-and-intents.md) explains when direct
  mutation or an admitted intent is the right authoring surface.
- [Existing Truth](../capabilities/existing-truth.md) covers verified existing
  targets and probe intent.
- [Basis Capability Lifecycle](../capabilities/basis-capability-lifecycle.md)
  owns operation-scoped truth-world authority.
- [Projection Consumption](../capabilities/projection-consumption.md) owns
  downstream fact authority; it is not another intent family.
- [Inspection](../capabilities/inspection.md) reads retained results, receipts,
  and denials after a decision.
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
  tells you which family/profile combinations are admitted today.

## Inspection And Debugging

The advanced path exposes:

- `request()`
- `eligibility()`
- `decision()`
- `decision_trace_envelope()`
- `consumer_inspection()`
- `handoff()` after admission
- `execution_binding()` after admission
- `execution_provenance()` and
  `execution_provenance_chain_digest()` on the result or receipt

Prefer typed outcome and stop classes for branching. Treat messages, rendered
labels, and digests as presentation or evidence projections.

## Anti-Patterns

- Constructing admission requests, eligibility artifacts, handoffs, or
  bindings outside the family-owned authoring surface.
- Treating every type with `Intent` in its name as an admitted product lane.
- Calling lower runtimes directly after Query has denied or deferred a family.
- Recomputing routing or support posture from result text.
- Flattening advisory and violation decisions into a boolean.
- Using intent admission to replace basis lifecycle or projection authority.

## Current Limits

- The named entry points above are real only for runtime profiles whose support
  rows admit the corresponding family.
- Temporal and async/resource meaning extends existing declaration and live
  lanes; it does not imply a blanket generic intent API.
- Store-backed replay, durable restart, and neighboring materialization
  families remain deferred where the support matrix says so.
- Certification tooling is available only through
  `worth_query::facade::certification`; it is not ordinary execution DX.

## Related Docs

- [Writes And Intent Boundaries](./writes-and-intents.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
- [Basis Capability Lifecycle](../capabilities/basis-capability-lifecycle.md)
- [Existing Truth](../capabilities/existing-truth.md)
- [Inspection](../capabilities/inspection.md)
- [Projection Consumption](../capabilities/projection-consumption.md)
