# Milestone 3.16: Appearance, Theme, and Visual State Projection

## Status and Placement

Status: implementation in progress. Gates 0 and 1 are complete. Gate 1's
approved integration SHA is `e1d61bce2aeaa059777028f861e797f09c0ea785`.
Gate 2 is next and in progress, but is not complete.

Milestone 3.16 follows the closed Milestone 3.15 production-runtime-services
slice and precedes Milestone 3.17 DSL expressions and semantic evaluation. It
closes the visual-meaning boundary that Milestone 9's professional design
system, Milestone 3.19 diagnostics, Milestone 3.20 visual invariants, and
Milestone 3.22 live style inspection will consume.

This is not a widget-library milestone and not a renderer-polish pass. It makes
appearance a first-class runtime-declarable semantic lane so a serious product
can be beautiful without making beauty adapter-local, component-folklore, or
an uninspectable pile of overrides.

The implementation remains subject to:

- every repository engineering law in `_docs/coding_guidelines/`;
- [Worth UI Vision](./worth-ui-vision.md);
- [Worth UI DSL Vision](./worth-ui-dsl-vision.md);
- [Milestone 3.15](./milestone-3.15.md);
- [Worth UI AI orientation](../../workspaces/worth-ui/AI_README.md);
- [Query AI orientation](../../workspaces/worth-query/crates/worth-query/docs/AI_README.md);
- the current authored-composition, hot-rebind, interaction, runtime-service,
  text, native-host, inspection, and visual-inspection contracts; and
- the existing atomic application/mounted publication and physical-settlement
  boundaries.

## Goal and Central Claim

The central claim is:

> A Worth UI node's visible treatment is a deterministic, inspectable,
> runtime-owned projection of an explicitly attached appearance role, an
> admitted theme capability, and one coherent vector of owner-issued semantic
> state. The host performs sealed mechanics for that projection but never
> selects UI meaning, and a visual-only change reaches only the consumers and
> mechanics whose declared inputs changed.

The sibling overlay claim is equally explicit: an authored backdrop owns its
identity, extent, presence, relative placement, optional Motion basis, and
appearance role. Portal kind creates none of those meanings. A derived overlay
planner combines declared relations with sealed current Portal/geometry inputs;
the host only presents the resulting ordered mechanics.

The complete semantic paths are:

```text
explicit node-to-role attachment
+ role aspect contract and disjoint state decision tables
+ one surface-bound admitted theme definition
+ coherent owner-issued state-axis vector
-> UiAppearanceProjection
-> mounted appearance facts and bounded physical mechanics
-> existing atomic publication and host settlement

explicit backdrop declaration
+ admitted extent, presence, placement, and optional Motion bases
+ explicitly attached background/opacity appearance role
+ sealed current Portal and geometry exports when referenced
-> UiBackdropAppearanceProjection + UiOverlayStackSnapshot
-> mounted backdrop facts and bounded ordered mechanics
-> existing atomic publication and host settlement
```

The milestone succeeds only if that path is simultaneously:

- truthful: appearance cannot invent focus, selection, operability,
  validation, hover, pressed, or Query posture;
- deterministic: no selectors, specificity, cascade, source-order winner,
  last-write-wins override, ambient theme lookup, or adapter default decides
  the result;
- local: state and theme changes use declared reverse indexes and do not
  rediscover unrelated graph, layout, text-shaping, Query-binding, or service
  neighborhoods;
- rebind-safe: invalid or stale appearance changes preserve the exact current
  predecessor, while a complete successor publishes through the existing
  governed path;
- host-neutral: headless and native hosts receive the same runtime-decided
  meaning and differ only in physical execution;
- beautiful: the cumulative Platform Pulse must pass a separate human product
  and design judgment against the contemporary Linear-or-Notion quality bar,
  not merely satisfy geometric assertions; and
- future-bearing: Milestones 3.17, 3.19, 3.20, 3.22, 9, 13, and 15 can add
  expressions, diagnostics, visual rules, style trials, component libraries,
  accessibility semantics, and plugin themes without moving the 3.16
  authority boundary or inventing a second appearance engine.

## Inherited Boundary

Milestone 3.16 may trust these predecessor guarantees:

- file-authored and Rust-authored composition converge on one sealed semantic
  package with stable identity and source provenance;
- the runtime graph, plan, mounting, presentation, and host layers have
  distinct authority and lifecycle;
- layout allocation, presentation-sampled geometry, hit testing, clipping,
  visible-region evidence, and mounted identity already have typed contracts;
- semantic text carries one qualified layout through measurement,
  accessibility geometry, headless presentation, native glyph rendering, and
  reconstruction;
- text foreground is paint-only when font collection lineage,
  layout-affecting style, and layer order are unchanged;
- interaction targeting, pointer capture, gesture continuity, semantic focus,
  focus-visible modality, selection, motion, scroll, portal, and command
  routing are runtime-owned;
- operability preserves support, mutability, readiness, occupancy, policy,
  affinity, and confirmation rather than flattening them into an enabled
  boolean;
- hot rebind admits owner-specific observations, resolves declared consumers,
  compiles an immutable plan, and publishes one successor atomically;
- mounted presentation is delta-based and the host retains mechanics by
  runtime-issued identity;
- inspection and visual snapshots are read-only, bounded projections whose
  receipts cannot be promoted into authority;
- the native host is the sole native-display platform and already owns
  physical scheduling, recovery, capture, and settlement; and
- Milestone 3.15's service state and Platform Pulse facts are real runtime
  products that appearance may consume but may not replace.

The inherited implementation is intentionally narrower than the destination:

- `ThemeTokenValue` currently admits only color;
- token descriptors currently carry bootstrap values rather than separating a
  stable semantic slot catalog from a selectable theme definition;
- component static paint resolves one token directly to one filled rectangle;
- the host contract exposes one `UiMountedFilledRectMechanic` family;
- qualified text already supports paint-only foreground replacement, but that
  replacement is not yet unified with component appearance roles;
- pointer motion maintains capture continuity but has no runtime hover owner;
- pressed posture exists only inside gesture progression; and
- no canonical appearance role, state vector, theme binding, coverage proof,
  resolver, invalidation index, or appearance explanation exists.

Those bootstrap paths are evidence of current reality, not parallel surfaces
that may survive the cutover.

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

The design is governed by three courtrooms. Native product quality, hostile
state/currentness, and scale/locality are different claims; none may stand in
for another, and none creates a new Cargo test target or executable.

### `AP-01`: Native cumulative Platform Pulse

Extend the existing `worth-ui-platform-pulse` process and its real
960-by-600/1120-by-700 native journey. Keep the same source installation,
Query installation, application generation, native window, service owners,
product facts, external event stream, runner restrictions, node bound, capture
bound, and at-most-45-second journey. This is one cumulative product, not an
appearance-only fixture.

The source explicitly attaches appearance roles to the existing masthead,
evidence rail, primary service stage, status band, real controls, text, portal
content, and the required modal stack. The result must feel
like one authored product with a composed Mosaic of materially distinct but
harmonious regions, not one flat surface, a generic enterprise dashboard, a
Microsoft/Fluent imitation, or a collection of disconnected cards.

The Pulse must visibly and truthfully demonstrate:

1. an operable primary action with normal, hovered, pressed, and
   focus-visible treatment;
2. a second real control whose inoperable or validation-bearing posture comes
   from its existing owner-issued fact and is visibly distinct without
   pretending to authorize or deny Query work;
3. a selected or focused region whose treatment follows the actual 3.15
   Selection or Focus owner;
4. a real two-deep `modal_dialog` stack plus two separately authored viewport
   backdrops. Each backdrop explicitly follows one portal's presented lifecycle
   and is placed immediately before that portal; the second therefore dims the
   first dialog and application beneath it. Both dialogs retain conventional
   title/content/action hierarchy and real Cancel/primary placement. One modal
   shown twice, an adapter-created dimmer, or a floating unshielded panel does
   not satisfy this item;
5. an admitted switch from the initial Pulse theme to one second application
   theme and back, using the same role declarations; and
6. exact mounted appearance explanations for at least one background,
   foreground, border, radius, opacity, and outline result.

Hover on an operable activation target darkens the control's background by an
authored semantic token; this is an `AP-01` Pulse visual-contract rule, not a
platform resolver law. The hovered token's relative luminance is at least eight
percent below the normal token while preserving its required foreground
contrast. Interior activation-target control points use channel tolerance 2,
and the authored normal/hover per-channel delta exceeds twice that tolerance.
It is not simulated by the runner or inferred by the adapter.
Pressed treatment is darker again or otherwise independently distinguishable
under the checked visual contract. Focus-visible uses a real outline and
appears only from the admitted focus-visible modality. A
pointer-capable interactive target exposes the runtime-declared activation
cursor affordance; decorative and inoperable regions do not acquire it from
their color or bounds.

The visual composition keeps the 24-pixel outer gutter, eight-point rhythm,
Mosaic region allocation, at-least-32-by-32 interactive targets, 4.5:1 body
text contrast, 3:1 purposeful non-text boundary contrast, and the inherited
text-containment safety insets. Radius and borders may not reduce text fit,
create clipped wrapping, disturb allocation, or make hit regions disagree
with the mounted contract. The Source Signal and Query Posture text still
exercise their truthful wrapped second lines at both native sizes.

The theme-switch turn changes at least one token that has multiple consumers,
one token with exactly one consumer, one declared token whose resolved value is
equal in both themes, and one unused token. The visible and recorded outcomes
must show:

- only consumers of semantically changed slots enter appearance resolution;
- equal resolved output produces no physical mechanic change;
- text color changes reuse qualified layout and glyph geometry;
- unused tokens produce no mounted work;
- structure, layout, hit-test membership, Query binding, service state, text
  shaping, and unrelated presentation commands remain unchanged; and
- the successor becomes current only after the existing presentation boundary
  settles it.

The journey crosses native pointer move, press, release, keyboard focus,
window-focus loss/regain, two-deep portal open/close, resize, and hot rebind.
During rebind the currently hovered target moves and one focused or selected target is
removed or reincarnated. Hover and pressed state must follow current
presentation and capture law, stale incarnations must open no door, and the
last admitted appearance remains visible after a deliberately invalid role or
theme edit.

The runner uses only real OS input, watched-file edits, native resize/focus,
pixel capture, and the public Pulse observation stream. It may not call the
appearance resolver, theme switch progression, state owners, Query, runtime,
mounting, or host facades; inject an appearance receipt; set a native cursor;
or publish expected colors.

Independent evidence consists of:

- mounted structure, appearance facts, state-vector bases, theme receipts,
  command deltas, damage, and resource census from the product;
- external compositor-visible client pixels from the executable child;
- the existing source-affine native visual snapshot path;
- an independent checked-in visual contract naming regions, semantic role and
  slot identities, expected state transitions, contrast pairs, containment
  bounds, control points, and permitted masks; and
- a recorded human product/design review of the complete 960-by-600 and
  1120-by-700 worlds, including zero-, one-, and two-modal depth and the
  focus-visible state. The review must judge the full composition, the dialog
  hierarchy, backdrop restraint, perceived elevation, and action placement;
  exact alpha or contrast checks cannot approve those qualities.

The structural and pixel proof cannot certify subjective quality, and the
design review cannot certify authority or locality. Both are mandatory.

At each native size the runner records the sequence `0 -> 1 -> 2 -> 1 -> 0`
modal layers from the real child. The checked visual contract names stable
control points on unobscured application content, on the exposed part of the
first dialog, and inside the second dialog. Equal dark backdrop values must
darken application content monotonically at depths one and two; the exposed
first dialog is dimmed only by the second backdrop; the second dialog is not
self-dimmed; and topmost close restores the exact depth-one appearance within
the qualified channel tolerance. Input/focus observations separately prove
that only the topmost modal accepts interaction and that dismissal restores the
lawful underlying focus scope. Masks exclude dialog edges, text anti-aliasing,
motion frames, and compositor-external chrome, but may not exclude stable
interior pixels merely because they disagree.

### `AP-07`: Coherent-state, authority, and protocol hostility

Use production public declarations, the active-session entry surfaces, the
existing observation/rebind/publication progression, and production headless
and native host contracts inside existing certification targets.

The world contains one node whose role consumes all six state axes, a text
child, a two-deep modal portal stack, a same-surface non-modal portal, a motion-
retained exiting portal, and two semantic surfaces with different active theme
bindings. Drive this hostile sequence with both modal layers initially visible:

1. pointer press begins on a focusable selected target while its operability
   and validation facts are current;
2. a motion track changes current presentation geometry without changing the
   semantic target;
3. pointer motion crosses the target boundary while capture remains held;
4. window focus changes focus-visible posture;
5. a theme switch is prepared for surface A;
6. a hot rebind retires the original target, reincarnates its stable authored
   identity, changes one role decision table, and changes the active binding of
   surface B;
