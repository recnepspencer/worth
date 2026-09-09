# Worth UI Future Roadmap

## Purpose

This document defines the future work for Worth UI.

It is a future-only roadmap. It does not treat Worth UI as a widget bundle or
an ornamental layer over a renderer. It exists to sequence the remaining work
required to turn Worth UI into a real desktop application platform with
hot-lowered iteration, canonical UI artifacts, Query-bound product surfaces,
platform-grade shell behavior, native desktop integration, and frame-efficient
execution lanes for both workbench UI and hostile real-time surfaces.

The governing rules remain:

- the canonical UI artifact is the source of runtime UI meaning
- compiled Rust defines capabilities; hot-reloadable UI source composes them
- file-authored UI and Rust-authored composition must converge on the same
  canonical artifact and execution-plan pipeline
- if Worth Query already owns a stronger runtime-backed public lane for
  application-operation admission, typed bindings, projection consumption,
  async or result posture, recovery, inspection, explanation, or grouped/read/
  query products, Worth UI must consume that lane through the declared audience
  facades rather than rebuild a UI-local pseudo runtime
- app-shell behavior, interaction semantics, and execution plans must be
  platform-owned rather than app-local folklore
- semantic richness must lower before the hot path runs
- desktop UX, runtime honesty, and performance certification are all part of
  product completeness

Delivery cadence is phase- and milestone-bound: commit and push when the phase
implementation satisfies its specification, focused tests and required checks
pass, and assigned reviewers clear the final diff. Merge to `master` when the
milestone acceptance conditions and CI are green. Historical proof and closure
ledgers are frozen, non-gating records; they are not updated or reopened. A
ledger-only failure does not block delivery. Current QA follows
[the QA review guide](../coding_guidelines/qa_review_guide.md) and
[testing laws](../coding_guidelines/testing_laws.md).

Worth UI must remain strong enough for workbenches, editors, topology and CAD
tools, AI-native editing systems, operational apps, data-heavy consoles,
simulation tools, plugin-driven products, and real-time visualization or HUD
surfaces that need stronger guarantees than "egui plus some widgets."

## Current Roadmap Position

The current state is foundation-rich rather than product-complete. Milestone
3.14.1 is closed and the product pulse now runs on the Worth-owned aspect-native
host; the remaining roadmap begins from that native baseline.

The shipped baseline for Worth UI today is:

- the platform thesis captured in
  [worth-ui-vision.md](./worth-ui-vision.md)
- the DSL direction captured in
  [worth-ui-dsl-vision.md](./worth-ui-dsl-vision.md)
- the inspection and AI-diagnostics direction captured in
  [ai-diagnostics.md](./ai-diagnostics.md)
- the completed retirement of the interim `egui` host in Milestone 3.14.1 and
  the resulting Worth-owned aspect-native host baseline
- Worth Query's current audience model: UI declarations and application-facing
  requirements cross `worth-query-decl`, host progression crosses
  `worth-query-host`, and Query replay or reconstruction crosses
  `worth-query-replay` only from certification-owned lanes
- the milestone ordering needed to avoid drifting into widget-first or
  application-local infrastructure before the platform foundations exist

This roadmap therefore tracks the work needed to turn the vision into a real
platform sequence.

The existing raw `worth-query` dependency in `worth-ui-query-binding` is a
predecessor edge, not the destination contract. Its remaining raw-engine
imports must be removed so `worth-query-decl` and `worth-query-host` are the
only ordinary Query audiences before later Query-facing breadth is admitted.
Milestone 3.15 may not widen that edge or use production runtime services to
disguise it.

## Roadmap Rules

Rules for every remaining Worth UI item:

- each milestone must describe a real platform capability, not just a component bundle
- each milestone must solve a structural problem before the dependent product features broaden
- each milestone must preserve the ownership boundary between Worth UI, Worth Query, the runtime bridge, truth/runtime authority, and lower native adapters
- Worth Query is the application-facing composition authority over the domain
  and lower runtimes that own truth; Worth UI consumes its published products
  through `worth-query-decl` and `worth-query-host`, never by treating the raw
  Query engine as an ordinary UI dependency
- replay and reconstruction are certification-only; ordinary UI inspection may
  retain, compare, and explain evidence but must not import
  `worth-query-replay` or open replay authority
- each milestone must preserve hot-lowered composition, canonical UI artifacts, and no per-frame source interpretation
- each milestone that touches authored UI source, declaration artifacts, or
  runtime composition must preserve the DSL rule that authoring is a semantic
  language for canonical runtime declarations rather than a component,
  modifier, selector, or render-local widget language
- each milestone must treat the running Worth runtime as the primary host for
  hot reload, diagnostics, stable identity reconciliation, and safe plan swaps
- each milestone must preserve explicit accessibility, keyboard, focus, and diagnostics posture rather than treating them as polish
- each milestone must preserve frame-cost honesty through named counters and execution-plan boundaries
- frame-cost claims that cross diagnostic, report, or certification boundaries
  should lower Worth UI evidence into WORTH Foundational performance claims,
  canonical bundles, counter-backed receipts, planned reports, and readiness
  envelopes instead of inventing local performance folklore
- each runtime milestone must ship the retained evidence, ordinary inspection,
  comparison, relevance, and, where certification requires it, separate
  certification replay inputs needed to explain the runtime families it
  introduces; explanation is not a late debug pass
- AI-facing inspection harnesses must arrive before, or at least alongside, the
  runtime families they need to inspect; a polished human inspector may arrive
  later, but formal AI entry points may not
- the human inspector must use a familiar, shallow developer-tools interaction
  model: point-to-select, visible highlighting, dockable panels, and a small
  stable set of task-oriented tabs; it should feel immediately usable to a
  Chrome DevTools user while replacing DOM, CSS, and network folklore with
  Worth declarations, appearance projections, data, schema, performance, and
  causal evidence
- each milestone must preserve a structurally explicit layout model rather than drifting back toward DOM-shaped percentage, overflow, and implicit-parent folklore
- each milestone must define concrete acceptance evidence through platform
  scenarios, diagnostics artifacts, performance counters, comparison-safe
  ordinary plans, certification replay where required, tooling evidence, or
  certification suites
- no milestone is complete until both implementation and trust evidence exist
- features that depend on stable identity, shell contracts, or interaction contracts must not ship before those foundations exist
- Worth UI must not become a second truth runtime, a second query runtime, or a web-runtime clone
- DSL sugar must follow admitted runtime lanes; authored syntax must not
  outrun declaration artifacts, aspect contracts, graph truth, measurement,
  Query binding, intent, services, or diagnostics support

## Foundation-First Critical Path

This section is the first build priority.

These are the milestones that determine whether Worth UI becomes a real
platform or a shallow collection of components:

- canonical lowering and artifact architecture
- hot composition runtime architecture
- frame-efficient execution lanes and cost certification
- app shell and command/interaction foundations
- Query-bound views and forms/workflow surfaces

If this section is weak, everything above it inherits the same failure mode:

- hot composition devolves into interpreted UI, Rust rebuild friction, or
  renderer-owned state carry-forward
- shell behavior becomes app-local glue
- components ship without coherent focus, accessibility, or runtime posture
- Query-bound surfaces drift back toward local caches and event plumbing
- performance claims remain vibes rather than certified contracts

## Milestone 1: Platform Skeleton, Facade, and Capability Registries

Detailed spec: [milestone-1.md](./milestone-1.md)

### Goal

Define Worth UI as one subsystem with a clean facade, stable vocabulary, and
mechanically visible capability boundaries before the rest of the platform is
built on top of ad hoc app-local abstractions.

### Must Ship

- the top-level Worth UI facade and crate topology
- explicit public vocabulary for UI source, canonical artifact, execution
  plan, capability registry, shell surface, command surface, view surface, and
  render surface
- capability registries for commands, components, domain-agnostic surfaces,
  mosaic region kinds, mosaic placement policies, mosaic sizing contracts,
  mosaic state slots, Query view bindings, runtime outcome projections,
  settings, task presentations, theme tokens, icons, command projections,
  plugin contribution slots, and native capability descriptors
- visibility boundaries that keep lower implementation topology private behind
  the facade
- typed registration contracts strong enough that later lowering can validate
  UI source without rediscovering platform meaning from strings or app-local
  code
- one small end-to-end registration path proving compiled Rust capabilities can
  be registered without hidden global state

### Must Preserve

- Worth UI meaning remains above native rendering and does not entangle
  low-level rendering ownership with platform ownership
- Worth UI does not become a second Query or truth runtime
- capability definition stays in compiled Rust rather than moving into
  untyped runtime source
- facade stability remains more important than internal topology convenience

### Acceptance Evidence

- one narrow public facade can build a minimal Worth UI app without deep
  imports into implementation modules
- adding a new registry family or lifecycle boundary forces explicit compiler
  updates at every construction site that must propagate it
- invalid capability registration shapes fail mechanically rather than through
  documentation or runtime folklore
- one audit pass can name the exact public types that own capability
  registration, lowering input, lowering output, and execution-plan input

## Milestone 2: Canonical UI Source, Lowering, and Runtime Artifact

Detailed spec: [milestone-2.md](./milestone-2.md)

### Goal

Make repo-authored UI source lower into one canonical runtime artifact so the
platform owns UI meaning explicitly before hot reload, shell work, or component
growth broaden the surface area.

### Must Ship

- codebase-authored UI source format for shell composition, panels, menus,
  toolbars, tables, inspectors, forms, tokens, and bindings
- Rust-native composition API or macro path that can emit the same canonical
  artifact input as file-authored UI source
- authored syntax shaped by [worth-ui-dsl-vision.md](./worth-ui-dsl-vision.md):
  semantic lanes, explicit declaration families, and no component-local
  modifier/selector/view-builder authority
- parser and validator that consume source and capability registries
- one canonical UI artifact carrying stable IDs, component references, command
  bindings, Query/view bindings, layout intent, accessibility metadata, and
  diagnostics
- typed rejection for invalid component IDs, invalid command IDs, mismatched
  bindings, missing required props, and illegal artifact structure
- source-level layout declarations strong enough to name mosaic regions,
  nesting, split or stack or overlay or pinned behavior, scroll ownership, and
  grow or shrink rules without DOM-style percentage-height ambiguity
- artifact inspection surfaces usable by later tooling and diagnostics
- at least one sample app expressed through the source -> artifact pipeline

### Must Preserve

- the artifact remains the source of runtime UI meaning once lowering
  completes
- file-authored source and Rust-authored composition do not fork artifact
  meaning, diagnostics, or execution planning
- Query-facing runtime surfaces referenced by the artifact remain Query-owned;
  Worth UI may bind, route, inspect, and present them, but must not recreate
  local query, result-state, recovery, or explanation models
- source parsing and validation do not leak into the steady-state frame path
- artifact meaning remains independent of diagnostics richness
- the lowering pipeline does not bypass capability registries or app facade
  boundaries
- the source format does not smuggle structure, layout, appearance,
  operability, intent, or service meaning through modifier order, selector
  reach, or render-local builder semantics

### Acceptance Evidence

- semantically identical UI source lowers to identical canonical artifact
  identity and structure
- intentionally different source lowers to mechanically different artifact
  identity or structure
- invalid source fails with structured diagnostics while preserving the prior
  valid artifact
- artifact inspection can explain what source bound to what registered
  capabilities without reading Rust control flow
- Rust-authored composition and file-authored source that declare the same UI
  meaning lower to equivalent artifact identity and structure
- semantically equivalent authoring does not depend on modifier order,
  selector precedence, or ambient builder context to lower correctly

## Milestone 3: Hot Composition Architecture Series

This roadmap milestone is now a series rather than one overloaded runtime lump.

It co-develops with [worth-ui-dsl-vision.md](./worth-ui-dsl-vision.md) and
[ai-diagnostics.md](./ai-diagnostics.md). The DSL is the semantic source
boundary for the same runtime architecture; the AI/inspection substrate is the
runtime explanation boundary for that architecture. Neither should evolve as a
separate folklore layer ahead of admitted runtime lanes.

The sequencing rule is:

```text
3.1 establishes inspection authority and the formal harness contract
later 3.x slices continuously enrich that harness with real runtime evidence
```

The sequence is intentional:

```text
meaning
-> identity
-> aspects
-> indexes
-> obligations
-> inspection evidence
-> measurement
-> allocation frame dispatch
-> stream/invalidation resolution
-> allocation receipts
-> lowered execution plans
-> visual snapshots
-> observations
-> rebind
-> Query binding
-> intent
-> services
-> diagnostics
-> visual evaluation
-> AI inspection tools
-> inspector surface
-> certification
```

Hot reload is not a standalone feature milestone. It is the behavior of this
entire authority pipeline under source, Query, measurement, host-observation,
and world changes.

### Goal

Turn canonical Worth UI artifacts into a runtime-owned hot composition
substrate whose declaration, aspect contract, graph, indexes, obligations,
inspection evidence, measurement, execution plans, receipts, observations,
rebind, diagnostics, and certification are real before shell, forms, and
richer product UX broaden the surface area.

### Must Preserve Across The Series

- semantic richness stays in lowering-time artifacts, admitted contracts, graph
  topology, and execution plans rather than leaking into per-frame host code
- invalid reloads never blank, corrupt, or silently replace the last admitted
  mounted truth
- identity changes remain explicit replacement events rather than accidental
  state loss or tree-position folklore
- Query-facing runtime surfaces preserve Query-owned support, admission, live,
  async/result, projection, recovery, inspection, and explanation posture
- aspects and indexes remain part of correctness and proof, not a later
  optimization pass
- host adapters remain native-mechanics translators rather than semantic owners
- diagnostics, inspection, and certification remain typed runtime artifacts
  rather than logs, strings, or demo-only helpers
- the AI harness remains a first-class runtime consumer rather than a late
  screenshot-and-log convenience layer
- the human inspector remains a projection over runtime evidence rather than a
  second explanation system

### Milestone 3.1: Runtime Boundary Closure and Crate Split

This slice gives future runtime work the correct architectural home before more
behavior lands. It also establishes the formal inspection authority boundary so
AI and human inspection never have to scrape logs or invent a second
explanation runtime later.

**Must ship**

- target architectural split for:
  - `worth-ui`
  - `worth-ui-dsl`
  - `worth-ui-runtime`
  - `worth-ui-inspection`
  - `worth-ui-query-binding`
  - `worth-ui-host-contract`
  - `worth-ui-host-native`
  - `worth-ui-host-headless`
  - `worth-ui-certification`
- `worth-ui-host-contract` as the stable host boundary, with native and
  headless mechanics outside runtime meaning
- sealed inspection and harness contract types:
  - `UiInspectionTarget`
  - `UiInspectionQuery`
  - `UiInspectionScope`
  - `UiEvidenceBudget`
  - `UiEvidenceRichness`
  - `UiInspectionReceipt`
- one formal inspection facade entry point that accepts typed inspection
  queries and returns typed receipts, even while later milestones broaden the
  supported targets, scopes, and evidence families
- typed unsupported/not-yet-admitted posture for inspection targets or scopes
  whose backing runtime families have not landed yet
- support snapshot shape on the public Worth UI side
- hard-prohibition audit for obvious forbidden imports, folders, and deep
  adapter/runtime coupling
- hard-prohibition audit for renderer-local debug helpers, panel-local
  diagnostic truth, and host-bypass explanation paths

**Acceptance evidence**

- public facade exists without reopening Milestone 1 as generic platform
  skeleton work
- DSL ownership exists as a first-class crate boundary from the start instead
  of being scattered through facade or runtime modules
- inspection authority exists as a first-class runtime boundary from the start
- AI-facing and human-facing inspection must route through the same runtime
  inspection facade rather than separate ad hoc lanes
- unsupported inspection scopes fail through typed posture on the formal
  harness rather than through missing APIs or string errors
- host adapter cannot depend on runtime internals except through admitted host
  contracts
- a second host adapter can be introduced against `worth-ui-host-contract`
  without changing runtime truth ownership or moving host-neutral types into
  the runtime or product facade
- certification crate exists as the canonical anti-cheating home rather than
  test helpers drifting through unrelated modules

### Milestone 3.2: Canonical Declaration Artifacts and Aspect Contracts

This slice extends Milestone 2 from "canonical source lowers once" into
"runtime-owned declarations already carry the semantic contracts hot
composition needs later."

This is the first 3.x slice that must stay in lockstep with the DSL vision.

**Must ship**

- `UiDeclarationArtifact`
- `UiDeclarationIdentity`
- `UiDeclarationFamily`
- `UiAspectContract`
- support snapshot and support-row scaffolding for declaration families
- initial admitted declaration families:
  - `page`
  - `page-set`
  - `region`
  - `mosaic`
  - `local-composition`
  - `control`
  - `query-binding`
  - `intent`
  - `diagnostic-surface`

**Acceptance evidence**

- authored declaration lowers once into a canonical artifact with stable
  identity, family, aspect contract, topology, touch meaning, measurement
  policy, service usage, and Query binding posture
- no runtime phase rediscovering UI meaning from source text or renderer code
- aspect coverage report exists and can explain what semantic slices the
  declaration publishes
- declaration artifacts are rich enough to support lane-oriented DSL authoring
  without falling back to component-local meaning blobs

### Milestone 3.3: UI Authority Graph, Identity, Participation, and Core Indexes

This slice creates the runtime-owned graph that decides what exists, where it
lives, what participates, and how bounded lookup works.

**Must ship**

- `UiGraphNodeIdentity`
- `UiGraphSnapshot`
- parent/child/slot topology
- page/region/mosaic membership
- explicit presence and participation posture
- declaration-instance correspondence
- stable repeated-instance identity
- initial core indexes:
  - `graph node identity -> node`
  - `declaration identity -> graph node(s)`
  - `parent identity -> child set`
  - `slot identity -> occupant set`
  - `page identity -> participating node set`
  - `published aspect -> publishing node/receipt set`
  - `consumed aspect -> dependent node/receipt set`
  - `mounted receipt identity -> mounted receipt`

**Acceptance evidence**

