# Milestone 3.15: Production Runtime Services

## Status and Placement

Status: Closed on 2026-08-29. Phases 1 through 7 are complete. This document
remains the governing design and acceptance contract for Milestone 3.15.

The integrated closeout source is `df9fc00908`. Native `RS-01`, protocol
`RS-07`, the scheduled `RS-10` scale courtroom, public-facade and DSL
contracts, the focused Pulse suites, formatting, dirty-set line caps, boundary
checks, generated context checks, and documentation-link validation passed on
that source. A product/design review of real native captures at the required
960-by-600 and 1120-by-700 logical client sizes gave a defensible yes to the
contemporary Linear-or-Notion quality bar: the shipped dark mosaic has a clear
hierarchy, restrained palette, deliberate spacing, contained typography, and
coherent truthful service regions. The automated visual contracts remain the
structural oracle; this recorded judgment is the separate aesthetic gate they
cannot replace.

The later `Current Boundary and Exact Gap` section records the predecessor
boundary frozen when this design was authored. It is historical design input,
not a description of the present repository.

Milestone 3.14 closed the path from native human input to one exactly admitted
intent attempt and typed consequences. Milestone 3.15 adds the cross-cutting UI
services that serious desktop products need on that path: portal, focus,
motion, command routing, scroll, and selection.

This milestone does not reopen interaction targeting, operability,
confirmation, generic intent attempt management, or consequence publication.
It consumes those 3.14 contracts. It also does not make the UI runtime a Query
authority, a replay engine, a persistence authority, or an undo/redo manager.

Milestone 3.16 may trust that focus, selection, portal, scroll, and motion state
are coherent inputs to appearance. Later shell, table, canvas, accessibility,
native-integration, plugin, and developer-tool milestones may trust the public
service contracts and evidence defined here without replacing their owners.

## Goal and Central Claim

The UI runtime owns six different semantic services behind one small common
request basis. Cross-service work is compiled by a non-authoritative proposal compiler
into the existing 3.12 observation/publication and mounted-presentation paths:

```text
admitted interaction, rebind, or governed runtime request
-> exact service request basis
-> one family-specific owner
-> typed requirements and proposed consequences
-> coherent proposal compilation, when more than one owner participates
-> family-produced facts and ordinary mounted presentation work
-> existing atomic application/mounted publication
-> existing presentation and host-truth settlement, when effects escape
```

Portal, focus, motion, command routing, scroll, and selection are not variants
of one generic state machine. Each owns its own vocabulary, state, cost model,
rebind law, and lifecycle. The common layer carries identity, currentness,
origin, cancellation, resource budget, and publication basis only. A service
never calls another service. When one service needs another, it emits a typed
requirement. `UiServiceProposalCompiler` above the owners validates one coherent
proposal set and lowers it into the already-owned publication paths. It is not
a state owner, publisher, service executor, or physical-effect settlement
authority.

The central product claim is:

> A designed native workbench can present only truthful product facts, open an
> anchored portal, place focus inside it, animate from committed predecessor
> geometry, route scrolling and selection, invoke the winning command, survive
> a hostile resize/rebind, and close without stale authority, duplicate
> effects, ghost resources, a full-graph scan, or a parallel Query lane.

## Inherited Boundary

Milestone 3.15 inherits these closed truths and does not redesign them:

- the 3.14 presentation-bound native observation, semantic interaction,
  route-binding, payload, operability, UI admission, managed execution,
  terminal outcome, and consequence contracts;
- the 3.12 observation compiler, rebind, atomic publication, and external
  completion contracts;
- the 3.13 Query-backed scalar and collection projection path;
- mounted identity, presentation binding, receipt, allocation, hit-test,
  portal-anchor, scroll-owner, and invalidation evidence already carried by
  the runtime;
- Query consumption only through `worth-query-decl` and `worth-query-host`
  from entry-band crates, and `worth-query-replay` only from cert-band crates;
- concrete proof-carrying authority at governed boundaries; and
- the permanent native Platform Pulse and external observation runner as the
  decisive product-world proof.

An explicit user action may enter a runtime service through the admitted 3.14
intent destination. A rebind, focus restoration, reduced-motion change, clock
tick, host observation, or service-owned continuation enters through its own
existing typed authority. None may forge `UiAdmittedIntent`, and
`UiAdmittedIntent` is never Query or domain mutation authority.

## Non-Goals and Explicit Exclusions

This milestone does not ship:

- undo or redo behavior, history capture, inverse operations, transaction
  grouping for history, or an actionable undo/redo public surface;
- `provisional_aftermath` as a supported contract or compatibility promise;
- Query identity, disclosure, capability, authorization, cursor,
  continuation, snapshot, basis, publication, recovery, or replay semantics;
- ordinary-lane event replay or reconstruction;
- durable persistence of transient service state;
- operating-system-native menus or dialogs, detached document windows,
  system notifications, or host-owned semantic popup state;
- user-editable keymaps, arbitrary shortcut scripting, or a second command
  registry;
- custom kinetic-scroll physics, custom overscroll physics, spring solvers,
  shader animation, or a host-owned animation timeline;
- spatial lasso, two-dimensional range selection, canvas gesture arbitration,
  or Query-result selection authority;
- full accessibility semantics, although focus, reduced-motion, modality, and
  inspection contracts must leave the lawful insertion points for Milestone
  13; or
- a general theme system, state-driven appearance engine, component library,
  dockable shell, or design-tool integration; or
- placeholder modules, empty successor facades, compatibility aliases, string
  escape hatches, or app-local replacement services.

The ordinary 3.15 scenario portfolio is intentionally concrete:

- same-surface anchored dropdowns, popovers, and context surfaces;
- same-surface modal layers and one or more nested portal descendants;
- direct, sequential, scoped, trapped, restored, roving, and active-descendant
  focus within the mounted application;
- system-respecting enter, exit, and geometry-transition motion with
  interruption and retargeting;
- typed single-stroke and ordinary two-stroke command shortcuts, route
  precedence, conflict reporting, and invocation;
- vertical and horizontal wheel, trackpad, keyboard, and programmatic scroll,
  including nested routing, reveal, anchoring, and host-provided momentum
  deltas; and
- single, toggle-multiple, contiguous-range, extend, replace, and clear
  selection over stable application item keys.

The ordinary portfolio also includes secondary-click context opening,
Shift+Tab, topmost-only Escape dismissal, exact-bound nested scroll chaining,
typeahead that does not counterfeit IME commit, and select-all scoped to the
current declared selection owner. Drag, native menus, document windows,
spatial selection, and custom scroll physics remain successors.

Future milestones may add siblings to these contracts. They may not reinterpret
the meanings established here.

## Current Boundary and Exact Gap

The repository has useful foundations but not production services:

- `UiIntentRuntimeServiceDestination` currently names only open portal, close
  portal, and invoke command intent destinations;
- the runtime registers those destinations through
  `UiUnsupportedServiceIntentExecutionBinding`, and every attempt settles as
  `RejectedBeforeEffect` with `worth_ui.runtime_service.unsupported`;
- `register_unsupported_intent_definition` exposes the placeholder through the
  application builder;
- `UiDeclaredServiceUsagePosture` names portal, scroll, focus routing, and
  motion but omits command routing and selection;
- host observations include focus, scroll delta, clock, and tick, but focus is
  only a window-level boolean and scroll delta lacks the complete input and
  routing basis required by the service owner;
- host capability negotiation names portal-anchor observation, scroll-owner
  observation, and native focus, while motion is incorrectly close to being
  treated as a host effect instead of runtime-sampled presentation;
- mounted focus scopes, motion participation, portal mechanical roles,
  portal-anchor measurements, scroll ownership, allocation evidence,
  invalidation indexes, and committed sources already exist;
- command descriptors carry `default_shortcut_reference: Option<String>` even
  though a display string cannot be shortcut identity or routing authority;
  and
- Mosaic already defines focus-scope kinds and scroll-ownership postures that
  the runtime must consume rather than duplicate; and
- the permanent Pulse is still a 160-by-96 three-region proof surface, although
  it already has real authored paint/theme tokens, qualified text, mounted
  Query projection, external native capture, structural point tracing, bounded
  inspection, DPI projection, and an independent control-point manifest.

Milestone 3.15 replaces the unsupported service path and string shortcut path.
It completes the existing mounted and host foundations. It may not wrap those
placeholders while leaving them as a parallel authority lane.

`UiIntentRuntimeServiceDestination` remains the intent-origin subset of
service operations, not an enum of every service or service state. Open portal,
close portal, and invoke command lower into typed family requests. Direct
focus, reveal, scroll-to, and selection-change requests may be added only when
they are genuine admitted intent destinations. Clock ticks, rebinds, host
observations, dismissal, restoration, and motion continuations do not pretend
to be intents.

## Decisive Proof Portfolio

Milestone 3.15 has three decisive courtrooms because native product behavior,
protocol faults, and scale are different claims. None may impersonate another,
and none creates a new Cargo test target or executable.

### `RS-01`: Native product pulse

Graduate the existing austere 160-by-96 `worth-ui-platform-pulse` page into a
designed 960-by-600 logical-client-area product workbench. Keep its inherited
Cargo-built child, single native window, isolated installation, source
watcher, Query installation, external observation stream, and at-most-45-
second journey. The Pulse remains bounded ordinary product evidence rather
than a scale world: every `RS-01` frame contains at most 128 mounted nodes,
including portal descendants and exit retention, and `RS-10` remains the only
service-scale courtroom.

The reference composition is exact:

- a 24-logical-pixel outer gutter and an eight-point spacing rhythm;
- a 912-by-56 identity masthead at `[24, 24]`;
- a 216-by-424 read-only evidence rail at `[24, 104]`;
- a 672-by-424 primary service stage at `[264, 104]`; and
- a 912-by-24 truthful status band at `[24, 552]`.