7. stale, duplicate, wrong-surface, wrong-generation, wrong-incarnation, and
   wrong-theme receipts are presented in the worst lawful order;
8. the qualified host rejects one successor before effects, then a separate
   presentation becomes indeterminate after physical work may have begun; and
9. shutdown begins while the pointer, prepared switch, retained exit node, and
   indeterminate presentation obligations are live.

Required typed outcomes:

- every resolved `UiAppearanceStateVector` cites one coherent application,
  surface, mounted incarnation, presentation, and owner-revision basis;
- hover uses current presentation hit testing, pressed uses the gesture/capture
  owner, focus uses semantic focus plus focus-visible modality, and none is
  reconstructed from pixels or colors;
- the reincarnated authored identity does not inherit hover, pressed, focus,
  selection, validation, or theme authority merely because its ID is equal;
- surface A's capability cannot switch surface B, and neither a theme ID,
  revision, digest, slot key, inspection receipt, nor serialized projection can
  construct switch or projection authority;
- ambiguity, missing coverage, type mismatch, unsupported host mechanics, and
  stale currentness deny before appearance or host effects;
- rejection before effects preserves the exact predecessor;
- an indeterminate host effect retains the existing reconciliation authority
  and is never reported as rolled back or completed from timeout;
- appearance does not replay focus, selection, gesture, Query, or service
  semantics to recover physical work;
- the overlay snapshot contains exactly the authored backdrop declarations at
  their admitted positions; one modal has no backdrop, one non-modal portal has
  an authored backdrop, and neither condition changes Portal shielding;
- equal dark backdrops accumulate, transparent backdrops remain paint-only, and
  no host, theme resolver, or appearance projection may invent, attenuate,
  flatten, or reorder layers;
- backdrop presence and Motion follow only their separately declared bases; a
  presence-following backdrop with `UiBackdropMotionBasis::None` does not
  silently inherit a Portal track;
- topmost close, parent close, hot-rebind removal, and exit retention update all
  declared presence/placement dependents atomically without one-frame flashes,
  orphan commands, or stale shielding;
- exit retention keeps only exact live projection/mechanic dependencies; and
- shutdown reaches zero appearance roles, active bindings, prepared switches,
  state vectors, retained projection facts, host commands, cursor mechanics,
  and inherited service/host obligations, or reports the exact retained
  indeterminate owner.

Mutation sensitivity is mandatory. Inverting currentness, reading committed
instead of presentation geometry for hover, copying stable-ID state to a new
incarnation, replacing decision-table admission with source order, accepting a
foreign theme receipt, dropping the text-layout reuse check, or converting
indeterminate to success must turn this courtroom red. So must composing
appearance and Motion opacity twice or after `u16` precision is discarded,
sorting overlay layers by portal identity or depth alone, auto-creating a modal
backdrop, retaining only the topmost backdrop, flattening the stack into one
effective alpha, implicitly copying Portal Motion, or letting any backdrop
create or disable shielding.

### `AP-10`: Scale, locality, and amplification

Use the existing explicitly filtered closure-stress/subprocess lane, made
required on the 3.16 integration spine and nightly/master qualification, with
production role/theme registration, graph indexes, state owners, mounting,
headless/native presentation contracts, and independent model oracles.

The scale world contains:

- 4,096 mounted nodes across 64 unrelated appearance neighborhoods, of which
  3,072 have appearance roles and 1,024 are unstyled unrelated-neighborhood
  oracles;
- 256 appearance roles and 512 semantic theme slots;
- all six state-axis families, with at most 64 simultaneously hovered,
  pressed, focused, selected, or validation-changing consumers;
- 32 qualified text paragraphs whose foreground changes while layout meaning
  does not;
- four semantic surfaces, each with one explicit active theme binding;
- one 32-deep portal stack on one surface with 48 separately declared
  backdrops at varied before/after/region positions, including portals with no
  backdrop and non-modal portals with backdrops; all count inside the ordinary
  projection bound and use mixed equal-dark, colored, and transparent values;
- 64 active Motion tracks whose sampled opacity composes mechanically with
  appearance opacity; and
- a theme switch changing 12 slots, of which three resolve to byte-equal values
  and four have no mounted consumers.

The ordinary mounted-projection limit remains 4,096. A separate focused
saturation proof fills all 4,096 projection records and requires a typed
capacity denial for the 4,097th while the predecessor remains current; `AP-10`
does not confuse that ceiling proof with the 4,096-node locality world.

Named counters include:

- `appearance_state_sources_read`;
- `appearance_vectors_resolved`;
- `appearance_decision_cells_visited`;
- `theme_slots_compared`;
- `appearance_consumers_selected`;
- `semantic_aspects_changed`;
- `mounted_mechanics_changed`;
- `equal_output_changes_suppressed`;
- `text_layouts_reused` and `text_layouts_requalified`;
- `damage_regions_emitted`;
- `host_commands_added`, `changed`, and `removed`;
- `portal_stack_rows_read`, `backdrop_declarations_selected`,
  `overlay_relation_edges_visited`, `backdrop_mechanics_changed`, and
  `backdrop_commands_replayed`;
- `pointer_targets_retested`; and
- `unrelated_neighborhoods_touched`.

Ordinary bounds are:

```text
state turn = changed owner facts + indexed appearance consumers
theme turn = changed slots + indexed consumers of those slots
projection = declared axes/aspects and compiled decision cells for that role
mounting = semantic aspects whose resolved values or attribution changed
host work = changed commands, order edits, and exact damage
portal turn = changed Portal rows + indexed backdrop scope/presence/placement/Motion dependents
backdrop turn = changed declarations + exact mechanics + exact extent damage
backdrop replay = retained commands intersecting that damage, reported separately
```

There is no whole-graph, whole-role-catalog, whole-theme, whole-backdrop-
declaration, whole-overlay-relation, whole-text-layout, or whole-draw-list
fallback after admission. `unrelated_neighborhoods_touched` is
exactly zero; `text_layouts_requalified` is zero for a color-only switch;
inactive state axes and unused roles cost zero per frame; and an unchanged turn
produces zero appearance and host work.

### Mutants the portfolio must kill

The portfolio fails if an implementation:

- styles by component kind, name, tree position, selector, registration order,
  adapter default, or last writer instead of an explicit role attachment;
- stores a generic map, JSON value, dynamic property bag, or untyped string for
  appearance aspects, state axes, theme values, or host mechanics;
- reduces operability, focus, validation, or selection to an unproven boolean;
- lets appearance own or mutate a service, interaction, Query, layout,
  participation, hit-test, text-layout, or accessibility truth;
- uses committed target geometry instead of current presentation for hover;
- treats pointer capture as hover or derives pressed state from a paint color;
- permits overlapping decision rows, implicit fallback, incomplete coverage,
  or source-order resolution;
- treats a token ID, role ID, digest, revision, source span, or inspection record
  as capability;
- applies one surface's theme through process-global or ambient state;
- keeps the legacy static-paint resolver or filled-rectangle contract alive as
  a parallel appearance authority;
- silently drops border, radius, outline, opacity, foreground, or cursor
  mechanics when a host lacks support;
- changes text color by reshaping, remeasuring, or rerasterizing alpha glyphs;
- lets opacity zero remove hit testing or radius alter child clipping;
- auto-creates or requires a backdrop from portal kind, collapses authored
  backdrops to the topmost or one effective alpha, uses implicit source order,
  or makes backdrop paint own an input shield;
- lets border change allocation or paints it outside the allocation;
- lets an outline accept input or omits its visual overflow from damage;
- publishes a theme switch directly, creates a new settlement/retry lane, or
  guesses success after uncertain native effects;
- scans all nodes or all theme slots for one state or token change;
- uses a detached screenshot, a generated mockup, or producer-authored expected
  pixels as the product oracle;
- passes all mechanical tests while the cumulative Pulse still looks like a
  framework demo; or
- exposes actionable undo/redo, raw Query, or ordinary replay.

## Supporting Proof Portfolio

The decisive courtrooms are supported by focused, mutation-sensitive evidence:

- compile-pass Rust/DSL lowering-equivalence fixtures and public native/
  host-neutral progression fixtures in the existing compile-contract sessions
  (two Cargo invocations);
- compile-fail proofs for forged capability receipts, raw role/theme IDs used
  as authority, preview-to-live promotion, wrong application/surface/
  generation/incarnation/axis version, value-kind mismatch, incomplete or
  overlapping partitions, missing host support, generic authority markers,
  and host attempts to construct semantic appearance;
- exhaustive finite-partition tests for every admitted role fixture, plus
  property tests that reorder declarations, aliases, slots, axes, and cells
  without changing normalized meaning;
- independent model tests for six-axis vector coherence, hover transitions,
  pointer identity, capture-aware pressed state, focus-visible changes,
  selection reincarnation, validation currentness, and duplicate/stale event
  ordering;
- theme catalog/definition tests for kind preservation, alias cycles and depth,
  monotonic revision, explicit surface binding, equal-output succession,
  source-edit and programmatic switch equivalence, capacity saturation, and
  wrong-world receipts;
- index deletion/reconstruction tests proving the state/role appearance
  indexes, existing consumed-fact slot relation, and overlay dependency index
  are derived, plus hostile mutants that add a second slot index or replace
  indexed selection with a whole-graph, whole-catalog, or whole-overlay scan;
- per-aspect equivalence and delta tests that independently compare semantic
  projection change, resolved value change, mounted attribution change,
  physical mechanic change, and equal-output suppression;
- text tests using mixed authored paint spans, bidi, alpha glyphs, color-only
  theme change, and opacity composition, with independent counters proving zero
  qualification, shaping, measurement, rasterization, atlas upload, and caret/
  hit-geometry drift. Intrinsic-color input is a typed denial under the frozen
  BodyDefault v1 profile, not a vacuous emoji raster;
- geometry/property tests for proportional radius normalization, inward border
  limits, outline expansion, Mosaic seam ownership, half-open bounds, clip/
  damage agreement, rounded paint with rectangular hit testing, and
  logical-to-physical conversion at 1.0, 1.25, 1.5, and 2.0 scale;
- headless/native contract parity for surface, outline, text foreground,
  opacity, pointer affordance, initial/delta/unchanged work, reconstruction,
  and explicit unsupported-version/mechanic denial;
- deterministic reference-raster checks for anti-alias fringe, border/radius/
  outline shape, premultiplication, opacity multiplication, and exact damage,
  while keeping external native pixels as the executable product evidence;
- rebind/failure tests for malformed role/theme edits, exact predecessor
  preservation, stale prepared switch, supersession, cancellation before
  effects, in-flight completion, indeterminate reconciliation, multi-surface
  partial failure, and shutdown;
- Platform Pulse structural tests against an independent visual contract,
  contrast/luminance computation from authored theme values, text containment,
  hit-test honesty, cursor semantics, ordered backdrop composition,
  source-span attribution, and native pixel masks at both sizes and at zero,
  one, and two modal depths;
- independent overlay-stack model and retained-order tests over nested and
  sibling portals; modal-without-backdrop and non-modal-with-backdrop cases;
  before/after/region placement; same-depth sibling ordering; colored and
  transparent backdrops; repeated per-portal instances; cross-scope denial;
  topmost and parent close; exit retention; resize; reconstruction; and
  reincarnation. The oracle folds source-over itself from
  authored layer values and the issued overlay order; it may use
  `1 - product(1 - alpha_i)` only as the equal-black-layer
  cross-check, never call the production compositor or consume its effective
  output;
- a separate recorded human review of the actual cumulative executable worlds,
  not a generated image or replacement mockup; and
- extended Worth UI boundary/generated-context, dependency-direction,
  feature-matrix, app-inclusive line-cap, protocol-version, documentation-link,
  public-example, deletion-inventory, closure-stress, and exact-resource-census
  checks.

Fixtures may share setup builders. Oracles may not call or copy the production
partition compiler, appearance resolver, theme comparer, reverse-index
selector, radius normalizer, delta producer, opacity or backdrop compositor,
or native shader logic whose correctness they claim. Test-only constructors may
script legal owner observations through certification boundaries; they may not mint a
live state vector, capability receipt, projection, prepared switch, mounted
fact, or successful host settlement.

## QA considerations

Architecture review must verify that Portal remains the only owner of portal
membership, ancestry, total stack order, shielding, focus containment,
dismissal, and exit retention; authored Backdrop owns declaration, presence
basis, extent, and placement; Appearance owns paint meaning; and the overlay
planner only combines current inputs. Lifecycle QA must exercise push, topmost
pop, parent close, rebind removal, resize, reconstruction, indeterminate
presentation, and shutdown with no orphan portal, backdrop, shield, or retained
command. Integration acceptance requires independent
ordered-compositing evidence, external native pixels, real input/focus evidence,
and recorded human design approval of the actual stacked Pulse; none of those
evidence classes substitutes for another.