- no recursive tree walk for ordinary lookup
- no tree-position identity
- graph mutation and index mutation are transactionally aligned
- participation axes stay explicit: mounted, visible, layout, hit-test, focus,
  accessibility, input, and diagnostic

### Milestone 3.4: Admission, Support, and Graph Touch Obligations

This slice closes the "who selects checks?" boundary. Callers declare touched
meaning; the runtime selects obligations.

**Must ship**

- `UiGraphTouchDescriptor`
- `UiSelectedObligationSet`
- `UiObligationDispatchPlan`
- `UiObligationVerdict`
- `UiAdmissionReport`
- world-aware support and admission posture
- initial obligation families:
  - `structural-legality`
  - `participation-legality`
  - `slot-contract`
  - `measurement-requirement`
  - `query-binding-requirement`
  - `intent-operability-requirement`
  - `diagnostic-surface-requirement`

**Acceptance evidence**

- declaration-change touches select obligations
- host-observation touches can select a different obligation neighborhood
- appearance-only touches do not trigger structure/focus obligations
- invalid structural edits deny before graph corruption
- denied admission preserves prior admitted mounted truth

### Milestone 3.5: Inspection Evidence Expansion and Relevance Indexes

This slice operationalizes the 3.1 inspection boundary by adding the first
substantial evidence families, relevance routing, and indexes over the runtime
truth already established by declarations, graph topology, and admission.

**Must ship**

- `UiEvidenceSlice`
- `UiRelevanceFilter`
- typed evidence families for:
  - declaration
  - admission
  - graph
  - aspects
  - obligations
- stable indexes for:
  - declaration identity -> evidence sets
  - source span -> declaration / admission evidence
  - graph node identity -> obligations / declaration / admission evidence
  - published aspect -> publishing nodes / receipts
  - consumed aspect -> dependent nodes / obligations / receipts
- the first formal AI harness path for targeted inspection by identity,
  source span, and scope before screenshot support exists

**Acceptance evidence**

- an agent can inspect a declaration artifact without receiving a giant dump
- an agent can inspect a graph node by identity and request only aspect-local
  evidence
- an agent can request `evidence_refs` before `materialized_detail`
- the 3.1 inspection facade can now answer real runtime questions instead of
  just carrying contract shape
- inspection responses emit typed receipts rather than strings or logs
- no renderer-local debug helper is required to explain declaration, admission,
  graph, or aspect posture

### Milestone 3.6a: Measurement Vocabulary, Basis Admission, and Host Evidence Boundaries

This slice closes the semantic and authority side of measurement before
allocation-neighborhood planning, committed allocation receipts, and
churn-heavy resize/scroll/drag behavior broaden the runtime.

**Must ship**

- `UiMeasurementRequest`
- `UiMeasurementResult`
- measurement vocabulary for:
  - `available-space`
  - `fixed`
  - `hug`
  - `fill`
  - `equal-share`
  - `min`
  - `max`
  - `bounded`
  - `content-measured`
  - `viewport-relative`
  - `scroll-owned`
  - `portal-anchored`
- typed host measurement-evidence exchange for intrinsic measurement, text
  measurement, viewport facts, and native sizing observations
- deterministic measurement-basis assembly from declaration posture, Query
  basis/facts, world posture, host capability posture, and host evidence

**Acceptance evidence**

- host supplies measurement evidence only; it does not decide layout meaning
- intrinsic measurement stays evidence and does not become host-owned or
  cache-owned layout truth
- equivalent declaration + graph + measurement evidence inputs converge to the
  same measurement basis
- unsupported, cyclic, or not-yet-admitted measurement modes deny through typed
  posture instead of heuristic fallback

### Milestone 3.6b: Allocation Neighborhood Planning And Constraint Propagation

Detailed spec: [milestone-3.6b.md](./milestone-3.6b.md)

This slice closes the planning kernel that consumes the 3.6a measurement basis
before committed allocation receipts and continuous interaction churn broaden
the runtime.

**Must ship**

- `UiAllocationPlan`
- allocation-neighborhood planning semantics for parent/child constraint flow
  and sibling negotiation
- explicit constraint-propagation rules for:
  - parent-to-child available-space
  - child intrinsic contribution
  - sibling negotiation
  - equal-share distribution
  - bounded min/max reconciliation
  - viewport-relative basis use
  - portal-anchor planning inputs
  - scroll-owner planning inputs
- planning-time denial posture for contradictory or cyclic allocation
  neighborhoods

**Acceptance evidence**

- equivalent declaration + graph + measurement basis inputs converge to the
  same allocation plan
- plan assembly does not rediscover declaration or Query meaning from source
  text, renderer code, or host helpers
- unsupported or contradictory allocation neighborhoods deny before committed
  receipt production
- local plan changes can identify a typed affected neighborhood instead of
  widening immediately to whole-page planning

### Milestone 3.7: Runtime Topology And Proof-Flow Cleanup Gate — **Closed**

This slice is a structural cleanup gate, not a product-capability expansion
milestone. It hardened the shipped 3.1 through 3.6b runtime surfaces before
allocation receipts, execution-plan lowering, and churn-heavy interaction
measurement could stack on broad facades, helper swamps, and topology ambiguity.

**Spec:** [`milestone-3.7.md`](./milestone-3.7.md) (Status: Closed; includes
structural closeout bundle and 3.8 start-here capabilities).

**Shipped (structural)**

- cleanup inventory covering runtime, inspection, query binding, and
  certification structural residue (critical findings cleared)
- public-surface cleanup so lifecycle and authority order are visible at the
  facade rather than hidden behind alphabet soup re-exports
- runtime tree cleanup that makes planning, activation, reconciliation, handle
  allocation, and host observation intake visible lanes
- evidence and proof-flow cleanup so planning, allocation, diagnostics, and
  certification consume named transition families rather than vocabulary
  warehouses
- function decomposition for identity match, measurement inspection projection,
  and constraint admission pipeline
- compile-fail and structural-scan enforcement for sealed construction
  boundaries, narrowed facades, and anti-bypass expectations
- certification/test cleanup: production authority remains distinct from
  SUPPORT AUTHORITY (`worth-ui-test-support` + feature-gated fixtures)

**Acceptance evidence**

- public callers can follow one stable lifecycle path without deep imports into
  internal runtime topology
- later milestones consume the refactored surfaces without reopening facade or
  topology shape (see 3.8 consumption rules below)
- structural scans and compile-fail coverage catch broad-facade, helper, and
  authority-bypass regressions

### Milestone 3.8: Allocation Receipts, Incremental Replanning, Scroll, Portal, And Continuous Interaction Measurement

Detailed spec: [milestone-3.8.md](./milestone-3.8.md)

**Status: Closed.** Phases 1-22 have landed. The post-Query-9.13 cutover closes
installed-domain composition, declarative binding, native projection,
identity, facade, and topology authority; the final phases close local
inspection, freshness, counters, typed denials, hostile runtime integration,
and anti-bypass certification.

This slice closes committed allocation truth and churn-heavy measurement
behavior after 3.6a and 3.6b planning semantics already exist **and after 3.7
cleaned the runtime topology**. It also owns the runtime frame-dispatch
boundary that turns admitted continuous-source facts into ordinary allocation
transitions; stream policy without that dispatcher is incomplete.

**Prerequisite consumption (non-reopen)**

3.8 must start from cleaned 3.7 capabilities, not raw internals:

- planning: `plan_allocation` after admitted measurement basis + neighborhood +
  constraint pipeline
- replacement continuity: identity match report after impact narrow
- measurement: host observation → admit measurement basis → inspection projectors
- public facade lifecycle order; no deep-import pressure into private runtime
  modules
- fixtures only via SUPPORT AUTHORITY (`worth-ui-test-support`); not production
  test mint paths

3.8 must **not** reintroduce broad root facades, same-level dumping-ground
trees, helper-only authority, host-owned UI semantics, or certification-as-law.

**Must ship**

- one runtime-installed Worth UI Query domain composition path and one
  declarative binding definition; Query-free apps remain ceremony-free
- `worth-ui-query-binding` as the enforced production Query/UI semantic edge,
  with Worth UI runtime consuming binding-owned admitted artifacts rather than
  Query topology directly
- Foundational-native Query projection values carried into measurement and
  allocation without local scalar, JSON, bit-pattern, or digest authority
- deletion of consumer-local Query mirrors, manual Query identity hashing,
  deprecated phase aliases, invalid public pseudo-Query constructors, and
  bucket-shaped allocation topology
- `UiAllocationReceipt`
- allocation equivalence and reuse basis
- runtime-owned frame epochs, sealed admitted stream frames, and one ordinary
  dispatcher from source-to-frame gateways through policy resolution and typed
  allocation transition outcome
- invalidation and affected-neighborhood replanning rules for:
  - viewport resize
  - local resize and splitter drag
  - content growth
  - scroll-owned extent changes
  - portal-anchor changes
- committed runtime semantics for:
  - `viewport-relative`
  - `scroll-owned`
  - `portal-anchored`
- measurement inspection evidence and allocation closeout receipts strong enough
  for later mounting and rebind milestones

**Acceptance evidence**

- an external consumer can install the Worth UI Query domain, derive and
  register a measurement binding, consume native projection facts, enter the
  allocation frame through one binding-owned settlement, and inspect the
  result using only admitted facades
- direct runtime Query dependencies, raw Query facade re-exports, copied digest
  authority, and foreign or stale installed handles fail compiler, boundary, or
  hostile certification checks before allocation work begins
- the ordinary API is shorter than manual assembly: callers do not separately
  wire Query capability, result shape, basis posture, live compatibility, or
  composition digests
- viewport resize enters as host observation, is admitted into a runtime-owned
  stream frame, and replans only the affected allocation neighborhood
- mosaic resize and local composition allocation use the same measurement lane
- scroll-owned and portal-anchored measurement remain runtime-owned rather than
  adapter-owned
- continuous resize and drag pressure stay bounded without broad unrelated
  replanning
- certification replay of the same admitted source facts produces the same
  sealed frame and policy/transition outcome; late ingress, duplicate dispatch,
  and overflow deny through typed dispatcher evidence
- new receipts attach to verified planning/measurement transitions, not host
  folklore or reconstructed booleans

### Milestone 3.8.1: Test-Program Topology and Runtime Authority Closure Gate

Detailed spec: [milestone-3.8.1.md](./milestone-3.8.1.md)

**Status: Structural and test-topology closure complete. A later hostile review
found that filesystem/watcher certification exercised injected source and
manufactured events rather than the production mechanisms named by the
contract. Milestone 3.9 Phase 2 has now closed that corrective gate with real
filesystem ingress and production watcher evidence; execution-plan work is
unblocked only through Milestone 3.9's sealed Phase-3 authority chain.**

This closure slice first repairs the test program's compilation, linking,
fixture, and nested-build topology so repeated hostile certification is an
ordinary affordable workflow. It then binds the artifact, declarations,
committed graph, Query binding, capability snapshot, host session, runtime
execution, and inspection surfaces into one application generation before
execution-plan lowering builds on the 3.8 substrate.

**Must ship**

- a proof-preserving test-topology cutover before production-authority work:
  batched compile contracts, bounded integration targets, compiled-once named
  scenario authority, no ordinary nested Cargo builds, explicit fast/full CI
  lanes, a proof-equivalent parallel premerge aggregate, pinned and
  cache-observable CI execution, and measured cold/warm improvement
- one sealed prepared application authority and one active application session
  that cannot be split into independently launchable runtime and inspection
  truth
- inseparable file- and Rust-authored candidate composition through
  preparation and atomic replacement cutover
- typed fallible freeze/preparation with no panic for publicly constructible
  invalid declarations or graph topology
- operational host-session authority; arbitrary per-turn adapter substitution
  is mechanically unavailable
- proof-bearing framework-turn transition planning and policy-family execution
  beneath one thin close/pump owner
- deletion of unreachable GPU/theme source and a mechanical production-source
  reachability gate
- hostile end-to-end and anti-bypass certification across application, Query,
  host, allocation, replacement, and inspection generations

**Must preserve**

- canonical artifact meaning, Query-owned binding authority, runtime-owned UI
  semantics, and host adapters limited to native mechanics
- invalid replacement preservation of the last complete active generation
- delta-bounded allocation work with explicit structural counters
- Query-free and headless paths without unnecessary ceremony
- no compatibility lane retaining artifact-only launch, split source ingress,
  decorative host configuration, or infallible public freeze

**Acceptance evidence**

- every pre-cutover test maps to a retained or stronger proof, test-target and
  compiler-session budgets are mechanically enforced, and isolated warm/cold
  measurements meet the milestone's relative performance gates
- mixed artifact/declaration/graph/Query/host/application generations are
  unrepresentable or deny before mutation even when capability digests match
- active runtime and active inspection report the same generation through
  launch, ordinary frames, valid replacement, no-op replacement, and denied
  replacement
- public invalid input returns typed denial without unwind or partial authority
  publication
- framework-turn planning is mutation-free, execution is transactional, and
  close/pump ownership remains exactly once
- every production Rust source file participates in a declared compiled module
  graph and all workspace quality/boundary/certification gates are green

### Milestone 3.9: Execution-Plan Lowering, Equivalence, and Frame-Cost Surfaces

Detailed spec: [milestone-3.9.md](./milestone-3.9.md)

**Status: Closed. Phases 1 through 18 are closed.**

The hostile reopening of Phases 9 through 13 is resolved with public real-
filesystem hook/resource-generation replacement proofs, public large-canvas
activation, the closed cross-lane bundle, executable field matrices, equal-
digest hostility, real Query no-op lifetime, and late interruption coverage.
Phase 14 now has persistent regional plan and allocation-catalog storage,
stable slots, lane-local successor sealing, regional Query succession, and a
candidate-owned allocation delta that derives the exact affected predecessor
closure. Unaffected rows remain complete active truth through structural
sharing; graph, invalidation, scroll, portal, Query, host, and durable indexes
change incrementally from the same proof. Public carry and removal-only
cutovers, scaled real-catalog storage retention, delta-local counters, and
sustained churn close the allocation-locality gate. Phase 15 now binds the
successor application into the same infallible publication as plan,
allocation, state, scheduler, Query, host, planning, and inspection truth.
Frame boundaries carry exact session authority, retryable denials retain the
candidate explicitly, and public receipts plus real Query lifecycle tests
prove coherent cutover and cleanup. Phase 16 binds final reload/frame counters
to independent scale and allocator evidence. Phase 17 curates named `app` and
`runtime` facades, compact inspection, and final lifecycle documentation.
Phase 18 removes predecessor and test-only authority paths and closes real
filesystem/watcher, Query, egui, headless, allocator, compile-contract,
topology, boundary, line-cap, and quality proof. Comparable closing timing
evidence records reviewed cost amendments while keeping real-mechanism waits
and storms outside the ordinary fast lane.

This slice begins only after Milestone 3.8.1 has closed the application,
replacement, host-session, and framework-turn composition boundaries.
Its first production gate closes the discovered filesystem/watcher reality gap:
external tests write actual `.wui` files, while production Worth UI reads,
watches, snapshots, and lowers them through the public application lifecycle.

This slice ensures execution consumes lowered plans instead of reconstructing
strategy from graph or declaration artifacts every frame. It consolidates the
existing provisional plan, handle, lane, equivalence, and counter surfaces into
one active-session-owned lifecycle; it does not add a parallel planner. The
regional storage/equivalence/reclamation kernel is established before lane
representations broaden so bounded replacement is foundational rather than a
late optimization.

**Must ship**

- execution-plan lowering from one sealed application + graph + capability/
  Query + host-support + committed-allocation authority into active runtime
  plans
- a distinct sealed pre-allocation projection that carries only allocation-
  relevant candidate facts and cannot be reconstructed into execution-plan
  input; committed allocation must join back to exact candidate authority before
  the sole execution-plan lowerer can run
- real filesystem source acquisition and operating-system watcher/debounce proof
  using actual temporary `.wui` writes, atomic renames, removals, malformed
  edits, and ordered frozen package snapshots; Worth UI never authors the files
- one collision-safe regional plan storage/equivalence kernel carrying exact
  predecessor proof, complete-successor semantics, bounded reuse/retirement,
  and `O(P)` reclamation under churn
- binding-owned Query projection/installed authority carried as one sealed
  constituent, with native aspect values preserved and no UI-local basis,
  status, fact, or lifecycle mirrors
- lifecycle-typed installed snapshot and live views, plus atomic affine
  admission of each managed live resource with its consumed projection;
  Phase 8 owns candidate-only cleanup before Phase 15 re-proves the final
  cross-family transaction
- virtualized-data plan/handle/visible-range execution substrate for already-
  admitted view references, without pulling general collection projection,
  cursor ownership, result-state, or live collection-patch semantics forward
  from Milestones 3.13 and 6
- equivalence and no-op classification for candidate replacements
- compact runtime handles for commands, components, children, tokens, and view
  bindings
- active-session-owned plan execution with no caller-built or caller-submitted
  lane plans
- named counters for reload/lowering work and steady-frame execution work
- WORTH Foundational performance-envelope integration for shared claim
  vocabulary and certified claim bundles
- one minimal sealed host-output envelope that keeps adapters away from owned
  plan/artifact authority and is extended into complete mounted receipts by
  Milestone 3.10
- phase-local compact evidence, relevance indexes, and budgeted inspection for
  lowering, regions, handles, lanes, equivalence, activation, and frame cost
- frozen compiler-session/target/case budgets and compiled-once adversarial
  scenarios that preserve iteration time
- real filesystem/watcher, public lifecycle, Query Consumer Kit, egui/headless,
  and allocator evidence placed as child modules of the existing
  `worth-ui-certification` `application_contracts` target; compiler proof uses
  the frozen checked-in two-invocation owner, with no generated/per-phase
  fixture workspace, additional compiler session, or fast-lane OS wait

**Acceptance evidence**

- identical canonical artifacts produce equivalent execution plans where the
  lane and capability set are unchanged
