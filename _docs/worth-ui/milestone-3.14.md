# Milestone 3.14: Intent, Operability, and Interaction Substrate

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../coding_guidelines/qa_review_guide.md) and
> [testing laws](../coding_guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Status and Placement

Status: Closed on 2026-08-01. Phases 1 through 5 and all thirteen interaction
and intent proof rows are closed on final source.

Milestone 3.13 closed the honest path from Query-owned projection truth to
mounted pixels. Milestone 3.14 adds the reverse product path: native human
input becomes a presentation-bound semantic interaction, routes to one
declared product intent, is admitted from coherent runtime truth, executes
through its declared application- or WUI-owned execution destination, and
returns typed consequences through the existing 3.12
observation/rebind/publication path.

This milestone does not make the UI runtime a domain mutation authority.
Query, application domains, and external systems retain their own admission
and effect boundaries. Milestone 3.15 still owns production portal, focus,
motion, command-routing, scroll, and selection services; 3.16 owns broad
interaction-state appearance.

## Goal and Central Claim

The only ordinary path from native input to product consequence is:

```text
native host observation
-> exact presented-frame target
-> semantic interaction
-> exact route binding and intent definition
-> typed payload projection
-> runtime-derived operability
-> UI routing admission
-> managed application execution
-> typed terminal outcome
-> declared UI consequences
-> 3.12 observation/rebind/publication
-> mounted posture and visible pixels
```

Every arrow preserves the application, host session, surface, binding
generation, mounted incarnation, presentation, source order, and relevant fact
revision. A host event, coordinate, graph identity, display string, diagnostic,
renderer callback, static readiness value, or equal-looking payload cannot
skip a stage or mint later authority.

UI admission proves only that the current UI may route one exact attempt. It
does not prove that a domain mutation is authorized or succeeded. A Query-
backed executor must separately use Query's admitted mutation/effect lane and
return its typed result; it may not reinterpret `UiAdmittedIntent` as Query
authority.

## Current Boundary and Exact Gap

The repository already has:

- bounded, sequenced, loss-aware host observation batches for pointer, key,
  focus, text, IME, and other native facts;
- presentation-bound hit-test maps and exact mounted receipt traces;
- stable declaration, graph, allocation, mounted, visual, and Query identities;
- the 3.12 observation compiler and atomic publication path;
- 3.13 scalar/collection projection facts and mounted semantic text; and
- the permanent native Platform Pulse process and external runner.

The current intent-shaped surfaces are not an implementation:

- `UiIntentDeclarationFamily` is only a sealed family marker;
- `CommandRuntimeIntentBinding` is a cloneable string placeholder;
- `CommandReadinessBinding` is static four-state metadata and defaults to
  always admitted;
- `WorthUiTransientInteractionState` names drop policy, not semantic
  interaction, draft, capture, or intent authority;
- text/IME observation payloads do not distinguish preedit, commit, and cancel
  or carry a canonical text-coordinate conversion receipt;
- pointer-button observations lack an exact lossless position or same-sequence
  position witness and therefore cannot yet prove button-time targeting
  without borrowing coalesced motion;
- the egui adapter retains host observations but does not yet translate native
  pointer/key/text/IME input into the production observation contract; and
- the Pulse renders and rebinds but accepts no native product interaction.

Milestone 3.14 replaces or completes those paths. It may not wrap them while
leaving a string-key, static-readiness, callback, or allocation-local
interaction lane alive.

## Decisive Product Courtroom: `IA-01`

### Real world

Extend the existing `worth-ui-platform-pulse` executable-world journey in the
same Cargo-built child, native window, isolated filesystem installation,
observation stream, source watcher, Query installation, and teardown contract.
Add one visible action control and a separate visible confirmation action.
The application composition root registers a typed intent definition, its
payload/operability inputs, and a typed execution provider.

The Windows runner sends real operating-system pointer motion, press, and
release to the process-bound native window. It may capture pixels, edit the
existing product input files, and release a deliberately held application
executor through its external product-world gate. It may not call an intent,
interaction, confirmation, Query, runtime, or host facade; construct authority;
or inject adapter observations.

The Pulse owns one versioned external action-service input beneath a new
`--intent-source-root`. Its production adapter translates policy and executor-
gate revisions into typed application facts; the UI runtime never parses that
transport. Releasing the gate lets the provider call an application-owned
action port, which separately enters Query/domain admission and publishes the
new product value. The runner edits only this external service input. It does
not publish the expected Query value or a completion event. The adapter uses
bounded watcher/channel delivery; it performs no per-frame file read, parse, or
policy evaluation. Every Pulse file watcher (intent source, Query value, theme
preference) admits an operating-system notification through one owner
predicate: the notification must name the watched file and must not be an
access notification. A watcher must never observe its own reads as change;
inotify emits an open for every read, so a watcher that queued raw
notifications re-read on its own read until its bounded channel overflowed,
while `ReadDirectoryChangesW` emits no access notifications and the first
qualified platform never saw the loop. Writers replace the record atomically
(temporary file, then rename); a truncating in-place write is not the contract.

### Hostile sequence

1. Complete the inherited pending/current Query, identity, overlay, valid
   rebind, malformed-preservation, schema-stop, and recovery sequence.
2. Capture the current frame and choose the action point independently from
   the external client image. Deliver native pointer press and release.
3. Hold the application executor after UI admission. Prove a visible
   `admitted` posture, one live attempt, no completion, no Query/domain
   consequence, and no duplicate execution.
4. Release the executor. Prove one typed completion, one declared consequence,
   one 3.12 turn, a visible completed posture, and the expected Query-backed
   value change.
5. Change the application-owned policy input to require confirmation. Activate
   again. Prove one exact challenge, visible confirmation-required posture, and
   zero executor calls.
6. Change a payload fact or application generation, then activate the
   confirmation control. The old challenge must stop as stale and must not
   execute.
7. Obtain a fresh challenge and confirm it through native input. Prove exactly
   one continuation and completion.
8. Drive disabled and policy-denied postures. Native activation may produce
   explanatory interaction evidence but must mint no admitted attempt.
9. Hold one final admitted attempt, apply an incompatible rebind or unmount,
   and prove its exact cancellation/retirement posture with no false rollback
   or late consequence.
10. Close normally and prove zero retained observation, gesture, capture,
    draft, payload, challenge, attempt, reservation, execution, completion,
    consequence, inspection, Query, host, visual, process, or installation
    resources.

### Required verdict and independent observations

External pixels, native input delivery, product-issued interaction/intent
receipts, provider call evidence, Query/domain outcome, 3.12 rebind/publication
receipts, mounted identity, and a framework-owned resource census must agree.
No one evidence class is sufficient.

At least these defects must turn `IA-01` red:

- calling a renderer or adapter callback directly;
- targeting the current graph or current coordinate instead of the presented
  frame;
- treating pointer press, release, or click delivery as intent success;
- using static `always_admitted` readiness;
- assembling payload in the renderer or executor;
- invoking the executor before confirmation;
- replaying a challenge after payload, target, world, policy, or generation
  changes;
- reporting UI completion before the product executor settles;
- treating Query change as authorized by UI admission;
- publishing the visible posture outside the 3.12 path; or
- leaking or double-disposing a held attempt during rebind or shutdown.

The cumulative journey remains at most 45 seconds, with every awaited
transition at most 5 seconds, zero blind retry, one child, one native window,
the existing evidence/artifact bounds, and the existing executable-world
target.

## Supporting Proof Portfolio

| ID | Adversarial world | Required verdict |
| --- | --- | --- |
| `IA-02` | Press/release on the same target, different targets, drag out/back, overlap, clip/occlusion, stale frame/binding/receipt, foreign surface, and compatible/incompatible remount. | Activation requires press/release continuity for one mounted incarnation across exact presented frames. Only an owner-issued continuity witness may cross a newer presentation; coordinates, current lookup, or equal IDs cannot retarget it. |
| `IA-03` | Duplicate, reordered, skipped, and delayed observations; coalesced motion; lossless button/key/text/IME transitions; capture-epoch change; active-pointer capacity minus one/at/plus one; overflow before and during a gesture; repeat and double activation. | An independent gesture model agrees. Loss or exhaustion cancels/stops exactly; two complete press/release pairs produce two interactions, while repeat metadata alone produces none. |
| `IA-04` | Exhaust the small operability lattice and pairwise coupled axes: support, mutability, readiness, target/declaration/definition/application occupancy, policy, freshness/affinity, and confirmation. | Only the exact operable proof advances. Target-scoped work does not disable peer routes; displayed enabledness and static command readiness have no authority. |
| `IA-05` | Payloads with 0, 1, and 64 fields; empty/exact/over-budget Unicode text; IME preedit/commit/cancel and native-coordinate conversion; impossible source shape at catalog formation; missing, malformed, stale, and reordered runtime inputs; selection reorder; Query change across predecessor/successor bases; attempted assembly during publication transition. | One sealed input basis produces the typed payload. Only committed text and exact current option identity enter it; no JSON, `Any`, string bag, renderer scan, positional selection, or mixed revision enters admission. |
| `IA-06` | Duplicate confirm, simultaneous confirm, changed payload/control/intent/world/application generation/policy, expiry, cancellation, and replay. | One affine challenge opens at most one exact continuation. All stale or foreign challenges stop before execution. |
| `IA-07` | Interrupt at host retention, target resolution, gesture completion, payload projection, operability evaluation, admission, reservation, before/after the external effect, completion, consequence admission, and publication. Cross relevant points with compatible/incompatible declaration/provider-schema replacement, unmount, and shutdown. | Each phase owns a typed terminal posture. Pre-effect work cancels or rebinds exactly; effecting work retains its bounded predecessor owner/version until settlement. Escaped effects are never called rollback and late evidence cannot transfer authority. |
| `IA-08` | Unsupported destination, destination-local/global reservation exhaustion, one blocked provider beside an unrelated ready provider, rejection/cancel/timeout before effect, success, downstream domain denial, partial effect, indeterminate effect, duplicate/late settlement, and retry from clean/indeterminate posture. | The framework owns every attempt, exhaustion stays at its declared scope, and one provider cannot create global head-of-line blocking. UI routing admission and domain mutation admission remain separate; partial/indeterminate outcomes retain recovery. |
| `IA-09` | Deterministically permute native interaction, source edit, Query change, and viewport change while an intent is effecting; exhaust all 24 orders in one cheap model. | One canonical order, exact target affinity, bounded scope, and at most one publication hold for every permutation. No OS timing oracle or second event queue exists. |
| `IA-10` | Reuse 1 intent definition across 1, 1,024, and 65,536 routed controls; run 0, 1, and 16 interactions across distinct targets; payload widths 0, 1, and 64; queue occupancy 0, 15, 16, and 17; motion storms and activation bursts. | Exact work and retained-memory counters obey the contractual slopes; target-scoped attempts stay independent, and provider registration/definition storage do not multiply per control. Unchanged and motion-only work is zero beyond bounded dispatch. |
| `IA-11` | Denial, cancellation, retry, replacement, repeated cleanup, and shutdown for every retained observation, gesture/capture, draft, payload, challenge, attempt, reservation, executor handle, recovery authority, evidence reference, and consequence receipt. | Framework census returns to the exact zero baseline; every resource is retired once and retry starts clean. |
| `IA-12` | Trace host sequence -> presentation -> target -> interaction -> intent -> payload revision -> operability -> admission -> attempt -> completion -> consequence -> mounted posture -> pixels. Substitute equal diagnostic IDs, digests, stale receipts, and reporting projections. | The causal trace remains complete, bounded, and non-authoritative; every substitute stops at its owning boundary. |
| `IA-13` | Existing two-session compile owner attempts raw-host-to-intent admission, authority construction/clone/reuse, string-key intent authority, renderer/adapter execution, transient allocation state as semantic intent, payload-shape crossover, diagnostic confirmation/completion, and WUI-to-Query mutation authority. | Valid twins compile; every invalid twin fails for the intended public type or dependency reason. Placeholder/callback residue is mechanically absent. |

Each case receives independent, risk-proportionate review. Parameterization
may share a process or model only when each verdict can fail independently.
`IA-01` cannot be replaced by an in-process facade call.

## Product Decision Lock

### Interaction is not intent

Native events are mechanical observations. The interaction compiler produces
only these initial semantic families:

- `activate` from admitted pointer or direct keyboard gesture;
- `edit-commit` from one runtime-owned draft session;
- `selection-commit` from one declared selection input; and
- `submit` from one declared submit route.

An interaction carries exact target, presentation, gesture/draft revision, and
source-order proof. It authorizes no product effect. `click` is not an intent
or canonical semantic-interaction family, and pointer press/release remain
separately observable.

The host adapter binds each native event to the last completely presented
frame, never the frame currently being assembled. A pointer gesture is scoped
by pointer identity and capture epoch. Press and release may cross
presentations only when the runtime issues a continuity witness for the same
surface, binding, mounted incarnation, and hit-test participation. Leaving and
re-entering that same target remains eligible; releasing over another target,
losing a lossless transition, or changing capture cancels it. Each complete
press/release pair produces one `activate`; OS double-click or key-repeat
metadata creates no additional semantic interaction.

Each lossless pointer-button observation carries its exact button-time
position or an owner-issued reference to a same-sequence lossless position
fact. It may not read the adapter's current pointer or borrow a coalesced motion
sample. Gesture/capture storage is bounded globally and per active pointer.

Keyboard text, IME, submit, or activation can target only an exact
runtime-owned local input recipient already bound to the same mounted
incarnation and generation. 3.14 cannot infer a recipient from native focus,
tree position, last interaction, or current coordinates. General keyboard
navigation, focus routing, and focus restoration remain 3.15 work.

Navigate-page and change-mosaic are typed product intents that may be executed
without a service. Open/close portal and invoke-command are typed requests for
3.15-owned services and therefore return unsupported in 3.14; no adapter-local
popup or shortcut may impersonate them.

The admission substrate accepts a sealed `UiIntentRouteSource`. In 3.14 its
only admitted variant is `MountedInteraction`. Later command, accessibility,
or service sources must add owner-issued variants and reuse the same payload,
operability, admission, and execution phases; they may not counterfeit a
mounted target or create a parallel intent facade.

### Intent meaning and execution registration are separate

Compiled Rust registers `UiIntentDefinition<I>` as capability meaning: stable
identity, payload/result types and schemas, accepted interaction families,
execution destination, and provider contract. File source cannot create that
capability.

File- and Rust-authored composition produce the same opaque
`UiIntentDeclaration`, which references one registered definition and declares
payload sources, operability dependencies, confirmation policy, consequences,
concurrency scope, and budgets. Each control route lowers separately to
`UiIntentRouteBinding`, binding one target and semantic interaction to that
reusable declaration. Definition, declaration, and route are distinct
canonical artifacts, not admitted attempts. Unknown definitions or schema/
destination mismatches stop during candidate preparation before graph or
provider effects.

Duplicate definition identity always stops; registration never uses last-wins
or printable equality. Repeated/virtualized controls derive route identity
from stable graph/mounted identity rather than position, so compatible reorder
can preserve the route without sharing target authority.

Definition, declaration, route, payload, result, and consequence artifacts
carry stable schema identity/version. Compatible replacement requires explicit
coverage for every retained field and destination. A pre-effect attempt may
rebind only with that proof; an effecting attempt retains its exact predecessor
definition/provider version under a bounded lease until terminal settlement.
New generations never reinterpret an old payload or completion.

The definition fixes exactly one execution destination:

- `ApplicationEffect` requires the application composition root to register
  one `UiIntentExecutionProvider<I>`;
- `UiTransition` resolves to the WUI-owned page/mosaic transition provider and
  still requires its own runtime admission; or
- `RuntimeService` is typed unsupported until the 3.15 service owner registers
  that destination.

Controls, renderers, host adapters, DSL source, and Query projections cannot
register or invoke execution. Missing/duplicate required application providers
or wrong-destination providers stop during application preparation. A declared
3.15 service destination remains a support/operability `unsupported` stop and
cannot reach admission; it is not mistaken for an application wiring defect.

The framework may erase provider representation privately after type-checked
registration. It may not erase payload/result meaning into `Any`, JSON, string
maps, untyped closures, or a cloneable intent key. Definitions and providers
are stored once per application generation; repeated controls add compact
route rows, not provider instances or generic executor copies.

### Operability is a proof over orthogonal axes

The runtime derives operability from one current coherent revision:

```text
support       supported | unsupported
mutability    writable | readonly
readiness     ready | pending
occupancy     idle | in-flight
policy        admitted | denied
affinity      current | stale | wrong-world | rebind-required
confirmation  not-required | required(policy)
```

`operable` and the human-facing disabled reason are projections of these axes,
not stored booleans. Multiple non-operable causes remain machine-readable and
deterministically prioritized only for compact display. A visible disabled
style or host widget flag never enters admission.

The runtime evaluates only the dependencies declared by the selected route. It
does not materialize the Cartesian product of axes per control or reevaluate
unrelated controls. `IA-04` exhausts the small semantic model as proof, not as
the runtime representation.

Occupancy uses a typed `UiIntentConcurrencyScope`. The ordinary default is
single-flight per mounted target/route. Declaration- or definition-wide
serialization must be explicit; application-wide serialization is an advanced
explicit choice. Provider queue occupancy cannot silently widen operability
scope or disable unrelated targets.

### Payloads are typed coherent projections

Payload declarations name typed inputs from admitted control draft state,
3.13 projection facts, canonical constants, or application facts admitted
through a declared observation/fact contract. Payload and operability compile
from one phase-scoped immutable view of active runtime publication; they may
not take separate snapshots. The resulting `UiIntentInputBasis` records that
publication plus the exact revision of every consumed owner. It does not lock,
copy, or rescan unrelated owners. Assembly completes before admission and
provider reservation.

Readonly is not a payload-shape or value failure. Payload projection preserves
the shared basis; Phase 3 operability alone derives and denies the mutability
axis from that basis. Payload code may not independently grant or re-evaluate
writability.

The executor receives the sealed admitted payload by value. It cannot add
fields, reread UI state, query the renderer, or reinterpret a field name.
Payload invalidation before execution stops the attempt; invalidation after an
external effect changes completion/recovery posture rather than claiming
rollback.

A selection payload carries the exact current option identity plus its
collection/binding revision, never label or position. Reorder may preserve an
option only through the 3.13 identity contract.

### Draft sessions are narrow and runtime-owned

3.14 replaces the `TextInput`/`InFlightGesture` placeholder use with bounded
runtime-owned draft and gesture lifecycles. A draft session is bound to one
mounted incarnation, application/binding generation, input revision, declared
payload field, byte budget, and IME composition revision.

The host contract distinguishes committed text from IME `preedit`, `commit`,
and `cancel`. Canonical text ranges are UTF-8 byte offsets proven to fall on
Unicode scalar boundaries; adapters translate native coordinate units and
carry the translation receipt. Preedit may update visible draft posture but
cannot enter a payload. `edit-commit` occurs only from a declared commit
gesture; window-focus loss, rebind, or native widget disposal never implies
commit.

It is not the focus service. Direct activation may establish the one local
draft recipient needed for edit evidence; 3.15 later owns focus traversal,
scopes, trapping, restoration, and cross-control routing. Rebind explicitly
preserves, rebases, or cancels a draft; it never commits one. Tree position and
native widget state decide nothing.

### Confirmation is an affine exact challenge

`UiIntentConfirmationChallenge<I>` is bound to the exact intent definition,
route, sealed prepared payload/input basis, operability proof/dependencies,
target incarnation, application/binding generation, world, policy identity,
attempt lineage, and expiry. It owns that candidate by value rather than
retaining only a digest or reassembling after approval. It is non-`Clone`,
consumed once, and only the admission owner can revalidate currentness and
continue it. Expiry uses the runtime-admitted monotonic time basis, not
wall-clock or renderer time.

A boolean, dialog result, diagnostic, or matching digest cannot confirm.
3.14 may mount a plain confirmation control. Its
`UiIntentConfirmationRouteBinding` turns ordinary `activate` into a request to
consume one runtime-owned pending-challenge slot for the named declaration;
none or multiple eligible challenges stop before continuation. It is not a
second product intent, and the control never stores or reconstructs the
challenge. Modal/dialog/portal presentation belongs to 3.15.

UI admission is exhaustive:

```rust
pub enum UiIntentAdmissionDecision<I: UiIntent> {
    Admitted(UiAdmittedIntent<I>),
    ConfirmationRequired(UiIntentConfirmationChallenge<I>),
    Stopped(UiIntentAdmissionStop),
}
```

Confirmation-required is not admitted. Continuing a challenge produces a new
admission decision; it cannot call the provider directly.

### Execution is framework-managed and effect-honest

Before minting `Admitted`, the runtime atomically reserves framework attempt
capacity and occupancy under the declared scope. The move-only
`UiAdmittedIntent<I>` owns that reservation. The runtime also owns the attempt,
deadline, cancellation safe points, idempotency identity, polling, terminal
settlement, recovery authority, and disposal. A provider may still return a
typed provider/resource rejection before external effect.

Capacity is explicit at application, execution-destination/provider, intent,
and retained-byte scopes. Exhaustion rejects at the narrowest exhausted scope
before `Admitted` or external effects and cannot consume an unrelated
provider's capacity. Providers receive the exact attempt/idempotency identity,
payload, deadline, and cancellation contract. The first admitted terminal
settlement wins; duplicate or late settlement is typed evidence with no second
consequence.

Framework admission/reservation is one effect-free progression; provider
invocation is a distinct phase. If dispatch is deferred, target, input basis,
and operability are revalidated immediately before invocation; queued work
cannot execute from stale readiness or policy. Retry of one indeterminate
attempt retains its idempotency identity, while a new human interaction
receives a new attempt identity.

Terminal topology is exhaustive:

```rust
pub enum UiIntentExecutionOutcome<O> {
    Completed(O),
    RejectedBeforeEffect(UiIntentExecutionStop),
    FailedBeforeEffect(UiIntentExecutionStop),
    CancelledBeforeEffect(UiIntentExecutionStop),
    TimedOutBeforeEffect(UiIntentExecutionStop),
    Partial(UiIntentPartialEffect<O>, UiIntentRecoveryHandle),
    Indeterminate(UiIntentRecoveryHandle),
    CompletedEffectConsequenceStopped(O, UiIntentConsequenceRecovery),
}
```

Only a variant explicitly ending `BeforeEffect` guarantees no external product
effect. Cancellation or timeout after provider invocation becomes partial or
indeterminate unless product evidence proves otherwise. Partial and
indeterminate outcomes retain typed recovery. No outcome is relabeled rollback
without a declared reversible transaction and recorded inverse.

### Consequences re-enter existing authority

An executor completion returns typed product evidence plus only consequences
declared by `UiIntentDefinition<I>`. The intent runtime converts those
consequences into owner-specific produced facts and submits them to the
existing 3.12 observation compiler. It cannot mutate the graph, mounted state,
projection cache, or renderer directly.

All consequences of one terminal outcome form one immutable consequence batch.
The 3.12 owner either admits the complete batch into one turn/publication or
returns the typed `completed-effect/consequence-stop` posture; partial UI
publication is forbidden. Retry after that posture may retry consequence
admission, never blindly repeat the completed product effect.

Query-backed change must first settle through Query's own mutation/execution
and projection boundaries. Worth UI consumes the resulting admitted Query
observation exactly as in 3.13.

### Evidence explains and never advances

Compact evidence correlates source observation, presentation, target,
interaction, definition, payload revision, operability decision, admission,
attempt, product outcome, consequence, publication, and mounted posture.
Rich detail is lazy, scoped, bounded, redactable, and disposable.
Ordinary retention records semantic interactions and attempt transitions, not
every pointer-motion/preedit observation. Summary/evidence rings have explicit
entry and byte capacities. The canonical outcome always retains the minimum
authority/correctness/recovery core. Rich-detail overflow leaves a typed
omission and never backpressures input or execution authority.

Raw payload values are not ordinary evidence. Challenge/attempt retention uses
the payload's classification, scope, byte budget, expiry, and disposal policy;
inspection receives schema, posture, and redacted references unless separate
disclosure authority permits materialization.

Inspection and lifecycle observations cannot construct a target, interaction,
challenge, admission, completion, or consequence.

### Cost follows the admitted route

After mounted hit-test adjudication, `(mounted incarnation, interaction
family)` resolves at most one preindexed route. Ambiguous routes deny during
candidate preparation. Required ordinary work is:

```text
target adjudication    O(h) for h bounded hit-test candidates
route resolution       O(1) after target adjudication
draft update           O(b) for b changed admitted text bytes
payload assembly       O(k + p) for k inputs and p admitted payload bytes
operability            O(d) for d declared dependencies
attempt lookup/update  O(1) by compact generational slot
consequence narrowing  O(c + a) for c consequences and a indexed consumers
retained evidence      O(s) for admitted semantic/terminal records, never motion volume
```

Counters separately expose hit candidates, route rows resolved, inputs read,
operability dependencies, challenges, reservations, provider calls,
settlements, consequences, affected consumers, publications, semantic evidence
records, and omitted detail. A generic work counter or elapsed-time threshold
cannot prove these slopes.

Retained memory is `O(r + i + g + q + m + e)`: route rows `r`, definitions and
providers `i`, bounded gesture/draft slots `g`, live attempts/challenges `q`,
their admitted bytes `m`, and evidence `e`. Route count may scale with controls;
definition/provider and generic execution state may not.

## Public Developer Experience

The Rust shape is normative at the semantic level:

```rust
struct AdvanceStatus;

impl UiIntent for AdvanceStatus {
    type Payload = AdvanceStatusPayload;
    type ProductOutcome = AdvanceStatusOutcome;
    const ID: UiIntentId = UiIntentId::stable("platform.pulse.advance");
}

let advance_route_declaration = UiIntentDeclaration::<AdvanceStatus>::activate()
    .payload_from(advance_payload_sources)
    .operability_from(advance_operability_sources)
    .confirmation(advance_confirmation_policy)
    .concurrency(UiIntentConcurrencyScope::TargetSingleFlight)
    .consequences(advance_consequences)
    .into_rust_authored_input()?;

let app = WorthUi::app()
    .register_intent_definition(
        UiIntentDefinition::<AdvanceStatus>::application_effect(),
    )?
    .register_intent_provider::<AdvanceStatus>(PulseActions::new(domain_port))?
    .with_rust_authored_input(advance_route_declaration)?
    .freeze()?;
```

`PulseActions` implements the typed provider port outside the control and host
adapter. The exact builder names may follow existing facade conventions, but
implementation may not weaken the type relationship among definition,
payload, provider, outcome, and consequences.

The direct DSL shape is:

```text
intent platform.pulse.advance_route {
  definition platform.pulse.advance
  interaction activate
  payload {
    revision from projection platform.pulse.revision
  }
  operability from platform.pulse.advance_operability
  confirmation from platform.pulse.advance_confirmation
}

control platform.pulse.advance_button {
  interaction activate routes platform.pulse.advance_route
}

control platform.pulse.confirm_button {
  interaction activate confirms platform.pulse.advance_route
}
```

The DSL names semantic references only. It cannot author executor code,
callbacks, host events, Query mutations, retry loops, confirmation booleans,
or renderer-local payload assembly. Expressions and pleasant reusable
composition remain 3.17-3.18 work.

## Compile-Time and Mechanical Enforcement

- Governed phase types have private fields/constructors and are consumed by
  value: targeted interaction, prepared payload, operability proof,
  confirmation challenge, admitted intent, reservation, completion, and
  consequence admission.
- `UiIntentDefinition<I>` and `UiIntentExecutionProvider<I>` share the concrete
  intent type `I`; mismatched payload/outcome providers do not compile.
- File source cannot mint `UiIntentDefinition<I>`; declarations and route
  bindings resolve through a registered definition and its exact schemas.
- Product routes and confirmation-continuation routes are distinct types;
  neither can be passed where the other is required.
- Shape-specific draft, selection, submit, and activation inputs expose only
  lawful next operations.
- Public outcomes are exhaustive and `#[must_use]`; wildcard translations over
  operability, execution, or recovery topology are forbidden.
- Raw host observations cannot call intent admission. Only the interaction
  owner can mint the sealed semantic interaction.
- Diagnostics, inspection receipts, lifecycle events, serialized artifacts,
  digests, strings, coordinates, graph IDs, and transient allocation policy
  implement no conversion into operational types.
- `worth-ui-runtime`, host contract, host adapters, and DSL do not import Query
  mutation authority. Application providers use audience facades at the
  composition root.
- Boundary and topology checks forbid renderer/adapter callbacks, the
  cloneable string intent placeholder, static always-admitted readiness as
  authority, direct graph/mounted mutation from completion, and a second
  interaction queue/executor.
- Compile evidence remains in the existing two Cargo sessions with positive
  twins. Production compilation and runtime/model tests enforce private
  typestate that has no public misuse value; compile tests are not multiplied
  to police implementation shape.

## Architectural Destination

### Ownership

| Owner | Owns | Excludes |
| --- | --- | --- |
| `worth-ui-dsl` | authored intent specs, interaction routes, and sealed payload/operability/confirmation references | runtime declarations, capability definitions, callbacks, executor code, Query mutation |
| `worth-ui-runtime` intent capability registry | compiled definition schemas and execution-destination identity | authored routes, provider effects |
| `worth-ui-runtime` interaction subsystem | gesture/draft compilation and presentation-bound targeting | product intent or effects |
| `worth-ui-runtime` intent subsystem | canonical declarations/routes, payload projection, operability, confirmation, UI admission | domain mutation authority |
| `worth-ui-runtime` execution subsystem | bounded attempts, provider port, settlement, recovery, consequence handoff | provider/domain truth |
| application composition root | typed provider registration and product/domain port wiring | control/renderer callbacks |
| `worth-ui-host-contract` / adapters | native input mechanics and observation transport | hit-test, intent, operability, payload, completion |
| `worth-ui-inspection` | bounded immutable interaction/intent evidence | operational construction |
| Platform Pulse | real product definition, provider, and external product inputs | test injection or framework authority |
| `worth-ui-certification` | adversarial worlds and anti-bypass proof | production truth |

### Destination tree

`create` means populated now; committed successors are destinations only and do
not require empty placeholders.

```text
workspaces/worth-ui/
  crates/
    worth-ui-runtime/src/
      capability/registry/intent/                 [create: compiled capability]
        {mod,definition,identity,payload_schema,
         result_schema,execution_destination}.rs
      declaration/intent/                         [create]
        {mod,declaration,route_binding,confirmation_route_binding,payload_source,
         operability_contract,confirmation_contract,
         consequence_contract}.rs
      runtime/interaction/                        [create: observation -> meaning]
        source/{mod,host_observation}.rs
        targeting/{mod,presented_frame,continuity}.rs
        gesture/{mod,pointer,keyboard,stop}.rs
        draft/{mod,session,text_input,ime,selection}.rs
        semantic/{mod,interaction,route_source}.rs
      runtime/intent/                             [create: UI admission authority]
        payload/{mod,projection,receipt,stop}.rs
        operability/{mod,axes,decision,proof}.rs
        confirmation/{mod,challenge,continuation,stop}.rs
        admission/{mod,prepared,admitted,stop}.rs
      runtime/intent_execution/                   [create: managed effects]
        {mod,reservation,attempt,provider,settlement,recovery,consequence}.rs
      fact_contract/produced/intent.rs             [create]
      mounting/projection/intent_posture/          [create]
        {mod,completion,table}.rs
      capability/registry/command/
        {command_runtime_intent_binding,
         command_readiness_binding}.rs             [replace; no legacy lane]

    worth-ui-host-contract/src/
      observation_report/{payload,family,batch}.rs [modify]
      observation_report/text_input.rs             [create]
    worth-ui-host-egui/src/adapter/
      input_observation/                           [create]
        {mod,pointer,keyboard,text_ime}.rs
    worth-ui-dsl/src/
      semantic/intent/                             [create]
        {mod,declaration,interaction_route,payload_source}.rs
      source/{parse,lower}/                        [modify]
    worth-ui/src/facade/
      intent.rs                                    [create: stable product facade]
    worth-ui-inspection/src/
      intent/                                      [create]
        {mod,interaction,admission,attempt}.rs
    worth-ui-certification/tests/application_contracts/
      intent/                                      [create inside existing target]
        {gesture,operability,payload,confirmation,execution,
         ordering,cost,lifecycle}.rs

  apps/platform-pulse/
    app/main.wui                                   [modify]
    intent_samples/                                [create: external product input]
      {ready,confirmation-required,denied}.json
    src/
      intent/{mod,definition,provider,product_input}.rs [create]
      native_frame/input.rs                        [create]
      observation_contract/intent.rs               [create]
    tests/executable_world/                        [modify `IA-01`; same target]
```

The stable axes are declaration meaning, mechanical-to-semantic interaction,
UI admission authority, managed execution lifecycle, host mechanics, and
derived evidence. Do not flatten them into `interaction.rs`, `intent_manager`,
`callbacks`, `handlers`, `common`, or the session composition root.
`runtime/interaction/source` is the source-authority axis: host observation is
its current member; 3.15 command/accessibility service sources enter as typed
siblings rather than forged mounted interactions.

Committed successors enter additively:

- 3.15 service providers consume admitted typed service requests and add
  portal/focus/motion/command/scroll/selection execution beside product
  providers;
- 3.16 appearance consumes mounted interaction/operability posture;
- 3.17 expressions may produce typed payload/operability inputs;
- 3.19-3.22 diagnostics, replay, and inspector surfaces consume retained
  evidence; and
- Milestones 5-6 broaden workflows/forms/data products without replacing the
  admission or effect topology.

## Ordered Phases

### Phase 1: Contract, topology, and producer freeze

Freeze `IA-01`-`IA-13`, definition/declaration/route separation, typed public
DX, operability axes, phase/outcome topology, text/IME units, budgets, execution
destinations, destination tree, and Query/domain non-authority rule. Prove
native eframe event reachability, freeze required translator/capability
contracts, and prove Query/domain audience seams; add dependency/topology/
compile enforcement and replace placeholder public contracts before behavior
broadens.

Phase 2 may trust one typed destination with no callback or string-authority
compatibility path.

### Phase 2: Native input and semantic interaction

Translate real egui pointer/key/text/IME input into the existing loss-aware
host observation contract, including composition phase and coordinate
conversion. Build exact presented-frame targeting/continuity, gesture model,
capture/loss behavior, bounded draft sessions, and semantic interaction
receipts. Version/negotiate the changed observation schema before retention.
Egui advertises pointer/key/text/IME support only for installed,
production-proved translators; unsupported families derive typed operability
stops. Close `IA-02`, `IA-03`, and the interaction half of `IA-11`.

Phase 3 may trust semantic interactions that carry no intent or effect
authority.

### Phase 3: Payload, operability, confirmation, and admission

Register compiled definitions, converge file/Rust declarations and compact
routes, project coherent typed payloads, derive the orthogonal operability
proof, issue exact confirmation challenges, and admit one move-only UI routing
attempt. Close `IA-04`-`IA-06`, route/payload scale in `IA-10`, and applicable
compile twins in `IA-13`.

Phase 4 may trust one exact admitted attempt and no product effect.

### Phase 4: Managed execution and consequence integration

Register typed application providers, reserve bounded attempts, execute and
settle every terminal posture, preserve partial/indeterminate recovery, and
submit declared consequences through 3.12. Close `IA-07`-`IA-09` and execution
lifecycle/census evidence without bypassing Query/domain admission.

Phase 5 may trust the complete in-process interaction-to-visible-consequence
path.

### Phase 5: Platform Pulse, cost, documentation, and deletion

Pass the inherited executable world, cost and lifecycle tests, continuing docs,
placeholder/callback deletion checks, and constitutional gates on the final
commit. Keep the ordinary warm lane below 60
seconds and the Pulse journey at or below 45 seconds. Put 65,536-control/input
storms and long pressure schedules in the named closure-stress lane when they
would damage ordinary iteration; do not add a target, binary, nested Cargo
invocation, or compiler session.

## Documentation Deliverables

| Document | Required continuing truth | Verification |
| --- | --- | --- |
| `workspaces/worth-ui/README.md` and `AI_README.md` | Product discovery path and separation of interaction, UI admission, provider execution, and domain admission. | Compiled facade example and documentary topology audit. |
| `workspaces/worth-ui/docs/interaction-and-intents.md` | Native input/IME units, gesture/draft semantics, definition/declaration/routes, concurrency, confirmation, version coexistence, typed outcomes, recovery, cost, and anti-patterns. | Compiled examples plus `IA-02`-`IA-08`. |
| `workspaces/worth-ui/docs/application-lifecycle.md` | Exact cumulative Pulse launch, native actions, visible postures, external gate, receipts, denial, rebind cancellation, and executable command. | `IA-01`. |
| `workspaces/worth-ui/docs/architecture.md` and `runtime-subsystems.md` | Owners, phase flow, provider/effect boundary, consequence handoff, and 3.15 insertion. | Topology/dependency audit. |
| `workspaces/worth-ui/docs/inspection.md` | Compact intent evidence, retention/disclosure, resource expiry, and non-authority. | `IA-12`. |
| `_docs/worth-ui/worth-ui-dsl-vision.md` | Interaction-versus-intent distinction and direct authoring shape. | DSL/Rust convergence evidence. |
| `_docs/worth-ui/ai-diagnostics.md` | Intent evidence family and causal trace without diagnostic authority. | Inspection queries and anti-readmission tests. |
| `_docs/worth-ui/worth_ui_roadmap.md` | Correct 3.14 scope and 3.15 service handoff. | Documentary consistency audit. |

## Must Ship and Preserve

Ship the typed semantic interaction, intent definition/declaration/route,
coherent payload, orthogonal operability, affine confirmation, UI admission,
managed execution, terminal/recovery, declared consequence, mounted posture,
evidence, native egui input, Pulse, and anti-bypass contracts above.

Preserve all closed 3.10-3.13 guarantees, especially exact mounted/presentation
identity, loss-aware host observations, one 3.12 observation/publication path,
Query-owned projection and mutation authority, predecessor truth on denial,
Query-free zero cost, unchanged-frame zero semantic work, bounded inspection,
and the single Pulse/test/compile topology.

## Acceptance and Successor Handoff

Milestone 3.14 is complete when its interaction, intent, lifecycle, recovery,
cost, inspection, and anti-bypass behaviors have honest, risk-proportionate
tests; every governed resource reaches zero; public examples compile;
continuing docs agree; format, strict lint, line-cap, topology, boundary,
agent-context, ordinary certification, compile-contract, and executable-world
checks are green on the final commit; and code review finds the evidence
adequate.

It does not implement a portal/dialog system, focus traversal, command
shortcuts/routing, motion, broad selection/scroll services, rich appearance,
arbitrary expressions, workflow orchestration, general form products, replay,
or the final inspector.

Milestone 3.15 may trust that a native human action already becomes one exact
admitted typed request with honest lifecycle and consequences. It adds service
providers and service-specific semantics; it may not reopen targeting,
operability, confirmation, generic attempt management, or consequence
publication.
