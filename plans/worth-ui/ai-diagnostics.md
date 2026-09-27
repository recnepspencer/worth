# Worth UI AI Diagnostics and Inspection Architecture

## Purpose

This document defines how AI inspection, human diagnostics, visual evaluation,
runtime explanation, and certification-only replay should work in Worth UI.

It is not a debug-feature wishlist, an implementation checklist, or a panel
mockup.

Its job is to lock the architecture early enough that:

- AI support does not become a pile of screenshots plus logs
- the human inspector does not become a second explanation system
- diagnostics do not become ad hoc text dumps
- replay and reconstruction remain certification-owned rather than leaking into
  the ordinary inspection runtime
- visual evaluation does not depend on pixel guessing when runtime geometry is
  already available

The central question is:

```text
How should Worth UI expose runtime truth, evidence, and explanation so that
both AI agents and humans can inspect, debug, and validate the live UI without
creating a second runtime or a second diagnostic folklore layer?
```

## Thesis

The correct move is:

```text
build one runtime-owned evidence and inspection substrate
then expose it through two consumers:
  - AI inspection tools
  - a human inspector surface
```

Do not build "AI diagnostics" and "inspector diagnostics" as separate systems.

Do not treat screenshots as the main truth.

Do not treat logs as the public inspection contract.

The runtime already owns stronger semantic meaning than normal UI stacks:
declarations, admitted identities, aspect contracts, graph topology, Query
bindings, measurement plans, execution-plan decisions, frame-cost receipts,
mounted receipts, host observations, rebind causality, and denial posture.

The inspection architecture must expose that meaning directly.

## Core Rule

The governing rule is:

```text
host code may allocate pixels and report observations
only runtime-owned truth and runtime-owned evidence may explain visible UI
meaning
```

That means:

- renderer code does not explain why something exists, changed, denied, or
  aligned
- logs may present evidence, but logs are not the evidence contract
- screenshots may assist inspection, but screenshots are not the primary truth
- diagnostics panels may project evidence, but panels do not author diagnostic
  truth
- AI tools may query evidence, but tools do not reconstruct meaning from host
  behavior

The host corollary is:

```text
worth-ui-host-contract is the stable host boundary
worth-ui-host-native and worth-ui-host-headless own mechanical implementations
adapter-specific mechanics must not become runtime truth or public host law
```

## Relationship To The Existing Runtime

This document extends the architecture in
[Worth UI runtime orientation](../../workspaces/worth-ui/docs/worth-ui-readme.md) and
the roadmap in [worth_ui_roadmap.md](./worth_ui_roadmap.md).

It does **not** propose a second truth graph.

The better model is:

```text
authority graph = runtime truth about what exists and participates
inspection substrate = typed evidence and indexes over runtime truth,
declaration artifacts, obligations, receipts, observations, and rebind history
```

That distinction matters.

The authority graph is where UI truth lives.

The ordinary inspection substrate is how that truth becomes explainable,
queryable, comparable, and relevant to a specific question. Certification may
consume versioned evidence through a separate replay boundary; ordinary
inspection cannot reconstruct execution or mint replay authority.

If the implementation creates a second graph-shaped authority system for
inspection, it has likely crossed the boundary and started rebuilding the
runtime.

## Why This Must Start Early

AI inspection and diagnostics are not end-of-roadmap polish.

They are runtime pressure.

If they arrive only after declarations, graph ownership, measurement, mounting,
services, and hot rebind are already built, then one of two bad things usually
happens:

- the architecture turns out not to be explainable without local hacks
- the team builds side channels that bypass runtime truth because the formal
  lanes were never made inspectable

Worth UI should take the opposite approach:

```text
every serious runtime family must ship with:
  - truth ownership
  - typed evidence artifacts
  - targeted inspection queries
  - relevance filters
  - certification-owned replay inputs where reconstruction is materially required
```

This is especially important for AI.

An AI agent does not need the full human diagnostics panel on day one. It needs
a formal, typed, scoped entry point into runtime evidence.

That harness should exist in the first serious hot-composition milestone.

## Architectural Goal

The best possible end state is:

