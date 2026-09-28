# worth-signals-wasm Router Test Requirements

> **Status:** Planned certification spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Milestone parent:** [router_navigation_projection_plan.md](./router_navigation_projection_plan.md)
>
> **Core lineage:** [plans/worth_signal/test-requirements.md](../WORTH_signal/test-requirements.md)

## Purpose

This document defines the certification bar for the `worth-signals-wasm`
router and navigation product surface.

It is not a list of example route tests.
It is the proof contract that closes the router milestone.

The milestone is not done when:

- path matching works for a handful of demo routes
- `navigate(...)` feels pleasant in app code
- browser back/forward appears to work in ordinary flows
- one framework adapter can render nested outlets

The milestone is done only when the product surface can prove that:

- browser-owned location changes, typed app navigation, restore, replay, and
  speculative branch navigation converge to the same admitted route truth
- route projection, outlet composition, resource continuity, and route-coupled
  form authority stay inside one graph-owned truth story
- worker-first mode and explicit compatibility mode preserve one route meaning
  rather than one semantic model per deployment posture
- visible route continuity and delayed projection refresh remain explicit policy
  rather than stale-cache accidents
- route ergonomics do not hide a second loader, guard, cache, or browser-local
  state machine behind pleasant API shape

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is hostile-proof product design. This
  certification must prove route truth under branch churn, stale links, restore,
  and continuity pressure rather than only nominal navigation.
- `arch_laws.md`
  The most important thing it protects is authority and proof honesty. Browser
  ingress, route admission, route outcomes, restore boundaries, and forms
  handoff must remain distinct typed categories rather than convenience blur.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. The suite must prove
  that route transitions, projection refresh, prefetch, and history reads scale
  with semantic delta rather than route-tree breadth or cached projection size.
- `dx_laws.md`
  The most important thing it protects is truthful ergonomics. The suite must
  prove that the common route lane reads like intent, the advanced lane exposes
  inspectable plan/policy truth, declaration boundaries turn strings into typed
  route references, and expensive navigation work looks expensive.
- `domain_laws.md`
  The most important thing it protects is proof-domain clarity. URL
  canonicalization, admission, history ingress, branch lifecycle, resource
  transitions, restore parity, and forms handoff need separate owning suites.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  execution substrate. The router must consume graph/runtime truth, not become
  a second authority store.
- `router_navigation_projection_plan.md`
  The most important thing it protects is the milestone boundary. The router is
  graph-native route truth with typed ingress, admission, branching,
  continuity, restore, and forms handoff, not a browser wrapper or framework
  router clone.
- `worker_runtime_placement_closeout.md`
  The most important thing it protects is worker-first deployment honesty.
  Browser history ingress and visible route projection must certify one route
  truth across worker-first and compatibility postures.
- `api_surface_closeout.md`
  The most important thing it protects is reuse of the closed resource line
  model. Route-local resources and prefetch must consume that substrate rather
  than reopening loaders or continuity semantics inside the router.
- `forms_product_surface_plan.md`
  The most important thing it protects is the forms/router split. Route-coupled
  form behavior must resolve through router authority while forms stays the
  owner of draft, readiness, validation, and action semantics.
- `worth_signal/test-requirements.md`
  The most important thing it protects is certification rigor. This document
  must require named hostile suites, replay/restore parity, compile-time
  boundaries, and cost proof rather than route-demo confidence.

## Adversarial Constraint

This certification program must survive the following hostile condition:

> A long-lived web application with nested layouts, multiple outlets, typed
> route params and search state, worker-first browser-history ingress,
> compatibility-mode fallback, route-local resources, route-coupled forms,
> speculative branch navigation, dirty-exit pressure, nearest-valid recovery,
> restore-backed back/breadcrumb flows, stale deep links, tenant/workspace
> switching, and replay/restore activity must converge to the same admitted
> route truth, visible outlet composition, route-local continuity truth, and
> diagnostics/history explanation regardless of whether the transition was
> driven by typed navigation, raw URL edits, popstate, speculative branch
> evaluation, route redirect, resource continuity preservation, restore, or
> replay.

If semantically equivalent histories can produce:

- different admitted route identity or route outcome truth
- different visible outlet composition after equivalent redirects or restores
- different continuity behavior between worker-first and compatibility modes
- stale visible route truth without explicit projection policy explaining it
- route-local resource or form semantics that drift from their owning products
- speculative branch discard/merge behavior that leaves orphaned route truth
- or a second loader, guard, cache, or browser-local route authority

then the milestone has failed certification.

## Certification Rules

Every required named suite in this document must:

- run with canonical artifact emission, not only assertion-style pass/fail
- define its hostile workload explicitly
- verify runtime behavior, public product behavior, and type-surface boundaries
  where relevant
- certify replay/restore/branch parity whenever the router claims those
  semantics exist
- certify worker-first versus compatibility parity whenever the public surface
  crosses the browser-history boundary
