# Worker-First Product Entrypoint Correction Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Web runtime parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **Corrective predecessor:** [worker_runtime_placement_plan.md](./worker_runtime_placement_plan.md)
>
> **Closeout completed by this plan:** [worker_runtime_placement_closeout.md](./worker_runtime_placement_closeout.md)
>
> **Test requirements parent:** [worker_runtime_test_requirements.md](./worker_runtime_test_requirements.md)

## Summary

This follow-on plan completes the remaining product-entrypoint work required for
the worker-runtime placement milestone closeout to be fully true in the shipped
package.

The worker substrate, typed bridge taxonomy, replay or restore capability
artifacts, and Phase 7 worker-first product-guidance certification are all
implemented. But the shipped package entry lane still constructs the main-thread
runtime by default and still teaches `createSignals()` as if it were an
ordinary synchronous main-thread constructor.

Until this plan lands, the crate remains incomplete at the product entry
boundary:

- worker-first is certified as the recommended default posture
- the shipped product entrypoint still defaults to main-thread runtime
- user-facing docs still teach the old default
- compatibility mode is ambient instead of explicit

This plan finishes that mismatch instead of working around it.

It keeps `createSignals()` as the normal front door, but changes its contract so
it becomes an async worker-first constructor for the canonical app surface.
Main-thread runtime remains supported through an explicit deployment option on
that same front door, plus explicit typed worker-unavailable artifacts.

## Goal

Make `createSignals()` the honest canonical worker-first product constructor for
`worth-signals-wasm`, while keeping main-thread compatibility available only as
an explicit secondary construction lane.

## Why This Plan Exists

The worker-runtime placement milestone solved the hard substrate:

- worker-owned runtime authority
- typed placement taxonomy
- typed host ingress and egress
- callback portability honesty
- replay, restore, import, and export capability posture
- parity and performance certifications

But the shipped app surface still exposes the old default construction story.

That means the worker-placement promise is still unfinished at the most
important user-visible place: the first line of application code.

This plan exists to finish that final product gap without undoing the good part
of the web runtime design:

- one obvious app-first entrypoint
- no user-facing worker ceremony for the normal lane
- no hidden fallback from worker-first into compatibility mode
- no second truth engine on the main thread

## Adversarial Constraint

This correction must survive the following hostile condition:

> A web application using the ordinary published package entry lane must
> construct a worker-owned runtime by default, expose the same committed truth
> and lifecycle semantics as explicit compatibility mode, reject worker
> unavailability without silently collapsing into main-thread authority, and
> keep construction, delivery, diagnostics, and host-bridge cost honest enough
> that no cheap-looking API hides broad main-thread runtime work.

If `createSignals()` can still:

- silently create a main-thread runtime in ordinary browser use
- partially bootstrap worker-first mode and then collapse into compatibility
- expose sync-looking semantics while actually depending on unacknowledged async
  readiness
- or teach docs and examples that the old main-thread default is the normal app
  lane

then this correction has failed.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the real structural lie
  first. The problem is not "API polish"; it is that the certified default
  posture and the shipped default posture disagree.
- `arch_laws.md`
  The most important laws here are 7, 17, 20, 21, 24, 27, 30, 33, 40, and 41.
  Construction must lower into an explicit deployment plan, boundary crossings
  must emit typed envelopes or artifacts, compatibility must remain explicit,
  and the public API must honestly declare worker bootstrap as a boundary.
- `composition_laws.md`
  The most important thing it protects is keeping product-entry construction,
  deployment planning, worker bootstrap, compatibility construction, and docs
  migration as named responsibilities rather than burying them inside one broad
  convenience wrapper.
- `domain_structure_laws.md`
  The most important thing it protects is responsibility placement. Worker
  construction planning, worker bootstrap, compatibility construction, and
  entrypoint documentation each need distinct homes instead of another giant
  `signals.ts` gravity well.
- `perf_laws.md`
  The most important thing it protects is boundary honesty. `createSignals()`
  must not conceal broad bootstrap, bridge, or fallback work behind a cheap
  synchronous-looking API, and compatibility fallback must not happen by hidden
  default.
- `web_runtime_spec.md`
  The most important thing it protects is that `createSignals()` remains the
  primary app-first entrypoint and that initialization burden belongs to the
  package, not the user.
- `wasm_product_roadmap.md`
  The most important thing it protects is roadmap sequencing. This work belongs
  immediately after worker placement because it closes the product-entrypoint
  contract that the milestone claimed, not a separate unrelated capability.
- `worker_runtime_placement_plan.md`
  The most important thing it protects is the product decision lock that
  worker-first is canonical and compatibility is explicit and secondary.
- `worker_runtime_test_requirements.md`
  The most important thing it protects is certification rigor. This correction
  is only closed when the entrypoint itself is covered by named proof, not when
  a package example "looks right."
