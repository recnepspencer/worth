# Appearance And Themes

## What Authors Declare

An appearance role gives visual meaning to one component kind. A role covers a
finite set of aspects—background, foreground, border, radius, opacity, outline,
and pointer affordance—with disjoint cells over supported state axes. A role is
not a component, and components do not gain adapter styling when no role is
attached.

A theme-slot catalog declares each slot identity and value kind. Every admitted
theme definition supplies a compatible value for every required slot. The
prepared application selects one initial definition and issues a
surface-scoped capability. Literal cells remain literal; they do not perform a
theme lookup.

The runtime combines these inputs in order:

```text
component attachment + role + coherent UI-owned state
-> selected finite cell
-> admitted theme value or authored literal
-> concrete mounted occurrence geometry
-> accepted Motion sample and Portal/Backdrop relation
-> prepared presentation work
-> host acceptance or retained predecessor
```

Presentation cannot consume the assembled frame before appearance, owner-state
succession, Motion/overlay composition, and retention admission have completed.
Acceptance commits those exact prepared results. Rejection preserves predecessor
pixels and owner state, and retry consumes the returned evidence.

## Rust Authoring

Use `worth_ui::facade::appearance`. This shortened form is taken from the
compiled Platform Pulse registration in
[`appearance.rs`](../apps/platform-pulse/src/application/presentation/appearance.rs):

<!-- compiled-example:appearance-role-rust -->
```rust
let role = UiAppearanceRole::authoring(
    UiAppearanceRoleIdentity::new("app.appearance.card")?,
)
.applies_to_component(UiDslComponentReference::new("app.component.card")?)
.cover(
    UiAppearanceAspect::Background,
    UiAppearancePartitionAuthoring::new([]).with_cell(
        UiAppearanceCell::when([]).uses_slot(
            UiThemeSlotIdentity::new("app.theme.surface")?,
            UiThemeValueKind::Color,
        ),
    ),
)?
.build()?;
```

Register the role and one `FrozenAppearanceThemeCapabilities` bundle on the
application builder. Build the catalog first, admit complete definitions against
that catalog, then admit the bundle with its initial definition. The live Pulse
theme construction is compiled in
[`theme.rs`](../apps/platform-pulse/src/application/presentation/appearance/theme.rs).

Component backgrounds also accept `UiThemeValue::LinearGradient`: two colors and
distinct `UiThemeGradientPoint` endpoints in the 0–10,000 allocation-relative
coordinate range. Declare its slot as `UiThemeValueKind::LinearGradient`; in WUI,
use `background use linear_gradient token(...)`. Hover may select a separate
solid-color slot, but one slot cannot claim both kinds. Foreground and Backdrop
backgrounds do not accept gradients.

Unknown roles, duplicate or overlapping partition cells, incomplete partitions,
wrong-kind slot values, missing slots, incompatible successor definitions, and
wrong-component attachments are typed denials. Invalid source edits retain the
last accepted application and paint.

## State And Authority

Operability, focus, validation, selection, hover, and pressed meaning remains
owned by the existing UI subsystem for that axis. Appearance consumes a sealed,
coherent snapshot and selects a cell. It cannot mutate an owner, admit an
interaction, or make an inoperable action available.

State is scoped to concrete mounted occurrences and surfaces. A changed axis
uses the existing consumed-fact indexes to select affected occurrence
neighborhoods. An unrelated copy of the same role on another surface does not
become affected merely because it consumes the same axis. Replacement uses
owner-issued succession so cleared pressed or pending-operability state is
reflected in the first accepted successor frame.

## Layout Containers

A container component declares where its members stand as a Mosaic layout on
its descriptor. Tracks are fixed, flexible with a weight and a minimum, or
flexible with a minimum and a maximum. Allocation gives fixed tracks and
minimums their extent first, then divides the remaining space by weight,
redistributing past any maximum. When space runs short, minimums hold and the
container overflows; nothing gets a negative size and the UI is never
rescaled. Gaps and padding are logical points; geometry keeps fractional
precision until the physical boundary quantizes shared edges once.

