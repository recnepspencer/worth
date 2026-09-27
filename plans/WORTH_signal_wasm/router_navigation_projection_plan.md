# worth-signals-wasm Router And Navigation Projection Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Web runtime parent:** [web_runtime_spec.md](./web_runtime_spec.md)
>
> **Composition prerequisites:**
>
> - [composition-api-plan.md](./composition-api-plan.md)
> - [controller_scope_and_graph_lifecycle_plan.md](./controller_scope_and_graph_lifecycle_plan.md)
> - [opaque_identity_and_ergonomic_authoring_plan.md](./opaque_identity_and_ergonomic_authoring_plan.md)
>
> **API-surface prerequisite:**
>
> - [api_surface_plan.md](./api_surface_plan.md)
> - [api_surface_closeout.md](./api_surface_closeout.md)
>
> **Worker-first runtime prerequisite:**
>
> - [worker_runtime_placement_closeout.md](./worker_runtime_placement_closeout.md)
> - [worker_runtime_product_entrypoint_correction_closeout.md](./worker_runtime_product_entrypoint_correction_closeout.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../../../plans/worth_signal/worth_signal_vision.md)
>
> **Core test requirements:** [plans/worth_signal/test-requirements.md](../../../plans/worth_signal/test-requirements.md)
>
> **Certification spec:** [router_test_requirements.md](./router_test_requirements.md)

## Goal

Build a graph-native router and navigation surface in `worth-signals-wasm` that
can replace framework-router-shaped usage without inventing a second
navigation, guard, loader, or browser-history truth engine, and without
pretending worker-first runtime truth is still a future deployment detail.

The target product shape is:

- URL state is graph-owned input, not ambient window folklore
- route matching is derived truth, not framework-local orchestration state
- route params, search params, hash fragments, and canonical URL equivalence
  are typed and normalized
- route-local prerequisites, redirects, and not-found/forbidden/unavailable
  outcomes are explicit typed route results
- browser-history ingress is a typed host-to-worker boundary by default rather
  than ambient main-thread context
- speculative branch navigation can admit, redirect, or discard candidate
  navigations before visible truth flickers
- route-local resources consume the completed resource/API surface rather than
  recreating loaders, resolvers, and cache folklore
- route-local forms consume explicit route authority and route-side
  preserve/discard policy rather than inventing controller-local route state
- nested layouts and outlets are graph-projected route composition, not
  framework glue
- reversible navigation can restore admitted route/outlet composition and the
  route-scoped graph truth the snapshot boundary honestly owns
- diagnostics, history, replay, restore, and breadcrumb/back semantics are
  real product surfaces rather than debug-only implementation accidents

This milestone is not complete if it only wraps `window.history` and path
matching behind a prettier TypeScript facade.

## Why This Milestone Exists

The completed wasm product line now has enough substrate to support a much
better router than the standard frontend stack:

- `createSignals()` is already the app-first product surface
- `createSignals()` is now explicitly async and worker-first as the canonical
  package front door
- controller-first composition and graph publication are real
- graph-owned lifecycle, branch, replay, and restore are real
- host capability exists for explicit browser-derived facts
- resource lines already own request posture, continuity, diagnostics,
  delivery, and branch/restore truth
- forms already expose route-coupled behavior as typed deferred posture rather
  than faking route authority locally

Without a router milestone, application code still faces the old split:

- one engine owns URL and navigation state
- another engine owns resources and loaders
- another layer owns auth/permission guards
- the UI framework owns outlet composition and route-local continuity
- browser history sits beside them as a loosely synchronized imperative system

That split recreates exactly the kind of convenience-era drift this roadmap has
been removing:

- route guards become imperative middleware instead of declared graph truth
- route loaders become a second resource lifecycle story
- tenant and workspace switching produce orphaned local state
- breadcrumbs and back buttons restore strings instead of admitted route truth
- search-param-heavy enterprise screens fall back to ad hoc string plumbing
- diagnostics explain resources and graph state, but not the navigation story

This milestone exists to prevent that collapse and make routing a consumer of
the completed graph/resource substrate rather than a rival state machine.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is starting from the hostile
  navigation failure mode instead of from path-matching ergonomics. The spec
  therefore starts from deep tenant switching, speculative admission, stale
  deep links, and restore parity rather than from "make `navigate(...)` nice."
