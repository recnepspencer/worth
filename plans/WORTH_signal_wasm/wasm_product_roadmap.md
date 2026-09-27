# worth-signals-wasm Product Roadmap

## Purpose

This document defines the remaining product roadmap for `worth-signals-wasm`
after callback-computed closeout.

It is a future-only roadmap. It does not treat the wasm package as unfinished
runtime foundation, and it does not treat later package products as incidental
helpers. It exists to sequence the remaining work required to turn the closed
callback/runtime substrate into a broader application-facing product line
without splitting reactive truth across multiple convenience layers.

The governing rule is authority-specific:

- native `worth-signal` owns derived execution, invalidation, recomputation,
  branch lifecycle, and reconstructible runtime history
- platform application truth remains Query/Relational authority carried into
  Signal through the runtime bridge
- standalone browser application values may use one explicit, in-memory
  TypeScript local truth authority inside the product runtime
- package projections, diagnostics, demos, and UI consume those authorities;
  they do not redefine them

`worth-signals-wasm` may author, type, route, explain, and package host-facing
surfaces. It may also own process-local standalone value truth when no
relational runtime exists, but that authority must be structurally named,
produce one canonical commit artifact, project one-way into Signal, and remain
explicitly weaker than relational durability. It must not define temporal or
derived-execution semantics that belong to core `worth-signal`, impersonate
authority semantics that belong to `WORTH-relational`, or hide truth engines
inside adapters, resources, forms, router code, demos, or UI state.

## Shipped Baseline

The roadmap no longer tracks the already-shipped foundation as future work.

The current shipped baseline includes:

- app-first `createSignals()` runtime product surface
- callback-first `computed(() => ...)` as a real runtime-owned derived lane
- typed host capability product lane with:
  - `hostCapabilityPlan(...)`
  - admitted first families:
    - `visibility`
    - `viewport`
    - `online`
    - `clock`
    - `persistence`
  - diagnostics-visible lineage, breadth, and transport reports
  - hostile certification and package-proof closeout
- coherent `input`, `computed`, and `output` product surfaces
- runtime-scoped callback lifecycle and generation-aware callback ownership
- React consumption as a disciplined consumer of runtime truth
- typed diagnostics, history, replay, merge, and package artifact surfaces
- release-proof packaging and temp-consumer verification
- explicit same-process exact restore vs callback-unavailable portable transport
  truth

The shipped closeout reference for the latest major wasm milestone is
[host_callback_computed_spec.md](./host_callback_computed_spec.md).

The shipped closeout reference for the first follow-on product milestone is
[host_capability_closeout.md](./host_capability_closeout.md).

## Roadmap Rules

Rules for every remaining wasm product item:

- each milestone must describe a real product capability, not just a helper API
  family
- each milestone must preserve separate authority, execution, and package
  ownership
- each milestone must preserve replay, restore, diagnostics, and boundedness
  honesty at the public boundary it introduces
- each milestone must consume core runtime semantics rather than redefining
  async, temporal, or invalidation truth in package glue
- each milestone must define concrete acceptance evidence through runtime
  proofs, package proofs, diagnostics artifacts, or product-level certification
  scenarios
- no milestone is complete until both implementation and product-facing trust
  evidence exist
- forms and resources must never become the first place where host-capability
  or async lifecycle meaning is truly defined
- package APIs, diagnostics, and type surfaces must remain clean, explicit, and
  library-grade rather than becoming bags of host-specific flags
- later package products must not require main-thread execution for graph work
  that does not actually depend on browser-owned host boundaries
- any standalone TypeScript truth layer must remain physically separate from
  native Signal state, use typed basis and commit artifacts, and prove that all
  Signal state can be destroyed and rebuilt from local truth alone
- no native Rust Signal type may own standalone application-value history,
  manual merge decisions, or custom resolved values

## Critical Path

This section is the first build priority.

These are the milestones that most directly keep the next wasm products from
recreating the exact convenience-era failure mode the callback-computed
closeout just removed:

- ambient browser facts turning back into reactive folklore
- forms inventing a second store
- API resources inventing a second async and freshness engine

Even here, the wasm package is still only productizing runtime truth.
Authority, derived execution, temporal legality, async lifecycle policy, and
canonical replay/restore semantics remain in the parent runtimes.

If this section is weak, the next wasm products will inherit the classic drift
pattern:

- browser facts become ambient closure reads again
- app-facing helpers redefine semantics that should have stayed runtime-owned
- diagnostics stop at the product boundary instead of explaining the real
  lifecycle
- one product surface quietly becomes the semantic owner for every unfinished
  concept underneath it

## Dependency Gate: Async-Node Substrate

This roadmap begins only after the core async-node substrate is available from
the `worth-signal` roadmap work currently being finished on another branch.

The intended dependency order is:

1. async nodes and resource lifecycle substrate exist in core runtime truth
2. wasm product work adds host capability
3. wasm product work adds controller-first composition and graph publication
4. wasm product work adds scoped controller identity and graph-owned lifecycle
5. wasm product work adds opaque identity and ergonomic authoring
6. wasm product work adds forms
7. wasm product work adds API surface
8. wasm product work hardens API-surface developer ergonomics
9. wasm product work adds router and navigation projection
10. wasm product work adds worker-first runtime placement and main-thread host
    bridge truth
11. wasm product work adds response lens contracts for advanced resource
    response topology

That order is normative for this roadmap.

Before Milestone 1 begins, the following must already be true:

- async/resource lifecycle is runtime-owned
- retry, timeout, cancellation, supersession, and revalidation semantics are
  real runtime policy families rather than adapter-local conventions
- replay, restore, branch, rollback, and diagnostics parity exist for async
  work
- wasm does not need to invent a second pending/fulfilled/rejected truth model

If those conditions are not true, this roadmap must pause rather than coding
against moving semantic targets.

## Dependency Gate: Generic Aspect Capacity

This roadmap also depends on one deeper `worth-signal` rewrite that belongs to
core runtime truth rather than the wasm package:

- generic aspect capacity

The current aspect substrate is good enough for the already-shipped wasm
surface, but it is not the end-state capacity story for the broader product
line. Later package products should be able to rely on aspect-heavy feature
models without forcing WORTH to choose between low capacity and dishonest
performance.

The required direction is:

- the runtime must support a generic aspect-capacity family rather than one
  fixed aspect width
- supported capacity classes must cover the practical range from `8` aspects up
  through `264` aspects
- the runtime must preserve bounded hot-path behavior across those capacity
  classes rather than quietly degrading into broad scans or allocation-heavy
  fallback

This is not a wasm-only milestone. It is a core `worth-signal` architectural
rewrite that the wasm roadmap must acknowledge because later package products
will lean on aspect breadth more aggressively than the current surface does.

The governing expectation is:

- small-capacity deployments should not pay for large-capacity machinery
- large-capacity deployments should not need a different semantic model
- aspect width must remain a declared/runtime-owned capability boundary, not an
  ambient implementation accident

Before later aspect-heavy wasm products are considered closed, the parent
runtime should be able to prove:

- one canonical aspect model across capacity classes
- no semantic drift between `8`, `16`, `32`, `64`, `128`, `256`, and `264`
  aspect regimes