## Product Decision Lock

### Appearance is a compiled semantic lane

`UiAppearanceRole` is a stable, versioned declaration capability. A node uses a
role only through an explicit attachment in its canonical declaration. The
role may declare an `applies_to` admission constraint over component semantic
capabilities, but that constraint validates an explicit attachment; it never
searches the graph or reaches into descendants.

A role declares:

- stable role identity and schema version;
- source owner and source provenance;
- exact appearance aspects it covers;
- the state axes consumed by each aspect;
- a complete finite decision table for each covered aspect;
- semantic theme slots and typed literals consumed by each result;
- host mechanic requirements;
- inspection/disclosure posture; and
- an explicit successor compatibility posture.

`UiAppearanceProjection` is the immutable resolved product for one exact node
incarnation and semantic surface. It carries the role, role revision, theme
capability identity/revision reference, coherent state-vector basis, per-aspect
resolution, source provenance, support posture, and semantic digest. It is
derived truth. It
authorizes neither a state change nor theme mutation and cannot be constructed
from its public fields, digest, or inspection form.

The capability itself remains surface-owned and is not copied into every node.
A binding/capability revision invalidates only indexed consumers of slots whose
resolved meaning changed, not every projection on the surface.

The canonical source artifact, runtime graph, execution plan, appearance
projection, mounted fact, and host mechanic remain distinct. Rust-authored and
file-authored roles lower to the same sealed role declaration and the same
resolver.

### Aspect coverage is explicit and typed

Milestone 3.16 admits exactly these semantic aspects:

| Aspect | Value | Physical meaning in 3.16 |
| --- | --- | --- |
| `appearance.background` | straight sRGBA color or explicit transparent | own surface fill |
| `appearance.foreground` | straight sRGBA color | admitted alpha-text ranges; successor monochrome-icon consumers use the same aspect |
| `appearance.border` | solid color plus nonnegative logical width | inward own-surface stroke |
| `appearance.radius` | four nonnegative logical corner radii | own surface silhouette only |
| `appearance.opacity` | canonical unit interval | own node mechanics, not descendants |
| `appearance.outline` | solid color, width, and nonnegative offset | non-hit-tested visual ring outside allocation |

Each component declaration publishes an exact required/optional appearance
aspect contract. A role attachment must cover every required aspect with the
right value family and may cover only admitted optional aspects. Missing,
extra-incompatible, type-mismatched, or host-unsupported coverage denies before
the candidate becomes active.

Backdrop is the one admitted non-node appearance target. Its declaration
requires complete background and opacity coverage and rejects border, radius,
outline, foreground, pointer-affordance, and component-state aspects. This is a
distinct typed applicability contract, not a generic optional aspect bag.

No aspect changes layout participation, allocation, visibility, hit-test
membership, focusability, semantic focus, accessibility state, portal
modality, or interaction routes. Explicit transparency and zero opacity are
still present, participating, and hit-testable when the declaration says so.

Borders paint inward and therefore consume no additional allocation. Radius
clips only the node's own background and border; descendant clipping remains a
separate existing clip contract. Outline is outside allocation, never enters
hit-test order, and expands visual bounds/damage by its exact width, offset,
and qualified anti-alias fringe. A parent clip may clip outline pixels; the
node's own allocation/radius clip may not. Damage retains the outline's
expanded visual box even where an ancestor clip removes pixels.
Hit testing remains the declared allocation/hit-test geometry; a rounded
painted corner does not create a rounded hit region in 3.16. During mounting,
corner radii are normalized once in logical space by the canonical
proportional-reduction rule: if either pair on an edge exceeds that edge, all
radii are scaled by the minimum ratio that makes every edge sum fit. A border
wider than half the normalized minimum dimension is denied rather than
clamped. Outline radii follow the normalized surface radii plus their declared
offset.
Milestone 3.16 adds `MosaicSeamPaintOwner` to Mosaic declarations. Every shared
edge names exactly one owning region; only that owner may paint the shared
border. Interior shared corners are never radiused. Exterior-corner posture is
declared by Mosaic rather than inferred from adjacency. A region that requests
border or radius on an undeclared shared edge is incomplete coverage, not a
collapse/default rule.

### Backdrops are authored overlay participants, not modal side effects

The existing `UiMountedPortalOverlayMechanic` remains the mounted portal
surface, placement, lifecycle, and shielding affinity. It is not stretched or
reinterpreted as a viewport dimmer. A backdrop is instead declared explicitly
as `UiBackdropDeclaration`, with stable `UiBackdropIdentity`, exact extent
basis, presence basis, overlay placement relation, optional declared Motion
basis, and appearance role. Portal kind never creates, requires, positions,
colors, animates, or removes a backdrop.

Milestone 3.16 admits `SurfaceViewport` and current presented Mosaic-region
extent bases. Geometry comes from the named surface or region owner; Backdrop
does not become a layout owner. Presence is either `Always` or the explicit
`WhilePortalPresented(UiPortalDeclarationId)` basis. The latter consumes a
sealed Portal lifecycle export but does not give Backdrop open/close authority.
`UiBackdropMotionBasis` is independently `None` or explicitly follows one
Portal presentation Motion export; presence does not imply motion and motion
does not imply presence.

Declaration and live instance identity remain distinct. Every backdrop declares
`UiBackdropScope::SurfaceSingleton` or
`UiBackdropScope::PerPortalInstance(UiPortalDeclarationId)`. The second form
materializes one sealed `UiBackdropInstanceIdentity` for each exact current
portal incarnation matching the declaration and resolves same-portal presence,
placement, and Motion against that incarnation. It is still authored behavior,
not a portal default. Reincarnation mints a new backdrop instance; repeated
component portals never share one global row. Cross-instance or cross-scope
relations that cannot identify exactly one anchor deny before publication.
Milestone 3.17 may supply pure authored presence inputs through the frozen
expression boundary without changing backdrop identity, placement, or host
mechanics.

Placement is an explicit acyclic relation to a semantic overlay anchor:
`AboveSurfaceContent`, `ImmediatelyBeforePortal`, `ImmediatelyAfterPortal`,
`ImmediatelyBeforeBackdrop`, or `ImmediatelyAfterBackdrop`. Portal and backdrop
anchors use typed declaration identities, never raw integers or runtime IDs.
The Portal anchor denotes its indivisible presented surface and ordinary
mounted-content subtree: `Before` precedes the surface and `After` follows that
content. A backdrop cannot split portal content from its children. Nested
portals remain separate Portal-stack participants with their own anchors.
The overlay-composition planner combines the admitted relation graph with the
current Portal stack and emits one sealed `UiOverlayStackSnapshot` for an exact
application generation, semantic surface, presentation, Portal revision, and
backdrop-declaration revision. Missing anchors, cross-surface references,
cycles, and overlapping participants whose order remains ambiguous deny before
publication. There is no CSS-like `z-index`, source-order tie-break, or adapter
sort.

Portal remains the sole owner of portal membership, parentage, activation
order, shielding, focus containment, dismissal, and lifecycle. It publishes a
sealed `UiPortalStackSnapshot`; every live row carries a
`UiPortalStackOrdinal` that totally orders nested and sibling portals. Depth is
ancestry evidence and cannot break same-depth ties. The Portal owner mints the
ordinal monotonically on a non-idempotent open, never reuses it during the
session, and retains it through visible and closing postures. An idempotent
duplicate does not move a portal; a lawful topmost replace receives a new
ordinal in the same prepared transition. Exhaustion denies before effects.

Because `UiAppearanceProjection` is node-incarnation meaning, mounting never
forges a node receipt for a backdrop. The resolver emits the sealed sibling
`UiBackdropAppearanceProjection`, bound to the backdrop declaration, exact
extent/presence/placement bases, semantic surface, theme capability, role
revision, and current overlay snapshot. It admits only background and opacity,
uses the same typed role/theme machinery, consumes ordinary projection
capacity, and grants no Portal, layout, or input operation.

The product may therefore author, among other lawful arrangements:

```text
application content
-> backdrop A, while modal A is presented
-> modal A portal surface and descendants
-> backdrop B, while modal B is presented
-> modal B portal surface and descendants
```

That arrangement produces familiar cumulative modal dimming, but it is product
composition rather than platform convention. A modal may lawfully have no
backdrop. A dropdown may lawfully have one, and an `Always` backdrop may exist
without any Portal. A backdrop may be placed after a portal to affect that
portal, or multiple independently authored backdrops may occupy admitted
positions in the same overlay plane.
Nothing infers modality, shielding, dismissal, presence, or placement from
paint.

Composition is canonical premultiplied Porter-Duff source-over in bottom-to-top
`UiOverlayStackSnapshot` order. Authored RGB remains straight sRGBA; the
qualified v2 profile decodes RGB through the named sRGB transfer function,
multiplies decoded linear-light channels by the exact product of color alpha
and canonical `u16` appearance opacity, folds source-over in linear light, and
encodes sRGB only at the qualified output boundary. Gate 0 freezes the transfer
constants, conversions, round-to-nearest-even points, and headless reference
algorithm. For equal black layers, `1 - product(1 - alpha_i)` is an independent
oracle cross-check, never a flattened runtime value.

Paint and interaction remain orthogonal. A backdrop is paint-only and never
creates or removes a Portal shield, hit target, dismissal route, focus scope,
or pointer cursor. A zero-opacity backdrop still occupies its declared visual
position but has no input authority. If a product wants a clickable scrim, it
must declare a real interaction surface and admitted action separately rather
than making color clickable.

A transition that changes Portal state may select zero, one, or many dependent
backdrop declarations. Proposal compilation reserves and publishes the complete
successor overlay snapshot atomically with the ordinary mounted successor, but
does not call this a portal/backdrop pair. Portal close, parent close, rebind,
resize, incarnation replacement, and exit retention recompute only declared
scope, extent, presence, placement, and Motion dependents. No frame may expose
stale geometry, orphan retained commands, guessed ordering, or partial dependent
publication.

The honest damage for a full-viewport backdrop change is its clipped viewport;
a region backdrop damages only its presented region. Mechanic construction is
proportional to changed declarations and indexed presence/placement dependents,
while raster replay is counted separately for retained commands intersecting
that damage. A lower-layer change recomposites its affected region and ordered
suffix without reconstructing Portal state or scanning the UI graph.

An action-bearing dialog or popover still uses authored header/body/action
regions with ordinary title placement and real semantic controls. Cancel and
primary actions route through admitted declarations and occupy conventional
secondary/primary positions. An icon appears only through an admitted icon
projection and host mechanic—never an adapter glyph or Unicode substitute.

### Theme slots and theme definitions are separate meanings

The bootstrap `ThemeTokenDescriptor` contract is cut over into two canonical
responsibilities rather than copied:

- a semantic slot catalog declares `ThemeTokenId`, family, value kind, source
  ownership, alias topology, disclosure, and compatibility; and
- `UiThemeDefinition` declares one stable theme identity and revision plus a
  complete, typed value table for the admitted slots it provides.

`ThemeTokenValue` expands from color-only bootstrap meaning to a closed typed
sum for color, opacity, logical length, corner radii, solid stroke, and solid
outline values. A role may consume only a slot whose declared kind matches its
aspect. Aliases are cycle-free, kind-preserving semantic slot aliases; a theme
definition cannot change an alias target or use an alias as override
precedence.

Canonical numeric forms are deterministic:

- authoring may use `#RRGGBB` or `#RRGGBBAA`, but registration parses it once
  into `UiThemeColor([u8; 4])`, logical straight sRGBA. Six-digit form implies
  alpha 255, ASCII hex case is immaterial, digests consume the four bytes, and
  no ambient color-space lookup or mounting-time string parse exists;
- opacity is an integer unit interval with 65,535 as one, not an unconstrained
  floating-point value;
- logical lengths use `UiLogicalLength(i32)` at the platform's 1,000-subpixel
  logical-point scale, are nonnegative where their value kind requires it,
  remain integers through normalization/digests, and convert to physical
  pixels only in the host; and
- four-corner radii, stroke, and outline values preserve their typed
  constituents and canonical ordering. Proportional radius reduction uses
  exact integer arithmetic with round-to-nearest-even; bare `f32` appearance
  lengths and negative zero cannot enter ordinary meaning.

Platform and application definitions are admitted in 3.16. Admission rejects
`ThemeTokenSource::{PluginCustom, PluginAlias, PluginPlatformOverride}` and
`ThemeTokenFamily::Unknown` before freeze. Plugin contribution and override
remain unsupported until Milestone 15. The successor insertion contract is a
typed `UiThemeContributionOwner` identity plus generation on registrations; no
constructible placeholder type or empty module ships in 3.16, and no plugin
owner may be fabricated merely to register an application theme.
Milestone 9 may register curated light, dark, high-contrast, density-aware, and
custom design-system definitions without changing slot or binding authority.