- `arch_laws.md`
  The most important laws here are 2, 7, 20, 24, 33, 34, 40, and 41. Route
  prerequisites must declare what they consume, browser-history boundaries must
  surface explicit envelopes, rejection must happen before route-local work is
  treated as admitted, and route, outlet, breadcrumb, redirect, and restore
  meanings must remain distinct proof-bearing categories.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. Cheap-looking route
  transitions must not hide broad route-tree rescans, broad loader/cache work,
  repeated URL reparsing, or rich-history reconstruction on the hot path.
- `dx_laws.md`
  The most important thing it protects is organized truth at the call site.
  The common router lane should read like route and navigation intent, the
  advanced lane should expose typed plan/policy surfaces, string paths should
  stop at declaration boundaries, and expensive navigation/resource/restore
  work must look expensive rather than masquerading as cheap reads.
- `domain_laws.md`
  The most important thing it protects is responsibility shape. URL authority,
  route schema, admission, browser-history integration, outlet composition,
  reversible navigation, and diagnostics must have distinct homes instead of
  collapsing into one giant router helper module.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  execution substrate rather than truth storage. The router must consume graph
  truth and snapshots; it must not become a new authority store or a framework
  orchestration engine in disguise.
- `web_runtime_spec.md`
  The most important thing it protects is the framework-agnostic wasm product
  thesis. The router must be a real wasm product surface that React, Angular,
  Vue, and plain TypeScript can consume, not a React-first convenience layer.
- `wasm_product_roadmap.md`
  The most important thing it protects is sequencing. Router work belongs after
  the API surface because it must consume completed resource/history/branch
  truth instead of inventing loaders, guards, and continuity semantics ad hoc.
- `worker_runtime_placement_closeout.md`
  The most important thing it protects is worker-first authority honesty.
  Browser history, route projection, and navigation continuity must now be
  designed as typed host-to-worker lanes and worker-owned route truth rather
  than as main-thread-first helpers that might move later.
- `worker_runtime_product_entrypoint_correction_closeout.md`
  The most important thing it protects is package-front-door honesty. The
  router must read as a normal async worker-first product lane with explicit
  compatibility posture, not as a main-thread router that merely tolerates
  worker deployment.
- `api_surface_plan.md`
  The most important thing it protects is that request posture, continuity,
  redirects, diagnostics, replay, and restore are already solved at the
  resource line boundary. The router must consume those capabilities rather
  than reinterpret them as route-local middleware.
- `api_surface_closeout.md`
  The most important thing it protects is that the resource line model is now
  closed and trustworthy. Route-local resource transitions, prefetch, and
  stale-deep-link recovery should lower through that line model rather than
  through a second client cache.
- `composition-api-plan.md`
  The most important thing it protects is controller-first graph publication.
  Route-local feature graphs, layouts, and outlets must compose through
  published graph handles instead of inventing a private router composition
  model.
- `host_callback_computed_spec.md`
  The most important thing it protects is callback-first app authoring without
  a second reactive engine. Router projections, prerequisites, and route-local
  controllers must remain callback-authored consumers of runtime truth.
- `forms_product_surface_plan.md`
  The most important thing it protects is the forms/router authority split.
  Route-coupled step behavior and route-scoped preserve/discard policy should
  be closed here as route authority while forms remains the owner of draft,
  readiness, and action semantics.
- `plans/worth_signal/test-requirements.md`
  The most important thing it protects is certification rigor. The router must
  close through hostile branch/replay/restore/navigation proof, not through
  nominal path examples.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A long-lived web application with nested layouts, multiple outlets,
> parameterized routes, typed search state, resource-backed route transitions,
> auth and policy prerequisites, speculative navigations, branch restore,
> reversible back/breadcrumb flows, tenant or workspace switching, stale deep
> links, and direct browser-history edits must converge to the same admitted
> route truth, the same visible outlet composition, the same route-local
> resource continuity truth, and the same diagnostics/history artifacts
> regardless of whether the navigation was driven by typed app navigation,
> direct URL mutation, browser back/forward, speculative branch evaluation,
> route-local redirect, restore, or replay.

If semantically equivalent navigation histories can produce:

- different admitted route identity for the same canonical URL truth
- different visible route or outlet composition after equivalent redirects
- partial cross-tenant state after deep workspace/project switching
- different breadcrumb/back outcomes depending on whether history was browser-
  driven or graph-driven
- loader/resolver/guard semantics that disagree with resource lifecycle truth
- or a second navigation/cache authority beside the runtime and browser-history
  boundary

then the milestone has failed.

## Product Decision Lock