- named counters and proof tests for invalidation breadth, version tracking,
  dependency filtering, and memory/coordination posture under wider aspect
  capacity
- no package-facing API lie where aspect-rich authoring looks cheap while the
  core runtime is doing hidden broad work

Roadmap consequence:

- this dependency should be treated the same way as the async-node substrate:
  wasm can productize on top of it, but wasm must not become the place where
  generic aspect-capacity semantics are invented ad hoc
- this dependency is not a blocker for the next wasm milestone on controller
  scope and graph-owned lifecycle, because that milestone is about naming,
  ownership, and boundary truth rather than aspect-width expansion

## Milestone 1: Host Capability Product Lane (Completed)

Engineering spec: [host_capability_spec.md](./host_capability_spec.md)

Formal closeout: [host_capability_closeout.md](./host_capability_closeout.md)

### Goal

Add a typed host-capability lane for non-signal host-derived facts so callback
code can consume approved browser/runtime inputs without pretending ambient
closure reads are reactive truth.

### Must Ship

- a frozen wasm-facing host-capability vocabulary
- typed handles or descriptors for approved host-capability families
- explicit ownership, invalidation, registration, and disposal semantics
- diagnostics and explanation artifacts that name host-capability reads and
  invalidation causes
- replay, restore, and import/export posture for each admitted capability
  family
- counters and boundedness proof for capability registration, invalidation,
  delivery, and callback reevaluation

### Must Preserve

- host capability remains a typed lane, not ambient closure permission
- capability semantics remain runtime-truth consumers rather than React-local
  or browser-local shadow state
- unsupported host reads remain explicitly non-reactive by contract
- callback-computed purity for ordinary signal-only code remains simple and
  does not become entangled with optional host-capability machinery

### Explicit Boundary

Milestone 1 includes viewport, visibility, online/offline, timers, local
persistence-backed facts, and similar host-facing runtime inputs where the
package needs an explicit typed lane above pure signal reads.

Milestone 1 does not include letting arbitrary closure reads become reactive by
declaration, nor does it include burying host invalidation inside forms or
resource APIs. Unsupported host reads must remain visibly non-reactive rather
than being grandfathered in as convenience behavior.

### Acceptance Evidence

This milestone is now closed. The admitted first-family lane ships as part of
the wasm baseline, and later milestones should treat host capability as an
existing product dependency rather than future exploratory work.

## Milestone 2: Composition API And Graph Publication

Engineering spec: [composition-api-plan.md](./composition-api-plan.md)

### Goal

Make controller-first signal authoring and explicit graph publication a real
package product surface so application code can compose feature controllers as
ordinary functions and publish outputs through `signals.graph(...)` without
falling back to graph-object registries or string-id wiring.

### Must Ship

- exported composition vocabulary such as `SignalNamespace`
- real `signals.graph(...)` API
- publication from typed readable handles
- deterministic output synthesis and graph publication artifacts
- diagnostics/history/compatibility alignment for published graphs
- docs and tests that teach controller-first composition as a real product path

### Must Preserve

- runtime truth remains runtime-owned
- graph publication remains an explicit public boundary
- compatibility/import/export graph-object lanes remain available but secondary
- forms/resources consume this composition surface instead of inventing their
  own feature-level graph model

### Explicit Boundary

Milestone 2 includes controller-first authoring and graph publication for app
code.

Milestone 2 does not include full forms behavior, resource lifecycle products,
or a second local graph engine living in package glue.

### Acceptance Evidence

This milestone is complete only when the package can prove:

- one flat runtime script and one controller-composed graph publish the same
  committed output truth
- `signals.graph(...)` is real, typed, and diagnostics-visible
- publication from computed handles is deterministic and same-runtime honest
- compatibility/export lanes stay aligned with the controller-first lane

This milestone is now closed. Controller-first composition and explicit graph
publication are part of the shipped wasm package surface, and later milestones
should build on them instead of treating them as future substrate work.

## Milestone 3: Scoped Controller Identity And Graph-Owned Lifecycle

Engineering spec:
[controller_scope_and_graph_lifecycle_plan.md](./controller_scope_and_graph_lifecycle_plan.md)

### Goal

Make controller-authored graphs safe for repeated real-world composition by
adding controller scope, graph-owned lifecycle, explicit public graph
contracts, graph-native operations, richer controller contract structure,
contract-level diagnostics/history/export truth, and graph-native historical
boundary artifacts before forms and API resources are allowed to build on the
current composition API.

### Must Ship

- standardized controller contract shape with explicit public and internal
  categories
- scoped controller/graph identity model
- graph-owned construction boundary instead of runtime-global id folklore
- explicit public graph input and output contract surfaces
- graph-native operational surface for public inputs and graph transactions
- collision-safe multi-instance controller composition
- repeated and dynamic instance identity story
- more unified `input` / `computed` / `output` authoring grammar
- contract-level diagnostics and dependency introspection
- graph-native export/import and historical boundary truth
- docs and certification that teach the significant-code path honestly

### Must Preserve

- runtime truth remains runtime-owned
- controller composition remains ordinary code rather than a second local graph
  engine
- forms/resources consume this ownership model instead of inventing their own
- the milestone may land before generic aspect-capacity work because it
  changes ownership and naming, not aspect breadth semantics

### Explicit Boundary

Milestone 3 includes scoped authoring, graph-owned lifecycle, public input and
output graph contracts, controller composition that survives repeated feature
instances, graph-native operations for public contracts, richer controller
contract structure, and graph-native historical/export truth.

Milestone 3 does not include full forms behavior, async resource product
semantics, or the generic aspect-capacity rewrite itself.

### Acceptance Evidence

This milestone is complete only when the package can prove:

- repeated instances of the same controller family do not collide
- graph-owned authoring/publication produces the same committed truth as
  equivalent flat or manually scoped runtime scripts
- public graph inputs and outputs are explicit, typed, and diagnostics-visible
- public graph inputs are operationally first-class at the graph boundary
- controller contracts preserve internal/public boundaries intentionally
- graph-native export/import and restore surfaces preserve public contract truth
- forms/resources no longer need to invent their own scope or lifecycle model

## Milestone 4: Opaque Identity And Ergonomic Authoring

Engineering spec:
[opaque_identity_and_ergonomic_authoring_plan.md](./opaque_identity_and_ergonomic_authoring_plan.md)

### Goal

Keep the current runtime, graph, controller, diagnostics, and restore
capabilities intact while making the main app-authoring lane materially easier,
especially for CRUD-scale application code.

### Must Ship

- runtime-owned opaque internal signal identity on the main lane
- optional debug names that are explicitly non-authoritative
- debug names restricted to readability and diagnostics search rather than globally
  addressable identity
- id-less `input`, `computed`, and `output` authoring on the main lane
- removal of authored string-id requirements from the normal app lane
- lighter controller and graph authoring on top of the current substrate
- lighter mutation ergonomics that still lower to the same canonical mutation
  envelope
- required and optional public input contract forms at graph boundaries
- a linked writable derived-state primitive for dependent writable state
- diagnostics/export/import parity between ergonomic and explicit authoring

### Must Preserve

