# WORTH Signal Wasm Gap Report

This document records product-surface and runtime-surface gaps discovered while
using and evaluating the current `worth-signals-wasm` library.

The goal is not to blame application code for every failure mode. Some observed
failures were triggered by local workarounds, but those workarounds also exposed
places where the library likely lacks a first-class, honest authoring lane.

This report currently tracks twenty-one gaps:

1. mutation-response finalizer / semantics / reconciliation drift
2. optional resource line consumption for React
3. re-entrant runtime reads and unstable React snapshot consumption
4. missing first-class managed transient write execution
5. missing standard resource-to-UI view-state bridge
6. fallback settlement exists, but recovery policy is too manual
7. missing first-class resource catalog / registry convention
8. form runtime is stronger than its React ergonomics layer
9. no first-class subscription-based "await settlement" lane
10. no standard "toast from write lifecycle" bridge
11. fallback severity and policy remain too raw for app UX decisions
12. API-scope creation is still too weakly modeled
13. local scope authoring is still too manual
14. managed local feature store remains an app-invented layer
15. history surfaces are rich but not first-class reactive consumables in React
16. runtime compatibility and contract introspection are not first-class
17. core async is first-class, but the web/wasm async consumption surface is still fragmented
18. router web-session orchestration is still too manual
19. route sequence simulation and playback are still app-authored
20. feature store state typing is too narrow for ordinary object-shaped app state
21. local dialog state is too small for real modal CRUD workflows

## Gap 1: Mutation Finalizer / Semantics / Reconciliation Drift

### Problem Statement

We found one concrete failure where the runtime rejected a mutation-response
target because the authoring lane and the reconciliation lane disagreed:

- transport / finalizer lane: `.create(...)`
- reconciliation lane: "replace an existing collection member"
- runtime response: hard rejection at route lowering

The bigger issue is not that single case. The likely broader gap is that the
current surface may be coupling together three concerns that need to be
separable:

- transport verb or request shape
- semantic mutation intent
- reconciliation capability actually declared and proven

If those are coupled too early, then legitimate API shapes can become
unrepresentable or can be admitted by one layer and rejected by another.

### Why This Looks Like A Real Library Gap

The current public surface teaches the common write lane through:

- `.create(...)`
- `.update(...)`
- `.remove(...)`

That strongly suggests those finalizers are carrying semantic meaning, not just
transport shape.

But real APIs often do not map cleanly to those buckets:

- `POST` used for update-like domain actions
- `POST` used for association or link writes
- `DELETE` used for relationship removal without enough topology proof
- `PUT` used for create-or-replace
- `POST` used for command endpoints that only return status

If the finalizer is acting as both transport vocabulary and semantic mutation
family, the surface becomes brittle for command-style and nonstandard APIs.

### Axes That Must Be Audited Separately

The audit should treat these as three separate axes:

#### Transport verb

- `POST`
- `PUT`
- `DELETE`

#### Semantic mutation intent

- create a new visible topology member
- update an existing visible member
- remove an existing visible member
- mutate an aggregate or relationship without creating a new visible item
- trigger a side effect that only yields fallback truth

#### Reconciliation capability actually declared / proven

- detail replace
- detail field
- detail region
- detail path
- collection insert
- collection item replace
- collection delete
- summary patch
- identity migration
- fallback-only

### Likely Pressure Point

The likely pressure point is that the current surface may couple these too
tightly:

- `.create(...)` implies create semantics
- `.update(...)` implies update semantics
- `.remove(...)` implies delete semantics

That may be acceptable for the dominant happy path, but it is likely too rigid
for APIs where:

- the HTTP method is transport detail
- the domain action is not a clean CRUD category
- the response proof only supports fallback truth for some or all related
  targets

### Concrete Audit Questions

The audit should answer the following:

1. Is `.create/.update/.remove` meant to describe transport method, semantic
   mutation family, or both?
2. Which reconciliation kinds are currently gated by finalizer family?
3. Which combinations are semantically valid in principle but currently blocked
   because of finalizer choice?
4. Are fallback-only command routes first-class, or just tolerated edge cases?
5. Are there places where the type surface admits a declaration shape that the
   runtime lowerer later rejects?
6. Are there places where the runtime would support a combination that the type
   surface cannot express?

### Specific Combinations To Investigate

These combinations should be checked explicitly:

- `.create(...)` + collection item replace
- `.create(...)` + detail replace against an already existing detail
- `.create(...)` + detail field/region/path patching of existing truth
- `.remove(...)` + summary-only reconciliation
- `.update(...)` + insert-like topology change
- command-style `POST` writes that should admit fallback-only targets without
  pretending to be create/update/delete in visible topology terms

### Fallback-Only Command Routes

This is an especially important audit lane.

The library should be checked for Ã¢â‚¬Å“status-only command endpointsÃ¢â‚¬Â where the
honest outcome is:

- accepted write line
- fallback-only related targets
- diagnostics clearly explaining why exact related reconciliation was not
  admitted

These routes should be first-class, not second-class oddities.

### Type / Runtime Parity Audit

This should be a full parity audit across:

- create target declarations
- update target declarations
- remove target declarations
- identity migration declarations
- fallback-only target declarations

Each combination should be classified as:

- supported
- denied by design
- denied accidentally because of finalizer coupling
- admitted by type surface but rejected by lowering
- supported by lowering but not expressible in the type surface

### Working Theory

The likely product gap is:

The response finalizer currently encodes semantic mutation class too early,
before reconciliation legality is evaluated independently.

If that theory is correct, the long-term fix probably involves separating:

- transport shape
- mutation semantic class
- reconciliation plan declaration

without lying about any of the three.

## Gap 2: Optional Resource Line Consumption For React

### Problem Statement

The current docs talk a lot about exact versus fallback reconciliation, but they
do not appear to present a first-class Ã¢â‚¬Å“optional resource lineÃ¢â‚¬Â or Ã¢â‚¬Å“disabled
line subscriptionÃ¢â‚¬Â pattern for React.

The visible type surface appears to give us shapes like:

- `family.line(params)`
- `line.signal()`
- `useSignalValue(...)`

but not something like:

- `family.lineIf(...)`
- `line.optional(...)`
- `enabled: false`
- `useMaybeResourceLineValue(...)`

That creates a real ergonomic problem because React requires hook-order
stability, while resource selection is often conditional by domain state.

### Why This Looks Like A Real Library Gap

When a detail line is selected conditionally, app authors are forced into one of
these patterns:

- conditional hooks
- fake or sentinel params
- synthetic fallback signals
- component boundary split

Those each have problems:

- conditional hooks are invalid in React
- fake params can create bad requests or bad identity
- synthetic fallback signals are workaround-ish rather than first-class
- component boundary split is the cleanest current answer, but creates
  structural churn for a common case

The immediate null-pointer failure came from a bad workaround, but the pressure
that led to the workaround seems like a library ergonomics gap.

### Desired Capability

The library likely needs a first-class way to consume a resource family in React
when the line is conditionally absent by domain state.

Required properties:

- hook-order safe
- no fake network request
- no synthetic null or sentinel signal hacks required by app code
- explicit Ã¢â‚¬Å“inactive / no selection / not mounted by intentÃ¢â‚¬Â posture
- diagnostics that distinguish Ã¢â‚¬Å“inactive by author intentÃ¢â‚¬Â from Ã¢â‚¬Å“loadingÃ¢â‚¬Â,
  Ã¢â‚¬Å“errorÃ¢â‚¬Â, Ã¢â‚¬Å“fallbackÃ¢â‚¬Â, or Ã¢â‚¬Å“staleÃ¢â‚¬Â

### Concrete Audit Questions

The audit should answer:

1. Is there already a hidden or underdocumented inactive-line pattern in the
   current surface?
2. Does the non-React surface already support an explicit inactive posture that
   the React adapter fails to expose?
3. Is component boundary splitting the intended official answer today?
4. Does the diagnostics surface have vocabulary for Ã¢â‚¬Å“intentionally inactiveÃ¢â‚¬Â?
5. Does `family.line(...)` materialize too eagerly for conditional UI cases?

### Current Workaround Classes

These should be documented as workaround classes, not first-class solutions:

- conditional hooks
- fake params
- synthetic fallback signals
- component boundary split

The audit should explicitly state which of these are:

- invalid
- risky
- acceptable temporary guidance
- preferred until a first-class API exists

### Desired Surface Direction

This report does not lock exact naming, but the library likely needs one of
these kinds of authoring lanes:

- optional line materialization
- inactive line handle
- maybe-line read surface
- React hook that accepts an explicit enabled/inactive posture

Possible example shapes worth evaluating:

- `family.lineOptional(...)`
- `family.lineWhen(...)`
- `useResourceLine(..., { enabled })`
- `useOptionalResourceLineValue(...)`

The important part is not the exact name. The important part is the semantic
contract:

- stable hook call
- no request while inactive
- explicit inactive result
- diagnostics clarity

### Acceptance Criteria For Closing The Gap

This gap is not closed until a React consumer can honestly express Ã¢â‚¬Å“nothing is
selected right nowÃ¢â‚¬Â without:

- violating hook order
- issuing a fake request
- inventing fake identity
- building its own sentinel signal folklore

At minimum, a first-class answer should make the inactive state:

- representable
- inspectable
- distinct from loading
- distinct from error
- distinct from fallback reconciliation

## Gap 3: Re-entrant Runtime Reads And Unstable React Snapshot Consumption

### Problem Statement

We observed a runtime panic and React subscription instability during ordinary
consumption of signal values:

- Rust panic: `RefCell already borrowed`
- React warning: `The result of getSnapshot should be cached to avoid an infinite loop`
- React failure: `Maximum update depth exceeded`

The panic surfaced from:

- `crates/worth-signal-wasm/src/boundary/signals/helpers.rs`
- `read_signal_value(...)`

The visible symptoms suggest two linked problems:

1. the runtime read path may permit or trigger re-entrant reads while the core
   is mutably borrowed
2. the React adapter may not fully satisfy the `useSyncExternalStore`
   expectation that snapshots stay referentially stable when nothing meaningful
   changed