```text
AI sees the current frame
-> targets a node, point, or source edit
-> asks a typed inspection query
-> receives a scoped evidence slice with stable identities
-> compares retained before/after evidence without re-executing it
-> patches source at the authored boundary
-> the runtime hot reloads
-> updated evidence proves whether the repair is correct
```

For humans, the same substrate should make it possible to:

- click a visible element and jump to its declaration
- see why a node is visible, hidden, disabled, denied, or remounted
- inspect what aspects were published or consumed
- inspect why a layout box was allocated as it was
- inspect what Query projection the UI consumed
- inspect why a retained change widened or stayed local
- inspect whether alignment, symmetry, spacing rhythm, or focus topology are
  correct

## Non-Goals

This architecture must not devolve into:

- a browser-DevTools clone with web baggage
- a console log surface with better formatting
- a screenshot-only AI workflow
- a renderer-local debug overlay that bypasses runtime receipts
- a second truth runtime for diagnostics
- an inspector that owns diagnostic truth
- a one-off debug surface that only works for the current demo

The goal is not:

```text
teach the AI to read pixels better
```

The goal is:

```text
let the AI ask semantically precise runtime questions using the same language
the architecture already uses
```

### Milestone 3.1 Non-Goals

Milestone 3.1 specifically does not implement:

- DSL parsing
- canonical UI declarations
- authority graph topology
- aspect contracts
- measurement/allocation
- mounted receipts
- visual snapshots
- certification replay
- AI screenshot tools
- human inspector UI
- Query projection binding
- portal/focus/motion services

Milestone 3.1 only creates the enforced homes, public lifecycle, inspection
contract shape, support posture, unsupported posture, and anti-bypass proof
that later milestones must use.

## Boundary Matrix

| Crate | Owns | Must Not Own |
| --- | --- | --- |
| `worth-ui` | product facade | runtime internals |
| `worth-ui-dsl` | source/semantic DSL boundary | graph truth |
| `worth-ui-runtime` | hot-composition truth | host mechanics |
| `worth-ui-inspection` | ordinary inspection contracts/evidence | panel UI truth, replay, or reconstruction |
| `worth-ui-query-binding` | Query consumption boundary | Query authority |
| `worth-ui-host-contract` | native host boundary facts | vendor mechanics |
| `worth-ui-host-native` | native mechanics | UI meaning |
| `worth-ui-host-headless` | deterministic recording mechanics | UI meaning |
| `worth-ui-certification` | anti-bypass proof and replay/reconstruction | production runtime truth or ordinary authority |

## Evidence Substrate

The inspection substrate should own typed evidence families such as:

- declaration artifact evidence
- admission and denial evidence
- authority-graph identity and topology evidence
- aspect publication and consumption evidence
- graph-touch and obligation-selection evidence
- Query binding and projection-consumption evidence
- measurement and allocation evidence
- execution-plan lowering, equivalence, activation, and frame-cost evidence
- mounted receipt evidence
- host-observation evidence
- semantic interaction and admitted-intent causal evidence
- bounded portal, focus, motion, command-routing, scroll, selection, proposal,
  and service-resource evidence
- rebind and preservation evidence
- diagnostic evidence
- visual snapshot evidence
- visual geometry and visual invariant evidence
- retained causal timeline and before/after comparison evidence
- certification-owned replay timeline and replay-step evidence, never ordinary
  inspection authority

In the earliest support-bearing slice, the substrate should also own support
and closure artifacts such as:

- `UiInspectionSupportReport`
- `UiInspectionScopeSupportRow`
- `UiInspectionClosureReport`

Each family must have:

- stable identity
- provenance
- semantic category
- causal links to upstream and downstream evidence
- source-span readiness where relevant
- relevance filtering support
- materialized detail that can be loaded lazily

Support reporting is not optional garnish.

The runtime should be able to say, structurally:

```text
scope: measurement
status: unsupported
reason: belongs-architecturally-not-yet-admitted
milestone_expected: 3.6 or later
```

That is stronger than returning `unsupported` from a query without saying what
the architecture expects.

### Evidence Is Not Logs

Logs are presentation.

Evidence is typed runtime output.

The public lane should be evidence first, with logs, test output, and UI panels
rendering projections of that evidence.

This prevents the usual failure mode where tools start matching strings instead
of consuming typed structure.

### Evidence Must Be Indexed

The substrate should expose indexes that make targeted questions cheap and
honest.