The evidence rail is not a fake shell or inspector. It presents only real
Pulse product facts: current source/application generation, admitted Query-
backed status, action posture, and the latest bounded service outcome. The
primary stage contains the real Query-backed value, the real action that opens
the anchored portal, the actual command-context winner/loser explanation, and
the actual typed precondition that stopped the separate Query-backed action
before Query was asked. The status band reflects real
focus, portal, motion, and reduced-motion posture. A region with no admitted
action is absent from the hit-test map; every visible interactive region must
route through a shipped 3.14/3.15 contract.

Pulse product progression is independent from its optional visual-comparison
diagnostic. Each product turn drains native input, newly admitted attempts,
executor transitions, consequence publication, and product-port replies to a
typed quiescent outcome through at most eight local rounds. An interrupted or
saturated cycle is explicit; three guessed advancement calls are not a
contract. Visual comparison may follow a product turn when its own state asks
for comparison, but visual readiness cannot gate product execution. A routed
attempt that stops as `Unrouted` publishes a bounded stopped observation with
the graph node and interaction; it does not disappear and does not manufacture
a posture or frame.

The Pulse uses a restrained, Pulse-private authored palette and qualified text
styles. Canvas, raised surface, rule, primary text, secondary text, accent,
positive, and caution roles are declared as real application theme tokens and
lower through mounted paint/text contracts. They are not renderer constants
or a public appearance system. All body text/background pairs meet at least
4.5:1 contrast, all purposeful non-text boundaries meet at least 3:1 against
their adjacent surface, and every interactive target is at least 32 by 32
logical pixels. Flat edges are intentional: 3.15 may not paint fake shadows,
rounded controls, icons, hover/pressed/focus skins, or other mechanics that the
runtime does not yet own. Milestone 3.16 may consume the service postures to
add state-driven appearance without replacing this composition or its facts.

The runner uses real operating-system pointer, secondary-click, key, focus,
wheel, and window actions. It may capture pixels and edit existing external
product inputs. It may not call a service, proposal compiler, Query, runtime,
or host facade; construct authority; inject adapter observations; or publish
expected outcomes.

The runner uses an independent checked-in visual-contract manifest rather than
production constants. The manifest names the reference regions, authored
semantic identities, token-role identities, control points, contrast pairs,
minimum targets, text-containment bounds, maximum node count, and capture
budget. Scale greater than 4 is typed unsupported for full-frame Pulse
inspection. The largest admitted 1120-by-700 scale-4 capture is bounded to
50,176,000 RGBA8 bytes, and two retained before/after captures are bounded to
100,352,000 bytes. Smaller structural or region evidence does not reserve that
maximum eagerly. The manifest is a design/proof oracle, not authority returned
to the product.

The journey:

1. launches at an externally observed 960-by-600 logical client area and proves
   masthead, rail, stage, status band, real text, and real hit targets from
   mounted structure plus native pixels;
2. opens one anchored dropdown through a real 3.14-admitted action;
3. shows coordinated initial focus and one entrance motion;
4. invokes the same typed shortcut in two coherent contexts and shows the one
   lawful winner plus inspectable losing candidate;
5. invokes one Query-backed command whose UI route succeeds and whose separate
   Query-backed action is then deliberately stopped by the projection owner's
   own source-revision precondition **before** any Query work is submitted,
   then publishes that exact typed precondition in the external Pulse
   observation stream, proving that a successful UI route and an admitted
   route receipt carry no Query authority and do not by themselves reach
   Query. This step deliberately does **not** claim a Query-owned admission
   denial: the installed Query workspace surface exposes no typed
   admission-denial outcome for this operation, so the Pulse must not
   manufacture or locally simulate one. A real Query-originated denial becomes
   a lawful successor claim only once the Query audience facade publishes that
   outcome;
6. resizes the native client area to 1120 by 700 while the portal is open, then
   hot-rebinds the external source so the anchor moves and the preferred focus
   participant disappears; the 216-pixel rail and 24-pixel outer/gutter rhythm
   remain fixed while the primary stage absorbs the additional extent;
7. dismisses only the topmost portal through real Escape or outside press,
   restores lawful focus, and completes or snaps exit motion according to the
   runtime reduced-motion policy; and
8. shuts down with exact-zero family, proposal, presentation, prefix, track,
   focus, portal, and host resources.

At both native sizes, mounted structural evidence proves region identity,
allocation, layer order, text source/provenance, and hit-test honesty. External
pixels independently prove canvas/surface/accent separation, alignment edges,
the eight-point rhythm, text ink containment, contrast-role samples, portal
placement, and absence of clipping or overlap. Deterministic headless/native
presentation cases repeat the geometry and text-containment contract at 1.0
and 1.5 device scale; the native runner records and projects through the
machine's real scale rather than assuming one. A single full-frame golden image
cannot close the claim, and anti-aliasing differences are excluded from the
stable masks instead of weakening structural assertions.

Every text-bound manifest entry declares its maximum admitted line count and
enough height for that line count plus an ink-free safety inset. At both Pulse
sizes, the real native journey drives the Source Signal detail and Query
Posture value through truthful content that actually occupies their declared
second line, then proves from the final client-area raster that neither region
paints into its bottom or right safety inset. Rectangle containment, repeated
line-height arithmetic, a synthetic raster, or non-wrapping copy may support
this proof but cannot substitute for the stateful native observation. A mutant
that restores either former one-line-height allocation must fail.

Pixels, public observation, product facts, bounded inspection, the independent
visual contract, and resource census must agree. Adapter-local popup/focus/
animation state, a synthetic `activate`, a direct service call, a mockup, or an
in-process reenactment cannot close `RS-01`.

### `RS-07`: Protocol ordering, fault, and settlement

Use public session/intent entry APIs in modules inside the existing
`application_contracts` certification target and the production headless/
native contracts. Script legal external host observations and solicited-effect
settlements only through production host ports; do not mint runtime verdicts,
construct family state, or call family owners.

The protocol world opens a portal and begins focus/motion, then crosses a hot
rebind while delivering a maximum-size legal batch, stale and reordered
reports, duplicate outside press and Escape, window-focus loss/regain, a
reduced-motion policy change, scroll phase boundaries, and one solicited native
focus-placement effect that becomes indeterminate after issue. A distinct
Windows subset proves every fault the actual platform can
cause without injection; it does not claim the scripted faults as native
executable-world evidence.

Required outcomes:

- stale generation, presentation, incarnation, and host-session evidence opens
  no door;
- duplicate dismissal is idempotent and closes only the current top portal;
- before-effect cancellation has no semantic effect;
- an issued indeterminate effect remains typed and is reconciled from current
  host truth; silence or timeout is never success;
- service semantics are not replayed to recover a physical effect;
- proposal compilation cannot advance application or mounted publication;
- the existing presentation/host-truth coordinator is the only physical-
  settlement owner; and
- all resources reach zero after reconciliation, typed abandonment, or
  shutdown.

### `RS-10`: Scale and amplification

Use the existing ignored, explicitly filtered closure-stress/subprocess lane
under the `application_contracts` target with production public declarations,
family owners, proposal compiler, mounted publication, headless/native ports,
and independent model oracles. `RS-10` is not part of the ordinary warm
60-second suite. The scale world contains:

- 4,096 mounted nodes and 64 unrelated service neighborhoods;
- 4,096 commands with deliberate context/shortcut overlap;
- 128 focus participants in nested scopes;
- 1,024 selectable stable item keys with reorder/removal;
- an eight-owner nested scroll chain;
- a four-level portal stack; and
- 64 concurrently active motion tracks.

It records named counters rather than relying on elapsed time alone:

- `portal_neighborhoods_visited` is bounded by affected layer/anchor work;
- `focus_participants_visited` is bounded by the selected scope;
- `motion_tracks_sampled` is bounded by active tracks, with zero inactive work;
- `scroll_chain_depth_visited` is bounded by the current chain;
- `selection_keys_visited` is bounded by changed keys, except an explicit range
  may be linear in that range;
- `command_candidates_resolved` is bounded by the indexed candidate set;
- `proposal_requirements_visited` is bounded by proposals in the transaction;
  and
- `unrelated_neighborhoods_touched` is exactly zero.

Ordinary evidence retains compact current summaries and bounded causal links,
not full deltas, samples, candidate lists, or narratives.

### Mutants the portfolio must kill

The portfolio fails if an implementation:

- keeps a host-local popup, focus tree, animation timeline, selection model, or
  command winner as semantic truth;
- lets `UiServiceProposalCompiler` publish, settle host effects, retain family
  state, switch on `UiRuntimeServiceFamily` to implement behavior, or bypass
  `UiMountedPresentationCoordinator`;
- introduces `ServiceManager`, a catch-all service-state enum, or a generic
  family payload bag;
- lets one family call another directly;
- uses IDs, digests, stable keys, shortcut text, or diagnostics as authority;
- confuses committed target, presentation-sampled, allocation, or host
  geometry;
- performs a forbidden full-world scan or sends every motion sample/scroll
  delta through managed intent execution;
- imports raw Query, turns Query extent into offset/cursor authority, or treats
  UI route success as Query admission;
- uses ordinary replay or timeout-as-success for indeterminate effect;
- preserves the string shortcut match path, allocates unused family owners, or
  creates a new cert target to hide missing Pulse evidence;
- shares the proposal compiler, planner, router, interpolator, shortcut
  resolver, or selection reducer with an alleged independent oracle;
- derives the Pulse control-point manifest from production constants, uses a
  whole-frame golden as its only visual oracle, lets decoration hit-test,
  clips/overlaps text at an admitted scale, or substitutes renderer ornament
  for mounted product facts;