### Why This Looks Like A Real Library Gap

Even if a local app used the adapter imperfectly, ordinary React consumption
should not end in:

- a Rust panic
- unbounded snapshot churn
- infinite render/update loops

The library should either:

- make the unsafe pattern structurally impossible
- convert it into a controlled and diagnosable error
- or provide a React-safe adapter contract that prevents the pattern from
  arising in normal use

### Observed Pressure Point

The current read path in `read_signal_value(...)` appears to:

- take a mutable borrow on the shared core
- read the value
- note app signal serialization before releasing that borrow

That makes this an immediate audit question:

Can signal read bookkeeping or serialization re-enter another runtime read
before the mutable borrow is released?

If yes, `RefCell already borrowed` is not just an app bug; it is a runtime
contract hazard.

### React Adapter Concerns

The React adapter appears to use `useSyncExternalStore` with a store snapshot
path built around:

- `subscribeSignal(...)`
- `getSignalSnapshot(...)`

The symptoms suggest a possible mismatch between that adapter contract and
React's expectations:

- snapshots may not stay referentially stable
- runtime reads during snapshot evaluation may trigger feedback
- computed or output reads may cause nested read behavior that the adapter does
  not guard against

### Concrete Reproduction In The Demo Site

This gap is no longer just a theoretical wrapper concern. A minimal live demo
in:

- `apps/worth-signal-demo/src/ui/demos/DemoOne.tsx`

was converted to use real WORTH surfaces:

- `signals.input(...)`
- `signals.computed(...)`
- `signals.output(...)`
- `signals.watch(...)`
- `signals.effect(...)`

The first sharp repro appeared when we tried to surface diagnostics reads from
that same live React demo, but the problem turned out to be broader than
diagnostics. Even after removing the diagnostics reads, ordinary button clicks
still produced browser-console panics such as:

- `RefCell already borrowed` from
  `crates/worth-signal-wasm/src/boundary/signals/helpers.rs:22:33`
- `RefCell already mutably borrowed` from
  `crates/worth-signal-wasm/src/boundary/diagnostics.rs:61:30`

The repro matters because it is tiny. This was not a complicated resource flow
or a large admin screen. It was a counter demo with:

- live React subscription reads
- a `watch(...)` observer
- an `effect(...)` observer

That is strong evidence that the current web/React consumption contract is
still too easy to re-enter unsafely under ordinary composition.

### Concrete Audit Questions

The audit should answer:

1. Why does `read_signal_value(...)` require `borrow_mut()` rather than an
   immutable read plus a separate bookkeeping path?
2. Can `note_app_signal_serialization(...)` trigger re-entrant signal reads,
   directly or indirectly?
3. Does `signals.read(...)` produce fresh JS object identity even when the
   logical signal value has not changed?
4. Can `getSignalSnapshot(...)` trigger nested reads that re-enter the runtime
   in an unsafe way?
5. Are computed, output, and resource-backed signals all safe under the same
   React adapter contract?
6. Are there conditions where the watch/notify path and the snapshot path form
   a feedback loop?

### Desired Outcome

The library should provide a React consumption contract where:

- ordinary reads never panic the runtime
- snapshots are stable when meaningful value did not change
- unsupported re-entrant patterns fail in a controlled way
- React users do not have to reverse-engineer which signal kinds are safe to
  read through the adapter

### Acceptance Criteria For Closing The Gap

This gap is not closed until:

- React adapter usage no longer triggers `RefCell already borrowed` panics in
  ordinary consumption flows
- `useSyncExternalStore` consumers no longer hit snapshot-instability warnings
  during correct usage
- the adapter preserves stable snapshot results when no meaningful change
  occurred
- runtime read behavior is either safely re-entrant where needed or explicitly
  guarded with diagnosable failure modes

## Current Recommendation

All three gaps appear real enough to deserve explicit audit work rather than
being treated as isolated application mistakes.

Priority recommendation:

1. audit mutation finalizer / semantic intent / reconciliation coupling as a
   product-surface and lowering-surface parity problem
2. audit optional resource line consumption as a first-class React ergonomics
   gap
3. audit re-entrant runtime reads and React snapshot stability as a
   runtime/adapter contract gap

These gaps hit different layers of the library:

- Gap 1 is deeper architectural and affects mutation-response honesty
- Gap 2 is a frequent developer-experience gap for ordinary React usage
- Gap 3 is a correctness and adapter-contract gap that can surface as runtime
  panic plus React render instability

## Gap 4: Missing First-Class Managed Transient Write Execution

### Problem Statement

The library gives us strong transient write primitives, but it does not appear
to provide a high-level React-oriented execution helper for the routine
lifecycle of a write operation.

In practice, each mutation required substantial repeated ceremony:

- create a transient write line
- expose pending state to the UI
- await settlement
- interpret exact success vs fallback vs failure
- optionally revalidate resident lines
- show toast or operation feedback
- free the transient line

### Why This Looks Like A Real Library Gap

This pattern is not an edge case. It is likely to repeat across every serious
application unless the library offers a stronger day-to-day lane.

In our integration, we had to build hooks that manually coordinated:

- transient line construction
- pending state
- settlement waiting
- fallback detection
- resident revalidation
- user feedback
- line cleanup

That is a sign that the current surface is powerful but underproductized for
routine app usage.

### Example Pressure Pattern

This is the shape app code naturally wanted:

```ts
const createProject = useManagedResourceWrite({
  line: (body) => catalog.projects.createProject.line({ body }),
  onPartial: ({ resident }) => resident.projects.revalidate(),
  toast: {
    success: "Project created",
    partial: "Project created, refreshing list",
    error: "Unable to create project",
  },
});
```

The point is not that this exact API must exist. The point is that the library
appears to lack a first-class lane for "execute a transient resource write and
manage its normal application lifecycle honestly."

### Concrete Audit Questions

The audit should answer:

1. Is the current expectation that every app will hand-roll transient write
   orchestration?
2. Is there already a hidden or undocumented helper lane for write execution
   lifecycle management?
3. Which parts of transient write execution are intentionally left to app code,
   and which are good candidates for a standardized helper?
4. Can the library expose settlement, cleanup, and fallback follow-up in a way
   that reduces ceremony without lying about runtime truth?

### Desired Outcome

The library should likely provide a stronger execution helper lane that can
standardize:

- pending state exposure
- settlement reading
- cleanup and `free()` ownership
- partial/fallback handling hooks
- optional resident revalidation hooks
- routine feedback integration points

### Acceptance Criteria For Closing The Gap

This gap is not closed until an app can execute a transient write in a routine
React workflow without having to re-author the same lifecycle wrapper for every
feature area.

## Gap 5: Missing Standard Resource-To-UI View-State Bridge

### Problem Statement

Resource lines expose rich runtime truth, but ordinary application UI often
needs a simpler view model:

- loading
- refreshing
- ready
- empty
- error

We had to build our own adapters to convert line/runtime truth into this more
ordinary presentation state.

### Why This Looks Like A Real Library Gap

Without a standard bridge, every app is forced to invent its own interpretation
of questions like:

- what exactly counts as `loading`?
- when is stale visible data plus a new request considered `ready` vs
  `refreshing`?
- how should error state coexist with prior visible data?
- how should empty state be layered on top of resource truth?

Those are common app-facing questions, and a mature product surface usually
offers a standard lane for them.

### Example Pressure Pattern

Our app had to author wrapper functions to map line summary truth into UI
content-state shapes such as:

- `loading`
- `error`
- `ready`
- `ready` with `isRefreshing`

The app-level shape was useful, but the fact that every app would need to
invent this independently suggests a missing standard bridge.

### Concrete Audit Questions

The audit should answer:

1. Does the library already have a hidden or underdocumented view-state bridge?
2. Is the current philosophy that apps should always own this mapping
   themselves?
3. Which runtime truth concepts are stable enough to project into a standard UI
   view-state helper?
4. Can a bridge be provided without collapsing away the richer exact/fallback
   runtime truth that makes the library valuable?

### Desired Outcome

The library should likely provide a standard app-facing projection from
resource/runtime truth into a conventional UI view-state helper, while still
allowing richer consumers to inspect the deeper line truth directly.

### Acceptance Criteria For Closing The Gap

This gap is not closed until apps can consume resource lines through a
first-class view-state bridge rather than re-authoring their own loading /
refreshing / ready / empty / error mapping in each codebase.

## Gap 6: Fallback Settlement Exists, But Recovery Policy Is Too Manual

### Problem Statement

The library can tell us whether a write settled as:

- exact success
- partial or fallback success
- failure

But after a fallback settlement, application code still has to decide too much
policy manually:

- which resident line to revalidate
- whether to preserve or clear selection
- whether optimistic or draft state should remain visible
- what user feedback to show
- whether the UI should close, remain open, or move to a different state

### Why This Looks Like A Real Library Gap

The library already exposes meaningful settlement truth, but it does not appear
to offer enough first-class recovery-policy structure on top of that truth.

As a result, app code repeatedly has to invent patterns such as:

- on partial settlement, revalidate one collection
- on partial settlement, revalidate detail plus collection
- clear current selection only if it matches the deleted or mutated subject
- preserve or discard local UI posture depending on operation type

That is not a correctness bug, but it does look like a missing policy layer.

### Concrete Audit Questions

The audit should answer:

1. Is fallback recovery policy intentionally left entirely to app wrappers?
2. Are there existing library hooks for catalog-authored or family-authored
   fallback recovery policy?
3. Which recovery patterns are common enough to deserve a first-class helper?
4. Can recovery hints be attached to resource authoring without obscuring the
   actual settlement truth?

### Desired Outcome

The library should likely support a clearer recovery-policy lane for fallback
settlement, whether through execution helpers, catalog conventions, or
authoring-time recovery hints.

### Acceptance Criteria For Closing The Gap

This gap is not closed until fallback settlement can be paired with a more
standard, honest recovery policy than "every app writes its own follow-up
rituals."

## Gap 7: Missing First-Class Resource Catalog / Registry Convention