- graph/public/export naming truth remains explicit
- portable/spec lanes remain explicitly named where portability requires it
- controller and graph ownership boundaries remain real
- we do not preserve the current main-lane authored-id ergonomics merely for
  backward compatibility if they obstruct the cleaner architecture
- this milestone must not become the forms product under another name

### Explicit Boundary

Milestone 4 includes opaque internal identity, lighter authoring, lighter
mutation ergonomics, stronger public input contracts, linked writable derived
state on top of the current graph/controller/runtime substrate.

Milestone 4 does not include a docs-journey overhaul as its primary scope, and
it does not include the full forms product.

### Acceptance Evidence

This milestone is complete only when the package can prove:

 - direct opaque authoring, controller-composed opaque authoring, and
  graph-published opaque authoring converge to the same committed truth
- public graph contracts remain explicit and export/import-honest
- duplicated debug names do not become identity collisions
- debug names never become the globally queryable string for authored app-lane
  signals
- required and optional public input contracts remain distinct and honest
- linked writable derived state remains a consumer of runtime truth instead of
  becoming a second source of authority

Why it belongs here:

- it comes after graph-owned lifecycle because that ownership model is the
  substrate it simplifies
- it comes before forms/resources because those later surfaces should inherit a
  better ergonomic foundation rather than compensating for current ceremony

## Milestone 5: Forms Product Surface

Engineering spec:
[forms_product_surface_plan.md](./forms_product_surface_plan.md)

### Goal

Build a first-class forms surface on top of callback-computed, observation,
async, and host-capability truth.

### Must Ship

- source, draft, effective, dirty, readiness, and submission vocabulary that
  feels native in app code
- validation and submission state derived through runtime-owned signals and
  async nodes
- rollback-safe observation and diagnostics for form activity
- explicit host-capability integration where browser-local facts matter
- package-facing types and examples that teach one obvious forms story

### Must Preserve

- forms do not become a second local store
- submit lifecycle consumes async runtime truth rather than inventing local
  pending/success/error grammar
- validation and readiness remain diagnosable runtime-derived state
- the forms surface remains a consumer of callback, async, and host-capability
  truth rather than redefining any of them

### Explicit Boundary

Milestone 5 includes humane forms authoring, validation, readiness, draft
state, and submission lifecycle on top of the completed runtime substrate.

Milestone 5 does not include treating local component state as the authority
for form lifecycle, inventing a separate async submission state machine, or
letting browser-local conditions bypass the typed host-capability lane.

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- forms are built as real runtime consumers rather than local UI sugar
- async submission truth, validation truth, and host-derived form facts do not
  split into separate semantic engines
- diagnostics explain form readiness, validation, submission, and rollback
  coherently
- the package examples teach a forms story that does not require tribal
  knowledge

## Milestone 6: API Surface

Engineering spec:
[api_surface_plan.md](./api_surface_plan.md)

Formal closeout:
[api_surface_closeout.md](./api_surface_closeout.md)

### Goal

Build the API surface as a consumer of the completed callback, temporal, async,
policy, graph, and host-capability semantics while preserving a clean
architecture for later external read and delivery systems.

We are not just replacing TanStack Query.
We are replacing a bunch of the frontend API integration layer too.

### Must Ship

- a resource family and line authoring surface that feels first-class in
  TypeScript
- typed parameter normalization and stable family-member identity
- one canonical line facade with first-class line-scoped derived views
- explicit freshness, retry, timeout, supersession, and revalidation semantics
  backed by runtime truth
- named continuity/freshness/retry policy postures backed by runtime policy
  truth
- output continuity and visibility behavior backed by runtime policy truth
- item-/aspect-aware partial patch reconciliation where the graph/runtime can
  prove narrower scope honestly
- resource reconciliation APIs that stay distinct from mutation/optimistic write
  intent
- diagnostics, replay, and restore truth for resource-backed application state
- ergonomics strong enough to replace query-library-shaped usage without
  inventing a second async truth model
- a compatibility boundary for later external read-definition and delivery
  systems
- request/auth/header/context posture strong enough to simplify common frontend
  API integration instead of leaving it as ambient glue
- typed upload posture for direct multipart and signed-upload flows that lowers
  through one coherent API model

### Must Preserve

- resource identity, freshness, retry, timeout, cancellation, and supersession
  remain runtime-owned semantics
- forms and resources share substrate truth rather than carrying separate async
  worlds
- later external systems must not split local lifecycle, freshness, or
  diagnostics truth into a second client engine
- host-derived revalidation facts flow through host capability rather than ad
  hoc browser checks inside resource callbacks
- resource ergonomics do not hide broad scans, broad invalidation, or broad
  refetch orchestration behind cheap-looking helper calls

### Explicit Boundary

Milestone 6 includes API-surface-grade resource authoring, request shaping,
freshness, retry/revalidation semantics, and diagnostics-rich resource state on
top of the completed substrate.

Milestone 6 does not include turning resources into the semantic owner of
external read definition, external delivery, host-derived facts, async policy,
or freshness meaning for the rest of the package. Resources must inherit those
semantics rather than define them.

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- API surface products consume the completed semantics above them
  rather than redefining them
- same parameters produce one stable family line under canonical normalization
- narrow item/aspect patch reconciliation remains honest instead of broad
  refetch by convenience
- host-derived revalidation and async lifecycle truth flow through named typed
  product lanes
- resource diagnostics explain freshness, retry, invalidation, and visibility-
  or host-driven revalidation honestly
- the initial signals-first surface preserves a clean convergence path to later
  externally-driven resource lines
- the API surface meaningfully reduces auth/header/request/callback/redirect
  boilerplate rather than only replacing query-cache usage
- the package can recommend the resource surface without a giant footnote about
  "real semantics living somewhere else"

This milestone is now closed. The resource/API surface is part of the shipped
wasm package product line, and later work should treat it as a completed
dependency rather than future exploratory work.

Follow-on note:

- exact resource-line restore is now a real shipped same-runtime path
- exact resource-line replay remains a typed unavailable surface until
  `worth-signal` exposes signal-exact replay execution at the history boundary

## Milestone 7: API Surface DX Hardening

Engineering spec:
[api_surface_dx_plan.md](./api_surface_dx_plan.md)

### Goal

Harden the closed API/resource surface into a much more pleasant developer
product without changing the already-certified resource semantics underneath it.

### Must Ship

- shared API defaults for common auth, headers, base URL, and inheritance
- nested API scopes for section-specific inherited request defaults
- declaration-site semantic types for common read and write intent
- one explicit `url(...)` declaration lane with path-param inference where
  honest
- `params(...)` request-parameter vocabulary
- signed upload, multipart upload, and deferred processing builder support
- custom-action and nonstandard endpoint support that stays inside the same
  grammar
- certified equivalence between `url(...)`-authored and raw-authored resource
  declarations
- roadmap follow-up for real signal-exact replay capability once the parent
  runtime exposes the necessary history operations

### Must Preserve

- the closed resource family and line semantics from Milestone 6
- explicit names and explicit route intent
- one canonical line model
- runtime-owned lifecycle, freshness, reconciliation, delivery, replay, and
  restore truth