- equivalent replacements avoid needless plan swaps
- steady-state frame execution proves source parsing, artifact validation,
  registry string lookup, and broad artifact scans remain absent
- initial lowering is bounded by plan width, replacement lowering by admitted
  affected scope, and frame execution by pre-admitted target breadth with
  actual touched rows no greater than requested rows
- Query-backed rows retain runtime-affine authority and Query-owned live-resource
  lifecycle; UI activation coordinates succession and exact-once release but
  cannot recreate Query authority
- file-authored acceptance starts from bytes written to disk and observed by the
  production filesystem/watcher adapters; `with_file` injection and handcrafted
  watcher events cannot certify that claim
- compiler proof runs through one checked-in owner in exactly two Cargo
  invocations (negative and positive), reuses the caller's ordinary target
  graph, and adds no private duplicate build, generated fixture workspace, or
  ordinary-test nested Cargo invocation
- predecessor and headless paths consume the same sealed host-neutral envelope meaning;
  at least one real predecessor-adapter frame and production headless frame
  exercise their adapters, and neither adapter can receive an owned plan or
  choose lane/UI semantics
- the zero-allocation executor claim is checked by an independent thread-scoped
  allocator observer as well as Worth UI counters

### Milestone 3.9.1: Query 9.14 Consumer-Path Modernization

Detailed spec: [milestone-3.9.1.md](./milestone-3.9.1.md)

Status: Closed on 2026-07-21. Exact-source closing evidence records a 0.462 s
targeted warm median, 8.070 s warm fast-lane median, and 80.738 s isolated cold
compile-contract median. Query 9.14 has since satisfied the bounded
managed-live seam's Phase 17/19/23/24 exit trigger. The dated amendment in the
detailed spec preserves the historical boundary and routes the resulting
cutover to Milestone 3.9.2.

This corrective modernization slice follows the closed 3.9 execution-plan
lifecycle and precedes mounted receipts, hot rebind, broader projection
consumption, and product Query surfaces. Query Milestone 9.14 Phases 1 through
14 now provide installed operation semantic closure, one operating-world root,
Query-minted consumer support, typed progression, settled projections, and
exact identity that were not available when the older Worth UI binding path was
formed.

**Must ship**

- a search-seeded, manually adjudicated subsystem inventory whose rows classify
  each Query-facing seam as replace, retain, transitional, diagnostic-only, or
  unrelated
- a manually adjudicated boundary-edge matrix naming producer, consumer,
  cardinality, lifetime, failure owner, cost, dependency direction, and the
  forbidden shortcut for every authority crossing
- typed installed Worth UI Query operations whose stable meaning is declared
  once and whose volatile executors register separately
- one binding-owned gateway that borrows Query's installed operating world per
  replacement-admission or fact-refresh attempt; no operating-world value is
  stored in active or frame state
- exactly one Query consumer contract minted and consumed per bound execution;
  the operating world and contract remain control-plane phase values while the
  settled projection is retained once in generation-owned storage
- compact generation-scoped UI fact references fan retained settlement proof
  into planning and frame ingress without copying heavyweight authority into
  each plan row
- application replacement publishes binding topology through the existing 3.9
  application transaction, while in-generation Query refresh replaces one
  complete settlement/fact slot through the existing framework-turn source
  transaction
- removal of UI-local Query support, basis, settlement, and operational digest
  authority while retaining legitimate UI-owned binding, invalidation,
  allocation, and presentation derivations
- exact installed-domain authority plus Query-minted binding identity and
  currentness plus UI-meaning replacement equivalence, published only through
  the complete 3.9 application transaction
- an explicitly bounded managed-live compatibility seam until Query 9.14
  Phases 17, 19, 23, and 24 provide the required public ordinary lifecycle
- real public application and Consumer Kit certification consolidated into the
  existing compiled scenario owners

**Must preserve**

- all closed Milestone 3.9 application, execution-plan, allocation-locality,
  filesystem-ingress, atomic-publication, host, cleanup, and frame-cost truths
- Query-free application construction and execution without dummy Query
  ceremony
- Query-owned stop, warning, result-state, counter, and authority topology
  through every UI projection
- UI-owned authored binding, result-shape, denial-presentation, dependency-
  impact, allocation, and inspection meaning
- the existing compiler-session and integration-target budgets

**Acceptance evidence**

- the manually resolved inventory has no ambiguous row and every transitional
  row has a named exit trigger; no test claims migration success from a clean
  grep or removed token
- one real `.wui` application journey installs, binds, executes, publishes,
  consumes, settles, lowers, activates, frames, inspects, replaces, and cleans
  up through public facades
- equal local representations cannot hide different Query authority, and exact
  Query authority cannot hide different UI meaning
- wrong-world, stale, mixed-receipt, unsupported, partial, interrupted, and
  live-compatibility cases deny at their owning boundary while prior active
  truth remains complete
- boundary, agent-context, line-cap, workspace quality, and existing compile-
  contract gates pass without a new nested build or test-target explosion

### Milestone 3.9.2: Query 9.14 Native Consumer and Identity Cutover

Detailed spec: [milestone-3.9.2.md](./milestone-3.9.2.md)

Status: Complete on 2026-07-23. Phases 1 through 7 are closed.

This final pre-mount modernization slice follows Query 9.14 completion and
precedes mounted receipts. It replaces Worth UI's remaining projection scans,
positional native-fact copies, and printable Query identity decisions with the
completed declaration-indexed native-access and owner-specific identity
contracts. It also narrows the already-correct operation-native patch path into
one sealed UI consequence boundary for later rebind and projection work.

**Must ship**

- a search-seeded, manually adjudicated subsystem inventory and boundary-edge
  matrix; no grep-based migration sentinel
- exact consumer-contract-derived native selections and
  `WorthQueryNativeAccessKey` values carried through `consume_bound(...)`
- `O(1)` settled and refreshed native access per admitted key and row, with no
  whole-projection scan, positional copy, or local selector authority
- separate sealed UI-local binding-authority and settlement references minted
  only after the Query binding owner validates its exact retained source
- graph, eligibility, fact, touch, allocation, replacement, planning, and
  execution decisions free of printable Query identity authority
- one retained Query settlement and one complete UI measurement consequence per
  source revision, fanned out through compact generation-scoped references
- the real Query live-owner, lease, invalidation-delta, compiled-impact,
  collection-window, and patch path translated exactly once into a sealed
  UI-owned change consequence without local Query-impact reconstruction
- exact Query and UI counters plus real public lifecycle certification inside
  existing compiled owners
- a narrow facade and updated 3.9.1/3.10/3.12/3.13 documentary seam

**Must preserve**

- all closed 3.9 and 3.9.1 application, plan, allocation, filesystem,
  replacement, host, cleanup, and frame-cost truths
- Query-owned native access, operational identity, compatibility, lifecycle,
  invalidation, row, ordering, window, patch, denial, and counter meaning
- UI-owned authored binding, graph, measurement, source-coordinate, allocation,
  dependency, invalidation-consequence, and presentation meaning
- Query-free applications without dummy Query ceremony or cost
- prior active truth on every stale input, denial, reset requirement, or
  interrupted replacement
- `worth-ui-query-binding` as the sole production Query importer
- 3.10 ownership of mounted receipts and host observations, 3.12 ownership of
  changed-fact/rebind planning, and 3.13 ownership of the broad projection
  product substrate
- existing compile-session and integration-target budgets

**Acceptance evidence**

- identical Query labels and native bytes from different runtimes,
  installations, capabilities, generations, settlements, windows, or leases
  cannot alias in any operational UI decision
- adding unrelated projected fields does not increase the work for `k` declared
  UI facts: Query reports exactly the admitted keyed accesses and zero fact,
  row, or path scans
- settlement refresh atomically replaces one UI consequence revision and
  rejects predecessor fact, touch, and allocation evidence while stable
  binding-slot plan links resolve the current consequence without relowering
- one real operation-native patch produces one sealed UI consequence; stale,
  foreign, duplicate, out-of-order, reset-required, and stopped-disposal cases
  preserve prior truth at their owning boundary
- Query artifacts and identity representations cannot reach host authority,
  future mounted receipts, or downstream Query re-entry
- boundary, agent-context, line-cap, format, clippy, workspace, compile-
  contract, and certification gates pass without new build topology

**Permanent 3.x platform-pulse and executable-world contract**

Milestones 3.10 through 3.23 carry one deliberately bounded, file-authored
Platform Pulse Page forward as cumulative product evidence. Milestone 3.10.2
supplies the retroactive visible seed; Milestone 3.10.3 supplies the permanent
executable-world foundation. Both gates close before 3.11 begins. The pulse is
one permanent downstream product exercise, not a disposable sample, a
replacement for each milestone's adversarial courtroom, or a new test topology
per milestone. It begins as the smallest honest page and graduates in 3.15 to
a 960-by-600 designed product workbench. It stays one bounded continuing
surface on which a human and an external product-world runner can see that
newly closed architecture has reached a coherent real product.

- The pulse starts from actual `.wui` bytes in an isolated filesystem
  workspace, enters through the production filesystem reader and watcher,
  crosses the public application lifecycle, and presents through the canonical
  mounted host contract. Deterministic integration observation uses the
  production headless adapter; native-visible product claims additionally
  launch the exact Cargo-built pulse binary through the required
  executable-world lane.
- The page keeps one stable logical scenario identity across milestones.
  Milestones extend its authored source and evidence instead of replacing it
  with a milestone-local fixture, screenshot, mock-data page, or inspector-only
  reenactment.
- Every milestone adds one bounded visible behavior and proves both sides of
  it: a human can run the pulse and see the promised change, and independent
  receipts or inspection evidence trace that change through the milestone's
  real authority boundary.
- Pulse requirements are cumulative. A later milestone must preserve the
  earlier visible behaviors and their identity/evidence chain unless it
  explicitly specifies a migration.
- Injected source, forged watcher events, handwritten adapter transcripts,
  renderer-local meaning, fake Query values, certification-replay-only output,
  or screenshots detached from frame identity cannot satisfy the pulse.
- Automated pulse proof has two explicit owners. The existing consolidated
  application-contract target owns fast in-process integration. The pulse
  package's sole `executable_world` target owns real product entry, native
  process/window evidence, external actions, and teardown. Human use and
  executable certification run exactly one permanent downstream pulse binary.
  Later milestones extend both evidence lanes inside their existing targets;
  they must not create another binary, executable-world target, composition
  root, or bespoke harness per milestone.
- `workspaces/worth-ui/docs/application-lifecycle.md` is the continuing human
  run authority. Each milestone must revise its `Platform Pulse` section with
  the exact launch command, the one user action or source edit, the visible
  result, the receipt or inspection handle that proves the result came from the
  real boundary, and the executable-world command that exercises the same
  product entry. If those workflows cannot be stated and run, the pulse
  requirement is not closed.
- A green pulse cannot compensate for failed local, hostile, compile-boundary,
  cost, or authority evidence. Its purpose is to catch the opposite dishonesty:
  locally correct architecture that never produces a coherent visible product.
- The executable world is cumulative. Later milestones add typed world deltas,
  actions, observations, and adjudications to the existing progression. By the
  milestone after 3.23, product launch, action, observation, identity
  correlation, failure-artifact retention, and teardown are mature
  infrastructure; that milestone may not become the first honest product entry.

### Milestone 3.10: Mounted Receipts and Host Contract

Detailed spec: [milestone-3.10.md](./milestone-3.10.md)

Status: Complete (2026-07-25). Milestone 3.10.2 subsequently closed the
human-visible pulse seed. Milestone 3.10.3 owns the later-discovered
executable-world evidence correction and blocks 3.11.

This slice closes the host boundary: host code may render and observe, but may
not own visible UI meaning. It extends/refines Milestone 3.9's minimal sealed
host-output envelope into complete mounted-node and mounted-frame receipts; it
must not introduce a parallel runtime-to-host output path or reopen active-plan
ownership. It consumes only UI-owned graph, measurement, allocation, and
presentation meaning admitted through 3.9.2; no Query key, settlement, patch,
row handle, operational identity representation, or change consequence becomes
a mounted or host authority artifact. One prepared frame is complete across all
participating execution lanes and surface bindings before effects begin;
runtime publication is atomic, while native multi-surface effects retain honest
rejection-before-effects, in-flight, complete-presentation, and indeterminate
outcomes. Application replacement does not become current until complete
presentation is followed by the matching infallible runtime publication.

**Platform pulse**

Post-completion obligation: seed the Platform Pulse Page from real `.wui` bytes
and mount, execute, and visibly render it through the public application
lifecycle and canonical host contract. Milestone 3.10.2 closed that product
capability; Milestone 3.10.3 closes required automation through the actual
product binary and native event loop. This paragraph is not a claim that the
original 3.10 close produced native shapes or executable-world evidence.
Machine evidence must bind the exact filesystem snapshot, published
mounted-frame receipt, complete static-paint mechanics, post-translation
headless or egui observation, and the externally observed product process where
the claim is native-visible. Injected source, a fake watcher, direct preview
paint, an in-process reenactment, or a host stub that does no adapter work
cannot satisfy executable product entry.

**Must ship**

- `UiMountedNodeReceipt`
- `UiMountedFrameReceipt`
- mounted-instance identity distinct from graph-node identity, including
  zero-to-many graph-node cardinality and remount incarnation
- separate semantic-surface, host-surface, surface-binding-generation,
  mounted-instance, and frame-scoped receipt identities
- typed known-empty host-surface registration baselines for honest first-frame
  recovery
- one cross-lane, multi-surface prepared-frame and publication lifecycle
- mounted receipt facts for paint intent, clip/layer, allocation box, input
  participation, focus participation, hit-test participation, accessibility,
  motion projection, and diagnostic projection
- specialized compact storage for canvas, realtime, and other high-volume
  batches rather than one generic receipt per primitive
- typed rejection-before-effects, bounded in-flight, complete-presentation,
  publication, unchanged-reuse, and presentation-indeterminate outcomes
- explicit reconciliation that restores known host presentation truth without
  pretending native rollback
- replacement publication, mounted indexes, and bounded retention tied to the
  exact application, graph world, plan, allocation, host session, surface set,
  and frame generation
- host contract for viewport, pointer, keyboard, text/IME, focus, scroll,
  time/tick, and text-measurement observations
- bounded, non-reentrant, generation-aware raw and structurally validated host
  report batches; Milestone 3.12 retains ownership of semantic observation
  admission and rebind planning
- real egui translation and an honest headless recorder through the same sealed
  host contract
- egui executes only effect families completely defined by mounted receipts;
  count-only or appearance-incomplete paint is rejected before effects rather
  than synthesized, while Milestone 3.16 retains visual appearance ownership
- independent authored, deterministic model, headless-transcript, egui-native,
  zero-effect-denial, and public-publication oracles inside the existing
  consolidated test owners; scripted host fault injection remains protocol
  evidence rather than a claim of real native effects
- explicit host-protocol and per-artifact schema identity, version negotiation,
  compatibility windows, and reject-before-effects posture
- named initial, delta, surface-specific, adapter, unchanged, retained,
  rejected, and observation cost evidence

**Must preserve**

- one canonical runtime-to-host presentation path; the minimal envelope,
  lane-local presentation, and direct preview-paint paths do not survive as
  compatibility lanes
- host ownership of native mechanics without host ownership of visibility,
  enabledness, validity, layout, semantic focus, hit testing, accessibility,
  motion, diagnostics, application lifecycle, or Query meaning
- predecessor complete truth on every denial before native effects and an
  explicit distinction between current runtime truth and known host
  presentation truth when effects become indeterminate
- Query-free applications without dummy Query ceremony or Query-derived cost
- existing consolidated integration targets, two-session compile contracts,
  ordinary fast-lane posture, zero flake retries, and no test-only runtime path
- Milestone 3.11 ownership of visual snapshots, Milestone 3.12 ownership of
  semantic observation admission, Milestone 3.13 ownership of broad Query
  projection, and Milestones 3.14 through 3.16 ownership of intent, services,
  and appearance

**Acceptance evidence**

- one real framework turn combining ordinary, virtualized, canvas, and
  realtime work across multiple surfaces reaches only complete presentation,
  rejection before effects, bounded in-flight work, or an explicit blocked
  indeterminate outcome
- egui consumes only mounted receipts and reports only native mechanics it
  actually executed through the bounded report boundary; incomplete paint
  produces typed denial and no synthetic shapes
- reorder preserves a live semantic mounted instance, while actual
  unmount/remount creates a distinct incarnation
- failed replacement or pre-effect host denial preserves one coherent
  predecessor application/plan/allocation/mounted-frame truth
- retained-predecessor reports remain distinguishable from foreign,
  never-presented, duplicate, reordered, overflowing, reentrant, or
  indeterminate-basis reports, and none mutates semantic UI truth in 3.10
- deterministic identity, presentation, publication, and report traces agree
  with an independent model, while real filesystem, watcher, headless, and egui
  scenarios separately prove their production boundaries without test-only
  runtime paths or retries
- steady projection work is bounded by changed semantic instances, affected
  indexes, honest batch granules, and affected surface-specific work; unchanged
  work is constant only through an exact carried reuse witness
- host cannot receive authored declarations directly
- host cannot decide visible/disabled/valid/layout meaning or recover Query,
  graph, plan, allocation, or publication authority from its sealed view

### Milestone 3.10.1: DSL Ownership, Runtime Subsystem Boundaries, and Facade Closure

Status: Complete (2026-07-25)

Detailed spec: [milestone-3.10.1.md](./milestone-3.10.1.md)

This corrective gate follows mounted-receipt and host-contract closure and
precedes visual snapshots. It aligns physical and public topology with the
already-declared architecture before later milestones harden the wrong source
owner, broaden a runtime macro-boundary, or consume transitional phase APIs.

**Platform pulse**

Post-completion verification obligation: Milestone 3.10.2 routed the
file-authored pulse through the DSL-owned sealed handoff and its mounted
execution through the condensed public facade. Milestone 3.10.3 must prove the
same route from the exact product binary rather than an in-process shell
reenactment. Milestone 3.10.1 itself added no pulse feature and did not prove
native-visible rendering or executable product entry. The complete catch-up
must show that no loose-source path, public midpoint, compatibility wrapper, or
test-only constructor is needed.

