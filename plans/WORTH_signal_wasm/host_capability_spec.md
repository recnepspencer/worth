# Host Capability Product Lane Spec

> **Status:** Completed 2026-04-30
>
> **Closeout:** [host_capability_closeout.md](./host_capability_closeout.md)
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Web runtime parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **React parent:** [react_adapter_spec.md](./react_adapter_spec.md)
>
> **Callback-computed prerequisite:** [host_callback_computed_spec.md](./host_callback_computed_spec.md)
>
> **Core runtime prerequisite:** [plans/worth_signal/worth_signal_temporal_async_roadmap.md](../../../plans/worth_signal/worth_signal_temporal_async_roadmap.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../../../plans/worth_signal/worth_signal_vision.md)
>
> **Core test requirements:** [plans/worth_signal/test-requirements.md](../../../plans/worth_signal/test-requirements.md)
>
> **Primary architectural driver:** add a typed host-capability lane to
> `worth-signals-wasm` so browser/runtime-local facts can participate in
> callback-authored product surfaces without turning ambient closure reads into
> fake reactive truth.

## Goal

Make host-derived runtime facts first-class product resources in
`worth-signals-wasm` so ordinary TypeScript can consume approved browser/runtime
inputs through typed capability handles instead of through ambient closure
folklore.

The milestone should enable authoring shapes in this family:

```ts
const signals = createSignals({
  hostCapabilities: hostCapabilityPlan({
    viewport: viewportCapability({ source: viewportSource() }),
    visibility: visibilityCapability({ source: visibilitySource() }),
  }),
});

const layout = signals.computed(() => {
  const width = signals.host.viewport.width();
  const visible = signals.host.visibility.isVisible();
  return visible && width > 900 ? "wide" : "narrow";
}, { id: "layout" });
```

The shipped surface now uses that explicit registration shape, and the
semantic contract is:

- host-derived facts are typed capability reads, not ambient JS reads
- capability invalidation is framework-owned
- restore, replay, import/export, and diagnostics remain honest about which
  host capabilities were present and what portability they permit

## Why This Spec Exists

Callback-computed closeout deliberately froze purity to captured signal reads
only.

That was the correct milestone boundary, but it left the next real wasm product
problem exposed:

- real products need viewport, visibility, connectivity, timers, persistence,
  and similar browser/runtime-local facts
- those facts cannot remain ambient closure reads without destroying replay,
  restore, diagnostics, and invalidation honesty
- if forms or API resources solve this locally, they will each invent a second
  host-truth model and the package will fracture immediately

This milestone exists to define one typed host-facing lane before broader
product surfaces are allowed to lean on host-local meaning.

## Hard Part

The hard part is not "read browser APIs from callbacks."

The hard part is freezing one honest contract across six things naive designs
collapse together:

- capability identity
- capability lifecycle and registration
- invalidation and delivery ownership
- replay/restore/import/export posture
- diagnostics and explanation artifacts
- product ergonomics

If the package lets any of those drift back into ambient callback behavior,
host capability will look ergonomic while silently becoming the new owner of
reactive truth outside the runtime.

The design fails if:

- arbitrary closure reads can masquerade as declared host dependencies
- capability changes invalidate broad runtime surfaces because precise routing
  is inconvenient
- replay or restore reuses committed values while hiding that the live
  capability was missing
- different capability families invent different lifecycle grammars
- React or product wrappers become the real owner of capability registration
  and invalidation
- the public API makes broad host access look cheap while hiding scans, glue,
  or compatibility fallbacks

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived web runtime with callback-authored derived nodes that depend on
> viewport, visibility, online/offline, timer, and persistence-backed host
> facts must converge to the same committed derived truth, the same invalidation
> history, the same restore/import compatibility classification, and the same
> diagnostics explanation regardless of whether work was driven by signal
> invalidation, host-capability change, branch restore, replay, React mount
> churn, or package-level consumer reuse.

If any supported path:

- lets ambient closure reads affect runtime truth without a typed capability
  contract
- changes host-capability meaning between ordinary execution, replay, restore,
  and import/export