A theme identity is stable within one application lineage and every semantic
change consumes its exact predecessor revision to create one monotonic
successor. The same identity/revision with different catalog, alias, value, or
support meaning is a conflict denial. An active binding carries its own binding
generation in addition to the chosen definition revision; equal numeric
revisions from another application or surface are never substitutable.

There is no ambient nearest-theme lookup. Every semantic surface has exactly
one explicit `UiActiveThemeBinding`. An application default is materialized as
an explicit per-surface binding during preparation, not inherited dynamically.
This makes multi-window and future surface-specific themes additive without a
process-global cascade.

### Theme capability and switching

`UiThemeCapabilityReceipt` proves that one registered theme definition, slot
catalog revision, required role set, semantic surface, application generation,
and host support profile were admitted together. Only the application/theme
admission owner constructs it. It authorizes resolution for that exact world;
it does not authorize source edits, state changes, publication, host effects,
another surface, or another theme revision.

Initial bindings are prepared before application activation. A live switch
uses:

```text
UiThemeSwitchRequest
-> admitted observation-family origin
-> current application/theme admission
-> existing observation-turn and affected-scope/rebind planning
-> existing mounted publication and host settlement
-> UiThemeSwitchOutcome projected from UiRebindOutcome
```

The outcome distinguishes published, observed-no-change, duplicate,
superseded-before-effects, rejected-before-effects, in-flight, and
indeterminate. It does not introduce a theme publisher or retry/recovery lane.
A source edit that changes a theme definition enters the same observation and
rebind progression; programmatic switch is a typed sibling origin, not a
parallel executor. `prepare_theme_switch` admits an origin and exact predecessor;
there is no theme-specific `execute_theme_switch` presenter. The existing
presentation-state theme revision/CAS becomes the active binding's internal
predecessor check, and no direct per-token public mutation lane remains.

Equal identity or equal pixels do not prove equivalence. Theme semantic
comparison includes definition identity/revision, slot catalog, alias
resolution, typed values, support requirements, and binding scope. A changed
definition with byte-equal resolved outputs may advance evidence without
physical work; inspection must distinguish that from no observation.

### State axes consume owner-issued truth

`UiAppearanceStateAxis` classifies a finite visual input family. It is not a
generic `(name, bool)` or extensible value bag. The 3.16 set is sealed:

- `operability`;
- `focus`;
- `validation`;
- `selection`;
- `hover`; and
- `pressed`.

Each has a separate adapter and typed source product. Several products do not
exist at the start of this milestone; 3.16 creates them in the contract wave
rather than treating test-only readers or six independent live reads as an
inherited boundary.

| Axis | Authority owner | Admitted appearance classes |
| --- | --- | --- |
| operability | Intent admission standing fact over the existing decision/affinity proof | ready, pending, occupied, denied, unsupported, stale |
| focus | sealed Focus appearance export including window and focus-visible modality | unfocused, focused, focus-visible, focused-window-inactive |
| validation | sealed typed application fact only | unspecified, valid, advisory, invalid, pending, stale |
| selection | sealed Selection per-key export under owner/revision/incarnation | unselected, selected, anchor, cursor, selected-anchor-cursor |
| hover | new pointer-presence owner over current presented hit testing | outside, hovered |
| pressed | sealed Gesture/capture export | idle, armed-inside, captured-outside |

Every axis has a stable schema identity and version. A role pins the axis
version whose finite classes it partitions. A new owner distinction that maps
honestly to an existing visual class does not change that version; a genuinely
new visual class requires a successor axis version and explicit role
admission. Unknown classes never enter an existing partition through a default
or wildcard.

The visual class is a lawful projection for decision-table finiteness; it does
not replace the source posture. Every axis value retains a bounded reference to
the exact owner-issued source, revision, currentness, and reason class needed
for inspection. Appearance cannot turn the class back into an operability
proof, focus request, selection receipt, validation fact, pointer observation,
or gesture continuation.

Intent admission adds `UiIntentOperabilityStandingFact`, keyed by graph node,
mounted incarnation, and declared intent route. It retains the complete
`UiIntentOperabilityDecision` and publishes when that decision's exact retained
meaning changes; a class-preserving change is evidence succession, not a
mechanical-state change.
The appearance adapter is the following closed projection under the existing
`primary_cause()` priority; `disabled` is not an axis class, preset name, or
diagnostic:

| Existing owner cause | Appearance class |
| --- | --- |
| no primary cause | `ready` |
| `Pending` | `pending` |
| `Occupied` | `occupied` |
| `Unsupported` | `unsupported` |
| `StaleTarget`, `WrongWorld`, `RebindRequired` | `stale` |
| `PolicyDenied`, `Readonly`, `ConfirmationRequired` | `denied` |

The adapter never re-runs intent admission and never reduces its retained
source to `is_operable()`. “Affinity” is the governing owner axis name;
“currentness” remains the admission/revalidation procedure.

Validation is presentation of admitted validation truth, not a new validator.
3.16 admits only `UiValidationAppearanceFact`, a sealed application-fact family
with identity, revision, currentness, and classes `unspecified | valid |
advisory | invalid | pending | stale`. Its private constructor is owned by the
existing typed application-fact state. Missing fact maps to `unspecified`, never
`valid`. Draft/IME state, Intent operability, Query outcomes, strings, and
adapter-side validation logic cannot populate it. A later forms milestone may
add an owner-issued source through a successor version without creating a
second validator.

Focus adds a production `UiFocusAppearancePosture` export carrying semantic
target, window focus, focus-visible modality, and owner revision. `Initial`
maps to `unfocused` when there is no semantic target and otherwise to `focused`;
an existing target with inactive window maps to `focused-window-inactive`
before modality is considered; otherwise only admitted keyboard-visible
posture maps to `focus-visible`, and pointer posture maps to `focused`.
Promoting a `#[cfg(test)]` reader is not a substitute for the sealed export.

Selection adds `UiSelectionAppearancePosture`, queried by owner identity,
stable key, and mounted incarnation. It carries owner revision and returns a
typed ambiguity denial instead of treating multiple owners as `None`. Its
`cursor` class is exactly `UiSelectionOwnerRecord::cursor`; no second lead field
or catalog-index inference exists. For one key the closed mapping is: anchor
and cursor -> `selected-anchor-cursor`; anchor only -> `anchor`; cursor only ->
`cursor`; selected only -> `selected`; none -> `unselected`. The retained owner
source still carries selected/anchor/cursor bits even when the visual class
groups them.

The pointer-presence owner is added under runtime interaction because pointer
motion currently has no hover truth owner. `MechanicalFamily::PointerMotion`
becomes an admitted observation family owned by pointer presence. Latest-value
coalescing remains lawful: hover means the latest admitted coalesced position,
and an intermediate enter/leave pair erased by coalescing is not semantic
history. The owner re-hit-tests on exactly two triggers: an admitted pointer
observation, and a committed presentation whose old/new hit geometry intersects
the last pointer position. Each retest is bounded to the changed hit-test
neighborhood, never the whole surface. It publishes only when the current hover
target changes, owns neither gesture capture nor appearance, and gives
same-target motion zero appearance work.

Presence is keyed by the host-issued pointer identity. The primary pointer is
the most recently admitted mouse or stylus identity on that surface; touch
cannot select `Activation`, and other pointers cannot overwrite it. One mounted
pointer-affordance mechanic exists per surface. Absence maps to `Default`.

Pressed state is an exported projection of the existing gesture/capture
lifecycle. A press remains bound to its original presented incarnation;
movement outside while captured becomes `captured-outside`, and replacement,
capture loss, cancellation, or terminal release clears the state with an exact
reason. Appearance may not infer pressed from a mouse button report alone.
The gesture owner consumes the admitted latest pointer position and committed
presentation trigger needed to maintain inside/outside posture; appearance does
not create a second pressed owner. The superseded replacement-inventory hover
and pressed placeholders are deleted with an exact rebind-policy migration.

### Coherent state vectors

`UiAppearanceStateVector` is a sealed snapshot over only the axes declared by
the role. Milestone 3.16 adds `UiAppearanceOwnerSnapshot` as one sealed product
of observation-turn close. Each consumed owner supplies one exact-current,
axis-relevant snapshot plus its revision to the session; the turn seals them
together with admitted observations. The appearance vector's only constructor
takes that one product and carries:

- application and generation;
- semantic surface and theme binding;
- graph node, mounted instance, and incarnation;
- completed presentation basis where pointer state is consumed;
- role identity/revision;
- exact owner revisions for every consumed axis; and
- source provenance/currentness posture.

The resolver cannot assemble a vector by reading six mutable owners at six
different times, accept six owner references, or stamp independent reads with
one application generation. State adapters are pure functions of the sealed
snapshot and cannot import mutable Focus, Selection, Gesture, Intent, Portal,
or Motion owner state. Any other construction is a typed denial and a
compile-fail case. A changed axis selects consumers through the existing
consumed-fact relation; it does not scan the role catalog or graph.

### Resolution is a finite partition, never a cascade

For each aspect, a role declares a finite set of state predicates over only
the axes that affect that aspect. Its reachable space is exactly the Cartesian
product of the admitted classes of those declared axes; no implicit cross-axis
exclusion or owner folklore prunes it, and 3.16 admits no author-supplied
reachability exclusions. Admission compiles predicates into one canonical
partition of that exact product; a larger product receives the typed cell-cap
denial rather than a wildcard or pruned lowering.

Every reachable cell must map to exactly one result. Two rows that can match
the same cell are an ambiguity denial. A cell with no result is a missing-
coverage denial. Source order, specificity, declaration order, registration
order, and `last wins` are never tie-breakers.

An author may use explicit equivalence groups and an `otherwise same_as ...`
lowering shorthand. One shared partition compiler expands that shorthand into
the finite complement and proves the final cells disjoint and total before
runtime. The cell capacity applies after complement expansion and equivalence-
group merge. There is no runtime default branch and no ambient fallback token.

Different aspects resolve independently. Hover and pressed may affect
background without multiplying focus-outline rows; focus may affect outline
without restating foreground. This prevents a Cartesian authoring explosion
while keeping every individual aspect total and unambiguous.

Canonical ordering is by stable axis identity, normalized value class, and
canonical cell encoding. Semantically equivalent Rust and DSL declarations
produce the same role meaning regardless of source ordering.

### Projection, invalidation, and equivalence

Appearance extends existing reconstructible indexing rather than creating a
second slot relation:

```text
state owner / axis source -> existing consumed-fact index -> appearance consumers
theme slot selector -> existing consumed-fact index -> appearance consumers
role revision -> attached consumers
appearance consumer -> emitted semantic aspects -> mounted mechanics
```

For each candidate it distinguishes:

- input evidence changed;
- semantic appearance projection changed;
- resolved aspect value changed;
- mounted mechanical output changed;
- physical output suppressed because the result is byte-equivalent; and
- work denied before effects.

Those distinctions appear in `UiAppearanceChangeReceipt` and bounded
inspection. A state-source revision may advance while all appearance outputs
remain equal. A theme value may change while no mounted consumer exists. A
role attribution change may require a mounted fact change even when pixels are
equal. None is mislabeled as another.

The current `theme_token_graph_consumers` relation is revised in place as the
canonical slot-selector specialization. The legacy
`UiMountedThemeValueSource::ActiveCurrent::changed_graph_nodes` selection lane
is removed rather than preserved beside it. Derived indexes can be independently
reconstructed from sealed application truth. Index absence is diagnostic/
reconstructive work, never authority for a whole-graph ordinary fallback.

### Text, deferred icons, motion, and opacity

Alpha text consumes `appearance.foreground` from the same projection and
retains original UTF-8 paint-span ownership. No semantic-span-slot type is
added. A text node that opts into `appearance.foreground` declares which
existing `ComponentSemanticTextSpanContract` original UTF-8 ranges consume the
role color. Those ranges keep their span identity; only the mounted resolved
RGBA changes. Token-retaining ranges remain unchanged, cluster boundaries are
preserved, and intrinsic-color clusters are excluded. A color-only change
reuses qualified layout, glyph positions, hit geometry, selection rectangles,
and alpha atlas entries. Only the paint command and exact damage change.

Icons are explicitly out of scope for 3.16. No reserved or empty icon mechanic
type ships. The successor home is a mounting/host projection consuming the
existing icon registry's color-support contract in Milestone 9. Pulse therefore
removes `portal_icon_text` and its Unicode arrow substitute and remains
aesthetically complete without an icon. `InheritsTextColor` icons remain
unmounted until that mechanic exists.

