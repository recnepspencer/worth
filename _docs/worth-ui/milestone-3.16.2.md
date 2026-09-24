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

## Requirements 1-4: presentation truth is compiler-enforced

Resize multiplies the moments at which the host, Motion, Scroll, hit testing,
and the cursor can disagree about what is on screen. Native Pulse defects found
after 3.16.1 had that shape and compiled cleanly. A Scroll settle could not see
an in-flight retarget's presented frame because presence was a boolean beside
its geometry. A Scroll group rebuilt before its settle counted the published-to-
accepted gap twice because both offsets were bare `[f64; 2]`. Those defects are
fixed; these requirements make their class uncompilable instead of reviewable.

Requirements 1-4 complete before delivery step 1 and are taken to completion,
not piloted. They cover every owner that produces or consumes presented truth:
runtime presentation, Motion, Scroll, occurrence and viewport geometry,
interaction, hit testing, Portal anchoring, pointer affordance, the host
contract, native and headless hosts, certification support, and Pulse. They
realize arch laws 9, 10, and 16. Each converted surface replaces its raw form;
no alias, wrapper, compatibility lane, or untyped path survives beside it.

**1. Geometry carries its truth status.** Published, accepted (sampled and
admitted for presentation), and displayed (host-confirmed) offsets,
translations, extents, and rectangles are distinct sealed types. They differ in
truth status, so law 10 forbids phantom tags on one shared array. Arithmetic is
defined only within one status. A translation names its source and destination
status. Crossing statuses requires a named conversion that consumes the owner
evidence proving the relation: an acceptance, a settle receipt, or requirement
3's presentation witness. A value captured at bind or prepare time carries the
basis it was captured against, so combining it with a value on another basis
does not compile. Raw coordinate arrays and scalar pairs remain only at
declared serialization, platform-event, and GPU-upload edges.

**2. Lifecycle truth is a sum type without defaults.** A flag or option that
qualifies other state becomes an enum whose variants own their data. For
example, a Motion track is `Unpresented { .. }` or `OnScreen { geometry, .. }`.
The same applies to Scroll settles, hover and cursor targets, prepared and
accepted presentations, pending extents, and retiring resources. These types
have no `Default`, no boolean-plus-optional pairs, and no constructor that picks
a variant silently. Every construction and fork site names its variant, so a
new variant or field breaks each site until it is propagated (law 9). Review
converts or explicitly justifies every `bool` and `Option` field in the covered
owners that qualifies another field.

**3. Only the host can say a frame is on screen.** A presented-frame witness
(name illustrative) is minted only by host physical-completion acknowledgement
for one surface, generation, and presentation basis. Its constructor is sealed
against every other crate. Scripted and headless hosts mint it through the same
boundary; certification gets no shortcut. Everything that commits displayed
truth consumes the witness:

- Motion commit, Scroll settle and offset write-back, and next-sample damage;
- hit testing, pointer affordance, and cursor selection;
- focus and Portal anchor retention, and predecessor resource retirement.

Values derived from a witness are scoped to its surface and generation. Using
one against a later generation requires that owner's rebase. Placement follows
the AGENTS.md three-question rule and is recorded before design.

*Placement record.* The three questions place each type as follows:

- **Host acknowledgement: `UiMountedSurfacePresentationCompletion` in
  `worth-ui-host-contract`.** It reports a physical fact and grants no
  permission, so it does not belong in `worth-proof`. It carries the runtime's
  live presentation-lease seal and the exact issued attempt, requirement, and
  frame. An attempt is minted per issuance, so those three name one piece of
  issued work and its content. That means it cannot exist without a live table,
  so it is not foundational vocabulary. It lives in the UI runtime's own host
  boundary, which already owns the issued view and token it is minted from. It
  has no constructor: it exists only as the acknowledgement of a view or token.
  A view built outside the runtime carries a seal no live lease issued, so
  admission refuses whatever it acknowledges; forged work opens no doors.
- **Witness and displayed basis: `UiPresentedSurfaceWitness` and
  `UiDisplayedSurfaceBasis` in `worth-ui-runtime` mounting presentation.** Both
  are `pub(crate)`. Admission needs the live lease and the issued work, so they
  belong to the owning runtime.