**Must ship**

- `worth-ui-dsl` as the sole production owner of authored syntax, source AST,
  language legality, source diagnostics, semantic normalization, and
  authored-to-canonical lowering
- one sealed semantic handoff shared by file-authored and Rust-authored
  composition before runtime preparation
- explicit runtime subsystem state, transition, failure, cost, dependency, and
  future-insertion contracts; crate splits only where autonomous authority and
  lifecycle justify them
- one ordinary product lifecycle centered on `execute_mounted_frame(...)`,
  with advanced continuation or recovery reachable only through typed handles
  returned by that path
- removal of loose-source runtime preparation, public midpoint lifecycle
  entry, broad intermediate re-exports, and predecessor compatibility routes
- mechanical DSL/runtime ownership, runtime-topology, public-surface,
  callable-entry, reachability, line-cap, and compile-contract enforcement
- real filesystem, Rust-authored, headless, egui, replacement, denial,
  publication, inspection, allocation, and build-cost certification through the
  condensed public path

**Must preserve**

- all Milestone 3.10 mounted identity, receipt, host-mechanics, effect,
  publication, reconciliation, predecessor-preservation, and cost semantics
- runtime ownership of active application, graph, planning, allocation,
  execution, mounted publication, and operational inspection truth
- filesystem/watcher mechanics with their mechanism owner rather than the DSL
- Query-free applications without dummy Query or advanced-lifecycle ceremony
- one canonical runtime-to-host presentation path and the existing
  consolidated test/compile topology
- Milestone 3.11 ownership of visual snapshots, Milestone 3.12 ownership of
  semantic observation admission, and Milestones 3.17 and 3.18 ownership of
  new language features

**Acceptance evidence**

- runtime contains no production parser, source AST, authored-language legality
  owner, semantic source lowerer, or callable facade presenting those powers
- equivalent real-file and Rust-authored applications cross one DSL-owned
  sealed handoff and enter one runtime preparation path
- ordinary downstream code can execute and inspect the complete mounted
  lifecycle without importing or constructing intermediate runtime phases
- predecessor aliases, forwarding wrappers, feature-hidden routes, and
  support-only production constructors fail mechanical and hostile compile
  checks
- future snapshot, rebind, expression, and module changes each have one
  unambiguous insertion owner without reverse dependencies
- poisoned DSL paths remain untouched during steady frames, and comparable
  allocator/counter/build evidence proves the migration added no hidden source
  work or build-topology expansion

### Milestone 3.10.2: Platform Pulse Seed and Visible Lifecycle Closure

Status: Product capability complete. A later evidence audit found that the
automated pulse worlds stop below the executable composition root. Milestone
3.10.3 owns that corrective proof gate and now blocks Milestone 3.11.

Detailed spec: [milestone-3.10.2.md](./milestone-3.10.2.md)

This successor gate catches the human-visible requirement adopted after 3.10
and 3.10.1 closed. It preserves their architectural truth while filling one
concrete product gap: the current real egui lifecycle intentionally accepts
only a no-effect frame and emits no native shapes because mounted paint rows
carry counts rather than complete static-paint mechanics.

**Platform pulse**

Check in one small `.wui` page and one permanent downstream pulse executable.
A human launches that page through the production filesystem reader and public
application lifecycle and sees at least one runtime-defined filled rectangle
presented through the canonical mounted host contract and real egui adapter.
Changing the page's admitted static color through a real file replacement must
visibly publish one coherent successor while an invalid edit preserves the last
admitted visible page. This replacement proof does not claim Milestone 3.12's
bounded semantic rebind.

**Must ship**

- one checked-in canonical pulse source and stable scenario identity
- one permanent `worth-ui-platform-pulse` downstream application package that
  imports only public product facades and the egui host adapter
- one minimal complete static filled-rectangle contract joining admitted
  authored color, committed allocation, mounted identity, layer, and frame
  authority before effects
- egui translation of that complete mechanic into real native shapes, with no
  adapter-selected color, geometry, visibility, or fallback appearance
- production headless observation of the same complete mechanic and real egui
  observation of the native consequence
- production filesystem/watcher replacement and malformed-edit predecessor
  preservation through the existing application transaction
- exact human launch/edit/visible-result/evidence instructions in the existing
  application lifecycle documentation
- consolidated adversarial and cost proof in the existing
  `application_contracts` target

**Must preserve**

- all completed 3.10 mounted identity, receipt, effect, publication,
  reconciliation, replacement, and cost guarantees
- all completed 3.10.1 DSL ownership, runtime subsystem, facade, and topology
  guarantees
- runtime ownership of complete paint meaning and host ownership of native
  mechanical translation only
- 3.11 ownership of pixel-to-mounted identity, 3.12 ownership of bounded hot
  rebind, and 3.16 ownership of appearance roles, state axes, theme switching,
  and rich component styling
- one pulse executable for all later milestones, the existing consolidated
  in-process proof owner, and the one successor executable-world owner added by
  3.10.3

**Acceptance evidence**

- a clean checkout can run the documented command and show a non-empty egui
  frame whose shapes derive only from the published mounted projection
- exact source snapshot identity, mounted frame identity, paint primitive
  identity, adapter observation, and native shape counts agree without sharing
  an expected-value generator
- a real watched valid edit changes the visible admitted color only after the
  complete successor publishes; a malformed edit leaves predecessor pixels and
  publication current with a typed source denial
- removing the authored color, committed allocation, paint payload, native
  paint capability, or canonical frame call makes the courtroom red at the
  owning boundary
- direct egui drawing, an adapter default, injected source, forged watcher
  delivery, certification-only construction, or a detached screenshot cannot
  satisfy the pulse
- the new executable's link/build cost is measured and accepted explicitly;
  ordinary in-process tests do not launch a window or add a nested Cargo
  invocation

The 3.10.2 automated courtroom proves real source, watcher, application,
mounted, and egui integration in-process. Its separately measured human launch
proves that the product binary can reach publication, and that launch found the
main-thread stack defect the automated suite missed. It does not count as
automated executable-world certification. Milestone 3.10.3 preserves both
valid evidence classes and closes their missing join through the actual binary.

### Milestone 3.10.3: Executable World Certification Foundation

Status: Closed on 2026-07-27. Phases 1 through 5 are closed; this corrective
gate is complete and Milestone 3.11 may inherit its permanent executable world.

Detailed spec: [milestone-3.10.3.md](./milestone-3.10.3.md)

This successor gate makes product entry a permanent proof boundary. It does not
add another product feature. It classifies the existing egui-context and
watched-lifecycle tests honestly as in-process integration, then adds one
separately budgeted executable-world target that launches the exact Cargo-built
pulse binary through `main`, the then-current predecessor shell, the operating-system native
event loop, `PlatformPulseNativeFrame`, its watcher worker, and normal native
shutdown.

**Platform pulse**

Copy the exact checked-in pulse source into one isolated installation, launch
the real product binary against that source root, externally observe a stable
blue native client area, apply the real green edit, preserve green after a
malformed edit, recover blue, and close the native window normally. The runner
must correlate external process/window/pixel facts with versioned product-
issued lifecycle observations without importing runtime internals or treating
either evidence class as sufficient alone.

**Must ship**

- one honest evidence-lane classification for every existing pulse claim
- one ordinary `--source-root` launch configuration with no test-only
  composition root
- one typed, versioned, monotonic, derived lifecycle-observation contract
- one pulse library facade exposing only that observation contract
- one explicit `executable_world` target in the existing pulse package
- one sealed typestate from installation through first frame, replacement,
  preservation, recovery, and shutdown
- one immutable canonical source baseline with isolated semantic deltas
- one Windows process-bound native window, client-pixel, liveness, and close
  adapter
- one cumulative real-process courtroom with independent pixel and causal
  observations
- mutation-sensitive event-only, pixel-only, premature-exit, wrong-reason,
  direct-paint, injection, forced-termination, and skipped-platform controls
- bounded failure artifacts, build/runtime budgets, and residue-free teardown
- committed additive homes for every 3.11 through 3.23 pulse extension

**Must preserve**

- every real 3.10 through 3.10.2 product and authority guarantee
- the sole checked-in pulse source, product binary, application lifecycle, and
  runtime-to-host path
- the consolidated fast integration target as a distinct cheaper proof lane
- no test-only product branch, hidden constructor, injected authority, nested
  Cargo invocation, or ordinary-frame observation cost
- 3.11 ownership of pixel-to-mounted identity and every later milestone's
  declared semantic authority
- explicit Windows executable certification without claiming unexecuted Linux
  or macOS platforms

**Acceptance evidence**

- the required Windows lane launches Cargo's exact child binary rather than an
  app function and executes at least one courtroom scenario
- first publication is followed by a 500-millisecond live process hold, a
  process-bound native window, independently expected blue client pixels, and
  matching product causal observation
- external blue-to-green edit, typed malformed denial with exact predecessor
  preservation, canonical blue recovery, native close, typed shutdown,
  successful exit, and zero residue all occur in the same child
- runner typestate makes assertion before observation, denial as success,
  consumed-world reuse, and completion without teardown unavailable
- dependency and topology checks forbid product internals, feature-dependent
  product semantics, second binaries, second executable targets, and zero-test
  certification
- build, launch, action, observation, artifact, timeout, retry, and cleanup
  budgets pass on exact final source

Milestone 3.11 inherits the closed `application`-owned replacement preparation
and commit boundary, the visible pulse seed, and
`PulseExecutableWorld<Published>`. It adds visual snapshot truth to that same
live process without reopening source ownership, public midpoint execution,
session-owned replacement, static-paint authority, or product-entry
infrastructure.

Closing evidence preserves three separate lanes: 3/3 observation-protocol
tests, 4/4 consolidated in-process pulse integration tests, and 6/6 Windows
executable-world tests through the exact Cargo-built binary. The cumulative
native journey proves blue publication, green replacement, malformed-source
predecessor preservation, canonical blue recovery, normal close, typed
shutdown, successful exit, and zero residue in one child with zero scenario
retries. Windows is executable-certified; unexecuted native platforms remain
compile-only. Clean and warm executable-target builds, journey duration,
observation bytes, captures, launches, retained failure evidence, and teardown
all remain inside the frozen Phase 1 budgets.

### Milestone 3.11: Visual Snapshot Receipts and Hit-Test Identity Bridge

Detailed spec: [milestone-3.11.md](./milestone-3.11.md)

Status: Closed on 2026-07-27. Phases 1 through 5 are complete and all nine
visual-snapshot scenario rows are proved on final source.

This slice makes screenshots, hit testing, and visible-region targeting
identity-backed runtime evidence instead of loose image bytes. Visible paint
attribution and input hit testing are separate results bound to one exact
presentation; neither may be guessed from the other.

**Platform pulse**

Extend the checked-in Platform Pulse Page into a deliberately nondegenerate
world with distinct background and inset target regions. Capture the running
page, choose independently known target and background points, and make the
target result visible as a canonical mounted overlay and human-readable trace.
For the target point, the independently represented visible contributor and
hit-test target deliberately agree on one mounted receipt; the supporting
scenario portfolio must also prove cases where paint and hit testing diverge.
The receipt must resolve through the exact captured presentation to graph,
declaration, provenance, and evidence. A sole-node fallback, loose PNG,
coordinate-only answer, current-at-completion lookup, renderer-local overlay,
or reconstructed parallel tree does not close the pulse.

Executable-world closure extends the inherited
`PulseExecutableWorld<Published<InitialBlue>>` with external client-area
capture and the point-to-mounted adjudication. An in-process egui capture or
new screenshot harness cannot substitute for the inherited product process.

The permanent Platform Pulse now renders distinct blue background and yellow
target regions, captures the WGPU product window through the process-and-HWND
bound WGC observer, traces independently chosen target and background pixels
to mounted identity, publishes and clears a magenta mounted overlay, survives
valid and malformed source replacement, closes normally, and reports zero
visual residue. The final courtroom uses one child and window, six native
captures, eleven lifecycle events, and no retry path.

**Must ship**

- `UiVisualSnapshotReceipt`
- concrete visual-inspection grant and artifact/disclosure policy
- presentation-bound host capture observation
- host-issued presentation epoch and exact readback fence
- managed capture polling, cancellation, timeout, and disposal
- compiler-enforced target, artifact-policy, grant, coordinate-scope, capture,
  and overlay phase progression
- typed screen/client/viewport/host coordinate transform
- frame capture by identity
- node capture by identity
- region capture by identity
- immutable visible-region map
- hit-test region map
- explicit total per-surface hit-test order
- separate visible-contributor and hit-test point outcomes
- bounded many-to-many region adjudication
- visible mounted node overlay through a successor canonical mounted frame
- `screen/client point -> mounted receipt identity trace`
- `screenshot region -> mounted receipt identity traces`
- `mounted receipt identity -> mounted instance / incarnation / declaration /
  graph / provenance / evidence` bridge
- bounded snapshot, pixel, overlay, and retained-frame lifecycle

**Acceptance evidence**

- the runtime can capture the current or retained frame without creating a
  second visual truth path or relabeling a capture across replacement
- an agent can ask what painted a pixel and what would receive input there and
  receive separately typed mounted identity results
- a screenshot region can be traced as a bounded many-to-many result back to
  mounted receipts, graph nodes, declarations, provenance, and evidence
- the visible target overlay is externally observable but was published
  through the same mounted host contract and cites its base snapshot
- screenshot support is tied to exact presentation, surface binding, viewport,
  retention, disclosure, and cost rather than loose PNG bytes

### Milestone 3.12: Observation Intake and Hot Rebind Planner

Detailed spec: [milestone-3.12.md](./milestone-3.12.md)

Status: Closed on 2026-07-28. Phases 1 through 5 and all thirteen Phase 5
courtroom rows are complete. The permanent Platform Pulse performs valid,
malformed, recovery, comparison, close, and cleanup work in one real process;
supporting evidence closes source affinity, semantic/pixel independence,
identity, mixed-owner ordering, Query consequences, effects/recovery,
cost/capacity, visual affinity, lifecycle cleanup, and compile boundaries.

Milestone 3.12 inherits Milestone 3.11's snapshot, overlay, mounted-trace,
disclosure, resource, and teardown contracts without relocation or
reinterpretation. Its closing QA additionally corrected real authored portal
measurement authority being absorbed by a broader unscoped allocation
neighborhood.

This slice makes hot reload real as bounded rebind rather than renderer
refresh. Query-backed changes enter only as the sealed UI source consequences
admitted by 3.9.2's framework-turn boundary. This milestone classifies their
affected UI scope and decides preservation or remounting; it does not reopen
Query leases, deltas, rows, ordering, patches, or identity.

The operating-system watcher remains transport only. A watched edit must settle
to one exact source revision, compile through `worth-ui-dsl`, and reach
invalidation only as canonical declaration-lane facts constrained by declared
aspect contracts. This slice also establishes bounded source-span, changed-fact,
stop-point, and rebind-decision evidence references for later human and agent
diagnostics; it does not pull 3.19 diagnostics or 3.21 certification replay
forward.

**Platform pulse**

Edit the real pulse `.wui` file and visibly change one bounded part of the
already-running page without restarting the application. Operating-system
watcher delivery must reach semantic observation admission and a typed rebind
plan; an unaffected visible region must remain stable where preservation is
admitted. A second invalid edit must leave the last admitted page visibly
current and expose the typed denial instead of blanking, partially replacing,
or renderer-refreshing the page.

Executable-world closure applies the edit through the inherited world's
external source delta, and the same child process must prove affected and
unaffected native consequences plus the typed rebind receipt. Direct
in-process replacement does not close the executable pulse.

The comparison consumes Milestone 3.11's retained predecessor and successor
visual snapshot identities. It may relate mounted identity and visible regions,
but it may not reinterpret old pixels as current or replace semantic rebind
evidence with a raw pixel diff.

**Must ship**

- exact watcher hint -> settled revision -> DSL compile outcome -> sealed
  candidate -> canonical declaration delta bridge
- typed DSL compile-stop receipt that cannot expose observation, scope, or plan
  identities
- `UiHostObservation`
- `UiRebindPlan`
- `UiRebindReceipt`
- concrete visual-comparison grant, borrowed predecessor/successor/rebind
  request, typed comparison outcome, and identity-aware receipt
- typed no-change, evidence-only-source, and changed-fact classification
  branches
- affected-aspect detection
- consumed-fact and consumed-aspect index lookup
- preserve/remount decisions
- invalidated measurement, binding, and obligation sets
- explicit observation/rebind profiles with the frozen Platform Pulse limits
- exact-generation authored-span and production-projected rebind inspection
  targets with bounded evidence-reference lookup
- self-audited Phase 5 command registry and direct rebind/transition tests

**Must handle**

- source declaration edits
- viewport resize
- host measurement results
- Query-backed fact changes
- existing committed scroll-extent and portal-anchor consequences; general
  service input remains owned by Milestone 3.15

**Acceptance evidence**

- one valid file edit is causally joined from exact settlement and DSL lowering
  through canonical declaration/aspect consequences to the published rebind
- one invalid file edit retains the exact typed DSL report and source span,
  constructs no later-phase identity, and preserves the last admitted page
- local source edits do not rebuild the whole page
- appearance changes do not invalidate structure
- layout changes do not invalidate Query binding unless declared
- resize invalidates allocation without broad graph rebind
- invalid hot edits preserve the last admitted mounted truth
- a valid edit retains exact predecessor and successor snapshots through one
  budgeted identity-aware comparison, then disposes both without treating
  pixel equality or difference as semantic rebind evidence
- the exact Phase 5 command registry and causal rebind/transition tests pass on
  the final commit
- rebind summary/evidence-reference inspection is available while causal
  diagnostic materialization and replay remain explicitly deferred

### Milestone 3.13: Query Binding and Projection Consumption Substrate

