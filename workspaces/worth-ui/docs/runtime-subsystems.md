# Runtime Subsystems

This map is for contributors placing runtime work. Application developers
should start with [Application lifecycle](./application-lifecycle.md).

## Runtime Owners

| Owner | State or contract | Owns |
| --- | --- | --- |
| application | `WorthUiApplicationSessionState` | active generation, framework transitions, source basis, application cutover |
| graph | `UiGraphSnapshot` and graph indexes | graph truth, produced-fact consumers, and mutation successors |
| planning | allocation and rebind plan compilers | admitted inputs, affected scope, identity lifecycle, and sealed plans |
| interaction | gesture, draft, targeting, and semantic-interaction registries | presented-frame continuity, capture/loss, bounded local input, and interaction receipts |
| intent admission | route, payload, operability, confirmation, and admission registries | typed candidates, affine challenges, concurrency, bounded admission slots, and settlement |
| intent execution | definition/provider bindings and attempt registries | destination dispatch, versioned providers, polling, cancellation, terminal posture, and recovery |
| portal | `UiPortalRuntimeState` plus moved anchored-allocation contracts | logical open/close lifecycle, layers, mounted-overlay placement, dismissal, rebind, and portal-produced facts |
| focus | `UiFocusRuntimeState` | semantic keyboard focus, scopes, participants, traversal, active descendant, modality, restoration, rebind, and focus-produced facts |
| scroll | `UiScrollRuntimeState` plus moved scroll-owned allocation contracts | semantic offset, nested routing, bounds, anchoring, reveal, rebind, and scroll-produced facts; Query may supply extent but never offset |
| selection | `UiSelectionRuntimeState` | stable application item keys, selection set/anchor/lead, range operations, reconciliation, rebind, and selection-produced facts |
| motion track | `UiMotionRuntimeState` | committed track identity, predecessor/prepared-successor basis, retarget policy, rebind, terminal posture, and motion-track facts |
| command routing | `UiCommandRoutingRuntimeState` | typed shortcut matching, declared-axis context, prefix occupancy, conflict resolution, registration-owner lifecycle, and `CommandRoute` source receipts |
| service proposal compilation | `UiServiceProposalIdentity` plus occupancy lease | non-authoritative cross-owner ordering, cycle/currentness/budget preflight, and cancellation posture; it owns no family successor, publication, or host settlement |
| mounting | `WorthUiMountedSessionState` | mounted identities, frame assembly, retention, reconciliation, and mounted publication |
| presentation producer | `UiMountedPresentationState` plus derived motion sampling | receipt-keyed retained commands, initial/delta/unchanged work, current track samples, total order, damage, and post-settlement candidate commit; it cannot mint motion-track meaning |
| semantic-focus host effect | `UiHostFocusPlacementRequest` plus acknowledgement/reconciliation state | the narrow prepared/issued/settled-or-indeterminate physical-focus lifecycle; it cannot decide semantic focus |
| host exchange | `WorthUiHostExchangeSessionState` | structural host reports, measurement exchange, quarantine, and transport evidence |
| visual inspection | `WorthUiVisualInspectionAuthority` plus capture/overlay registries | grants, retained snapshots, comparison, overlays, and visual resource bounds |
| rebind lifecycle | `UiRebindRuntimeState` | plan, receipt, completion, recovery, terminal-decision, and causal-evidence capacity |
| inspection bridge | typed inspection queries and projections | read-only indexed summaries, exact references, support, and expiry posture |
| session | `WorthUiActiveApplicationSession` | thin composition of complete transitions across established owners |

The session is not a bag of the other owners' fields. It coordinates named
transitions and returns product outcomes. New state belongs in the owner that
can establish, rebuild, and dispose its truth.

Query projection state is not another session owner. Query owns the live
resource; application requirements and host progression cross
`worth-query-decl` and `worth-query-host`; `worth-ui-query-binding` converts
the resulting Query-issued products into shape-specific UI observations and
affine facts. The existing observation, planning, mounting, and publication
owners consume the resulting UI meaning.

Product intent effects follow the same rule. UI admission owns no product or
Query mutation authority. The exact typed provider calls a separately admitted
product action; only that owner-issued result may become a declared consequence
for ordinary rebind and mounted publication.

## Allowed Dependency Direction