- **Displayed truth.** Only an admitted witness produces a displayed basis. Every
  writer of displayed truth consumes the witness: Motion commit and acceptance,
  retention epochs, receipts, and the Scroll settle. The Scroll settle takes the
  surface a witness committed, never a displayed basis; a settle it defers keeps
  that surface, one owed settle per semantic surface, and is paid there while
  the host still shows its generation, or released when it does not. A Motion
  tick pays what is owed: every surface's settle when it commits no pixels, and
  every other surface's once its own committed settle lands. Readers read the retained record that the witness
  wrote. A basis a host reports on an observation only selects which record to
  read.
- **Scripted, headless, and certification hosts.** They acknowledge the view or
  token they were issued. Certification witnesses run the same issue,
  acknowledge, and admit path. Evidence that measures the Motion sampler alone,
  with no session under test, admits through an isolated session of its own.

**4. Owner changes record themselves.** In-flight work lands against state that
may have changed after it was prepared, and each such change is reconciled at
landing. This covers owner installs, retirements and rebinds during a Motion
tick, newer pending extents, and host presentation-lifecycle transitions. That
state changes only through an owner edit handle that records the change as part
of the mutation; its fields are private and no mutation bypasses the record.
Landing consumes the record with an exhaustive match, so a new change kind does
not compile until it is reconciled. This replaces every hand-called
change-tracking path, such as `note_owner_change` and `rebound_since_prepare`.

**Enforcement and acceptance.**

- Boundary-check forbids three things in the covered modules: raw coordinate
  representations outside declared edges, construction of requirement 1 and 3
  types outside their owners, and `Default` on requirement 2 state. Clippy
  `disallowed_types` and `disallowed_methods` cover what boundary-check cannot.
- Compile-fail cases prove one forbidden move for each requirement, each with a
  valid counterpart:
  - adding values across statuses;
  - claiming `OnScreen`, or minting a witness, outside the host;
  - mutating reconciled state without the edit handle.
- A deterministic interleaving model test mixes prepare, owner edit, rebind,
  retarget, commit, pre-effect rejection, and resize. After every step it
  asserts that displayed Motion, Scroll, hit-test, and cursor geometry equal the
  latest witness's geometry. Debug builds assert the same invariant at every
  commit.
- The two post-3.16.1 Scroll regressions keep their focused tests, and both
  must fail when their conversion is reverted.
- Closure requires total conversion across the covered owners. A remaining raw
  lane is a material review finding.

## Decisive production proof

Drive the existing native Pulse through actual OS border dragging with the button
held. Move width and height independently, then diagonally and rapidly reverse.
Cross the responsive breakpoint in both directions, from 1536x1024 through
1120x800 to 800x600 and back. Observe several intermediate accepted extents and
external client captures before release; repeated programmatic resize calls prove
only their narrower path, not interactive border-drag behavior.

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
must progress while the native border drag is held; normal idle callbacks
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

Qualify on a 60 Hz display on a supported desktop OS. Retain three timestamped
10-second active edge-drag traces across the stated sizes, including reversal
and breakpoint crossings. Each trace correlates native size observations,
consumed extents, frame submission and acceptance, and independently captured
visible client pixels on a common monotonic time basis. The capture mechanism
may differ by OS, but callback logs, offscreen renders, and captures taken only
after release cannot substitute for presented frames observed during the drag.
Require p95 consumed size-event-to-first-matching-frame <=50 ms, p95 accepted
visible-frame gap <=25 ms, p99 <=50 ms, no unexplained active gap >100 ms, and
final exact extent publication <=100 ms after release. Measure intermediate
extents at submission time against the latest consumed observation; legitimate
coalesced samples are not required to render. Report raw timing intervals,
OS/window system, capture and clock-correlation method, measured display refresh,
hardware/DPI, layout/text work, target allocations, and peak retained resources.
Do not exclude slow breakpoint frames or claim 120 Hz performance from this run.

During resize, work may scale with affected layout dependencies and physically
changed visible commands. It must not scan unrelated native surfaces or
mounted regions, recompile source, or reshape unchanged-width text. Pending
work/resources remain bounded independently of raw resize-event count;
reclamation respects GPU completion.
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
and state/pixel mismatches. Closure requires requirements 1-4 fully converted, the full responsive
demo, live-drag evidence, recovery, bounded resources, and no material
unresolved review finding.