This slice broadens Milestone 2's declared binding references into a minimal
runtime binding substrate, but not yet the full product surface richness of
Milestone 6. Milestone 3.9 proves only plan/handle/visible-range execution for
already-admitted opaque view references; 3.13 owns the broader UI projection
facts, schema/view-shape posture, result-state, and invalidation semantics. It
generalizes 3.9.2's contract-derived access and admitted-source pattern; it may
not restore field scans, positional fact bags, printable Query identity, or raw
Query patch types as a convenience layer.

Detailed spec: [milestone-3.13.md](./milestone-3.13.md)

**Platform pulse**

Add one small semantic-text status group bound to a real installed scalar
Query projection. Its stable value and posture slots use distinct mounted
receipts. The production process starts with Query meaning installed and an
external value absent, visibly presents Query-owned `pending`, then consumes an
atomically published external value through an application-owned source
adapter and visibly presents its exact native text without restarting. A
second value, valid but schema-incompatible `.wui` binding edit, recovery,
stable control pixels, and close complete the journey.

This remains one cumulative Pulse process: it also repeats the 3.11
snapshot/identity/overlay round trip and the 3.12 valid, malformed,
preservation, and recovery sequence. Adding Query evidence may not replace
those inherited operations.

Executable-world closure must install Query meaning through the production
declaration/host audience facades and observe the real child process. The
runner may mutate external source bytes; it cannot inject Query workspaces,
projection facts, Query receipts, UI posture, mounted content, or a
product-local substitute. External screenshots, mounted receipts, Query and UI
evidence, and stable control pixels jointly prove the result.

**Must ship**

- `UiProjectionBinding`
- `UiProjectionFactReceipt`
- shape-specific scalar and collection bindings and receipts
- contract-derived field and row identity
- schema/view-shape and payload-shape admission
- binding invalidation
- mounted host-neutral semantic text and native egui presentation
- orthogonal posture axes:
  - admission: `ready` or `denied` / `unsupported` / `schema-mismatch` /
    `wrong-world` / `rebind-required`
  - availability: `pending` or present
  - present currency: `current` or `stale`
  - stale refresh: idle or `revalidating`
  - collection completeness and continuation
- exhaustive Query posture mapping and production reachability for every
  claimed transition; WUI may not simulate a missing upstream producer
- explicit application dependency grammar through `worth-ui`,
  `worth-query-decl`, and `worth-query-host`, never raw Query or a direct Pulse
  dependency on `worth-ui-query-binding`

**Acceptance evidence**

- the permanent Pulse visibly progresses pending -> first current value ->
  second current value -> schema-mismatch preservation/diagnostic -> recovered
  current value in one process
- selected fields come from exact Query projection contracts with work
  independent of unrelated projection width
- schema-swap rebinding preserves compatible field identity where admitted
- stale and revalidating can coexist without a flat local status enum
- collection reorder preserves Query-authorized row identity, never position
- invalid schema/payload posture emits typed mounted diagnostics
- mixed Query/source/viewport changes reuse the 3.12 observation, rebind, and
  publication path
- Query-free and unchanged-frame paths perform zero projection work
- no local loading/error enum, renderer-side query builder, raw Query patch,
  or diagnostic-to-authority path exists

### Milestone 3.14: Intent, Operability, and Interaction Substrate

Detailed spec: [milestone-3.14.md](./milestone-3.14.md)

Status: Closed on 2026-08-01. Phases 1 through 5 and all thirteen interaction
and intent proof rows are complete on the permanent executable Platform Pulse.

This slice turns native host observations into presentation-bound semantic
interactions and then into runtime-admitted product intents. Pointer
press/release observations may compile into `activate`; neither the observations
nor an adapter-reported click are an intent or mutation result. UI routing
admission does not replace Query/domain mutation admission.

**Platform pulse**

Add one visible operable action and one confirmation action. Native
pointer press/release must target the exact presented mounted incarnation,
route through a typed activation interaction, derive one coherent payload and
operability proof, and enter managed execution. The page visibly distinguishes
admitted, completed, confirmation-required, denied, stale-confirmation, and
rebind-cancelled posture. The host, renderer, and control may project those
facts but may not own callback, payload, operability, or completion meaning.

Executable-world closure must send the interaction through the cumulative
target's current native-platform input adapter and correlate the visible
consequence with the typed intent outcome. Calling the intent facade directly
from certification is integration evidence only.

**Must ship**

- compiled `UiIntentDefinition`, canonical `UiIntentDeclaration`, and compact
  per-control `UiIntentRouteBinding`
- exhaustive `UiIntentAdmissionDecision` and move-only `UiAdmittedIntent`
- presentation-bound semantic interaction families for:
  - `activate`
  - `edit-commit`
  - `selection-commit`
  - `submit`
- typed product-intent support for:
  - `navigate-page`
  - `change-mosaic`
- direct portal and command service intents remain typed unsupported until
  Milestone 3.15 owns their execution
- operability as orthogonal support, mutability, readiness, occupancy, policy,
  affinity/freshness, and confirmation axes rather than one flat status
- target-scoped single-flight as the ordinary concurrency posture; broader
  declaration/definition/application serialization requires an explicit type
- coherent typed payload projection, bounded draft/gesture lifecycle, affine
  exact confirmation, framework-owned execution attempts, typed
  partial/indeterminate recovery, and declared consequences through the 3.12
  observation/rebind/publication path
- explicit application-effect, WUI-transition, and deferred runtime-service
  execution destinations; UI routing admission never widens their authority
- replacement of cloneable string intent binding and static
  always-admitted readiness placeholders; no parallel compatibility lane

**Acceptance evidence**

- controls, renderers, and adapters own no callback or product-effect meaning
- targeting is bound to the frame the human saw, not current coordinates or
  an equal graph identity
- edit/submit payloads come from one coherent runtime revision rather than
  renderer or executor rereads
- operability and confirmation are runtime proofs; visible enabledness and a
  boolean confirmation result carry no authority
- UI admission, provider execution, Query/domain admission, effect completion,
  and visible consequence remain separately typed and causally traceable
- gesture loss, stale challenge, rebind at every effect phase, exhaustion,
  cancellation, partial/indeterminate outcome, retry, and shutdown have exact
  bounded lifecycle evidence
- work scales with selected target, payload width, and affected consumers, not
  mounted graph width; unchanged input has zero semantic work

### Milestone 3.14.1: Aspect-Native Host Platform and egui Retirement

This slice replaces the interim `egui`/`eframe` host with a Worth-owned
aspect-native presentation platform and removes `egui` from the workspace
dependency graph. It exists now, between intent and services, because it is
the last point where the host adapter surface is small: after 3.15 and 3.16
every service, focus, motion, and appearance lane would be built twice.

The detailed specification closes the governing Phase 1 inputs:
`worth-ui-body-default-v1` pins Noto Sans v2.015, printable Basic Latin,
rustybuzz 0.20.1, swash 0.2.10, exact raster/atlas bounds, and typed rejection
outside the support set; `worth-ui-windows-dx12-v1` pins winit 0.30.13, wgpu
29.0.4, DX12, exact surface/blend/rounding policy, and every queue/resource
capacity. It also freezes the runtime-private affine platform binding,
borrowing preparation-builder scope, sealed host-native resource/readiness
registration, host-neutral prepared application, and cleanup-proved denial
progression. The public `worth-ui-native-platform` crate is a facade over that
runtime-owned gate; downstream product crates cannot mint a grant or bind a
host-neutral Worth application. The lower host-mechanics crate remains a
deliberately callable integration surface and is not misrepresented as a
cross-crate friend boundary.
Phase 1 must materialize and prove those records without weakening them.

That v1 text record is immutable Phase 1-2 migration evidence, not the final
text platform. The repository-owned `worth-ui-global-text-v2` candidate now
pins the exact 30-face default collection, application-font-pack admission,
Unicode 17 conformance data, deterministic cluster fallback, complex shaping,
bidirectional and line layout, original-range and caret geometry, Unicode 17
RGI color emoji, alpha/color raster formats, and bounded layout/atlas
capacities under manifest digest
`cec6005c5baef6d69ada9c30c02ced25b0f253f80c012784fe925e307935c3f2`.
Phase 4 may consume it only after the manifest and its referenced assets,
licenses, generated indexes, dependency posture, and digest pass focused
qualification checks and code review. The text work is
split into canonical Unicode layout/measurement first and color glyph/emoji/
native atlas presentation second; neither may begin from the Basic-Latin seed.

The mounted host contract is the subject under test and may not change
meaning. `worth-ui-host-contract` semantics, the 3.12 observation turn, the
3.13 projection path, and the 3.14 interaction and intent path are frozen; if
replacement pressure demands a host-contract semantic change, that is a
specification defect, not a migration convenience.

Phase 1-2 production requirements were historically reviewed at revision
`234c3aaf4`. Their ledger artifacts are frozen and do not reopen. Current
changes run the predecessor contract tests affected by their causal scope and
are reviewed against the final diff. The native seed remains deliberately
narrow: retained deltas, the v2 text platform, input, resize, capture, recovery,
Pulse parity, and egui deletion stay in their ordered phases. The stronger
future text requirement supersedes the earlier Phase 4 qualification target.

Each phase states its architecture, authority, lifecycle, recovery, performance,
resource, integration, developer-experience, and test considerations in prose.
Focused tests run during implementation; required constitutional checks and
affected CI or native lanes run on the final commit; code review decides whether
the evidence is adequate. Shared Cargo discovery, exhaustive corpora, compile
contracts, and native worlds should execute once per applicable test run.
Retained diagnostics may aid reproduction but do not create QA authority.

The custody boundary moves down, not sideways. The new host owns a retained
draw list in which every quad and glyph run carries its mounted node receipt,
frame generation, surface generation, and binding generation. Damage-scoped
presentation derives from an owner-issued, compiler-total
`Initial | Delta | Reconstruction | Unchanged` work contract and a runtime-issued total paint
order, not host-side projection comparison. An unchanged turn performs zero
draw-list, Unicode analysis, shaping, line layout, raster, atlas, and surface
work, provable by named counters rather than elapsed time. A dedicated host-
neutral WORTH text-mechanics boundary consumes pinned Unicode, shaping, and
raster dependencies: WORTH owns font-collection admission, deterministic
fallback, run segmentation, layout identity, measurement equivalence, and
resource bounds, while dependencies receive only qualified runs and decide no
semantic value, mounted identity, product language, wrapping policy, or host
authority. Complex scripts, mixed-direction text, locale-sensitive wrapping,
original-range cluster/caret mapping, application-bundled fonts, and Unicode
17 color emoji are required. Writing a custom shaping algorithm, owning IME
composition semantics, rich-text authoring/editing authority, vertical
writing, general vector tessellation, any platform beyond the currently
certified Windows lane, and accessibility-tree projection remain explicit
non-goals; the v2 artifacts must nevertheless be the foundation those later
semantic owners consume.

Application-bundled fonts are a normal Phase 4 product capability, not a
single-font exception. Applications can admit multiple content-addressed
OpenType `TTF`/`OTF`/`TTC`/`OTC` packs from owned bytes, address distinct
application-scoped families, and author an ordered family stack with explicit
weight, width, slant, variable-axis, and feature requests on each style span.
Static regular/bold/italic/oblique faces, variable fonts, multiple collection
face indices, and deterministic family-name collision handling are must-ship.
After later authored families, complete RGI emoji clusters use qualified color
emoji then Last Resort; non-RGI clusters use qualified profile defaults then
Last Resort. Neither route may decompose a cluster or use the other route as a
silent substitute. Ambient OS fonts, path/name lookup, registration-order tie
breaks, and renderer-local defaults remain forbidden; `WOFF`/`WOFF2` remain a
typed unsupported format until a later profile qualifies their decoding and
limits.

Font-pack changes advance `UiFontCollectionGeneration` rather than mutating a
live collection. Existing layouts pin their exact predecessor bytes; new work
uses the successor generation; stale layouts deny before effects; and only
paragraphs whose resolved layout-affecting input changes may be reanalyzed.
That input includes the authored family stack, exact selected face, weight,
width, slant, variation coordinates, OpenType features, language, source text,
and paragraph constraints. A foreground-color value change with unchanged
paint-span boundaries is the sole explicit no-reshape exception.
The same selected-face and generation identity must be consumed by layout,
measurement, hit testing, accessibility geometry, headless evidence, native
rendering, and reconstruction. Phase 4 cannot close on a default-only font
demo: its font-collection courtroom must use at least two overlapping
application families, multiple styles/axes, a multi-face collection, missing-
cluster and emoji fallback, generation replacement/removal with a live old
layout, and hostile hard-coded-family/system-font/stale-generation mutants.

Mixed span appearance is also must-ship. One line may combine different
application families, font sizes, weights/slants, languages, bidi directions,
and foreground colors alongside intrinsic-color emoji. Layout owns stable
original-range paint-span boundaries but not authored color authority; mounted
appearance supplies logical straight RGBA to headless/native glyph-run paint.
A color-value-only edit reuses analysis, shaping, layout, metrics, interaction
geometry, and atlas entries and updates only affected paint commands/damage.
Foreground follows logical source ranges through bidi reordering, alpha glyphs
are tinted by the qualified pipeline, and color emoji is never accidentally
tinted by neighboring text.
Canonical text interaction includes visual-edge plus upstream/downstream caret
affinity, point-to-line-to-visual-run-to-cluster hit records, discontiguous
bidi selection rectangles, and one shared accessibility-geometry consumer.
Content-only, paragraph-width-only, and document-wide width changes have
separate locality courtrooms. Capacity uses conservative qualified-font bounds
and bounded unpublished staging, so derived overflow can deny atomically before
publication or raster effects.
Phase 4 treats every Unicode 17 RGI emoji sequence as one qualified text unit
through segmentation, fallback, shaping, wrapping, ellipsis, caret movement,
hit testing, selection, and original-range mapping. That includes variation
selectors, keycaps, flags, tag sequences, skin tones, and gendered/family ZWJ
sequences. Phase 5 owns exhaustive exact-sequence color-glyph raster, atlas,
and representative native pixel evidence for COLRv0/CPAL, COLRv1/CPAL,
CBDT/CBLC, and sbix `png`/one-hop-`dupe`; sbix `jpg`/`tiff` and OpenType SVG are
explicitly rejected by this profile.
It may not compensate for a Phase 4 sequence split.

Phase 5 remains an ordered gate inside Milestone 3.14.1. Its detailed raster/atlas authority graph, staged
native transaction, `HP-03` courtroom, exact cost/resource contract,
implementation gates, documentation, and Phase 6 handoff are governed by
[`milestone-3.14.1-phase-5.md`](milestone-3.14.1-phase-5.md). The subordinate
specification exists to keep this roadmap outcome-readable; it does not create
a second milestone or independently closable product surface.

Phase 5 was implemented and historically reviewed on 2026-08-20. Phases 6-7
were implemented and independently reviewed on 2026-08-23. Phase 8 was
implemented and independently reviewed on 2026-08-24. The corrected Phase 9
implementation record and its frozen source snapshot were independently
certified and closed on 2026-08-24. The corrected Phase 10 native-cutover
implementation and its frozen source snapshot were independently certified and
closed on 2026-08-24, completing Milestone 3.14.1.
The frozen QA
ledger and retained artifacts are not current authority and are never reopened.
The accepted test coverage
includes deterministic alpha/color raster and
emoji coverage, bounded native atlas ownership and pins, exact DPI/paint
locality, a 4×8 fresh-world cost matrix through 4,096 items, external native
pixels, complete derived-state reconstruction, Query-owned async presentation,
and terminal-zero Signal/native/Query census. Phase 6 consumes that closed
text-presentation authority and does not reopen it.

Phase 5 also closes one narrow downstream Query integration: each semantic
mounted text-presentation attempt is installed and retained as Query-owned
async result state through `worth-ui-query-binding`. Runtime owns attempt
lineage, host-native owns physical completion and live recovery, Signal supplies
eligibility/invalidation evidence only, and Foundational supplies canonical
boundary/reporting meaning only after those owners decide. Glyphs, atlas
entries, pages, uploads, WGPU indexes, readiness wakes, terminal projections,
JSON, strings, digests, and numeric generations are never presentation
authority. The broad Query audience migration remains separate roadmap work.

Phase 3 keeps protocol revision 4, advances the mounted-presentation schema to
revision 5 for the distinct cold reconstruction envelope, and keeps the
2,048-row/1 MiB text-carrier ceiling. Its
native damage courtroom is a 2,048-rectangle pixel world; its separate
4,096-command mixed rectangle/text world proves runtime/headless carrier and
index slope without claiming text layout or native glyph pixels. After the v2
qualification gate closes, Phase 4 advances the protocol, mounted-frame, and
measurement schemas to revision 5, retains presentation schema 5, and admits
4,096 paragraphs, 65,536 UTF-8
bytes per paragraph, and 8 MiB aggregate retained text. Mixed revision-4/v5
frames and retained generations reject before effects.

`worth-ui-text` owns concrete immutable layouts while depending only on inert
host-contract records. Runtime retains each layout with mounted affinity and
seals a borrowed host-contract layout view into active presentation work.
Headless and native consumers can inspect that view but cannot shape, refallback,
rebreak, clone authority, or retain it beyond the owning work envelope.

**Platform pulse**

The same cumulative Pulse binary, `main.wui`, and executable-world target run
on the Worth-owned host. During transition both hosts present the same
mounted content behind the unchanged host contract, and the same external
courtroom adjudicates both: semantic, receipt, identity, and control evidence
must be identical, while glyph-region pixel expectations are re-baselined
exactly once for the new text metrics, with the re-baseline explicitly reviewed.
Completion requires the full inherited journey — pending,
two current values, overlay round trip, green/malformed/recovery source
sequence, schema stop with predecessor preservation, the 3.14 native operable
and confirmation actions, and normal close with zero window, surface, device,
atlas, and draw-list residue — through the native host, followed by deletion
of the `egui` adapter and shell. A new harness, scripted host closure, or
retained dual-host fallback does not close the pulse.