Appearance opacity applies to the node's own surface, outline, and text, not
recursively to descendants.
Motion's presentation-sampled opacity remains a separate mechanical factor.
Both are `u16` with 65,535 as one from the Motion sample onward; ordinary
Motion ingress cannot use `from_runtime_sampling(f32)`. The runtime presentation
producer multiplies them exactly once as a `u32` product divided by 65,535,
saturating and rounding ties to even, and emits one composed
`UiMountedPresentationOpacity`. The host performs no second multiplication or
independent `u16`-to-`u8` requantization. DSL opacity is an integer or exact
ratio such as `40/64`, never a decimal; `from_ratio(n, d)` is the only public
non-integer authoring form. Appearance cannot retarget a Motion track, and
Motion cannot mutate the semantic appearance projection.

A backdrop follows the same once-only multiplication law without pretending to
be a node. Its appearance opacity composes once with any exact Motion sample
named by its own declaration to produce the mechanic's presentation opacity;
a Portal-following presence basis does not imply a Motion track. Source-over
among backdrop layers is a later physical fold and must not be confused with
per-layer multiplication. Neither mounting nor the host may multiply factors
twice, and no effective stack alpha returns to Appearance, Backdrop, Portal,
hit testing, or inspection as authority.

### Pointer affordance is an adjacent semantic mechanic

A cursor is not a color and therefore is not an appearance aspect.
Nevertheless an interactive surface that visually promises activation must not
depend on adapter inference. Milestone 3.16 adds the narrow
`UiPointerAffordanceProjection` sibling with `Default` and `Activation`
families. It derives from the declared interaction kind, current operability,
current hover target, and exact mounted incarnation.

The runtime mounts one current pointer-affordance mechanic for the active
pointer target. The native adapter maps the sealed family to the qualified OS
cursor; headless records the same semantic mechanic. Paint, bounds, component
name, and host widget class cannot select it. This projection has its own
identity and invalidation row and cannot be placed inside
`UiAppearanceProjection` as an optional field.

Later text-entry, resize, drag, precision, and accessibility cursor families
enter as typed siblings after their semantic owners exist. They do not change
the appearance aspect algebra.

### Host contract and clean protocol cutover

The host receives physical mechanics, never roles, theme tokens, state axes,
coverage rules, or resolver instructions.

Milestone 3.16 replaces the filled-rectangle-only bootstrap with these cohesive
families:

- `UiMountedSurfaceAppearanceMechanic`: own-surface fill, optional inward solid
  border, own-surface radii, canonical opacity, exact bounds/clip/layer, node
  receipt, projection attribution, and visual bounds;
- `UiMountedOutlineAppearanceMechanic`: outside solid ring, opacity, exact
  expanded damage/visual bounds, and non-hit-test posture;
- `UiMountedBackdropMechanic`: backdrop identity, overlay placement receipt,
  exact presented extent, straight sRGBA color, canonical opacity, clip,
  appearance attribution, and paint-only posture;
- the existing semantic-text mechanic extended with appearance foreground,
  semantic paint-span attribution, and appearance opacity while preserving the
  qualified layout identity; and
- `UiMountedPointerAffordanceMechanic`: current pointer/surface/target affinity
  and sealed cursor family.

The surface mechanic admits explicit fill-only, border-only, and
fill-and-border variants. It is not a growing optional property bag. Radius is
shape geometry shared by fill and border because those mechanics have one
physical render fate; semantic aspect facts remain separate for inspection and
invalidation.

Rounded surface rendering uses one qualified analytic anti-aliasing rule at
1.0 and fractional device scale. Border width is resolved in logical units,
painted inward, and snapped only by the host's recorded device transform.
Damage derives from each mechanic's visual bounds, not allocation
`bounds ∩ clip`. Only outline visual bounds may exceed their own allocation;
ancestor clipping remains recorded separately. Outline damage includes the
full physical anti-alias fringe. Damage normalization unions overlapping and
edge-adjacent canonical boxes deterministically before command emission.
Headless records the same completed mechanics and deterministic reference
raster posture; it does not claim native compositor pixels.

`UiMountedPortalOverlayMechanic` remains the portal surface/lifecycle/shielding
mechanic. Backdrop is a separate protocol row and retained command because its
extent, appearance attribution, damage, presence, and replacement fate are
independent. Runtime validation rejects duplicate backdrop identities, stale or
foreign extent/presence/placement bases, unresolved or cyclic anchors, and any
row whose application/surface/presentation basis disagrees with the sealed
overlay snapshot. Headless transcripts preserve the issued total order; native
retained commands use the same identities and order. The host receives no
portal-kind-to-backdrop rule.

The contract wave freezes these as intended successor values without changing
any live `CURRENT` constant. One later atomic cutover commit changes all of:

- host protocol `COMPATIBLE_FLOOR` and `CURRENT` advance together from 6 to 7,
  leaving the admissible protocol range a single point;
- mounted-frame and presentation schemas advance from 5 to 6;
- `UiMountedTextSchemaVersion` advances from 3 to 4;
- observation schema remains 7;
- measurement schema remains 5;
- solicited-effect schema remains 1; and
- the qualified Windows appearance profile becomes
  `worth-ui-windows-dx12-v2` while the qualified text profile remains
  `worth-ui-body-default-v1`.

Mounting enforces `worth-ui-body-default-v1`'s declared U+0020–U+007E range
through `UiUnsupportedBodyDefaultCodePoint::first_in` and denies unsupported
text before effects. Because that profile has no reachable intrinsic-color
glyph, 3.16 proves non-retinting with compile-fail/typed-denial evidence rather
than a vacuous emoji raster. Adoption of the staged `worth-ui-global-text-v2`
profile and its intrinsic-color raster proof is explicitly deferred to the
global-text successor milestone.

Negotiation rejects protocol 6, mounted-frame/presentation schema 5,
`UiMountedStaticPaintSchemaVersion` 1, and every
`UiMountedFilledRectMechanic`. The old
`UiMountedStaticPaintSchemaVersion`,
`static_paint` modules, public exports, native/headless translators, and Pulse
bootstrap resolver are removed in the cutover. There is no compatibility
alias, dual emission, or adapter fallback.

The component cutover removes `ComponentDescriptor::static_paint_contract`,
legacy `theme_token_dependencies` that existed only to feed bootstrap paint,
`ComponentStaticPaintContract`, `ComponentStaticPaintOrder`, and their public
facade exports. `ComponentSemanticTextSpanContract` remains canonical text
meaning and is extended only by explicit original-range foreground adoption;
it is not replaced by appearance roles.

All workspace Rust registrations, checked-in `.wui` sources, fixtures,
examples, and docs move in the same cutover. Former component-primary-token or
static-paint source forms receive one source-linked migration diagnostic that
names the required explicit role attachment; they do not lower to an implicit
role. A running pre-cutover binary may preserve its last valid generation after
an invalid edit under its existing rules, but one process never negotiates or
publishes static-paint and appearance commands side by side.

The Windows v2 qualification manifest updates analytic rounded-surface/
outline anti-aliasing, command-family capacities, and its single native host
surface. `AP-10`'s four semantic surfaces are application theme-binding scopes
mounted into that one native host surface, not four OS windows or swapchains.
All native-profile literals and qualification pins in host-native,
native-platform, and runtime move together.

Host support is negotiated per mechanic family and schema before effects. A
host that cannot render a role's required border, radius, outline, opacity,
text foreground, or pointer affordance returns a typed support denial; runtime
does not substitute a square fill or drop the unsupported part.

### Publication, physical failure, and recovery

Appearance uses the existing observation, affected-scope, identity-lifecycle,
plan, publication, presentation, physical-work, and reconciliation owners.
Resolver output alone cannot publish. A prepared theme switch or state change
cannot bypass predecessor currentness, capacity, cancellation, deadline,
surface support, or multi-surface atomicity.

Rejection before host effects preserves the current appearance and active
theme binding. In-flight work retains its completion handle. Effects that may
have begun remain indeterminate and retain exact reconciliation authority.
Recovery reconstructs physical mechanics from current mounted/runtime
authority; it does not replay theme switching or state-owner semantics and
does not infer appearance from retained pixels.

### Inspection and developer courtesies

`worth-ui-inspection` owns the public appearance inspection contracts and
result vocabulary. The runtime owns only bounded producers that populate those
contracts; it cannot mint a second inspection type family. The stable bounded
surface includes:

- `why_appearance(node, aspect)`;
- current role identity/revision and source span;
- active theme identity/revision, surface binding, slot and alias provenance;
- active state classes and bounded owner-evidence references;
- the matched canonical decision cell;
- coverage and host-support posture;
- semantic value, mounted mechanic, and physical-suppression posture;
- exact invalidation cause and changed/visited counters; and
- `why_pointer_affordance(target)`.

`why_appearance` returns found, expired, unsupported, unavailable, or wrong-
world posture. It is a read-only explanation; it cannot construct a state
vector, capability receipt, projection, prepared switch, mounted fact, or host
command.

The public tooling also supplies:

- an effect-free appearance view through the existing mounted-preview lane,
  revised rather than duplicated. `UiMountedThemeValueSource::PreviewOnly`
  takes one explicitly named admitted preview theme binding instead of
  resolving every slot to `None`. Typed preview state remains non-authoritative
  and cannot construct `UiAppearanceOwnerSnapshot`, a live state vector,
  projection authority, mounted publication, or a state transition;
- a normalized role matrix printer that shows finite cells rather than source
  order;
- missing/ambiguous coverage diagnostics naming role, aspect, state cell,
  source spans, expected value kind, and lawful repair;
- a theme-switch receipt summarizing changed slots, selected consumers,
  changed semantic aspects, changed mechanics, equal-output suppression, text
  reuse, and unrelated work; and
- application-profile presets for common focus rings and activation-state
  patterns that lower to ordinary explicit declarations rather than hidden
  runtime branches.

Rich detail remains lazy and budgeted. Ordinary projection facts retain compact
provenance and bounded references, not full histories or formatted narratives.

### Capacity and lifecycle

`UiAppearanceCapacityProfile` is sealed during application preparation. The
qualified ordinary profile admits at most:

- 4,096 appearance roles;
- 4,096 backdrop declarations, each with one extent basis, one presence basis,
  one materialization scope, one explicit placement relation, and at most one
  explicit Motion basis;
- 4,096 semantic theme slots and 32 registered theme definitions;
- 64 slot references per role and 512 canonical decision cells per aspect after
  complement expansion and equivalence merge;
- 4,096 simultaneously mounted appearance projections across 64 semantic
  surfaces;
- 1,024 simultaneously mounted portal-surface rows and 1,024 independently
  mounted backdrop rows; every backdrop also consumes one of the 4,096
  appearance-projection records;
- four concurrently prepared or in-flight theme switches; and
- 64 retained compact appearance-change/inspection records.

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

The public facade is `worth_ui::facade::appearance`. It exports declaration,
theme, switch, receipt, inspection-query, and typed value contracts. Runtime
owners, indexes, state adapters, resolver internals, mounting tables, and host
mechanics remain private to their owning crates.

The intended Rust declaration shape is:

```rust
use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearanceRole, UiAppearanceRoleId,
    UiAppearanceStatePredicate, UiBackdropDeclaration, UiBackdropExtentBasis,
    UiBackdropIdentity, UiBackdropMotionBasis, UiBackdropPresenceBasis,
    UiBackdropScope, UiOverlayPlacement,
    UiSemanticSurfaceId, UiThemeColor, UiThemeDefinition, UiThemeDefinitionId,
    UiThemeOpacity, UiThemeSlotCatalog,
};
use worth_ui::facade::declaration::{ComponentId, ThemeTokenFamily, ThemeTokenId};
use worth_ui::facade::service::UiPortalDeclarationId;

let slots = UiThemeSlotCatalog::new()
    .declare_color(
        ThemeTokenId::new("action.primary.background")?,
        ThemeTokenFamily::accent(),
    )?
    .declare_color(
        ThemeTokenId::new("action.primary.hover")?,
        ThemeTokenFamily::accent(),
    )?
    .declare_color(
        ThemeTokenId::new("action.primary.pressed")?,
        ThemeTokenFamily::accent(),
    )?
    .declare_color(
        ThemeTokenId::new("action.primary.foreground")?,
        ThemeTokenFamily::text(),
    )?
    .declare_radii(
        ThemeTokenId::new("control.radius.medium")?,
        ThemeTokenFamily::surface(),
    )?
    .declare_outline(
        ThemeTokenId::new("focus.ring")?,
        ThemeTokenFamily::focus(),
    )?
    .declare_opacity(
        ThemeTokenId::new("action.inoperable.opacity")?,
        ThemeTokenFamily::disabled(),
    )?;

let dusk = UiThemeDefinition::new(UiThemeDefinitionId::new("pulse.dusk")?)
    .with_color(ThemeTokenId::new("action.primary.background")?, UiThemeColor::hex("#6657D9")?)?
    .with_color(ThemeTokenId::new("action.primary.hover")?, UiThemeColor::hex("#5849C5")?)?
    .with_color(ThemeTokenId::new("action.primary.pressed")?, UiThemeColor::hex("#493CAD")?)?
    .with_color(ThemeTokenId::new("action.primary.foreground")?, UiThemeColor::hex("#FFFFFF")?)?
    .with_uniform_radii(ThemeTokenId::new("control.radius.medium")?, 8)?
    .with_solid_outline(ThemeTokenId::new("focus.ring")?, 2, UiThemeColor::hex("#8B7CF6")?, 2)?
    .with_opacity(ThemeTokenId::new("action.inoperable.opacity")?, UiThemeOpacity::from_ratio(40, 64)?)?;

let primary_action_id = UiAppearanceRoleId::new("action.primary")?;
let primary_action = UiAppearanceRole::new(primary_action_id)?
    .applies_to(ComponentId::new("platform.control.activation")?)
    .cover(
        UiAppearanceAspect::Background,
        [
            UiAppearanceCell::named("ready-outside")
                .when(UiAppearanceStatePredicate::ready_outside())
                .uses_color("action.primary.background")?,
            UiAppearanceCell::when(UiAppearanceStatePredicate::pressed_inside())
                .uses_color("action.primary.pressed")?,
            UiAppearanceCell::when(UiAppearanceStatePredicate::ready_hovered_not_pressed())
                .uses_color("action.primary.hover")?,
            UiAppearanceCell::otherwise_same_as("ready-outside"),
        ],
    )?
    .cover_foreground("action.primary.foreground")?
    .cover_radius("control.radius.medium")?
    .cover_outline_from_focus_visible(ThemeTokenId::new("focus.ring")?)?;

let app = WorthUi::app()
    .register_theme_slot_catalog(slots)?
    .register_theme(dusk)?
    .register_appearance_role(primary_action)?
    .attach_appearance_role(
        ComponentId::new("platform.pulse.action")?,
        primary_action_id,
    )?
    .bind_initial_theme(
        UiSemanticSurfaceId::new("platform.pulse.surface")?,
        UiThemeDefinitionId::new("pulse.dusk")?,
    )?
    .freeze()?;
```

`register_theme_slot_catalog`, `register_theme`,
`register_appearance_role`, `attach_appearance_role`, `register_backdrop`, and
`bind_initial_theme` are new 3.16 builder surfaces and preserve the existing
type-state route to `freeze()`. Every public identity-bearing parameter is a
typed `ThemeTokenId`, `UiThemeDefinitionId`, `ComponentId`, semantic-surface
identity, or `UiAppearanceRoleId`; `&str` is never a second runtime identity
lane. Hex is authoring sugar parsed once into canonical bytes.

Backdrop authoring uses the same role and theme registries but its own explicit
declaration surface. `UiAppearanceRole::applies_to_backdrop()` admits only
background and opacity coverage. A representative Rust-authored declaration is:

```rust
let surface = UiSemanticSurfaceId::new("platform.pulse.surface")?;
let portal = UiPortalDeclarationId::new("pulse.confirmation")?;
let scrim = UiBackdropDeclaration::new(
    UiBackdropIdentity::new("pulse.confirmation.scrim")?,
)?
.with_scope(UiBackdropScope::per_portal_instance(portal))?
.with_extent(UiBackdropExtentBasis::surface_viewport(surface))?
.with_presence(UiBackdropPresenceBasis::while_portal_presented(portal))?
.with_placement(UiOverlayPlacement::immediately_before_portal(portal))?
.with_motion(UiBackdropMotionBasis::follow_portal_presentation(portal))?
.with_appearance_role(UiAppearanceRoleId::new("overlay.scrim")?)?;

let app = WorthUi::app()
    .register_backdrop(scrim)?
    // ordinary portal, role, theme, and component registrations
    .freeze()?;
```

Component attachment cannot target a backdrop, and backdrop registration cannot
target a component. A backdrop may reference a dropdown, popover, modal, or no
portal at all; portal kind is not an admission constraint. Raw runtime portal
identities and integer layer positions are not authoring surfaces.

The public type and progression names in this specification are required.
Builder receivers follow the repository's existing affine/borrowing builder
conventions without changing these static relationships. In particular:

- registration cannot accept an untyped map;
- role attachment names both canonical identities;
- each aspect table is admitted as a disjoint total partition;
- token kind mismatch is reported before freeze;
- initial theme selection is surface-scoped and explicit; and
- builder conveniences lower to the same sealed declarations used by the DSL.

The complete native and host-neutral forms are compile-pass fixtures in the
existing compile-contract sessions (two Cargo invocations). Compile-fail fixtures prove that a
foreign theme receipt, raw role ID, inspection result, wrong value kind,
uncovered state cell, and mismatched component contract cannot advance the
governed path.

A programmatic theme switch follows one affine progression:

```rust
let origin = session.prepare_theme_switch(
    UiThemeSwitchRequest::new(surface, UiThemeDefinitionId::new("pulse.paper")?)
        .observed_at_tick(now)
        .with_deadline(deadline),
)?;
let prepared_rebind = session.prepare_rebind(origin.into_observation_origin())?;

match UiThemeSwitchOutcome::from(prepared_rebind.execute()?) {
    UiThemeSwitchOutcome::Published(receipt) => inspect_change(receipt),
    UiThemeSwitchOutcome::ObservedNoChange(receipt) => inspect_no_change(receipt),
    UiThemeSwitchOutcome::RejectedBeforeEffects(denial) => preserve_predecessor(denial),
    UiThemeSwitchOutcome::InFlight(handle) => retain_completion(handle),
    UiThemeSwitchOutcome::Indeterminate(handle) => reconcile_or_close(handle),
    other => handle_terminal_switch_posture(other),
}
```

The source bridge remains simpler: editing a role or theme in a held `.wui`
snapshot enters `begin_source_rebind` and the ordinary rebind outcome. It does
not construct `UiThemeSwitchRequest` or call a theme-specific publisher.

The DSL exposes the same semantic lane. A representative target is:

```text
theme pulse.dusk revision 1 {
  color action.primary.background = #6657D9
  color action.primary.hover = #5849C5
  color action.primary.pressed = #493CAD
  color action.primary.foreground = #FFFFFF
  radii control.radius.medium = uniform(8)
  outline focus.ring = solid(2, #8B7CF6, offset 2)
  opacity action.inoperable.opacity = ratio(40, 64)
  color overlay.scrim.color = #080B14
  opacity overlay.scrim.opacity = ratio(18, 64)
}

appearance role action.primary applies_to platform.control.activation {
  background over [operability, hover, pressed] {
    cell ready-outside when operability = ready, hover = outside, pressed = idle
      use token(action.primary.background)
    when operability = ready, hover = hovered, pressed = idle
      use token(action.primary.hover)
    when operability = ready, pressed = armed-inside
      use token(action.primary.pressed)
    otherwise same_as ready-outside
  }

  foreground use token(action.primary.foreground)
  radius use token(control.radius.medium)

  outline over [focus] {
    when focus = focus-visible use token(focus.ring)
    otherwise use transparent-outline
  }
}

appearance role overlay.scrim applies_to backdrop {
  background use token(overlay.scrim.color)
  opacity use token(overlay.scrim.opacity)
}

component platform.pulse.action {
  appearance { role action.primary }
}

portal pulse.confirmation {
  kind modal_dialog
}

backdrop pulse.confirmation.scrim {
  scope per_portal_instance pulse.confirmation
  extent surface_viewport platform.pulse.surface
  presence while portal pulse.confirmation presented
  place immediately_before portal pulse.confirmation
  motion follow portal pulse.confirmation presentation
  appearance { role overlay.scrim }
}
```

This syntax contains no selector and no cascade. `applies_to` is a constraint
checked on the explicit component attachment. `otherwise` is expanded and
proved as the finite complement during lowering. Runtime never parses token
paths or evaluates source-order rules.

The default experience includes small courtesies developers otherwise
implement inconsistently:

- a canonical focus-visible outline pattern;
- canonical ready/hover/pressed/inoperable activation partitions;
- semantic transparent values rather than missing paint;
- automatic text-layout reuse for paint-only foreground changes;
- a precise missing-cell diagnostic with a suggested finite predicate;
- an inspectable theme-switch summary;
- current-target hover cleanup on replacement, capture loss, surface loss, and
  shutdown;
- qualified activation cursor projection for operable pointer targets; and
- zero live owner/index/frame cost for unused roles, axes, themes, and pointer
  affordances.

Presets are ordinary normalized declarations. They may not branch inside the
resolver, hide slot dependencies, install every state owner, or acquire theme
authority implicitly.

## Compile-Time and Mechanical Enforcement

Milestone 3.16 adds or extends enforcement for:

- no raw `worth-query` dependency from the new appearance, theme, pointer-
  presence, mounting, host, Pulse, or public-facade lanes, and no ordinary
  `worth-query-replay` dependency;
- no `worth-*` dependency on `worthy-*`;
- no host dependency on runtime appearance, theme, interaction, or service
  internals;
- no selector engine, specificity, cascade, style class list, ambient theme,
  inherited-style walk, dynamic property bag, JSON appearance payload,
  callback, or string-keyed runtime value map;
- no public generic `AuthorityMarker` bound at governed appearance or theme
  surfaces;
- private constructors for capability receipts, coherent state vectors,
  projections, Portal/overlay stack snapshots, prepared switches, mounted facts,
  and host mechanics;
- exhaustive sealed classification for appearance aspects, state axes, theme
  value kinds, host mechanic families, support rows, resource census, and
  inspection posture;
- no role resolution by component name/kind/tree position and no role
  attachment from outside the canonical declaration;
- no overlapping or incomplete decision-table partition after lowering;
- no role or theme registration order in semantic digest or resolution;
- no backdrop creation, requirement, presence, placement, or appearance inferred
  from portal policy/kind; no integer `z-index` or source-order overlay tie-break;
- no untyped substitution among theme definition, slot catalog, active
  binding, capability receipt, role revision, state vector, projection,
  mounted fact, and host command;
- no appearance dependency edge into Focus, Selection, Gesture, Intent,
  Portal, Motion, Query binding, layout, or text internals; adapters consume
  their sealed public runtime exports;
- no Backdrop dependency on Portal internals or mutation surface; a declared
  presence basis consumes only the sealed current lifecycle export;
- no vector constructor that accepts independent owner references or reads
  owners after observation-turn close;
- no state owner callback into appearance and no family-to-family mutation;
- no appearance-triggered layout, participation, hit-test-membership,
  focusability, Query, or text-qualification work;
- no use of color, opacity, bounds, component kind, or host widget type to
  infer pointer affordance;
- no legacy `static_paint` module, `UiMountedFilledRectMechanic`, schema,
  public export, headless/native translator, Pulse bootstrap color resolver,
  or compatibility alias after the protocol cutover;
- no `portal_icon_text`, Unicode icon substitute, mounting-time color string
  parse, legacy component-primary-token dependency, or direct per-token theme
  publisher after the cutover;
- no renderer-default color, border, radius, focus ring, cursor, or unsupported
  mechanic fallback;
- no process-global mutable theme, nearest-parent theme lookup, or cross-
  surface capability reuse;
- no direct theme/appearance publication or second retry/recovery coordinator;
- no new Cargo test target or executable for the proof portfolio;
- complete appearance/theme/support/census classification when a committed
  successor adds a new aspect, value family, state axis, or host mechanic;
- compile-pass Rust/DSL lowering-equivalence and native/host-neutral facade
  examples in existing matrices;
- compile-fail forged/wrong-world/coverage/value-kind/host-support cases;
- the repository 400-line Rust cap for every touched code, test, fixture, and
  support file unless an explicit governing exemption exists.

The existing boundary and generated-context tools are extended to discover and
classify `workspaces/worth-ui`; naming them before that extension does not count
as evidence. Milestone 3.16 creates and wires four missing mechanical gates:

- a checked-in exact-count deletion manifest for the frozen legacy inventory,
  ending at zero for every removed symbol/path;
- a protocol manifest asserting all live and intended-next protocol, mounted,
  presentation, text, observation, measurement, solicited-effect, and native-
  profile values, including `COMPATIBLE_FLOOR == CURRENT` after cutover;
- a documentation/link gate over the continuing Worth UI docs and examples;
  and
- a feature-matrix gate extending the native feature checker across the new
  mechanic families and qualified profiles.

The line-cap guard expands to `workspaces/worth-ui/apps/**/*.rs`, is wired into
CI, and the existing over-cap Worth UI files are split before that required lane
is declared green. `closure-stress` becomes required on every 3.16 integration-
spine merge and on the nightly/master qualification cadence. A smaller
deterministic locality world runs per ordinary merge if the full subprocess
world exceeds the fast-lane budget.