Important indexes include:

- declaration identity -> evidence sets
- source span -> declaration / diagnostics / rebind evidence
- graph node identity -> obligations / bindings / receipts / diagnostics
- active plan identity -> lowering basis / equivalence decision / activation /
  lane receipts / frame-cost evidence
- plan handle -> exact plan generation / handle family / admitted target
- mounted receipt identity -> visible region / source / graph node / services
- published aspect -> publishing nodes / receipts
- consumed aspect -> dependent nodes / obligations / receipts
- changed fact -> affected rebind evidence
- diagnostic identity -> attached evidence neighborhood
- screen point or region -> mounted receipt identity
- frame identity -> visible snapshot / mounted receipts / causal timeline entry
- certification replay step -> versioned replay input / produced comparison evidence
- semantic interaction input -> exact intent evidence reference
- admission or attempt generational slot -> exact intent causal record

Without these indexes, inspection drifts back toward scans, dumps, and
hand-built explanations.

### Implemented intent causal index

Milestone 3.14 implements one compact ordinary-lane intent evidence index. A
presentation-bound semantic interaction receives an opaque
`UiIntentEvidenceReference`; the runtime carries it through route, payload
owner revisions, operability, admission, attempt, and product outcome. Stable
lookup returns `Found`, `Expired`, or `ForeignSession` and never reconstructs a
missing record.

The index is a 64-entry semantic-only replacement ring with an exact byte
capacity. Input correlation and admission/attempt updates are preindexed; they
do not scan retained evidence, renderer state, motion history, or logs. Platform
Pulse adds consequence publication, exact Query projection, and mounted frame
to the same reference. Its executable world supplies the independent visible
pixel oracle.

This is compact causal evidence, not the final rich neighborhood system. Lazy
detail materialization, disclosure-controlled payload detail, and arbitrary
causal graph traversal remain later work. Replay and reconstruction remain
certification-only work. Ordinary
inspection cannot turn copied references, equal digests, serialized reporting
projections, or expired records back into admission or execution authority.

### Implemented runtime-service evidence

Milestone 3.15 adds one compact current summary per service owner and an exact
live resource census. The stable public questions are:

- `why_portal_closed`
- `why_focus_moved`
- `why_focus_restoration_failed`
- `why_motion_interrupted`
- `why_scroll_owner`
- `why_selection_dropped`
- `why_command_won`
- `runtime_service_resource_census`

Each summary carries `UiRuntimeServiceInspectionSource`: the exact family,
optional owner identity, and revision of the evidence being projected.
`UiRuntimeServiceInspectionCost` states the owner-index lookups, retained
records examined, projected items, and world records scanned. Command routing
may project a bounded winner/loser explanation;
ordinary inspection does not retain the entire candidate catalog. Portal,
Focus, Motion, Scroll, and Selection likewise project their current bounded
decision, not event history.

The resource census covers live family records, proposal occupancy, command
prefixes and routes, motion tracks, and portal exit retention. Physical focus
placement is separate: `UiFocusHostPlacementShutdownReport` records an
abandoned indeterminate request, if any. Both are disposal evidence, not cleanup
commands. AI and human tools can identify residual work, but cannot remove it by
dropping a projection or reconstruct operational authority from reported
counts.

These summaries are ordinary-lane evidence. Detailed protocol fault scripts,
scale oracles, full motion samples, every scroll delta, and replay belong to
certification or opt-in diagnostics. Tools must not scrape logs or read mutable
owner fields when the bounded public summary answers the question.

## AI Inspection Protocol

The AI should not receive a random bag of debug commands.

It should receive a typed inspection query surface.

Example shape:

```text
UiInspectionQuery {
  target:
    - declaration(identity)
    - graph_node(identity)
    - mounted_receipt(identity)
    - source_span(file, range)
    - screen_point(x, y)
    - screenshot_region(identity)
    - diagnostic(identity)
    - frame(identity)

  scope:
    - declaration
    - admission
    - graph
    - aspects
    - obligations
    - query_binding
    - measurement
    - execution_plan
    - plan_equivalence
    - frame_cost
    - mounting
    - host_boundary
    - services
    - rebind
    - diagnostics
    - visual_geometry

  richness:
    - summary
    - evidence_refs
    - materialized_detail
    - causal_trace

  relevance:
    - attached_only
    - nearest_cause
    - denied_only
    - unsupported_only
    - changed_since(frame)
    - source_edit(change_identity)

  budget:
    max_nodes
    max_edges
    max_bytes
}
```