- certify breadth or cost honesty whenever the API looks cheap
- certify DX honesty whenever the public surface claims a friendly lane:
  common-path calls must preserve semantic truth, advanced lanes must expose the
  next lower inspectable plan/policy boundary, and string boundaries must not
  leak past declaration surfaces without typed references
- include denial, defer, redirect, or unavailable artifacts where route truth
  cannot be admitted honestly

Representative DX examples the suites should certify:

```ts
const routes = signals.router.define({
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

```ts
const plan = routes.userDetail
  .intent({
    params: { userId: "u1" },
    search: { tab: "activity" },
  })
  .policy({
    continuity: "preserve-visible-while-pending",
    projectionRefresh: "explicit",
  })
  .compile();

plan.explain();
await signals.router.execute(plan);
```

```ts
// out of spec after declaration
router.push(`/users/${userId}`);
window.history.pushState({}, "", `/users/${userId}`);
```

Where a suite names a compile-time boundary, the package must maintain explicit
compile-fail fixtures or equivalent type-surface proof artifacts that stay in
sync with the public route contract.

## Verification Package Standard

Every broad certification family should emit a canonical verification package
containing the categories relevant to that suite.

The package vocabulary for this milestone is:

- route schema digest
- canonical URL digest
- route reference and declaration digest
- route candidate digest
- route admission digest
- route outcome digest
- route projection and outlet digest
- browser-history ingress digest
- navigation intent and policy digest
- navigation plan explainability digest
- visible projection freshness digest
- speculative branch lifecycle digest
- resource continuity and prefetch digest
- restore-boundary digest
- forms authority handoff digest
- diagnostics/history provenance digest
- worker-first truth digest
- compatibility truth digest
- boundary performance envelope
- typed denial/defer/unavailable artifact

Equivalent runs must match exactly except for fields explicitly declared
non-semantic.

## 0. The Full Router Hostile Convergence Test

Purpose

Prove that the complete router product surface remains one coherent system
rather than one route matcher plus several drifting helper engines for browser
history, resource continuity, restore, and form handoff.

Why it matters

Phase-local suites can all pass while the real product still forks into:

- one truth story for typed app navigation
- another truth story for raw browser history ingress
- one continuity story for resource-backed transitions
- another continuity story for visible route projection
- one restore story for back/breadcrumb
- another authority story for route-coupled forms

That is exactly the failure mode this milestone exists to prevent.

What to stress

Build one medium-large application graph containing:

- nested layouts with multiple outlets
- typed path params, search params, and hash posture
- route-local auth and policy prerequisites
- route-local resource lines with continuity and prefetch posture
- route-coupled form steps and route-side preserve/freeze/discard policy
- speculative route branches with dirty-exit pressure
- browser-history ingress, direct URL edits, and app-issued navigation
- restore-backed back/breadcrumb flows

Run one hostile script with:

- repeated equivalent URLs expressed in non-canonical forms
- direct URL edits interleaved with typed navigation and popstate
- redirects, denials, nearest-valid recovery, and stale deep links
- speculative navigation before and after route-local mutable work
- resource completion churn while visible route continuity is preserved
- route authority changes that preserve, freeze, discard, or defer form
  continuity
- branch restore before and after route-local resource and form activity
- dynamic deep links whose breadcrumb ancestry is only partially durable and
  otherwise depends on carried or restored provenance
- replay from retained history and replay from full canonical history
- worker-first and compatibility-mode execution of the same semantic workload

Execute the full scenario in at least:

- worker-first mode
- explicit compatibility mode
- branch fork plus restore execution
- retained-history replay
- full canonical replay

What to verify

- all modes converge to identical admitted route truth when semantically
  equivalent
- all modes converge to identical visible outlet composition and route outcome
  explanation
- visible projection freshness remains explicitly attributable rather than
  stale-by-accident
- route-local resource continuity and route-coupled form handoff stay aligned
  with their owning products
- no path creates a second loader, guard, cache, or browser-local route
  authority

Pass condition

The verification package must emit canonical URL, route candidate, route
admission, route outcome, outlet composition, browser ingress, navigation
policy, projection freshness, speculative branch, resource continuity, restore,
forms handoff, diagnostics/history, worker-first truth, compatibility truth,
and boundary performance artifacts. Equivalent histories must match exactly.

## Phase Coverage Map

- Full milestone closeout additionally requires suite 0.
- Phase 1 is closed only by suites 1 through 3.
- Phase 2 is closed only by suites 4 through 6.
- Phase 3 is closed only by suites 7 through 9.
- Phase 4 is closed only by suites 10 through 12.
- Phase 5 is closed only by suites 13 through 15.
- Phase 6 is closed only by suites 16 through 18.
- Phase 7 is closed only by suites 19 through 21.
- Phase 8 is closed only by suites 22 through 24.
- Phase 9 is closed only by suites 25 through 27.
- Phase 10 is closed only by suites 28 through 30.

## Phase 1: URL Authority, Canonicalization, And Route Schema Lock

This phase certifies that route identity is canonical before projection or
admission begin. It protects the milestone from devolving into stringly URL
helpers with late canonicalization.

The intended authoring shape for this phase is:

```ts
const routes = signals.router.define({
  userDetail: signals.router.route("/users/:userId", {
    search: {
      tab: signals.router.search.optional.string(),
    },
  }),
});