Compile-fail evidence explicitly forbids three dishonest intermediate facades:
a default appearance for unattached nodes, an always-`outside` pointer-presence
stub, and a “coherent” basis assembled from independent live owner reads.

Mechanical source scans are supporting proof, not substitutes for type and
runtime tests. The implementation must not rename a forbidden bag or wrapper
to evade a word-based guard.

## Architectural Destination

The destination tree below is normative. Legend:

- `[E]` existing and retained;
- `[C]` created in 3.16;
- `[M]` existing responsibility moved and replaced in 3.16;
- `[R]` removed in 3.16; and
- `[S]` committed successor destination; no empty placeholder is created now.

```text
workspaces/worth-ui/crates/
├── worth-ui/
│   └── src/facade/
│       ├── appearance/                                    [C]
│       │   ├── mod.rs
│       │   ├── role.rs
│       │   ├── backdrop.rs
│       │   ├── theme.rs
│       │   ├── state.rs
│       │   ├── switching.rs
│       │   └── inspection.rs                              [C, facade re-export only]
│       ├── declaration.rs                                 [E, revised exports]
│       └── service.rs                                     [M, portal declaration identity export]
│
├── worth-ui-dsl/
│   └── src/
│       ├── source/parse/                                  [M, parser forms]
│       ├── source/legality/                               [M, partition admission]
│       ├── source/lower/                                  [M, Rust/file equivalence]
│       ├── source/compile/                                [M, sealed package]
│       └── semantic/
│           ├── appearance/                                [C]
│           │   ├── mod.rs
│           │   ├── role.rs
│           │   ├── aspect.rs
│           │   ├── state_partition.rs
│           │   ├── theme.rs
│           │   └── diagnostic.rs
│           ├── overlay/                                   [C]
│           │   ├── mod.rs
│           │   ├── backdrop.rs
│           │   ├── scope.rs
│           │   ├── extent.rs
│           │   ├── presence.rs
│           │   ├── motion.rs
│           │   ├── placement.rs
│           │   └── diagnostic.rs
│           ├── expression/                                [S, 3.17]
│           └── module/                                    [S, 3.18]
│
├── worth-ui-runtime/
│   └── src/
│       ├── capability/registry/
│       │   ├── component/                                 [M, role attachment; static-paint fields removed]
│       │   ├── mosaic_region/                             [M, seam paint owner]
│       │   ├── appearance_role/                           [C]
│       │   │   ├── mod.rs
│       │   │   ├── identity.rs
│       │   │   ├── descriptor.rs
│       │   │   ├── registration.rs
│       │   │   ├── frozen_entry.rs
│       │   │   └── support.rs
│       │   ├── theme_token/                               [E, revised]
│       │   │   ├── descriptor/                            [E, split slot/value meaning]
│       │   │   ├── registration/                          [E]
│       │   │   ├── frozen_theme_token_capabilities.rs     [E]
│       │   │   └── theme_token_registry.rs                [E]
│       │   └── theme/                                     [C]
│       │       ├── mod.rs
│       │       ├── definition.rs
│       │       ├── identity.rs
│       │       ├── registration.rs
│       │       ├── frozen_entry.rs
│       │       └── support.rs
│       ├── declaration/
│       │   └── appearance/                                [C]
│       │       ├── mod.rs
│       │       ├── attachment.rs
│       │       ├── aspect_contract.rs
│       │       ├── decision_cell.rs
│       │       ├── decision_partition.rs
│       │       ├── state_axis.rs
│       │       ├── theme_slot_use.rs
│       │       └── pointer_affordance.rs
│       │   └── overlay/                                   [C]
│       │       ├── mod.rs
│       │       ├── backdrop.rs
│       │       ├── scope.rs
│       │       ├── extent.rs
│       │       ├── presence.rs
│       │       ├── motion.rs
│       │       └── placement.rs
│       ├── runtime/
│       │   ├── appearance/                                [C]
│       │   │   ├── mod.rs
│       │   │   ├── capacity.rs
│       │   │   ├── state/
│       │   │   │   ├── mod.rs
│       │   │   │   ├── vector.rs
│       │   │   │   ├── coherent_basis.rs
│       │   │   │   └── adapter/
│       │   │   │       ├── mod.rs
│       │   │   │       ├── operability.rs
│       │   │   │       ├── focus.rs
│       │   │   │       ├── validation.rs
│       │   │   │       ├── selection.rs
│       │   │   │       ├── hover.rs
│       │   │   │       └── pressed.rs
│       │   │   ├── theme/
│       │   │   │   ├── mod.rs
│       │   │   │   ├── active_binding.rs
│       │   │   │   ├── capability_receipt.rs
│       │   │   │   ├── switch_request.rs
│       │   │   │   ├── prepared_switch.rs
│       │   │   │   └── switch_outcome.rs
│       │   │   ├── projection/
│       │   │   │   ├── mod.rs
│       │   │   │   ├── resolver/
│       │   │   │   │   ├── mod.rs
│       │   │   │   │   ├── cell_lookup.rs
│       │   │   │   │   ├── aspect_resolution.rs
│       │   │   │   │   ├── provenance.rs
│       │   │   │   │   └── support.rs
│       │   │   │   ├── resolved_aspect.rs
│       │   │   │   ├── projection.rs
│       │   │   │   ├── backdrop.rs                       [C]
│       │   │   │   └── change_receipt.rs
│       │   │   ├── invalidation/
│       │   │   │   ├── mod.rs
│       │   │   │   ├── state_consumer_index.rs
│       │   │   │   ├── slot_consumer_query.rs             [M, consumed-fact specialization]
│       │   │   │   ├── role_consumer_index.rs
│       │   │   │   └── affected_scope.rs
│       │   │   ├── inspection/
│       │   │   │   ├── mod.rs
│       │   │   │   └── producer.rs                        [C]
│       │   │   └── trial/                                 [S, 3.22]
│       │   ├── interaction/
│       │   │   └── pointer_presence/                      [C]
│       │   │       ├── mod.rs
│       │   │       ├── owner.rs
│       │   │       ├── current_target.rs
│       │   │       ├── transition.rs
│       │   │       └── inspection.rs
│       │   ├── overlay_composition/                       [C, derived planner]
│       │   │   ├── mod.rs
│       │   │   ├── relation_graph.rs
│       │   │   ├── dependency_index.rs
│       │   │   ├── planner.rs
│       │   │   └── snapshot.rs
│       │   ├── focus/                                     [M, sealed appearance export]
│       │   ├── portal/state/mounted_projection.rs          [M, sealed total stack snapshot]
│       │   ├── motion/                                    [E, sealed export]
│       │   ├── selection/                                 [M, per-key sealed export]
│       │   ├── intent/                                    [M, standing facts]
│       │   └── observation/                               [M, coherent close and pointer motion]
│       ├── runtime/presentation_state.rs                  [M, binding revision owner]
│       ├── runtime/session/application_state/
│       │   └── theme_token_consumers.rs                   [M, canonical slot index]
│       ├── facade/entry/mounted_preview/                  [M, appearance-aware existing lane]
│       ├── mounting/theme_values.rs                       [M, explicit preview binding]
│       ├── mounting/portal_overlay.rs                     [M, ordered portal surface]
│       ├── mounting/backdrop.rs                           [C, authored overlay layer]
│       ├── mounting/presentation/work_producer.rs         [M, visual-bounds damage]
│       └── mounting/projection/
│           ├── appearance/                                [C]
│           │   ├── mod.rs
│           │   ├── fact.rs
│           │   ├── lowering.rs
│           │   ├── surface.rs
│           │   ├── outline.rs
│           │   ├── backdrop.rs                            [C]
│           │   ├── text_foreground.rs
│           │   └── pointer_affordance.rs
│           ├── semantic_text/                             [E, revised consumer]
│           └── static_paint/                              [R]
│
├── worth-ui-host-contract/
│   └── src/
│       ├── mounted_frame/protocol.rs                      [M, atomic version cutover]
│       ├── mounted_frame/presentation_work/
│       │   └── command_change.rs                          [M, command-family cutover]
│       └── mounted_projection/
│           ├── appearance/                                [C]
│           │   ├── mod.rs
│           │   ├── color.rs
│           │   ├── surface.rs
│           │   ├── outline.rs
│           │   ├── backdrop.rs                            [C]
│           │   └── pointer_affordance.rs
│           ├── semantic_text/                             [E, revised]
│           ├── portal_overlay.rs                          [M, portal surface retained]
│           └── static_paint.rs                            [R]
│
├── worth-ui-host-headless/
│   └── src/
│       ├── headless_translation/appearance/               [C]
│       │   ├── mod.rs
│       │   ├── surface.rs
│       │   ├── outline.rs
│       │   ├── backdrop.rs                                [C]
│       │   └── pointer_affordance.rs
│       ├── headless_transcript/appearance/                [C]
│       │   ├── mod.rs
│       │   ├── surface.rs
│       │   ├── outline.rs
│       │   ├── backdrop.rs                                [C]
│       │   └── pointer_affordance.rs
│       └── *_static_paint*                                [R]
│
├── worth-ui-host-native/
│   ├── profiles/*.toml                                   [M, v2 qualification]
│   ├── src/native_profile.rs                              [M]
│   ├── src/qualification_tests.rs                         [M]
│   └── src/native/
│       ├── presentation/
│       │   ├── appearance/                                [C]
│       │   │   ├── mod.rs
│       │   │   ├── command.rs
│       │   │   ├── surface_pipeline.rs
│       │   │   ├── outline_pipeline.rs
│       │   │   ├── backdrop_pipeline.rs                   [C]
│       │   │   ├── antialiasing.rs
│       │   │   └── cursor.rs
│       │   ├── text/                                      [E, revised]
│       │   ├── retained_draw_list/                        [E, new command families]
│       │   └── damage_regions.rs                          [E, revised]
│       └── event_loop/
│           └── pointer_cursor.rs                          [C]
│
├── worth-ui-inspection/                                   [M, appearance query/result contracts]
├── worth-ui-retained-order/                               [M, new command families]
├── worth-ui-certification/                                [M, courtrooms/static-paint fixtures]
├── worth-ui-test-support/                                 [M, honest builders/oracles]
└── worth-ui-native-platform/
    └── src/profile.rs                                     [M, v2 identity]

workspaces/worth-ui/apps/platform-pulse/                    [M, integration consumer]
├── src/product_world/visual_composition/                   [M]
├── src/application/presentation/                          [M]
├── app/*.wui                                              [M]
└── tests/executable_world/adjudication/                    [M]
```

The dominant axes and enforcement are:

- capability registries own stable admissible declarations; they do not own
  live surface bindings or state;
- `declaration/appearance` owns canonical authored meaning and total coverage;
  it does not resolve live values;
- `declaration/overlay` owns stable backdrop, extent, presence, and relational
  placement meaning; it owns no live Portal state or physical ordering;
- `runtime/appearance/state` adapts sealed owner exports into a coherent vector;
  it does not own the source state;
- `runtime/appearance/theme` owns live binding and switch progression; token
  and theme registries remain immutable capability truth;
- `runtime/appearance/projection` owns deterministic derived visual meaning;
  it does not publish or perform host work;
- `runtime/overlay_composition` combines admitted authored relations with exact
  current Portal and extent-owner snapshots into one derived total order; it
  owns no source declaration, Portal lifecycle, appearance, or host effect;
- `runtime/appearance/invalidation` owns reconstructible reverse indexes and
  affected-scope selection while reusing the consumed-fact relation; it is not
  source truth and owns no parallel slot index;
- `worth-ui-inspection` owns public appearance inspection contracts; runtime
  owns only their bounded producer;
- the existing mounted-preview lane is the sole preview integration surface;
- `mounting/projection/appearance` turns a resolved projection plus allocation
  and mounted receipts into semantic mounted facts and physical requirements;
- host-contract appearance owns versioned mechanics only;
- native/headless appearance modules execute or record mechanics and cannot
  import roles, slots, state axes, or resolver code; and
- successor `expression` and `trial` destinations add typed producers or
  replacement origins. They do not insert an override/cascade layer.

The tree explicitly forbids:

- `style.rs`, `styles.rs`, `theme_manager.rs`, `appearance_manager.rs`,
  `visual_state.rs`, `properties.rs`, or generic `helpers/common/util/shared`
  bags;
- placing theme binding inside the immutable token registry;
- placing hover inside appearance or native host code;
- placing state adapters inside the state owners they consume;
- creating a second theme-slot consumer index or a second preview facade;
- placing semantic aspect facts inside host mechanics;
- keeping filled-rectangle bootstrap beside the appearance surface mechanic;
  and
- flattening role declaration, resolution, invalidation, mounting, and
  inspection into one file or directory because they all mention appearance.