- hides missing capability state by reusing committed values as though live
  reevaluation had succeeded
- broadens one capability change into unrelated callback reevaluation
- makes diagnostics unable to explain which capability family dirtied or denied
  the work

then the milestone has failed.

## Explicit Assumptions

- callback-computed closeout is the baseline; ordinary callback purity still
  means captured signal reads only unless a typed host capability is involved
- core async substrate work is a prerequisite for this roadmap sequence, but
  this milestone does not redefine core async lifecycle truth
- `worth-signals-wasm` may own host-facing capability packaging, registration,
  and bridge artifacts, but it may not turn host capabilities into a second
  source of derivation semantics separate from the runtime
- React remains a consumer of the product lane, not the owner of capability
  lifecycle or invalidation semantics
- capability families may differ in portability, but they may not differ in
  whether typed lifecycle and diagnostics obligations apply

## Governing Summaries

- `MENTALITY.md`
  The important thing it protects here is solving the dangerous substrate
  before the attractive product. This spec therefore starts from "how do host
  facts remain typed and replay-honest?" rather than "what browser APIs should
  be easy to call?"
- `arch_laws.md`
  The dominant laws here are 1, 7, 16, 32, 33, 34, 37, 40, and 41. Host
  capability must be a framework-owned resource family with explicit envelopes,
  lifecycle, counters, and proof-bearing handles rather than ambient callback
  convenience.
- `perf_laws.md`
  The important thing it protects is that host invalidation breadth,
  capability-registration churn, and reevaluation fanout cannot hide behind
  "easy" browser-facing APIs. This milestone must name measurement boundaries
  and boundedness counters at the public capability facade.
- `domain_laws.md`
  The important thing it protects is that capability families, capability
  lifecycle, capability diagnostics, and product ergonomics must live in
  responsibility-shaped domains rather than another giant wasm helper bucket.
- `worth_signal_vision.md`
  The important thing it protects is the separation between truth, derived
  execution, and integration. Host capability must reinforce that split, not
  smuggle host-local truth into derived execution as undocumented ambient
  behavior.
- `test-requirements.md`
  The important thing it protects is replay, restore, and boundedness honesty.
  This milestone must produce named capability-parity and boundedness tests
  rather than claiming host correctness from a few happy-path browser examples.
- `worth_signal_temporal_async_roadmap.md`
  The most important thing it protects is sequencing. Host capability belongs
  after canonical async/temporal substrate truth exists and before forms or
  resources try to consume browser-local facts.
- `wasm_product_roadmap.md`
  The most important thing it protects is that this milestone must be the first
  product lane that solves host-local facts once, so later forms/resources
  inherit one story instead of inventing their own.
- `web_runtime_spec.md`
  The most important thing it protects is that the wasm package is a real
  product surface, not low-level glue. Host capability therefore needs a
  coherent app-first product story, not just more raw functions.
- `react_adapter_spec.md`
  The most important thing it protects is that React remains a consumer of
  runtime truth. Capability subscription and invalidation must not drift into
  React-owned lifecycle or cache semantics.
- `host_callback_computed_spec.md`
  The most important thing it protects is the explicit deferral that made
  callback purity honest. This milestone should end that deferral by adding a
  separately typed resource family, not by weakening the callback-computed
  contract retroactively.

## Product Decision Lock

- host capability is a separately typed product lane, not an option bag on
  ordinary callback purity
- unsupported host reads remain non-reactive by contract
- capability families must be declared through one framework-owned registration
  plan, not by implicit ambient discovery or scattered ad hoc registration
  calls across product code
- capability reads must be mechanically distinguishable from signal reads in
  diagnostics and explanation artifacts
- each capability family must declare its restore/replay/import/export posture
- product surfaces may offer convenient accessors, but those accessors must
  lower to typed capability descriptors and runtime-owned invalidation
- capability handles and capability read contexts must be opaque/branded enough
  that ordinary structural objects cannot masquerade as registered capabilities
- live capability reads must require a runtime-owned evaluation/read witness at
  the Rust/lowering boundary so out-of-phase host reads are uncallable rather
  than merely discouraged