This is the right shape because it forces inspection to stay:

- target-aware
- scope-aware
- budget-aware
- relevance-aware
- identity-backed

The companion support/reporting lane should be able to answer:

- does this target/scope belong architecturally?
- is it admitted yet?
- if not, why not?
- when is it expected to become admitted?

The point is to make “not yet” machine-checkable instead of forcing clients to
guess from missing behavior.

Replay is not another `UiInspectionQuery` target or scope. Certification owns a
separate query and result family whose inputs are versioned source, declaration,
recorded observation, and external-evidence artifacts. Query reconstruction may
enter that family only through `worth-query-replay`; ordinary Worth UI and
`worth-ui-inspection` cannot depend on it.

### AI Tools

The formal tool lane should support capabilities like:

- `capture_frame`
- `capture_node`
- `inspect_at_point`
- `inspect_target`
- `inspect_source_span`
- `inspect_diagnostic`
- `inspect_rebind`
- `inspect_layout`
- `inspect_query_binding`
- `inspect_execution_plan`
- `inspect_host_observation`
- `explain_visibility`
- `explain_operability`
- `explain_allocation`
- `explain_plan_equivalence`
- `explain_frame_cost`
- `explain_rebind`
- `why_portal_closed`
- `why_focus_moved`
- `why_focus_restoration_failed`
- `why_motion_interrupted`
- `why_scroll_owner`
- `why_selection_dropped`
- `why_command_won`
- `runtime_service_resource_census`
- `diff_frames`
- `list_relevant_diagnostics`
- `compare_evidence_points`
- `evaluate_alignment`
- `evaluate_spacing`
- `evaluate_symmetry`
- `show_visual_overlay`

The important part is not the tool count.

The important part is that each tool must query the same evidence substrate
instead of scraping logs, reading private host fields, or interpreting pixels
in isolation.

Certification tooling may separately expose `start_replay`, `step_replay`, and
`compare_replay_points`. Those commands operate on a certification-owned replay
session, never the live application's ordinary authority graph, and their
reports cannot be submitted as application, Query, plan, rebind, or publication
authority.

Inspection receipts must be sealed but projectable.

That means:

- public consumers may inspect identity, target, scope, posture, budget, and
  evidence refs
- public consumers may project those receipts into AI responses, tests, logs,
  or inspector UI
- only the inspection runtime may construct receipt identities, posture
  witnesses, support rows, or evidence claims

The wrong alternatives are:

- externally WORTHable receipts, which make proof meaningless
- privately unreadable receipts, which make the runtime impossible to consume

## Visual Snapshots

Worth UI supports screenshots, but a screenshot is not the public truth
object. The implemented public object is
`UiVisualSnapshotReceipt<ArtifactPosture>`: a bounded, immutable evidence
bundle for one exact presentation basis.

Its implemented shape includes:

```text
UiVisualSnapshotReceipt<ArtifactPosture> {
  UiVisualSnapshotIdentity
  UiVisualSnapshotAffinity {
    presentation_attempt
    frame
    semantic_surface
    host_surface
    binding_generation
    presentation_epoch
    relation
  }
  UiVisualCoordinateObservation
  UiVisibleRegionIndexIdentity
  UiHitTestRegionIndexIdentity
  optional or required UiVisualPixelArtifact
  UiVisualInspectionCostReceipt
  UiVisualSnapshotEvidence
}
```

Coordinates created through `UiVisualCoordinateScope` cannot escape their
snapshot. Point and region adjudication keep visible contributors distinct from
the total-ordered hit-test target. Each result may carry
`UiVisualIdentityTrace`, which follows mounted receipt and incarnation through
graph and declaration identity to authored source provenance and typed evidence
references.

The host supplies capture and presentation observations. The runtime supplies
the meaning those observations explain. AI and human consumers project the
same receipt, omission, denial, indeterminate posture, cost, and lifecycle
evidence; neither consumer reconstructs authority from pixels.

### Implemented Visual Snapshot Closure: Milestone 3.11