A responsive layout selects its tracks by viewport width. Variants hold
half-open, nonoverlapping width intervals over a required fallback, and every
variant places the same members. Crossing a breakpoint moves members between
cells; it never remounts or reparents them, so identity, focus, Scroll
ownership and Portal anchors survive. This shortened form compiles in the
certification guide test:

<!-- compiled-example:container-layout-rust -->
```rust
let wide = MosaicLayoutContract::grid(
    [MosaicTrack::flex(2, 480)?, MosaicTrack::flex(1, 320)?],
    [MosaicTrack::fixed(66)?, MosaicTrack::flex(1, 348)?],
)?
.with_gaps(20, 20)
.with_padding(24, 24)
.with_member(title.clone(), MosaicLayoutCell::spanning(0, 0, 2, 1)?)?
.with_member(chart.clone(), MosaicLayoutCell::at(0, 1))?
.with_member(health.clone(), MosaicLayoutCell::at(1, 1))?;
let stacked = MosaicLayoutContract::rows([
    MosaicTrack::fixed(66)?,
    MosaicTrack::flex(1, 348)?,
    MosaicTrack::flex(1, 348)?,
])?
.with_gaps(20, 20)
.with_padding(24, 24)
.with_member(title, MosaicLayoutCell::at(0, 0))?
.with_member(chart, MosaicLayoutCell::at(0, 1))?
.with_member(health, MosaicLayoutCell::at(0, 2))?;
let page = page.with_layout(
    MosaicResponsiveLayout::new(stacked)
        .with_variant(MosaicViewportWidthInterval::at_least(1200), wide)?,
);
```

The same declaration can be restated in `.wui` source. The top-level block
is the fallback and each `width from <min> [to <max>]` block is a variant:

```text
layout app.component.page {
  columns flex 1 min 480;
  rows fixed 66, flex 1 min 348, flex 1 min 348;
  gap 20 20;
  padding 24 24;
  member app.component.title at 0 0;
  member app.component.chart at 0 1;
  member app.component.health at 0 2;
  width from 1200 {
    columns flex 2 min 480, flex 1 min 320;
    rows fixed 66, flex 1 min 348;
    gap 20 20;
    padding 24 24;
    member app.component.title at 0 0 span 2 1;
    member app.component.chart at 0 1;
    member app.component.health at 1 1;
  }
}
```

Rust authors state the same block with
`WorthUiRustAuthoredArtifactInputModule::with_layout`, using the
`worth_ui::facade::layout` vocabulary. The runtime lowers a block through the
Mosaic constructors and admits it only where it equals the layout its
container registered. A disagreement, a duplicate block, or an unregistered
container is a typed `UiAuthoredLayoutDenial`, and neither source wins.
Platform Pulse restates its page in
[`dashboard_layout.wui`](../apps/platform-pulse/app/dashboard_layout.wui).

A container that owns Scroll lays its members out at their minimums and
scrolls them when they outgrow its viewport. Height constrains that viewport,
not type size. Modal cards stay centered in the current viewport, and open
popovers are placed again from each frame's layout with the fit, flip and
clamp policy they opened with. A Portal opened fitted to its content fits the
extent each frame lays that content out at, not the extent it opened at.
Backdrops cover the current extent.

## Geometry, Borders, And Text

Appearance resolves against mounted occurrence rectangles. A region-level seam
contract identifies the owner of an actual shared edge. Completion intersects
concrete region occurrences, translates the overlap into owner-surface local
coordinates, and omits a surface border only when that regional edge coincides
with the corresponding exterior surface edge. Multiple regions may aggregate
applicable exterior omissions for one owner. Internal seams do not erase an
unrelated outer border.

Rounded borders use the surface's resolved corner radii and keep border coverage
through all four curved corners. Visual bounds include outline and anti-alias
fringes; rectangles and damage use half-open edges.