- the router is a first-class wasm product surface, not framework glue
- the router must obey the DX laws:
  - the common lane reads like route and navigation intent rather than browser
    plumbing
  - the advanced lane exposes the next lower typed plan/policy surface instead
    of hiding it
  - raw strings are acceptable at route declaration boundaries, but reusable
    route references and navigation should become typed artifacts after that

Common lane example:

```ts
const routes = signals.router.define({
  home: signals.router.route("/"),
  userDetail: signals.router.route("/users/:userId", {
    search: {
      tab: signals.router.search.optional.string(),
    },
  }),
});

await signals.router.navigate(
  routes.userDetail.to({
    params: { userId: "u1" },
    search: { tab: "activity" },
  }),
);
```

Advanced lane example:

```ts
const navigationPlan = routes.userDetail
  .intent({
    params: { userId: "u1" },
    search: { tab: "activity" },
  })
  .policy({
    continuity: "preserve-visible-while-pending",
    projectionRefresh: "explicit",
    artifactPolicy: "diagnostics",
  })
  .compile();

navigationPlan.explain();
await signals.router.execute(navigationPlan);
```

Out of spec:

```ts
router.push(`/users/${userId}?tab=activity`);
window.history.pushState({}, "", `/users/${userId}`);
```
- URL truth is graph-owned and explicitly admitted through a browser-history
  boundary rather than being ambient runtime context
- worker-first deployment is the default product posture:
  - browser-history events, manual URL edits, and explicit navigation intents
    enter through typed host boundary envelopes
  - admitted route truth is worker-owned runtime truth unless the caller
    explicitly chooses compatibility deployment
- route matching, param extraction, search-param normalization, hash handling,
  and route projection are derived graph truth
- route schema and route identity are typed and canonicalized; ad hoc string
  path concatenation is not the primary public story
- route schema authoring should share one canonical route grammar with the
  route-first API lane where that grammar is semantically the same
- route maps should prefer declarative definition surfaces for structural truth,
  while navigation planning may use builder/progression surfaces only where
  ordered policy accumulation is the actual responsibility
- navigation actions are typed intent classes such as push, replace, soft
  refresh, same-route param mutation, and canonicalization redirect
- route candidate, projected match, and admitted outcome are distinct proof
  categories; the public surface must not pretend a matched route is already
  admitted
- route outcomes are typed and explicit:
  - admitted
  - redirect
  - notFound
  - forbidden
  - unavailable
  - denied
- route prerequisites are declared graph consumers, not imperative middleware
- access-policy composition may consume auth, tenant capability, feature flags,
  licensing, environment posture, maintenance posture, and regional posture
  through existing graph/resource/host-capability truth
- speculative branch navigation is a first-class product capability, not an
  optional optimization
- nearest-valid-truth recovery for stale deep links, deleted entities, and
  disabled modules is part of the product contract
- route-local continuity and prefetch posture must lower through completed
  resource line semantics rather than inventing a second loader cache or route
  pending grammar
- expensive router work must look expensive:
  - prefetch, resource-backed transitions, restore-backed navigation, and
    branch-native speculative evaluation should expose explicit intent/plan
    surfaces rather than feeling like ordinary property reads
- visible route projection freshness is explicit policy, not an accident:
  - worker-owned route truth may advance before a visible projection refresh is
    requested or admitted
  - continuity and pending-visibility policy must say when stale visible route
    truth is allowed, when projection must refresh immediately, and which
    diagnostics explain that choice
- nested layouts and outlets are route-scoped graph composition artifacts, not
  framework-owned slot glue
- reversible navigation is a first-class strength of the router:
  - back and breadcrumb return restore admitted route/outlet composition through
    graph restore/snapshot truth
  - the router must promise an explicit restore boundary rather than hand-wave
    "full app state"
- semantic draft preservation/discard across authority changes is part of the
  contract; the router owns route authority changes and route-side
  preserve/discard policy, while forms remains the owner of draft semantics
  once that route policy has been resolved
- canonical route equivalence is a real product law for duplicate URL forms,
  canonicalization redirects, analytics correctness, and redirect-loop
  prevention
- branch merge, merge-preview, and dirty-evaluation substrate should be used
  where they materially improve route transitions rather than being ignored in
  favor of browser-router-shaped heuristics
- URL writeback from graph state must happen only through explicit declared
  projection contracts
- SSR and hydration posture must be reserved honestly even if full SSR product
  work is later
- cross-tab and external navigation coherence must be stated explicitly at the
  authority boundary, even if the first implementation scope focuses on one tab

Normative consequence:

- any implementation that treats browser location as an ambient read instead of
  an admitted host boundary is out of spec
