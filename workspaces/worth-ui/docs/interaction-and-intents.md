# Interaction And Intents

## What This Feature Is

Worth UI turns loss-aware native input into presentation-bound semantic
interactions, then routes those interactions into typed product intents. Each
step has its own authority: a pointer release is not an activation, an
activation is not an admitted intent, and UI admission is not permission to
mutate Query or another product domain.

## Why You Use It

Use this path when a visible control should request product work while keeping
targeting, operability, confirmation, concurrency, execution, and visible
consequences explicit. It replaces callbacks and string command dispatch with
compiler-visible contracts and bounded runtime lifecycles.

## Stable Entry Points

- `worth_ui::facade::observation_report` exposes the loss-aware host input
  contract and protocol negotiation.
- `worth_ui::facade::interaction` exposes semantic interaction, gesture, draft,
  and presented-target contracts.
- `worth_ui::facade::intent` exposes definitions, declarations, typed payloads,
  operability, confirmation, admission, providers, recovery, and consequences.
- `worth_ui::facade::service` exposes runtime-service policies, typed command
  routing, and normalized installation plans.
- `WorthUi::app()` registers application facts, definitions, and providers.
- `WorthUiActiveApplicationSession` admits native observations, dispatches or
  advances attempts, retries recovery and consequences, and exposes bounded
  causal lookup.

The complete public type relationship is compiled from
[`typed_intent_relationships.rs`](../crates/worth-ui/tests/ui/facade/intent/pass/typed_intent_relationships.rs).

## Core Mental Model

```text
host observation batch
-> exact presented target and gesture/draft lifecycle
-> semantic interaction
-> typed route and coherent payload projection
-> operability and optional confirmation
-> move-only UI admission
-> destination-specific managed execution
-> separate product or Query admission
-> declared consequence
-> ordinary rebind and mounted publication
```

Reporting identities and causal traces may explain this path. They cannot be
converted into any value on the path.

### Native units and IME

Pointer positions carry both a coordinate space and unit. The canonical
viewport-logical representation stores 1,000 subpixels per logical point;
`UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT` freezes that scale. Adapters must
convert explicitly rather than pass ambiguous floats. Pointer identity,
pressed buttons, capture epoch, presentation basis, sequence, and time basis
travel with the observation batch.

IME is phaseful: `Preedit`, `Commit`, or `Cancel`. A preedit selection arrives
as a Unicode-scalar range and is converted once into canonical UTF-8 byte
boundaries with `UiHostImeRangeConversionReceipt`. Empty, reversed,
out-of-range, and overflowing preedit coordinates are typed denials. Preedit is
draft state; only commit can produce committed edit meaning.

### Event-time affinity and settlement

Native input is admitted against the last completed presentation and the
event-time profile. A successor may be pending while observations remain bound
to that completed predecessor; a profile or scale transition blocks input until
its profile evidence is completed. Resize and scale evidence uses the actual
client size and event tick, without writing through a resize-event writer or
feeding back a synthetic size.

Readiness is level-triggered by retained batches. The event loop rearms the
signal only after the runtime callback has drained through the authorized
interaction owner. Close waits for retained observations to settle, and typed
denials remain observable instead of being collapsed into a generic failure.

### Gestures and drafts

A pointer gesture binds its press to the exact presented mounted incarnation.
Release must satisfy the same continuity and capture contract; loss, stale
presentation, or replacement stops the gesture instead of retargeting current
coordinates. Keyboard activation carries its own evidence and joins the same
semantic `Activate` family only after targeting succeeds.

Draft sessions have explicit recipient identity, revision, session count, and
UTF-8 byte budgets. Edit and selection changes mutate bounded draft state;
`EditCommit`, `SelectionCommit`, and `Submit` are distinct semantic families.
An executor never rereads renderer state to assemble a payload.

### Definition, declaration, and route

`UiIntent` fixes an intent ID, payload type, product outcome type, and accepted
semantic interaction families. `UiIntentDefinition<I>` selects exactly one
execution destination:

- application effect, paired with `register_intent_provider`;
- UI transition (`NavigatePage` or `ChangeMosaic`), registered directly; or
- runtime service (`OpenPortal`, `ClosePortal`, or `InvokeCommand`), registered
  with `register_runtime_service_intent_definition` and executed by its shipped
  family owner.

Those are the intent-origin subset of service work. Window-focus observation,
scroll delta, Tick, reduced-motion policy, hot rebind, portal dismissal, focus
restoration, and motion continuation enter through their own typed origins.
They do not allocate or counterfeit an intent attempt.

`UiIntentDeclaration<I>` binds an authored route to typed payload sources. Its
typestate cannot become a DSL specification until operability, confirmation,
concurrency, and consequences are all supplied. Mounted controls retain compact
route bindings; they do not retain providers or callbacks.

### Operability, confirmation, and concurrency

Operability is a coherent proof over separate support, mutability, readiness,
occupancy, policy, and affinity/currentness axes. Visible enabledness is a
projection of that proof, never a substitute for it.

Confirmation issues an affine challenge bound to its slot, generation,
lineage, route, input basis, and expiry tick. Only the matching fresh
continuation can become a confirmed candidate. A boolean dialog result or a
stale challenge cannot bypass re-admission.

`TargetRouteSingleFlight` is the ordinary concurrency scope. Wider
serialization must be selected explicitly. Capacity pressure returns a typed
occupancy or reservation stop; it does not create a hidden queue.

### Version coexistence

Payload schema, outcome schema, and provider version contribute to prepared
application identity. Replacement therefore cannot silently reinterpret an
old attempt with a new provider. Running before-effect work is cancelled
against its predecessor binding; partial or indeterminate recovery retains the
predecessor provider until the affine recovery authority settles.

## How It Executes

1. `admit_host_interaction_batch` validates the host batch and updates bounded
   gesture or draft state.
2. `admit_native_intent_observations` resolves semantic interactions against
   typed product and confirmation routes, projects one coherent payload, and
   evaluates operability.
3. Admission returns an exhaustive decision. An admitted candidate is
   move-only and must be cancelled, confirmed where required, or dispatched.
4. `dispatch_admitted_intent` creates a bounded attempt and calls only the
   exact `UiIntentExecutionProvider<I>`. `advance_intent_executions` polls and
   settles managed work.
5. Before-effect rejection/cancellation, completion, partial effect, and
   indeterminate effect remain distinct. Partial or indeterminate work returns
   `UiIntentRecoveryHandle`; recovery cannot be recreated from diagnostics.
6. A typed product outcome yields only its declared consequences. Query or
   domain mutation still requires that owner’s admission and receipt before
   ordinary rebind can publish a visible consequence.

For an intent-origin runtime service, step 4 dispatches to the exact Portal or
Command Routing destination. A multi-family portal operation stages
Portal-owned work plus typed Focus and Motion requirements; the non-publishing
proposal compiler orders those owner stages and hands the coherent batch to
the existing publication boundary. No family calls another, and proposal
compilation does not become product execution or host settlement.

## Small Example

This is the authored shape used by Platform Pulse:

```wui
component platform.pulse.component.identity_target {
  interaction activate routes platform.pulse.action.route;
}
intent platform.pulse.action.route {
  definition platform.pulse.action;
  interaction activate;
  payload action_input_revision application-unsigned64 platform.pulse.action.input-revision;
  operability platform.pulse.action.operability
    mutability-application-boolean platform.pulse.action.mutable
    readiness-application-boolean platform.pulse.action.ready
    policy-application-boolean platform.pulse.action.policy-allowed;
  confirmation platform.pulse.action.confirmation
    application-boolean platform.pulse.action.confirmation-required;
  concurrency target-route-single-flight;
  consequences mounted-posture-and-query platform.pulse.status;
}
```

The component names a route. The declaration names typed sources and policy;
neither embeds an effect handler.

## Real Example

An application-effect definition changes the builder typestate until the exact
typed provider is registered:

```rust
let app = WorthUi::app()
    .register_intent_boolean_fact(operable, true)?
    .register_intent_definition(UiIntentDefinition::<SaveIntent>::application_effect())?
    .register_intent_provider(SaveProvider)?
    .freeze()?;
```

`SaveProvider` must implement `UiIntentExecutionProvider<SaveIntent>` and its
`begin` method receives `UiIntentExecutionRequest<SaveIntent>`. A provider for
another intent cannot satisfy the builder, and a definition without its
provider cannot freeze. See the linked compile-pass source for a complete
payload, outcome, declaration, provider, dispatch, and recovery example.

Platform Pulse is the permanent real-world example. Its native-window journey
proves ready, held, completed, confirmation-required, stale-confirmation,
denied, and rebind-cancelled postures through the same mounted page. See
[Application lifecycle](./application-lifecycle.md) for the executable command
and external input workflow.

## How It Relates To Other Features

- Query binding owns Query-issued facts and mutation receipts; an admitted UI
  intent does not grant either.
- The 3.12 observation/rebind transaction publishes intent posture and product
  consequences atomically with mounted truth.
- Visual inspection can correlate an intent with the resulting frame and
  pixels, but cannot execute or re-admit it.
- Portal and command intent destinations use this same admission path. Focus,
  motion, scroll, and selection also respond to non-intent origins through
  their own typed contracts; none replace generic intent admission.
- A command-route receipt proves only which UI route won. Any Query or product
  operation it requests performs separate admission.

## Inspection And Debugging

`lookup_intent_causal_trace` returns `Found`, `Expired`, or `Unavailable` for
an exact evidence reference. The runtime retains at most 64 semantic
interaction records and a fixed causal-evidence byte budget. Motion and IME
preedit do not manufacture intent records. Trace projections correlate source,
host sequence, target, route, payload, operability, admission, attempt,
consequence, Query fact, mounted frame, and pixels without retaining any of
their authority.

## Cost Model

Ordinary work scales with observations selected for semantic handling, the
resolved target and route, projected payload width, affected consumers, and
occupied attempt slots. Route lookup is indexed across large catalogs. Pointer
motion and unchanged turns perform zero semantic intent work. Capacity limits
are public constants and saturation is a typed outcome, never an unbounded
scan or allocation fallback.

## Anti-Patterns

- Treating a click, key, visible enabled flag, or confirmation boolean as an
  admitted intent.
- Storing callbacks, handlers, string commands, providers, or product payloads
  in controls, renderers, or host adapters.
- Reading draft or product state again inside a provider instead of consuming
  the admitted typed payload.
- Calling Query or domain mutation from UI admission or diagnostic code.
- Publishing completion before the declared consequence is admitted and the
  mounted successor commits.
- Retrying partial or indeterminate work without its affine recovery handle.
- Retargeting a gesture after capture loss, frame replacement, or stale
  confirmation.
- Treating a window-focus event, scroll delta, Tick, dismissal, restoration,
  or rebind as an intent merely because it can trigger service work.

## Current Limits

`OpenPortal`, `ClosePortal`, and `InvokeCommand` are the shipped intent-origin
runtime-service destinations. Direct focus, scroll-to, reveal, and selection
destinations are not public intent shortcuts; their current operations use the
family-specific origins and proposal requirements described in
[Runtime services](./runtime-services.md).

Appearance consumes sealed hover, pressed, selection, focus, operability, and
validation facts after their owners decide meaning. It can select paint and
pointer-affordance cells but cannot admit an intent, mutate an owner, or make a
disabled control operable. Pressed and pending-operability replacement clear or
carry state only through the owner-issued succession used by the accepted
frame. Intent detail and appearance inspection remain bounded and
non-reconstructive; replay is not an ordinary runtime feature.

## Related Docs

- [Worth UI architecture](./architecture.md)
- [Application lifecycle and Platform Pulse](./application-lifecycle.md)
- [Runtime subsystems](./runtime-subsystems.md)
- [Runtime services](./runtime-services.md)
- [Query-backed UI views](./query-binding.md)
- [Application inspection](./inspection.md)