The 3.11 snapshot lane is honest because the system can:

- capture the current frame, a retained presentation, a selected mounted node,
  or a snapshot-scoped region;
- map a client physical point or region to distinct visible and hit-test
  outcomes;
- map mounted receipt identity to graph, declaration, authored source, and
  evidence references;
- retain, supersede, expire, cancel, dispose, and shut down snapshots through
  typed bounded lifecycle outcomes;
- publish and clear a fixed identity overlay through successor mounted frames;
  and
- keep optional pixels as disposable evidence rather than truth.

### Committed Successor: Milestone 3.12

Identity-aware predecessor/successor snapshot comparison is not a 3.11
capability. Milestone 3.12 owns the first admitted comparison that can relate
changed facts, preserved identity, remount decisions, and bounded hot rebind.
Until that contract exists, neither raw pixel diff nor local identity matching
may be presented as semantic rebind evidence.

## Certification Replay

Replay and reconstruction are certification-only protocols. The ordinary
runtime and human inspector expose retained causal timelines, evidence
comparison, and explanations; they do not re-execute prior authority or rebuild
missing state. Certification replay is first-class because it has its own
versioned inputs, resource budget, lifecycle, and non-authoritative results,
not because it is available to ordinary application code.

Certification replay lets reviewers and agent tooling answer:

- what changed?
- when did the meaning diverge?
- where was the first denial introduced?
- why did this rebind widen?
- why did this node preserve identity?
- why did this node remount?

### Certification Replay Stop Points

The certification surface should support meaningful stop points such as:

- after parse
- after semantic lowering
- after canonical declaration artifact
- after admission
- after graph touch
- after obligation selection
- after Query binding
- after measurement planning
- after committed allocation
- after execution-plan lowering
- after plan equivalence/no-op classification
- after plan activation
- after frame execution
- after mounted receipts
- after host observation intake
- after rebind planning
- after diagnostics

That enables targeted questions like:

- replay the last edit until admission
- replay the failed change until the first denial point
- compare before/after mounted receipts by aspect
- stop at rebind planning and explain why breadth widened

At the Query-binding stop, Worth UI certification may inspect the exact retained
Query publication or projection evidence. If Query semantic replay is required,
the certification owner must use `worth-query-replay`; it may not reconstruct a
Query operation from UI facts, digests, reports, or mounted evidence. No replay
result can re-enter the live application as authority.

## Relevance Filtering

One of the most important parts of this architecture is relevance.

The AI almost never wants "all diagnostics."

It wants:

- diagnostics relevant to this node
- diagnostics relevant to this source edit
- diagnostics relevant to this failed rebind
- diagnostics relevant to this screenshot region
- diagnostics relevant to unsupported aspects
- diagnostics relevant to this denied service attachment

The same rule applies to humans using the inspector.

The inspection substrate must therefore provide relevance filters as first-class
runtime features rather than leaving the panel or tool client to guess.

## Human Inspector Surface

The human-facing surface should not be framed as "browser DevTools for Worth."

That mental model is too web-shaped.

This surface is better described as:

- Worth Inspector
- Runtime Inspector
- Evidence Inspector
- Composition Inspector

Its job is to inspect runtime authority and runtime evidence, not HTML nodes or
CSS style cascades.

### Inspector Views

The following are required evidence views, not a mandate for one top-level tab
per item. The human inspector keeps the small task-oriented shell defined by
Milestone 3.22 and exposes these as contextual drill-down for the current
selection or event.

Useful evidence views include:

1. Visual Tree  
   Mounted receipt topology, visible region identity, declaration identity,
   graph node identity, and diagnostic badges.

2. Authority Graph  
   Runtime topology for pages, page sets, mosaics, regions, controls, portals,
   focus scopes, services, and diagnostics attachments.

3. Aspect Inspector  
   Published aspects, consumed aspects, changed aspects, aspect coverage,
   aspect-fit denials, and aspect-sensitive obligations for the selected target.

4. Rebind Timeline  
   Changed fact, affected aspects, invalidated obligations, preserved nodes,
   remounted nodes, denial artifacts, and updated mounted receipts.

5. Measurement Inspector  
   Constraints, sizing mode, intrinsic requests, host measurements, allocation
   plan, overflow posture, and scroll ownership.