routes.userDetail.to({
  params: { userId: "u1" },
  search: { tab: "activity" },
});
```

1. `The Canonical Route Equivalence Test`

2. `The Typed Search And Hash Normalization Test`

3. `The Shared Route Grammar And Compile-Time Boundary Test`

## Phase 2: Route Projection, Layouts, And Outlet Composition

This phase certifies that route matching projects real route-composition truth
without smuggling admitted outcome semantics into a mere match result.

4. `The Route Projection And Layout Composition Test`

5. `The Projected Candidate Versus Admitted Outcome Separation Test`

6. `The Route Capability Boundary Compile-Time Test`

## Phase 3: Admission, Access Policy, And Nearest-Valid Recovery

This phase certifies that prerequisites, redirects, and stale-link recovery
resolve before route-local work is treated as admitted.

7. `The Prerequisite And Access Policy Admission Test`

8. `The Nearest Valid Recovery And Stale Deep Link Test`

9. `The Redirect And Denial Provenance Test`

## Phase 4: Browser History Boundary And Route Truth Convergence

This phase certifies the host boundary itself: raw browser events, direct URL
edits, and graph-issued writeback must converge to one route truth in both
worker-first and compatibility postures.

10. `The Browser History And App Navigation Convergence Test`

11. `The Worker-First Browser History Ingress Parity Test`

12. `The URL Writeback And External Navigation Boundary Test`

## Phase 5: Typed Navigation Policy And Visible Projection Freshness

This phase certifies the semantic layer above ingress: navigation intent and
visible projection freshness must be explicit policy, not timing accidents.

13. `The Typed Navigation Intent Policy Test`

14. `The Visible Projection Freshness Policy Test`

15. `The Freshness Diagnostics And Continuity Attribution Test`

## Phase 6: Speculative Navigation And Branch Lifecycle

This phase certifies branch-native candidate route truth. Speculation must use
real branch lifecycle, dirty-exit proof, and merge/discard posture rather than
router-local provisional state.

16. `The Speculative Branch Navigation And Flicker Suppression Test`

17. `The Speculative Route Branch Merge And Dirty Exit Test`

18. `The Speculative Navigation Diagnostics And Discard Honesty Test`

## Phase 7: Resource-Native Route Transitions, Prefetch, And Continuity

This phase certifies that route transitions consume the closed resource line
model instead of reopening loader, prefetch, and pending-state folklore.

19. `The Resource Native Route Transition And Continuity Test`

20. `The Route Prefetch Without Second Loader Cache Test`

21. `The Pending Continuity Visibility Policy Test`

## Phase 8: Reversible Navigation And Restore Parity

This phase certifies that restore-backed navigation is an honest product
capability with explicit restore boundaries, replay parity, and breadcrumb
truth that does not collapse back into URL-shaped reconstruction folklore.

22. `The Reversible Outlet Composition Restore Test`

23. `The Breadcrumb And Back Provenance Test`

   This suite must include hostile dynamic deep-link cases where the leaf route
   cannot honestly reconstruct its ancestry from URL shape alone, such as
   `/search/results/:resultId` when the search filters that produced the result
   are not present in the URL. The proof bar is:

   - durable ancestry recompute wins when real route/resource truth exists
   - explicit carried or restored breadcrumb provenance wins when durable
     ancestry does not exist
   - otherwise the trail degrades to an honest fallback artifact
   - the router never fabricates missing filter/query ancestry from path or
     search shape alone
   - breadcrumb entry status must distinguish recomputed, carried, restored,
     recovered, fallback, pending, and unavailable truth classes if those
     classes are observable at the public boundary

24. `The Replay Restore Route Parity Test`

## Phase 9: Route Authority Handoff To Forms And Draft Continuity

This phase certifies the router/forms seam. Route authority may decide
preserve, freeze, discard, or defer posture, but it must not silently take
ownership of form semantics.

25. `The Route Authority To Forms Handoff Test`

26. `The Semantic Draft Preservation Across Authority Change Test`

27. `The Route-Coupled Forms Deferred-To-Admitted Transition Test`

## Phase 10: SSR/Hydration Boundary, Cross-Tab Coherence, And Certification Closeout

This phase certifies the reserved deployment boundaries and the final
auditability story that closes the milestone.

28. `The SSR Hydration Route Truth Handoff Test`

29. `The Cross-Tab And External Navigation Coherence Test`

30. `The Navigation Diagnostics And Auditability Closeout Test`
