# Milestone 3.16.2: Continuous Window Resize and Responsive Containers

## Goal and placement

Dragging a native window edge must continuously update a usable responsive
application before the mouse is released. Containers redistribute space, text
reflows where necessary, and scrollbars, hit targets, overlays, and borders remain
coherent. This follows [3.16.1 scrolling](./milestone-3.16.1.md), preserving
[3.16](./milestone-3.16.md) appearance and acceptance authority. Milestone 4 retains
workspace docking, persisted layouts, split-handle editing, and multi-window work.

Currently Pulse registers almost every dashboard allocation as a fixed rectangle;
only its root fills the viewport. Native resize replaces a retained target at each
size event before redraw requests coalesce. Existing viewport measurement and
mounted layout completion are the integration path, not proof of continuous
resize. Neither a scaled screenshot nor correct geometry only after release closes
this milestone. Measure callback/layout/GPU costs before choosing private optimizations.

## Decisive production proof

Drive the existing native Pulse through actual OS border dragging with the button
held. Move width and height independently, then diagonally and rapidly reverse.
Cross the responsive breakpoint in both directions, from 1536x1024 through
1120x800 to 800x600 and back. Observe several intermediate accepted extents and
external client captures before release; repeated programmatic resize calls prove
only their narrower path, not Windows interactive resize-loop behavior.

Run one readable dashboard journey with scrolled lists and a focused control;
open a modal/popover for a separate resize segment. Verify stationary chrome,
readable text, retained anchors, correct pointer targets, and relational overlays.
Cross the breakpoint with a scrolled region and focused/Portal-anchored child;
their mounted identities, Scroll state, focus, and anchor binding must survive.
Use focused production-handoff cases for resize during pending presentation,
rejection/retry, minimize/restore, DPI change, owner reincarnation, and close.
Include `resize A -> pointer/button or wheel input -> resize B`; that input must
retain its lawful geometry basis and cannot be silently routed in B's geometry.
One valid scale/locality proof protects unrelated surfaces; do not replace the
existing 512-instance fixture or force every case into one process lifetime.

An independent geometric oracle derives expected slot rectangles from viewport,
declared tracks, gutters, and constraints. External pixels corroborate selected
intermediate layouts. The proof must expose delayed-until-release painting,
per-event obsolete allocation, stretched prior frames presented as new layout,
stale viewport commitment, mixed text/hit geometry, and incorrect overlay order.

## Product and geometry decisions

- Support a dashboard minimum client size of 800x600 logical points and ordinary
  enlargement through at least 1920x1200. Host minimum-size hints are not validation:
  admit unexpected smaller extents safely with clipping, never negative geometry
  or process failure. Full content reachability below the supported floor is not
  claimed. Zero extent suspends physical presentation.
- Keep the sidebar at 236 points, the main gutter at 24, and card gaps at 20.
  At client width >=1200, main content uses flexible left and right columns with
  2:1 remaining-space weights and minimum widths 480 and 320. Below 1200, stack
  chart, Service health, Recent activity, and Deployments in source-declared order.
  Summary cards use four columns at >=1200, two below; their contents retain padding.
  The sidebar footer anchors to the bottom; the header fills available width.
- Height constrains the page viewport, not font scale. The main page scrolls when
  its declared minimum content height exceeds available height. Service health and
  Recent activity retain independent region scrolling, inherited remainder policy,
  and thumb derivation. A height-only change cannot arbitrarily resize text glyphs.
- Use explicit fixed, bounded flexible, and content-measured tracks with declared
  gaps and child-slot membership. Allocate fixed/minimum extents first, distribute
  remaining space by weights with deterministic cap redistribution, then apply
  declared overflow. Insufficient space preserves minimum content extents and
  creates overflow; it never invents negative sizes or rescales the whole UI.
- Logical geometry keeps fractional precision. Shared edges are quantized once
  at the owning physical boundary; independently rounded neighboring cards must
  not leave gaps or double borders. Preserve half-open bounds and seam ownership.
- Text wraps/ellipsizes according to its declared contract at the actual allocated
  width. Reuse shaping when its constraints are equal; moving a label alone cannot
  reshape it. Icons, radii, and typography retain logical sizes through resizing.
- Modal cards stay centered within the usable viewport, with 24-point minimum
  insets, bounded width, and scrollable content when height is constrained. Popovers
  remain anchored with existing fit/flip/clamp policy. Backdrops use current extent;
  topmost shielding, focus restoration, and relational paint order stay intact.