6. Query Binding Inspector  
   Query artifact consumed, basis/world posture, projection facts, schema
   posture, async/result posture, payload shape, and invalidation posture.

   The current projection evidence family correlates a Query transition or
   attempt, shape-specific affine fact, application generation, mounted node
   and frame, presentation attempt, and visible pixels. Availability,
   current/stale activity, stop kind, compatibility, native family,
   collection continuation, and structural cost remain separate typed fields.
   Compact identities are retained first; detail is materialized lazily under
   explicit evidence, disclosure, and retention budgets.

   Support reporting distinguishes the installed product backend and consumer
   contract from unsupported, remasked, wrong-world, stale-generation,
   incompatible-schema, and expired-detail posture. A matching identity,
   digest, diagnostic string, pixel, or inspection record cannot construct a
   binding, fact, Query operation, rebind plan, or publication authority.

7. **Execution Plan And Frame Cost** -- lowering authority, active plan
   generation, host-neutral lane partitions, typed handle families, exact
   equivalence/no-op decision, affected closure, activation receipt, and
   ordinary versus reconstructive cost counters.

8. **Interaction And Intent Inspector** -- host observation sequence, exact
   presented-frame target, semantic interaction, route binding, reusable intent
   definition, payload/input-basis revision, orthogonal operability decision,
   confirmation challenge posture, UI admission, framework-owned attempt,
   product/domain outcome, declared consequence, and mounted visible posture.
   Compact evidence can explain the chain but cannot target, confirm, complete,
   retry, or readmit it.

9. **Services Inspector** -- portal topology, focus routing, motion, command
   routing, selection, scroll, and other runtime services.

10. **Diagnostics Feed** -- typed, filterable diagnostics grouped by relevance,
   not a console.

11. **Causal Timeline** -- retained source edits, artifacts, admissions, graph
    mutations, observations, rebinds, mounted frames, and diagnostics. A
    certification replay report may be projected here, but the inspector cannot
    start replay or reconstruct execution.

12. **Visual Evaluation** -- alignment groups, baselines, spacing rhythm,
    symmetry axes, visual bounds, overlays, invariant violations, and
    perceptual advisories.

### Dogfooding Rule

The inspector should, where feasible, be authored and rendered through Worth
UI itself.

That proves:

- diagnostics can mount through the standard path
- inspection can be product-facing rather than side-loaded
- Query bindings can power complex support surfaces
- services such as portals and focus survive pressure

But the inspector must never become the source of truth.

It consumes runtime evidence. It does not mint it.

## Visual Evaluation And Design Invariants

Worth UI should support a category stronger than "eyeball the screenshot."

This document calls that category:

```text
visual evaluation and design invariants
```

This family owns questions like:

- do these text baselines align?
- do these labels share a leading edge?
- do these inputs occupy equal allocated width?
- is spacing rhythm consistent?
- is this icon optically centered with its text?
- are these controls symmetric around the parent centerline?
- do visual bounds match declared or allocated expectations?
- did geometry stay correct while paint visually drifted?

### Two Kinds Of Visual Evaluation

Worth UI should distinguish:

1. Runtime-semantic checks  
   These are based on receipts, anchors, baselines, bounds, groups, and
   declared tolerances. They should be deterministic and inspectable.

2. Perceptual checks  
   These are based on screenshot pixels, rasterization, anti-aliasing, and
   visual mass. They are valuable, but usually advisory unless promoted into a
   declared invariant.

The rule is:

```text
receipt-backed first
screenshot-confirmed second
```

### Visual Evidence Families

Examples of useful evidence families:

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

### Advisory vs Blocking

Not every visual finding should fail the world.

Worth UI should support at least three levels:

- declared invariant violation
- design advisory
- ad hoc inspection result

This allows authors to say:

- "these labels must align" as a declared invariant
- "this region feels optically heavy" as an advisory
- "show me near misses within 1px" as an ad hoc inspection query

## Diagnostics Are Not Special-Cased Text

Diagnostics must remain typed runtime artifacts.

They should preserve:

- identity
- source declaration identity
- world and support posture
- aspect posture
- denial or degraded reason family
- affected artifact identities
- evidence references
- relevance metadata
- presentation rows or projections