Normative consequence:

- any implementation that treats `window.innerWidth` read directly from a
  callback as equivalent to a registered viewport capability is out of spec
- any implementation that replays host-capability-derived state without an
  explicit compatibility or unavailability artifact is out of spec
- any implementation that lets forms or resources own capability invalidation
  semantics before the product lane exists is out of spec

## Scope

### In Scope

- typed host-capability family vocabulary for the wasm product surface
- framework-owned registration, lifecycle, and invalidation for admitted
  capability families
- capability reads from callback-authored product surfaces
- diagnostics and explanation artifacts for capability reads, dirtiness, and
  denial/unavailability
- restore, replay, and import/export compatibility classification for
  capability-bearing derived state
- counters and boundedness proofs at the product boundary

### Explicitly Out Of Scope

- broad arbitrary host object support
- letting any closure variable be upgraded into reactivity by declaration
- redefining core async lifecycle semantics
- forms abstractions
- API resource/query replacement ergonomics
- React-specific helper APIs beyond consuming the completed capability lane

## Architectural Model

This milestone introduces one new product/runtime-facing category:

- `HostCapabilityFamily`

That category must remain distinct from:

- authoritative host truth
- ordinary signal inputs
- derived signal state
- callback capability identity
- async lifecycle capability

The model is:

- the host adapter registers typed capability families with the wasm runtime
- each family exposes explicit read operations and invalidation events
- callback-authored product surfaces read capabilities through typed read
  contexts, not arbitrary host calls
- runtime-owned artifacts record which capability families were consumed
- restore, replay, and transport surfaces classify whether equivalent
  capabilities are still live, reattached, unavailable, or intentionally
  non-portable

At minimum, the milestone should freeze product/runtime concepts in this shape:

- `HostCapabilityRegistrationPlan`
- `HostCapabilityFamilyId`
- `HostCapabilityRegistration`
- `HostCapabilityDescriptor`
- `HostCapabilityReadWitness`
- `HostCapabilityReadArtifact`
- `HostCapabilityInvalidationArtifact`
- `HostCapabilityCompatibilityArtifact`
- `HostCapabilityUnavailabilityArtifact`

Exact names may evolve, but the ontology must survive intact.

## Declaration Model

This milestone should not permit host capability setup to sprawl across helper
calls, hidden globals, and adapter-local registration side tables.

The intended shape is one declaration source of truth at runtime creation or an
equally explicit sealed builder boundary, for example:

```ts
const signals = createSignals({
  hostCapabilities: hostCapabilityPlan({
    viewport: viewportCapability({
      source: windowViewportSource(),
      compatibility: "Reattachable",
    }),
    visibility: visibilityCapability({
      source: documentVisibilitySource(),
      compatibility: "LiveOnly",
    }),
  }),
});
```

The exact API may evolve, but the milestone should preserve these properties:

- product code declares capability families once
- each family declaration is explicit about compatibility posture
- runtime creation consumes one sealed capability plan rather than ambient
  later registration
- the same declaration shape feeds docs, diagnostics naming, and lifecycle
  setup

If the final implementation needs late registration for specific operational
reasons, it still must lower through the same declaration model and must not
become a second untyped capability surface.

## Capability Family Lock

This milestone should freeze a small, explicit first capability set rather than
claiming general host omniscience.

The initial admitted families should be chosen from product-critical,
structurally understandable browser/runtime facts such as:

- viewport or media-query facts
- document visibility/focus facts
- online/offline/connectivity facts
- timer/clock-facing host facts that belong above pure signal reads
- browser-local persistence-backed facts where the runtime can define explicit
  invalidation ownership

Each family must declare:

- identity basis
- registration shape
- invalidation mode:
  - push-driven
  - polled
  - manually committed
- invalidation trigger model
- read surface
- replay/restore/import/export compatibility posture
- diagnostics vocabulary
- public counters

Families are allowed to differ in portability and invalidation granularity.
They are not allowed to differ in whether those declarations exist.

For the first admitted families, the spec should push toward a concrete matrix
like this:

| Family | Likely invalidation mode | Likely compatibility posture | Practical example |
| --- | --- | --- | --- |
| Viewport / media facts | push-driven | `Reattachable` | width/height or media breakpoint affecting layout-derived signals |
| Visibility / focus facts | push-driven | `LiveOnly` or `Reattachable` | recomputation or suppression when a tab becomes hidden |
| Online/offline facts | push-driven | `Reattachable` | resource refresh policy or submission readiness |
| Timer / host clock facts | polled or push-driven | `SnapshotPortable` or `ImportDenied` depending on family | host-facing wall-clock or timer gates above pure signal reads |
| Persistence-backed facts | manually committed or push-driven | `ImportDenied` or `SnapshotPortable` depending on artifact story | local preference or draft source coordinated through explicit persistence events |

The exact cells may change, but the milestone should force this level of
specificity before implementation starts.

## Compatibility Taxonomy Lock

This milestone must not let every capability family invent its own vague
"sometimes portable" story.

Each admitted family must classify into one of a small explicit compatibility
taxonomy at registration time:

- `LiveOnly`
  Meaning the family can be consumed only while the exact live capability is
  registered in the current runtime.
- `Reattachable`
  Meaning replay or restore may continue if an equivalent family descriptor is
  reattached and compatibility checks succeed before resumed evaluation.
- `SnapshotPortable`
  Meaning committed derived artifacts may be carried forward honestly without
  claiming the live capability itself was transported.
- `ImportDenied`
  Meaning cross-host import or replay with missing capability must emit an
  explicit typed denial/unavailability result.

The exact names may evolve, but the milestone must freeze one shared taxonomy.
Capability families may map to different variants. They may not invent private
per-family compatibility language.

## Compile-Time Enforcement Targets

This milestone should make the dangerous mistakes hard to express, not merely
well-documented.

At minimum, the design should aim for these enforcement properties:

- ordinary product code cannot synthesize a registered capability handle
- live capability reads cannot occur without a runtime-owned read witness for
  the active evaluation frame
- out-of-phase reads, stale registrations, and family-mismatched descriptors
  are unrepresentable or uncompilable inside the Rust/lowering boundary
- family registration must flow through one sealed declaration plan so adding a
  new capability family forces compile-time updates at every lifecycle boundary
- compatibility classification artifacts are explicit enum/tag families rather
  than stringly-typed result blobs

TypeScript alone cannot carry the full proof burden here. The Rust/lowering
boundary must own the stronger guarantees and expose opaque product handles on
top.

## Phases

### Phase 1: Capability Ontology And Registration Boundary

Deliver:

- frozen capability-family vocabulary
- registration/destruction boundary for capability families
- one sealed `HostCapabilityRegistrationPlan`-style declaration path for
  capability setup
- proof-bearing capability descriptors and handles
- explicit capability-family compatibility classification categories

Must prove:

- capability families are not structurally WORTHable by ordinary product code
- capability registration is framework-owned rather than ambient global state
- each family has an explicit portability/compatibility classification
- adding a new capability family forces compile-time updates at lifecycle and
  propagation boundaries rather than silently defaulting

### Phase 2: Capability Read Contract And Callback Integration

Deliver:

- callback-visible capability read surface
- typed lowering from product convenience APIs into capability descriptors
- runtime-owned `HostCapabilityReadWitness`-style proof that live reads occur
  only in a valid evaluation/read phase
- runtime-owned read artifacts that distinguish signal reads from capability
  reads
- denial path for undeclared or unavailable capability access

Must prove:

- callback-computed diagnostics can distinguish capability reads from signal
  reads
- undeclared host reads do not silently become reactive
- capability reads compose with callback dependency capture without corrupting
  signal dependency truth
- live reads outside the allowed evaluation/read phase are mechanically denied
  by the boundary design rather than only by doc guidance

### Phase 3: Invalidation, Delivery, And Lifecycle Semantics

Deliver:

- capability-owned invalidation model
- family-specific invalidation routing and delivery artifacts
- disposal/unregistration semantics
- stale-registration and stale-read protection
- explicit batching/commit posture for capability invalidation so push-heavy
  families do not degenerate into per-event semantic churn by accident