- any implementation that rebuilds guard/loader semantics as framework-local
  middleware or cache glue is out of spec
- any implementation that restores only raw URL strings while claiming route
  restore semantics is out of spec
- any implementation that treats search params and hash fragments as ad hoc
  string bags instead of typed route state is out of spec
- any implementation that preserves route-local drafts across tenant or entity
  authority changes without declared policy is out of spec
- any implementation that hides stale deep-link recovery or canonicalization as
  undocumented framework behavior is out of spec

## Architectural Model

### Ownership split

This milestone freezes the intended ownership boundary:

1. **`worth-signal`**
   - owns branch, replay, restore, snapshot, and diagnostics truth
   - owns derived evaluation and transaction semantics
2. **browser host boundary**
   - owns raw `location`, `history`, `popstate`, and direct URL mutation
   - does not own typed route meaning
3. **`worth-signals-wasm` router surface**
   - owns typed route schema authoring, URL normalization, route projection,
     navigation intent lowering, browser-history integration, and route-facing
     diagnostics/history surfaces
   - hosts route truth inside the same worker-owned graph/runtime substrate
     rather than a second router state machine
4. **resource and forms products**
   - remain the owners of resource lifecycle and draft/product semantics
   - may be consumed by route-local controllers and route transitions
   - must not be semantically redefined inside the router

The router is therefore not:

- a framework-owned orchestrator
- a browser-history wrapper with added type sugar
- a second loader cache
- a second guard engine
- a stringly breadcrumb generator

It is the local product layer that turns URL and navigation into graph-native
route truth.

### Worker-first authority model

The router should now be designed against the closed worker-first product
surface rather than against a hypothetical future worker migration.

That means:

- browser-owned `location`, `history`, and `popstate` are admitted through
  typed host boundary envelopes into worker-owned runtime truth
- the worker-owned route graph is the canonical route authority in the default
  deployment posture
- compatibility deployment is explicit and must preserve the same route truth
  semantics rather than a second router model
- router APIs must be honest about when visible route projection is reading
  cached worker truth versus freshly refreshed admitted route truth

### URL authority model

The router should organize URL truth into explicit typed categories rather than
one raw string:

1. **Raw location authority**
   - browser-provided pathname, search, hash, and navigation type
2. **Canonical URL projection**
   - normalized path segments
   - normalized search params
   - normalized hash posture
   - canonical equivalence digest
3. **Route match projection**
   - matched route identity
   - typed params
   - typed search state
   - typed fragment/hash state
4. **Admitted route truth**
   - route outcome after prerequisites, redirect posture, and nearest-valid
     recovery have been resolved

The normalization contract must not collapse to path strings too early. The
router needs typed canonical artifacts to justify:

- canonical redirect detection
- same-route param mutation
- duplicate URL equivalence
- cache and prefetch coherence
- replay and restore parity

Where application route schema and route-first API declarations share the same
route-pattern semantics, the package should prefer one canonical grammar and
one canonical normalization story rather than parallel route parsers that drift.

### Route composition model

Route matching alone is not enough. The router must project:

- route shell
- nested layout stack
- outlet composition
- route-local controller graph handles
- route capability boundary for child routes and outlet consumers

The intended product direction is:

- layouts are first-class route composition units
- outlets are explicit route-scoped projection slots
- route-local feature graphs publish through the existing composition API
- route capability handles prove which params, prerequisites, and route-local
  resources are admitted before downstream code can use them

This is where ordinary routers usually collapse back into framework-local
component trees. This milestone must not.

### Admission and prerequisite model

The router must treat route admission as a declared graph-evaluated boundary.

Prerequisite sources may include:

- auth/resource truth
- tenant capability
- feature flags
- licensing posture
- environment posture
- maintenance windows
- regional or deployment constraints
- route-local resource existence checks

Prerequisite resolution must structurally precede admitted route construction.

The route product should therefore lower through explicit categories such as:

- route intent
- validated route candidate
- prerequisite evaluation plan
- admitted route outcome
- redirect or denial artifact

This keeps route-local work, layouts, resources, and drafts from being treated
as admitted before the prerequisite story has actually been resolved.

The forms/router ownership split should be explicit:

- the router owns whether a route authority change preserves, freezes, discards,
  or defers route-scoped draft continuity
- forms owns draft structure, readiness, validation, action, and rollback
  semantics once the route-side policy has been declared
- route-coupled step behavior that currently emits typed deferred posture in
  forms must resolve through router authority here rather than through
  controller-local navigation conventions

### Navigation model

The router must distinguish navigation intent from navigation outcome.