- `worker_runtime_placement_closeout.md`
  The most important thing it protects is the concrete list of already-built
  worker substrate capabilities and the fact that its closure claim should end
  up being true in the shipped package. This plan must consume that substrate
  rather than inventing a second worker story in package glue.

## Product Decision Lock

- `createSignals()` remains the normal app-facing entrypoint
- `createSignals()` becomes asynchronous and worker-first by contract
- ordinary browser callers do not manually instantiate or configure a worker to
  get the canonical runtime posture
- main-thread compatibility remains supported, but only through an explicit
  deployment option on `createSignals()`
- the common path must stay tiny enough that normal app code usually needs no
  construction options at all
- the advanced path must expose only the next real caller-owned choice rather
  than a large bootstrap bag
- worker unavailability must never trigger hidden fallback from
  `createSignals()` into main-thread authority
- the package must expose typed worker-unavailable construction artifacts or
  equivalent typed rejection surfaces
- worker-unavailable artifacts must teach the explicit compatibility recovery
  path rather than merely reporting failure
- docs, examples, README, and package types must teach the same construction
  truth the runtime now implements

## Phases

### Phase 1: Construction Contract Correction

Correct the public construction contract before changing bootstrap internals.

This phase exists because the current mismatch is public and architectural:

- `createSignals()` currently returns a ready callable surface synchronously
- worker-first construction is a real boundary crossing
- the package currently claims worker-first recommendation while exposing a
  main-thread default

This phase must define one canonical public construction vocabulary:

- `createSignals()` returns `Promise<CallableSignals<...>>`
- `createSignals()` with no deployment option means "construct the canonical
  worker-first runtime posture"
- `createSignals({ deployment: "mainThreadCompatibility" })` exists for the
  supported explicit main-thread mode
- the construction options surface stays intentionally small and does not become
  a grab bag of bootstrap mechanics
- one typed worker-unavailable construction artifact exists for environments
  where dedicated worker construction cannot succeed honestly

Required consequences:

- ordinary package consumers can still start from one obvious entrypoint
- the API shape now acknowledges worker bootstrap as a real boundary
- compatibility construction is explicit at the call site instead of ambient in
  implementation defaults
- no shipped package version may expose two ambiguous meanings of
  `createSignals()` across docs, types, and runtime behavior at once
- normal app docs can teach the common path without detouring through
  construction-policy ceremony

Phase 1 gate:

- no later phase begins until the package-level construction contract names the
  worker-first lane, the compatibility lane, and the worker-unavailable lane
  explicitly enough that docs and tests can target them without ambiguity

### Phase 2: Deployment Planning And Bootstrap Boundary

Construction must lower into a deployment plan before execution begins.

This phase creates a product-entry deployment planner that classifies the
requested construction into exactly one lowered family:

- `workerFirst`
- `mainThreadCompatibility`
- `workerUnavailable`
- `denied`

The planner owns:

- environment and capability admissibility checks relevant to worker
  construction
- worker script/bootstrap URL resolution or equivalent package-owned bootstrap
  record preparation
- compatibility-lane admission
- typed artifact production when worker-first construction cannot proceed
- typed explanation surfaces for why worker-first construction was denied or
  unavailable

Required consequences:

- construction legality is resolved before runtime authority is instantiated
- `createSignals()` does not guess, partially construct, and silently fall back
- compatibility mode is never entered from the worker-first entry lane except
  through explicit typed rejection and caller choice
- advanced users can inspect why a construction request could not admit the
  worker-first posture without having to reverse-engineer package internals

Normative proof chain:

- raw caller intent lowers into a sealed construction request
- the sealed request lowers into a sealed deployment plan
- the bootstrap executor consumes only the lowered plan, never re-deciding
  deployment posture internally

Phase 2 gate:

- no worker-first constructor path may directly call raw runtime creation
  without first consuming a deployment plan that records posture, artifact
  family, and compatibility stance

### Phase 3: Worker-First Callable Surface Construction

This phase binds the existing worker substrate to the ordinary callable app
surface.

The package must construct one worker-owned runtime and then expose the same
app-facing callable surface categories that current main-thread code exposes:

- inputs, computed, outputs
- controllers and graphs
- resources and API families
- diagnostics, history, and adapters
- host capability registration and host effect routing

This phase is not "proxy arbitrary calls to a worker."

It must define:

- a worker-owned callable surface construction path
- package-owned readiness semantics for the resolved `Promise`
- bridge ownership for host capabilities, browser history, and host effects
- disposal and lifecycle coordination for the constructed callable surface
- explicit handling for public delivery and diagnostics reads so worker-first
  runtime truth does not rehydrate a hidden main-thread engine

Required consequences:

- after `await createSignals()`, the caller holds the normal app-facing surface,
  but runtime-owned derived truth lives in the worker