Must prove:

- one capability change invalidates only the semantically affected callback
  surface
- stale capability registrations cannot silently deliver new lifecycle events
- lifecycle ownership remains in the runtime/package boundary rather than in
  React or convenience wrappers
- capability invalidation mode is visible and cost-honest at the public
  boundary rather than hidden behind callback convenience

### Phase 4: Restore, Replay, And Import/Export Honesty

Deliver:

- capability-bearing restore/replay classification artifacts
- explicit reattached, live, unavailable, and incompatible capability outcomes
- same-process and cross-host posture definitions where relevant
- package-facing restore/import/export truth that does not hide capability loss
- one shared compatibility taxonomy used across all admitted families rather
  than family-local ad hoc result language

Must prove:

- restore and replay preserve the same capability story the original runtime
  had, or emit typed incompatibility/unavailability
- missing or incompatible capability families do not masquerade as successful
  live reevaluation
- capability-bearing artifacts remain self-describing without querying the
  producer runtime
- capability family portability claims are explicit enough that a consumer can
  tell whether it got live, reattached, snapshot-portable, or denied behavior
  without reverse-engineering family-specific semantics

### Phase 5: Diagnostics, Counters, And Product Surface

Deliver:

- diagnostics artifacts that explain capability reads, invalidation causes, and
  denial/unavailability posture
- named public counters for capability registration, disposal, invalidation,
  reevaluation, and compatibility denial
- product docs and examples teaching one honest host-capability story
- package/runtime proofs at the public surface
- one final hostile certification suite that treats host capability as a
  long-lived runtime subsystem rather than as a convenience API family

Must prove:

- the public diagnostics boundary can explain host-capability-driven changes as
  clearly as signal-driven changes
- counters are visible at the product facade they justify, not hidden in
  internals
- a product user can learn host capability without being taught callback
  folklore or raw compatibility escape hatches
- the runtime/package closeout can survive hostile capability churn, restore,
  replay, React mount churn, and import/export denial paths without drifting
  into a second hidden truth engine

## Must Ship

- one frozen first-class host-capability family vocabulary
- framework-owned capability registration and lifecycle management
- typed capability descriptors, handles, and read artifacts
- callback integration that preserves the ordinary signal-read purity story
- restore/replay/import/export compatibility and unavailability artifacts
- diagnostics-visible capability causality
- named counters and boundedness contracts at the public boundary

## Must Preserve

- captured signal reads remain the ordinary callback purity basis
- unsupported host reads remain explicitly non-reactive
- replay, restore, and import/export never pretend unavailable capability state
  was live reevaluation
- host capability remains a product/runtime consumer lane rather than a new
  source of truth
- later forms and resource surfaces consume this lane instead of redefining it

## Acceptance Evidence

This milestone is complete only when all of the following are proven:

- `The Host Capability Purity Boundary Test`
  Proves that undeclared ambient host reads remain non-reactive while admitted
  capability reads produce explicit typed read artifacts.
- `The Host Capability Invalidation Scope Test`
  Proves that capability changes invalidate only the intended callback surfaces
  and expose boundedness counters for the work performed.
- `The Host Capability Restore And Replay Honesty Test`
  Proves that equivalent restore/replay paths preserve capability truth and
  that missing/incompatible capability families emit typed outcomes rather than
  fake success.
- `The Host Capability Product Boundary Typing Test`
  Proves that package-facing handles, descriptors, diagnostics artifacts, and
  compatibility artifacts are typed and self-describing rather than blob-like.
- `The Host Capability React Consumer Parity Test`
  Proves that React consumption of capability-bearing derived state remains a
  pure consumer of runtime truth rather than a second capability lifecycle
  engine.

These named tests are the minimum bar. They do not replace the final hostile
closeout suite described below.

## Adversarial Closeout Certification Matrix

The milestone is not closed because a few capability demos work. It closes only
after one explicit hostile certification pass proves that the whole
host-capability lane remains:

- deterministic
- runtime-owned
- replay/restore honest
- bounded by semantic delta
- diagnostics-explainable
- React-consumer-safe

The final suite must include at least the following hostile families.

### 1. The Ambient Read Rejection Torture Test

Purpose:

- prove unsupported host reads stay non-reactive
- prove capability authoring cannot silently widen into ambient closure truth

Stress:

- callbacks that mix declared capability reads with ambient reads such as:
  - `window.innerWidth`
  - `document.visibilityState`
  - `Date.now()`
  - mutable captured objects
- callbacks that change output because of ambient host state while no declared
  capability changed
- nested callback-computed graphs where only one layer uses a capability

Must verify:

- only declared capability reads produce capability read artifacts
- ambient reads never create dependency edges
- impurity or undeclared-host-read diagnostics stay explicit if the package
  chooses to surface them
- equivalent declared-only runs and ambient-mixed runs do not get conflated in
  diagnostics or replay classification

### 2. The Stale Registration And Zombie Delivery Test

Purpose:

- prove capability lifecycle is framework-owned and generation-safe

Stress:

- rapid register/dispose/re-register cycles for the same family
- updates delivered after disposal
- updates delivered after replacement with a new registration
- mixed capability updates while React mounts and unmounts repeatedly

Must verify:

- stale registrations cannot deliver into current runtime truth
- disposed capabilities cannot continue invalidating derived nodes
- replacement registrations do not inherit stale lifecycle events
- teardown removes product-surface observers cleanly instead of relying on
  process-lifetime globals

### 3. The Fanout Boundedness And Frontier Precision Test

Purpose:

- prove capability invalidation breadth scales with affected meaning, not with
  graph size or registered capability count

Stress:

- large graphs with many derived nodes, where only a small frontier depends on
  one capability family
- mixed capability and signal invalidations in one transaction window
- high-frequency push-driven capability updates
- branch-local graphs with different host-capability dependency frontiers

Must verify:

- invalidation touches only capability-dependent derived surfaces
- named counters explain:
  - invalidation breadth
  - callback reevaluation breadth
  - denied broad-fanout or degraded modes if they exist
- unrelated signal-only graphs do not reevaluate because a capability changed

### 4. The Restore / Replay / Reattach Honesty Nightmare

Purpose:

- prove capability-bearing historical artifacts never fake live reevaluation

Stress:

- same-process restore with live capability still registered
- restore with missing capability
- restore with equivalent but newly attached capability
- replay from snapshots with branch churn before and after capability changes
- import/export between runtimes with intentionally incompatible capability
  descriptors

Must verify:

- exact live, reattached, unavailable, and denied outcomes stay distinct
- restore and replay produce explicit capability compatibility artifacts
- missing capability state never masquerades as successful recomputation
- equivalent restore suffixes converge to identical digests when compatibility
  posture allows them to proceed

### 5. The Host Capability Identity WORTHry Test

Purpose:

- prove compile-time and runtime boundaries reject fake capability objects and
  mismatched family descriptors

Stress:

- structurally WORTHd handles
- family-mismatched descriptors
- out-of-phase reads without a live read witness
- cross-runtime capability handle reuse

Must verify:

- normal product code cannot synthesize valid registered handles accidentally
- raw/foreign capability lookalikes deny before semantic work begins
- read-witness requirements remain enforced at the Rust/lowering boundary
- cross-runtime capability misuse cannot silently participate in dependency
  capture

### 6. The React Consumer Parity And Mount Churn Test

Purpose:

- prove React remains a consumer of runtime truth instead of becoming a shadow
  capability lifecycle owner

Stress:

- repeated mount/unmount cycles while capability updates continue
- multiple stores consuming the same runtime
- direct runtime updates without any adapter-local patching
- capability-backed derived nodes that are also observed through diagnostics and
  history

Must verify:

- React never becomes the only place where capability updates stay fresh
- mount churn does not mutate capability registration semantics
- diagnostics and rendered values stay in parity with the underlying runtime
- installing React consumers does not widen capability invalidation semantics

### 7. The Multi-Family Mixed Churn Test

Purpose:

- prove family-local correctness composes under real mixed workloads