- honest cost boundaries at the new builder surface

### Explicit Boundary

Milestone 7 includes API-surface ergonomics, shared request-default
inheritance, nested API scopes, declaration-site `url(...)` authoring, and
advanced-path builder support for the
already-closed resource line model.

Milestone 7 does not include inventing a new async engine, a magical resource
convention layer, a second request lifecycle, or router-owned API semantics.

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- shared defaults lower identically across multiple endpoint families
- scoped inherited defaults lower identically to explicit endpoint-local
  request posture
- `url(...)`-authored and raw-authored resources converge to the same family
  identity, line identity, lifecycle truth, and diagnostics/history truth
- conventional CRUD-shaped reads are materially less bureaucratic
- adversarial custom endpoints, uploads, and deferred-processing flows still
  fit the same grammar instead of forcing immediate escape to raw declarations

Why it belongs here:

- it comes after API-surface closeout because the semantic model had to be
  closed before a pleasant default lane could be wrapped around it honestly
- it comes before the router because route-local resources should consume a
  humane API lane rather than asking the router milestone to compensate for API
  declaration ceremony

## Milestone 8: Router And Navigation Projection Surface

Engineering spec:
[router_navigation_projection_plan.md](./router_navigation_projection_plan.md)

### Goal

Build a graph-native router and navigation surface on top of the completed
composition, graph lifecycle, host-capability, and resource/API substrate so
URL state, route matching, route-local prerequisites, navigation continuity,
and redirect outcomes stop living in a second frontend state machine.

The router should be a real package product, not a thin adapter around browser
history and not a framework-owned orchestration layer.

### Must Ship

- a typed URL and navigation vocabulary that treats browser location as
  explicit graph-owned state rather than ambient runtime context
- route-schema authoring with typed params, typed navigation builders, and
  explicit route identity
- route matching and route projection as runtime-consumed derived truth rather
  than framework-local state
- declaration-driven prerequisite and redirect posture for route admission so
  auth, permissions, tenant availability, and similar preconditions can deny,
  redirect, or supersede before route-local work is treated as admitted
- branch-native speculative navigation posture so candidate navigations can be
  evaluated, redirected, or rejected without polluting committed visible truth
- route-local continuity semantics that can consume resource continuity instead
  of inventing a second loading/resolver grammar
- typed browser-history integration for push, replace, popstate, and direct URL
  edits through one explicit router product boundary
- diagnostics, history, replay, and restore truth for route state and
  navigation outcomes
- app-facing ergonomics strong enough to replace framework-router-shaped usage
  without reintroducing URL, guard, resolver, and loader folklore as separate
  local engines

### Must Preserve

- URL, route projection, navigation admission, and redirect truth remain
  graph-consumed semantics rather than a second frontend orchestration engine
- route-local prerequisites consume existing resource, host-capability, and
  graph truth instead of redefining auth, permission, or lifecycle semantics
- resource continuity, freshness, redirect, and request posture remain owned by
  the completed API surface rather than being redefined inside the router
- browser history integration stays an explicit host boundary and does not turn
  ambient window reads or imperative history calls into hidden authority
- speculative navigation must not silently commit partially-admitted route truth
  or leave orphaned branch-local state behind after rejection or redirect
- tenant/workspace/project switching across deeply nested routes must preserve
  one coherent authority story instead of splitting URL, resource, and visible
  route truth

### Explicit Boundary

Milestone 8 includes URL state, route matching, typed navigation, route-local
prerequisite/redirect posture, branch-native speculative navigation, browser
history integration, and diagnostics-rich route state as a first-class wasm
product lane.

Milestone 8 does not include making the router the semantic owner of resource
identity, async lifecycle, form draft state, browser capability semantics, or
mutation/workflow orchestration. The router must consume those truths rather
than redefine them.

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- URL source changes, typed navigation calls, browser-history events, and
  branch-native speculative navigation converge on one canonical route truth
- equivalent navigations through direct URL edits, programmatic navigation, and
  browser back/forward reconstruct the same visible route, params, redirects,
  and diagnostics/history artifacts
- route-local prerequisite denial and redirect posture happen before route-local
  resource work is treated as admitted
- resource-backed route transitions consume existing continuity and branch/restore
  semantics instead of inventing separate guard/loader/resolver state machines
- rejected or redirected speculative navigations leave no orphaned route-local,
  tenant-local, or resource-local truth behind
- deeply nested tenant/workspace/project switches either converge to the
  nearest valid route truth or emit explicit denial/redirect artifacts instead
  of producing partial cross-tenant state
- the package can recommend the router surface without a giant footnote saying
  that "real navigation semantics still live in framework glue"

Why it belongs here:

- it comes after the API surface because route-local prerequisites,
  redirects, continuity, and speculative navigation need the completed
  resource/request/history substrate instead of re-inventing it inside router
  glue
- it comes after composition and graph-owned lifecycle because route state and
  route-local feature state need scoped controller identity, explicit graph
  publication, and branch/restore truth to stay coherent under speculative
  navigation
- it remains a separate milestone from forms because forms should consume the
  resulting route/navigation substrate where needed, not become the place where
  route semantics are first solved

## Milestone 9: Worker-First Runtime Placement And Main-Thread Host Bridge

Engineering spec:
[worker_runtime_placement_plan.md](./worker_runtime_placement_plan.md)

### Goal

Make dedicated web-worker deployment the preferred execution posture for
`worth-signals-wasm` so most invalidation, recomputation, async/resource
lifecycle, routing/resource/forms continuity, and diagnostics/history work
leave the UI thread, while browser-only host facts and host-side effects remain
explicit main-thread boundaries.

This milestone exists because the package is no longer just choosing a nicer
API shape. It now has enough real product surface that leaving the bulk of
runtime work on the main thread would turn correctness-success into UI-freeze
failure under load.

### Must Ship

- one worker-owned runtime posture for graph state, invalidation,
  recomputation, async/resource lifecycle, route/resource/forms continuity,
  history, replay/restore coordination, and diagnostics production
- one typed main-thread/worker bridge for:
  - committed transaction submission
  - typed host-capability delivery into the runtime
  - committed output and observation delivery back to the UI thread
  - host-effect requests that must execute on the main thread
  - diagnostics/history/export/import requests
  - disposal, detach, and capability-lifecycle updates
- one explicit worker-admissibility taxonomy for authored work, so the package
  can distinguish:
  - worker-executable runtime work
  - main-thread-only host work
  - typed unavailable or denied work
- an honest lowering path for the dominant app lane so ordinary computed,
  resource, router, and graph work can execute in the worker without pretending
  that live JavaScript closures are portable runtime data
- bounded batching/coalescing rules for host-capability updates, transaction
  submission, output delivery, and diagnostics reads so the bridge does not
  become a hidden per-node chatter channel
- counters and certification surfaces that expose:
  - main-thread bridge breadth
  - worker evaluation breadth
  - host-capability delivery breadth
  - output delivery breadth
  - worker/main-thread round-trip counts
  - typed fallback or denial counts
- docs and examples that teach worker-first as the recommended heavy-app
  deployment posture and main-thread execution as an explicit compatibility or
  host-boundary posture