A semantic text span opts into its node's role foreground with
`with_appearance_foreground()`. The original UTF-8 range, style, and paint
identity remain authoritative. Content edits, authored succession, partial
surface acceptance, detached retry, and reconstruction settle the exact text
revision per occurrence. Intrinsic-color glyphs retain their own color; Motion
and appearance opacity still compose over the result.

A semantic text contract declares its flow with `with_flow`.
`ComponentSemanticTextFlow::wrapping()`, the default, breaks at word
boundaries and clips what its box cannot show.
`ComponentSemanticTextFlow::single_line_ellipsis()` keeps one line and ends it
with an ellipsis. Both are shaped at the width layout allocates, so resizing a
container rewraps or re-ellipsizes its text. Box height is not part of the
flow: it clips the shaped lines but never decides how many there are, so a
height-only change, or moving a label, reuses its shaping. Pulse card labels
ellipsize; panel copy wraps.

Scroll clipping does not discard offscreen glyph images or their adopted
foreground. Intrinsic text clipping travels with the text; ancestor viewports
remain stationary. The accepted sample derives their intersection and snaps
presentation to the device grid. Explicit content suppression, unlike empty
viewport coverage, retires the text. Rejection restores the predecessor sample
and its coverage rather than acknowledging the refused foreground or pose.

## Backdrops, Portals, And Motion

A Backdrop is independently authored. Its declaration states identity, scope,
extent, presence basis, placement, appearance role, and optional Motion basis.
Portal state may provide lifecycle and issued order, but a Portal does not
implicitly create or reject a Backdrop.

Overlay order is relational. Authored before/after relations and Portal-issued
stack order decide composition. Raw z-index, source order, object identity, and
arbitrary numeric ranks do not resolve ambiguous overlap. Equal-order overlap
without a lawful relation is denied.

Use `place immediately_above_surface_content` for a scrim that dims the dashboard
while leaving all presented Portals above it. `place above_surface_content` only
states precedence and still denies an ambiguous order relative to Portals.
Modal input shielding follows the accepted Portal-issued stack order, including
sibling Portals at the same nesting depth; a visible panel below a modal does
not gain interaction authority merely because it is painted above the scrim.

Motion owns committed tracks and accepted samples. Appearance composes the exact
accepted command sample during prepared presentation; it never turns a visual
sample into layout or interaction authority, and it never feeds appearance
opacity back into Motion's retarget predecessor.

## Scroll Chrome

A scrollable region's track and thumb are painted by declared roles and placed
by the runtime. An author says which axes get bars and which two roles paint
them; the runtime derives both rectangles from the accepted offset and the
content extent. There is no authored thumb position, and reading one back out
of chrome facts to compute another is the same second data path a renderer-side
field lookup would be.

Chrome roles are painted beside a region rather than on a component inside it,
so they must not attach to a component: leave `applies_to_component` off and
they apply to any. A role a contract names is required by the theme binding
like any other, so both must be registered before the region that names them is
mounted, and a region that does not own its scroll derives no chrome at all.

<!-- compiled-example:scroll-chrome-rust -->
```rust
let track_identity = UiAppearanceRoleIdentity::new("app.appearance.scroll_track")?;
let thumb_identity = UiAppearanceRoleIdentity::new("app.appearance.scroll_thumb")?;
let thumb = UiAppearanceRole::authoring(thumb_identity.clone())
    .cover(
        UiAppearanceAspect::Background,
        UiAppearancePartitionAuthoring::new([]).with_cell(
            UiAppearanceCell::when([]).uses_slot(
                UiThemeSlotIdentity::new("app.theme.scroll_thumb")?,
                UiThemeValueKind::Color,
            ),
        ),
    )?
    .build()?;
let region = MosaicRegionKindDescriptor::new(
    MosaicRegionKindId::new("app.region.list")?,
    MosaicRegionRole::auxiliary(),
)
.with_scroll_ownership(MosaicScrollOwnership::region_owned())
.with_scroll_line_extent(UiScrollLineExtent::logical_points(20)?)
.with_scroll_chrome(UiScrollChromeContract::new(
    UiScrollAxisSupport::Block,
    track_identity,
    thumb_identity,
)?);
```