### Problem Statement

Once an application grows beyond a few resource families, it naturally needs:

- one runtime-scoped resource catalog
- grouped domain families
- a shared API scope
- stable cached family construction
- predictable discovery and debug points

We had to invent our own catalog pattern around WeakMap-scoped caching and
domain grouping.

### Why This Looks Like A Real Library Gap

This is not just local taste. It is a predictable architecture need for any
serious application with multiple resource domains.

Without a first-class convention, every app will end up inventing some version
of:

- catalog construction
- runtime-scoped caching
- domain grouping
- stable family access
- discovery points for debugging

That is a sign the library may be missing a standard architecture pattern for
resource authoring at scale.

### Concrete Audit Questions

The audit should answer:

1. Does the library already recommend a standard resource-catalog architecture?
2. Is ad hoc app-defined catalog layering the intended long-term pattern?
3. Which parts of catalog construction belong naturally to the library instead
   of app code?
4. Can a first-class registry or catalog helper be introduced without forcing
   one rigid domain topology?

### Desired Outcome

The library should likely provide a first-class catalog or registry convention
for resource families that supports:

- runtime-scoped caching
- grouped domain construction
- stable access patterns
- clear integration with React/runtime ownership

### Acceptance Criteria For Closing The Gap

This gap is not closed until serious apps no longer have to invent their own
resource catalog conventions from scratch just to organize and cache family
authoring sanely.

## Gap 8: Form Runtime Is Stronger Than Its React Ergonomics Layer

### Problem Statement

The form runtime appears to have strong underlying concepts:

- readiness
- actions
- patch plans
- visible validation and messaging

But to make those concepts usable in real modal and CRUD-heavy React forms, we
still had to build a substantial wrapper layer ourselves.

That wrapper had to bridge concerns such as:

- field binding
- checkbox/select/multiselect binding
- blur-driven visible errors
- dialog hydration and reset
- action-style submit state

### Why This Looks Like A Real Library Gap

The problem is not that the form core looks weak. The problem is that the React
ergonomics layer still appears too thin for normal product-team usage.

Without a stronger adapter surface, teams will naturally:

- build their own form facades
- duplicate binding logic
- approximate interaction visibility policy
- risk bypassing the richer form model entirely in favor of simpler local glue

That makes this an adapter maturity gap rather than evidence against the form
runtime itself.

### Concrete Audit Questions

The audit should answer:

1. What level of React form adapter does the library intend to provide?
2. Is wrapper authorship considered the expected path today for all nontrivial
   forms?
3. Which form interaction patterns are stable enough to deserve first-class
   React bindings?
4. Can the adapter expose richer form runtime truth without collapsing back
   into a shallow field-library model?

### Desired Outcome

The library should likely provide a stronger React ergonomics lane for forms,
covering common real-world interaction surfaces while still preserving the
runtime's stronger readiness, action, and patch semantics.

### Acceptance Criteria For Closing The Gap

This gap is not closed until a typical CRUD or modal form can consume the form
runtime through a first-class React adapter layer rather than forcing each app
team to build its own shared facade.

## Gap 9: No First-Class Subscription-Based "Await Settlement" Lane

### Problem Statement

During integration, we had to implement mutation settlement waiting by polling
line status with `setTimeout`.

That is a strong smell. It suggests the library does not expose a clean enough
reactive or awaitable lifecycle for transient writes and operation lines.

This stands out even more because the wider runtime already has strong native
async posture in other surfaces. If an app still has to poll `line.status()` in
a loop just to await a transient write, then the settlement lifecycle is not
surfaced honestly enough for ordinary application use.

That is not just a local taste complaint. It cuts directly against the core
milestone intent in `worth-signal`:

- Milestone B says async/resource lifecycle must become runtime-owned derived
  truth so pending, fulfilled, rejected, cancelled, stale, superseded,
  retried, and timed-out states stop being adapter-local conventions, in
  `plans/worth_signal/milestone-b-plan.md`
- Milestone B also says if async/resource state remains a UI adapter state
  machine, the milestone is incomplete, in
  `plans/worth_signal/milestone-b-plan.md`
- Milestone C says if route resources, query views, form actions, or browser
  adapters still need to invent retry, timeout, visibility, retention, or
  freshness semantics above the runtime, the milestone is incomplete, in
  `plans/worth_signal/milestone-c-plan.md`
- Milestone D says async should read like a node capability rather than a
  parallel subsystem, in `plans/worth_signal/milestone-d-plan.md`

The codebase already proves that the product surface is comfortable exposing
first-class async semantics elsewhere:

- `createSignals(...)` returns a `Promise<CallableSignals>` in
  `crates/worth-signal-wasm/package/types/callable_surface.d.ts`
- async signal authoring is first-class through:
  - `inputAsync(...)`
  - `linkedAsync(...)`
  - `computedAsync(...)`
  - `outputAsync(...)`
  in `crates/worth-signal-wasm/package/types/callable_surface.d.ts`
- async graph/runtime work is first-class through:
  - `transactionAsync(...)`
  - `batchAsync(...)`
  in `crates/worth-signal-wasm/package/types/callable_surface.d.ts`
- history operations already admit `Promise` results through:
  - `create_branch(...)`
  - `switch_branch(...)`
  - `restore_snapshot(...)`
  - `merge_branches(...)`
  - `plan_merge_policy_preview(...)`
  and related methods in
  `crates/worth-signal-wasm/package/types/callable_surface.d.ts`
- resource-backed form exact history is already documented as `await`-driven:
  - `await form.replayExactResourceSource(...)`
  - `await form.restoreExactResourceSource(...)`
  in `crates/worth-signal-wasm/docs/forms/resource-backed/replay-and-restore.md`

So the problem is not that WORTH avoids native async APIs. The problem is that
resource-operation settlement is still forcing app authors down to polling,
which looks especially out of place given how many adjacent surfaces already
have an explicit awaitable contract.

### Why This Looks Like A Real Library Gap

Without a first-class awaitable lane, every app is pushed toward reinventing:

- polling cadence
- timeout behavior
- settlement interpretation
- error conversion
- cancellation behavior

Those are not feature-specific concerns. They are normal write-lifecycle
concerns that deserve a clearer standard lane.

This is especially important for serious apps because transient writes almost
always want:

- await completion
- timeout support
- rejection propagation
- and often cancellation or structured abort behavior

### Desired Outcome

The library should likely offer a first-class settlement await path, whether
that is:

- line-owned awaiting
- resource helper awaiting
- or a managed execute-write helper that subsumes waiting

The exact API shape is less important than the contract:

- no app-authored polling loop
- clear timeout and cancellation behavior
- honest settlement result

### Acceptance Criteria For Closing The Gap

This gap is not closed until transient and operation-style resource lines can
be awaited through a first-class subscription-based or runtime-native lane
rather than app polling.

## Gap 10: No Standard "Toast From Write Lifecycle" Bridge

### Problem Statement

We had to build a toast bridge ourselves by interpreting write settlement and
mapping it into success / partial / error UI feedback.

The runtime already knows meaningful lifecycle states such as:

- pending
- success
- partial
- error
- fallback detail

But there does not appear to be a first-class bridge for converting that
operation lifecycle into routine app-facing feedback policy.

### Why This Looks Like A Real Library Gap

The library does not need to own a toast system directly, but repeated
settlement-to-feedback bridging is common enough that apps will otherwise all
invent their own policy adapters.

That leads to fragmentation in:

- what "partial" means to users
- when to show a warning vs success
- how fallback reasons surface
- how write lifecycle is explained consistently

### Desired Outcome

The library should likely provide a standard write-lifecycle adapter concept
that UI systems can consume without each app rebuilding the same
settlement-to-feedback mapping from scratch.

### Acceptance Criteria For Closing The Gap

This gap is not closed until apps can bridge write lifecycle truth into
standard user feedback with less bespoke wrapper policy.

## Gap 11: Fallback Severity And Policy Remain Too Raw For App UX Decisions

### Problem Statement

The runtime exposes fallback kinds and settlement truth, but it still leaves
apps without enough higher-level help to answer routine policy questions like:

- is this safe success with delayed refresh?
- is this warning-but-fine?
- is this likely stale until reloaded?
- should the dialog close or stay open?

In practice, semantically different outcomes were flattened into a broad
`partial` bucket because the raw fallback surface was too low-level for
consistent UX policy.

### Why This Looks Like A Real Library Gap

This is not a correctness failure. The underlying truth is valuable.

The gap is that the library appears thinner on recovery severity and policy
projection than apps need for consistent product decisions.

### Desired Outcome

The library should likely provide a stronger recovery summary or policy-facing
projection on top of raw fallback truth, while preserving access to the lower
level details.

### Acceptance Criteria For Closing The Gap

This gap is not closed until apps can distinguish meaningful fallback severities
and follow-up posture without flattening everything into one custom partial
bucket.

## Gap 12: API-Scope Creation Is Still Too Weakly Modeled

### Problem Statement

We had to build our own cached API-scope helper around ad hoc keys,
`WeakMap<object, Map<string, unknown>>`, string cache identifiers, and weakly
typed options.

That suggests the library may not yet provide a strong enough convention for:

- stable scoped API instances
- typed cache identity
- cache invalidation or remount semantics
- composition of scopes

### Why This Looks Like A Real Library Gap

Serious apps usually want more than repeated direct `signals.api(...)` calls.
They want stable scope construction tied to runtime ownership and feature
topology.

If every app has to invent its own cache-key and scope factory story, the
surface is probably under-modeled.

The practical app need is predictable:

- stable scope identity
- runtime-local caching
- typed options
- deterministic reuse
- clear lifecycle semantics

### Desired Outcome

The library should likely provide a stronger API-scope convention with clearer
identity and reuse semantics so apps do not fall back to string cache keys,
`unknown`, and opaque wrapper maps.

### Acceptance Criteria For Closing The Gap

This gap is not closed until apps can author stable API scopes through a
first-class lane rather than building their own weakly modeled cache wrappers.

## Gap 16: Runtime Compatibility And Contract Introspection Are Not First-Class