```text
graph
  -> planning
host observation + mounting identity
  -> interaction
interaction + declarations + application facts
  -> intent admission
intent admission + definition/provider bindings
  -> intent execution
host observation + declarations + planning/presented evidence
  -> portal + focus + scroll
declarations + application collection facts
  -> selection
command capabilities + key observation + declared focus/selection axes
  -> command routing
command routing
  -> intent admission through UiIntentRouteSource::CommandRoute
committed predecessor + planning-issued prepared successor + motion declaration
  -> motion track
family-owned staged facts + sealed stage-complete witnesses
  -> service proposal compilation
service proposal compilation
  -> existing application/mounted publication inputs
application + graph + planning
  -> mounting
committed motion track + Tick + mounted presentation
  -> presentation producer motion sampling
mounting
  -> host exchange
application + graph + planning + mounting + host exchange
  -> rebind lifecycle
application + graph + planning + mounting + host exchange + rebind + visual
  -> inspection bridge (read-only)
named owners
  -> session composition
```

More exactly:

- graph depends on none of the mounted, host-exchange, visual, rebind, or
  inspection owners;
- planning may read graph and sealed application meaning;
- interaction may consume admitted host observations and exact presented
  targets but cannot route, execute, or publish product effects;
- intent admission may consume semantic interactions, declarations, and
  application facts but cannot call providers or mutate product domains;
- intent execution consumes one move-only admission and the exact typed
  destination binding; provider completion still cannot publish directly;
- portal, focus, scroll, selection, motion-track, and command-routing owners
  retain their own staged/current state and emit owner-ranked facts; none may
  call another family;
- command routing is a sibling of interaction and intent admission. It consumes
  only declared context axes and emits a `CommandRoute` source receipt; it does
  not execute or admit an intent;
- service proposal compilation retains only identity, occupancy, and
  cancellation posture. Its fixed transaction preflights the coherent family
  set, uses a keyed application-generation/semantic-surface neighborhood index,
  reserves exact family/scope occupancy, accepts one sealed witness per
  owner, canonicalizes inert fact/work references, consumes one result from the
  existing publication boundary, and releases only after every exact owner
  acknowledges that result. Denials and all prepublication teardown routes are
  census-atomic. Zero-witness work remains shutdown-abandonable; once a witness
  closes the before-effect window, shutdown returns recoverable typed remainder
  evidence until every outstanding owner supplies its terminal outcome. The
  compiler cannot retain successors, implement family algorithms, publish, or
  settle host effects;
- the motion-track owner may consume predecessor and prepared-successor
  evidence from planning but mounting/presentation cannot emit motion-track
  facts. Presentation samples consume committed track meaning one-way;
- the appearance owner resolves declared roles against the admitted theme and
  coherent owner snapshots for exact mounted occurrences. It consumes accepted
  Motion samples and Portal-issued relations during preparation but owns no
  interaction, service, geometry, or host paint truth;
- mounting may consume application, graph, and sealed planning output;
- the presentation producer derives host work from mounted authority; hosts do
  not rediscover deltas from a complete projection;
- semantic-focus placement crosses one narrow solicited-effect port after the
  Focus owner selects the semantic target. Window focus remains an unsolicited
  observation, Portal remains mounted overlay work, and Motion adds no host
  animation protocol;
- host exchange may observe mounted transport but cannot publish;
- rebind may coordinate owner-issued observation, plan, application, mounting,
  appearance/theme preparation, and host outcomes but does not absorb their
  source truth. Acceptance commits the exact prepared owner succession used for
  pixels; rejection keeps predecessor paint and state;
- visual inspection borrows exact mounted evidence and rebind affinity without
  becoming a publication owner;
- inspection reads named projections and cannot mutate or reconstruct them; and
- session composition may call complete transitions but cannot invent a
  parallel state store.

Graph must never import mounting. Planning cannot mutate mounted or observation
state. Observation cannot publish a frame. Inspection cannot reconstruct
operational truth.

## Scroll, Motion, And Direct Control

Scroll owns region bounds, routing, semantic targets and accepted offsets.
Motion owns interpolation and velocity-continuous retargeting. Mounted
presentation binds each track to its exact Scroll occurrence and indexed paint
group, then carries one prepared result through ordinary host acceptance.
Preparation is not acceptance: refused frames preserve predecessor offsets,
geometry and pixels, with bounded pending evidence available for retry.