- Direct resize does not ease behind the window. Each accepted frame uses one
  coherent current extent; it does not animate the dashboard toward an obsolete size.
  In-progress Scroll motion reconciles its bounds/anchor using 3.16.1; accepted pixels
  remain the retarget origin. Dashboard breakpoint selection changes track/slot
  allocation only. Surviving children retain mounted identity/incarnation, semantic
  parent, region/Scroll ownership, source order, focus, and Portal anchor binding.
  Reparenting or remounting to implement the column-to-stack transition is invalid.

## Ownership and phase contract

The native host owns observed client physical extent, scale, window lifecycle,
swapchain/target resources, and redraw scheduling. Layout/Mosaic owns structural
allocation and breakpoint selection in logical space. Scroll, text, Portal,
appearance, and interaction consume that exact mounted geometry. A host cannot
select columns, fabricate a measurement receipt, or turn a stretched bitmap into
an accepted layout. An application cannot own a raw resize event loop.

Native size delivery records the newest qualified extent and requests redraw.
Coalesce obsolete positive-size samples before expensive target preparation, using
the existing readiness owner: at most one pending latest extent and one active
prepared successor per surface. Do not coalesce away close, zero-size suspension,
binding changes, or DPI transitions. Geometry-dependent pointer/button/scroll
observations are also coalescing barriers: consume them in order against their
lawful observed and accepted presentation bases before replacing pending size
evidence. A barrier does not require presenting an unaccepted extent merely to
route input. Later size evidence cannot relabel or invalidate that input. Rendering
must progress within the Windows interactive sizing loop; normal idle callbacks
alone are insufficient. Keep callbacks bounded and preserve the existing host
thread-affinity and physical completion contracts.

At redraw, consume the qualified viewport basis, prepare one layout and affected
resource successor, then admit all dependent presentation results. The downstream
phase carries exact extent/scale/binding/generation, occurrence geometry, text
constraints, Scroll succession, Portal/Backdrop placement, and retention admission.
Reuse paths consume dependency equivalence; assembly alone grants no presentation.
Retain device, pipelines, and unchanged resources across ordinary size changes;
allocate extent-dependent targets only for consumed work. No hidden readback,
busy loop, source interpretation, Query evaluation, or replay on ordinary resize.

Acceptance commits those prepared results and retires predecessor resources only
after physical completion. Pre-effect rejection preserves accepted application
state, retains coherent retry/newer-extent handling, and never marks the failed
extent presented. A newer pending observation is not cleared by an older frame's
completion. No obsolete prepared candidate may newly submit after supersession;
already in-flight effects settle honestly before the latest pending extent runs.
Indeterminate effects use existing reconciliation and forbid predecessor rollback
claims. Minimize suspends without spinning; restore prepares the latest nonzero
basis. Close cancels pending work and releases each resource exactly once.

Public layout authoring extends registered Mosaic meaning through
`worth_ui::facade::declaration`, then canonical Rust/file lowering. Proposed shape:

```rust
let columns = MosaicLayoutContract::columns([
    MosaicTrack::fixed(236)?,
    MosaicTrack::flex(1, 480)?,
])?.with_gap(24)?;
let page = page.with_layout(columns);
```

Tracks address declared child slots; responsive variants attach to the same layout
declaration using ordered, nonoverlapping viewport-width intervals and a required
fallback. They contain no executable application closure. Canonical validation
rejects missing/duplicate slot membership, nonfinite/negative extents, zero weights,
inconsistent min/max, overlapping intervals, and recursive sizing dependencies.
Measurement requirements lower before execution; unavailable measurements produce
typed pending/denial rather than a guessed size. The code example is proposed API,
not an assertion that these methods already exist; implementation supplies a real
compiling author example and equivalent `.wui` lowering.

## Destination topology

Paths are relative to `workspaces/worth-ui`. Reuse existing owner directories;
create only the populated responsibilities needed for this contract.