- host-boundary work remains on the main thread through typed lanes only
- no compatibility-mode runtime is constructed as a hidden helper for worker
  mode

Phase 3 gate:

- no worker-first callable surface is accepted until ordinary app reads,
  writes, resources, diagnostics, history, and host-capability surfaces can be
  reached without introducing a second main-thread runtime authority

### Phase 4: Explicit Compatibility Construction

Main-thread compatibility remains real, but it must become unmistakably
secondary and explicit on the same entrypoint.

This phase defines the explicit compatibility deployment option and its product
guidance contract.

The compatibility lane must:

- construct the supported main-thread runtime posture directly
- preserve semantic parity with worker-first mode
- keep all existing worker-unavailable and parity artifacts honest
- expose an option shape that makes its authority posture obvious at the call
  site

This phase must also define exactly how `createSignals()` failure hands off to
compatibility:

- the worker-first constructor rejects with a typed worker-unavailable artifact
- callers that want compatibility must opt into the explicit deployment option
- the package may document a guarded fallback pattern, but it may not perform it
  automatically

Required consequences:

- compatibility remains supported without becoming ambient
- product guidance and actual package behavior now agree
- environments without worker support still have a real lane, but users must
  name it
- documented recovery from worker-unavailable to explicit compatibility mode is
  short, obvious, and identical across README, package docs, and examples

Phase 4 gate:

- no docs or examples may present compatibility construction as the default or
  implied behavior of `createSignals()`

### Phase 5: Documentation, Type Surface, And Certification Closeout

This phase closes the product-facing mismatch and proves it stays closed.

It must ship:

- package type-surface updates for async `createSignals()`
- explicit compatibility deployment-option types
- typed worker-unavailable construction artifacts or equivalent typed rejection
  surfaces
- one canonical documented recovery snippet for explicit compatibility fallback
- one inspectable construction explanation surface for advanced debugging
- updated package README and crate docs
- worker-placement closeout updates that become true because this plan lands,
  not because the closeout language is weakened
- updated examples that `await createSignals()`
- certification and package tests that prove the entrypoint contract instead of
  assuming it

The docs correction is not optional. The worker placement plan explicitly
required docs and examples to teach worker-first as the recommended posture.
This phase closes that promise at the package entry boundary.

Phase 5 gate:

- this corrective milestone is not closed until user-facing docs, package types,
  and certification all teach the same worker-first construction truth
  implemented by the package

## Must Ship

This correction is not done because one demo can `await createSignals()`.

It is done only when `worth-signals-wasm` ships:

- an async `createSignals()` contract that constructs the canonical
  worker-first runtime posture
- one explicit compatibility deployment option for supported main-thread runtime
  construction
- one typed construction artifact family for worker-unavailable or denied
  worker-first construction
- a sealed deployment-planning layer that lowers public construction intent
  before runtime authority is instantiated
- a deliberately small public options surface whose common path is zero-option
  worker-first construction
- a worker-first callable app-surface construction path that consumes the
  existing worker substrate instead of bypassing it
- explicit lifecycle and disposal coordination for worker-first constructed app
  surfaces
- package type surfaces, README, docs, and examples that all teach the same
  worker-first construction truth
- one canonical explicit compatibility recovery snippet shared across user-facing
  docs
- one construction explanation surface that lets advanced callers inspect denial
  or unavailability causes without widening the common path
- package or cert-visible entrypoint verification proving that the normal app
  lane no longer defaults to main-thread runtime authority

At minimum, the construction API must expose named authority categories for:

- canonical worker-first construction
- explicit main-thread compatibility construction selected through deployment
  options
- worker-unavailable construction failure
- denied construction failure when the requested lane cannot be admitted

The package may add helper ergonomics around those categories later, but it may
not collapse them back into one ambiguous constructor.

## Must Preserve

- `createSignals()` remains the primary app-facing entrypoint name
- runtime truth remains singular; worker-first construction must not create a
  second main-thread runtime authority
- host capability, browser history, host effect, replay, restore, diagnostics,
  and public delivery semantics continue to consume the existing worker
  substrate rather than being reimplemented in package glue
- compatibility mode remains semantically real and supported
- compatibility mode remains explicit and secondary rather than ambient and
  default
- worker unavailability remains a typed artifact, not a hidden fallback
- callback portability limits remain honest; this correction must not weaken the
  already-closed worker placement truth just to make app construction look
  simpler
- user ergonomics remain front-door simple even though the runtime authority
  contract becomes stricter

## Acceptance Evidence

This correction is complete only when the package can prove all of the
following:

- `createSignals()` constructs worker-first runtime authority by default in
  environments where dedicated worker support is available
- `createSignals()` does not silently fall back to main-thread compatibility
  when worker-first construction is unavailable