The native shell and admitted tick observations use the same settling path:

```text
prepare_motion_tick
-> present_prepared_motion_tick
-> settle_accepted_scroll_sample
```

The curve is time-based; Pulse selects a 120 ms horizon from the latest coarse input.
A dropped or rejected frame does not advance accepted state or change the
endpoint. New input accumulates against the pending target and retargets from
accepted position and velocity, including direction reversals. An elapsed
horizon alone cannot retire a target whose endpoint the host has not accepted.
Displayed geometry, hit testing, hover and
thumbs consume the accepted sample, never the ahead target. A first sample may
still be at rest; settlement is not a promise about a fixed number of frames.
The native wake lane offers samples frequently enough not to withhold a display
interval; FIFO presentation and the actual display determine visible cadence.

Pixel deltas already express travel and apply directly, including OS momentum.
For coarse input, the host admits the platform lines-per-notch and runtime
multiplies by the consuming region's declared line extent. Pulse declares
20 points, giving 60 points for the usual three-line notch. Page input uses the
viewport-minus-line step. Missing required line extent is a typed refusal, not
an implicit one-point conversion. The immediate default remains usable without
Motion; smooth policy requires admitted Motion support.

A phased gesture remains latched to its first consuming owner until end or
cancellation, even when that owner reaches an edge. A new gesture beginning at
an edge may select a consuming ancestor. Unphased wheel input uses its declared
quiet interval. Modality loss, removal and reincarnation invalidate the latch.
Nested Scroll groups compose each ancestor displacement exactly once. Their
membership and clip provenance are indexed at ordinary frame admission, not
rediscovered by a graph walk on each motion sample.

Moving commands retain intrinsic geometry and clipping separately from their
stationary viewport clips. A fully clipped glyph remains available to reveal
when content moves back into view; explicitly suppressed content retires.
Presentation derives device-grid-snapped text, outlines and thumb geometry
without changing semantic subpixel offsets. Hit targets retain the unsnapped
accepted pose. Native sampled-appearance coverage has its own spatial index,
updated transactionally and restored on rejection or reconstruction, so local
damage does not scan every retained sampled command.

A Mosaic region declares chrome axes, metrics and registered appearance roles.
Runtime derives the track and thumb from accepted bounds and offset; applications
do not supply thumb coordinates. The thumb's clip remains its viewport, not its
old rectangle, so subsequent samples can move throughout the track.

Pressing a moving thumb takes capture at the accepted pose and ends its easing
with `DisplacedByDirectControl`. The press does not place content. Drag and track
placement stage exact direct successors; their offsets and transition retirement
commit only when the corresponding ordinary frame is accepted. Further direct
input accumulates on bounded pending intent. Wheel and keyboard input cannot
restart the captured axis. Release outside the gutter retains capture semantics
until release and leaves the accepted drag offset in place.
Native client exit preserves the held button and capture epoch; focus loss
still cancels capture, and a release outside the window clears the held button.
Release submits its event-time position through ordinary direct succession
before dropping capture, so a coalesced final move cannot lose the endpoint.

Layout extent succession also stays pending until presentation. Accepted
bounds and offsets remain coherent with displayed content while a resized or
reclamped candidate is refused. Acceptance reconciles the pending target and
accepted offset together. New input cannot overwrite an unpublished layout or
direct successor with incompatible evidence.

Empty paint groups still cross ordinary host completion: an accepted semantic
Scroll sample can change geometry without inventing a paint command. No-host
acknowledgement is not a substitute. Stationary-pointer hover re-resolves from
accepted geometry across ordinary frame succession. Reconstruction restores
accepted geometry but retires pointer presence from the old binding; newly
admitted input resolves against the replacement binding.

Empty content, removal, attention loss and shutdown terminate tracks with named
causes and retire capture, latches and retained sample work. Routing and sampling
visit the consuming chain and indexed affected commands/regions, not unrelated
mounted instances. Delivered reports retire after synchronous consumers; exact
pending input consequences remain bounded until acceptance or cancellation.
Neither event history nor wheel duration grows retained storage.

## Rebind Construction