The frozen Phase 9 candidate uses event-driven readiness on both hosts and
adds a real 500 ms unchanged segment after final recovery: the process remains
alive, emits no lifecycle envelope, and retains byte-identical client pixels.
The existing certification-owned mounted-session lane separately requires
exact unchanged reuse with zero allocation, zero mount work, zero adapter work,
and no additional host presentation; the executable runner imports none of
that lane's internal authorities. External color, geometry, control-point,
DPI, and text-profile expectations come from one versioned test-owned literal
manifest rather than production visual constants. All visual await/observe
actions advance the shared causal cursor at their real boundaries, and process
identity plus exit poll multiplicity remain typed host-mechanical exclusions
with their raw pairs retained.

**Must ship**

- Phase 1 materializes a contract-only Worth native host-mechanics crate as
  profile/capacity/inert-contract owner, a runtime-private effect-free
  application-platform gate, and a thin public platform facade; Phase 2
  activates the host's event loop, readiness, surface/device/DPI/close
  lifecycle without changing that topology
- a sealed public native-application preparation contract whose platform grant,
  host-neutral builder, readiness owners, resource registry, prepared
  application, denial cleanup, and run transition are compiler-visible
- a contract-only production headless host crate moved out of runtime and
  migrated through the same presentation-work protocol
- a retained, receipt-keyed aspect-native draw list with layer order, clip
  bounds, and damage-scoped presentation derived from admitted plans
- a surface-issued canonical transparent baseline for clearing uncovered
  damage; every opaque background remains an attributed runtime-issued command
- a host-neutral initial/delta/reconstruction/unchanged presentation-work protocol and unique
  total paint order that make host-side semantic diffing unnecessary
- a host-neutral Unicode 17 text-mechanics crate shared by headless and native
  hosts, with content-addressed default/application font collections,
  deterministic cluster fallback and Last Resort posture, UAX #29/#14/#9
  analysis, complex shaping, line layout, original-range cluster/caret/
  selection geometry, and one canonical measurement/rendering layout artifact
- filled-rectangle plus multilingual, bidirectional, complex-script, symbols,
  CJK, mixed-family/mixed-size/mixed-foreground spans, and Unicode 17 RGI
  color-emoji glyph-run presentation with bounded alpha/color atlas ownership
  and metrics derived from that canonical layout, never from adapter-local
  font defaults
- snapshot capture through surface readback feeding the existing visual
  snapshot contract
- host-neutral input observation translation sufficient for the shipped 3.14
  presentation-bound interaction families
- pinned, audited mechanics dependencies and Unicode data for segmentation,
  bidi, line breaking, shaping, and rasterization with a recorded trust posture
- removal of `egui` and `eframe` from every workspace dependency edge, with
  the boundary gate permanently denying their return; isolated egui-era
  theme/component crates are retired rather than prematurely ported
- named platform qualification for the certified native lane

**Acceptance evidence**

- dual-host parity: identical semantic, receipt, and control evidence for the
  same mounted content, with exactly one recorded glyph re-baseline
- every presented draw command is causally attributable to a mounted node
  receipt and generation set; receiptless paint is unrepresentable
- unchanged-frame zero work extends through the draw list, atlas, and surface
  counters, not just the semantic turn
- the cumulative dual-host journey contains a bounded quiescent interval with
  zero lifecycle events and byte-identical external client pixels, while the
  certification-owned mounted-session lane independently proves exact-zero
  adapter admission without entering the executable runner
- Unicode, shaping, fallback, and line-layout dependencies demonstrably receive
  only their completed typed inputs and cannot alter value, semantic style,
  mounted identity, language/direction policy, wrapping, or clipping evidence
- the independent text courtroom convicts Basic-Latin-only handling, scalar
  fallback, bidi/source-order corruption, grapheme/emoji-sequence splitting,
  missing color layers, single-foreground substitution, visual-order color
  assignment, intrinsic-emoji tinting, layout regeneration on a color-only
  edit, duplicate measurement shaping, system-font lookup, stale width/text-
  scale/DPI reuse, pinned-glyph eviction, and broad retained-paragraph
  reshaping
- hostile controls turn red on receiptless paint, whole-frame repaint of an
  unchanged turn, adapter-invented geometry, internal-state snapshot
  substitution, and leaked window, surface, device, or atlas resources
- `egui`, `eframe`, `egui-wgpu`, and `egui_extras` appear in zero dependency
  declarations, resolved edges, or final lockfiles across the repository-root
  and Worth UI workspaces, the boundary gate enforces the prohibition, and the
  cumulative executable world closes on the native host with no new test
  target or harness

### Milestone 3.15: Production Runtime Services

Detailed spec: [milestone-3.15.md](./milestone-3.15.md)

Status: Closed on 2026-08-29. Phases 1 through 7, the six public service
families, the native `RS-01` journey, protocol `RS-07` portfolio, scheduled
`RS-10` scale courtroom, cumulative Platform Pulse, and separate
product/design quality judgment are complete.

This slice closes the cross-cutting service lanes at production-ready common-
case breadth before the certification vertical slice depends on them.

It inherits 3.14's presentation-bound interaction, typed request,
operability, confirmation, managed-attempt, and consequence contracts.
Intent-origin service requests lower from that provider topology. Host
observations, rebinds, and high-frequency continuation lanes keep their own
typed entry contracts rather than pretending to be managed intent attempts.
None reopen targeting or regress to adapter callbacks.

Each family owns its state. Cross-owner requirements are compiled by a
non-publishing proposal compiler into the existing 3.12 observation/
publication and mounted-presentation paths. The compiler is not a seventh
service, frame publisher, host-effect issuer, or settlement/recovery owner.

**Authority boundary**

- these services own UI-runtime meaning only; command routing selects and
  progresses a UI route, while any resulting application operation remains a
  separate Query admission through the appropriate audience facade
- selection identity is not Query identity, capability, disclosure, or
  authorization
- scroll position, viewport state, and extent are not Query cursors,
  continuations, snapshots, or evaluation bases
- Query may supply admitted content extent as allocation evidence; it never
  supplies scroll offset or routing authority
- portal, focus, and motion receipts are not Query publication, recovery, or
  external-completion receipts
- adding service support does not imply Query support, and this milestone may
  not add a raw `worth-query` import or widen the predecessor edge in
  `worth-ui-query-binding`
- undo and redo remain aspirational. Command routing may expose the future
  attachment point, but this milestone admits no undo/redo authority or
  actionable presentation until the owning operation runtime publishes it;
  Query's `provisional_aftermath` experiment is not that contract

**Coverage bar**

`p95 coverage` means product breadth, not test or line coverage. Each service
family must feel complete for ordinary serious desktop use across its lifecycle,
host integration, hot rebind, typed denial/cancellation/cleanup, inspection,
bounded cost, and keyboard/accessibility semantics where applicable. The
detailed spec must name the ordinary scenario portfolio and the small set of
genuinely uncommon exclusions; one admitted happy path cannot satisfy this
milestone. Later work may add specialist behavior or polish, but must not be
required to repair a skeletal service.

**Platform pulse**

Graduate the austere 160-by-96 pulse control into a designed 960-by-600 logical
client area with a 24-pixel outer gutter, eight-point rhythm, identity masthead,
read-only evidence rail, primary service stage, and truthful status band. The
composition uses only real authored fills/theme tokens, qualified text,
mounted content, Query projection, admitted controls, and service facts. It
must not imitate a dockable shell, inspector, toolbar, theme system, or control
state that the platform has not shipped. Decorative regions do not hit-test;
every visible interactive region has real admitted behavior.

Within that composition, extend the pulse control into one anchored portal
interaction: opening it visibly creates the portal, moves focus through a
coordinated typed focus requirement, applies one receipt-derived motion
transition, and dismisses through the declared service rules. A human must be
able to follow the open/focus/move/dismiss sequence, while receipts preserve
logical owner, anchor, layer, focus, and motion provenance. Adapter-local
popup, focus, animation, decoration, or fabricated product state does not close
the pulse.

The same cumulative journey resolves one typed shortcut in two coherent
contexts and externally shows the winning and losing candidates. It also routes
one Query-backed command successfully through UI admission, receives a separate
typed Query audience denial, and publishes that denial through the external
Pulse observation stream. A UI-operability denial is not equivalent evidence.

Executable-world closure requires the inherited child to receive the native
open and dismiss actions and expose externally observable portal, focus, and
motion consequences. A scripted host remains useful service integration
evidence but cannot close the product pulse.

Keep the Pulse bounded to ordinary product scale and within its inherited
journey budget. The native runner proves the exact 960-by-600 composition,
resizes it to 1120 by 700 while the portal is open, and checks structure plus
stable pixel masks against an independent design manifest. Deterministic
presentation evidence covers 1.0 and 1.5 device scale, text containment,
contrast roles, alignment/rhythm, minimum target size, and hit-test honesty.
A whole-frame golden, mockup, or oracle copied from production constants is
insufficient. Scale and protocol-fault claims still close separately inside
existing certification targets: the former proves bounded work at the detailed
spec's stated service scale; the latter proves stale/reordered delivery and
indeterminate settlement through production host contracts. Neither may be
relabeled as native Pulse evidence, and no new Cargo test target or executable
is created.

**Must ship**

- first-class service lanes for:
  - `portal`
  - `focus`
  - `motion`
  - `command-routing`
  - `scroll`
  - `selection`
- p95 product coverage for every family, with explicit support posture for the
  uncommon cases intentionally left outside this milestone

**Acceptance evidence**

- dropdowns open through the portal service with logical owner, anchor, layer
  posture, measurement plan, and focus/dismissal rules
- focus scopes, participant sets, route requests, host focus observations, and
  runtime focus receipts are real runtime artifacts
- motion projections derive from previous receipt + next receipt + motion
  declaration once per semantic retarget; intra-track samples stay in a
  presentation-only lane rather than becoming semantic publications or
  host-local animation meaning
- each family passes its named ordinary-scenario portfolio through public
  runtime and host surfaces, including denial, rebind, and teardown where
  applicable
- a serious downstream application needs no app-local replacement service or
  renderer-owned state for an ordinary supported workflow
- every unsupported case is typed, named, and demonstrably outside the
  milestone's ordinary p95 portfolio
- command-routing evidence distinguishes UI route progression from Query
  application-operation admission, and no service receipt is accepted as
  Query authority

### Milestone 3.16: Appearance, Theme, and Visual State Projection

Detailed spec: [milestone-3.16.md](./milestone-3.16.md)