Tests and tools should consume those typed artifacts.

They should not match the final display string and pretend that is proof.

## Formal AI Harness

The first serious hot-composition milestone should establish a formal AI entry
point.

That harness is more important than a polished panel in the early sequence.

### Harness Requirements

At minimum, the AI harness should expose:

- frame capture by identity
- point and region hit testing
- target inspection queries with scoped evidence
- relevant-diagnostics lookup
- retained causal-timeline comparison
- certification replay session creation and stepping through a separate
  certification surface
- rebind explanation
- frame diff by identity and aspect scope
- execution-plan lowering, equivalence/no-op, activation, and frame-cost
  inspection without exposing executable plan ownership
- visual evaluation queries for alignment, spacing, and symmetry

The harness should be:

- typed
- budgeted
- identity-backed
- comparison-capable in ordinary inspection
- replay-capable only through the certification boundary
- independent of ad hoc console output

### Harness Rule

If an AI repair workflow cannot be expressed through the formal harness without
reading a giant dump, the harness is not mature enough yet.

## Per-Milestone Integration Rule

The right roadmap posture is not:

```text
build the runtime first
add AI/diagnostics at the end
```

The right posture is:

```text
every milestone adds the evidence and inspection surfaces for the runtime
families it introduces, plus versioned certification replay inputs where
reconstruction is materially required
```

That means:

- declaration milestones add declaration inspection
- admission milestones add denial and support inspection
- graph milestones add topology and aspect inspection
- measurement milestones add allocation and geometry inspection
- execution-plan milestones add lowering, equivalence, activation, handle/lane,
  and frame-cost inspection
- mounting milestones add visible-region and mounted-receipt inspection
- services milestones add service topology inspection
- rebind milestones add change-diff and preservation inspection
- visual milestones add visual-evaluation inspection

Here, “replay surfaces” means versioned certification inputs and cert-owned
adapters where replay is materially required. It never means an ordinary
runtime import or a reporting artifact that can reopen execution. This keeps
inspection honest and the runtime explainable while preserving the ordinary
versus reconstructive cost and authority boundary.

## Suggested Structural Boundaries

The final crate topology may vary, but the conceptual boundaries should remain
separate:

```text
worth-ui-inspection
  runtime-owned evidence/query/comparison substrate

worth-ui-agent-tools
  AI-facing tool protocol over the inspection substrate

worth-ui-inspector
  human-facing inspector projections over the same substrate

worth-ui-certification
  replay/reconstruction sessions over versioned certification inputs
  optional Query replay only through worth-query-replay
```

The key is not the exact crate count.

The key is preserving the lifecycle boundaries:

- runtime evidence substrate
- AI tool surface
- human panel surface

They should not collapse into one implementation blob.

## Acceptance Standard

This architecture is only real if it can prove all of the following:

- AI can inspect a declaration artifact without a dump
- AI can ask why a node is visible, hidden, disabled, denied, rebound, or
  remounted
- AI can move from screenshot region to mounted receipt identity
- AI can move from mounted receipt identity to declaration, source, graph, and
  evidence
- AI can compare retained evidence for a live change and, through the separate
  certification surface, replay versioned inputs to the first denial point
- AI can explain why a candidate was a semantic no-op, required a bounded plan
  replacement, or was denied; which exact plan generation is active; and what
  work an ordinary frame performed
- humans can inspect the same evidence through the inspector
- diagnostics are relevant, typed, identity-backed, and filterable
- visual alignment and spacing can be evaluated from runtime geometry rather
  than screenshots alone
- the inspector consumes evidence but does not author truth
- plan and Query inspection or replay cannot mint handles, activate candidates,
  promote receipts/digests into authority, or submit an executable plan
- projection evidence cannot be reassembled into a binding or fact, and lazy
  detail cannot widen Query, application, mounted, or disclosure authority
- no explanation path requires renderer-local semantic reconstruction

## Final Rule

Worth UI should aim for something better than conventional UI tooling:

```text
not screenshots plus logs
not devtools plus folklore
not AI guessing from pixels

but a live, semantically indexed runtime that can explain itself through the
same declaration, graph, aspect, Query, measurement, plan, mounting, and
service language that already defines product truth

with replay and reconstruction confined to certification
```

That is the standard.