Stress:

- visibility, viewport, online/offline, timer, and persistence-backed families
  all changing in one long-lived session
- overlapping derived nodes that read different family subsets
- branch fork / restore during mixed capability churn
- policy-rich async consumers once later milestones begin to depend on the same
  lane

Must verify:

- one family does not invent lifecycle grammar that leaks into another
- mixed-family invalidation remains explainable by family
- per-family compatibility posture survives mixed replay and restore paths
- counters remain attributable by family and by public boundary

### 8. The Long-Session Retention And Diagnostics Integrity Test

Purpose:

- prove the lane remains honest after long sessions, not just short demos

Stress:

- many capability changes over long-lived sessions
- repeated snapshot/restore cycles
- diagnostics richness changes if the package exposes tiering later
- retained history truncation or availability-policy changes

Must verify:

- retained and unavailable capability history stays explicitly classified
- diagnostics richness changes do not alter canonical runtime truth
- history retention does not silently erase the difference between
  reattached/live/unavailable outcomes
- package diagnostics remain self-describing after long-session churn

## Complexity / Proof Obligations

The milestone must expose named counters for at least:

- capability registration count
- capability disposal count
- capability invalidation count
- capability-driven callback reevaluation count
- capability compatibility denial count
- capability unavailability artifact count
- broad-fanout denial count for capability invalidation

The milestone must also declare named complexity contracts for:

- capability registration
- capability read lowering
- capability invalidation routing
- capability-bearing restore/replay classification
- capability diagnostics materialization

These counters must be exposed through named public artifacts or entrypoints at
the product boundary. They may not live only in internal telemetry.

The final hostile certification bundle must also emit canonical reports for:

- capability registration and disposal lineage
- capability invalidation breadth and reevaluation breadth
- capability-family compatibility outcomes across restore/replay/import
- React-consumer parity under mount churn
- retained vs unavailable capability-history posture where historical artifacts
  are intentionally compacted

Each report must be machine-checkable enough that equivalent runs can be
compared by canonical digest rather than by manual inspection alone.

## Sequencing Notes

This milestone belongs before forms and resources because:

- both products need browser-local facts
- neither product is allowed to become the first owner of host invalidation
  semantics
- callback-computed closeout already made the deferral explicit

This milestone belongs after the async prerequisite because:

- later host-capability families will intersect with async/resource products
- restore, replay, compatibility, and diagnostics posture should consume one
  canonical async substrate rather than coding against a moving target

## Milestone Done When

This milestone is done only when ordinary TypeScript can consume host-local
facts through typed capability reads and get:

- explicit capability identity
- runtime-owned invalidation
- replay/restore/import honesty
- diagnostics that name the capability story directly
- bounded counters at the public surface

without teaching users that ambient closure reads are secretly reactive after
all.

Operationally, that means the milestone does not close until the final hostile
certification suite has passed and the emitted reports prove:

- no ambient host read became fake reactive truth
- no stale registration or stale delivery survived lifecycle churn
- no restore/replay/import path lied about live capability availability
- no React consumer became a second capability-truth engine
- no long-session history or diagnostics path erased the difference between
  live, reattached, unavailable, and denied capability outcomes

## Closeout Summary

This milestone is now closed.

The delivered surface includes:

- explicit `hostCapabilityPlan(...)` registration
- admitted families for `visibility`, `viewport`, `online`, `clock`, and
  `persistence`
- typed `signals.host.*` reads with runtime-owned invalidation
- explicit same-runtime exact restore vs portable import truth
- host-capability diagnostics events, lineage, breadth, and canonical digests
- host-capability transport reports over unavailable callback-bearing artifacts
- runtime, package, React, and hostile certification evidence for the shipped
  family set

What this milestone intentionally did not do:

- admit arbitrary ambient host reads as reactive
- move host-capability lifecycle into React or framework adapters
- solve forms or API resources
- generalize transport posture beyond the admitted compatibility taxonomy and
  first shipped families

The formal closeout map, evidence table, and verification record live in
[host_capability_closeout.md](./host_capability_closeout.md).
