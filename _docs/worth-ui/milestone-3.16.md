# Milestone 3.16: Appearance, Theme, and Visual State Projection

## Status and Placement

Implementation is in Gate 4. Gates 0 and 1 are complete; Gate 1's approved
integration is `e1d61bce2aeaa059777028f861e797f09c0ea785`. All five Gate 4
sections remain open. Static paint stays the live publisher until Gate 5.

| Section | Required result |
| --- | --- |
| [4a Mounted geometry](#gate-4a--mounted-geometry) | Exact occurrence allocation, surface binding, clipping, and paint order |
| [4b Text foreground](#gate-4b--text-foreground) | Adopted spans, actual-image damage, accepted coverage, and paint-only work |
| [4c Motion composition](#gate-4c--motion-composition) | Accepted samples composed once in both directions |
| [4d Backdrop and Portal](#gate-4d--backdrop-and-portal) | Authored instances, extents, total order, and lifecycle |
| [4e Integrated closure](#gate-4e--integrated-closure) | Combined mounting/headless behavior, locality, failure, reconstruction, and cleanup |

**Open product decision:** Gate 4a needs a structural placement contract.
Explicit parent-relative placement for 3.16 and advancing automatic split/stack
layout from Milestone 4 have different product meanings. Region roles, scalar
sizing, and eligibility do not determine sibling rectangles. Settle that choice
before occurrence geometry can close.

This spec governs decisions and acceptance. Routine implementation history and
test-run reports belong in task output and code review, not this document.

## Goal and Central Claim

Appearance is a deterministic, inspectable, runtime-owned projection of an
explicit role attachment, an admitted surface theme, and one coherent vector
of owner-issued state. Hosts execute sealed mechanics rather than choose UI
meaning. Changes reach only consumers whose declared inputs changed.

```text
node + role + surface theme + coherent owner snapshot
  -> UiAppearanceProjection -> mounted facts/mechanics -> existing settlement
backdrop declaration + extent/presence/placement/Motion bases + role/theme
  -> UiBackdropAppearanceProjection + UiOverlayStackSnapshot -> same settlement
```

Preserve canonical Rust/file lowering, qualified text, owner-issued interaction
and service truth, delta mounting, atomic rebind/publication, and native physical
reconciliation. Pixels, inspection, indexes, and projections are derived evidence,
not authority. Repository coding guidelines, [UI vision](./worth-ui-vision.md),
[DSL vision](./worth-ui-dsl-vision.md), [3.15](./milestone-3.15.md), and existing
Query-binding, source, interaction, text, and host contracts remain governing.

## Non-Goals and Explicit Exclusions

Milestone 3.16 does not ship:

- a CSS-compatible style system, selectors, specificity, inheritance,
  cascading variables, class lists, modifier-order semantics, or an ambient
  environment;
- a broad professional component library, curated light/dark/high-contrast
  suite, density system, component gallery, or design-system governance; those
  are Milestone 9 work built on this runtime lane;
- arbitrary shaders, gradients, image fills, materials, blur, shadows,
  backdrop filters, child clipping, masks, or renderer escape hatches;
- arbitrary animation transitions for appearance values; Motion remains the
  owner of temporal progression and 3.16 supplies only stable projected
  endpoints;
- layout, participation, presence, visibility, allocation, hit-test
  membership, focus eligibility, accessibility state, or product behavior
  changes disguised as appearance;
- author-defined executable expressions or conditions; Milestone 3.17 may
  produce typed inputs that this same resolver consumes;
- live style trials or source patch proposals; Milestone 3.22 adds those as a
  canonical replacement workflow over this lane;
- plugin theme contribution, precedence, permissions, unload, or platform
  override authority; Milestone 15 consumes the insertion point established
  here;
- OS-wide theme detection, multi-monitor color management, HDR, or universal
  pixel identity;
- a second publisher, retry loop, recovery registry, render tree, style cache,
  or host-local widget state;
- a new Query interpretation layer, raw Query import, Query replay in ordinary
  code, or any widening of the predecessor `worth-ui-query-binding` dependency
  edge;
- actionable undo or redo, or any exposure of Query's
  `provisional_aftermath` experiment; and
- fake controls, modal behavior, icons, status, or state used only to make a
  screenshot look capable.

Unsupported visual families are named and typed. They may not fall back to a
nearby color, square corner, missing outline, adapter theme, or silent no-op.

## Decisive Proof Portfolio

Use existing certification targets and the cumulative Pulse, not new executables
or a proof ledger. Focused tests protect local behavior; integrated claims cross
real owner boundaries. Simulated observations establish only their stated scope.
Oracles may not call/copy the disputed resolver, selector, comparer, normalizer,
compositor, or shader. Fixtures cannot mint live authority or physical success.

### `AP-01`: Native cumulative Platform Pulse

Extend the same process at 960x600 and 1120x700, retaining source/Query/service
owners, node/capture bounds, runner restrictions, and <=45-second journey. Runner
uses OS input, watched edits, resize/focus, external capture, and public observations;
no owner/resolver calls, receipt/color injection, or direct cursor setting.

Show real normal/hover/pressed/focus-visible activation, owner-backed inoperability
or validation, Selection/Focus, theme switch/back, resize, and rebind that moves
hover and removes/reincarnates a focused/selected target. Invalid edits preserve
the predecessor. At both sizes drive a real two-deep modal stack through
`0 -> 1 -> 2 -> 1 -> 0`; each dialog has title/body/real Cancel and primary actions
and a separately authored viewport backdrop immediately before it. Verify topmost
input shielding and focus restoration independently of paint.

Keep 24px gutters, eight-point rhythm, >=32x32 targets, 4.5:1 body text and 3:1
purposeful-boundary contrast, text safety insets, and truthful wrapped Source
Signal/Query Posture lines. Hover luminance is >=8% darker with preserved contrast;
authored channel differences exceed twice interior channel tolerance 2. Pressed
is independently distinct; focus-visible comes from modality and cursor from Intent.
These are Pulse visual contracts, not platform defaults.

Switch changed shared/single-consumer slots, an equal-value slot, and an unused
slot. Observe exact selection/suppression, zero unrelated/paint-only text work,
and settlement. Explain all six aspects. Independent visual-contract control
points, source-affine snapshots, external pixels, and resource census prove
cumulative application dimming, the second backdrop dimming the exposed first
dialog, no top-dialog self-dimming, and depth-one restoration within tolerance.
Masks may exclude AA/edges/motion/external chrome, never disagreeing stable interiors.

Human review of the actual executable at both sizes/modal depths/focus-visible
must meet the Linear-or-Notion quality bar: coherent non-flat Mosaic, conventional
hierarchy/actions, restrained elevation, no disconnected cards/Fluent imitation/
muddy slab/weak wash. Mechanical checks cannot approve design; design cannot
approve authority or locality.

### `AP-07`: Coherent-state, authority, and protocol hostility

Through production declarations/session/rebind/publication and headless/native
contracts, combine a six-axis node, text, two modal depths, same-surface non-modal
Portal, Motion-retained exit, and two themed surfaces. Press a selected focusable
target; move presented geometry and captured pointer; change window modality;
prepare A's theme switch; rebind/reincarnate target and change role/B's binding;
deliver stale/duplicate/foreign bases in worst lawful order; reject before effects,
then produce indeterminate physical work; shut down with pointer/switch/exit and
reconciliation obligations live.

Require coherent exact bases, no stable-ID inheritance or cross-surface theme
authority, exact pre-effect preservation, retained indeterminate owner, and zero
clean-shutdown census. Include modal-without-backdrop/non-modal-with-backdrop;
preserve explicit presence/Motion/order and atomic close/rebind/exit dependents.
Wrong currentness, mixed reads, committed-geometry hover, cascade, double opacity,
early quantization, depth/ID sorting, automatic/flattened backdrops, implicit Portal
Motion, paint-controlled shielding, or timeout-as-success must fail observations.

### `AP-10`: Scale, locality, and amplification

Use the filtered closure-stress/subprocess lane on integration-spine and nightly/
master qualification. World: 4,096 nodes in 64 neighborhoods (3,072 styled, 1,024
unstyled controls), 256 roles, 512 slots, six axes with <=64 simultaneously changing
consumers, 32 text paragraphs, four themed surfaces, 64 Motion tracks, and a 32-deep
Portal stack with 48 independently placed dark/colored/transparent backdrops.
All projections fit the ordinary bound. Change 12 slots, including three equal-value
and four unused. Separate saturation denies projection 4,097.

| Operation | Charged scope |
| --- | --- |
| State | Changed owner facts + indexed exact consumers |
| Theme | Changed slots + indexed consumers |
| Resolution | Declared axes/aspects and compiled cells |
| Mounting | Changed values or attribution |
| Host | Changed commands/order + exact damage |
| Portal/backdrop | Changed rows/declarations + indexed scope/presence/placement/Motion dependents |
| Raster replay | Intersecting retained commands, separately from construction |

Observe source reads, vectors/cells, slots compared, consumers, aspect/mechanic
changes, suppression, layout reuse/requalification, damage, command add/change/
remove, Portal rows, backdrop selections/changes/replay, relation edges, pointer
retests, and unrelated neighborhoods through existing named counters.
`unrelated_neighborhoods_touched` and color-only `text_layouts_requalified` are
zero; unchanged turns/inactive demand do no work. No whole graph/catalog/theme/
relation/text/draw-list fallback. Charge cold reconstruction, maintenance, memory,
and failed admission/comparison separately; partial counters are not total cost.

## Product Decision Lock

### Appearance is a compiled semantic lane

`UiAppearanceRole` declares identity/version, source ownership/provenance, exact
aspects, per-aspect axes/total decision tables, typed slots/literals, host support,
disclosure, and successor compatibility. Attachment is explicit in canonical
component meaning. `applies_to` validates attachment, never selects descendants
or nodes by kind/name/tree position. Rust and DSL share sealed meaning/resolution.

`UiAppearanceProjection` is privately constructed immutable derived meaning for
one node incarnation/surface. It carries role revision, theme reference, coherent
state basis, per-aspect results, provenance/support/digest. It authorizes no state,
theme, or publication effects. Theme capability stays surface-owned, not copied
per node. Declarations, graph, plans, projections, mounted facts, and mechanics
remain distinct; IDs/digests/inspection cannot replace concrete authority.

### Aspect coverage is explicit and typed

| Aspect | Value | Meaning |
| --- | --- | --- |
| `appearance.background` | Straight sRGBA or explicit transparent | Own surface fill |
| `appearance.foreground` | Straight sRGBA | Adopted alpha-text ranges; future monochrome icons |
| `appearance.border` | Solid color/nonnegative logical width | Inward own-surface stroke |
| `appearance.radius` | Four nonnegative logical radii | Own background/border silhouette |
| `appearance.opacity` | Canonical unit interval | Own mechanics, not descendants |
| `appearance.outline` | Solid color/width/nonnegative offset | Outside non-hit visual ring |

Components declare required/optional aspects. Roles cover all required and only
admitted optional aspects with matching kinds; missing/extra-incompatible/wrong-
kind/unsupported coverage denies before activation. Backdrop separately admits
only background and opacity, not node-state, cursor, or other aspect coverage.

Paint changes no allocation/participation/visibility/hit/focus/accessibility/
modality/routing truth. Zero opacity/transparency do not remove declared interaction.
Rounded paint keeps declared hit geometry and does not clip descendants. Border
paints inward without allocation cost. Normalize all radii once by the minimum
proportional reduction fitting every edge pair, exact integers with nearest-even
rounding; deny border width over half the normalized minimum dimension. Outline
radii add offset to normalized surface radii; width/offset/qualified AA expand
visual bounds/damage. Ancestors may clip pixels, but own allocation/radius cannot
clip the outline or erase its expanded damage box.

`MosaicSeamPaintOwner` assigns one region per shared edge; only it paints the seam.
Interior shared corners have no radius; Mosaic declares exterior-corner posture.
Undeclared shared-edge paint is incomplete coverage, not a collapse/default rule.

### Backdrops are authored overlay participants, not modal side effects

`UiBackdropDeclaration` owns stable identity, role, extent, presence, scope,
placement, and optional Motion. Extent is SurfaceViewport or current presented
Mosaic region; presence is Always or WhilePortalPresented(declaration). Motion
independently is None or an explicit Portal presentation export. Portal kind
creates/requires no backdrop and implies no presence, Motion, or visual default.

Scope is SurfaceSingleton or PerPortalInstance(declaration). The latter mints
`UiBackdropInstanceIdentity` per exact incarnation and resolves references there;
reincarnation cannot reuse rows, and ambiguous cross-instance/scope references
deny. `UiBackdropAppearanceProjection` binds these bases, role/theme, surface, and
overlay snapshot, consumes projection capacity, and uses no fabricated node receipt.
Future pure presence expressions may enter without changing these authorities.

Placement is an acyclic typed relation: AboveSurfaceContent, ImmediatelyBefore/
AfterPortal, or ImmediatelyBefore/AfterBackdrop. Portal anchors group surface and
ordinary content indivisibly; nested Portals are separate participants. Missing,
cyclic, cross-surface, ambiguous order denies. Raw z-index, source order, depth-only
ties, and identity sorting cannot supply order.

Portal alone owns membership/parentage/activation/shielding/focus/dismissal/exit.
`UiPortalStackSnapshot` carries total `UiPortalStackOrdinal` order. Non-idempotent
open mints a monotonic session ordinal retained through closing; duplicates do
not move it, topmost replacement gets a new ordinal, exhaustion denies.

The derived planner emits one `UiOverlayStackSnapshot` bound to application
generation, surface, presentation, Portal revision, and declaration revision.
Reserve/publish all selected dependents atomically. Close/parent-close/resize/
rebind/reincarnation/exit leave no flashes, orphan commands, or partial dependencies.
Backdrops create no input shields/cursors/dismissal/focus. Clickable scrims need
separately declared interaction. Modal-without-backdrop, non-modal-with-backdrop,
Always-without-Portal, before/after, and multiple layers are lawful. Dialog actions
remain real declarations with conventional title/body/secondary/primary placement.

Compose bottom-to-top with premultiplied Porter-Duff source-over: decode straight
sRGBA using qualified sRGB transfer, multiply linear RGB by color alpha and exact
composed opacity, fold in linear light, encode at output. Gate 0 freezes transfer
constants, conversions, nearest-even points, and reference algorithm. Never flatten
runtime layers; `1 - product(1 - alpha_i)` is only an equal-black oracle cross-check.
Viewport damage is clipped viewport; region damage is presented region. Indexed
dependent construction and intersecting ordered replay have separate costs, without
graph scans or recovery-time resolution. Paint never feeds effective alpha back
as Appearance/Backdrop/Portal/input authority.

### Theme slots and theme definitions are separate meanings

The slot catalog owns `ThemeTokenId`, family/kind, source, aliases, disclosure,
and compatibility. `UiThemeDefinition` owns theme identity/revision and its complete
typed value table. Cut over bootstrap descriptor value ownership, not copy it.

| Canonical value | Rule |
| --- | --- |
| `UiThemeColor([u8; 4])` | Straight sRGBA; parse ASCII `#RRGGBB`/`#RRGGBBAA` once; six digits imply alpha 255; hex case immaterial; digest bytes |
| Opacity | Integer interval, 65,535 is one; no unconstrained float |
| `UiLogicalLength(i32)` | 1,000 subpixels/logical point; integer normalization/digests; nonnegative where required; physical conversion at host |
| Radii/stroke/outline | Typed constituents/corner order; exact nearest-even proportional reduction; no bare float/negative-zero meaning |

Aliases are kind-preserving cycle-free slot relationships, never precedence;
themes cannot change targets. Semantic changes consume exact predecessors for
monotonic revisions. Same identity/revision with different catalog/alias/value/
support meaning denies conflict. Binding generation is separate; equal revisions
from another application/surface do not substitute.

Each surface has one explicit `UiActiveThemeBinding`. Materialize defaults during
preparation, not parent/global lookup. Admit platform/application definitions;
PluginCustom/PluginAlias/PluginPlatformOverride sources and Unknown families deny
before freeze. Milestone 15 adds typed contribution-owner identity/generation;
no fabricated plugin owners or empty placeholders. Milestone 9 adds curated
light/dark/high-contrast/density themes through the same registry.

### Theme capability and switching

Application/theme admission alone constructs `UiThemeCapabilityReceipt`, binding
one definition, catalog revision, required role set, surface, application generation,
and host profile. It authorizes resolution only there, not edits/state/publication/
host effects or another revision/surface.

```text
UiThemeSwitchRequest -> admitted observation origin -> current theme admission
  -> existing affected-scope/rebind -> mounted publication/settlement
  -> UiThemeSwitchOutcome projected from UiRebindOutcome
```

Initial binding precedes activation. Source edits and programmatic switches share
this progression. `prepare_theme_switch` admits origin and exact predecessor;
no theme executor/publisher/token mutation/retry/recovery lane. Existing
presentation-state revision/CAS owns the predecessor check. Outcomes distinguish
published, observed-no-change, duplicate, superseded before effects, rejected
before effects, in-flight, and indeterminate.

Compare definition identity/revision, catalog/aliases/values, support, and scope;
equal IDs/pixels alone are insufficient. Equal output may advance evidence without
physical work; inspection distinguishes this from no observation.

### State axes consume owner-issued truth

Six sealed versioned schemas retain bounded references to full owner posture,
revision/currentness, and reason. Roles pin versions; new visual classes require
successor admission, never wildcards. Classes cannot become owner authority.

| Axis | Source owner | Visual classes |
| --- | --- | --- |
| Operability | Intent standing decision/affinity | ready, pending, occupied, denied, unsupported, stale |
| Focus | Focus target/window/modality | unfocused, focused, focus-visible, focused-window-inactive |
| Validation | Sealed application fact | unspecified, valid, advisory, invalid, pending, stale |
| Selection | Selection owner/key/incarnation | unselected, selected, anchor, cursor, selected-anchor-cursor |
| Hover | Pointer presence over presented hit testing | outside, hovered |
| Pressed | Gesture/capture | idle, armed-inside, captured-outside |

`UiIntentOperabilityStandingFact` retains the complete decision for node,
incarnation, and route. Consume the prepared catalog's unique product route;
missing/ambiguous routes deny distinctly. Multi-route use requires authored
selection. Never rerun admission, fabricate ready, or reduce evidence to a boolean.
Class-preserving changes advance evidence; instance/binding retirement and
application cutover clear indexed facts. Map existing `primary_cause()` exactly:

| Cause | Class |
| --- | --- |
| None | ready |
| Pending / Occupied / Unsupported | pending / occupied / unsupported |
| StaleTarget, WrongWorld, RebindRequired | stale |
| PolicyDenied, Readonly, ConfirmationRequired | denied |

“Disabled” is not a class; affinity remains the owner axis and currentness the
admission procedure. `UiValidationAppearanceFact` comes only from typed application
state; missing means unspecified, not valid. Draft/IME, Query, operability, strings,
and host logic cannot validate. `UiFocusAppearancePosture` maps no target to
unfocused, inactive window before modality, then keyboard-visible to focus-visible;
initial/pointer posture with a target is focused.

`UiSelectionAppearancePosture` uses owner/application key/incarnation and denies
multiple-owner ambiguity. Map anchor+cursor, anchor, cursor, selected, none in
that order. Preserve source bits and existing owner cursor, not a second lead or
catalog-index inference. Pure appearance classes do not replace owner truth.

Pointer presence owns admitted latest-value/coalesced motion. Retest on positions
or committed old/new presented geometry intersecting the last position, scoped to
binding/neighborhood. Same-target motion has zero appearance work; coalescing
invents no history. Only successful no-hit admits outside; denial preserves state.
Receipt refresh preserves input sequence/primary identity. Presence and Gesture
preflight their own batches. Host pointer identity keys presence; latest admitted
mouse/stylus on a surface is primary, touch cannot select Activation, absence is
Default. Pressed stays on its original presented incarnation: capture outside
changes posture; replacement/loss/cancellation/release clear it with exact reason.
No second pressed owner or mouse-button-only inference. Remove superseded hover/
pressed replacement placeholders with their rebind-policy migration.

Canonical hit rows/order/spatial membership share one owner; no caller-supplied
second predecessor. Derived indexes partition completed geometry by binding and
coordinates. Disjoint clips retain rows without spatial membership; non-area
completion denies. Presented targeting uses Portal baselines and accepted Motion,
with receipt/order/currentness checks separate. Query budgets are 1,024 spatial
visits and 256 candidates; exhaustion denies without partial winner or full scan.
Invisible rows reserve capacity. Motion refreshes indexed Portal children while
triggers stay stationary; retirement cannot erase accepted geometry. Charge the
at-most-64 ordinary track scan separately; point queries do not scan tracks.
Preflight cannot materialize full hit tables. Report row/map/spatial maintenance
and retained predecessors separately from cold binding reconstruction.

### Coherent state vectors

Observation close seals `UiAppearanceOwnerSnapshot` from exact-current owner
exports/revisions and admitted observations. `UiAppearanceStateVector` consumes
that single product and only declared axes, carrying application/generation,
surface/theme binding, node/instance/incarnation, relevant presentation, role
revision, owner revisions, and provenance/currentness.

Pure adapters cannot import mutable Focus/Selection/Gesture/Intent/Portal/Motion
internals. Six independent reads/references or generation-stamped mixed snapshots
cannot construct a coherent vector. These are compiler-visible boundaries;
changed axes select through consumed facts, not graph or catalog scans.

### Resolution is a finite partition, never a cascade

Each aspect partitions exactly the Cartesian product of its declared axis classes.
Admission proves exactly one result per cell: no authored reachability exclusions,
owner folklore, overlap, holes, source-order/specificity/last-writer winners, or
runtime defaults. Oversized products deny rather than prune or add a wildcard.

One compiler expands equivalence groups and `otherwise same_as ...` into a total
disjoint complement. Apply cell capacity after expansion and equivalence merge.
Aspects partition independently; canonical axis/class/cell ordering makes Rust/DSL
meaning invariant under declaration order.

### Projection, invalidation, and equivalence

Use the existing consumed-fact relation for owner/slot changes, attached-consumer
indexes for role revisions, and the mount journal for new/retired instances.
Intersect axis dependencies with UI target/incarnation/surface/binding membership;
an axis revision cannot select every consumer of that axis.

Reuse Signal source/aspect/partition/region propagation and existing Query-installed
conditional instances/subscribers. `mark_dirty_with_regions` cannot infer UI
identity. No parallel Signal graph, slot relation, raw Query lane, or presentation
owner. UI retains membership, interaction, and acceptance authority.

New mounts enter without source/theme/state change. Retained appearance capsules
are predecessors; receipt-only/PhysicalOnly entries are not new mounts. Preparation
reads rather than consumes the journal: abandonment retries, unmount cancels, and
reconstruction preserves explicit retirements after semantic rows disappear.

Selection key deltas do not prove collection currentness. `bind_selection_item`
requires current node receipt, same-surface declared owner, and admitted Query
option; the owner consumes that projection or declares its exact SelectionCommit
payload. Query row identity is not an application key; collection text rows are
not independent item authority. Prepare changed collections before selection,
including omitted replacement slots. Validate posture/revision/row/key/world/
binding; preserve surviving bits/revision, deny removed/remapped keys before effects,
and never install abandoned bindings or mutate Selection during preparation.

`UiAppearanceChangeReceipt` distinguishes input evidence, semantic projection,
resolved value, mounted output, equal-output suppression, and denial. Attribution
may change with equal pixels. Revise `theme_token_graph_consumers` in place and
remove `ActiveCurrent::changed_graph_nodes`. Indexes reconstruct from sealed truth;
absence never licenses an ordinary whole-graph fallback.

### Text, deferred icons, motion, and opacity

Adopt existing original UTF-8 ranges through
`ComponentSemanticTextSpanContract::with_appearance_foreground()`. Preserve span
identity, clusters, token-retaining ranges, and independent posture/collection
rows. Adoption affects frozen/executed meaning, not range/style/identity; a role
does not adopt all text and no replacement semantic-span-slot type is added.
Color-only changes paint/damage with zero qualification/shaping/measurement/
rasterization/alpha upload or layout/glyph/caret/selection/hit-geometry drift.

Text damage has two stages:

1. Mounting emits unresolved `UiAppearanceTextDamageRequirement` target/span
   changes plus qualified layout/placement/clip/binding candidates. Candidate
   uniqueness is command-based (instance/slot/collection correlation); foreground
   and damage identity remain target/span. Preserve all commands
   sharing a span; duplicate commands or duplicate span IDs within a candidate
   deny. Current receipt/span membership separately proves foreground adoption.
2. Existing text/atlas admission supplies actual images. Presentation joins complete
   candidates with the exact binding's accepted old coverage, clips/finalizes
   old/new physical damage before submission. Missing/ambiguous matches deny;
   allocation/predicted extents cannot substitute. Empty non-text `damage()`
   cannot erase text work. Mounting cannot bypass host miss admission by eager raster.

Authenticate `CompleteLayout` and exact ordered command-specific runs through the
text-owner callback. Empty `LogicalDamage` selects no glyphs. Scope belongs in
demand v2 identity, not raster keys, and is independent of cost lane/effect authority.
Flags/counts/broad rectangles or reordered/deduplicated runs cannot prove completeness;
complete non-drawable text may have no images. Demand validation and its costs,
atlas admission, and presentation acceptance remain distinct and effect-honest.

Accepted coverage survives cache eviction. Validate candidate atlas/device affinity
at the physical boundary. Pending/rejected/indeterminate/partial-surface work,
rebind, and reconstruction use existing presentation ownership and each binding's
accepted predecessor. Geometry-only changes refresh coverage with color fixed.
Historical coverage is not reusable image authority; headless unresolved requirements
are not physical damage. Intrinsic-color clusters cannot be retinted. Under frozen
BodyDefault v1, prove exclusion by typed/compile admission, not unsupported emoji.
Icons/global-text v2 remain deferred; remove Pulse's `portal_icon_text`/Unicode arrow.

Appearance and Motion supply separate raw `u16` factors (65,535 is one). Runtime
presentation multiplies exactly once: `u32` product / 65,535, saturation and
nearest-even rounding, producing `UiMountedPresentationOpacity`. No float Motion
sampling ingress, early `u8`, host remultiplication, or descendant propagation.
DSL opacity is integer or exact ratio; `from_ratio(n,d)` is the public non-integer
form. `UiMountedAppearanceOpacity` stays raw without composition. Static-paint
sampling uses the identity appearance factor until cutover.

Only physical acceptance installs retained samples. Appearance-only changes consume
them; Motion-only changes refresh exact mechanics without semantic resolution.
Retarget/terminal retirement/rejection cannot erase accepted overrides. Surface/
outline/text/Portal/backdrop share composed opacity; backdrops consume Motion only
when declared. Appearance cannot retarget Motion or mutate its timeline; Motion
cannot mutate semantic appearance. Per-layer multiplication is separate from
source-over, whose effective output cannot become semantic/input authority.

### Pointer affordance is an adjacent semantic mechanic

`UiPointerAffordanceProjection` has Default/Activation, separate from appearance
aspects. Derive it from declared interaction, current Intent operability, primary
presented target/incarnation, never paint/bounds/names/host widgets. Observe Activate
without an event, payload projection, occupancy reservation, or appearance role.
Standing reads share canonical decision/dependency admission and current physical
epoch; do not duplicate catalog checks or create admission candidates.

Confirmation `observe_intent_confirmation(target,time_basis)` shares challenge
matching/validation, yielding eligibility, typed stop, bounded lookup cost. It
consumes no marker or counter. Equality at expiry is valid; next millisecond is
expired. Ambiguous/terminal reads remain effect-free.

Native close uses the read-only native-input-epoch clock installed before activation.
`seal_at_host_time` is an as-of fallback only without it. Pointer timestamps, Signal
ticks, retry clocks cannot establish freshness; untimed empty close denies. Final
idle close handles stationary truth and composes eligible expiry+1ms with existing
deadlines, restoring carried predecessors including ties. No timer thread/readiness
owner/presentation queue; visibility cannot erase another owner's wake.

Compare current pointer meaning with committed rows. Preparation/rejection and
older in-flight settlement cannot acknowledge newer desire. Custom runtimes get
the required callback through their ordinary pending-frame owner; fixed programs
distinguish Program(index) from PointerRefresh without changing authored progress,
capturing authored-frame evidence, or superseding another owner. Close samples
after callbacks; host retry/physical work composes with existing waits.

Snapshots are independent of appearance demand. One surface row holds primary
mouse/stylus; touch supplies no cursor. Refresh full targets on receipt/physical-
epoch succession. Reuse compares surface/binding/pointer/target/family, excluding
receipt/sequence churn; suppression keeps last emitted predecessor. Admission
cannot resurrect historical observations or be consumed by failed output batches.
Observation, selected-membership, and targeting costs remain distinct.

Surface-pointer fragments may replace departing with arriving primary identity;
predecessor/successor each contain at most one correctly attributed pointer mechanic.
Cursor-only work needs no damage/fake node. Requested surfaces emit independently,
preserving omitted predecessors/retries. Native maps the sealed family to OS cursor;
headless records it. Later text-entry/resize/drag/precision/accessibility cursors
enter as typed siblings, not appearance fields.

### Host contract and clean protocol cutover

Hosts receive physical mechanics, never roles/slots/axes/resolver rules:

| Family | Required contract |
| --- | --- |
| Surface | Explicit fill-only/border-only/fill-and-border; radii, composed opacity, bounds/clip/order/receipt/attribution |
| Outline | Outside non-hit ring, composed opacity, expanded visual bounds/damage |
| Text foreground | Original-span adoption, qualified layout identity, composed opacity |
| Backdrop | Independent instance/order receipt/extent/color/opacity/clip/attribution, paint-only |
| Pointer affordance | Exact pointer/surface/target and sealed family |

Portal overlay retains surface/lifecycle/shielding affinity, not dimmer meaning.
Backdrop has an independent row/fate. Deny duplicate IDs, foreign/stale bases, and
mismatch with current sealed overlay order before effects. Radius shares surface
render fate, while semantic aspect facts remain separate.

Component rendering meaning owns concrete paint order independently of static paint.
Role order, execution lanes, plan indices, identity, and host iteration cannot
substitute. Surface then outline is the same-node exception; text keeps declared
order; Portal content stays in its group. Other overlapping equal-order participants
deny. Overlap is positive-area half-open visual-bound intersection after ancestor
clip within one surface/binding/coordinate/Portal partition. Transparency, rounded
pixels, and hollow rings do not relax it. Remove departures before inserting
successors; validate retained peers and preserve the complete index on failure.
Acceleration confers no paint order or interaction authority.

Qualified analytic AA supports integer/fractional scale; host's recorded transform
alone snaps physical geometry. Keep visual bounds/ancestor clips separate; only
outline exceeds own allocation. Normalize damage deterministically, merging only
rectangular unions; retain offset overlaps/gaps, remove containment, deny overflow
rather than widen. Hulls are lookup-only. Headless/reference raster is not native
compositor evidence.

Gate 5 changes these values atomically; earlier gates leave live values unchanged:

| Contract | Current -> successor |
| --- | --- |
| Host protocol floor/current | 6 -> 7; floor equals current |
| Mounted/presentation schemas | 5 -> 6 |
| Text schema | 3 -> 4 |
| Observation / measurement / solicited effect | Remain 7 / 5 / 1 |
| Windows profile | `worth-ui-windows-dx12-v1` -> `worth-ui-windows-dx12-v2` |
| Qualified text | Remains `worth-ui-body-default-v1` |

BodyDefault v1 admits U+0020–U+007E through
`UiUnsupportedBodyDefaultCodePoint::first_in`, denying before effects. Intrinsic-color
adoption is excluded at compile/typed admission; global-text v2 raster proof waits
for that successor. The Windows v2 qualification manifest freezes AA, family
capacities, and one native surface. AP-10 semantic surfaces are not OS windows.

After cutover reject protocol 6, mounted/presentation 5, static-paint schema 1,
and every `UiMountedFilledRectMechanic`. Remove `UiMountedStaticPaintSchemaVersion`,
static-paint modules/translators/exports, `ComponentStaticPaintContract`,
`ComponentStaticPaintOrder`, `ComponentDescriptor::static_paint_contract`,
bootstrap-only `theme_token_dependencies`, string-backed `ThemeColorValue`, Pulse
bootstrap resolver/`portal_icon_text`, direct token publication, and legacy changed-
node selection. Preserve canonical `ComponentSemanticTextSpanContract` with adoption.

Migrate Rust/DSL registrations, fixtures, examples, docs, profile literals/pins
in contract, native/native-platform/runtime and Pulse together. Old source forms
receive source-linked explicit-role migration diagnostics, not implicit lowering.
No aliases/dual emission/fallback. A pre-cutover process may retain its valid prior
generation after invalid edit, but never publish both protocols. Negotiate required
mechanic family/schema support before effects; never drop or approximate unsupported
border/radius/outline/opacity/foreground/cursor.

### Publication, physical failure, and recovery

Use existing observation/scope/identity/plan/publication/presentation/physical-work/
reconciliation owners. Resolution cannot bypass predecessor currentness, capacity,
deadlines, cancellation, support, or multi-surface atomicity. Before-effect denial
preserves exact appearance/theme predecessors. In-flight work retains handles;
uncertain effects stay indeterminate with reconciliation authority, never timeout-
as-success or fictitious rollback. Recovery reconstructs mechanics from mounted/
runtime truth, not pixels or replayed state-owner/Query/service/theme semantics.

### Inspection and developer courtesies

`worth-ui-inspection` owns public contracts; runtime supplies bounded producers.
`why_appearance(node, aspect)` and `why_pointer_affordance(target)` explain role,
theme/binding/slot/alias/source, coherent owner evidence, canonical cell, coverage/
support, semantic/mechanical/suppressed posture, invalidation and costs. Outcomes:
found, expired, unsupported, unavailable, wrong-world; no construction/mutation
or preview-to-live authority.

Extend the existing mounted-preview lane: PreviewOnly consumes an explicitly
admitted preview theme rather than returning None for every slot. Preview cannot
construct live snapshots/vectors/publication. Supply normalized role matrices,
source-linked missing/ambiguous-cell diagnostics (role/aspect/cell/kind/repair),
switch summaries, and focus/activation presets lowered as ordinary declarations.
Rich detail is lazy/budgeted; ordinary facts retain compact provenance, not histories.

### Capacity and lifecycle

`UiAppearanceCapacityProfile` is sealed during preparation.

| Resource | Qualified ordinary maximum |
| --- | --- |
| Appearance roles / backdrop declarations / semantic slots | 4,096 each |
| Registered themes | 32 |
| Slot references per role | 64 |
| Cells per aspect after complement expansion/equivalence merge | 512 |
| Mounted projections / semantic surfaces | 4,096 / 64 |
| Portal-surface rows / backdrop rows | 1,024 each; backdrops also consume projection capacity |
| Prepared/in-flight switches | 4 |
| Compact change/inspection records | 64 |

Each backdrop has one extent, presence, scope, placement, and at most one Motion
basis. The remaining alias, spatial-row, reservation, and lifecycle bounds below
also apply.

Alias resolution admits at most 16 hops and rejects cycles before activation.
Dangling targets are typed registration denials and no alias path may `expect`
its target to exist.
The existing host observation profile continues to bound pointer identities;
pointer presence allocates at most one current target record per admitted live
pointer and only when hover or pointer-affordance consumers exist.

Profiles may choose smaller public bounds. A future qualified profile may
increase a bound without changing semantic identity, but it may not remove a
bound or fall back to unbounded allocation. Capacity is reserved before the
corresponding live state or host work. Saturation yields a typed denial or
backpressure posture, never eviction of current authority, partial role
coverage, a catalog scan, or a coarser global theme update.

Ordinary surface/outline order admission reserves derived storage in the owning
appearance candidate, within the same 4,096-node and 64-surface bounds. Each node
retains at most two spatial rows (surface and outline), so partition and tree
membership are bounded by those rows. The candidate reports its actual retained
structural bytes, including exact-size row payloads; this is separate from a
presented-frame visual lease. Failed admission preserves the prior index and
mechanics, and successful retirement releases their derived entries.

A proposal reserves every portal row, backdrop projection/row, overlay-order
row, and retained command selected by its exact affected scope before mounted
publication. Failure preserves the complete prior portal and overlay snapshots;
it cannot partially publish a presence dependent, reorder an unaffected
participant, or convert backdrop capacity into Portal shielding capacity.

Focused saturation proofs cover the role catalog, slot catalog, decision cells
per aspect, mounted projections, concurrent switches, retained records, alias
hops, independent portal/backdrop rows, placement edges, and stack ordinals.
Each denial names the exceeded bound and preserves exact current authority.

Unused roles, themes, state axes, pointer presence, inspection detail, and
switch capacity create no live owner, per-frame poll, or physical resource.
Application shutdown stops admission, settles or reports in-flight physical
work, releases prepared switches and current pointer records, drops derived
indexes/projections, and proves the complete appearance census zero before the
application close receipt may claim clean closure.

## Public Developer Experience

`worth_ui::facade::appearance` exports typed declaration/theme/switch/receipt/
inspection contracts, not owners/indexes. Keep `register_theme_slot_catalog`,
`register_theme`, `register_appearance_role`, `attach_appearance_role`,
`register_backdrop`, and `bind_initial_theme` through the affine route to `freeze()`.
Identity-bearing parameters use typed IDs; hex/path sugar parses at authoring.
Component and backdrop attachments cannot substitute for each other.

Representative shape, with ordinary registrations omitted:

```rust
let role = UiAppearanceRole::new(UiAppearanceRoleId::new("action.primary")?)?
    .applies_to(ComponentId::new("platform.control.activation")?)
    .cover_foreground("action.primary.foreground")?
    .cover_radius("control.radius.medium")?
    .cover_outline_from_focus_visible(ThemeTokenId::new("focus.ring")?)?;

let scrim = UiBackdropDeclaration::new(UiBackdropIdentity::new("dialog.scrim")?)?
    .with_scope(UiBackdropScope::per_portal_instance(portal))?
    .with_extent(UiBackdropExtentBasis::surface_viewport(surface))?
    .with_presence(UiBackdropPresenceBasis::while_portal_presented(portal))?
    .with_placement(UiOverlayPlacement::immediately_before_portal(portal))?
    .with_motion(UiBackdropMotionBasis::follow_portal_presentation(portal))?
    .with_appearance_role(UiAppearanceRoleId::new("overlay.scrim")?)?;

let origin = session.prepare_theme_switch(
    UiThemeSwitchRequest::new(surface, UiThemeDefinitionId::new("pulse.paper")?)
        .observed_at_tick(now).with_deadline(deadline),
)?;
let prepared = session.prepare_rebind(origin.into_observation_origin())?;
let outcome = UiThemeSwitchOutcome::from(prepared.execute()?);
```

Rust/DSL produce byte-equivalent sealed meaning, including roles with >64 cells.
DSL `foreground use token(action.primary.foreground)` matches the builder above;
`applies_to` validates attachment and `otherwise` expands to a proved complement.
Existing native/host-neutral compile sessions hold complete examples and invalid
counterparts. Public type/progression names remain required. Presets lower as
ordinary declarations, not hidden resolver branches or implicit owner demand.

## Architectural Destination

Keep these ownership boundaries and stable facades. E = retained/revised,
C = introduced by 3.16, R = removed at cutover, S = committed successor.
Private leaf files follow composition laws; no empty successor placeholders.

```text
workspaces/worth-ui/crates/
  worth-ui/src/facade/appearance/                         C: role/backdrop/theme/state/switch/inspection contracts
  worth-ui-dsl/src/
    source/{parse,legality,lower,compile}/                E: source progression
    semantic/{appearance,overlay}/                       C: canonical role/partition/theme and backdrop declarations
    semantic/{expression,module}/                        S: 3.17/3.18
  worth-ui-runtime/src/
    capability/registry/{appearance_role,theme}/          C: frozen admission
    capability/registry/theme_token/                     E: slots, no live binding
    capability/registry/{component,mosaic_region}/        E: attachment/order/seams
    declaration/{appearance,overlay}/                    C: authored meaning
    runtime/appearance/
      state/adapter/                                     C: six pure owner adapters
      state/{vector,coherent_basis}.rs                   C: sealed snapshot consumption
      theme/                                             C: active binding/capability/switch origin
      projection/resolver/                               C: canonical cells/aspects/support
      projection/                                        C: node/backdrop/change receipts
      invalidation/                                      C: consumed-fact selection
      inspection/                                        C: bounded producers only
      trial/                                             S: 3.22 canonical replacement
    runtime/interaction/pointer_presence/                 C: pointer truth/lifecycle
    runtime/overlay_composition/                          C: derived relations/dependencies/order
    runtime/{focus,selection,intent,motion,portal}/        E: authoritative exports
    runtime/observation/                                 E: coherent close
    runtime/presentation_state.rs                        E: binding revision/CAS
    runtime/session/application_state/theme_token_consumers.rs E: canonical slot relation
    facade/entry/mounted_preview/                         E: sole preview lane
    mounting/spatial_index/                              E: geometry-only acceleration
    mounting/session_state/appearance_mount.rs            C: journal inputs
    mounting/projection/frame_storage/                   E: completed geometry/text/affinity
    mounting/projection/appearance/                      C: surface/outline/text/backdrop/pointer lowering
    mounting/projection/semantic_text/                   E: range formatting/adoption
    mounting/presentation/                               E: accepted samples/publication/damage
    mounting/projection/static_paint/                    R
  worth-ui-host-contract/src/
    mounted_projection/appearance/                       C: sealed versioned mechanics
    mounted_projection/semantic_text/                    E: qualified text
    mounted_projection/portal_overlay.rs                 E: Portal affinity
    mounted_frame/                                       E: protocol/command cutover
    mounted_projection/static_paint.rs                   R
  worth-ui-host-headless/src/
    {headless_translation,headless_transcript}/appearance/ C: translation/observation
    *_static_paint*                                      R
  worth-ui-host-native/src/native/
    presentation/appearance/                             C: pipelines/AA/coverage/cursor mechanics
    presentation/{text,retained_draw_list}/               E: glyphs and transaction/replay lifecycle
    event_loop/pointer_cursor.rs                         C: qualified OS effect
  worth-ui-inspection/                                   E: public query/result vocabulary
  worth-ui-retained-order/                               E: mechanic families
  worth-ui-native-platform/                              E: profile/pins
  worth-ui-{certification,test-support}/                 E: existing targets/oracles
workspaces/worth-ui/apps/platform-pulse/                  E: cumulative integration consumer
  src/product_world/visual_composition/
  src/application/presentation/
  app/*.wui
  tests/executable_world/adjudication/
```

Registry/declaration truth flows toward derived resolution, mounting, then host
mechanics. Overlay planning owns no Portal/layout/paint truth or effects. Spatial
indexes confer no order/input authority. Inspection vocabulary stays in inspection;
runtime produces it. Hosts cannot import runtime semantic internals. No style
managers/property/helper bags, live bindings in slot registries, hover inside
appearance/hosts, duplicate preview/slot indexes, or flattened cross-owner modules.
Successors add within these axes, not through another authority.

## Implementation Gates

Contracts and proof dependencies govern order. Later foundations may be retained
while a predecessor is open; they do not close missing integration.

### Gates 0–3 — Contracts and foundations

- **Gate 0:** Freeze aspects/numbers, owner exports/coherent close, partitions,
  slots/themes/switch origins, geometry/text/opacity/order, intended versions,
  removal inventory, and enforcement. Real compilable contracts, no live emission.
- **Gate 1:** Implement declaration/DSL, state/pointer, Portal lifecycle, overlay
  planning, resolver/inspection, unpublished mounting/text/headless, and native
  mechanics. No static-paint bridge, unattached default, always-outside hover stub,
  independent owner reads, or duplicate index/preview.
- **Gate 2:** Merge declaration/state/Portal/order with equivalent lowering,
  sealed exports, bounded triggers, and typed malformed/stale/capacity denials.
- **Gate 3:** Merge resolver/indexes with distinct change postures, independent
  reconstruction, one slot relation, and non-authoritative preview.

### Gate 4: mounting, text, and headless integration

Before further integration, check Query/Signal propagation against UI target and
surface membership using the real multi-neighborhood proof in 4e. Keep that proof
as parts compose; do not build a reactive framework or postpone locality to closure.

Each section closes through its production handoff and acceptance together.
Typed denial cannot substitute for required supported success. Gate 4 permits
unpublished mechanics, headless transport, and reference raster; Gate 5 owns live
cutover and Gate 6 owns native executable/design qualification.

#### Gate 4a — Mounted geometry

**Status: Open.**

**Result and ownership.** Surface and outline mechanics consume exact mounted
allocation, layer/order, ancestor coverage, device-scale qualification, and
visual bounds. Layout, Mosaic, Scroll, and Portal retain their geometry and
lifecycle authority. Appearance consumes their completed facts; it cannot infer
a surface binding from graph ancestry or treat its own allocation as an ancestor
clip. This section supplies the geometry and attribution consumed by 4b–4d.

**Plan.**

1. Settle and admit the structural placement contract that determines exact
   parent/child occurrence rectangles. Region role names, `Fill`, a scalar named
   measurement, and placement eligibility cannot supply an implicit axis,
   sibling distribution, or overlay arrangement. Freeze coordinate interpretation,
   sizing/constraint interaction, overflow, and repeated surface placement before
   implementing the producer. This is a layout-owned prerequisite; appearance
   does not gain layout policy. Bind completed evidence to the exact executed
   parent/child occurrences, current generation, coordinate space, and runtime
   surface. A graph-node key cannot distinguish repeated layout occurrences.
2. Carry exact declared-region-to-mounted-surface binding and current executed
   geometry through mounted projection. Resolve Mosaic, Scroll, and Portal
   ancestor coverage from their owners, including translated Portal children.
   Replace the interim plan-wide Mosaic denial with target-local evidence;
   retain typed denial when the required evidence is actually unavailable.
   Preserve each independent geometry requirement: resolving a Portal binding
   cannot waive an unresolved Scroll or Mosaic contribution. A closed Portal
   child carries explicit suppression so its predecessor can be removed; it
   cannot fall back to ordinary geometry or a fabricated zero allocation.
3. Complete `mounting/projection/appearance/` geometry, clip, surface, and outline
   lowering and its `frame_storage/` inputs. Carry layer/order explicitly;
   remove omitted order from supported mounted paths. First separate surviving
   ordinary paint-order meaning from the bootstrap static-paint contract in
   `capability/registry/component/`, include it in frozen capability and executed
   component meaning, and carry it through mounted node facts. Cut appearance
   surface/outline transport over to the completed order; a bare reference into
   the existing execution-lane table cannot close this handoff. Preserve canonical seam
   ownership instead of painting a shared edge from both neighbors.
4. Use the same completed geometry in unpublished initial, delta, removal, and
   reconstruction work and headless translation. Derive damage from old/new
   visual coverage, including outline expansion and the qualified AA fringe;
   record ancestor clipping independently of own shape/allocation.

**Acceptance.**

- Real mounted cases cover ungoverned, Mosaic, Scroll, and Portal-child geometry,
  including two surfaces with copies of one declaration. Every emitted bound,
  clip, layer, and attribution field agrees with independently stated expected
  geometry. An unrelated nested Mosaic mount cannot deny an ordinary target.
- The authored geometry world includes unequal sibling rectangles at nonzero
  origins, repeated region and surface declarations under distinct parent
  occurrences, and separate runtime surfaces. Resize one parent through the
  real allocation path: only its dependent occurrence geometry and damage may
  change. Independent expected coordinates must expose a graph-keyed overwrite,
  root-viewport substitution, invented equal division, or reconstruction of a
  Scroll viewport from maximum travel. Replacing arrangement evidence with any
  of those proxies must fail the proof. Missing arrangement denies before
  publication and preserves predecessors, but that denial cannot close a
  supported geometry case.
- At device scales 1.0, 1.25, 1.5, and 2.0, border/radius/outline cases preserve
  seam ownership, half-open coverage, and exact visual damage. Unclipped outlines
  extend beyond allocation; disjoint ancestor coverage suppresses visible work;
  visible -> clipped -> visible succession restores the correct mechanic.
- Missing, stale, foreign-surface, or unresolved geometry denies before output
  publication and preserves the predecessor. Geometry repair does not alter hit,
  shielding, or focus authority; rounded paint retains the specified hit shape.
  Include a Portal child with an independently required ancestor clip: supplying
  the Portal alone must still deny missing ancestry, while supplying both yields
  their exact translated intersection. Closing and reopening the Portal removes
  and restores the child's appearance without clipping an outline to its own
  allocation.
- Order evidence includes an outline-only node, nontrivial declared ranks that
  disagree with plan/registration order, and an order-only change between
  overlapping nodes. Exact output and independent overlap samples must change
  together while unrelated surfaces retain their predecessors. Removing static
  paint cannot remove appearance's order authority. Ambiguous overlapping order
  denies before publication; neither receipt identity nor fragment iteration
  may silently decide the winner.
- Ordinary owner-state changes consume carried geometry without an ancestor or
  whole-plan walk. Existing structural counters expose selected-instance/index
  work separately from cold geometry reconstruction. Headless/reference-raster
  checks would detect allocation-clipped outlines or a missing/duplicate seam.

#### Gate 4b — Text foreground

**Status: Open.**

**Result and ownership.** Appearance changes paint on explicitly adopted original
UTF-8 ranges while the existing semantic-text owner retains span identity and
qualified layout. This section consumes 4a geometry and supplies exact text
mechanics for 4c. `ComponentSemanticTextSpanContract` remains the authoring
boundary; no semantic-span-slot replacement or broader font profile is introduced.

**Plan.**

1. Carry authored foreground adoption through the existing text declaration,
   qualified semantic-text projection, and mounted original-range attribution.
   Connect real text input to `mounting/projection/appearance/text_foreground.rs`;
   replace the empty foreground collection on supported production paths.
2. Lower resolved RGBA only into adopted paint spans. Preserve token-retaining
   spans, cluster boundaries, layout handles, and original text. Carry 4a clip,
   layer, and damage facts into text output and headless transport.
3. Route color-only changes through existing paint-only text updates and retained
   raster/layout ownership. Exercise initial, changed, unchanged, removal, and
   reconstruction paths without introducing an appearance-owned text cache.
4. Connect text-content and adoption changes to exact mounted invalidation even
   when no appearance owner axis changes. Join adopted ranges to current qualified
   candidates, remove foreground output when text becomes empty or fully clipped,
   and restore it from current text authority when visible again. Preserve the
   distinction between semantic text changes that require layout work and
   paint-only changes that do not.
5. Route allocation, placement, ancestor-clip, layer/order, and surface-binding
   changes into exact mounted foreground refresh even when owner state, theme,
   text content, and resolved RGBA remain unchanged. Use the existing consumed
   facts and mounted dependency machinery. Retain the geometry comparison basis
   with the accepted appearance facts through identity-only advancement,
   abandonment, and reconstruction; missing evidence cannot justify suppression.
   Physical-input-only refresh reuses retained semantic appearance, while mixed
   semantic changes retain their normal resolution requirements.
6. Complete the staged damage handoff described in the text decision lock:
   keep semantic target/span requirements in mounting and prepare actual image
   geometry through existing text/atlas admission. Finalize exact physical
   damage before submission and include it in the ordered raster replay plan,
   including when ordinary non-appearance commands have no changes. Remove every
   allocation/predicted-extent fallback; unresolved requirements cannot satisfy
   the physical acceptance boundary.
7. Integrate accepted per-command coverage with the existing presentation owner
   across initial presentation, replacement, removal, and reconstruction. Bind
   each candidate to its exact target, surface binding, attempt, qualified text,
   and geometry. Carry its inverse through the existing delta and pending
   settlement lifecycle; rejection preserves the accepted predecessor and an
   omitted surface cannot be acknowledged. Signal completion alone cannot grant
   presentation acceptance or establish a second coverage owner.
8. Verify the complete handoff using the acceptance cases below, including mixed
   text and non-text mechanics, geometry-only successors, rejection/retry, and
   partial-surface settlement. Observe actual replay coverage, text/atlas work,
   and retained resources independently. Charge candidate retention, comparison,
   damage construction, and rollback work in their owning lanes; successful
   finalization counters alone do not establish the full transaction cost.

**Acceptance.**

- A real mounted text consumer with adopted and non-adopted ranges changes only
  the expected RGBA/span output. Exact original ranges, span identities, text,
  qualified layout identity, glyph positions, selection/caret rectangles, and hit
  geometry remain stable during a color-only change.
- With theme, owner state, and resolved RGBA held fixed, change text placement,
  qualified layout, partial ancestor clipping, and paint order through their
  owning production paths. Assert an exact affected target/span replacement
  even when both old and new text remain visible. Preserve every qualified
  candidate belonging to that span; copies of the same span on another mounted
  target or surface remain unchanged. Repeating identical geometry suppresses
  work, and reconstruction reproduces the current geometry. Comparing color
  alone, selecting the first matching candidate, or keying retention by span
  alone must fail this proof.
- Physical damage finalization joins those semantic requirements to actual
  admitted glyph-image geometry and the exact binding's accepted predecessor
  coverage. Insert, replace, and remove cover independently expected clipped
  images, including disjoint images within one span. Cache eviction cannot erase
  accepted coverage; rejected preparation cannot replace it. Unresolved span
  requirements in a headless transcript and allocation-sized rectangles do not
  satisfy this acceptance requirement.
- Independent work observations show zero new qualification, shaping,
  measurement, glyph rasterization, or alpha-atlas upload for color-only output.
  Exact paint damage changes; unrelated text and equal-output consumers do not
  acquire work. A changed paint receipt cannot masquerade as a new layout.
- Invalid ranges, stale layout/span attribution, and unsupported foreground
  adoption deny typed before effects. Frozen BodyDefault v1 coverage remains
  enforced; intrinsic-color exclusion is proved at admission/contract boundaries,
  not by pretending an unsupported emoji or global-text profile was rendered.
- Headless output preserves the same text/paint identities through delta and
  reconstruction. The oracle checks identities and work independently; it must
  fail if a text node is reshaped or every span is retinted to pass a color test.
- With owner state and theme held fixed, update text, clear and restore it, and
  change adoption through the real declaration/content paths. Assert exact
  affected mounted instances and current qualified ranges; no stale span may
  survive and unrelated neighborhoods retain their predecessors. A simultaneous
  focus change cannot supply the invalidation that this case is meant to prove.
- Abandon a prepared text change and reject another before effects; both remain
  pending for a successful retry. Publishing only one surface cannot acknowledge
  an omitted mounted copy. When that copy returns, it uses its own accepted
  presentation predecessor, with complete catch-up charged as reconstruction.
  A stale acknowledgment cannot consume a newer text revision.
- Drive visible -> fully ancestor-clipped -> visible text, including a translated
  Portal child. Suppression removes the old foreground, restoration uses current
  span attribution, and damage matches independently expected old/new visible
  span coverage. Stable layout identity alone does not prove zero raster or
  atlas work; observe those owners separately for the paint-only transitions.

#### Gate 4c — Motion composition

**Status: Open.**

**Result and ownership.** Appearance's raw opacity and Motion's exact accepted
presentation sample remain distinct inputs. Runtime presentation composes them
once into `UiMountedPresentationOpacity`. Motion owns tracks, timing, retargeting,
and terminal state; UI presentation owns physical acceptance. This section
consumes 4a geometry and 4b text identity and supplies the composition contract
used by authored backdrop Motion in 4d.

**Plan.**

1. Finish accepted-sample retention beside the exact presentation command owner
   in `mounting/presentation/`, including current frame/binding/epoch admission,
   pre-effect reservation, unchanged-command inheritance, and replacement,
   reconstruction, rebind, and shutdown retirement. Retargeting or sampler
   retirement cannot erase a physically retained override. Prepared or rejected
   samples cannot become accepted evidence.
2. Bind the exact accepted raw sample to mounted surface, outline, and text
   lowering. Replace the production `motion_opacity: None` placeholder for bound
   consumers. Distinguish no declared/current sample from stale, unavailable,
   foreign, or ambiguous evidence; do not infer a node factor from the first of
   several conflicting command/Portal samples.
3. Complete both directions: appearance-only changes consume retained accepted
   Motion, and Motion-only changes refresh exact affected unpublished mechanics
   from retained semantic appearance without rerunning the resolver. Preserve
   Portal target/group attribution and the stationary trigger's separate fate.
4. Keep one runtime integer composition operation: `u32` product divided by
   65,535, saturation and nearest-even rounding, with full `u16` precision until
   host output. Hosts consume the composed value without another multiplication.

**Acceptance.**

- Real declared Motion enters the mounted producer and affects surface, outline,
  and text field-for-field. Arithmetic fixtures alone do not close this section.
  Endpoint and non-8-bit cases include `40000 * 32768 -> 20000` and
  `40000 * 8192 -> 5000`; an independent integer oracle detects double
  composition, requantization, and semantic/physical factor confusion.
- Appearance-only, Motion-only, and simultaneous changes preserve exact target
  scope. Motion-only work records zero semantic appearance resolutions and no
  text-layout/atlas rebuilding. Geometry-free opacity still produces correct
  command damage; appearance opacity does not recurse into descendants.
- Rejected, pending, cancelled, and indeterminate presentations preserve or deny
  physical truth according to the existing lifecycle. Prove semantic candidate
  in flight -> Motion accepted -> unchanged command candidate settled, preserving
  the accepted override. For forbidden overlap, prove the real admission denial
  before mutation; do not mint an impossible newer frame to simulate adversity.
- Retarget, terminal-track retirement, selective same-instance command
  replacement, and rebind cannot reuse stale or clear unrelated evidence. Late
  accepted work validates all current owners before any retained write. Budget
  exhaustion denies before effects, accepted samples fit their reservation, and
  shutdown releases all retained sample/command resources.

#### Gate 4d — Backdrop and Portal

**Status: Open.**

**Result and ownership.** Authored backdrop instances and Portal surfaces lower
from their distinct owners into the issued total overlay order. This section
consumes 4a geometry, 4b Portal-child text, and 4c opacity composition. Portal
retains membership, lifecycle, shielding, dismissal, and focus authority;
backdrops remain paint-only authored participants.

**Plan.**

1. Connect admitted backdrop scope, extent, presence, placement, role/theme, and
   optional Motion through `runtime/overlay_composition/`, mounted backdrop/
   Portal projection, and the appearance lowering families. Materialize exact
   surface-singleton or per-Portal-incarnation identities without fake node
   receipts. Remove absent Portal identity from supported Portal surface inputs.
2. Preserve the sealed overlay snapshot's bottom-to-top order through unpublished
   output and headless translation. A Portal anchor includes its surface and
   ordinary content subtree; a backdrop cannot split that group. Use 4c once-only
   composition only when the backdrop explicitly declares a Motion basis.
3. Complete indexed lifecycle deltas for open/close, parent close, exit retention,
   resize, rebind, and reincarnation. Atomically retain or replace all affected
   overlay dependents and compute old/new damage and the intersecting raster
   suffix without scanning the UI graph or resolving appearance during recovery.

**Acceptance.**

- An authored mounted world covers nested and same-depth sibling Portals,
  repeated per-Portal instances, viewport and Mosaic-region extents, before/after
  placement, modal-without-backdrop, non-modal-with-backdrop, and `Always` without
  a Portal. Exact identities, extents, presence, order, and attribution agree with
  declarations and sealed owners; portal kind supplies no visual default.
- An independent spatial oracle folds colored and transparent layers in issued
  order using the specified linear-light premultiplied source-over. Compare
  overlap and uncovered regions at zero, one, and two backdrop layers. The
  equal-black `1 - product(1 - alpha_i)` result is an additional cross-check, not
  a flattened runtime alpha or a call to the production compositor.
- Close, parent close, retained exit, resize, rebind, and reincarnation leave no
  orphan rows or cross-instance bindings. Missing/cyclic/ambiguous anchors and
  stale or foreign extent/presence/Motion receipts deny before publication;
  rejection and partial failure preserve or reconcile the complete predecessor.
- Zero-opacity or absent backdrop paint changes no hit, shield, focus, dismissal,
  or cursor authority. Full-viewport changes report full clipped viewport damage;
  region changes report region damage. Indexed dependent construction and
  intersecting ordered raster replay have separate, observable costs.

#### Gate 4e — Integrated closure

**Status: Open.**

**Result and ownership.** The combined production mounting path satisfies 4a–4d
at one coherent boundary. This is Gate 4's acceptance decision, not a sixth
implementation lane or a substitute for the later live cutover.

**Plan.**

1. Extend the existing shared authored certification world to exercise surface,
   outline, original-range text foreground, Motion, independent pointer output,
   backdrops, and Portal content together on multiple semantic surfaces. Reach
   unpublished mechanics through ordinary declaration, owner, and mounting
   entry points; reuse the same world for headless transport/reference raster.
2. Drive initial -> appearance-only -> Motion-only -> mixed change -> unchanged
   -> removal -> reconstruction. Include rejection/retry and legal in-flight
   ordering at the boundaries changed by 4a–4d. Compare exact predecessors,
   successor identities, damage, retained resources, and output equivalence.
3. Run focused owner and affected integration/host-contract/headless/native
   contract tests, the relevant compile-time denials, formatting, scoped limits,
   boundary/context checks, and removal/version guards. Review the final causal
   diff independently and reconcile the status in this spec and the roadmap.

**Acceptance.**

- All 4a–4d acceptance requirements pass on the final shared implementation.
  Every new mechanic family is field-for-field attributable to current mounted
  facts, and the combined reference raster agrees with independently stated
  spatial/color expectations. A collection of isolated mechanic constructors,
  screenshots, or passing test counts cannot establish this result.
- Multiple neighborhoods and surfaces preserve exact consumer scope, including
  copies of one declaration. Existing Query/Signal and consumed-fact boundaries
  remain canonical. Equal-output and unchanged cases prove the specified zero
  resolution/layout/raster work where applicable; cold reconstruction is reported
  separately. Pointer and interaction authority remain independent of paint.
  Use at least two mounted neighborhoods on one surface and a copy on another
  surface, all consuming the changed axis. Change an owner in only one
  neighborhood and assert exact selected identities and unchanged unrelated
  predecessors, then move the owned target between neighborhoods and verify
  both departure and arrival. This must expose axis-wide selection even when
  equal resolved values would hide it in final pixels. Signal transports declared
  invalidation through its existing contracts; it cannot decide UI target
  membership, presentation acceptance, or interaction state.
- Real-path failure and cleanup evidence preserves predecessors before effects,
  denies stale authority, and reconciles uncertain physical effects. No remaining
  placeholder, conservative denial of a supported case, or fixture-only adapter
  stands in for required geometry, foreground, Motion, or overlay integration.
- Required checks include `cargo run --manifest-path tools/boundary-check/Cargo.toml
  -- --root .`, `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`,
  `scripts/ci/check_workspace_rust_line_caps.sh dirty`, and
  `python scripts/quality/scrutinize_rust_functions.py --dirty .`, together with
  affected workspace formatting/build/test commands. Review advisory findings by
  causal scope under the coding guidelines; do not absorb unrelated worktree debt.
- Appearance still emits zero live host commands, the static-paint publisher and
  live version constants remain in place, and the Gate 5 atomic cutover is the
  explicit next handoff. Gate 6 retains full live switching/recovery, native
  executable pixels and design adjudication, AP-07/AP-10 closure qualification,
  Pulse migration completion, and milestone-wide documentation/shutdown closure.
  Passing 4e closes Gate 4 only.

### Gate 5: one atomic live cutover

No branch may merge between Gate 4 and Gate 5. One commit across
`worth-ui-host-contract`, `worth-ui-host-headless`, `worth-ui-host-native`,
`worth-ui-runtime`, `worth-ui-native-platform`, certification/retained-order,
and Platform Pulse:

- advances protocol floor/current 6 -> 7, mounted/presentation 5 -> 6, text
  schema 3 -> 4, and Windows profile v1 -> v2;
- makes the first live surface/outline/text-foreground/backdrop/pointer-
  affordance emission;
- removes every legacy static-paint command, symbol, translator, fixture,
  facade export, component field, and Pulse bootstrap path; and
- wires one host-neutral certification world and the mechanical Platform Pulse
  migration to roles/themes in the same spine.

The first new host emission and last old host emission are the same cutover.
There is no unpublished dual-runtime window. Protocol/version and deletion
manifests must be green in that commit.

### Gate 6: integration and parallel closeout

First finish live switching, source-edit rebind, exact predecessor/CAS checks,
multi-surface scope, cancellation, in-flight/indeterminate settlement,
reconstruction, and Pulse migration with zero `ComponentStaticPaintContract`
or Unicode icon substitute. Pulse migration is an integration consumer, not a
late documentation task.

Only then parallelize the independent closeout lanes: public-DX compile
fixtures, `AP-07`, `AP-10` plus saturation/CI qualification, `AP-01` stacked-
modal visual polish and adjudication, enforcement/deletion verification, and
continuing documentation. The milestone closes only after native/headless
parity, contrast/containment/state/locality proofs, exact-zero shutdown, and the
independent design review all pass.

The real Pulse must be aesthetically excellent and mechanically honest. A
green test suite does not waive the design judgment; a beautiful screenshot
does not waive the contracts.

## Verification and Documentation Deliverables

Use focused owner tests, affected integration/headless/native contracts, Rust/DSL
equivalence and valuable compiler denials in existing targets. Cover partitions,
kinds, forged/wrong-world authority, preview promotion, support, aliases/revisions,
geometry, exact damage, text reuse, lifecycle, and saturation at actual boundaries.

Required boundary-relevant implementation checks:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
python scripts/quality/scrutinize_rust_functions.py --dirty .
```

Also run affected formatting/build/tests, protocol/deletion manifests, docs/link
checks, feature/profile matrix, and required closure-stress. Enforcement includes
Worth UI apps, concrete authority/private constructors, exhaustive families/census,
dependency direction, and no forbidden fallback/legacy lane. Preserve exact-count
legacy deletion, intended/live version, documentation/link, and feature-matrix
gates. Dirty line caps do not prove broader CI coverage; review advisory findings
by causal scope. Source scans support, not replace, compiler/runtime evidence.
No new executable/test target, progress ledger, or test-count quota is required.

The implementation must revise these continuing documents rather than create
milestone residue:

- `workspaces/worth-ui/docs/application-lifecycle.md` for the cumulative Pulse
  appearance/theme journey, external event fields, design evidence, switch
  behavior, and cleanup;
- `workspaces/worth-ui/docs/authored-composition.md` for Rust/DSL role and theme
  lowering, invalid edit preservation, and source provenance;
- `workspaces/worth-ui/docs/hot-rebind.md` for appearance/theme observation
  families, indexes, evidence-only succession, affected-scope cost, switch
  failure, and reconstruction boundaries;
- `workspaces/worth-ui/docs/interaction-and-intents.md` for hover/pressed/
  operability adapters and the explicit non-authority of visual state;
- `workspaces/worth-ui/docs/runtime-services.md` for exported Focus/Selection/
  Motion state consumed by appearance, Portal-issued total stack order,
  backdrop presence as a non-authoritative Portal consumer, and the prohibition
  on callbacks;
- `workspaces/worth-ui/docs/runtime-subsystems.md` for the new coherent owner
  snapshot, standing operability/validation facts, existing consumed-fact index
  ownership, and owner-table changes. Its existing “`BodyDefault` appearance
  role” wording becomes `UiSemanticTextProfile::BodyDefault`; no 3.16 role may
  be named `BodyDefault`;
- `workspaces/worth-ui/docs/text-platform.md` for role-driven foreground,
  paint-span preservation, alpha-layout reuse, intrinsic-color exclusion, and
  opacity composition;
- `workspaces/worth-ui/docs/native-host-platform.md` for protocol floor/current
  7, mounted/presentation 6, text schema 4, surface/outline/cursor mechanics,
  ordered authored-backdrop source-over, anti-aliasing, damage, reconstruction,
  and v2 qualification;
- `workspaces/worth-ui/docs/inspection.md` for `why_appearance`, theme-switch
  summaries, relevance/expiry, and non-authority;
- `workspaces/worth-ui/docs/visual-inspection.md` for appearance attribution,
  rounded/outline visual bounds, state/theme comparison, stacked-modal visual
  adjudication, and the continued secondary status of pixels; and
- `workspaces/worth-ui/AI_README.md` for the stable appearance/theme/state
  mental model, exact owners, public facade, current support, and successor
  boundaries.

Create one continuing developer-facing document:

- `workspaces/worth-ui/docs/appearance-and-themes.md`, for app authors. It must
  explain roles versus components, slot catalogs versus theme definitions,
  explicit surface bindings, aspect coverage, finite state partitions, Rust
  and DSL examples, independent backdrop declaration/presence/placement and
  accumulation, live switching,
  typed denials, text behavior, inspection, performance, current limits, and
  anti-patterns. Its examples must be compiled
  or semantically lowered against the real public facade in existing test
  matrices.

The docs state that `worth-cert-ui` is not a workspace crate or certification
owner, that mounted preview is the one preview lane, that
`worth-ui-global-text-v2` remains staged/out of scope, and that icons await the
Milestone 9 host mechanic rather than being represented by text glyphs.

Revise `_docs/worth-ui/worth_ui_roadmap.md` to link this governing spec and keep
the 3.17, 3.19, 3.20, 3.22, and Milestone 9 handoffs accurate. Remove or correct
any text that still presents private static paint as the current appearance
contract after cutover.

No phase closeout, duplicate architecture summary, test-count ledger, or
speculative theme cookbook is a deliverable.

## Acceptance and Successor Handoff

The milestone closes when its locked contracts, Gates 4–6, decisive proofs,
required checks, continuing docs, native/headless parity, and exact shutdown census
are verified on the final implementation. No static-paint or parallel appearance/
theme/preview/index/presentation authority survives.

- **3.17/3.18:** pure typed, aspect-tracked expressions/modules; no hidden resolver.
- **3.19/3.20:** richer diagnostic projections and mounted visual invariants;
  explanations and pixels remain evidence.
- **3.22:** affine canonical style replacement/source edits, never inspector cascade.
- **9:** registered theme/density/component families and admitted icon mechanics.
- **13:** accessibility/state/contrast coverage without color-as-semantic-truth.
- **15:** typed contribution owner/generation, admission/unload through registries,
  not global overrides.

Successors must not move the facade, replace the resolver, or teach hosts
role/theme/state meaning.