- worker-unavailable construction surfaces emit explicit typed artifacts or
  equivalent typed rejection surfaces naming the compatibility option
- explicit compatibility construction converges to the same committed runtime
  truth, lifecycle truth, and diagnostics or history truth as worker-first
  construction for semantically equivalent workloads
- package docs and README teach `await createSignals()` as the normal app lane
- package docs and README teach one short explicit compatibility recovery path
  instead of forcing callers to invent one
- package type surfaces prevent ordinary consumers from treating worker-first
  construction as synchronous
- no package-level entrypoint test can pass while ordinary browser construction
  still defaults to main-thread runtime authority
- the worker-placement closeout narrative is fully true for the shipped package
  entrypoint rather than only true for the underlying substrate
- advanced construction inspection can explain worker denial or unavailability
  without changing the common construction lane

Required named proof families:

- `The Async Worker-First Entrypoint Construction Test`
- `The Explicit Compatibility Construction Test`
- `The Worker Unavailable Construction Artifact Test`
- `The No Hidden Main-Thread Fallback Test`
- `The Worker-First Entrypoint Semantic Parity Test`
- `The Docs And Package Surface Alignment Test`
- `The Construction Explanation Surface Test`
- `The Canonical Compatibility Recovery Documentation Test`

The verification package for this correction should include canonical artifacts
or equivalent typed evidence for:

- construction request digest
- deployment plan digest
- bootstrap posture digest
- worker runtime identity digest
- compatibility runtime identity digest
- worker-unavailable artifact digest
- construction explanation digest
- compatibility recovery guidance digest
- entrypoint guidance digest
- entrypoint type-surface digest
- entrypoint docs alignment digest

## Architectural Notes

- This plan intentionally preserves the app-first front door from
  `web_runtime_spec.md`. The correction is not "make users learn a worker
  constructor." It is "make the existing front door mean what worker placement
  says it means."
- This plan also follows the DX rule that the common path should read like
  intent and the advanced path should expose the next lower control layer
  without fracturing into sibling entrypoints. `await createSignals()` remains
  the common path; `await createSignals({ deployment: "mainThreadCompatibility" })`
  is the explicit advanced path.
- Worker-unavailable recovery is part of DX, not just error handling. The
  package must make the honest recovery path obvious without silently taking it
  for the caller.
- Construction explanation belongs at the advanced inspection layer, not in the
  common-path call signature. That preserves a small front door while still
  honoring the inspectable-plan requirement from the DX laws.
- The most honest API shape is async worker-first construction plus explicit
  compatibility construction. That follows `arch_laws.md` law 20: public API
  shape must acknowledge real orchestration boundaries.
- The deployment planner must live above runtime creation, not inside it.
  Otherwise the package will keep violating `arch_laws.md` laws 17, 27, 30, and
  41 by re-deciding posture during execution.
- The worker-unavailable lane is not an implementation nuisance. It is part of
  the product truth contract and must remain typed and machine-visible.
- Package docs and README are part of the product surface here. Because the
  current mismatch is user-facing, documentation correction is architectural
  completion work, not optional cleanup.

## Sequencing Notes

This plan belongs immediately after worker-runtime placement closeout because it
finishes the last product-entrypoint condition required for Milestone 9 closure
to be fully true in the shipped package.

It exists because the current state is:

- substrate complete enough to support worker-first product construction
- product guidance certified as worker-first by default
- package entrypoint and docs still teaching the old main-thread default

That makes this work the final completion pass for Milestone 9 at the package
entry boundary, not a separate future feature family.

It belongs before more worker-adjacent product work because:

- later worker-facing features would otherwise inherit a dishonest entry lane
- keeping the old synchronous main-thread default would continue to falsify the
  certified product guidance
- package-level DX and publish-facing examples are part of the shipped product,
  not post-closeout polish

## Required Self-Check

- Does this plan solve a real structural problem or just package work
  cosmetically?
  Yes. It corrects a direct contradiction between certified product posture and
  shipped package construction semantics.
- Is the adversarial constraint precise and load-bearing?
  Yes. It targets silent main-thread defaulting, hidden fallback, async-boundary
  dishonesty, and doc-surface drift directly.
- Does the plan preserve crate authority boundaries?
  Yes. Worker substrate remains the owner of runtime placement truth; the
  package entry surface becomes a planner and constructor, not a new runtime.
- Does the plan define proof obligations, not just implementation tasks?
  Yes. It names construction artifact families, deployment-plan lowering, docs
  alignment, and explicit certification families.
- Could a competent engineer map this plan into honest types, modules, and
  tests?
  Yes. The phases isolate contract correction, planning, worker construction,
  compatibility construction, and closeout proof cleanly.
- Does this work belong here in roadmap order?
  Yes. It closes the remaining user-facing contract gap immediately after the
  worker substrate milestone that made the correction possible.