Required intent classes:

- `push`
- `replace`
- `softRefresh`
- `sameRouteMutation`
- `canonicalize`
- `restoreBack`
- `breadcrumbReturn`

Required policy dimensions:

- speculative branch or direct commit
- branch merge, merge-preview, or discard posture when speculative route-local
  truth needs explicit reconciliation
- continuity posture while pending
- redirect-on-failure or hard-denial
- prefetch/warmup posture
- projection refresh posture for when worker truth may advance ahead of visible
  route projection
- URL writeback posture

This allows navigation semantics to stay explicit instead of hiding multiple
history and continuity behaviors behind one `navigate(...)` helper.

The DX consequence is:

- ordinary application code should be able to express route intent through
  typed route references and typed navigation helpers
- the lower-level route plan/policy surface should remain inspectable and
  executable for callers that need stronger control over cost, continuity,
  artifact policy, or deployment posture

Common-path shape:

```ts
const detailRoute = routes.userDetail.to({
  params: { userId: "u1" },
  search: { tab: "activity" },
});

await signals.router.navigate(detailRoute);
```

Inspectable plan shape:

```ts
const plan = detailRoute.plan({
  continuity: "preserve-visible-while-pending",
  projectionRefresh: "explicit",
  deployment: "workerFirst",
});

plan.cost();
plan.explain();
plan.projectionPolicy();
await signals.router.execute(plan);
```

### Reversible navigation model

The router must treat reversible navigation as a product capability built on
real snapshot and restore semantics.

That means:

- back and breadcrumb return are not just prior URL lookup
- route history entries may carry route-composition restore truth
- multi-outlet and nested-layout pages can restore the previously admitted
  route shell and outlet composition
- route restore boundaries are explicit about what is guaranteed:
  route truth, outlet composition, and graph-owned state within the admitted
  restore boundary

The router must not promise arbitrary non-graph local UI restoration by
accident. But it also must not under-sell the snapshot substrate by pretending
that route restore is only string navigation.

## Phases

### Phase 1: URL Authority, Canonicalization, And Route Schema Lock

Purpose:

- make URL truth and route-schema truth explicit typed product categories
- prevent the milestone from collapsing into string path helpers

This phase must ship:

- typed URL state vocabulary for pathname, search params, and hash fragments
- canonical normalization and route-equivalence contracts
- route-schema authoring with typed params and search state
- explicit route identity and navigation-intent vocabulary
- canonical shared route-pattern substrate for app route schema and route-first
  API authoring wherever their grammar is semantically the same
- declaration-first route definition surfaces for route truth; this phase must
  not default to browser-router-style string push helpers as the primary story
- compile-time boundaries that keep raw URL strings distinct from canonical
  route artifacts where the product contract requires it

Phase 1 gate:

- no later phase begins until canonical path/search/hash equivalence is frozen
  strongly enough that redirect loops, duplicate URLs, and route identity drift
  are structurally prevented

### Phase 2: Route Projection, Layouts, And Outlet Composition

Purpose:

- turn route matching into real route composition truth
- prevent layouts and outlets from falling back into framework glue

This phase must ship:

- route projection from canonical URL truth into matched route identity and
  projected route candidate truth
- nested layout and outlet composition vocabulary
- route-scoped controller/publication integration through the existing
  composition API
- typed route capability boundaries for admitted params, outlet contracts, and
  route-local APIs

Phase 2 gate:

- no later phase begins until one route can project one explicit layout/outlet
  composition story without requiring framework-local orchestration state, and
  projected route candidate truth remains visibly distinct from admitted route
  outcome truth

### Phase 3: Admission, Access Policy, And Nearest-Valid Recovery

Purpose:

- make prerequisites, denial, redirect, and stale-link recovery real product
  semantics instead of middleware folklore

This phase must ship:

- declaration-driven prerequisite posture
- layered access-policy composition over graph/resource/host-capability truth
- typed route outcomes for admitted, redirect, notFound, forbidden,
  unavailable, and denied paths
- explicit denial/redirect/not-found/unavailable artifacts
- nearest-valid-truth recovery for stale deep links, deleted entities, disabled
  modules, and tenant/workspace/project switching
- explicit route-to-forms authority handoff for preserve, freeze, discard, or
  defer posture across route authority changes
- typed diagnostics for why admission succeeded, denied, redirected, or fell
  back to a nearest valid route

Phase 3 gate:

- no later phase begins until route-local work is structurally impossible to
  treat as admitted before prerequisite resolution finishes