- retains unbounded history; exposes undo/redo or `provisional_aftermath`; or
- leaks a task, waiter, host handle, owner registration, route index, prefix,
  track, proposal, or recovery record after shutdown.

## Supporting Proof Portfolio

The decisive courtrooms are accompanied by focused, mutation-sensitive
evidence:

- compile-fail proofs for forged UI admission, wrong application/surface/
  binding generations, wrong service owner, wrong selection key family, raw
  Query imports, replay imports from ordinary lanes, and host attempts to mint
  semantic receipts, plus structural proofs that the proposal compiler cannot
  publish, issue/settle host effects, retain family state, or switch on family
  classification to implement behavior;
- exhaustive transition tests for each family lifecycle, including duplicate,
  stale, cancellation, rebind, teardown, and indeterminate cases;
- model-based portal-stack, focus-scope, nested-scroll, selection-delta,
  command-conflict, and motion-retarget tests;
- typed shortcut parser and formatter round trips, platform-alias tests,
  sequence-prefix conflict tests, repeat/IME suppression tests, and proofs that
  display formatting is not identity;
- selection reorder/removal/range tests with an oracle built independently of
  the production selection engine;
- deterministic motion tests driven by the production presentation-sample time
  contract and a test clock, including once-per-retarget semantic facts,
  presentation-only samples, interruption, reduced motion, exit retention,
  damage, and presentation hit testing;
- Pulse composition tests that compare mounted structure to the independent
  control-point manifest, compute contrast from authored token values, prove
  text ink containment and non-overlap at 1.0/1.5 scale, prove decorative
  regions are absent from the hit map, and use stable native pixel masks to
  verify alignment/spacing without treating one screenshot as authority;
- observation and narrow solicited-effect negotiation/settlement tests against
  both native and headless hosts;
- protocol compatibility tests for supported schema versions and explicit
  rejection of unsupported versions;
- exact census and bounded-work assertions at empty, ordinary, and courtroom
  scale; and
- boundary, dependency-direction, feature-matrix, generated-context,
  line-cap, documentation-link, and deletion-inventory checks.

Fixtures may make setup convenient, but proof oracles must not share the
production proposal compiler, planner, router, interpolator, shortcut resolver,
or selection reducer whose honesty they claim.

## Product Decision Lock

### Platform Pulse is a designed, truthful product surface

The permanent Pulse is a quality gate for product composition as well as an
architecture gate. Its visual direction is a calm graphite desktop workbench:
an asymmetric Mosaic of a large editorial hero, a narrow live-signal tile,
stacked Query/native fact tiles, restrained native chrome, one luminous violet
accent, one positive signal, one caution signal, and qualified typography with
an obvious reading order. It must not resemble a light enterprise dashboard,
Fluent-style control gallery, or a set of flat white rectangles. The
Pulse-private authored palette is fixed:

| Role | Value |
| --- | --- |
| canvas | `#0B0F14` |
| raised surface | `#11161C` |
| elevated surface | `#171D25` |
| structural rule | `#5F6977` |
| primary text | `#F2F4F7` |
| secondary text | `#A1A9B4` |
| principal accent | `#AC67F2` |
| action text | `#FAFBFC` |
| positive | `#5CC978` |
| caution | `#E0AD62` |

Typography uses the existing qualified application-font path and these
Pulse-private roles:

| Role | Size / line | Weight | Use |
| --- | --- | --- | --- |
| display | 44 / 52 | 500 | two-line primary stage title only |
| masthead | 16 / 20 | 600 | application identity and compact fact value |
| section | 11 / 16 | 650 | short rail/stage labels |
| body | 13 / 20 | 450 | product facts and explanations |
| meta | 12 / 16 | 500 | status band and secondary evidence |
| action | 13 / 20 | 600 | admitted interactive labels |

Sizes and line heights are logical typographic points represented by qualified
`UiTextStyle` values. Hierarchy may not be faked by rasterized labels or
host-selected fonts.

The inherited exact blue `#2F81F7` to green `#3FB950` source-edit proof
survives as a compact live source-signal swatch inside its narrow Mosaic tile;
it no longer floods the whole canvas. The yellow and purple predecessor
controls are replaced by the role palette in the same explicit visual migration. Authored
scenario identity, Query/source causality, overlay tracing, and lifecycle
evidence remain continuous; the old 160-by-96 layout and control points do not
remain as a compatibility mode.

The Pulse must feel considered without impersonating future framework
features:

- the masthead identifies the real application and current source generation;
- the body is visibly composed as multiple unequal Mosaic tiles rather than
  one flat stage decorated with card-shaped rectangles;
- the read-only rail makes real Query, intent, and service evidence pleasantly
  legible, but is not a dockable panel, navigation tree, or developer tool;
- the stage gives the portal action a clear focal point and keeps the Query
  value, command decision, and Query denial in one readable causal story;
- motion is short, purposeful, receipt-derived, and reduced-motion aware;
- copy uses product language and concise typed outcomes, never raw debug dumps;
- every visible transient surface is a complete product composition with a
  normally placed title, recognizable icon, useful body, and conventional
  action row; primary and secondary actions must be visually legible and
  semantically real, including Save and Cancel when the surface actually owns
  savable state;
- decorative regions do not hit-test, and controls without behavior do not
  render; and
- renderer-only ornament, fabricated data, fake disabled affordances, future
  toolbar/menu icons, and hard-coded milestone-success badges are forbidden.

Aesthetic quality is a deliberately judgment-based ship criterion, not a
property that the geometry manifest, contrast checks, pixel masks, or other
automated evidence can certify. A designated product/design reviewer must
inspect full-window screenshots captured from the real running Pulse at both
the 960-by-600 reference size and the 1120-by-700 resize posture and answer:

> Would this look like it belongs alongside Linear or Notion as a contemporary
> desktop product?

Milestone closure requires a defensible **yes**. Linear and Notion are a quality
bar for visual hierarchy, proportion, spacing, typography, restraint, polish,
and product coherence; they are not layouts or brands to copy. A technically
correct screenshot still fails when it looks like a framework sample, a proof
harness, a collection of empty bordered rectangles, or an obviously unfinished
developer surface. The reviewer must consider the whole composition rather
than isolated compliant control points. This judgment cannot demand fabricated
data, fake controls, unsupported interaction states, or renderer-only mechanics:
capability honesty remains the harder constraint.

A host-colored portal rectangle, anonymous popover, misplaced title, missing
icon, or actionless modal is an aesthetic failure even when its lifecycle and
pixels are otherwise correct. Conversely, a convincing Save button that does
not save real state is a capability-honesty failure. Product-authored portal
content must travel through qualified mounted mechanics; hosts and renderers
may not hard-code application copy, icons, or buttons to satisfy this gate.

This is application composition, not 3.16 appearance authority and not a
general component library. Static Pulse theme tokens, qualified text styles,
authored component geometry, and content projections remain real current
capabilities. Hover, pressed, focused, selected, disabled, and validation-
bearing visual semantics are successors unless the runtime already produces
the exact mounted fact; implementation may not infer those states in the host.

### Six owners, not one manager

There are exactly six semantic service families in this milestone:

```rust
pub enum UiRuntimeServiceFamily {
    Portal,
    Focus,
    Motion,
    CommandRouting,
    Scroll,
    Selection,
}
```

This enum is classification and inspection vocabulary only. It does not carry
requests, state, plans, outcomes, receipts, providers, or callbacks. Code may
exhaustively classify support and evidence with it. Code may not switch over it
to become the owner of all family behavior.

Each family has a named semantic owner and separate modules for declaration,
request, planning, state/lifecycle, rebind, produced facts, receipt, evidence,
and cost where those responsibilities exist. Family-produced facts are inputs
to existing publication, not family publication authorities. A family may
expose fewer files when one responsibility is genuinely absent; it may not
combine unrelated responsibilities to save files.

### Common request basis

All explicit service requests carry `UiServiceRequestBasis` containing:

- service request identity and causal parent, when one exists;
- exact application identity and application generation;
- semantic surface identity and host-surface identity when physically bound;
- binding generation and mounted presentation basis when relevant;
- family-specific owner and scope identity, stored in the family request rather
  than erased to a common string or integer;
- request origin: admitted intent, host observation, rebind, service
  continuation, runtime policy, or teardown;
- source order or causal sequence;
- cancellation and resource-budget identity; and
- the concrete inherited authority that makes that origin legal.

`UiServiceRequestBasis` is carried proof, not proof by possession. Constructors
are sealed behind the legal producer facades. It cannot be made from IDs,
digests, diagnostics, or equal-looking values. It does not itself prove that
any family-specific operation is legal.

Every family defines its own typed request, plan, receipt, terminal outcome,
and rejection vocabulary. There is no `UiServiceRequest { family, payload }`,
`serde_json::Value`, dynamic property map, stringly request, generic callback,
or erased provider lane.

High-frequency host scroll and motion continuation lanes carry this basis or a
sealed compact derivative, but do not allocate a managed 3.14 intent attempt
per delta or sample. Explicit user operations still use 3.14 admission.

### Authority and truth sources

| Question | Sole authority | Not authority |
| --- | --- | --- |
| May this explicit UI operation route? | exact 3.14 UI admission | command, shortcut, focus, selection, host event |
| Which portal is logically open? | portal owner current state | host popup, mounted node, pixels |
| Which participant has semantic keyboard focus? | focus owner current state | window focus boolean, DOM/native focus alone |
| What is the semantic layout target? | committed mounted/runtime publication | current animation sample |
| What geometry is presented now? | motion owner sample from committed bases | target geometry or host interpolation |
| Which command route wins? | command-routing plan over current coherent context | registration order alone or shortcut string |
| What is the current scroll offset? | scroll owner state reconciled with typed host observations | Query cursor or renderer transform |
| Which application items are selected? | selection owner over stable typed item keys | focus, row index, Query identity |
| May domain or Query work occur? | that domain's separate admission | any UI service receipt |
| May a stale/indeterminate physical effect be retried? | typed recovery state and host observation | ordinary replay or timeout guess |

