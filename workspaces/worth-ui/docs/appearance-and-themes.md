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

## DSL Authoring

The DSL lowers to the same sealed role and attachment meaning. This excerpt is
compiled as part of the real Pulse source journey:

<!-- compiled-example:appearance-role-dsl -->
```text
appearance role platform.pulse.appearance.query_card
    applies_to platform.pulse.component.query_card {
  background use token(theme.platform_pulse.raised_surface)
  border use token(theme.platform_pulse.border.query_card)
  radius use token(theme.platform_pulse.radius.query_card)
}

component platform.pulse.component.query_card {
  appearance { role platform.pulse.appearance.query_card }
}
```

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
Mounted preview remains the single preview lane. `worth-cert-ui` is neither a
workspace crate nor a certification owner.

## Anti-Patterns

- Selecting colors or state styles in a renderer or host adapter.
- Treating a component name as an implicit role or theme slot.
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