Committed successors remain additive:

- 3.17 expressions produce typed state/theme/role-selection inputs and declared
  consumed facts; they do not change the resolver;
- 3.19 diagnostics expand the existing denial/evidence projections;
- 3.20 visual invariants consume mounted visual bounds and appearance facts;
- 3.22 style trials enter `runtime/appearance/trial` as affine canonical
  replacement plans bound to one selected projection, never ambient overrides;
- Milestone 9 registers role/theme/component families against the same facade;
- Milestone 13 consumes foreground/focus/contrast semantics for accessibility
  without making appearance accessibility truth; and
- Milestone 15 adds plugin-owned theme/role registrations with typed owner
  generation and unload through the existing registries.

## Contract-First Parallel Implementation

The following gates are normative implementation order, not suggested project
management. A worktree may begin only when its consumed contracts are merged.
The contract baseline contains real compilable types, owners, and guards; empty
module roots, placeholder mechanics, and public facades that claim unresolved
behavior violate the composition laws.

### Gate 0: one contract baseline before any fork

One short foundation wave freezes and merges:

1. aspect/value kinds, canonical numeric types, host mechanic shapes, and the
   intended-next protocol/version manifest while every live `CURRENT` remains
   at protocol 6, mounted/presentation 5, and text schema 3;
2. all six axis schemas, the operability cause table, validation application-
   fact family, focus/selection/gesture exports, pointer-motion admission,
   Mosaic seam ownership, Portal's sealed total stack snapshot/ordinal, and
   `UiAppearanceOwnerSnapshot` construction at observation-turn close;
3. role attachment, backdrop identity/extent/presence/placement, acyclic overlay
   relation admission, aspect contracts, typed slot uses, exact Cartesian
   reachability, 512-cell capacity, and the one partition-compiler contract
   shared by Rust and DSL;
4. slot catalog/theme-definition split, active binding, theme-switch origin,
   capability receipt, existing consumed-fact index ownership, existing
   mounted-preview ownership, and the presentation-state CAS decision;
5. damage/clip/outline, integer opacity composition, text-range foreground,
   independent portal-surface/backdrop rows, overlay-order snapshots,
   source-over transfer/rounding, primary-pointer, and native-profile contracts;
6. the exact removal/migration inventory, including static paint, bootstrap
   component token dependencies, the string-backed `ThemeColorValue`, Pulse's
   Unicode icon text, direct token publication, and legacy changed-node
   selection; and
7. boundary/context coverage for Worth UI, all four new enforcement gates,
   expanded app line-cap coverage, and required closure-stress scheduling.

This baseline may expose sealed types to implementation crates and compile-
contract fixtures. It emits zero appearance host commands, changes no live
protocol current/floor, creates no icon mechanic, and exposes no public facade
that claims a resolved live appearance.

### Gate 1: first parallel wave, no host emission

After Gate 0, these worktrees can proceed concurrently against the merged
contracts:

- declaration + DSL: one partition compiler, Rust/DSL byte-equivalent lowering,
  appearance attachment plus backdrop/overlay-relation admission, and typed
  diagnostics;
- state + pointer: pure adapters over sealed owner snapshots, pointer presence,
  pressed posture, state-consumer selection, and currentness/reincarnation tests;
- Portal stack + lifecycle: monotonic ordinals, sealed ordered snapshots,
  topmost/parent close, rebind, and exit-retention behavior with no backdrop or
  host convention;
- overlay composition: relation compiler, indexed presence/placement
  dependents, current extent bases, total snapshot, capacity reservation, and
  reconstruction without appearance resolution or host emission;
- resolver + inspection producer: node and backdrop projection,
  equivalence, role/state/slot queries over existing indexes, change receipts,
  bounded inspection, and the revised mounted-preview lane;
- mounting + text + headless: unpublished surface/outline/text/cursor/backdrop
  mechanics, issued overlay order, exact deltas/damage/
  reconstruction, and text-layout reuse; and
- native long pole: rounded-surface/outline pipelines, analytic AA, retained
  command families, and OS cursor mapping behind non-current schema contracts.

No worktree may introduce `Projection -> UiMountedFilledRectMechanic`, default
appearance for unattached nodes, an always-outside hover stub, independent
owner reads, a second slot index, or a second preview facade. Resolver/index
tests use sealed projections with zero host commands. Native begins at Gate 0
because it is the long pole; headless structural parity does not certify native
pixels.

### Gates 2–4: ordered merges into the integration spine

- **Gate 2 — declaration, state, Portal, and overlay order:** lowering is byte-
  identical for a corpus including a role with more than 64 canonical cells;
  every axis consumes only sealed owner exports; Portal snapshots totally order
  nested and sibling portals; backdrop relations compile to one acyclic total
  overlay order; pointer/presentation triggers are bounded; malformed,
  ambiguous, incomplete, wrong-kind, stale, and saturated cases deny typed.
- **Gate 3 — resolver and indexes:** the six change distinctions are separately
  observable; consumed-fact reconstruction matches an independent oracle; no
  second slot-selection lane exists; preview stays non-authoritative.
- **Gate 4 — mounting, text, and headless:** all new mechanic families are
  field-for-field attributable to mounted facts; damage uses visual bounds;
  per-layer opacity composes once; authored backdrop and portal rows preserve
  the issued overlay order and reference-raster with source-over; text identity/
  atlas reuse is exact.
  Appearance host emission remains disabled and static paint remains the only
  live publisher.

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

## Documentation Deliverables

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

## Must Ship and Preserve

Milestone 3.16 must ship:

- `UiAppearanceRole` with explicit attachment and component applicability;
- `UiAppearanceProjection` and per-aspect resolved facts;
- `UiBackdropDeclaration`, declaration/instance identity, scope, extent,
  presence, optional Motion, relational placement, and
  `UiBackdropAppearanceProjection`; `UiPortalStackSnapshot`,
  `UiPortalStackOrdinal`, and `UiOverlayStackSnapshot`; and no fabricated
  backdrop node or modal default;
- `UiAppearanceOwnerSnapshot`, `UiAppearanceStateAxis`, and coherent
  `UiAppearanceStateVector`;
- owner-issued adapters for operability, focus, validation, selection, hover,
  and pressed;
- the pointer-presence owner and exact-current hover lifecycle;
- `UiThemeDefinition`, stable theme identity/revision, typed slot catalog,
  explicit active surface binding, and `UiThemeCapabilityReceipt`;
- typed color, opacity, logical length, radii, solid border, and solid outline
  values with canonical encoding;
- disjoint and total per-aspect state partitions with no cascade;
- background, foreground, border, radius, opacity, and outline coverage;
- one mandatory two-deep Pulse modal stack with two independently declared,
  explicitly placed appearance-backed viewport layers, cumulative source-over,
  and conventional real-control composition in both dialogs;
- reconstructible state/role appearance indexes, the existing consumed-fact
  slot relation, and exact cost receipts;
- mounted surface, outline, backdrop, text-foreground, and pointer-
  affordance facts;
- headless/native mechanics and one clean protocol/schema cutover;
- paint-only text foreground reuse and Motion-opacity composition;
- live programmatic and source-edit theme/role changes through existing rebind,
  publication, settlement, and recovery;
- typed missing/ambiguous/type/support/currentness/wrong-world denials;
- bounded `why_appearance`, role matrix, existing mounted-preview extension,
  switch summary, and resource census;
- cumulative Platform Pulse state/theme behavior with external pixel agreement
  and separate product/design acceptance; and
- complete removal of the legacy static-paint authority path, Unicode icon
  substitute, and parallel token/preview/index lanes;
- real Worth UI coverage in boundary/context/line-cap enforcement plus the four
  new version/deletion/docs/feature gates.

It must preserve:

- the native host as sole native-display platform;
- canonical source lowering and one runtime graph truth;
- runtime ownership of appearance and owner-specific ownership of every input
  state;
- Query audience and authority boundaries, including no raw Query use in the
  appearance lane and no confusion of UI operability with Query admission;
- exact distinction among semantic target, presentation sample, allocation,
  visual bounds, hit testing, and host geometry;
- existing atomic application/mounted publication and physical settlement;
- predecessor truth on before-effect denial and typed indeterminate posture
  after uncertain effects;
- qualified text layout, paint-span, intrinsic-color, raster, and atlas laws;
- Motion as temporal owner and appearance as endpoint meaning;
- appearance-only locality and unchanged-frame zero work;
- explicit resource capacity and exact shutdown census;
- pixels and inspection as evidence, never authority;
- real product actions behind visible affordances; and
- the explicit aspirational-only posture of undo/redo.

## Acceptance and Successor Handoff

The milestone is accepted only when all of the following are true:

- `AP-01`, `AP-07`, and `AP-10` pass through their named production entry
  surfaces and kill their specified mutants;
- the cumulative native Pulse is coherent, polished, non-flat, unclipped,
  usable at both sizes and admitted scales, and separately accepted against the
  contemporary Linear-or-Notion quality bar at zero, one, and two modal
  depths. At two depths the underlying dialog must visibly recede, the top
  dialog must remain crisp and dominant, and the composition must avoid both a
  muddy near-black slab and a mechanically weak translucent wash;
- hover, pressed, focus-visible, selected, inoperable, and validation-bearing
  outcomes shown by the Pulse cite the actual owner-issued state and current
  mounted incarnation;
- theme switching changes exactly the indexed consumers, suppresses equal
  physical output, preserves unrelated structure/Query/service/text-layout
  work at zero, and publishes through the existing successor boundary;
- every admitted visible outcome is explained by declaration, role, theme
  capability, state vector, projection, mounted fact, and host mechanic;
- missing, ambiguous, type-mismatched, unsupported, stale, foreign, and
  wrong-world inputs deny with distinct typed posture before effects;
- native and headless hosts agree on semantic mechanics and support outcomes;
- color-only text changes produce zero text qualification, shaping,
  measurement, glyph-raster, and atlas-upload work;
- radius, border, outline, per-layer opacity, ordered source-over, Mosaic seams,
  hit testing, visual bounds, and damage obey the locked box laws;
- the Pulse necessarily opens a real two-deep `modal_dialog` stack with Portal-
  owned total order and shielding plus two separately declared backdrops whose
  authored presence follows and whose authored placement precedes the relevant
  portal. Both dialogs have real title/content/actions, conventional Cancel/
  primary placement, topmost-first dismissal, and no adapter-made controls or
  icons;
- native pixels and the independent oracle prove cumulative darkening at stable
  background and exposed-underlying-dialog control points, no self-dimming of
  the top dialog, exact restoration after topmost close within the qualified
  tolerance, and order-sensitive colored-layer behavior; transparent-layer
  input tests separately prove that paint opacity never owns shielding;
- host-neutral proofs admit a modal with no backdrop and a non-modal portal with
  a backdrop, reject cyclic/ambiguous/cross-surface placement, and prove that
  portal kind never auto-creates or rejects backdrop appearance;
- old static-paint symbols, modules, translators, protocol rows, fixtures,
  public exports, `portal_icon_text`, bootstrap token dependencies, and direct
  token publication are absent from the production path;
- no selector, cascade, ambient theme, generic property bag, adapter style,
  second publisher, or hidden whole-graph fallback exists;
- exact-zero appearance, pointer, theme-switch, mounted, host, and inherited
  service resources are observed on clean shutdown;
- all named continuing docs and the one app-author appearance guide agree with
  real code and compiled/lowered examples; and
- repository boundary, generated-context, app-inclusive line-cap,
  feature-matrix, protocol, link, formatting, deletion-inventory, and required
  closure-stress checks pass.

The successor handoff is exact:

- Milestone 3.17 may produce pure, aspect-tracked inputs for role selection or
  state facts; it cannot add a hidden evaluator inside appearance;
- Milestone 3.19 may make appearance denials and causal reports richer; it
  cannot turn diagnostics into resolver or switch authority;
- Milestone 3.20 consumes mounted visual bounds and appearance provenance for
  invariants; pixels remain secondary;
- Milestone 3.22 may trial one selected appearance through an affine canonical
  replacement and propose an exact source edit; it cannot add an inspector-only
  style overlay or cascade;
- Milestone 9 may add themes, density, component roles, and professional
  widgets through the frozen registries and facade; it cannot reopen theme,
  state, or host authority as design-system folklore;
- Milestone 13 may require accessibility/high-contrast coverage and semantic
  state projection without allowing color alone to carry accessibility truth;
  and
- Milestone 15 may add plugin-owned definitions with capability, provenance,
  precedence admission, and unload generation, never arbitrary global
  overrides.

If any successor requires moving the appearance facade, splitting a style
manager, inventing ambient inheritance, replacing the resolver, or teaching a
host to understand roles/state/theme meaning, Milestone 3.16 is not complete.