New legality authority belongs in `worth-proof`. Portable value vocabulary
that means the same thing across a runtime boundary belongs in
`worth-foundational` only when the dependency manifest permits it. Live
service state, clocks, counters, route tables, cancellation, and `Drop`
lifecycles stay in `worth-ui-runtime`. Host wire values stay in
`worth-ui-host-contract`. Declaration meaning stays in its declaration owner.
No substrate crate hand-rolls an inaccessible duplicate.

### Compiler-visible geometry brands

Milestone 3.15 seals non-interchangeable geometry brands for:

- declaration geometry and placement preferences;
- admitted allocation geometry;
- committed successor layout geometry;
- current presentation-sampled geometry; and
- host-surface coordinates and physical pixels.

Named owners perform explicit conversions and issue typed receipts. Portal
placement, hit testing, clipping, pointer targeting, and motion sampling accept
presentation-sampled geometry where current pixels matter. Layout, rebind, and
successor planning accept committed layout geometry. Host adapters receive
host-coordinate types only. A generic rectangle plus a runtime `kind` field or
unchecked `From` conversion does not satisfy this contract.

### Proposal compiler and dependency graph

Services never invoke one another. The permitted dependency graph is:

```text
portal ----- emits focus and motion requirements -----+
focus ------ emits reveal requirement ----------------|
command ---- reads declared focus/selection axes ------|--> proposal compiler
scroll ----- owns scroll state; emits facts -----------|
selection -- owns selection state; emits facts --------|
motion ----- reads committed predecessor/successor ---+
                         |
                         +--> family-produced 3.12 facts
                              + ordinary mounted presentation work
                              -> existing application/mounted publication
                              -> existing presentation coordinator
```

`UiServiceProposalCompiler` may read typed proposals and the exact declared
axes of coherent snapshots exported by family owners. A snapshot cannot widen
scope by carrying unrequested participants, keys, routes, or facts. The
compiler may not implement a family's routing/lifecycle, retain family state,
mint an application or mounted frame, publish facts, issue host effects, own
physical settlement, or reconstruct authority from IDs. Its sole
responsibility is to:

- validate that all proposals share one coherent application, generation,
  surface, binding, presentation, and causal basis;
- order requirements by a fixed dependency DAG;
- reserve explicit proposal/family budgets;
- reject cycles, ambiguity, stale proposals, unsupported mechanics, and
  partial preflight before effect;
- emit one typed batch of family-produced observations/facts plus ordinary
  mounted presentation work for existing publication; and
- retain only the bounded `UiServiceProposalIdentity`, occupancy lease, and
  cancellation posture until existing publication accepts or rejects the
  batch.

Unpublished successors, plans, and produced-fact candidates remain inside the
family owner that can commit or discard them. Existing publication reads those
family-owned staged facts under the proposal identity. The compiler cannot own
or copy an owner receipt, successor, placement, focus route, scroll result,
selection delta, or motion track.

Existing publication returns one typed accept/reject receipt under that
proposal identity. Session composition presents the exact receipt to each
participating family owner: acceptance commits its staged successor; rejection
discards it without changing current state. Each owner returns a terminal
settlement acknowledgement. The compiler releases its occupancy lease only
after those acknowledgements, but still retains no family data. Pending or
indeterminate physical work proceeds independently under the existing
presentation/host-truth owner.

Service-produced facts extend the existing 3.12 owner-ranked ordering. Existing
`CommittedPortalAnchor` and `CommittedScrollExtent` stay with their existing
owners. New owner-specific families cover committed focus, selection, motion
track, and command route meaning. There is no universal service-event bag.

Default occupancy is one proposal compilation per exact application, semantic
surface, `UiRuntimeServiceFamily`, and that family's declared owner/scope
identity. A conflicting proposal returns a
typed occupied, superseded, coalesced, or cancelled-before-effect outcome
according to family policy; it never queues without a declared bound or wins by
arrival time. Host observation order, service-produced fact order, and 3.12
source/Query/viewport order are one explicit owner-ranked contract. Fairness,
queue bounds, and cancellation posture are inspectable.

Coordination uses fixed semantic stages rather than a switch over service
families:

1. validate the coherent pre-state and reserve budgets;
2. require each participating family owner to stage its own transition and
   return only a sealed stage-complete witness;
3. require existing planning/mounting preparation and the portal owner to
   assemble the successor target and placement from branded anchor, scroll,
   viewport, and layer evidence;
4. require the focus owner to resolve against that successor target; focus may
   emit at most one reveal refinement, which requires the scroll owner to
   replan once and existing preparation plus the portal/focus owners to refresh
   and revalidate their own staged work;
5. require the runtime motion owner to derive a track from the exact committed
   predecessor and planning-issued prepared successor receipt;
6. emit owner facts and mounted work to the existing atomic publication owner;
   and
7. end compilation when publication accepts/rejects the batch. Any resulting
   host work and settlement belong to `UiMountedPresentationCoordinator` and
   existing host-truth recovery.

Command route resolution precedes the destination request it emits; the
destination then starts a new causal operation and may require its own
proposal compilation. A family with no proposal in a transaction is not
visited.

These are family-owned operations in compiler-enforced order, not algorithms
implemented by the compiler. Cycle detection is a product invariant, not
defensive logging. A focus reveal
may request scrolling once in the current transaction. That scroll consequence
cannot request focus again. A portal close may restore focus and start exit
motion. Motion completion cannot reopen the portal.

### Publication, physical effects, and recovery

Family owners produce semantic service facts, mounted projections,
invalidation, inspection summaries, and causal receipts. The existing 3.12
application/mounted publication boundary publishes them atomically. Neither
the proposal compiler nor a family owner becomes another publisher.

Native focus placement or a future solicited host mechanic can fail after
effect. Those operations use a typed
four-stage posture:

```text
prepared-before-effect
-> issued-pending-settlement
-> settled-terminal
or indeterminate-reconciliation-required
```

Before-effect rejection has no semantic effect. The existing presentation and
host-truth owners retain a bounded pending/indeterminate record keyed by the
exact host session, surface, service request, effect identity, and protocol
schema. Reconciliation consumes current typed host observation. It never
replays the semantic request and never treats a timeout as success. Shutdown
cancels what is cancellable, reconciles what is observable, abandons only
through a typed terminal reason, and brings the resource census to zero.

### Hot rebind and multi-surface identity

Every service owner implements an explicit rebind decision for each live
record:

```text
preserve unchanged
rebase to new presentation/binding
replace with a new incarnation
cancel before effect
drop with typed reason
enter reconciliation
```

Family rules decide among those outcomes. Equal-looking declarations or
geometry do not imply preservation. Stable semantic identity permits a rebase
only when policy, owner, scope, and required authority remain coherent.

No service is a process-global singleton. All indexes are rooted in
application identity and generation, then semantic surface and family owner.
Host mechanics additionally bind host session and physical surface.
Milestone 3.15 proves one semantic surface bound to one host surface. Portals
are ordinary mounted overlays within that surface, not popup windows. The
request basis carries the established semantic-surface + host-surface pair so
Milestone 4 can add window topology above it without new global keys or
reinterpreting an overlay as a window.

### Persistence classification and future undo attachment

Service state carries a persistence posture even though 3.15 performs no disk
I/O:

- `ephemeral`: motion samples, focus-visible modality, prefix occupancy, and
  other state that must not survive a session;
- `session_restore_candidate`: declared scroll offsets, selection, focus
  restoration posture, or portal-stack state that Milestone 11 may later admit
  for persistence; and
- `effecting`: a state transition associated with issued native-focus or a
  future separately governed solicited effect and therefore subject to the
  four-stage settlement posture.

This classification is not persistence authority, undo authority, aftermath,
or an inverse-operation log. Milestone 3.15 retains only current family state,
bounded deltas required by its ordinary contracts, and settlement evidence. It
does not retain historical inverse data for future undo. As the roadmap
requires, a future operation owner retains undo/redo history, legality,
execution, and recovery; WORTH UI later presents that owner-admitted capability
through command identity. Focus, scroll, selection, portal, or motion receipts
cannot mint it.

### Support and capability negotiation

Support is explicit at declaration, runtime, and host boundaries:

- a declaration states which family and policy it requires;
- runtime installation reports family support and configured limits;
- the host negotiates the exact observation, mounted-presentation, and narrow
  solicited-effect schemas/mechanics it supports; and
- mounted preflight rejects unsupported required mechanics before effect with
  a typed diagnostic and lawful fallback options, when any exist.

Unused families allocate no live owner state, start no task, request no clock,
and perform no per-frame work. A used family is installed through its canonical
owner. There is no dummy successful service, silent renderer fallback, or
application-local replacement hidden behind a callback.

### Evidence and cost

Every family publishes a compact current inspection summary and bounded recent
causal links. Family counters include admitted/rejected requests, active/live
records, rebind outcomes, stale denials, host effects, indeterminate effects,
reconciliations, affected consumers, and visited candidates appropriate to
that family.

Ordinary production behavior does not retain every host delta, motion sample,
route candidate, or historical state. Detailed narratives, full traces, and
reconstruction are opt-in diagnostic or certification work and remain outside
the ordinary hot path. All summaries carry currentness and truncation posture.

## Family Contract: Portal

The portal owner decides logical portal lifecycle. Its stable vocabulary
includes:

- `UiPortalIdentity` and `UiPortalOwnerIdentity`;
- logical owner, anchor reference and anchor measurement basis;
- layer and parent-portal relationship;
- same-surface mounted overlay presentation strategy;
- modality, pointer/input shielding, and dismissal policy;
- initial-focus, trap, and restoration requirements;
- entrance and exit motion declarations;
- open, visible, closing, closed, and indeterminate physical-settlement
  postures; and
- typed anchor-loss, owner-loss, rebind, dismissal, and teardown reasons.

Open and close are idempotent for the exact request and portal identity.
Opening an already-open portal with a different coherent request is a typed
replace, rebase, or rejection according to declaration policy; it is never a
second invisible portal record.

An anchored portal uses the existing portal-anchor allocation and observation
foundation, then produces final current presentation geometry. Declaration
geometry, allocation geometry, target layout, host coordinates, and
presentation-sampled geometry remain distinct types. Placement owns collision,
viewport fit, preferred side, and fallback ordering. The host presents ordinary
mounted overlay work, but it never owns whether the semantic portal is open or
turns that overlay into a document window.

Dismissal supports Escape, outside press, accepted selection, explicit owner
request, anchor loss, owner loss, application shutdown, and declared window
focus policy. Each cause is typed and inspectable. Duplicate causes coalesce by
portal identity and causal basis. Parent closure deterministically closes its
descendant chain.

Portal emits focus and motion requirements; it does not import or invoke those
owners. Closing retains only the mounted resources needed for a declared exit
motion. Hit testing and focus shielding follow current presentation and
lifecycle posture, not the final target or invisible retained nodes.

## Family Contract: Focus

The focus owner keeps these meanings separate:

- host window activation/focus;
- semantic keyboard focus;
- accessibility-focus integration point;
- focus-visible input modality;
- text/IME composition ownership; and
- pointer capture or hover, which are not focus.

Those distinctions are sealed types, not enum flags or comments:

- `UiWindowFocus` is host-window activation observation;
- `UiSemanticKeyboardFocus` is the runtime-owned focused participant;
- `UiFocusVisibleModality` records the lawful input-modality basis;
- `UiActiveDescendant` identifies a descendant within a semantically focused
  composite without moving keyboard focus to every descendant; and
- `UiAccessibilityFocusHook` is a typed unsupported integration point whose
  support row names Milestone 13. It is not an accessibility-focus boolean or
  a second live focus tree.

Its stable vocabulary includes participant and scope identities, Mosaic's
existing scope kinds, traversal order, trap posture, roving and
active-descendant policy, eligibility, initial-focus policy, restoration token,
focus cause, focus-visible cause, and typed focus outcome.

The owner supports direct request, forward/backward traversal, first/last
eligible, roving movement, active-descendant movement, portal initial focus,
portal restoration, and rebind fallback. Traversal is scoped and indexed. A
removed or disabled participant is never restored because its ID compares
equal; its current incarnation and eligibility must be proven.

Window focus loss may suspend native focus mechanics and focus-visible posture
according to policy. It does not erase semantic focus unless declared policy
requires that consequence. Focus loss never commits or cancels a draft by
itself. IME composition suppresses command shortcuts and traversal according
to typed key policy; no heuristic string check decides this.

A focus placement may emit one reveal requirement to the proposal compiler.
Focus does not mutate scroll. Focus restoration failure yields a typed fallback
or terminal reason and is visible through `why_focus_moved` inspection.

The existing host observation family named `Focus` is cleanly renamed to
window focus vocabulary. Semantic focus placement and acknowledgement use the
narrow solicited focus-placement contract. No boolean host observation becomes
the semantic focus owner.

## Family Contract: Motion

The runtime motion owner derives and owns committed motion-track meaning from
an exact committed predecessor, a planning-issued prepared successor, and a
motion declaration. A separate presentation sampling mechanism consumes the
committed track plus the runtime presentation-sample clock. Neither decides
semantic layout, portal lifecycle, selection, focus, or command state.

Its stable vocabulary includes motion-track identity, declaration, predecessor
receipt, successor receipt, affected property/channel, clock basis, easing,
duration, delay, fill/exit-retention policy, interruption policy, reduced-
motion policy, current sample receipt, and terminal outcome.

Presentation sampling uses one typed monotonic clock. Existing host
`Tick` observation and level-triggered readiness wake presentation work; 3.15
does not create a motion-wake protocol or second scroll-animation clock. The
host does not interpolate semantic state.

The runtime motion owner publishes committed track identity, predecessor/
successor receipts, policy, and affected channels through 3.12 once per
semantic retarget or terminal transition. Intra-track samples are bounded
presentation-only work keyed by track identity and owned beside mounted
presentation. They update current presentation values and damage without
minting a new semantic fact, application generation, or mounted frame per
sample. Sampling cannot construct, retarget, or terminate the semantic track.

Hit testing, pointer targeting, portal anchoring, clipping, and visual damage
use current presentation samples. Layout and rebind reasoning use committed
semantic targets. The types must make accidental substitution visible to the
compiler.

Interruption policy explicitly chooses retarget-from-current-sample,
restart-from-current-semantic-predecessor, finish-then-apply, snap-to-target,
or cancel/drop where lawful. Rebind never guesses from wall time. Reduced
motion can suppress, shorten, or preserve a semantically necessary transition
according to declaration policy; the default is system-respecting and snaps
decorative motion while preserving final state.

Inactive motion requests no ticks and performs zero per-frame work. Exit
retention is bounded, cancellable, and included in shutdown census. Smooth
programmatic scrolling, when later enabled, composes the scroll owner's target
with this clock through proposal compilation; it cannot create another scroll
truth or motion clock.

`UiMountedMotionProjection` remains declared/mounted participation and
inspection metadata. It is not a host animation command and does not require a
fictional generic `Motion` host capability. The host receives final mounted
presentation changes only. Existing readiness and `Tick` provide wakeup.

## Family Contract: Command Routing

The command-routing owner maps one typed invocation origin and coherent context
to one winning command route. It does not execute domain mutation itself.

`default_shortcut_reference: Option<String>` is removed. The canonical
shortcut vocabulary contains typed logical and physical keys, modifier set,
platform alias such as `Primary`, stroke, one-or-more-stroke sequence,
repeat policy, text/IME suppression policy, and display formatting. Identity
and matching do not depend on localized display text.

The ordinary support bar includes single-stroke and common two-stroke
sequences. Prefix waiting is bounded and cancellable. Ambiguous complete
matches are rejected or resolved only by declared route precedence; timing or
registration order is not an implicit winner.

The coherent routing context contains application/generation, surface, active
focus scope and participant, selection owner and compact selection posture,
active portal/modal chain, command scope stack, declaration readiness, and
invocation origin. A route declares which parts it consumes. Command routing
reads exported coherent focus and selection snapshots restricted to the exact
context axes declared by the route; it does not import internal state, receive
undeclared participants/keys, widen scope, or call those owners.

Precedence is explicit: active modal/portal scope, focused control scope,
active region or document scope, surface scope, then application scope.
Within one level, declared priority and specificity apply. Equal lawful
candidates yield typed ambiguity rather than first-registration wins.

An authored component scope has one identity derivation shared by DSL
admission, Rust registration, and runtime context construction. A DSL binding
that does not name a declared component is rejected during semantic handoff.
For an active-portal command, the authored component binding identifies the
Portal owner/anchor graph node: the Portal overlay descendants do not silently
become additional command scopes. This owner/anchor interpretation is the
3.15 insertion point for richer Portal composition and may not be replaced by
an app-local string convention.

The route result names winner, losing candidates with bounded reasons,
consumed context revisions, and destination. Invocation then enters the 3.14
admission/execution lane or another explicitly governed destination. A
Query-backed command separately crosses Query admission. A command receipt is
never domain success.

Typed shortcut vocabulary is stable command-capability meaning on
`CommandDescriptor`; matching, prefix occupancy, conflict resolution, and
context indexes belong only to the command-routing owner. A successful route
constructs `UiIntentRouteSource::CommandRoute(UiCommandRouteReceipt)`, the
typed sibling reserved by 3.14. That source still crosses payload projection,
operability, UI admission, and managed execution. `InvokeCommand` consumes the
route receipt as an intent destination; it does not resolve shortcuts or skip
admission. Native menus later add another route-source sibling rather than
reinterpreting command routes.

Command-route causal evidence is retained when the host observation is
received and attached to the route receipt before intent admission. Intent
resolution is therefore a read-only consumer of a concrete retained reference;
it neither reconstructs evidence from later mounted state nor mutates the
evidence registry. Missing retained command evidence is a typed route stop.

Future plugin ownership is carried as an optional typed registration owner and
generation, not a string namespace. Unload removes registrations and pending
prefix state exactly. Any live portal, focus request, selection, motion track,
or other record carrying that registration owner/generation terminates with a
typed `owner_unloaded` cause and reaches census zero. Milestone 15 supplies
plugin permission and unload authority; 3.15 supplies only the typed owner-
generation insertion point and lifecycle.

## Family Contract: Scroll

The scroll owner owns semantic scroll offset and nested routing over Mosaic's
existing scroll-ownership declarations. It never owns Query cursor,
continuation, collection snapshot, or result-window basis.

Query may already supply content extent through
`UiAdmittedScrollQuerySource`. That is admitted allocation evidence only. It is
not offset, cursor, continuation, snapshot, or evaluation-basis authority, and
3.15 must not delete or reinterpret it. Milestone 6 may add revision-bound
virtualized extent/window evidence without changing scroll ownership.

Its stable vocabulary includes scroll-owner identity and incarnation, axis,
viewport and extent basis, current offset, bounds posture, nested parent chain,
input source and phase, delta precision, anchor policy, programmatic reveal or
scroll-to request, and typed route/settlement receipt.

Host scroll reports carry exact surface and presentation basis, position or
target affinity required for routing, high-precision deltas, input source, and
phase. The runtime resolves the current scroll chain. A host may provide
trackpad momentum deltas, but it does not choose the semantic owner or retain
the authoritative offset.