### Problem Statement

During integration, we ended up writing executable runtime contract assertions
just to verify that the signals surface still matched the assumptions our
wrappers were built on.

That included checks for things like:

- required callable runtime methods
- presence of the explicit spec namespace
- basic scoped authoring behavior
- preservation of initial values and evaluation correctness

### Why This Looks Like A Real Library Gap

These checks are useful guardrails, but their existence signals a deeper
problem:

- the surface has enough motion or ambiguity that wrapper authors do not feel
  safe relying on docs and types alone
- executable compatibility probes feel necessary before building higher-order
  app foundations

That is not a correctness bug by itself, but it is a product-surface gap. A
mature downstream integration story usually includes some machine-readable way
to ask what surface family and capabilities are actually present.

### Concrete Audit Questions

The audit should answer:

1. Is there already a hidden or underdocumented compatibility descriptor on the
   runtime surface?
2. Are downstream wrapper authors expected to probe runtime behavior manually to
   establish trust?
3. Which callable/scoped authoring capabilities are stable enough to expose as
   a machine-readable contract?
4. Can the library provide compatibility introspection without freezing all
   experimentation or internal evolution?

### Desired Outcome

The library should likely expose a first-class compatibility or contract
surface, such as:

- a runtime descriptor
- a capability report
- or a built-in compatibility assertion lane

The important part is not the exact API shape. The important part is that app
wrappers can discover supported contract features without hand-authored probes.

### Acceptance Criteria For Closing The Gap

This gap is not closed until downstream wrappers can verify runtime surface
compatibility through a first-class machine-readable contract rather than
executable ad hoc probes.

## Gap 17: Core Async Is First-Class, But The Web/Wasm Async Consumption Surface Is Still Fragmented

### Problem Statement

The deeper issue exposed by settlement polling is not just that one missing
primitive. It is that the core `worth-signal` async model appears more
first-class than the current web/wasm product surface that is supposed to
deliver it.

In core milestone language:

- Milestone B makes async/resource lifecycle runtime-owned derived truth
- Milestone C says policy families above that lifecycle should also be
  runtime-owned rather than adapter folklore
- Milestone D says async should become a first-class capability attachable to
  ordinary nodes, not a separate subsystem

But on the current web-facing side, app and wrapper authors still run into a
fragmented async story:

- resource-operation settlement awaiting is missing and falls back to polling
- React integration still needs custom subscription/snapshot glue
- history-oriented UI still mirrors imperative reads into React state
- product-layer wrappers still have to interpret lifecycle truth into app
  policies by hand
- even a tiny live demo that combines React subscriptions, `watch(...)`,
  `effect(...)`, and diagnostics reads can still fall into borrow/re-entry
  panics

That suggests the web/wasm surface is still exposing async truth unevenly
across adjacent product lanes.

### Why This Looks Like A Real Library Gap

The `worth-signals-wasm` planning docs already acknowledge this pressure:

- `web_runtime_spec.md` says the package should feel native in web codebases
  and warns against apps building their own subscription layer because wasm
  does not feel natural enough
- `react_adapter_spec.md` says React should be a disciplined consumer of
  runtime truth and explicitly requires subscription/snapshot wiring derived
  from runtime observation rather than polling
- `api_surface_plan.md` says core async is already closed and the wasm API
  surface should consume that runtime-owned async capability rather than
  inventing a second async truth model

So this is not merely "the product is unfinished." The published plans
themselves say the web surface is supposed to converge on one honest async
story.

### Concrete Audit Questions

The audit should answer:

1. Which async truths are genuinely first-class on the current web surface, and
   which still require wrapper folklore?
2. Is there one coherent app-facing async-consumption model across:
   - resource lines
   - resource operations
   - forms
   - history/replay
   - React subscription
3. Where do current web APIs still force apps to reconstruct lifecycle meaning
   instead of consuming runtime-owned async truth directly?
4. Is the fragmentation mainly a missing adapter layer, or are there deeper
   mismatches in the wasm product surface itself?

### Desired Outcome

The library should likely converge toward one coherent web-facing async model
where:

- runtime-owned async lifecycle is directly consumable
- awaitable, subscribable, and inspectable lanes align
- React/web adapters consume runtime async truth rather than rediscovering it
- product layers do not need a second local async state model

This likely requires touching more than one point:

- resource-operation settlement
- React subscription/snapshot surfaces
- history consumption
- form action/submit lifecycle consumption
- app-facing async policy projections

### Concrete Example Of The Missing Shape

If the web-facing surface were consuming the async substrate at full strength,
an app should be able to write something closer to this:

```ts
const signals = await createSignals();

const api = signals.apiScope("workplace-admin", {
  baseUrl: "/api",
});

const projects = api.url("/admin/projects").collection({
  load: async () => fetch("/admin/projects").then((r) => r.json()),
});

const createProject = api.url("/admin/projects").create({
  body: ({ name, description }) => ({
    name,
    description,
  }),
});

const projectsLine = projects.line({});
const createLine = createProject.line({
  body: {
    name: "Roadmap rewrite",
    description: "Async surface hardening",
  },
});

const execution = createLine.execute();

const settlement = await execution.settled({
  timeoutMs: 15_000,
});

if (settlement.kind === "fulfilled") {
  console.log(settlement.value);
  console.log(settlement.confirmation.kind);
  console.log(settlement.lifecycle.status);
}

if (settlement.kind === "partial") {
  await settlement.recovery.revalidate([projectsLine]);
}
```

And the same runtime truth should be consumable from React without a second app
invented async state machine:

```ts
const store = createReactSignalsStore(signals);
const projectList = useResourceLine(projectsLine, store);
const createStatus = useResourceOperation(execution, store);

const form = useSignalsForm({
  source: signals.form.source.resourceLine(projectsLine.detail("project-12")),
  actions: {
    submit: createLine,
  },
});
```

This is not meant to freeze final names. The point is the shape:

- one runtime-owned execution object
- one awaitable settlement lane
- one subscribable lifecycle lane
- one recovery surface
- one React consumption story that reads the same async truth instead of
  rebuilding it

### Acceptance Criteria For Closing The Gap

This gap is not closed until a web app can consume WORTH async truth through
one coherent product surface rather than mixing:

- runtime lifecycle reads
- wrapper-authored polling
- wrapper-authored mirror state
- wrapper-authored policy projections
- framework-specific subscription folklore

## Gap 13: Local Scope Authoring Is Still Too Manual

### Problem Statement

We repeatedly had to author small local runtimes by hand using raw scope
primitives such as:

- `signals.scope(identity)`
- `scope.spec.input(...)`
- manual debug names
- manual aspect assignment
- manual disposal ownership

The primitive is powerful, but it is still quite low-level for recurring app
patterns such as:

- local dialog state
- local list state
- local form source state
- scoped feature stores

### Why This Looks Like A Real Library Gap

The issue is not that local scope primitives are bad. The issue is that
high-frequency patterns still require comfort with too much raw runtime
plumbing.

That means wrapper authors must repeatedly reason about:

- identity allocation
- scope creation
- input authoring
- aspect assignment
- lifecycle ownership

### Desired Outcome

The library should likely expose higher-order constructors or conventions for
common local-scope patterns so apps do not have to rebuild them from raw
primitives every time.

### Acceptance Criteria For Closing The Gap

This gap is not closed until common local feature-state patterns can be
authored through stronger first-class lanes than raw scope/spec plumbing.

## Gap 14: Managed Local Feature Store Remains An App-Invented Layer

### Problem Statement

We ended up building a `createScopedSignalStore(...)`-style abstraction to get
the kind of scoped feature-state model that React apps routinely want.

That pattern now appears across multiple feature areas, which suggests it is
not just app-specific taste.

### Why This Looks Like A Real Library Gap

There seems to be a missing layer between:

- raw signal primitives
- and a managed local feature-store shape with actions and scoped ownership

When many unrelated features all reinvent that same layer, it becomes evidence
of a recurring product-surface gap.

### Desired Outcome

The library should likely provide a standard helper or convention for scoped
feature-state authoring with actions, ownership, and runtime integration.

### Acceptance Criteria For Closing The Gap

This gap is not closed until apps no longer need to invent their own managed
local feature-store abstraction in order to use the runtime ergonomically at
feature scale.

## Gap 15: History Surfaces Are Rich But Not First-Class Reactive Consumables In React

### Problem Statement

The history surface appears to expose a rich imperative API:

- `current_branch()`
- `branches()`
- `create_branch(...)`
- `switch_branch(...)`
- replay, restore, snapshot, and merge helpers

But it does not appear to expose a first-class reactive or subscribable lane
for React consumption.

In practice, demo and app code end up doing the wrong kind of work:

- call imperative history methods
- manually re-read branch lists and current branch
- mirror that data into React `useState`
- refresh the mirrored state after every history mutation

That suggests the history surface is powerful, but still under-modeled for
routine React UI consumption.

### Additional Evidence From The Demo Site

The beefed-up router demo in:

- `apps/worth-signal-demo/src/ui/demos/DemoThree.tsx`

strengthened this gap rather than weakening it. To show a growing navigation
history panel, the demo had to:

- create a `signals.router.browserHistory.story()` manually
- call `story.record(...)` after every admitted boundary report
- re-read `story.current()`, `story.backProvenance()`,
  `story.breadcrumbTrail()`, and `story.admittedEntries()`
- mirror all of that into React `useState`

So even when the history story itself is the correct source of truth, the
current React/web consumption story still pushes apps toward imperative re-read
and local mirror state.

### Why This Looks Like A Real Library Gap

The issue is not a lack of history capability. The issue is that history truth
appears to stop one layer too low for UI integration.

If React consumers cannot subscribe to branch registry truth or current-branch
truth directly, then every app will have to invent its own mirror layer for:

- current branch
- known branches
- post-command refresh behavior
- branch-related UI invalidation

That is strong evidence of a missing productized history-consumption lane.

### Concrete Audit Questions

The audit should answer:

1. Does the history surface already have a hidden subscription mechanism that
   is simply not documented?