Every active session constructs `UiRebindRuntimeState` from the prepared
application's `UiRebindProfile`. The state owns bounded registration for plans,
terminal receipts, in-flight completion, uncertain recovery, terminal decision
records, and retained causal evidence. A profile is required; there is no
ambient default hidden in the executor.

The ordinary source route is:

```text
native shell
-> begin_source_rebind
-> application-owned observation/classification/scope/plan
-> rebind lifecycle final admission
-> mounting and host effects
-> application + mounted atomic cutover
```

The legacy whole-application replacement facade is not an alternate route.

## Failure Preservation

Each subsystem owns its denial:

- source/DSL denial retains the exact attempted revision and compile report;
- application denial retains predecessor app, graph, allocation, and Query
  state;
- graph or planning denial leaves current snapshots and indexes unchanged;
- mounting denial preserves prior mounted identity and publication;
- host-exchange denial cannot mutate application or mounted truth;
- rebind before-effect denial retains the predecessor and exact valid next
  action;
- uncertain effects retain completion or recovery authority until
  reconciliation or shutdown; and
- inspection denial has no operational side effects.

The session preserves this ordering and never publishes a partial
cross-subsystem successor.

## Cost Ownership

Owners report their own work. The session carries receipts but does not invent
duplicate totals.

- source acquisition and DSL compile are reconstructive source cost;
- classification/scope/planning use `O + F + A + C + R + G + M + B`;
- mounting and adapter effects report physical presentation cost;
- delta carriage, draw-list/order mutation, damage-index/replay work, pixels,
  GPU writes, passes, copies, acquisitions, submissions, and presents remain
  separately named counters;
- reconciliation reports recovery cost separately; and
- rich inspection/report materialization is explicit diagnostic cost outside
  measured executor intervals.

Runtime-service owners additionally report their local candidates, affected
records, stale denials, rebind outcomes, and live resources. The proposal
compiler reports only proposals and requirements visited. Motion sampling
reports active tracks, with zero inactive-track work. Scroll reports current
chain depth, Selection reports changed keys or the explicit range, Command
Routing reports its indexed candidate set, and Portal reports only affected
layer/anchor neighborhoods. The scheduled scale courtroom requires
`unrelated_neighborhoods_touched` to remain exactly zero; this documentation
lane does not close that ignored reconstructive proof. These counters are
owner evidence, not totals reconstructed by session composition.

Saturation returns typed denial or backpressure. It must not trigger a hidden
whole-graph scan, universal remount, or unbounded queue.

## Successor Homes

| Capability family | Existing owner to extend | Forbidden alternate |
| --- | --- | --- |
| projected product data | Query binding consequence plus owner-specific observation facts | session data cache |
| semantic interactions and product intents | interaction, intent admission, and intent execution owners feeding declared consequences | host callback or session executor |
| portal, focus, motion, command-routing, scroll, and selection services | Milestone 3.15 family owners; intent-origin requests enter through typed destinations, while a non-publishing proposal compiler lowers cross-owner work into the existing 3.12 publication and mounted-presentation owners | reopening generic intent admission, adding a service publication/settlement authority, implying undo/redo authority, or using renderer state |
| portals, focus, motion, appearance | typed produced facts and consumed aspects feeding rebind planning | renderer-local state |
| expression evaluation | planning over sealed DSL expression artifacts | runtime reparsing |
| authored modules and composition | `worth-ui-dsl` before sealed semantic handoff | session composition |
| human and agent diagnostics | inspection projections over retained exact references | replay in ordinary runtime |

Future work should insert at these homes and reuse the canonical rebind
executor. If a feature requires moving source settlement, graph truth,
publication, or host authority, its architecture is not yet honest.

For projected text, planning selects the declared scalar or collection fact,
mounting owns the semantic text node and `BodyDefault` appearance role, and the
host adapter translates only the resulting mounted mechanics. A renderer-side
field lookup or Query call is therefore both an ownership violation and a
second data path.

## Related Docs

- [Worth UI architecture](./architecture.md)
- [Interaction and intents](./interaction-and-intents.md)
- [Runtime services](./runtime-services.md)
- [Hot rebind](./hot-rebind.md)
- [Application inspection](./inspection.md)
- [Visual inspection](./visual-inspection.md)
- [Worth Query AI README](../../worth-query/crates/worth-query/docs/AI_README.md)