Nested routing consumes as much delta as the current owner lawfully can and
passes typed remainder to its declared parent. Axis locking, bounds, and
bubbling are explicit. Routing is bounded by chain depth and detects cycles.

Programmatic reveal names a target mounted participant and alignment policy;
the scroll owner derives the required offset from current viewport, extent,
and presentation evidence. Focus may emit that requirement through the
proposal compiler. The scroll owner does not call focus.

Anchoring uses stable application item or mounted anchor identity according to
declaration, plus exact binding/incarnation currentness. Rebind may preserve,
rebase, clamp, replace, or drop an offset. Equal numeric bounds do not prove
preservation. Virtualized collection integration later supplies collection
window evidence through an additive contract; it may not turn scroll offset
into Query cursor truth.

High-frequency deltas use a bounded compact lane and aggregate lawful
publication where observation semantics permit. Lossless phase boundaries,
direction changes needed by policy, and terminal reconciliation cannot be
coalesced away. No generic intent attempt is allocated per delta.

## Family Contract: Selection

The selection owner owns UI selection over stable typed application item keys.
It does not own focus, row index, Query identity, Query authorization, or the
underlying collection.

Its stable vocabulary includes selection-owner identity, item-key family,
collection/binding revision, selection mode, ordered selected-key set where
order is semantically required, anchor key, lead key, change operation,
invocation cause, reconciliation policy, and typed delta/receipt.

The owner supports replace, clear, toggle, add, remove, contiguous range,
extend, select-all within an explicitly bounded current collection scope, and
rebind reconciliation. Range selection uses the current application-provided
ordering basis. Stable keys survive reorder; removed or unauthorized keys drop
with a typed reason. A numeric index is never retained as item identity.

Single, multiple, and range policies are sealed canonical policies rather than
booleans distributed through controls. Focus and selection may move together
only through a proposal set triggered by a declared interaction.
Changing focus alone does not silently change selection, and changing
selection does not mint focus.

Selection publishes compact posture and deltas to appearance and command
routing. It does not copy the entire collection into ordinary evidence. Later
Query-backed tables provide an application mapping from Query result identity
to the selection owner's typed item key under a current collection revision;
the UI service never imports Query to invent that mapping.

Spatial lasso and two-dimensional selection are successors. The owner and
item-key contracts must admit additive spatial operations without changing the
meaning of existing set/range state.

## Host-Contract Decision Lock

Milestone 3.15 does not create a catch-all service-mechanics protocol. It uses
the existing mounted-frame, mounted-presentation, observation, measurement,
presentation-settlement, and host-truth boundaries.

Only native semantic-focus placement, which is genuinely solicited and cannot
be represented by window-focus observation, gains a narrow typed request/ack
sibling.

Those requests are family-specific types, negotiate explicit optional support,
and settle through the existing presentation/host-truth coordinator. Command
routing and selection have no direct host mechanics. Motion adds no host
animation or wake protocol. Scroll adds no offset-reconciliation protocol.
Portals use ordinary mounted overlay/layer presentation. An operating-system
popup surface is typed unsupported in 3.15; Milestone 14 may govern one only
after naming a real host limitation and preserving semantic portal authority.

The host observation currently named `Focus` becomes `WindowFocus` with a
window-focus payload and exact surface identity. Native semantic-focus
observation/ack is a different typed contract. `ScrollDelta` is revised in the
observation schema to include source, phase, precision, exact coordinate or
presented-target affinity, and the batch's current presentation basis. This is
a clean protocol schema cutover with explicit version negotiation, not an
alias.

The host reports mechanics and observations. It does not admit service
requests, choose semantic owners, resolve commands, interpolate semantic
motion, restore focus, route nested scroll, decide selection, or mint runtime
receipts.

Headless and native hosts implement the same production contract. Test-only
control is supplied by certification modules inside existing targets that
script legal host observations, ticks, and solicited-effect settlements through
those contracts.

## Public Developer Experience

The public facade exposes family declarations and canonical policies, not
runtime owners or mutable service internals. Policy defaults configure only a
family demanded by installed declarations/capabilities; they never install all
families. The intended Rust experience follows the existing builder and native
launch split:

```rust
struct WorkbenchApplication {
    change_profile: UiChangeProfile,
}

impl UiNativeApplicationDefinition for WorkbenchApplication {
    fn prepare(
        self,
        mut preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome {
        let builder = WorthUi::app()
            .with_change_profile(self.change_profile)
            .with_portal_policy_defaults(UiPortalPolicy::dropdown())
            .with_focus_policy_defaults(UiFocusPolicy::workbench())
            .with_motion_policy_defaults(UiMotionPolicy::system_respecting())
            .register_command(
                CommandDescriptor::new(
                    CommandId::new("workbench.command.save_all")
                        .expect("static command ID is valid"),
                    "Save All",
                )
                .with_default_shortcut(shortcut!(Primary + Shift + S)),
            );

        let builder = match builder.register_runtime_service_intent_definition(
            UiIntentDefinition::<OpenPalette>::runtime_service(
                UiIntentRuntimeServiceDestination::InvokeCommand,
            ),
        ) {
            Ok(builder) => builder,
            Err(_) => return preparation.deny(
                UiNativeApplicationPreparationDenialCause::ApplicationRejected,
            ),
        };

        if let Err(cause) = preparation.install_application_composition(builder) {
            return preparation.deny(cause);
        }
        preparation.complete()
    }
}

let platform = WorthUiNativePlatform::prepare(profile)?;
let outcome = platform.run(WorkbenchApplication { change_profile });
```

Native composition remains unfrozen until affine native preparation installs
and completes it. `run` never accepts `WorthUiHostNeutralApp`, and 3.15 adds no
adapter or compatibility implementation that makes it do so. The post-cutover
`register_runtime_service_intent_definition` replaces
`register_unsupported_intent_definition` in place.

Headless/certification composition uses the separate host-neutral path:

```rust
let host_neutral = WorthUi::app()
    .with_change_profile(change_profile)
    .with_portal_policy_defaults(UiPortalPolicy::dropdown())
    .register_runtime_service_intent_definition(
        UiIntentDefinition::<OpenPalette>::runtime_service(
            UiIntentRuntimeServiceDestination::InvokeCommand,
        ),
    )?
    .freeze()?;
```

Both examples are compile-pass fixtures in the existing native/headless
two-session contract matrix.

The macro produces the same typed shortcut declarations available through
ordinary constructors. It is not a parser bypass. Invalid key names,
modifier combinations, empty sequences, unsupported aliases, and unbounded
sequences fail at compile time where possible and otherwise at declaration
validation before runtime installation.

Canonical presets include:

- `UiPortalPolicy::dropdown()`, `popover()`, and `modal_dialog()`;
- `UiFocusPolicy::workbench()` and scoped canonical variants;
- `UiMotionPolicy::system_respecting()`;
- `UiScrollPolicy::nested_region()`;
- `UiSelectionPolicy::single()`, `multiple()`, and `range()`; and
- `UiCommandRoutingPolicy::desktop()`.

Presets lower into the exact public declarations and can be inspected as a
normalized plan. They are conveniences, not hidden service paths. An advanced
declaration may replace one policy field without rebuilding the owner.

The DSL exposes the same contracts. A representative target is:

```text
portal completion_menu {
  anchor editor_input
  layer transient
  dismiss escape outside_press accepted_selection anchor_gone
  focus first_enabled restore
  motion system_popover
}

selection results_selection {
  mode multiple
  identity result_key
  preserve stable_key
}

command show_palette {
  shortcut Primary+Shift+P
  scope application
}
```

Parsing, lowering, and validation are source-linked. Runtime code never parses
these strings. Invalid combinations produce a typed diagnostic naming the
source span, violated service law, current support posture, and a lawful repair
when one exists.

The default experience supplies the small courtesies developers otherwise
reimplement incorrectly:

- automatic lawful focus restoration for portal closure;
- canonical Escape, outside-press, accepted-selection, and anchor-loss
  dismissal policies;
- focus reveal through proposal compilation rather than direct family calls;
- stable-key preservation for selection and declared scroll anchors;
- system reduced-motion behavior;
- shortcut suppression during IME composition and text entry according to
  typed policy;
- deterministic conflict diagnostics that show command winner and losers; and
- capability-driven installation, so an unused family costs no live state or
  per-frame work.

Inspection offers source-linked `why_portal_closed`, `why_focus_moved`,
`why_focus_restoration_failed`, `why_motion_interrupted`, `why_scroll_owner`,
`why_selection_dropped`, and `why_command_won` summaries. These methods return
bounded typed evidence, not formatted logs as truth.

The certification test kit uses production public declarations and host
contracts with a deterministic clock, scripted legal observations, independent
oracles, and exact resource census. It cannot mutate service owners directly.

## Compile-Time and Mechanical Enforcement

The milestone adds or extends enforcement for:

- no raw `worth-query` dependency from WORTH UI crates and no
  `worth-query-replay` outside cert lanes;
- no `worth-*` dependency on `worthy-*`;
- no host or declaration crate dependency on runtime service internals;
- no direct dependency edge between family implementation modules;
- no `ServiceManager`, `helpers`, `common`, `util`, `shared`, generic service
  payload, JSON/dynamic property bag, or callback registry in the new path;
- no proposal compiler publication, host-effect issue/settlement, retained
  family state/successor/owner receipt, `UiRuntimeServiceFamily` behavior
  switch, or bypass of `UiMountedPresentationCoordinator`;
- no committed motion-track fact produced from mounting/presentation and no
  presentation sample allowed to construct, retarget, or terminate a track;
- no 3.15 solicited popup-layer module, OS popup surface, or old portal/scroll
  module re-export after the topology move;