2. Is the intended integration path that apps manually re-read history after
   every history mutation?
3. Should current branch and branch registry truth be publishable as signal
   handles or another subscribable surface?
4. Can a reactive history lane be introduced without collapsing the richer
   proof and replay APIs into a shallow state store?

### Desired Outcome

The library should likely provide a first-class reactive consumption lane for
history state in UI code, such as:

- current branch truth
- branch registry truth
- branch transitions
- replay/restore lifecycle visibility where appropriate

The exact shape is less important than the contract:

- no manual React mirror state for routine history reads
- no "command, then re-read everything" boilerplate
- clear subscription semantics for history-bearing UI

### Acceptance Criteria For Closing The Gap

This gap is not closed until a React consumer can render branch-oriented UI
from a first-class reactive history surface rather than hand-maintaining a
mirror of imperative reads.

## Gap 18: Router Web-Session Orchestration Is Still Too Manual

### Problem Statement

The router runtime can clearly represent:

- typed route references
- browser-history ingress envelopes
- admitted boundary reports
- retained browser-history stories
- breadcrumb policy and provenance

But the web-facing app lane for turning those capabilities into an ordinary
navigation session still appears too manual.

In the beefed-up router demo, a small but real navigation console required
explicit app-side orchestration to:

- build an ingress envelope with `signals.router.browserHistory.push(...)`
- admit it with `routes.admitBrowserHistoryIngress(...)`
- record it with `story.record(...)`
- recompute candidate, breadcrumb, and history panels
- mirror the resulting truth into React state for rendering

That is powerful and honest, but it suggests the library does not yet provide a
first-class "router session/controller" lane for ordinary web apps.

### Why This Looks Like A Real Library Gap

The issue is not that the router lacks capability. The issue is that using the
capability in a real app still requires stitching together multiple imperative
steps that feel like framework session plumbing rather than feature code.

Without a stronger lane, every serious web app will likely reinvent some local
wrapper that owns:

- current visible href
- browser ingress creation
- route admission
- history story recording
- breadcrumb refresh
- current/previous route snapshots for UI

That is a recurring architectural seam, not just demo ceremony.

### Concrete Evidence

The current docs and types clearly expose the low-level pieces:

- `signals.router.browserHistory.push(...)`
- `routes.admitBrowserHistoryIngress(...)`
- `signals.router.browserHistory.story(...)`
- `story.record(...)`

But they do not appear to expose a higher-level session surface such as:

- a retained current route session
- a first-class navigation controller
- a subscribable current-route/current-story model for React

### Desired Outcome

The library likely needs a higher-level web navigation lane that preserves the
explicit boundary truth model while removing routine session boilerplate.

Possible shapes worth evaluating:

- a router session/controller object created from a resolved tree
- a React-facing adapter that owns ingress + admission + story recording
- a current-route/current-story subscribable surface that updates after
  navigation without app-authored mirror state

The important part is not the exact API name. The important part is that a
normal app should not have to hand-author the full browser-history ingress to
story pipeline every time it wants ordinary routed UI.

### Acceptance Criteria For Closing The Gap

This gap is not closed until a web app can express ordinary typed navigation
with:

- one retained session authority
- explicit but low-ceremony browser-boundary integration
- first-class current route and history consumption
- no repeated app-authored ingress/admit/record/mirror pipelines

## Gap 19: Route Sequence Simulation And Playback Are Still App-Authored

### Problem Statement

The beefed-up replay demo in:

- `apps/worth-signal-demo/src/ui/demos/DemoSeven.tsx`

showed that the router can absolutely support navigation-sequence simulation,
but there is still no first-class lane for expressing it.

To build a drag-ordered "click these links, simulate them, then replay the
outcomes" demo, the app had to author:

- its own sequence model
- its own loop over hrefs
- its own ingress creation (`load` for the first step, `push` for the rest)
- its own `routes.admitBrowserHistoryIngress(...)` calls
- its own `story.record(...)` lifecycle
- its own derived replay views for:
  - boundary outcomes
  - breadcrumb evolution
  - history growth

That is powerful, but it suggests the library still stops short of a
first-class scenario / playback lane.

### Why This Looks Like A Real Library Gap

The issue is not that replay is missing entirely. The router already exposes:

- retained history story entries
- `entry.replay(history)`
- breadcrumb trail replay helpers
- back provenance replay helpers

But those are per-entry evidence surfaces. They are not yet a higher-level
sequence simulation or playback surface.

For a common developer workflow like:

- queue navigation steps
- simulate the route outcomes
- inspect what happened after each step
- replay one or more views of the sequence

the current app must still invent the orchestration model itself.

### Concrete Evidence

Demo 7 could only be built by combining several low-level lanes manually:

- `routeRef.to(...).href`
- `signals.router.browserHistory.load(...)`
- `signals.router.browserHistory.push(...)`
- `routes.admitBrowserHistoryIngress(...)`
- `signals.router.browserHistory.story()`
- `story.record(...)`
- app-authored accumulation of step-by-step replay rows

That is enough to prove the substrate is real, but it is also enough to show
that a productized simulation/playback lane is still missing.

### Desired Outcome

The library likely needs a higher-level navigation scenario surface, such as:

- a route-sequence simulator
- a story playback helper over ordered hrefs or route references
- a retained artifact that exposes step-by-step replay summaries directly

The exact name is not the point. The point is that the app should not have to
hand-author the full sequence engine every time it wants to rehearse or
visualize navigation outcomes.

### Acceptance Criteria For Closing The Gap

This gap is not closed until a web app can express route-sequence rehearsal
with:

- a first-class ordered navigation input
- retained step outcomes
- replay views over the resulting sequence
- no app-authored ingress/admit/record loop for the common case

## Gap 20: Feature Store State Typing Is Too Narrow For Ordinary Object-Shaped App State

### Problem Statement

`signals.featureStore(...)` works cleanly for simple scalar state, but the
public type lane becomes too narrow once a store needs ordinary object-shaped
UI values.

Examples from the admin app included state values such as:

- `DataTableUserLayoutConfig`
- `WorkplaceAuditLogsAdminQueryValues`

Those values are routine app state, not exotic runtime payloads. But the
feature-store type surface currently pushes authors toward casts like:

- `INITIAL_LAYOUT as SignalValue`
- `INITIAL_QUERY as unknown as SignalValue`
- `set("queryValues", next as unknown as SignalValue)`

That pressure showed up in:

- `workplace-api-keys-admin.store.ts`
- `workplace-audit-logs-admin.store.ts`
- `workplace-projects-admin.store.ts`
- `workplace-user-groups-admin.store.ts`

### Why This Looks Like A Real Library Gap

The runtime clearly supports object values. The issue appears to be the public
state contract:

- `type FeatureStoreStateDefinition = Record<string, SignalValue>`

If an app's domain state is structurally richer than that alias admits
comfortably, authors fall into cast-land even though the runtime behavior is
fine.

That creates bad incentives:

- shared store code gets noisy
- authors stop trusting inference
- wrappers accumulate `unknown as SignalValue`
- proof-carrying ergonomics break down at the boundary

### Why This Matters

This is not just an aesthetic type complaint.

Feature stores are supposed to be a first-class app-facing lane for ordinary
local state. If object-shaped state values require casts at declaration and
write sites, the library is effectively saying:

- scalar state is first-class
- ordinary structured app state is tolerated through type erasure

That is too weak for a store surface that is meant to replace app-invented
store scaffolding.

### Desired Outcome

The ideal surface should let feature stores accept a broader domain-state
generic and only lower to signal-compatible runtime values internally.

The important shape is:

- app authors declare their real domain state type directly
- `set(...)` is typed in terms of that domain state
- runtime-safety narrowing happens inside the implementation, not at app call
  sites

### Acceptance Criteria For Closing The Gap

This gap is not closed until a feature store with ordinary object-shaped state
can be authored without:

- `as SignalValue`
- `unknown as SignalValue`
- duplicated state wrapper types whose only job is to appease the current
  boundary

## Gap 21: Local Dialog State Is Too Small For Real Modal CRUD Workflows

### Problem Statement

`signals.local.dialogState(...)` is fine for plain boolean open/close state,
but it is too small to act as a first-class lane for real admin dialogs.

The current helper basically offers:

- `open()`
- `close()`
- `toggle()`
- `signal` as a boolean

That is enough for a toy modal. It is not enough for the kinds of dialog
authority real product flows actually need:

- mode / key
- payload data
- optional context
- loading state
- close readiness / blocker posture
- reset / discard semantics
- action lifecycle
- inspectable dialog history

### Why This Looks Like A Real Library Gap

This mismatch showed up directly during the admin integration. We could not
hard-cut the app's `useDialogState.ts` helper, because it still owns real
behavior the library does not replace yet.

The app-side shape we still need looks more like:

- typed dialog mode
- typed payload
- typed context
- lifecycle state like loading
- open helpers that carry mode plus payload together

That is not an edge case. It is the normal shape of modal CRUD workflows.

### Why This Matters

This gap is especially embarrassing relative to the rest of the library,
because the form/runtime surface is aiming at much richer truth:

- readiness
- patch plans
- actions
- canonical source vs draft posture
- visible validation and message policy

Against that backdrop, a dialog helper that stops at `boolean open/close` feels
far below the surrounding product ambition.

If the form lane can think in terms of source truth, draft truth, action
lifecycle, and validation posture, the dialog lane should not force app authors
back into custom wrappers just to represent:

- create vs edit vs delete
- current payload target
- contextual modal metadata
- transient dialog loading state

### Desired Outcome

The library likely needs either:

- a richer dialog controller surface
- or a dialog state helper that can declare typed keys plus typed payload and
  contextual state

But the actual target should be set by the form system that already exists.
Relative to `useSignalsForm(...)`, the dialog lane should not just learn
"payloads." It should cover the same categories of workflow authority at the
modal level.

At minimum, a first-class dialog lane should cover:

- source / draft / effective dialog state
- typed mode, payload, and context
- dirty and reset semantics
- close readiness / blocker posture
- dialog actions with pending / result lifecycle
- visible messages or interaction posture for close / discard workflows
- first-class composition with `useSignalsForm(...)`
- dialog history / diagnostics surfaces