Status: In progress, Gate 4. Gates 0 and 1 are complete. Gate 4's five sections
remain open: [mounted geometry](./milestone-3.16.md#gate-4a--mounted-geometry),
[text foreground](./milestone-3.16.md#gate-4b--text-foreground),
[Motion composition](./milestone-3.16.md#gate-4c--motion-composition),
[backdrop and Portal](./milestone-3.16.md#gate-4d--backdrop-and-portal), and
[integrated closure](./milestone-3.16.md#gate-4e--integrated-closure).
The spec owns their plans and acceptance. Emission remains disabled until Gate 5's
atomic cutover; Gate 6 owns live/native/design milestone closure.

The governing design freezes explicit role attachment,
surface-bound theme capability, coherent owner-issued state vectors, finite
disjoint appearance resolution without selectors or cascade, a clean mounted
appearance protocol cutover, and bounded pointer-affordance projection. A
single compilable contract baseline precedes parallel worktrees; the first new
appearance emission and final static-paint removal occur in one atomic cutover.
Backdrops are independently authored overlay participants rather than a modal
side effect. Applications explicitly declare each backdrop's extent, presence
basis, appearance role, and before/after placement. Portal supplies sealed
lifecycle and total-order inputs when referenced, while the overlay planner and
native/headless hosts preserve the authored order and source-over composition
without inventing or flattening backdrop meaning.

This slice closes runtime-owned visual semantics before Milestone 9 broadens
into a professional component set. Theme and style meaning already belong to
Worth UI; this milestone makes that meaning admitted, inspectable, and
rebind-safe rather than renderer-local or component-folklore.

**Platform pulse**

Give the running pulse a visibly coherent runtime-decided appearance and show
at least two state-axis outcomes already earned by the page, such as focused,
disabled, selected, validation-bearing, hovered, or pressed. Change an admitted
theme or appearance input and visibly update only the affected consumers;
mounted appearance facts must explain the rendered background, foreground,
border, radius, opacity, or outline. Adapter-default styling or a screenshot
without appearance receipts does not close the pulse.

The cumulative Pulse opens a real two-deep modal stack and separately declares
two viewport backdrops whose presence follows and whose placement precedes the
relevant portal. Each dialog keeps conventional title/body/action composition
and Portal-owned shielding; the second authored backdrop visibly and correctly
dims both the application and the first dialog. Zero-, one-, and two-modal
worlds at both native sizes require separate recorded design acceptance against
the contemporary Linear-or-Notion bar in addition to structural, input,
retained-order, and external-pixel proofs.

Executable-world closure extends the same world with appearance deltas and
native input, and external client pixels must agree with mounted appearance
observations from that child. A new visual fixture, renderer default, or
detached golden image cannot replace the cumulative world.

**Must ship**

- `UiAppearanceRole`
- `UiAppearanceProjection`
- `UiAppearanceStateAxis`
- `UiThemeCapabilityReceipt`
- typed appearance coverage for semantic aspects such as:
  - `appearance.background`
  - `appearance.foreground`
  - `appearance.border`
  - `appearance.radius`
  - `appearance.opacity`
  - `appearance.outline`
- typed state-axis projection for at least:
  - `operability`
  - `focus`
  - `validation`
  - `selection`
  - `hover`
  - `pressed`
- explicit theme-capability consumption and invalidation posture
- mounted visual-projection facts that let hosts render runtime-decided visual
  outcomes without deciding semantic appearance locally
- independently declared backdrop identity, surface-singleton/per-portal-
  instance scope, extent, presence, optional Motion, appearance, and relational
  overlay placement; Portal-issued stack/lifecycle inputs when referenced;
  canonical cumulative source-over; and paint/shield independence
- typed denial posture for missing appearance coverage, unsupported state axes,
  or wrong-world theme consumption

**Acceptance evidence**

- a node's visual outcome is derived from declaration + appearance role + theme
  capability + state-axis posture rather than adapter-local style logic
- theme changes invalidate only the affected appearance consumers instead of
  widening immediately to unrelated structure or Query-binding neighborhoods
- disabled, focused, selected, and validation-bearing visual states are runtime
  projections rather than host-local booleans or component conventions
- nested and sibling modal layers preserve Portal order, cumulative pixels,
  topmost-first dismissal, exact restoration, bounded locality, and separately
  approved stacked-dialog visual quality
- modal-without-backdrop and non-modal-with-backdrop proofs establish that
  portal kind neither creates nor rejects authored backdrop composition
- Milestone 9 can consume appearance and theme runtime lanes without reopening
  their authority as design-system folklore

### Milestone 3.17: DSL Expressions, Conditions, and Semantic Evaluation

This slice makes authored conditions and computed semantic values part of the
real language rather than helper sugar or renderer-side convenience.

**Platform pulse**

Author one pulse condition or derived value that visibly controls presence,
operability, appearance, options, or payload shaping. Edit the expression in
the real source file and observe the bounded result through hot rebind, while
inspection explains its consumed facts/aspects and true, false, stale, or
denied outcome. Rust closures, arbitrary code, renderer conditionals, or
ambient environment reads do not close the pulse.

Executable-world closure applies the expression edit as an authored-source
delta to the inherited live product process. Its external visible consequence
and product-issued evaluation/rebind evidence must join in the existing
executable adjudication.

**Must ship**

- canonical expression artifact families for pure DSL evaluation
- typed expression evaluation for:
  - conditional presence
  - conditional participation
  - operability projection
  - projected options or payload shaping
  - simple derived scalar selection
- explicit consumed-fact and consumed-aspect capture for every admitted
  expression
- denial posture for unsupported, impure, ambiguous, or wrong-world
  expressions
- source-span-preserving diagnostics for expression admission and evaluation
- rebind classification that treats expression invalidation as a first-class
  runtime lane instead of rediscovering it from mounted behavior

**Acceptance evidence**

- `when`-style authored conditions lower into canonical artifacts with typed
  fact consumption and bounded invalidation
- expression changes can invalidate presence, operability, appearance, or
  payload lanes without forcing unrelated declaration rediscovery
- the runtime can explain why an authored condition evaluated true, false,
  denied, or stale without reading renderer code
- no authored expression requires arbitrary code execution, hidden closures, or
  ambient environment fog

### Milestone 3.18: DSL Composition, Modules, and Lowering Equivalence

This slice makes the DSL feel like a real language with reusable composition,
module-scale structure, and honest sugar that lowers into the same runtime
truth rather than creating a second semantic path.

**Platform pulse**

Refactor the same pulse source into at least two authored modules with one
reusable, typed fragment while preserving the page's visible and receipt-backed
meaning. Editing the shared fragment must visibly affect exactly its intended
instances, and an introduced import, symbol, or expansion error must point back
to the authored span. A copied widget helper, textual include, or alternate
lowering lane does not close the pulse.

Executable-world closure grows the installation from one file to a causally
compiled module world inside the existing sandbox. Cross-file edits, failures,
visible consequences, and cleanup extend the same process progression; a
second module-only harness cannot close product composition.

**Must ship**

- module and import boundaries for authored DSL source
- symbol resolution for declarations, fragments, roles, tokens, and bindings
- parameterized composition or fragment expansion with typed inputs
- canonical source-span and expansion provenance for lowered artifacts
- duplicate-name, unresolved-symbol, and import-conflict diagnostics
- lowering-equivalence proof for semantic sugar and fragment expansion
- stable canonical identity rules for cross-file and expanded declarations

**Acceptance evidence**

- multi-file DSL source can be authored with imports and reusable composition
  without smuggling meaning through local widget helpers
- fragments and sugar lower to the same declaration, aspect, and graph truth as
  their fully expanded equivalents
- diagnostics on expanded or imported constructs still point back to authored
  spans rather than only lowered artifacts
- the language can broaden in ergonomics without creating a second authority
  lane beside canonical lowering

### Milestone 3.19: Diagnostics, Inspection, and Evidence Closure

This slice makes denials, support gaps, and rebind decisions typed, mountable,
and inspectable instead of spooky.

**Platform pulse**

Trigger one visible pulse denial—such as an invalid edit, unsupported
appearance, or denied intent—and mount its typed diagnostic without losing the
last admitted page. Selecting the affected visible node or edit must yield a
bounded causal report that explains whether it was preserved, rebound,
remounted, or denied and names the relevant authority evidence. String-only
errors, log archaeology, or an explanation synthesized from private renderer
state do not close the pulse.

Executable-world closure observes the diagnostic from the inherited child
after a real external action, while the last admitted native consequence
remains independently visible. The runner may retain the diagnostic
observation but may not become its truth source.

**Must ship**

- `UiDiagnosticArtifact`
- `UiCausalInspectionReport`
- diagnostic identity, stop class, world, source declaration identity,
  affected graph identity, selected obligations, admission evidence, binding
  evidence, measurement evidence, host capability evidence, rebind evidence,
  aspect-fit denials, and aspect-coverage reports

**Acceptance evidence**

- diagnostics mount through the mounted receipt path
- no string-only errors
- tests do not match diagnostic messages
- unsupported vs denied vs stale vs wrong-world vs rebind-required remain
  distinct
- inspection explains why a node rebound, preserved, remounted, or denied

### Milestone 3.20: Visual Geometry, Design Invariants, and Perceptual Inspection

This slice lets the runtime answer alignment, spacing, symmetry, and visual
consistency questions from receipt-backed geometry first and screenshot pixels
second.

**Platform pulse**

Declare one alignment or spacing invariant over visible pulse nodes and render
its receipt-backed evaluation as a human-visible overlay or focused finding.
Then introduce one deliberate source edit that violates the invariant and show
the changed finding linked to mounted identity, declaration, and source span.
Pixel-only guessing or an overlay detached from the current frame does not
close the pulse.

Executable-world closure treats both the valid and violating worlds as deltas
of the cumulative installation. External pixels confirm the visible overlay
while receipt-backed geometry remains the causal oracle; neither an in-process
layout world nor pixel-only comparison closes the claim.

**Must ship**

- `UiTextRunReceipt`
- `UiTextBaselineReceipt`
- `UiGlyphBoundsReceipt`
- `UiVisualBoundsReceipt`
- `UiVisualAnchor`
- `UiAlignmentGroup`
- `UiSpacingGroup`
- `UiSymmetryAxis`
- `UiVisualInvariantDeclaration`
- `UiVisualEvaluationQuery`
- `UiVisualEvaluationReport`
- `UiVisualFinding`
- `UiVisualOverlayReceipt`
- declared invariant, advisory, and ad hoc inspection levels

**Acceptance evidence**

- the runtime can answer whether two labels align by baseline or leading edge
- the runtime can evaluate spacing rhythm or equal-width allocation without
  pixel-only guessing
- visual findings can link back to mounted receipt, graph node, declaration,
  and source span
- screenshot-confirmed visual inspection remains secondary to receipt-backed
  geometry rather than replacing it

### Milestone 3.21: AI Agent Inspection Tools and Certification Replay Protocol

This slice turns the evidence substrate into a real agent-facing repair and
inspection interface.

**Platform pulse**

Against the live pulse, an agent must inspect a visible point, identify the
same mounted node a human sees, compare the before/after receipts for the last
file edit, and locate its first denial or successful rebind point from the
retained causal timeline. The response must stay targeted to the selected page
evidence and preserve its frame/edit identity; giant dumps or a parallel
agent-only model do not close the pulse.

A separate certification lane must replay the versioned edit to the same stop
point without opening authority in the live ordinary runtime. Query-owned
replay, when required, is consumed only through `worth-query-replay` from that
certification lane.

Executable-world closure attaches the agent tools to the already-running child
and starts from its externally selected visible point and recorded edit. They
may not launch an agent-only composition root, import runner artifacts as
runtime truth, replace the existing process with replay, or grant replay
controls to tools attached to the live child.

**Must ship**

- formal AI tool registry over the runtime inspection substrate
- screenshot/frame capture tool
- inspect-target tool
- inspect-at-point tool
- diagnostic-relevance tool
- frame-diff tool
- rebind-explanation tool
- ordinary retained-timeline comparison and stop-point explanation
- certification-only replay session, replay cursor, and stop-point selection
- certification replay stop points for parse, semantic lowering, declaration
  artifact, admission, graph touch, obligation selection, Query binding,
  measurement, mounted receipts, host observations, rebind planning, and
  diagnostics
- a typed boundary between ordinary inspection evidence and certification
  replay inputs; certification results may be inspected but may not reenter the
  live application as authority

**Acceptance evidence**

- an agent can debug a failed hot reload without reading giant dumps
- an agent can request only diagnostics relevant to a selected node, source
  edit, or denied rebind
- an ordinary agent can identify the first denial point from retained evidence
  without acquiring replay authority
- a certification harness can replay the last versioned edit to that denial
  point through cert-owned inputs, including `worth-query-replay` only when
  Query reconstruction is actually required
- an agent can compare before/after frames or receipts by aspect scope

### Milestone 3.22: Worth Inspector Surface

This slice adds the human-facing runtime inspector as a projection over the
same evidence substrate the AI harness already uses. Its interaction model
deliberately borrows the useful familiarity of Chrome DevTools without copying
the web's ontology: point at the running product, select what is visible, trial
its appearance, inspect live data separately from schema, and understand cost
without first learning Worth's internal artifact graph.

**Product decision**

Keep the first-use surface small: `Elements`, `Styles`, `Data`, `Schema`,
`Performance`, and `Diagnostics`. Authority graph, aspects, measurement, Query
binding, services, rebind, certification replay reports, and visual evaluation
are contextual drill-down for the current selection or event, not ten more
top-level panels.
Raw identities, receipts, and causal traces use progressive disclosure.

The inspector may be closed, docked on the right, left, or bottom, or detached
into a native window over the same runtime session. Placement changes preserve
selection, exact frame affinity, active tab, filters, and any uncommitted style
trial; detaching creates no second application truth or inspector-only
composition root. Milestone 4 may generalize this minimum placement contract,
and Milestone 16 may deepen the same tools without replacing them.

`Styles` supports bounded, reversible live appearance trials through canonical
appearance and rebind and can produce an exact source-span patch proposal; it
never becomes adapter-local or silently authoritative. `Data` shows current
Query projection values and posture, while `Schema` separately shows declared
view, payload, field, row, and binding shape. `Performance` answers "what
became expensive and why?" from a bounded frame/rebind timeline, selected-
target attribution, and named counters, keeping ordinary work distinct from
replay, capture, and inspection cost. Full profiling breadth remains Milestone
16 work.

**Platform pulse**

Attach the Worth Inspector to the same running pulse rather than a special
inspector fixture. A human opens it, uses point-to-select on a visible pulse
node, sees the canonical mounted highlight, trials one appearance value, and
finds live data, schema, current-frame cost, and the relevant diagnostic
without navigating an artifact dump. The same session exercises right, left,
bottom, and detached placement without losing selection. The inspector and
3.21 agent tools must agree on identity and causal evidence.

Executable-world closure opens and drives the inspector inside the inherited
product child. Inspect mode must consume its selection gesture without
admitting the product intent underneath it. Docking, detaching, style trial and
reset, Data/Schema separation, and Performance attribution must stay in that
child; an inspector-only application cannot close the pulse.

**Must ship**

- open, close, right-dock, left-dock, bottom-dock, and detached-window posture
  over one inspector session
- point-to-select mode with mounted highlight, explicit visible-contributor
  versus hit-test outcomes, overlap choice, and stale-selection posture
- `Elements`: visual tree, mounted/declaration/graph breadcrumbs, source
  provenance, participation, aspects, measurement, services, and relevant
  evidence
- `Styles`: authored and resolved appearance, theme and state axes, bounded
  reversible live trials, reset, and exact source patch proposals
- `Data`: live scalar and collection projection facts, Query posture,
  invalidation, and receipt provenance
- `Schema`: declaration, view, payload, field, row, and binding shape with
  compatibility and mismatch evidence
- `Performance`: bounded frame/rebind timeline, selected-target attribution,
  named structural counters, measured timing posture, and lane separation
- `Diagnostics`: relevance-filtered typed findings with causal drill-down
- contextual authority graph, aspect, measurement, Query binding, service,
  rebind, certification replay-report, and visual-evaluation detail without
  additional top-level panel sprawl
- keyboard-operable tab, tree, selection, docking, and close workflows

**Acceptance evidence**

- the inspector consumes the same evidence substrate as the AI tools
- inspect mode selects the exact presentation-bound mounted target, publishes
  its highlight through the canonical mounted path, and does not trigger the
  underlying product interaction
- selection, frame affinity, tool state, and live style trial survive every
  placement change without remounting the product or forking runtime truth
- a live style trial visibly changes the selected target through canonical
  appearance and rebind, reset restores the admitted appearance, and the
  proposed persistent edit points to the exact authored span
- `Data` updates from real Query projection truth while `Schema` remains bound
  to declared shape; a schema mismatch cannot be misreported as an absent or
  stale value
- `Performance` identifies the selected frame and target's named work without
  charging certification replay, capture, or inspector materialization to the
  ordinary frame lane
- advanced graph, aspect, measurement, service, rebind, certification replay,
  and visual evidence remains reachable from the selected item without making
  the common path read like an internal artifact browser
- the inspector can be authored through Worth UI where feasible
- the inspector never becomes the source of diagnostic, Query, schema, style,
  or performance truth

### Milestone 3.23: Hot Composition Certification Vertical Slice

This slice is certification, not product scope broadening. It proves the
runtime architecture through one hostile, realistic workflow.

**Scenario**

```text
Workflow Editor Page
  left: step list
  center: workflow graph canvas
  right: selected step inspector
```

This is the second canonical executable-world regime, not the first executable
entry. It reuses 3.10.3's product-process progression, platform actions,
external observations, failure artifacts, and teardown while adding the
workflow-specific installed world and adversarial sequence.

**Must prove**

- hot page/mosaic edits change graph topology through declarations
- resize invalidates allocation without broad unrelated rebind
- selected-step schema comes from Query projection facts
- field swaps preserve compatible identity where admitted
- dropdowns open through the runtime portal service
- focus is routed through the runtime focus service
- motion uses the runtime motion service
- submit payloads are admitted against Query/schema posture
- invalid payloads emit mounted diagnostics
- host adapters never decide disabled/visible/valid meaning
- receipts prove bounded rebind
- unrelated nodes remain untouched

### Acceptance Evidence For The Series

- the same running app can accept valid replacement declarations produced from
  file-authored UI or Rust-authored composition through the same activation and
  rebind pipeline
- equivalent replacements classify as no-op where the series says they should
- valid reloads preserve eligible durable state and explicitly replace or drop
  ineligible state
- invalid reloads preserve the previous admitted mounted truth while surfacing
  typed diagnostics
- execution-plan lowering, mounted receipts, observation intake, Query binding,
  intent routing, service routing, and diagnostics all prove their work through
  runtime-owned artifacts instead of renderer-local helpers
- steady-state frame execution proves source parsing, artifact validation,
  registry string lookup, and broad artifact scans remain absent
- every human-visible pulse claim runs through the exact product binary in the
  sole cumulative executable-world target and joins external native
  consequences to product-issued causal evidence
- in-process integration, executable product entry, certification replay, and
  inspection remain distinct proof lanes; no aggregate of narrower evidence
  silently claims a stronger boundary
- the hostile Workflow Editor world reuses established installation, process,
  platform, action, observation, artifact, and teardown infrastructure

### Sequencing Notes

- Milestone 3.1 through 3.23 replace the old single Milestone 3 runtime lump
  with a narrower authority-first sequence
- Milestone 3.10.3 is the permanent executable-world foundation; every later
  pulse milestone extends it rather than creating its first real product entry
- Milestone 3.14.1 has retired the interim `egui` host; 3.15 and 3.16 build
  directly on the closed aspect-native host and may not restore adapter-owned
  semantics
- before later Query-facing breadth, `worth-ui-query-binding` must replace its
  remaining raw Query imports so the `worth-query-decl` and `worth-query-host`
  audience boundaries are its only ordinary Query edges; 3.15 may proceed
  without widening that edge because its service authority is UI-local
- `ai-diagnostics.md` co-develops across the full 3.x series; each runtime
  family must become inspectable as it lands instead of waiting for the end
- the formal AI inspection harness begins in Milestone 3.1; later milestones
  enrich it with real evidence families, visual capture, comparison, and
  inspector projections, while certification replay remains a separate
  cert-owned lane
- the DSL vision must co-develop with Milestone 2 and Milestone 3.2 through
  3.21; sugar follows admitted runtime lanes instead of running ahead of them
- Milestones 4 through 7 now build on this substrate instead of reopening
  runtime authority, layout truth, Query posture, or interaction ownership
- detailed specs should split into milestone-3.x docs as each slice begins
  rather than trying to keep one giant Milestone 3 spec honest
- any future Milestone 3.24 begins after executable installation, launch,
  native action, observation, identity correlation, failure retention, and
  teardown are already mature. Its work may refine quality and polish but may
  not introduce the series' first honest product-world machinery.

## Milestone 4: Application Shell and Workspace Layout

### Goal

Build real desktop shell products on top of the Milestone 3.x hot composition
substrate by owning the app shell, workspace model, and persisted layout
semantics rather than leaving them to every downstream application.

### Must Ship

- mosaic as the primary structural layout model for shell and page composition
- multi-window application model
- nested mosaic regions that can split, stack, overlay, and pin
- region-level sizing contracts such as fixed, fill, ratio, bounded, and hug
- explicit scroll ownership and grow-then-scroll behavior at the region level
- dock, split, tab, sidebar, bottom-panel, and status-surface layout primitives
  expressed through or alongside the mosaic model where appropriate
- persisted workspace layout and restore semantics
- menu bar, toolbar, command palette, context menu, dialog, and modal-sheet
  shell surfaces
- active document, active panel, and active window routing contracts
- enough shell polish that one real workbench-style app can be built without
  custom layout infrastructure

### Must Preserve

- workspace layout remains a platform artifact, not a pile of widget-local
  geometry state
- mosaic remains the structural space-allocation language rather than
  collapsing into a grab bag of unrelated layout models for ordinary shell work
- shell meaning remains command-routed and identity-stable across reloads and
  restore flows
- shell does not force app authors to choose between multi-window support and
  hot-lowered composition
- persisted shell state remains distinct from authoritative runtime truth
- Milestone 4 must consume the page, mosaic, measurement, receipt, and rebind
  substrate from Milestone 3.x rather than reopening renderer-owned layout or
  shell-local topology state

### Acceptance Evidence

- one sample workbench can open, close, dock, split, tab, persist, and restore
  its shell without app-local shell logic
- one nested mosaic shell can express pinned sidebar, stacked scroll regions,
  and overlay surfaces without DOM-style height or overflow hacks
- shell state restore is deterministic enough that restart and recovery do not
  invent layout drift
- command palette, menus, and context surfaces all project the same command
  backbone
- workspace layout edits can survive hot reload when stable IDs remain intact

## Milestone 5: Command Spine, Focus, Selection, and Keyboard Routing

### Goal

Broaden the interaction substrate closed in Milestones 3.13 and 3.14 so
actions, focus, selection, and keyboard workflows become rich platform
semantics instead of widget-local conventions.

### Must Ship

- canonical command registry with stable identifiers, labels, shortcuts, icons,
  and readiness or posture hooks
- focus model and traversal rules for shell, widgets, dialogs, and command
  surfaces
- selection primitives strong enough for tables, trees, inspectors, canvases,
  and multi-panel workflows
- keyboard routing and shortcut conflict handling
- owner-admitted undo and redo presentation surfaces tied to command identity;
  until an operation owner publishes that capability, the surface remains
  typed unsupported rather than UI-simulated
- command-projection surfaces for menu items, toolbar items, palette entries,
  and context actions

### Must Preserve

- command meaning stays canonical across all projections
- focus and selection state remain identity-bound rather than implicit widget
  side effects
- keyboard ergonomics do not bypass accessibility semantics
- command readiness does not collapse into generic booleans when runtime
  posture can remain structured
- command identity alone does not authorize undo or redo; the operation owner
  retains history, legality, execution, and recovery authority
- Milestone 5 must deepen command, focus, selection, and keyboard richness on
  the admitted interaction and service substrate rather than reintroducing
  host-local callback routing or native-widget focus truth

### Acceptance Evidence

- the same command can be invoked consistently through button, menu, palette,
  shortcut, and context-entry surfaces
- focus traversal and selection behavior remain deterministic across reload,
  restore, and multi-window flows
- command conflicts and invalid routes surface structured diagnostics rather
  than silent precedence accidents
- when an operation owner admits undo or redo, its presentation can name the
  action and show the owner's current posture without becoming the history
  authority

## Milestone 6: Query-Bound Views and Live Surface Binding

### Goal

Broaden the minimal Query binding and projection-consumption substrate from
Milestone 3.13 into serious data surfaces that bind to declared Query meaning
instead of app-local caches, host-shaped events, or widget-owned live-update
folklore.

### Must Ship

- table or grid surface bound to collection queries with typed columns,
  ordering, cursor semantics, virtualization, and query-shaped patches
- tree or graph-navigation surface bound to bounded traversal or grouped
  neighborhood semantics
- inspector or detail surface bound to detail and inspector view shapes
- timeline or history surface bound to historical and diff-capable Query views
- live view binding that promotes one-shot declared meaning into ongoing
  runtime-backed delivery without raw event handling in the widget
- typed surface contracts for selection, sort, filter, and visible-state
  bindings that remain subordinate to Query meaning rather than replacing it

### Must Preserve

- Worth UI does not become the owner of query legality, basis semantics, or
  truth authority
- UI declarations cross `worth-query-decl`, host progression crosses
  `worth-query-host`, and ordinary view binding does not import raw
  `worth-query` or certification-only `worth-query-replay`
- table, detail, grouped, timeline, and inspector semantics, plus query
  planning, saved-query meaning, projection consumption, and typed fact
  receipts remain Query-owned runtime lanes rather than UI-local data-source
  abstractions
- live updates remain query-shaped rather than raw CDC or raw widget events
- view surfaces remain honest about policy masks, unsupported families, denied
  basis combinations, and deferred capability rows
- app authors do not need local caches to keep core surfaces usable
- Milestone 6 must build richer table, tree, inspector, and timeline surfaces
  on the admitted binding substrate instead of reopening local Query clones,
  local result-state models, or renderer-owned live semantics

### Acceptance Evidence

- one table, one tree, and one inspector surface can be driven from declared
  Query meaning with live updates and no app-local cache repair layer
- equivalent declared views receive equivalent live update behavior across
  reload and restart boundaries where the runtime supports it
- unsupported or denied view bindings fail explicitly and typed
- Query-bound surfaces can explain what view meaning, basis, or runtime posture
  they are currently presenting

## Milestone 7: Forms, Validation, and Editing Workflows

### Goal

Broaden the interaction, Query-binding, and diagnostic substrate from
Milestones 3.12, 3.13, and 3.18 so forms and editing become a platform
capability rather than a loose pile of input widgets, local booleans, and
submission folklore.

### Must Ship

- form surface model with field binding, draft values, dirty or touched state,
  reset or revert behavior, and validation presentation
- local validation and runtime-backed validation or admission presentation
- typed submit, retry, cancel, and reset flows
- inline, grouped, and panel-level error presentation
- editing flows that can participate in Query-bound detail or inspector
  surfaces without collapsing into local truth ownership
- one editing example proving that a serious form can be built without app-local
  form framework code

### Must Preserve

- forms remain subordinate to platform interaction and runtime posture models
- draft or editing state does not masquerade as authoritative truth
- runtime validation or admission stays structured rather than flattened into
  one error string or one boolean
- async or result-state posture, recovery, preview, and ordinary outcome
  semantics compose with existing Query/runtime lanes rather than a Worth-UI-
  owned form status model
- form behavior remains accessible, keyboard-usable, and hot-reload-safe
- Milestone 7 must consume the runtime-owned intent, operability, binding, and
  evidence substrate instead of inventing a Worth-UI-local form framework above
  it

### Acceptance Evidence

- one nontrivial form supports edit, validation, submit, retry, reset, and
  runtime error presentation without custom app-local form infrastructure
- local validation and runtime-backed validation remain distinguishable in the
  UI and diagnostics
- form reloads preserve the stable state they should and explicitly replace the
  state they should not preserve
- form submission results can surface structured success, advisory, violation,
  and recoverable outcomes

## Milestone 8: Runtime UX, Preview, Recovery, and Explanation

### Goal

Make runtime posture visible as ordinary UX so Worth UI can present previews,
recoverable failures, structured denials, and explanations without app-local
status folklore.

### Must Ship

- structured runtime-state surfaces for loading, denied, advisory, violation,
  stopped, recoverable, stale, failed, and completed outcomes
- preview, before/after, accept/discard, and commit-oriented workflow surfaces
- runtime explanation and inspection panels for command readiness, view
  posture, mutation evidence, and cross-runtime "why" answers where admitted
- recovery affordances and recovery-surface presentation for typed failures
- one end-to-end preview workflow showing staged or speculative state, review,
  acceptance, and discard
- enough reusable runtime UX that downstream apps do not have to invent their
  own runtime-state taxonomy

### Must Preserve

- Worth UI does not invent a second mutation, recovery, or explanation runtime
- runtime posture stays structured through the UI boundary
- preview and staged states remain distinct from authoritative truth
- recovery briefs, async or result-state posture, projection-consumption facts,
  Query inspection, and cross-runtime causal explanation remain runtime-owned
  contracts that Worth UI presents rather than redefines
- richer diagnostics do not change the operational outcome being presented

### Acceptance Evidence

- the same structured runtime posture can be presented in a button, panel, form,
  preview flow, and diagnostics surface without reinterpretation
- preview flows can be abandoned without authoritative residue
- explanation surfaces can answer what command or view was bound, why it was
  denied or advised, and what recovery path exists where admitted
- app authors can build one nontrivial review or recovery flow without creating
  a second status model above the runtime

## Milestone 9: Design System and Professional Component Set

### Goal

Turn Worth UI from shell-plus-binding infrastructure into a coherent,
production-grade component system that downstream products can use without
rebuilding basic desktop UX.

### Must Ship

- semantic token system for surfaces, text, borders, accent, warning, danger,
  success, selection, focus, overlays, and runtime states
- light, dark, high-contrast, and custom-theme support
- density modes and component-state contracts
- layout composition semantics for seam ownership, spacing resolution,
  scroll-container behavior, resize behavior, and region-edge behavior within
  the mosaic model
- professional workbench component set including tables, lists, trees,
  inspectors, split panes, tab bars, toolbars, searchable selects, breadcrumbs,
  notifications, logs, progress views, and file or project browser surfaces
- visual-state consistency rules across the component library
- one sample app proving components compose into a product rather than a demo
  gallery

### Must Preserve

- component growth must not outrun shell, interaction, or accessibility
  foundations
- semantic tokens remain meaning-bearing rather than raw color piles
- specialized inner surfaces such as forms, tables, lists, trees, and canvases
  remain region content inside the mosaic structure rather than forcing the
  mosaic model to impersonate every inner content layout
- components remain compatible with hot-lowered composition and stable identity
- workbench-grade components stay stronger than ornamental one-off widgets

### Acceptance Evidence

- a realistic data-heavy workbench can be composed from the shipped component
  set without major custom infrastructure
- theme and density changes propagate coherently through the same component set
- layout composition rules are strong enough that common shell and page layouts
  do not require margin or padding folklore, percentage-height hacks, or
  accidental overflow behavior
- components preserve focus, keyboard, accessibility, and runtime-state
  semantics under real product composition
- component surfaces remain narrow enough that platform tooling can inspect them

## Milestone 10: Canvas, Spatial, and Real-Time Product Surfaces

### Goal

Build product-grade canvas, spatial-tool, real-time overlay, and
renderer-integrated UI surfaces on top of the execution lanes and frame-cost
certification established in Milestone 3.

### Must Ship

- canvas and spatial product primitives with pan, zoom, hit testing, overlays,
  snapping, tool state, and command integration
- renderer-facing product surfaces for custom draw passes and world or screen
  projection work
- real-time overlay and HUD product primitives with shader or material-backed
  surfaces
- higher-level tool-state, selection, overlay, and command workflows over the
  Milestone 3 lane substrate
- one sample hostile surface proving platform shell, diagnostics, and runtime
  binding can coexist with a high-frequency render surface
- expanded performance counters and certification scenarios for real-time and
  spatial product workflows

### Must Preserve

- Worth UI does not attempt to own the full volatile scene renderer
- spatial and real-time lanes remain semantically integrated with commands,
  views, and inspection rather than becoming a disconnected side runtime
- performance claims remain counter-backed instead of anecdotal
- hostile product surfaces consume the Milestone 3 execution lanes rather than
  redefining lane mechanics

### Acceptance Evidence

- one canvas-like product surface and one real-time overlay prove the lane
  substrate can support real product interaction
- UI structure remains hot-reloadable while the render surface maintains
  specialized mechanics
- frame counters expose where work is spent on spatial and real-time surfaces
- renderer-integrated surfaces still participate in platform command, focus,
  and diagnostics systems where applicable

## Milestone 11: Persistence, Settings, and Document or Project Workflows

### Goal

Finish the user-visible persistence model so settings, layout, projects,
documents, and recovery state become platform capabilities rather than app-local
storage conventions.

### Must Ship

- typed settings model with user, workspace, and project scopes
- settings-surface composition strong enough for real settings panels
- persisted workspace layout, recent files, and recent project flows
- document or project workflows with dirty tracking, autosave, and recovery
  snapshots
- restore semantics for panels, tabs, and relevant in-progress UI state
- migration posture for persisted platform state

### Must Preserve

- persisted UI and project state remain separate from authoritative domain
  truth unless explicitly routed through runtime contracts
- restore behavior remains deterministic rather than best-effort folklore
- settings do not become an untyped bag disconnected from the platform facade
- autosave and recovery behavior remain explicit rather than ambient

### Acceptance Evidence

- one sample app can restore layout, tabs, settings, and recent project context
  without app-local persistence plumbing
- autosave and recovery snapshots restore a meaningful working session after a
  forced interruption
- settings remain typed and scoped through the same platform contracts used by
  UI composition
- persistence migrations fail explicitly when incompatible rather than drifting
  silently

## Milestone 12: Background Tasks, Diagnostics, and Recovery Tooling

### Goal

Make long-running work, operational diagnostics, and supportability visible as
platform behavior so desktop apps do not freeze, hide failures, or depend on
log archaeology.

### Must Ship

- task model for progress, cancellation, retry, completion, and failure
- status-bar, panel, and notification surfaces for task presentation
- diagnostics surfaces for errors, traces, command history, task history, and
  performance panels
- recovery presentation for task or workflow failures where recovery exists
- one support-oriented diagnostic flow proving a user or developer can inspect
  a failure without raw logs
- one performance panel or profiling surface consuming the platform's named
  counters

### Must Preserve

- task state remains distinct from authoritative truth
- diagnostics richness does not change the task or workflow result
- support surfaces remain platform-native rather than special-purpose app code
- recovery actions remain explicit instead of silent retries or hidden cleanup

### Acceptance Evidence

- one long-running task can be started, observed, cancelled, retried, and
  diagnosed through platform surfaces alone
- one failure can be explained through structured diagnostics without scraping
  implementation logs
- counters and traces remain connected back to named platform operations
- app authors can expose support-grade diagnostics without inventing a second
  infrastructure layer

## Milestone 13: Accessibility and Interaction Quality Completion

### Goal

Close the accessibility and interaction-quality bar as a real product
completion milestone rather than a deferred compliance sweep.

### Must Ship

- accessible roles, names, descriptions, and state semantics across the core
  component and shell set
- focus visibility, keyboard traversal, reduced motion, scaling, contrast, and
  comfort rules enforced through platform behavior
- accessibility inspection tooling sufficient to audit platform surfaces
- screen-reader and keyboard-only support path for core product patterns
- one hostile accessibility pass over shell, command, form, and view surfaces

### Must Preserve

- accessibility remains built into platform primitives rather than layered on
  as opt-in app-local metadata
- keyboard ergonomics and accessibility semantics reinforce rather than fight
  each other
- accessibility completion does not fork the component model into a "special"
  accessibility-only path
- quality rules remain compatible with hot reload and high-density product use

### Acceptance Evidence

- core shell, command, form, table, tree, and inspector surfaces pass a named
  accessibility audit path
- keyboard-only flows remain usable across the same scenarios
- accessibility tooling can inspect platform-generated semantics without
  implementation archaeology
- contrast, scaling, and reduced-motion rules are proven in real sample apps

## Milestone 14: Native Platform Integration and Delivery

### Goal

Make Worth UI shippable as real desktop software with strong platform
integration and delivery mechanics instead of stopping at "the app runs on my
machine."

### Must Ship

- native menus, dialogs, notifications, clipboard, drag and drop, tray, and OS
  theme integration adapters
- file associations, URL handlers, single-instance behavior, and app metadata
  surfaces
- packaging, installers, update-channel support, crash capture, and session
  restore infrastructure
- keychain or credential integration and explicit permission surfaces where the
  platform owns them
- enough release and runtime behavior that one real app can be packaged and
  maintained through platform tooling

### Must Preserve

- native integration remains adapter-shaped and explicit rather than ambient
  host knowledge spread through the app layer
- packaging and delivery work do not redefine app shell or runtime semantics
- crash and update infrastructure remain distinct from authoritative truth
- platform differences stay behind named boundaries rather than leaking across
  app code

### Acceptance Evidence

- one sample app can be packaged and launched with native integration features
  working through platform adapters
- restart after update or crash can restore enough state to feel like a real
  desktop product
- native integration failures surface explicitly and diagnosably
- delivery surfaces remain stable enough to support real release channels

## Milestone 15: Plugin and Extension Architecture

### Goal

Let Worth UI apps grow into platforms without collapsing their shell, runtime
honesty, or security model under extension pressure.

### Must Ship

- typed plugin contribution points for commands, panels, inspectors, settings,
  query views, themes, toolbars, and project templates
- capability and permission model for filesystem, network, credentials,
  commands, panels, runtime mutation, and project access
- runtime-aware extension hooks that consume platform commands, views, and
  structured outcomes instead of lower-runtime internals
- inspection surfaces showing what each plugin contributed and why
- one sample plugin host proving multiple extensions can coexist through the
  same platform contracts

### Must Preserve

- plugin power remains capability-bounded and inspectable
- extensions do not bypass Query-facing or command-facing platform surfaces
- host apps retain shell, accessibility, and diagnostics coherence under
  extension growth
- plugin contribution points remain part of the public platform facade rather
  than deep imports into internals

### Acceptance Evidence

- one host can load multiple plugins that contribute commands, panels, or views
  through typed contribution points without custom per-plugin glue
- capability violations fail explicitly and typed
- plugin-contributed surfaces still participate in focus, accessibility, theme,
  and diagnostics systems
- host inspection can explain what a plugin added and what authority it holds

## Milestone 16: Developer Tooling, Templates, and Platform Inspection

### Goal

Make Worth UI teachable, inspectable, and self-hosting enough that teams can
understand the platform visually instead of learning it through source diving
alone.

### Must Ship

- component gallery
- theme editor and layout debugger that deepen the 3.22 `Styles` and `Elements`
  tools instead of creating parallel inspector shells
- command registry inspector
- accessibility inspector
- richer Query/view tooling that extends the 3.22 `Data` and `Schema` tabs
- profiler and frame-counter tooling that extends the 3.22 `Performance` tab
- screenshot-test harness
- sample templates for workbench, data app, graph editor, runtime inspector,
  dashboard, and plugin host shapes
- one end-to-end platform inspection story that uses the same runtime artifacts
  the platform itself owns

### Must Preserve

- tooling consumes canonical platform artifacts rather than shadow metadata
- templates remain examples of real platform usage rather than special internal
  paths
- the familiar point-select, docking, tab, selection, and progressive-
  disclosure model established by Milestone 3.22 remains the one human
  inspection shell
- inspection surfaces remain diagnostic and educational rather than becoming a
  second imperative editing runtime
- tooling breadth does not dilute facade clarity or runtime ownership

### Acceptance Evidence

- a new team can start from a template and stay within the ordinary platform
  path
- a new user can open the inspector, select a visible element, trial its
  appearance, inspect data and schema, and identify a frame-cost cause without
  learning internal receipt topology first
- platform tooling can explain what a shell, command, artifact, view, or plan
  is doing without source spelunking
- screenshot and inspection tooling can certify real product examples
- sample apps expose roadmap gaps honestly rather than hiding them

## Milestone 17: Worth UI Certification Program

### Goal

Run the full platform certification pass after the remaining product milestones
exist, and create the missing Worth UI test-requirements contract if it does
not yet exist.

### Must Ship

- `_docs/worth-ui/test-requirements.md` as the authoritative acceptance source
  if it does not already exist by this milestone
- named certification suites for hot reload, stable identity carry-forward,
  shell restore, Query-bound view parity, forms and validation behavior,
  preview or recovery flows, accessibility, native integration, plugin
  isolation, and frame-budget certification
- canonical machine-checkable artifact bundles for certification results where
  the platform claims structured proof rather than visual impressions
- explicit distinction between verified platform paths and intentional debt
  paths in certification output

### Must Preserve

- certification remains a proof program, not a feature-discovery bucket
- the platform is judged against its declared ownership boundaries rather than
  app-local workarounds
- end-to-end claims remain tied back to named counters, artifact identities,
  and structured outcomes
- hostile cases remain part of the bar rather than optional extended tests

### Acceptance Evidence

- every prior milestone has named certification coverage, either through the
  Worth UI requirements doc or through milestone-native acceptance programs
- certification runs can prove that hot-lowered composition, platform shell,
  Query-bound views, forms, accessibility, native integration, plugins, and
  frame-budget surfaces behave according to the roadmap claims
- certification artifacts are sufficient for offline review without leaning on
  private host memory or narrative-only logs
- all declared high-value debt paths are either verified closed or still
  explicitly marked as debt

## Per-Milestone Format

For consistency and readability, every milestone in this roadmap uses the same
shape:

- `Goal`
- `Must Ship`
- `Must Preserve`
- `Acceptance Evidence`

## Completion Standard

Worth UI is roadmap-complete only when:

- all foundation-first critical-path milestones are shipped
- all product-complete platform milestones are shipped
- hot-lowered composition, stable identity, and execution-plan swaps are
  proven under hostile edit and reload scenarios
- frame-cost counters and performance certification exist for high-frequency
  surfaces rather than only best-effort profiling
- shell, command, focus, accessibility, Query binding, forms, preview, and
  recovery behavior are platform-owned rather than app-local
- native integration, delivery, plugins, tooling, and certification are strong
  enough that teams can ship and maintain real desktop products on top of the
  platform
- a Worth UI test-requirements program exists and has closed the platform's
  claimed hostile cases rather than leaving them as implied future work

## Companion Documents

- [worth-ui-vision.md](./worth-ui-vision.md)
- [Worth Query AI README](../../workspaces/worth-query/crates/worth-query/docs/AI_README.md)
- [_docs/worth-query/worth_query_vision.md](../worth-query/worth_query_vision.md)
- [_docs/worth-runtime-bridge/worth_runtime_bridge_vision.md](../worth-runtime-bridge/worth_runtime_bridge_vision.md)
- [_docs/worth-relational/worth_relational_vision.md](../worth-relational/worth_relational_vision.md)
- [_docs/worth_signal/worth_signal_vision.md](../worth_signal/worth_signal_vision.md)