Partitioning a chrome role on Hover and Pressed gives a resting bar, a bar under
the pointer and a thumb being dragged three distinct declared appearances. A
drag that leaves the gutter is still a drag, so `PressedCapturedOutside` and
`PressedArmedInside` normally resolve to one held cell. Hover and Pressed here
mean what they mean everywhere else; appearance reads the axis, it does not
decide it. The live construction is compiled in
[`scroll_chrome.rs`](../apps/platform-pulse/src/application/presentation/appearance/scroll_chrome.rs).

Declaring chrome on an axis a region cannot travel on reserves a gutter for a
bar that can never move, so name only the axes whose content actually overflows.

During easing, the thumb and content follow the same accepted sample. Thumb
clipping is against the stationary viewport, not the thumb's previous bounds.
Hover is re-resolved when accepted content moves beneath a stationary pointer;
appearance does not require a synthetic pointer event to catch up. Captured
dragging and extent changes publish their exact prepared geometry through the
same host-acceptance boundary as other mounted changes.

## Live Theme Switching

Obtain the current surface, predecessor presentation, and
`UiThemeCapabilityReceipt`, then prepare a `UiThemeSwitchRequest` through the
active session or native shell. The selected definition must belong to the
current bundle and be compatible with its catalog. Preparation resolves only
consumers of slots whose terminal values changed. Unused changed slots produce
validated no-work evidence.

The switch publishes through ordinary mounted presentation. Host rejection or
cancellation keeps the predecessor theme and pixels. Acceptance installs the
same prepared binding that produced the successor pixels. Source replacement,
retry, and reconstruction preserve that rule.

## Inspection And Cost

Use `why_appearance` to inspect the role, selected cell, theme source, coherent
state basis, mounted attribution, relevance, and expiry. Theme-switch summaries
explain changed slots and affected consumers. Inspection is read-only and cannot
be converted into resolution or publication authority.

Ordinary invalidation is proportional to changed facts, slots, and affected
mounted neighborhoods. It does not scan every consumer or remount the graph.
Complete reconstruction is an explicit recovery path with separate cost.

## Current Limits

`worth-ui-global-text-v2` remains staged and is not the live text profile. Icons
await the Milestone 9 host mechanic and must not be represented by text glyphs.
Mounted preview remains the single preview lane. Scroll chrome paints a track
and a thumb only; there is no declared arrow, corner or gutter-only appearance,
and a chrome role cannot be attached to a component. `worth-cert-ui` is neither a
workspace crate nor a certification owner.

A `.wui` layout restates a registered layout; it cannot yet be the sole
source, because registration checks membership and containment against it.
Layout tracks are whole logical points. An ellipsis needs a qualified U+2026
glyph in the font collection; without one, shaping is denied with
`EllipsisRequiresQualifiedGlyph`. A container's content extent is measured
when it opens, not again while it stays open.

## Anti-Patterns

- Selecting colors or state styles in a renderer or host adapter.
- Treating a component name as an implicit role or theme slot.
- Computing breakpoints or container rectangles in an application, or
  remounting members to move them between columns.
- Computing a scroll thumb rectangle in an application or reading one back
  from chrome facts to place anything.
- Reading mutable service state while resolving appearance.
- Applying region-relative seam offsets directly to a surface allocation.
- Ordering overlays by raw z-index, source order, identity, or arbitrary rank.
- Publishing assembled output before required prepared evidence exists.
- Acknowledging text or owner state for a surface the host rejected or omitted.
- Reconstructing operational meaning from inspection output or pixels.

## Related Docs

- [Authored composition](./authored-composition.md)
- [Application lifecycle](./application-lifecycle.md)
- [Hot rebind](./hot-rebind.md)
- [Runtime services](./runtime-services.md)
- [Text platform](./text-platform.md)
- [Native host platform](./native-host-platform.md)
- [Application inspection](./inspection.md)
- [Visual inspection](./visual-inspection.md)