The key requirement is that modal CRUD workflows become first-class enough that
app foundations do not need to re-invent the dialog authority model beside the
form authority model.

### Acceptance Criteria For Closing The Gap

This gap is not closed until a typical CRUD modal workflow can express:

- open/close
- create/edit/delete mode
- typed payload data
- optional contextual metadata
- loading or pending posture
- close/discard blockers
- reset / restore semantics
- action lifecycle and result posture
- honest composition with the library's form surface

without falling back to an app-authored dialog controller wrapper for the
common case.

## Integration Warnings

These are not necessarily correctness bugs, but they are important warnings
from the integration work.

### Warning 1: The Proof Surfaces Are Powerful But Too Buried For Routine App Work

The library appears to have strong proof surfaces:

- diagnostics
- mutation responses
- history
- verification-style truth surfaces

But ordinary feature code does not naturally consume those directly. In
practice, we had to build internal bridges before they became usable in routine
application workflows.

### Warning 2: The Library Currently Assumes Wrapper Authorship More Than Direct Feature Usage

The practical usage pattern that emerged was:

- foundation helpers
- app resource layer
- feature hooks

That is a reasonable architecture, but it suggests the library is still one
layer too low for many product teams if they try to use it directly without an
internal standard library.

### Warning 3: Lifecycle Ownership Is Easy To Get Subtly Wrong

Real apps must make repeated lifecycle decisions around:

- transient line creation
- transient line cleanup
- family caching
- resident line retention
- fallback revalidation
- bridging runtime truth into UI state

All of that is possible today, but it is easy for applications to get subtly
wrong unless stronger helper lanes exist.

### Warning 4: The Library Is Very Strong On Truth, But Thinner On Policy

The broad pattern exposed by the integration is:

- the library is strong at telling us what the truth is
- the library is thinner at telling us what standard application policy should
  follow from that truth

This shows up in:

- content-state policy
- fallback recovery policy
- toast policy
- transient write lifecycle policy
- form binding policy

That is a valid philosophy, but it means serious apps will likely need their
own internal standard library before the overall system feels ergonomic.

### Warning 5: Hidden Fallback-Signal Workarounds Remain A Smell

Even aside from the broader optional-resource-line gap, the integration had to
fabricate hidden local fallback signals just to keep React subscriptions stable
when a resource line was absent.

That is a warning sign because:

- fallback values are not first-class runtime concepts
- app-authored fallback signal truth can drift from runtime-understood
  inactive/missing/unselected posture
- hook stability currently encourages workaround folklore

This reinforces that the library likely needs a more principled inactive
observable surface.

### Warning 6: Construction-Time Configuration Rules Are Sharp And Runtime-Enforced

Some of our convenience hooks treat identity, aspects, or initial values as
construction-time only and throw if they change later.

This is partly wrapper policy, but it exposes a broader ergonomics issue:

- scope-bound signals and local runtimes are often mount-oriented
- reconfiguration tends to require remount rather than update
- React developers can trip over this unless the library has clearer
  construction-policy or keyed-remount conventions

This is a warning that construction-vs-reconfiguration patterns need clearer
product guidance.

What appears to be missing is a first-class policy surface for questions like:

- mount-only configuration
- keyed remount
- replace-runtime reconfiguration
- safe versus unsupported live reconfiguration

### Warning 7: Repeated-Item Identity In Forms Is Too Underspecified

In our form bridge, repeated fields currently had to fall back to weak identity
choices such as `String(item)` when no serious domain identity strategy was
available.

That is not proof of a library bug, but it is a warning sign that repeated-item
identity may still be under-specified in the product story.

Areas to watch:

- explicit identity requirements for repeated form items
- defaults versus mandatory author intent
- diagnostics when item identity is weak or unstable
- whether object-valued repeated items should require explicit identity

### Warning 8: Higher-Order React Integration Still Leans On Type Erasure In Wrappers

Even where the runtime appears conceptually sound, our adapter layers
frequently had to cross type-erasure boundaries to get practical React
ergonomics.

This is not identical to the earlier broad `any` concern. The warning here is
more specific:

- the type system is not yet carrying intent cleanly enough through
  higher-order runtime authoring
- that makes it harder to build clean, generic wrapper layers without escape
  hatches

This is a warning that the type surface may still need another maturity pass in
the helper-layer and wrapper-authoring zone.

## Current Recommendation

All seventeen gaps appear real enough to deserve explicit audit work rather than
being treated as isolated application mistakes.

Priority recommendation:

1. audit mutation finalizer / semantic intent / reconciliation coupling as a
   product-surface and lowering-surface parity problem
2. audit optional resource line consumption as a first-class React ergonomics
   gap
3. audit re-entrant runtime reads and React snapshot stability as a
   runtime/adapter contract gap
4. audit transient write execution, resource view-state projection, and
   fallback recovery policy as app-facing ergonomics / policy-layer gaps
5. audit resource catalog conventions and React form ergonomics as
   architecture-layer maturity gaps
6. audit settlement awaiting, write-lifecycle feedback, API-scope modeling,
   local scope authoring, and managed local feature-store helpers as missing
   higher-level authoring lanes
7. audit history consumption as a reactive-surface gap so branch-oriented UI
   does not require imperative mirror-state wrappers
8. audit runtime compatibility/contract introspection so downstream wrappers do
   not need to probe the callable/scoped surface by hand
9. audit the broader web/wasm async-consumption model so core async
   first-classness is delivered coherently across resource lines, forms,
   history, and React integration

These gaps hit different layers of the library:

- Gap 1 is deeper architectural and affects mutation-response honesty
- Gap 2 is a frequent developer-experience gap for ordinary React usage
- Gap 3 is a correctness and adapter-contract gap that can surface as runtime
  panic plus React render instability
- Gaps 4 through 7 point at missing productized helper lanes for serious
  application architecture
- Gap 8 points at a strong form core whose React ergonomics layer likely needs
  another level of maturity
- Gaps 9 through 14 continue the same pattern: the runtime appears strong, but
  many routine app-facing workflows still require custom wrapper layers that
  likely deserve stronger first-class support
- Gap 15 extends that pattern into history: the proof and branch capabilities
  look strong, but routine React consumption still appears to require manual
  mirror-state wiring
- Gap 16 extends the pattern into integration trust: the runtime may be
  conceptually strong, but downstream wrappers still lack a first-class way to
  verify which callable/scoped contract they are actually standing on
- Gap 17 is the broader product-shape concern underneath several smaller gaps:
  core async looks first-class in `worth-signal`, but the web-facing product
  surface still delivers it unevenly enough that apps reconstruct too much of
  the async story themselves

## Addendum: Concrete DX Targets For Existing Gaps

### Gap 1: Mutation Finalizer / Semantics / Reconciliation Drift

**DX Target**

A resource author must be able to express transport shape, semantic mutation
intent, and reconciliation legality without forcing them into a dishonest CRUD
bucket.

**Desired authoring shape**

```ts
api.url('/groups/:groupId/users/:userId').command({
  method: 'POST',
  semantics: 'relationshipUpdate',
  response: statusOnly(),
  reconciles: [
    fallbackOnly(groupsCollection, { reason: 'statusOnlyResponse' }),
  ],
});
```

Or, if the response really proves an update-like result:

```ts
api.url('/groups/:groupId/users/:userId').mutation({
  method: 'POST',
  semantics: 'update',
  reconciles: [
    collectionItemReplace(groupsCollection, {
      by: 'groupId',
    }),
  ],
});
```

**Boilerplate that should disappear**

- lying with `.create(...)` just because the transport is `POST`
- app authors guessing which finalizer is "least wrong"
- runtime-lowering surprise after a type-valid declaration

**Failure mode improvement**

- denied at authoring time with a precise diagnostic about semantic/finalizer
  mismatch
- or supported through an explicit command/update lane

**Wrapper reduction goal**

- this should be a true peer surface to forms rather than a tiny visibility
  helper that forces workflow authority back into app-local wrappers

- wrappers should not need to reinterpret transport method as mutation
  semantics

**Acceptance signal**

- the same flow can also express close blockers, reset semantics, and dialog
  action lifecycle without app-authored controller glue

- a `POST` command-style route can be authored honestly without pretending to be
  create/update/delete

### Gap 2: Optional Resource Line Consumption For React

**DX Target**

A React consumer must be able to represent "nothing is selected right now"
without violating hook order, inventing sentinel params, or fabricating
fallback signals.

**Desired authoring shape**

```ts
const projectPermissions = useResourceLine(
  catalog.projects.projectPermissions,
  selectedProjectId
    ? { projectId: selectedProjectId }
    : { enabled: false },
);
```

Or:

```ts
const projectPermissions = catalog.projects.projectPermissions.optionalLine(
  selectedProjectId ? { projectId: selectedProjectId } : null,
);

const permissions = useOptionalResourceLineValue(projectPermissions);
```

**Boilerplate that should disappear**

- conditional hook calls
- fake params
- hidden fallback signal creation
- component splitting just to preserve hook order

**Failure mode improvement**

- inactive state should be explicit and inspectable, not collapse into
  loading/error/null-pointer behavior

**Wrapper reduction goal**

- eliminate custom inactive-line workarounds in app foundations

**Acceptance signal**

- a detail panel can remain mounted while its selected resource is
  intentionally inactive, with no fake request and no hook-order workaround

### Gap 3: Re-entrant Runtime Reads And Unstable React Snapshot Consumption

**DX Target**

A normal React consumer must be able to subscribe to signals without risking
runtime borrow panics or snapshot-instability loops.

**Desired authoring shape**

```ts
const count = useSignalValue(counterSignal, store);
const summary = useSignalValue(summarySignal, store);
```

And for diagnostics-bearing demo/debug code:

```ts
const latestFlow = useSignalsDiagnosticsValue((d) => d.latestFlow());
```

**Boilerplate that should disappear**

- app authors having to guess which reads are "safe"
- wrapper-level snapshot caching folklore
- defensive avoidance of ordinary diagnostics reads in React