### Phase 4: Browser History Boundary And Route Truth Convergence

Purpose:

- make browser-history input, manual URL edits, and graph-owned route truth one
  coherent product boundary
- freeze the host-to-worker route ingress contract before richer transition
  policy layers depend on it

This phase must ship:

- typed browser-history integration for push, replace, popstate, manual URL
  edits, and external navigation entry through explicit host-to-worker
  envelopes in the default deployment posture
- explicit worker-first versus compatibility-deployment route-truth parity at
  the ingress boundary
- URL writeback contracts from graph state to browser URL
- route-local breadcrumb/back provenance built from route history truth rather
  than path slicing
- diagnostics that explain which browser boundary event or route writeback
  produced the current raw-location-to-route-truth transition

Phase 4 gate:

- no later phase begins until browser-driven and app-driven source changes can
  be shown to converge on one route truth and one history story across both
  worker-first and explicit compatibility deployment postures

### Phase 5: Typed Navigation Policy And Visible Projection Freshness

Purpose:

- freeze declared navigation semantics instead of per-screen improvisation
- make visible route projection freshness an explicit policy surface rather than
  an accidental byproduct of worker truth timing

This phase must ship:

- typed navigation intents for push, replace, soft refresh, same-route
  mutation, canonicalize, breadcrumb return, and restore-back
- explicit transition policy for speculative branch, direct commit, continuity,
  and redirect handling
- an inspectable lower-level navigation plan/policy surface for callers who
  need to own cost, continuity, artifact, or deployment decisions explicitly
- explicit projection refresh policy that distinguishes worker-owned route truth
  from visible cached route projection
- diagnostics that explain whether visible route truth is freshly refreshed,
  continuity-preserved, or intentionally stale while the admitted route truth
  has advanced

Phase 5 gate:

- no later phase begins until typed navigation intent and visible projection
  policy can be reasoned about independently from browser ingress mechanics

### Phase 6: Speculative Navigation And Branch Lifecycle

Purpose:

- close candidate navigation truth as a first-class branch-native capability
- prevent speculative navigation from collapsing into browser-router-shaped
  provisional state

This phase must ship:

- branch-native speculative navigation
- branch merge, merge-preview, or discard posture for speculative navigations
  that accumulate route-local mutable truth
- dirty-evaluation-backed exit and continuity posture where route leave safety
  or visible preservation depends on route-local mutable truth
- canonical diagnostics for whether a candidate navigation committed,
  redirected, discarded, merged, or remained pending

Phase 6 gate:

- no later phase begins until speculative route branches can be committed,
  discarded, or merged through explicit proof-bearing history operations rather
  than router-local heuristics

### Phase 7: Resource-Native Route Transitions, Prefetch, And Continuity

Purpose:

- turn the router into a consumer of the completed resource substrate instead
  of a second loader/cache engine
- separate speculative route lifecycle from route-local resource continuity and
  prefetch proof

This phase must ship:

- route-local resource admission and continuity through the completed resource
  API surface
- route-local prefetch and warmup posture driven by hover, focus, viewport, or
  explicit intent without inventing a second loader cache
- transition semantics that can preserve visible truth while pending when the
  declared continuity/resource policy allows it
- canonical diagnostics for whether a visible route change came from direct
  navigation, speculative branch commit, redirect, prefetch admission, or
  resource continuity preservation

Phase 7 gate:

- no later phase begins until route transitions can consume resource continuity
  and prefetch posture without inventing framework-local resolver grammar

### Phase 8: Reversible Navigation And Restore Parity

Purpose:

- expose the snapshot/restore substrate as a real navigation strength instead
  of treating browser back as the only history story
- make breadcrumb trails explicit route-history and restore artifacts rather
  than URL-shaped string reconstruction folklore

This phase must ship:

- route-history entries that preserve admitted route/outlet composition truth
- reversible navigation through restore-backed back and breadcrumb return
- route- and layout-owned breadcrumb contribution artifacts with explicit crumb
  identity, typed navigation target posture, and typed label status rather than
  path-segment inference
- breadcrumb ancestry strategies that resolve in ordered truth classes:
  recompute from durable route/resource truth, else consume carried or restored
  breadcrumb provenance, else degrade through explicit fallback artifacts
- typed restore-boundary truth for what route-local graph state is guaranteed to
  restore
- parity between direct restore-backed navigation and equivalent replay/history
  inspection artifacts

Phase 8 gate:

- no later phase begins until multi-outlet and nested-layout navigation can be
  restored honestly without overclaiming arbitrary non-graph local UI state