- no unbranded substitution among allocation, committed-target,
  presentation-sampled, and host geometry;
- no `Option<String>` or string alias in canonical shortcut identity/matching;
- no public generic `AuthorityMarker` bound at governed service surfaces;
- no ordinary replay/reconstruction dependency in service execution;
- no `provisional_aftermath`, undo, or redo public symbol;
- no app-local production service owner in the Platform Pulse or examples;
- no new Cargo test target or executable for the 3.15 proof portfolio;
- compile-pass native affine-preparation and host-neutral freeze examples in the
  existing two-session matrix;
- no Rust file over the repository line-cap law without an explicit governing
  exemption; and
- complete service family/support/census classification when a new family is
  added later.

Compile-time feature matrices cover minimum public/runtime, headless, native,
DSL, inspection, and certification combinations. Generated `AGENT_CONTEXT.md`
files remain tool-owned.

## Architectural Destination

The destination is responsibility-oriented. Names below are committed
destinations; exact leaf splitting may become narrower to satisfy composition
and line-cap laws, but responsibilities may not be recombined into bags.

```text
workspaces/worth-ui/crates/
  worth-ui/
    src/facade/service/
      mod.rs                              create: canonical public re-exports
      portal.rs focus.rs motion.rs        create: family policy facades
      command_routing.rs                  create: shortcut/route facade
      scroll.rs selection.rs              create: family policy facades

  worth-ui-runtime/
    src/capability/registry/
      runtime_service/
        family.rs                         create: classification only
        support.rs                        create: installed support posture
      command/command_descriptor.rs       modify: typed shortcut, no String path
      intent/execution_destination.rs     modify: typed intent-origin lowering

    src/declaration/
      declared_posture/
        service_usage_posture.rs          modify: all six typed families
      service/
        portal.rs                         create
        focus.rs                          create
        motion.rs                         create
        command_routing.rs                create
        scroll.rs                         create
        selection.rs                      create

    src/runtime/session/service_proposal/
      request_basis.rs                    create: common carried basis only
      occupancy.rs                        create: bounded conflict/cancel law
      compiler/
        proposal.rs                       create
        dependency.rs                     create
        preflight.rs                      create
        staging.rs                        create: no publication/effect authority
        receipt.rs                        create
      census.rs                           create: proposal resources only

    src/runtime/portal/                   create: canonical portal owner axis
      anchored_allocation/                move: portal_anchored_allocation/*
      request.rs planning.rs              create
      state.rs lifecycle.rs               create
      placement.rs dismissal.rs           create
      rebind.rs receipt.rs                create

    src/runtime/scroll/                   create: canonical scroll owner axis
      allocation/                         move: scroll_owned_allocation/*
      request.rs routing.rs               create
      state.rs anchoring.rs               create
      rebind.rs receipt.rs                create

    src/runtime/focus/                    create: semantic focus owner
      request.rs routing.rs state.rs      create
      participant.rs restoration.rs       create
      modality.rs active_descendant.rs    create
      rebind.rs receipt.rs                create

    src/runtime/selection/                create: UI selection owner
      request.rs state.rs range.rs        create
      reconciliation.rs rebind.rs receipt.rs

    src/runtime/interaction/
      source/route_source.rs              modify: CommandRoute sibling

    src/runtime/command_routing/          create: sibling owner emitting route
      context.rs index.rs planning.rs     create
      prefix.rs occupancy.rs receipt.rs   create

    src/runtime/motion/                   create: committed track owner
      declaration.rs track.rs retarget.rs create
      produced_fact.rs rebind.rs          create
      receipt.rs census.rs                create

    src/mounting/presentation/
      motion_sampling/                    create: derived presentation only
        sampling.rs interruption.rs       create
        damage.rs receipt.rs              create

    src/runtime/observation/family/mod.rs modify: owner-ranked service facts
    src/runtime/invalidation_narrowing/   modify: point derived indexes at the
                                                  canonical portal/scroll/
                                                  selection owners; do not move
                                                  selection authority here

    src/runtime/intent_execution/provider/
      registry.rs                         modify: install typed destinations
      prepared.rs                         modify: remove unsupported settlement

    src/facade/entry/app_builder/
      intent_registration.rs              modify: remove unsupported helper
      service_policy.rs                   create: per-family policy defaults

    src/mounting/
      presentation/preflight.rs           modify: narrow solicited mechanics
      projection/                         modify: family projections in existing
                                                  semantic/mechanical owners

    src/inspection/service/               create: bounded internal summaries

  worth-ui-host-contract/
    src/mounted_frame/protocol.rs         modify: observation + focus-placement
                                                  solicited-effect schema
    src/runtime/solicited_effect/
      focus_placement.rs                  create
      outcome.rs cancellation.rs          create: transport values; settlement
                                                  stays with presentation owner
    src/observation_report/
      family.rs                           modify: WindowFocus
      payload.rs                          modify: exact scroll/window payloads
      report.rs                           modify: required typed bases

  worth-ui-host-native/
    src/native/solicited_effect/
      focus_placement.rs                  create
    src/native/event_focus.rs             modify: WindowFocus + surface
    src/native/event_scroll.rs            modify: exact ScrollDelta

  worth-ui-host-headless/
    src/solicited_effect/
      focus_placement.rs                  create
    src/headless_transcript/              modify: observe exact mechanics

  worth-ui-dsl/
    src/semantic/service/
      portal.rs focus.rs motion.rs command.rs scroll.rs selection.rs
    src/source/compile/service/           create: source-linked typed lowering

  worth-ui-inspection/
    src/service/
      portal.rs focus.rs motion.rs command.rs scroll.rs selection.rs

  worth-ui-certification/
    tests/suites/application_contracts.rs modify: reference service modules
    tests/application_contracts/
      runtime_services/                   create: RS-07 plus ignored/filtered
                                                  RS-10 modules inside the
                                                  existing Cargo test target

workspaces/worth-ui/apps/
  platform-pulse/
    src/product_world/
      runtime_services/                   create: truthful service story only
        portal_story.rs                   create: portal/focus/motion product use
        command_story.rs                  create: winner/loser product use
        query_denial_story.rs             create: separate Query-backed action
                                                 precondition stop
      visual_composition/                 create: Pulse-private authored design
        geometry.rs                       create: reference/expanded layout
        palette.rs                        create: application token roles
        typography.rs                     create: qualified Pulse text styles
        projection.rs                     create: real fact-to-content projection
    tests/executable_world/adjudication/
      platform_pulse_control_points.json   modify: independent design oracle
```

This migration establishes one canonical family owner without pretending that
the existing allocation slices already own the whole service. Current
`portal_anchored_allocation` and `scroll_owned_allocation` modules move beneath
the new portal and scroll axes; their public meaning and proof types are
preserved byte-for-byte where representation is unchanged while imports cut
over in one phase. Existing focused allocation tests stay green in that same
commit. Old module names are not retained as re-export aliases. Existing
committed observation owners and derived invalidation indexes are extended to
cite the canonical owners. They are never copied or kept as compatibility
lanes.

Mounted focus participation, motion participation, presentation receipts,
host batching, Mosaic focus-scope, and Mosaic scroll-ownership definitions are
modified and consumed in place. Command descriptor meaning stays in the
command registry, command routing is a sibling runtime owner, and only its typed
route-source progression enters interaction/intent admission. Runtime motion
owns committed track meaning; derived sampling stays spatially next to mounted
presentation because samples share presentation lifecycle and scale, not
observation or host-effect lifecycle.

The following obsolete surfaces are removed at cutover:

- `UiUnsupportedServiceIntentExecutionBinding`;
- `register_unsupported_service` and
  `register_unsupported_intent_definition`;
- `worth_ui.runtime_service.unsupported` as the universal service outcome;
- `default_shortcut_reference: Option<String>` and any string shortcut match
  path; and
- the ambiguous host observation name `Focus` for window-focus state.

No compatibility module, re-export alias, deprecated duplicate, or old/new
switch remains. Git history is the migration archive.

The public facade has one canonical service declaration path. The runtime has
one canonical owner per family. The host contract extends existing protocol
families and adds only narrow solicited-effect siblings. Inspection consumes
produced evidence and cannot become another owner.

## Ordered Phases

### Phase 1: Contract, topology, and protocol freeze

Make the family boundaries, common request basis, authority placement,
proposal DAG, geometry brands, observation/solicited-effect schemas, migration
inventory, private destination modules, 960-by-600 Pulse composition contract,
and independent visual-contract manifest compiler-visible. Move the current
portal-anchor and scroll-owned allocation modules beneath their canonical
family axes in the same cutover; leave no old import lane. Extend dependency,
family-switch, publication-owner, geometry, target, and feature-matrix
enforcement. Export no new public service facade names or DSL grammar. Do not
claim the redesigned Pulse complete before its service facts are real. The
next phase may trust that illegal dependency directions, competing owners,
erased payloads, string shortcuts, raw Query/replay imports, fake Pulse
controls, and a production-constant-derived visual oracle cannot become the
foundation.

### Phase 2: Proposal compilation and lifecycle ownership

Implement typed proposal preflight, fixed dependency ordering, occupancy,
cancellation, budgets, bounded evidence, and proposal census against recorded
typed fixture proposals. The compiler emits candidate 3.12 facts and mounted
work but cannot publish or issue effects. Prove stale denial, cycles, scope
widening denial, bounded conflict handling, family-switch enforcement, and
zero-resource teardown. The next phase may trust one lawful way to stage
independent family proposals without creating another publisher.

### Phase 3: Focus and portal