**Failure mode improvement**

- unsupported read/re-entry patterns should fail as structured diagnostics, not
  Rust panics or infinite render loops

**Wrapper reduction goal**

- wrappers should not need to defend against ordinary read instability

**Acceptance signal**

- tiny React demos with live subscriptions, effects, watches, and diagnostics do
  not panic or trip `getSnapshot` instability warnings

### Gap 4: Missing First-Class Managed Transient Write Execution

**DX Target**

A feature author must be able to execute a transient write through a first-class
lifecycle surface instead of hand-authoring pending, settlement, partial
handling, and disposal every time.

**Desired authoring shape**

```ts
const createProject = useManagedResourceWrite({
  line: (body) => catalog.projects.createProject.line({ body }),
  successTitle: 'Project created',
  partialTitle: 'Project created, refreshing list',
  errorTitle: 'Unable to create project',
});
```

Or lower-level but still first-class:

```ts
const execution = catalog.projects.createProject.execute({ body });
const settlement = await execution.settled();
```

**Boilerplate that should disappear**

- `useState(false)` pending flags per mutation
- manual `try/finally` disposal
- repeated settlement interpretation
- repeated success/partial/error branching

**Failure mode improvement**

- execution misuse should fail through a structured execution contract, not ad
  hoc app glue

**Wrapper reduction goal**

- app wrappers should shrink to policy composition, not lifecycle reinvention

**Acceptance signal**

- multiple feature mutations can use the same first-class execution lane without
  custom orchestration helpers

### Gap 5: Missing Standard Resource-To-UI View-State Bridge

**DX Target**

A feature author must be able to consume resource truth as a conventional UI
resource shape without rebuilding loading/refreshing/error mapping in every
app.

**Desired authoring shape**

```ts
const projects = useResourceView(projectsLine, {
  errorMessage: 'Unable to load projects.',
  emptyWhen: (rows) => rows.length === 0,
});
```

**Boilerplate that should disappear**

- custom `toLineContentState(...)`
- custom `toLineListResource(...)`
- repeated "loading vs refreshing vs ready" translation

**Failure mode improvement**

- error/empty/refreshing interpretation should be consistent by default, not
  app-specific folklore

**Wrapper reduction goal**

- eliminate repeated app-local view-state mappers

**Acceptance signal**

- a feature author can render standard loading/empty/error UI from a resource
  line without building a custom adapter first

### Gap 6: Fallback Settlement Exists, But Recovery Policy Is Too Manual

**DX Target**

A feature author must be able to pair fallback settlement with an honest,
standard recovery policy instead of reinventing revalidation choreography.

**Desired authoring shape**

```ts
const execution = deleteProject.execute({ projectId });

const settlement = await execution.settled();

await settlement.recovery.apply({
  partial: [
    projectsLine.revalidate(),
    selectedProjectLine.clearIfMatches(projectId),
  ],
});
```

Or authored at the family level:

```ts
deleteProject: api.url('/projects/:projectId').remove({
  reconciles: [...],
  recoveryPolicy: ({ resident, params }) => ({
    partial: [
      resident.projects.revalidate(),
      resident.selectedProject.clearIfMatches(params.projectId),
    ],
  }),
})
```

**Boilerplate that should disappear**

- ad hoc `if (partial) revalidate X and Y`
- per-feature selection clearing rituals
- custom fallback-close-vs-stay-open policy

**Failure mode improvement**

- fallback should surface structured recovery guidance, not just raw fallback
  truth

**Wrapper reduction goal**

- wrappers should compose declared recovery policy rather than invent it

**Acceptance signal**

- fallback-heavy writes can be integrated without bespoke per-feature follow-up
  choreography

### Gap 7: Missing First-Class Resource Catalog / Registry Convention

**DX Target**

An app architect must be able to declare a runtime-scoped resource catalog
through a first-class lane rather than inventing a `WeakMap` registry pattern.

**Desired authoring shape**

```ts
export const workplaceAdminCatalog = createResourceCatalog({
  id: 'workplace-admin',
  scope: signals.apiScope('workplace-admin', { baseUrl: '/api' }),
  domains: {
    users: buildUserFamilies,
    groups: buildGroupFamilies,
    apiKeys: buildApiKeyFamilies,
    projects: buildProjectFamilies,
    auditLogs: buildAuditLogFamilies,
  },
});
```

Then:

```ts
const catalog = useResourceCatalog(workplaceAdminCatalog);
```

**Boilerplate that should disappear**

- `WeakMap<object, Catalog>`
- string-keyed domain registries
- app-authored cache ownership rules

**Failure mode improvement**

- catalog identity/reuse mistakes should be diagnosable through the catalog
  surface, not opaque app caching bugs

**Wrapper reduction goal**

- wrappers should declare domain grouping, not invent caching architecture

**Acceptance signal**

- a multi-domain app can stand up a stable catalog without custom registry
  infrastructure

### Gap 8: Form Runtime Is Stronger Than Its React Ergonomics Layer

**DX Target**

A feature author must be able to use the form runtime in ordinary CRUD dialogs
through a first-class React adapter, not through a custom app-specific form
facade.

**Desired authoring shape**

```ts
const form = useSignalsForm({
  source: dialogSource(initialValue),
  validate,
  actions: {
    submit: updateUserLine,
  },
});

<TextField {...form.field('email')} />
<SelectField {...form.select('role', roleOptions)} />
<MultiSelectField {...form.multiSelect('appIds', appOptions)} />
<Button disabled={form.actions.submit.disabled}>
  Save
</Button>
```

**Boilerplate that should disappear**

- custom field binding helpers
- custom checkbox/select/multiselect bridges
- custom readiness-to-button mapping
- custom blur/touched error visibility approximations

**Failure mode improvement**

- invalid form wiring should fail through a form adapter contract, not by
  forcing every app to rediscover interaction policy

**Wrapper reduction goal**

- app wrappers should become optional styling/composition layers, not required
  ergonomics layers

**Acceptance signal**

- a typical modal CRUD form can be authored directly against the libraryÃ¢â‚¬â„¢s
  React form surface without a large app-local facade

### Gap 9: No First-Class Subscription-Based "Await Settlement" Lane

**DX Target**

A write author must be able to await resource-operation settlement natively,
without polling loops.

**Desired authoring shape**

```ts
const execution = createProject.line({ body }).execute();
const settlement = await execution.settled({ timeoutMs: 15_000 });
```

**Boilerplate that should disappear**

- polling `line.status()`
- `setTimeout` loops
- ad hoc timeout helpers

**Failure mode improvement**

- timeout/cancellation should be structured outcomes, not app-authored polling
  failures

**Wrapper reduction goal**

- remove custom await-settlement helpers entirely

**Acceptance signal**

- no app helper like `waitForResourceLineSettlement(...)` is needed for
  ordinary write awaiting

### Gap 10: No Standard "Toast From Write Lifecycle" Bridge

**DX Target**

A product app should be able to bridge write lifecycle truth into standard user
feedback without rebuilding the same toast policy from raw settlement facts.

**Desired authoring shape**

```ts
const createProject = useManagedResourceWrite({
  line: (body) => catalog.projects.createProject.line({ body }),
  feedback: {
    success: 'Project created',
    partial: 'Project created, refreshing list',
    error: 'Unable to create project',
  },
});
```

Or:

```ts
const feedback = execution.feedback();
toastBridge.consume(feedback);
```

**Boilerplate that should disappear**

- custom settlement-to-toast mapping
- custom dedupe logic
- per-feature success/partial/error branching

**Failure mode improvement**

- lifecycle feedback should be standardized and inspectable, not app-local
  interpretation drift

**Wrapper reduction goal**

- wrappers should provide product copy, not invent lifecycle classification

**Acceptance signal**

- multiple write flows can share one standard feedback bridge with only message
  customization

### Gap 11: Fallback Severity And Policy Remain Too Raw For App UX Decisions

**DX Target**

A feature author must be able to distinguish meaningful fallback severities
without flattening everything into one custom `partial` bucket.

**Desired authoring shape**

```ts
const summary = settlement.recovery.summary();

if (summary.severity === 'info') {
  // safe delayed refresh
}

if (summary.severity === 'warning') {
  // meaningful stale risk
}
```

**Boilerplate that should disappear**

- custom severity heuristics
- flattening all fallback into one UI treatment
- ad hoc "warning vs okay" policy

**Failure mode improvement**

- fallback should expose policy-facing severity/provenance directly

**Wrapper reduction goal**

- wrappers should style severity, not invent it

**Acceptance signal**

- different fallback kinds can drive different UX posture without app-authored
  heuristics

### Gap 12: API-Scope Creation Is Still Too Weakly Modeled

**DX Target**

An app author must be able to create stable API scopes with first-class
identity, reuse, and typing semantics.

**Desired authoring shape**

```ts
const adminApi = signals.apiScope('workplace-admin', {
  baseUrl: '/api',
  headers: {
    'x-app-surface': 'admin',
  },
});
```

Or:

```ts
const rootApi = signals.apiScope('root', { baseUrl: '/api' });
const adminApi = rootApi.scope('workplace-admin', {
  headers: {
    'x-app-surface': 'admin',
  },
});
```

**Boilerplate that should disappear**

- `WeakMap<object, Map<string, unknown>>`
- string cache-key folklore
- weakly typed scope factory wrappers

**Failure mode improvement**

- misuse should fail as explicit scope identity/configuration issues, not
  wrapper-cache weirdness

**Wrapper reduction goal**

- remove custom scope-cache helpers

**Acceptance signal**

- stable API scope reuse works without app-authored caching infrastructure

### Gap 13: Local Scope Authoring Is Still Too Manual

**DX Target**

A wrapper or feature author must be able to stand up common local runtime shapes
without raw `scope.spec.*` plumbing every time.

**Desired authoring shape**

```ts
const dialogState = signals.local.dialogState({
  identity: 'invite-user-dialog',
});
```

Or:

```ts
const listState = signals.local.listState({
  identity: 'candidate-users',
  initial: [],
  aspects: selectionAspects,
});
```