- no later phase begins until dynamic deep links can either recompute
  breadcrumb ancestry from durable truth, restore it from explicit carried
  provenance, or degrade to an honest fallback without pretending URL shape
  alone explains the trail

### Phase 9: Route Authority Handoff To Forms And Draft Continuity

Purpose:

- close the router/forms authority seam explicitly
- prevent restore, authority changes, and route-coupled forms from sharing
  draft semantics by accident

This phase must ship:

- semantic draft preservation/discard policy across route and authority changes
  with the router owning route authority policy and forms consuming the result
- explicit route-to-forms authority handoff for preserve, freeze, discard, or
  defer posture across route authority changes
- route-coupled form behaviors resolving through router authority instead of
  permanent deferred posture once the required route truth is present
- diagnostics that explain whether route authority preserved, froze, discarded,
  or deferred route-scoped draft continuity

Phase 9 gate:

- no later phase begins until route-coupled form behavior can consume router
  authority honestly without the router becoming the owner of general draft
  semantics

### Phase 10: SSR/Hydration Boundary, Cross-Tab Coherence, And Certification Closeout

Purpose:

- reserve the larger deployment boundaries honestly and close the milestone with
  hostile proof rather than examples

This phase must ship:

- explicit SSR and hydration posture for initial URL admission and route truth
  handoff
- explicit cross-tab and external-navigation coherence contract at the browser
  authority boundary
- router-first certification and closeout vocabulary strong enough to support a
  dedicated router test-requirements and closeout document rather than leaving
  certification implicit in the milestone prose
- canonical verification-package vocabulary for route truth, route outcomes,
  outlet composition, navigation provenance, and restore parity
- docs that teach the router as a graph-native product lane rather than a
  framework-specific helper

Phase 10 gate:

- the milestone is not closed until speculative branch navigation, stale-link
  recovery, resource-native transitions, reversible navigation, route-to-forms
  authority handoff, and browser history all converge under hostile
  replay/restore proof

## Must Ship

- typed URL, search-param, and hash-fragment route state
- canonical route equivalence and normalization contracts
- canonical shared route-pattern grammar with the route-first API lane where
  the underlying semantics are the same
- typed route schema authoring and typed navigation builders
- route-scoped layout and outlet composition
- typed route capability boundaries
- declaration-driven prerequisites and access-policy composition
- explicit typed route outcomes for admitted, redirect, notFound, forbidden,
  unavailable, and denied
- explicit worker-first browser-history ingress and route projection refresh
  policy
- speculative branch navigation
- branch merge, merge-preview, discard, and dirty-evaluation posture where
  speculative route truth needs reconciliation or exit proof
- nearest-valid-truth recovery for stale deep links and invalid tenant/project/
  module targets
- typed navigation intent classes and transition policy
- browser-history integration and URL writeback contracts
- resource-native route transitions and route-local prefetch/warmup posture
- semantic draft preservation/discard across authority changes
- reversible navigation backed by route restore truth
- diagnostics/history/replay/restore surfaces for navigation and route outcomes
- SSR/hydration and cross-tab/external-navigation posture stated honestly at
  the product boundary

## Must Preserve

- `worth-signal` remains the owner of branch, replay, restore, and diagnostics
  truth
- the browser host remains the owner of raw URL/history mutation, but not of
  typed route meaning
- worker-first deployment remains the default product authority posture; route
  truth must not silently fall back to a second main-thread router engine
- resources remain the owner of request, continuity, freshness, delivery, and
  diagnostics semantics
- forms remain the owner of draft semantics, validation, readiness, and action
  semantics outside the explicit route-side preserve/freeze/discard/defer
  contract
- the router does not become a second cache, loader engine, guard engine, or
  authority store
- route ergonomics must not hide broad tree rescans, broad resource work, or
  rich-history reconstruction behind cheap-looking APIs
- projected visible route truth must not silently masquerade as freshly
  refreshed admitted worker truth when policy intentionally preserves stale
  visibility

## Required Named Proof Families

- `The Canonical Route Equivalence Test`
- `The Typed Search And Hash Normalization Test`
- `The Route Projection And Layout Composition Test`
- `The Route Capability Boundary Compile-Time Test`
- `The Prerequisite And Access Policy Admission Test`
- `The Nearest Valid Recovery And Stale Deep Link Test`
- `The Browser History And App Navigation Convergence Test`
- `The Worker-First Browser History Ingress And Projection Refresh Test`
- `The Speculative Branch Navigation And Flicker Suppression Test`
- `The Speculative Route Branch Merge And Dirty Exit Test`
- `The Resource Native Route Transition And Continuity Test`
- `The Route Prefetch Without Second Loader Cache Test`
- `The Semantic Draft Preservation Across Authority Change Test`
- `The Reversible Outlet Composition Restore Test`
- `The Breadcrumb And Back Provenance Test`
- `The Navigation Diagnostics And Auditability Test`
- `The SSR Hydration And External Navigation Boundary Test`

## Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- canonical path/search/hash normalization yields one route identity story for
  semantically equivalent URLs
- direct URL edits, typed app navigation, browser back/forward, and external
  navigation entry converge to the same admitted route truth across worker-first
  and explicit compatibility deployment postures
- nested layout and outlet composition are projected from route truth rather
  than framework-local orchestration state
- the visible route projection can explain when it is showing cached continuity
  truth versus freshly refreshed admitted route truth
- prerequisite denial, redirect, and nearest-valid recovery happen before
  route-local resources, drafts, or layout truth are treated as admitted
- resource-backed route transitions consume existing continuity, freshness, and
  diagnostics truth instead of inventing resolver/loader folklore
- speculative navigation avoids half-committed route state and yields explicit
  diagnostics about why a candidate navigation committed, redirected, or
  discarded
- speculative route-local truth can be discarded, merged, or previewed through
  explicit branch/history proof instead of ad hoc router-local heuristics
- deep tenant/workspace/project switching either restores or converges to one
  nearest valid route truth without orphaned cross-authority state
- back, breadcrumb return, and reversible navigation restore admitted
  route/outlet composition through explicit restore-boundary truth
- breadcrumb trails for dynamic deep links either recompute parent ancestry
  from durable route/resource truth, restore it from explicit carried
  breadcrumb provenance, or degrade to typed fallback artifacts without
  fabricating missing search/filter context from URL shape alone
- route-coupled form behavior resolves through an explicit router authority
  handoff instead of remaining in permanent typed deferred posture
- diagnostics can explain why the current route is visible, which prerequisite
  or redirect shaped it, which navigation intent produced it, and which route
  history or restore boundary currently explains it
- SSR/hydration and cross-tab/external-navigation boundaries are stated
  honestly enough that later work can extend them without semantic drift

## Architectural Notes

- the strongest shape is route-family-first rather than component-instance-first
  so canonical identity, matching, and equivalence stay centralized
- the router should consume the closed worker-first host boundary and history
  surface directly instead of introducing a parallel navigation authority layer
- route-local resources should be published graph consumers of route params and
  route search state, not hidden inside loader callbacks
- route-local forms should consume explicit route authority and route-side
  draft-preservation policy rather than encoding route semantics inside form
  controllers
- layout and outlet projection should reuse the existing controller and graph
  publication substrate instead of defining a parallel router composition model
- reversible navigation should prefer explicit restore-boundary artifacts over
  vague promises about "restoring page state"
- breadcrumb composition should remain router-owned and history-aware so child
  routes never need to reconstruct ancestor trail truth they do not
  semantically own
- where route schema and route-first API declarations mean the same thing, they
  should reuse one grammar, one canonicalization story, and one denial posture
- if a route transition needs richer state than URL alone to be explainable,
  that state should become a named route artifact or carried breadcrumb
  provenance artifact rather than hidden framework memory

## Sequencing Notes

This milestone belongs after the API surface because:

- route transitions, prefetch, redirects, and continuity need the completed
  resource request/lifecycle/diagnostics substrate
- speculative navigation and reversible back/breadcrumb flows are much stronger
  once branch, restore, and route-local resource truth are already real

This milestone remains separate from forms because:

- forms should consume route authority changes and route-local draft
  preservation rules where appropriate
- the router should not become the first place where general draft semantics
  are solved

This milestone should now also be treated as explicitly downstream of the
closed worker-first runtime lane because:

- browser history ingress, committed route truth, branch operations, restore,
  and diagnostics already have a real worker-owned boundary
- the router no longer needs to reserve worker placement as future design
  space; it should consume the closed substrate directly

Certification follow-on:

- this plan should close through a dedicated router test-requirements document
  and a dedicated router closeout document, matching the resource and worker
  milestone pattern instead of leaving proof obligations only in the milestone
  spec

Current judgment:

- write the router as a graph-native navigation product, not a browser-history
  helper
- make stale-link recovery, speculative branch navigation, and reversible
  route/outlet restore first-class strengths
- preserve headroom for more ambitious constraint-solved navigation later
  without making it part of the initial milestone contract