Implement focus scopes, traversal, restoration, focus-visible modality,
window/semantic focus separation, portal placement/layers/lifecycle/dismissal,
native/headless focus mechanics, negotiated portal mechanics, and their
compiled proposal set. Establish the larger Pulse geometry, authored palette,
qualified typography, and truthful static product regions before adding the
real portal interaction. Install the real `OpenPortal` and `ClosePortal` intent
providers through 3.14 admission and delete only their unsupported bindings.
Consume existing Mosaic, anchor, allocation, presentation, and mounted
evidence. Prove the native dropdown/open/focus/dismiss slice that will later be
consumed by `RS-01`, plus focused protocol cases:
anchored placement, nested/modal posture, rebind fallback, duplicate dismissal,
anchor loss, and indeterminate settlement through the existing presentation
owner. The next phase may trust coherent portal and focus truth on a public-
behavior path, although broad public presets remain unexported.

### Phase 4: Motion and presentation sampling

Implement one runtime presentation-sample clock, existing Tick/readiness wake,
track derivation from exact predecessor/successor receipts, once-per-retarget
semantic publication, presentation-only sampling, interruption, reduced motion,
exit retention, branded hit testing/anchoring, damage, and zero inactive work.
Integrate portal entrance/exit and rebind retargeting through proposal
compilation and existing mounted presentation.
Remove any assumption that a host generic motion capability owns animation.
The next phase may trust exact current presentation geometry.

### Phase 5: Scroll and selection

Implement nested scroll routing, high-precision host delta contracts,
programmatic reveal, anchoring, bounds, rebind, and reconciliation. Implement
stable-key single/multiple/range selection, reorder/removal reconciliation, and
compact deltas. Coordinate focus reveal and declared focus-selection actions
through proposal compilation without direct family calls. Preserve Query-
provided extent while proving it cannot become offset/cursor authority. Prove
high-frequency bounds and Query separation. The next phase may trust coherent,
declared-axis focus and selection context for commands.

### Phase 6: Command routing and public cutover

Replace string shortcuts with typed keys, modifiers, aliases, strokes, and
sequences. Implement context indexes, precedence, prefix state, IME/repeat
policy, ambiguity, registration-owner lifecycle, and route receipts. Expose the
`UiIntentRouteSource::CommandRoute` sibling and prove that route success still
crosses 3.14 admission. Expose the canonical Rust facade, per-family policy
defaults, presets, DSL grammar/lowering, source-linked validation, and
normalized plans for all six families only after behavior is real. Install the
real `InvokeCommand` provider. Delete the remaining unsupported-service and old
shortcut paths. The next phase may trust that product code cannot reach a
parallel lane or a premature facade.

### Phase 7: Inspection, product courtrooms, and closure

Complete bounded `why_*` inspection, native/headless parity, deterministic test
kit, named cost counters, exact resource census, documentation, native `RS-01`,
protocol `RS-07`, scale `RS-10`, deletion inventory, and all repository checks.
Keep all proofs in the inherited Pulse executable and existing certification
targets. Close the Pulse visual contract from structural facts, stable native
pixel masks, deterministic 1.0/1.5-scale presentation evidence, a real-runtime
documentation screenshot, the two-size native journey, and the explicit
judgment-based product/design screenshot review; no mockup or whole-frame
golden substitutes. Automated visual evidence protects objective contracts but
cannot substitute for the contemporary-product quality judgment. Only the
faults the operating system can honestly cause are claimed by the native
subset; scripted protocol faults remain
protocol evidence. Only this phase closes Milestone 3.15 and hands the
contracts to 3.16.

Evidence is built with each owner phase, not postponed to Phase 7. Phase 7 is
the cumulative adversarial use of that evidence.

## Documentation Deliverables

Implementation is incomplete until the same change set:

- creates `workspaces/worth-ui/docs/runtime-services.md` as the durable public
  mental model and family guide;
- revises `workspaces/worth-ui/docs/runtime-subsystems.md` with the six owners,
  proposal compiler, existing publication/presentation owners, narrow
  solicited-effect boundary, and cost lanes;
- revises `workspaces/worth-ui/docs/interaction-and-intents.md` to show which
  explicit service operations consume 3.14 admission and which service events
  do not pretend to be intents;
- revises `workspaces/worth-ui/docs/native-host-platform.md` for window focus,
  solicited semantic-focus placement, rich scroll input, existing Tick/
  readiness wake, mounted portal overlays, typed-unsupported OS popup surfaces,
  settlement, reconciliation, and schema negotiation;
- revises `workspaces/worth-ui/docs/architecture.md` and
  `workspaces/worth-ui/docs/worth-ui-readme.md` with the final authority map;
- revises `workspaces/worth-ui/docs/application-lifecycle.md` with the exact
  960-by-600 default client area, reference-region geometry, honest visible
  capabilities, resize behavior, design-token roles, launch/actions, and one
  current screenshot captured from the real runtime rather than a mockup;
- revises `plans/worth-ui/ai-diagnostics.md` so AI and developer tools consume
  bounded service evidence rather than logs or internal state;
- updates `workspaces/worth-ui/README.md` and `workspaces/worth-ui/AI_README.md`
  orientation links; and
- compiles every public Rust and DSL example through certification fixtures.

The docs must teach portal/focus/motion proposal compilation, semantic versus
presentation geometry, command versus Query admission, scroll versus Query
cursor, selection versus Query identity, hot rebind, host settlement, resource
budgets, the Pulse's product-quality and capability-honesty rules, and the
explicit undo/redo exclusion. They may not promise a future service or visual
state that the code does not ship.

## Must Ship and Preserve

Milestone 3.15 must ship:

- one production owner each for portal, focus, motion, command routing, scroll,
  and selection;
- the small typed request basis and non-publishing proposal compiler;
- owner-produced facts and mounted work consumed by existing atomic publication
  plus existing typed physical settlement/recovery;
- compiler-visible geometry brands, coordination occupancy/order, and
  persistence postures without persistence or undo authority;
- explicit family support, capability negotiation, budgets, census, and
  bounded inspection evidence;
- the typed shortcut vocabulary and conflict-aware command route;
- the revised observation schema and any narrow solicited-effect contract in
  native and headless hosts;
- the canonical Rust facade, policies/presets, DSL lowering, validation, and
  deterministic test kit;
- the designed 960-by-600 Platform Pulse composition, truthful service story,
  real-runtime lifecycle screenshot, independent visual-contract oracle, and
  two-size/scale-sensitive visual evidence, plus an explicit product/design
  review against the Linear-or-Notion contemporary-product quality bar;
- native `RS-01`, protocol `RS-07`, scale `RS-10`, and the focused proof
  portfolio inside existing targets; and
- deletion of unsupported service and string shortcut authority paths.

It must preserve:

- 3.14 admission and managed attempt semantics;
- Query audience and replay boundaries;
- declaration, allocation, mounted, presentation, and host identity
  distinctions;
- existing Mosaic focus-scope and scroll-ownership meaning;
- one-way tier direction and proof-carrying authority;
- ordinary versus reconstructive cost separation; and
- exact lifecycle and shutdown ownership.

## Acceptance and Successor Handoff

Milestone 3.15 closes only when:

- native `RS-01`, protocol `RS-07`, and scale `RS-10` each pass for their exact
  claim without substituting one proof world for another;
- the Pulse launches at a 960-by-600 logical client area, remains coherent at
  1120 by 700, and satisfies its region, rhythm, target-size, contrast, text-
  containment, wrapping-safety-inset, hit-test-honesty, node-count, and
  capture-budget contracts, including stateful two-line native raster evidence
  for Source Signal detail and Query Posture at both sizes;
- a designated product/design reviewer inspects real-runtime screenshots at
  both sizes and gives a defensible yes to “Would this look like it belongs
  alongside Linear or Notion as a contemporary desktop product?”, judging the
  whole composition's hierarchy, proportion, spacing, typography, restraint,
  polish, and coherence rather than treating automated checks as an aesthetic
  oracle;
- every visible Pulse fact comes from the real source, Query, intent, service,
  mounted, or host boundary it claims, every visible control has shipped
  behavior, and no screenshot, host decoration, or production-derived oracle
  manufactures success;
- every supporting proof is mutation-sensitive and uses independent evidence
  appropriate to its claim;
- all six public service contracts are usable without runtime-internal imports
  or app-local replacement owners;
- portal opening, focus placement, motion, scrolling, selection, command
  routing, rebind, physical settlement, reconciliation, and shutdown have
  typed terminal outcomes;
- no ordinary operation performs a forbidden full-world scan or retains
  unbounded history;
- unsupported mechanics fail before effect or enter the exact typed recovery
  posture required by when failure occurred;
- `UiServiceProposalCompiler` cannot publish, issue/settle host effects, retain
  family state, or implement family behavior through classification switches;
- no raw Query, ordinary replay, generic authority marker, service-to-service
  call, string shortcut route, unsupported placeholder, undo/redo promise, or
  `provisional_aftermath` surface remains;
- native and headless hosts satisfy the same versioned contract;
- public and DSL examples compile;
- documentation and roadmap links resolve; and
- formatting, focused tests, feature matrices, line caps, boundary checks,
  generated context checks, and the broader lanes demanded by implementation
  risk pass on the final scoped source.

Milestone 3.16 may consume focus, selection, portal, scroll, motion, and command
posture as typed appearance inputs. It may not read owner internals or invent a
second interaction-state lane.

Milestones 4 through 16 may add multi-window topology, shell policies, richer
controls, Query-backed tables, canvas/spatial operations, persistence,
accessibility, native menus/dialogs, plugin registrations, and developer tools
through the insertion points above. They may not replace service identity,
authority, proposal compilation, host settlement, rebind, cost, or evidence
contracts.

Undo and redo remain aspirational. A future governing specification must
decide their history authority, grouping, domain admission, persistence,
indeterminate-effect, and service-participation laws before any actionable API
or runtime behavior is added.