### Must Preserve

- runtime truth remains singular; worker placement must not create a second
  cache, store, scheduler, or lifecycle authority on the main thread
- browser-only host facts remain typed host-capability inputs rather than
  ambient worker access to DOM/window state
- host effects that mutate DOM, browser APIs, framework state, or imperative
  platform objects remain explicit main-thread work
- live callback closures remain process-local host capabilities; the milestone
  must not lie that arbitrary authored closures can be migrated into a worker
- replay, restore, diagnostics, and compatibility artifacts must remain honest
  about whether work was worker-executable, main-thread-hosted, or unavailable

### Explicit Boundary

Milestone 9 includes moving the runtime-owned work of invalidation, planning,
recomputation, async/resource lifecycle, route/resource/forms continuity, and
diagnostics/history production behind a worker-owned execution boundary when
the authored graph admits that lowering honestly.

Milestone 9 does not include:

- granting workers ambient access to DOM, `window`, or framework-owned objects
- pretending ordinary main-thread closure capture is portable worker data
- silently pinning an entire application graph to the main thread because one
  node or effect was worker-ineligible
- redefining host capability, router, resource, or form semantics just to make
  the worker boundary convenient

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- the same semantically equivalent graph converges to the same committed truth,
  lifecycle truth, and diagnostics/history truth in both:
  - main-thread compatibility mode
  - worker-first deployment mode
- recompute storms, invalidation bursts, route churn, and resource-refresh
  pressure leave the main thread responsible only for host-boundary and public
  delivery work rather than full internal graph breadth
- browser-history, viewport, visibility, online/offline, timers, persistence,
  and similar admitted host-capability families remain typed main-thread lanes
  rather than being reintroduced as ambient reads from worker code
- worker-ineligible callbacks, resources, or effects emit explicit typed
  fallback, denial, or unavailability artifacts instead of silently weakening
  the placement contract
- replay, restore, export/import, and branch histories preserve the worker/main
  thread capability story explicitly and never pretend that live worker-hosted
  or main-thread-hosted callbacks were portable when they were not
- named counters prove that main-thread bridge cost scales with changed host and
  public-delivery surface, not with total graph size or total dependency count

Why it belongs here:

- it comes after composition, graph-owned lifecycle, API/resource closeout, and
  router work because the worker split needs stable public graph boundaries and
  stable product semantics to move wholesale instead of asking each feature area
  to invent its own background engine
- it comes after host capability because main-thread-only browser facts must
  already exist as typed runtime inputs before worker execution can stay honest
- it belongs before roadmap completion because keeping most non-host work off
  the UI thread is a product boundary for serious web apps, not an optional
  post-roadmap optimization

### Worker Placement Follow-On: Product Entrypoint Correction

Engineering spec:
[worker_runtime_product_entrypoint_correction_plan.md](./worker_runtime_product_entrypoint_correction_plan.md)

Formal closeout ledger:
[worker_runtime_product_entrypoint_correction_closeout.md](./worker_runtime_product_entrypoint_correction_closeout.md)

### Goal

Correct the remaining product-entrypoint mismatch after Milestone 9 closeout so
the shipped package entry lane actually matches the certified worker-first
default posture.

### Why This Follow-On Exists

Milestone 9 closed the worker substrate, bridge taxonomy, compatibility proof,
and product-guidance certification. But the shipped package entry lane still
constructs the main-thread runtime by default and still teaches synchronous
`createSignals()` as the normal browser path.

That means the substrate is ahead of the product surface.

This follow-on closes that gap by making:

- `createSignals()` async and worker-first by contract
- `createSignals({ deployment: "mainThreadCompatibility" })` the explicit
  secondary construction lane
- worker unavailability a typed construction artifact instead of hidden fallback

### Must Ship

- async worker-first `createSignals()` package contract
- explicit main-thread compatibility deployment option
- typed worker-unavailable construction artifacts
- construction planning that lowers deployment posture before runtime authority
  is instantiated
- docs, README, examples, and package types that all teach the corrected
  entrypoint truth
- package-level certification proving that ordinary browser construction no
  longer defaults to main-thread runtime authority

### Must Preserve

- `createSignals()` remains the normal app-facing entrypoint name
- worker substrate and compatibility proofs from Milestone 9 remain the
  authority source rather than being reimplemented in package glue
- main-thread compatibility remains supported, but explicit and secondary
- no hidden fallback from worker-first construction into main-thread runtime

### Acceptance Evidence

This follow-on is complete only when:

- `await createSignals()` constructs worker-first runtime authority by default
- `createSignals()` emits typed worker-unavailable failure instead of silently
  constructing compatibility mode
- `createSignals({ deployment: "mainThreadCompatibility" })` remains available
  and semantically equivalent for supported compatibility workloads
- docs and README teach the corrected entrypoint story instead of the old
  main-thread default

### Closeout Status

This follow-on is now closed. The worker-first entry contract, explicit
compatibility recovery, typed worker-unavailable construction artifact, docs
alignment, and named entrypoint proof families all ship together, and the final
release-build aggregate runtime verification is green.

Why it belongs here:

- it comes immediately after Milestone 9 because it consumes the already-built
  worker substrate instead of introducing a new worker capability family
- it belongs before additional worker-facing or DX-facing product work because
  otherwise later features would inherit a dishonest default construction lane

## Milestone 10: Branch-Native Resource Effects And Response Lenses (Completed)

Engineering spec:
[resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md)

Predecessor feature closeout:
[resource_response_auto_patching_remaining.md](./resource_response_auto_patching_remaining.md)

### Goal

Make local patches, delivered patches, optimistic writes, server
confirmations, server failures, rollbacks, branch restores, rebases, and
advanced response topology automatic by lowering them into one branch-native
resource effect model.

This milestone exists because the closed collection response contract slice is
correct and useful, but the next product layer must not build an implicit
optimistic/cache authority inside resource helpers beside the existing
`worth-signal` branch, snapshot, restore, replay, aspect, and proof substrate.

Response lenses remain part of the milestone, but they are now topology
lowering strategies into resource effect loci rather than the foundational
authority model.

### Must Ship

- product exposure for native branch/merge policy dimensions resource effects
  need, including merge strategy, merge base, source-only policy, conflict
  isolation, identity matcher, deletion policy, and aspect policy bindings
- typed resource effect profiles inherited through API, scope, and route
  settings beside request posture, with preconfigured profiles for common
  branch-native, server-canonical, pessimistic, delivery-authoritative,
  non-reversible, and sensitive-data postures
- one canonical branch-native resource effect envelope for local patch,
  delivery, optimistic write, confirmation, failure, rollback, restore, merge,
  and rebase provenance
- effect identity, idempotency, server correlation, retry-lineage, and causal
  sequencing proof so duplicate packets, retries, confirmations, failures, and
  replay observations cannot be confused with distinct same-locus work
- speculative branch lifecycle as the default optimistic resource posture
- explicit resource line visible-branch selection so committed, speculative,
  confirmed, restored, or merged truth never lives in a package-local optimistic
  overlay
- typed optimistic lifecycle events for applied, committed, rolled back,
  denied, rebased, conflicted, superseded, and unavailable outcomes