```text
crates/worth-ui-dsl/src/                         [extend canonical layout declarations]
crates/worth-ui-runtime/src/
  capability/registry/{mosaic_region,mosaic_sizing}/
                                               [extend admitted tracks/constraints]
  runtime/planning/                             [extend existing executable layout lowering]
  runtime/mosaic/layout/                        [new track allocation, consuming that plan]
  runtime/viewport_resize/                      [extend existing extent commit basis]
  mounting/occurrence_geometry/                 [extend exact viewport succession]
  facade/entry/native_application_shell/viewport_measurement.rs
                                               [extend consumed/latest basis settlement]
  facade/entry/active_application_session/      [extend cross-owner preparation]
  mounting/presentation/                        [extend extent-affine acceptance/retry]
crates/worth-ui-host-native/src/native/
  event_loop/resize/                           [new bounded observed/pending resize work]
  lifecycle/surface_succession.rs               [extend consumed target replacement]
  graphics/backend/                            [reuse device and extent-dependent resources]
apps/platform-pulse/src/
  application/mosaic.rs                        [extend responsive product declarations]
  native_application/layout.rs                 [replace dashboard rectangle orchestration]
```

The new track allocator consumes existing executable region/sizing meaning; it
does not create a parallel planner or replace viewport-resize commit authority.
Layout is private runtime allocation, not appearance or host policy. New pure
canonical vocabulary remains with DSL/capabilities; live revisions/leases remain
runtime-owned. Milestone 4 adds docking and persisted workspace constraints beside
this layout owner without moving public facades. Remove the displaced dashboard
fixed-coordinate path and per-event target replacement after the production
cutover. Keep legitimate fixed-size icons, typography, and direct primitive
allocation contracts. No second layout tree, compatibility renderer, global
container registry, or app-owned breakpoint calculator may survive.

## Delivery sequence and acceptance

**1. Reach continuous presentation.** Trace and measure one real border drag, fix
the existing host-to-layout-to-presentation path, and show intermediate correctly
laid-out Pulse frames while held. Include one flexible container and its actual
text/hit geometry immediately. Do not end this phase at event delivery or receipt
publication. This is the endpoint used to drive the remaining work.

**2. Complete responsive composition.** Apply declared tracks and responsive variants
to the whole dashboard. Finish scroll extent/anchor succession, text constraints,
modal/popover placement, and Backdrop extent through the same preparation boundary.
No stretched-canvas substitute or separate initial/rebind/retry layout formula.

**3. Harden and qualify.** Cover delayed completion followed by a newer extent,
pre-effect rejection/retry, scale transitions, minimize/restore, reconstruction,
and shutdown in focused cases. Admission denial must not partially update owners.
Use the existing compile-fail target only if the actual readiness boundary changes;
keep one incomplete-frame denial and its valid counterpart, not a phase matrix.

Use the 3.16.1 recorded Windows 60 Hz release qualification environment. Perform
three 10-second active edge drags across the stated sizes, including reversal and
breakpoint crossings. Require p95 native size-event-to-first-matching-frame <=50 ms,
p95 accepted visible-frame gap <=25 ms, p99 <=50 ms, no unexplained active gap
>100 ms, and final exact extent publication <=100 ms after release. Measure
intermediate extents at submission time against the latest consumed observation;
legitimate coalesced samples are not required to render. Capture real presented
frames before release, not only application callbacks. Report raw timing intervals,
hardware/DPI, layout/text work, target allocations, and peak retained resources.
Do not exclude slow breakpoint frames or claim 120 Hz performance from this run.

During resize, work may scale with affected layout dependencies and physically
changed visible commands. It must not scan unrelated windows or mounted regions,
recompile source, or reshape unchanged-width text. Pending work/resources remain
bounded independently of raw resize-event count; reclamation respects GPU completion.
After settling, no duplicate frame loop or residual resize wake remains.

Reuse valid scrolling, rounded-border/seam, text, theme, overlay, locality, and
cleanup results unless changed machinery invalidates them. Test semantic parity
through headless and selected physical native paths, without replaying every case
on both hosts. Run affected tests, warning-free default and demo-feature builds,
formatting, dirty line caps, boundary-check, and agent-context check. No suppressed
warnings, fresh runner, progress ledger, or blanket legacy-fixture rewrite.

Implementation updates the same three continuing guides named in 3.16.1:
`docs/appearance-and-themes.md` for author layout/chrome behavior and examples,
`docs/runtime-subsystems.md` for allocation/owner succession, and
`docs/native-host-platform.md` for live resize, DPI, lifecycle, and timing contracts.
Reviewers must explicitly check skipped preparation, stale latest-extent clearing,
and state/pixel mismatches. Closure requires the full responsive demo, live-drag
evidence, recovery, bounded resources, and no material unresolved review finding.