**Boilerplate that should disappear**

- manual `signals.scope(identity)`
- manual `scope.spec.input(...)`
- manual disposal wiring
- manual debug-name and aspect repetition

**Failure mode improvement**

- common local-state patterns should fail through named local-surface
  contracts, not raw scope misuse

**Wrapper reduction goal**

- wrappers should compose local primitives, not recreate them

**Acceptance signal**

- common local patterns like dialog/list/form-source state can be authored
  without raw scope/spec ceremony

### Gap 14: Managed Local Feature Store Remains An App-Invented Layer

**DX Target**

A feature architect must be able to author a scoped feature store with actions
through a first-class lane, not a custom store framework built on raw signals.

**Desired authoring shape**

```ts
const userGroupsStore = signals.featureStore({
  id: 'workplace-user-groups-admin',
  state: {
    selectedGroupId: null,
    selectedCandidateId: '',
    view: 'users',
  },
  actions: ({ set }) => ({
    setSelectedGroupId: (next) => set('selectedGroupId', next),
    setSelectedCandidateId: (next) => set('selectedCandidateId', next),
  }),
});
```

**Boilerplate that should disappear**

- custom `createScopedSignalStore(...)`
- repeated action/store scaffolding across apps
- custom debug/view/action conventions

**Failure mode improvement**

- feature-store misuse should fail through a store contract, not
  wrapper-specific behavior

**Wrapper reduction goal**

- app foundations should not need to invent their own managed feature-store
  abstraction

**Acceptance signal**

- multiple unrelated features can share one first-class scoped store pattern
  without app-authored store infrastructure

### Gap 15: History Surfaces Are Rich But Not First-Class Reactive Consumables In React

**DX Target**

A React consumer must be able to render branch/story/history UI from a
first-class reactive surface instead of mirroring imperative reads into local
state.

**Desired authoring shape**

```ts
const historyView = useSignalsHistory(historyHandle);

historyView.currentBranch;
historyView.branches;
historyView.canUndo;
historyView.canRedo;
```

Or for router stories:

```ts
const storyView = useBrowserHistoryStory(story);

storyView.current;
storyView.entries;
storyView.breadcrumbTrail;
```

**Boilerplate that should disappear**

- manual "command then re-read"
- manual `useState` mirrors of history truth
- custom refresh-after-history-mutation glue

**Failure mode improvement**

- history/session UI should degrade as reactive state, not stale imperative
  mirrors

**Wrapper reduction goal**

- wrappers should style or project history, not invent reactivity for it

**Acceptance signal**

- branch/story UI can be rendered directly from subscribed history truth
  without local mirror state

### Gap 16: Runtime Compatibility And Contract Introspection Are Not First-Class

**DX Target**

A downstream foundation author must be able to ask the runtime what
callable/scoped contract it supports without hand-authored executable probes.

**Desired authoring shape**

```ts
const contract = signals.contract();

contract.surfaceVersion;
contract.capabilities.scopeAuthoring;
contract.capabilities.specNamespace;
contract.capabilities.workerRuntime;
```

Or:

```ts
signals.assertCompatibility({
  requires: ['callableSurface', 'scopedAuthoring', 'specNamespace'],
});
```

**Boilerplate that should disappear**

- custom `assertCallableSignalsRuntime(...)`
- custom `assertScopedAuthoringContract(...)`
- hand-maintained method lists

**Failure mode improvement**

- compatibility mismatch should be machine-readable and explicit, not
  discovered via bespoke contract checks

**Wrapper reduction goal**

- remove ad hoc runtime contract probes from app foundations

**Acceptance signal**

- a downstream wrapper can verify compatibility without executing its own probe
  graph

### Gap 17: Core Async Is First-Class, But The Web/Wasm Async Consumption Surface Is Still Fragmented

**DX Target**

A web app should be able to consume one coherent async truth model across:

- resource operations
- resource lines
- forms
- history
- React subscriptions

**Desired authoring shape**

```ts
const execution = createProject.line({ body }).execute();
const settlement = await execution.settled();

const createStatus = useResourceOperation(execution, store);
const projects = useResourceLine(projectsLine, store);

const form = useSignalsForm({
  source: signals.form.source.resourceLine(projectDetailLine),
  actions: {
    submit: execution,
  },
});
```

**Boilerplate that should disappear**

- wrapper-authored polling
- wrapper-authored mirror async state
- wrapper-authored async policy projection per subsystem
- different async consumption idioms for adjacent surfaces

**Failure mode improvement**

- async lifecycle mismatch should be structured and coherent, not fragmented
  across subsystems

**Wrapper reduction goal**

- wrappers should not need to reassemble one async story from several partially
  productized ones

**Acceptance signal**

- an app can use the same mental model for async truth across writes, reads,
  forms, history, and React consumption

### Gap 18: Router Web-Session Orchestration Is Still Too Manual

**DX Target**

A web app author must be able to stand up a typed router session without
hand-authoring the full ingress/admit/record/mirror pipeline.

**Desired authoring shape**

```ts
const session = useRouterSession(routes, {
  history: 'browser',
});

session.currentRoute;
session.navigate(routes.admin.users.to());
session.story;
session.breadcrumbs;
```

**Boilerplate that should disappear**

- manual ingress envelope creation
- manual `routes.admitBrowserHistoryIngress(...)`
- manual `story.record(...)`
- manual current-route React mirror state

**Failure mode improvement**

- session-level wiring errors should fail through a router-session contract,
  not app-authored orchestration bugs

**Wrapper reduction goal**

- wrappers should compose route policy, not invent browser session plumbing

**Acceptance signal**

- a normal routed app can use one retained session authority instead of custom
  ingress/admit/record choreography

### Gap 19: Route Sequence Simulation And Playback Are Still App-Authored

**DX Target**

A developer, tester, or tooling surface must be able to simulate ordered route
sequences without hand-building a playback engine.

**Desired authoring shape**

```ts
const scenario = routes.simulateSequence([
  routes.admin.users.to(),
  routes.admin.projects.to(),
  routes.admin.auditLogs.to(),
]);

const result = await scenario.run();

result.steps;
result.story;
result.replay.breadcrumbTrail();
result.replay.outcomes();
```

**Boilerplate that should disappear**

- app-authored sequence models
- custom ingress loops
- manual `admitBrowserHistoryIngress(...)`
- manual `story.record(...)`
- custom step accumulation and replay views

**Failure mode improvement**

- invalid or denied route sequences should produce structured scenario
  diagnostics, not custom app replay logic

**Wrapper reduction goal**

- wrappers should visualize scenarios, not invent sequence execution engines

**Acceptance signal**

- route rehearsal/playback can be done from a first-class sequence surface
  without app-authored orchestration loops

### Gap 20: Feature Store State Typing Is Too Narrow For Ordinary Object-Shaped App State

**DX Target**

A feature architect must be able to declare ordinary object-shaped store state
without casts, while keeping `set(...)` and `read(...)` typed in terms of the
real domain state.

**Desired authoring shape**

```ts
const auditStore = signals.featureStore({
  id: "workplace-audit-logs-admin",
  state: {
    search: "",
    page: 1,
    layoutConfig: {} as DataTableUserLayoutConfig,
    queryValues: INITIAL_QUERY_VALUES,
    quickReportId: null as string | null,
  },
  actions: ({ set, read }) => ({
    setQueryValues: (next) => set("queryValues", next),
    resetFilters: () => {
      set("queryValues", INITIAL_QUERY_VALUES);
    },
  }),
});
```

**Boilerplate that should disappear**

- `as SignalValue`
- `unknown as SignalValue`
- wrapper-only state aliases whose sole purpose is satisfying the current store
  boundary

**Failure mode improvement**

- ordinary structured app state should fail only when it is truly unsupported
  by runtime value semantics, not because the public state alias is too narrow

**Wrapper reduction goal**

- stores should carry real domain state directly instead of accumulating
  cast-heavy type-erasure at declaration and write sites

**Acceptance signal**

- feature stores with layout config, query values, and other object-shaped app
  state compile cleanly without cast folklore

### Gap 21: Local Dialog State Is Too Small For Real Modal CRUD Workflows

**DX Target**

A feature author must be able to model real modal CRUD authority through a
first-class dialog lane that is meaningfully peer-level with the form surface,
not just boolean open/close state.

**Desired authoring shape**

```ts
const dialog = signals.local.dialogController({
  identity: "project-dialog",
  initial: {
    isOpen: false,
    key: null as "create" | "edit" | "delete" | null,
    data: null as Project | DeleteTarget | null,
    context: null as DialogContext | null,
    loading: false,
  },
});

dialog.open("edit", { data: project });
dialog.setLoading(true);
dialog.close();
```

And at the richer form-symmetry level:

```ts
dialog.source;
dialog.draft;
dialog.effective;
dialog.dirty;
dialog.readiness;
dialog.visibleMessages;
dialog.action("close");
dialog.action("confirm");
dialog.reset();
dialog.stateHistory();
```

Or at minimum:

```ts
const dialog = signals.local.dialogState({
  identity: "project-dialog",
  keys: ["create", "edit", "delete"] as const,
});

dialog.open("edit", { data: project, context });
dialog.loading.set(true);
```

**Boilerplate that should disappear**

- app-authored dialog controller wrappers for mode/payload/context
- parallel local state just to represent loading inside a modal workflow
- custom glue to bind dialog identity and typed payload together
- custom close-blocker and discard-confirmation glue beside forms
- custom result/pending wiring for modal actions

**Failure mode improvement**

- modal workflow wiring should fail through a dialog contract with typed mode
  and payload semantics, not through ad hoc app controllers
- modal close/discard policy should fail through explicit readiness/blocker
  posture, not app-local folklore

**Wrapper reduction goal**

- app foundations should not need to re-invent dialog authority beside the
  libraryÃ¢â‚¬â„¢s much richer form authority surface

**Acceptance signal**

- a typical create/edit/delete modal flow can be expressed with typed mode,
  payload, context, and loading posture without a custom dialog controller