- server confirmation canonicalization for exact confirmation, transformed
  canonical server truth, partial confirmation, failure, and drift-driven
  merge/rebase
- rollback through exact branch restore, inverse effect, or explicit
  optimistic-unavailable artifact
- inverse and preimage storage posture with named breadth, privacy, and cost
  evidence
- rebase through native branch merge planning plus resource-locus conflict
  explanation
- response-lens declaration and compiled-lens proof vocabulary that lowers
  topology into branch-native effect loci
- collection and paged parity with the already-shipped response contract slice
- JSON item aspect effects with hostile path denial, identity preservation,
  rollback or unavailability posture, and path cost proof
- advanced topology effect support for GraphQL connections, normalized entity
  bags, grouped collections, tuple-discriminated envelopes, sparse page chunks,
  map-backed collections, multiple named collections, recursive trees, detail
  responses, and summary responses
- diagnostics/history derived from the canonical effect envelope
- runtime-issued proof brands for branch summaries, lowered effect plans,
  compiled response lenses, and effect loci so object-shape forgery denies at
  admission

### Must Preserve

- the closed resource family and line model from the API surface milestone
- the shipped collection response contract behavior as a valid subset
- runtime-owned lifecycle, freshness, delivery, branch, replay, restore, and
  diagnostics truth
- native Signal branch lifecycle and derived-execution authority; resource code
  must consume it without claiming that Signal owns application values
- confirmed standalone resource values must have one explicit TypeScript local
  authority rather than coordinator-local or UI-local shadow truth
- response lenses as topology lowering rather than resource truth authority
- detail, collection, paged, summary, membership, entity-store, JSON aspect,
  item aspect, and broad-response distinctions
- broad replacement as an explicit branch effect with broad scope and broad cost
- UI policy separation; toasts, banners, modals, logging, and analytics consume
  typed lifecycle events rather than executing inside the resource runtime

### Explicit Boundary

Milestone 10 includes branch-native resource effect envelopes, product-level
branch/merge exposure, speculative branch lifecycle, optimistic lifecycle
events, response-lens effect-locus lowering, JSON item aspect effects, advanced
response topology effects, rollback/rebase/conflict certification, and derived
patch or delivery helpers for admitted effect families.

Milestone 10 does not include network transport ownership, service-worker
synchronization, UI toast/banner/modal execution, arbitrary identity inference,
automatic topology inference without declaration, identity migration after
patch, mutation response lenses for create/update/remove, write-result-to-read
family reconciliation, granular detail field/path/region lenses, placement or
deletion topology for create/remove responses, or core branch/merge semantics
that belong in native `worth-signal` first.

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- product history exposes the native branch/merge controls resource effects
  require
- resource effect posture is typed, mostly preconfigured, and inherited through
  API/scope/route settings rather than embedded in response topology
- local patch, delivery patch, optimistic write, confirmation, failure,
  rollback, branch restore, merge, and rebase derive from one effect envelope
- optimistic execution is isolated on Signal branches by default while
  confirmed standalone resource values remain owned by the product's explicit
  local truth boundary
- resource line visible truth derives from explicit branch selection proof
- server confirmation can admit canonicalized server truth instead of merely
  keeping the speculative value
- duplicated, retried, confirmed, failed, and replayed observations converge
  through effect identity and idempotency proof
- response lenses lower declared topology into effect loci rather than running
  a second response patch engine
- rollback is exact branch restore, inverse effect, or explicit unavailable
  artifact
- rebase and conflict explanation preserve native branch execution evidence
  beside resource-locus value evidence without promoting derived proof into
  application truth authority
- direct arrays, object envelopes, custom collections, and paged responses
  preserve the behavior already closed by the current response contract slice
- JSON and advanced topology effects prove admitted local, delivery,
  optimistic, broad replacement, denial, branch restore, and merge/rebase
  posture where applicable
- each advanced topology family closes through its own proof lane rather than a
  single representative topology
- diagnostics and history distinguish item-local, JSON aspect-local,
  membership-local, entity-store-local, summary-local, detail-local,
  optimistic, rollback, rebase, conflict, and broad replacement scopes

### Closeout Status

Milestone 10 is closed. The implementation now ships the canonical
branch-native resource effect envelope, typed effect profiles, speculative
branch lifecycle, response-lens effect-locus lowering, JSON path aspect effects,
advanced response topology effect families, merge/rebase conflict
certification, closeout matrix evidence, executable feature docs, and public
type-surface proof.

Why it belongs here:

- it comes after the worker milestone because the worker closeout already
  exists as Milestone 9 and should not be renumbered retroactively
- it comes after API DX hardening and the response auto-patching closeout
  because those closed the common route/resource lane and exposed the need for
  branch-native resource effects rather than route-local optimistic caches
- it remains before roadmap completion because advanced response topology,
  delivery, forms submission, router continuity, and external integration must
  not normalize weaker JS-side resource effect machinery while native signal
  branches already exist

## Milestone 11 Foundation: Native Branch Execution Substrate (Retained)

Superseded design record:
[branch_merge_materialization_foundation_plan.md](./branch_merge_materialization_foundation_plan.md)

Current completion spec:
[branch_merge_resolution_and_materialization_plan.md](./branch_merge_resolution_and_materialization_plan.md)

Predecessor milestone:
[resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md)

Successor milestone:
[resource_mutation_response_reconciliation_plan.md](./resource_mutation_response_reconciliation_plan.md)

### Goal

Retain the native branch capabilities that later standalone truth work consumes:
explicit fork basis, scoped derived-execution requests, targeted transactions,
branch lifecycle, proof-bearing heads, invalidation, recomputation, and worker
translation.

The earlier design record incorrectly treated native merge proof as application
value truth. The completion spec supersedes that authority placement. Native
Signal remains reconstructible derived execution; TypeScript owns standalone
application-value truth and manual merge resolution.

### Must Ship

- explicit native branch fork basis and branch-head proof
- branch-targeted transactions with stale-head denial
- branch retirement and retained lifecycle evidence
- scoped derived-execution planning and proof where reconstructible Signal state
  benefits from it
- worker commands carrying native and authored execution basis together
- exact invalidation, recomputation, diagnostics, replay, and boundedness proof

### Must Preserve

- Signal branches remain derived execution state over host-owned values
- native receipts attest execution and lifecycle, never local application truth
- native merge remains available only for reconstructible Signal-owned state
- application-value history, aspect deltas, conflict policy, human choices, and
  custom values do not enter Rust Signal
- worker and compatibility deployments preserve the same derived semantics

### Explicit Boundary

This retained foundation includes native derived-branch lifecycle and execution
capabilities. It does not include standalone application truth, value history,
manual merge policy, local truth commits, custom resolutions, or UI merge work.

### Acceptance Evidence

This foundation remains valid only while tests prove explicit basis, stale-head
denial, targeted branch isolation, retirement without heavy residue, worker
parity, rebuild equivalence, and the inability of native artifacts to construct
TypeScript local truth authority.

Why it belongs here:

- it supplies derived execution primitives used by the concurrent effect DAG
  and the standalone local truth projection boundary
- it remains below value authority so the same Signal crate composes honestly
  inside the larger Query/Relational platform

## Milestone 11 Foundation Follow-On: Concurrent Resource Effect Branch DAG (Completed)

Engineering spec:
[concurrent_resource_effect_branch_dag_plan.md](./concurrent_resource_effect_branch_dag_plan.md)

Predecessor milestone:
[resource_response_lens_contracts_plan.md](./resource_response_lens_contracts_plan.md)

Successor milestone:
[branch_merge_resolution_and_materialization_plan.md](./branch_merge_resolution_and_materialization_plan.md)

### Goal

Replace shared-branch optimistic rollback with effect-owned native branches that
support independent siblings, explicit dependent children, arbitrary response
order, worker-first execution, resource-locus reconciliation, and a rebuildable
visible projection.

This follow-on exists because Milestone 10 claimed speculative branch lifecycle,
but the resource effect path can still select the current runtime branch and
capture a snapshot on it. Milestone 11 supplies explicit fork basis and merge
materialization foundations, but resource-line materialization and concurrent
effect ownership remain unresolved. Mutation-response reconciliation must not
inherit that gap.

### Must Ship

- native branch retirement with retained lineage and zero heavy-state residue
- atomic branch-targeted transaction execution with expected-head proof
- worker-first explicit fork, targeted transaction, and retirement commands
- live worker branch basis that includes authored graph generation rather than
  cached import-context snapshot posture
- one effect-owned native branch per admitted optimistic effect
- separate proof types for native execution ancestry and semantic effect
  dependency, including siblings, children, and multi-dependency effects
- resource-locus three-way reconciliation that never replaces a whole native
  value node for a narrower admitted locus
- one disposable native projection branch for visible canonical-plus-open-effect
  truth
- arbitrary-order success, failure, retry, duplicate, supersession, dependency
  cancellation, and proof-bearing rebase closeout
- targeted concurrent effect inspection and rollback APIs
- worker-first and explicit main-thread compatibility parity
- Worth Signals Demo 5 proof with ten concurrent requests, mixed sibling and
  dependent effects, five successes, five failures, and seeded random settlement
- feature documentation and migration guidance for effect branches, dependency
  DAGs, projection, targeted rollback, and history restore separation

### Must Preserve

- native `worth-signal` branch, transaction, derivation, and lifecycle authority
- one canonical resource effect envelope for speculative intent
- one explicit confirmed local resource truth authority outside native Signal
- response lenses and resource loci as declared topology lowering
- exact history restore as a separate capability rather than optimistic rollback
- runtime truth singularity across worker-first and compatibility deployments
- UI, forms, router, demo, and docs as consumers of runtime receipts rather than
  local optimistic truth engines

### Explicit Boundary

This completed follow-on includes native retirement and targeted branch transactions,
worker branch command parity, effect branch acquisition, semantic dependency
planning, resource-locus reconciliation, derived projection, concurrent
settlement, public inspection, Demo 5, docs, and a sealed hostile certification
bundle.

It does not include network transport ownership, arbitrary topology inference,
user-supplied merge functions, UI feedback execution, or treating branch
ancestry as semantic dependency. Unsupported loci deny optimistic admission
rather than silently adopting whole-line state.

### Acceptance Evidence

This follow-on is complete only when the wasm product surface can prove:

- ten or more effects converge under arbitrary success/failure response
  permutations with no cross-effect erase or resurrection
- independent siblings settle independently while dependent children wait,
  cancel, or rebase only through declared policy and proof
- disjoint loci stored in one native value node preserve every successful change
- same-locus conflicts converge by server revision or declared deterministic
  order, never response arrival order
- worker-first and compatibility modes produce equivalent truth, lifecycle,
  diagnostics, history, and boundedness digests
- projection state can be destroyed and rebuilt from canonical truth plus open
  effect envelopes
- every settled effect branch releases heavy state while retaining audit lineage
- Demo 5 contains no compensating rejection patch or React-side merge compositor
- documentation examples execute against the packaged public surface
- exact counters show settlement and reprojection scale with affected branch,
  dependency, and locus breadth rather than total graph or open-effect population

Why it belongs here:

- it consumes the retained explicit fork-basis and targeted-execution substrate
- it corrects the concurrency and worker-first authority gap in Milestone 10
  before more mutation families normalize that behavior
- it precedes manual merge completion because Demo 5's confirmed local state
  must move onto the explicit TypeScript truth boundary before Demo 6 adds
  human resolution

## Milestone 11 Completion: Standalone Local Truth Merge And Signal Projection

Engineering spec:
[branch_merge_resolution_and_materialization_plan.md](./branch_merge_resolution_and_materialization_plan.md)

Predecessor milestone:
[concurrent_resource_effect_branch_dag_plan.md](./concurrent_resource_effect_branch_dag_plan.md)

Successor milestone:
[resource_mutation_response_reconciliation_plan.md](./resource_mutation_response_reconciliation_plan.md)

### Goal

Add one explicit, in-memory TypeScript local truth authority for standalone
browser application values; support proof-bearing sibling, parent/child,
selected-aspect, repeated partial, and manually resolved merges there; and
project each canonical commit one-way into rebuildable native Signal execution.

This completion exists because Signal is both the standalone WASM derived
runtime and a lower layer in the larger Rust platform. Application value
history and manual resolution must not be placed in native Signal merely
because WASM does not embed Relational.

### Must Ship

- declared local truth schemas and top-level plain-object aspect materializers
- one canonical `LocalTruthCommit` pipeline with immutable snapshots, branch
  heads, checkpoints, bounded history, and per-locus integration lineage
- exact common-basis and delta extraction for sibling and parent/child branches
- automatic policy, reviewable conflicts, source/target choices, and custom
  values authored on local truth resolution branches
- sealed executable merge plans and atomic local truth merge publication
- one-way committed-truth-to-Signal targeted projection with exact aspect
  invalidation, derived stale posture, and deterministic rebuild
- worker-first and explicit main-thread compatibility parity
- Demo 5 migration off coordinator-local authority shadow
- complete gear-based Demo 6 manual merge certification
- platform boundary docs, public types, examples, hostile tests, and counters

### Must Preserve

- native Signal owns derived execution, not application values or value merge
  policy
- the larger Rust platform remains Query -> Relational -> Bridge -> Signal
- the TypeScript authority is process-local and does not claim relational
  durability, MVCC, or cross-process recovery
- UI renders runtime artifacts and submits issued choices; it never composes
  merged values
- committed truth survives Signal derivation failure and can rebuild every
  Signal branch and projection
- work remains bounded by visited ancestry and semantic delta

### Explicit Boundary

This completion includes standalone local value authority, aspect declarations,
branch history, manual merge review and resolution, atomic truth commit, Signal
projection, Demo 5 hardening, Demo 6, and documentation.

It does not include durable/shared persistence, platform Query mutation
workflow, Relational merge execution, nested or collection merge inference,
identity migration, arbitrary merge callbacks, network transport, or UI-owned
truth.

### Acceptance Evidence

This completion is closed only when the package proves:

- no Rust type or journal owns local application-value history or manual choices
- every local mutation and merge emits one canonical commit or no mutation
- repeated partial merges retain exact per-locus source integration lineage
- stale bases, reviews, policies, schemas, and resolution branches deny before
  commit
- all native Signal and UI state can be destroyed and rebuilt from local truth
- derived projection failure cannot roll back or corrupt committed truth
- worker-first and compatibility modes converge on identical semantic digests
- Demo 5 retains arbitrary-order concurrent effect convergence
- Demo 6 contains no React/Three.js object merge compositor
- exact counters expose ancestry, delta, conflict, publication, invalidation,
  recomputation, rebuild, and bridge breadth

Why it belongs here:

- it follows the concurrent effect DAG because that work supplies isolated
  branches, targeted transactions, retirement, and rebuildable projections
- it precedes mutation-response reconciliation because create/update/remove and
  identity work need an honest local truth commit before updating read families
- it closes the WASM-only authority gap without contaminating native Signal or
  creating a speculative Relational dependency

## Milestone 12: Resource Mutation Response Reconciliation And Detail Lenses

Engineering spec:
[resource_mutation_response_reconciliation_plan.md](./resource_mutation_response_reconciliation_plan.md)

Predecessor milestone:
[branch_merge_resolution_and_materialization_plan.md](./branch_merge_resolution_and_materialization_plan.md)

### Goal

Finish the resource response product surface so mutation responses, detail
resources, canonical server confirmations, creates, updates, removes, identity
migration, and multi-family reconciliation are as explicit and ergonomic as the
collection response lane.

This milestone exists because Milestone 10 correctly closed the branch-native
effect substrate and advanced response topology lowering, and Milestone 11
completion closes the standalone local truth and Signal projection boundary.
The remaining
consumer-facing asymmetry is still real: collections have strong ergonomic
topology, while writes and detail resources still ask application code to
compose lower-level primitives.

### Must Ship

- response-owned `.create(...)`, `.update(...)`, and `.remove(...)` lanes after
  `.response(...)`
- mutation response lenses distinct from read response lenses when write
  payloads contain canonical values plus metadata, warnings, validation, or
  delivery hints
- one canonical mutation-response reconciliation plan before read-line mutation
- explicit write-result-to-read-family reconciliation for detail, collection,
  paged, summary, and auxiliary read families
- detail field, detail JSON path, detail region, and whole-detail response
  effect loci
- save/update response reconciliation that can replace or patch canonical
  detail truth without feature-local commit logic
- create response placement for collection topologies or typed
  placement-unavailable/refetch/delivery-awaited posture
- remove response deletion, tombstone, detail invalidation, summary update, or
  typed deletion-unavailable/refetch/delivery-awaited posture
- temporary/client/draft/import id to canonical server id migration through
  proof-bearing branch-native effects
- partial mutation response mapping for server fragments such as version,
  updated-at, warnings, validation, operation status, and delivery hints
- multi-family mutation reconciliation with per-target exact, partial,
  invalidated, refetch-required, delivery-awaited, or declined outcomes
- rollback, replay, branch restore, diagnostics, history, and merge/rebase
  proof derived from mutation-response plans
- cost counters for response extraction, target fanout, detail traversal,
  topology lookup, reconstruction, identity migration, placement, deletion,
  and fallback breadth

### Must Preserve

- Milestone 10's canonical speculative resource effect envelope
- Milestone 11's explicit standalone local truth commit authority
- response lenses as topology lowering rather than resource truth authority
- read-family ownership of declared line behavior, lifecycle, freshness,
  diagnostics, delivery basis, branch posture, and projections over local truth
- the distinction between read response topology and mutation payload topology
- whole-response detail replacement as a legal broad effect, while no longer
  treating it as the full detail story
- explicit declaration over automatic topology or identity inference
- typed unavailability as honest fallback, not as ergonomic completion
- UI policy separation and network transport separation

### Explicit Boundary

Milestone 12 includes mutation response reconciliation, granular detail lenses,
create placement, remove deletion, identity migration, partial response
mapping, multi-family target convergence, and explicit refetch/delivery
fallback posture.

Milestone 12 does not include network transport ownership, service-worker
synchronization, UI toast/banner/modal execution, arbitrary topology
inference, arbitrary item identity inference, framework-specific cache
integration, or native branch/merge semantics that belong in core
`worth-signal` first.

### Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- write response lenses exist for create, update, and remove routes
- mutation responses lower into one canonical reconciliation plan
- write responses can reconcile read-family truth without feature-local cache
  or normalization glue
- detail resources support field, JSON path, region, and whole-response loci
- updates can reconcile detail, collection, paged, and summary truth
- creates can insert into declared collection topologies or emit typed
  placement fallback
- removes can delete, tombstone, invalidate detail truth, or emit typed
  deletion fallback
- identity migration updates every declared target it claims to update or emits
  typed partial/unavailable posture
- partial mutation responses update only declared canonical fragments
- multi-family mutations leave no declared target implicitly stale
- rollback, replay, branch restore, diagnostics, history, and merge/rebase
  explain every reconciled target from the mutation-response plan
- docs and closeout matrices distinguish admitted ergonomic happy paths from
  denial-only support and typed unavailable fallback

Why it belongs here:

- it comes after Milestone 11 completion because it depends on one canonical
  local truth commit, manual/automatic merge decisions, Signal projection,
  branch-native effects, response-lens proof, and rollback evidence
- it closes the product asymmetry exposed by workflow/editor writes: collection
  topology is elegant, but details, creates, updates, removes, and canonical
  write-result reconciliation still need first-class product treatment
- it belongs before roadmap completion because serious forms, workflow
  editors, router continuity, and external integration should consume this
  write/detail reconciliation surface rather than hand-normalizing mutation
  results in feature code

## Roadmap Done When

This roadmap is complete only when:

- async-node core truth is present and inherited honestly
- host capability exists as a typed wasm/product lane
- forms are built as real runtime consumers rather than local UI sugar
- API surface work consumes the completed semantics above it
- API surface ergonomics are hardened enough that common app authoring does not
  require substrate-shaped ceremony
- branch-native resource effects let local patch, delivery, optimistic write,
  rollback, rebase, and advanced response topology consume explicit local truth
  plus derived Signal branch execution without manual route patch plumbing or
  an implicit optimistic cache
- concurrent resource effects own isolated native branches, represent sibling
  and dependent work explicitly, settle in arbitrary order, and derive one
  rebuildable visible projection without shared snapshot rollback
- standalone branch merge resolution lives in one TypeScript local truth
  authority, records exact aspect lineage and manual decisions, then projects
  committed truth into rebuildable Signal state instead of native or UI merge
  folklore
- mutation response reconciliation lets create, update, remove, canonical
  server responses, granular detail effects, identity migration, placement,
  deletion, partial responses, and multi-family target updates converge without
  feature-local cache or normalization glue
- route and navigation products consume URL, browser-history, branch, and
  resource continuity truth without creating a second state machine
- worker-first deployment keeps most runtime work off the UI thread while
  preserving explicit main-thread host boundaries and one canonical runtime
  truth
- no milestone creates a second reactive or async engine beside Signal, and no
  convenience surface creates a second application-value authority beside the
  explicit standalone local truth layer or the platform's Relational authority
