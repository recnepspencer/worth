# Milestone 3.16.1: Responsive Scrolling and Scrollbars

## Goal and placement

Scrolling must feel attached to input: immediate, finely controllable, steady,
interruptible, and stable under content changes. Ship functional themed
scrollbars and genuine two-axis content in the existing Platform Pulse. This
follows [3.16](./milestone-3.16.md) and precedes
[3.16.2 resizing](./milestone-3.16.2.md). It extends the existing Scroll, Motion,
Mosaic, interaction, appearance, and publication owners; it does not reopen
their accepted architecture or require the entire predecessor portfolio again.

The current path applies coarse wheel lines as immediate 40-point changes.
Pulse has only 67–75 points of vertical travel and no horizontal overflow or
scrollbar presentation. Mounted clipping and report retirement exist, but are
not evidence of smooth scrolling. Reducing a multiplier alone cannot close this.

## Decisive production proof

Start with the real Pulse executable and its ordinary source, native input,
mounted geometry, and host acceptance path. Service health remains a vertical
list. Recent activity gains meaningful trailing columns and sufficient rows
to exceed its viewport in both dimensions. Headers and neighboring panels stay
fixed. Use at least two viewport lengths of vertical content and one additional
viewport width horizontally; do not create empty overflow solely for a test.

1. Wheel slowly, then burst, reverse mid-motion, stop, and click a visible row.
   Observe intermediate accepted positions, immediate interruption, the exact
   final offset, and agreement between pixels, hit targets, and thumb position.
   Across a burst, consecutive accepted per-frame displacements must not fall to
   zero or reverse sign between two same-direction inputs; the row under the
   stationary pointer shows hover state at settlement, and a glyph row captured
   mid-transition is pixel-sharp at the device grid.
2. Scroll horizontally, diagonally with precision input, and with Shift+wheel.
   Press a moving thumb during wheel easing, then drag each thumb to both ends,
   including releasing outside the track/window. Capture must not cause a jump.
   Return to the first row and verify restored text and unchanged neighbors.
3. In a separate focused production-handoff test, reject an intermediate frame,
   retry, change content extent, remove/reincarnate the owner, and reconstruct.
   No rejected sample becomes physical truth; stale input cannot target the
   replacement owner. Shrink extent while a transition is in flight and confirm
   the pending target and accepted offset reclamp together. Shutdown leaves no
   capture, track, wake, latch, or report lease.
4. In a focused nested-routing case with a scripted phased gesture, start inside
   an inner region, drive it to its edge, and continue: the ancestor must not move
   until the gesture ends and a new one begins. Start a second gesture already at
   the inner edge and confirm it latches to the ancestor from its first sample.

Independent rectangle arithmetic and known row/column labels are the oracle.
External native captures and accepted sample identities must agree. A final
screenshot, injected offset, synthetic successful host receipt, or helper-only
curve test cannot establish the input-to-presentation claim. Failure injection
may use the existing scripted host; it does not claim native timing evidence.

## Product decisions

- **Input:** preserve subpixel precision and platform direction preferences.
  Pixel/trackpad deltas, including OS momentum, apply directly without a second
  inertia layer. Coarse wheel input uses a 120 ms ease-out transition, retargeted
  from the latest accepted sample. Further input accumulates against the Scroll
  target, not an obsolete animation endpoint. Opposite input takes control on
  the next eligible frame; no animation FIFO or per-event timer is allowed.
- **Transition continuity:** the transition is time-based against accepted sample
  timestamps, never frame-count based, so a rejected or dropped frame changes
  neither the curve nor the endpoint. A retarget starts from the accepted sample's
  position *and velocity*; it must not reset velocity to zero or produce a visible
  velocity discontinuity during a wheel burst. Use one declared curve family
  (velocity-matched cubic or critically damped settle) and reuse it for every
  retarget; the 120 ms is the settle horizon from the latest input, not a fixed
  duration restarted per event. Programmatic reveal and keyboard steps use the
  same declared transition unless reduced motion selects direct settlement.
- **Line extent:** one coarse-wheel notch moves the system lines-per-notch count
  (Windows `SPI_GETWHEELSCROLLLINES`, default 3) times the region's declared line
  extent, so Pulse moves 60 points per notch, not the current 40-point profile
  constant. The host admits the platform count as device normalization; the
  runtime supplies the declared line extent and the product. A page-per-notch
  system setting maps to the track-click page step. Fractional line deltas from
  high-resolution wheels preserve their fraction. A missing or invalid platform
  count falls back to 3 and is reported, never silently substituted.
- **Axes:** vertical-only regions ignore incidental inline movement. Both-axis
  regions preserve intentional diagonal input. Shift maps a vertical wheel to
  inline movement only when no horizontal delta is already supplied. Nested
  routing uses the declared chain and passes only lawful remainder; modal
  shielding and captured scrollbar gestures take precedence over pointer location.
- **Gesture latching:** a phased gesture (trackpad Started through Ended or
  Cancelled, including OS momentum) latches to the Scroll owner that first
  consumed it and stays there for the gesture's lifetime; remainder does not
  bubble to an ancestor mid-gesture once the latched owner reaches an edge. A
  gesture that starts at an edge in the direction of travel latches to the first
  ancestor that can consume it. Unphased coarse wheel events latch for a short
  declared quiet interval after the last consumed event, then re-resolve from
  pointer location. Latching ends explicitly on gesture end, cancellation,
  owner removal, or modality loss; it never survives owner reincarnation.
- **Edges:** the dashboard clamps without bounce. Wheel/thumb input cannot leave
  legal bounds. OS momentum phases remain explicit; momentum that reaches an edge
  is discarded, not transferred to an ancestor. Apple-style rubber-banding is not
  required here and must not be imitated with an unowned renderer offset.
- **Presentation snapping:** the semantic and accepted offsets keep subpixel
  precision, but presentation snaps each moving text command, one-point outline,
  and border to the device pixel grid at the current scale from the same accepted
  sample, so no glyph or hairline blurs during easing. Snapping is a presentation
  derivation and never feeds back into Scroll or Motion truth. Thumb geometry
  snaps the same way; hit testing uses the unsnapped accepted offset.
- **Chrome:** show a track and thumb for every overflowing enabled axis. Reserve
  a stable 12-point gutter; use a centered 6-point rounded thumb, minimum length
  24 points, a subtle track, and distinct normal/hover/drag appearance roles.
  These are Pulse's declared metrics, not hardcoded host defaults; the platform
  admits typed chrome metrics with nonnegative extents and a usable thumb range.
  The effective pointer target fills the gutter. Respect an admitted system
  always-show preference; Pulse itself keeps overflowing bars visible for testing.
  No-overflow axes have no active thumb or misleading drag target. Wheel input
  over a gutter, track, or thumb routes to that scrollbar's declared Scroll owner
  exactly as if delivered over the content. Overlay autohide, fade, and
  hover-expand chrome are out of scope; bars are reserved and visible or absent.
- **Thumb semantics:** length reflects viewport/content proportion subject to
  the minimum; position reflects accepted displayed offset. The thumb does not
  jump to the final target ahead of content. Track clicks page toward the pointer
  by one viewport minus one line, clamped at the target. Dragging is direct and
  preserves the initial pointer-to-thumb grab offset, with no easing lag. While a
  thumb is captured, wheel and keyboard input for the same axis is ignored and
  cannot start a transition; input for the other axis routes normally. Releasing
  the thumb leaves the displayed offset exactly where the drag placed it.
- **Keyboard/accessibility:** focused scroll regions support arrows, Page Up/
  Down, Home/End and accessible axis/range/current-value semantics through existing
  owners. Home/End act on block when available, otherwise inline. Keyboard steps
  use a declared line extent (20 points in Pulse), never an incidental row height. Respect reduced
  motion by direct settlement; retain fine control and keyboard reachability.
- **Stability:** preserve a current stable item anchor and its viewport-relative
  position across content insertion/removal when that anchor survives. Otherwise
  clamp the existing offset. When extent changes during an active transition,
  reclamp the pending target and the accepted offset together in the same
  preparation; a transition may never continue toward a target that is no longer
  legal. Empty content, owner removal, modality loss, focus loss, and capture
  cancellation settle explicitly and do not leave motion alive.
- **Hover under motion:** when content moves beneath a stationary pointer, pointer
  affordance and hover appearance re-resolve from the accepted sample on the frame
  that sample is accepted, without a synthetic pointer event. Hover never resolves
  against the ahead target. The row under the pointer at settlement is the row
  that receives a click; no hover state may linger on a row that has moved away.

## Authority and publication contract

Mosaic owns viewport/content extents and exact region occurrences. Scroll owns
axis policy, bounds, anchoring, routing, latch resolution, and semantic target
offsets. Interaction owns capture, gesture lifecycle, and the latch's lifetime.
Motion owns interpolation, velocity-continuous retargeting, reduced-motion
settlement, and scheduling through the existing sampled lane. Appearance owns
track/thumb styling. Presentation owns device-grid snapping as a derivation.
Hosts execute admitted mechanics only and admit the platform line extent.

Prepare Scroll succession and any Motion request before publication. The ordinary
frame must carry the same prepared results used for its first visible sample;
acceptance commits them together. Rejection preserves predecessor state and paint,
retains coherent retry evidence, and cannot acknowledge the refused input effect.
Later samples advance displayed position only after host acceptance. In-flight
input may update a bounded pending target; it cannot overwrite accepted evidence.
The semantic target may lead displayed content. The accepted Motion sample alone
governs displayed mounted geometry, hit testing, and thumb position. Thumb capture
interrupts easing from that sample and its pointer grab offset, never from the
ahead target; the direct drag then supplies the successor target.
Retire delivered observation reports after every synchronous consumer, preserving
sequence validation and bounded duplicate fingerprints. Raw delivery retirement
does not acknowledge a presentation effect: a staged successor retains the exact
input consequence needed for acceptance, retry, or explicit cancellation.

Bind a Motion target to an exact Scroll region occurrence/incarnation, its indexed
descendant paint group, and stationary viewport clip. Keep ordinary component,
Portal-content, and Scroll-content targets structurally distinct. Every moving
surface, text command, outline, hit target, and derived thumb uses the same sample;
ancestor and viewport clips do not move with their content. Nested clips retain
their own coordinate ownership. No per-frame graph search or appearance resolution.

Scrollbar declarations name their Scroll owner and axis; applications never read
diagnostic offsets to move rectangles or mutate Scroll through a paint callback.
Missing ownership, duplicate/ambiguous binding, stale incarnation, invalid extent,
unsupported capability, and exhausted admission are typed failures before effects.
Required preparation strengthens the existing phase consumed by presentation;
an assembled frame or marker claiming readiness cannot bypass it. Direct/no-work
paths require validated precision, dependency, and reuse evidence.

Proposed public authoring shape, to be implemented through the existing
`worth_ui::facade::declaration` facade and tested with a valid example:

```rust
let scrolling = UiScrollPolicy::nested_region()
    .with_wheel_behavior(UiScrollWheelBehavior::smooth(120)?);
let region = region.with_scroll_chrome(UiScrollChromeContract::new(
    UiScrollAxisSupport::Both, track_role, thumb_role,
)?);
```

These declarations carry policy and registered role identities, not live authority.
Rust and file authoring lower to the same canonical contract. Installation rejects
smooth policy without admitted Motion support. Existing policy construction stays
direct unless smooth behavior is explicitly declared. No raw offset setter, new
host animation protocol, generic service pipeline, or Query ownership is introduced.

## Destination topology

Paths below are relative to `workspaces/worth-ui`; existing owners stay in place.
Names under new directories describe responsibilities, not mandatory file counts.

```text
crates/worth-ui-dsl/src/                         [extend canonical region declarations]
crates/worth-ui-runtime/src/
  declaration/service/scroll.rs                 [extend public policy meaning]
  capability/registry/mosaic_region/            [extend chrome capability admission]
  runtime/scroll/{chrome,transition}/            [new derived chrome and offset succession]
  runtime/motion/                               [extend exact target scope/retargeting]
  runtime/interaction/                          [extend scrollbar capture, latch, keyboard routing]
  runtime/pointer_affordance/                   [extend hover re-resolution from samples]
  facade/entry/active_application_session/      [extend cross-owner preparation]
  mounting/occurrence_geometry/                 [extend exact region/anchor geometry]
  mounting/presentation/motion_sampling/        [extend accepted Scroll-group sampling]
  mounting/projection/                          [extend ordinary chrome lowering]
crates/worth-ui-host-native/src/native/input/   [extend device/axis normalization only]
apps/platform-pulse/src/product_world/visual_composition/dashboard/
                                               [extend real content/chrome declarations]
```

Runtime state and leases remain private to their owners. Pure declarations cannot
mint capture, presentation, or offset authority. Appearance cannot import Scroll
mutation. 3.16.2 adds extent succession to these owners; it must not move facades
or replace this path. Remove displaced immediate-only coarse-wheel handling,
fixed/app-calculated thumb positions, and invalid overflow fixtures on cutover;
preserve the valid direct pixel path and existing 512-occurrence proof.

## Delivery sequence and acceptance

**1. Reach the endpoint.** Implement the smallest complete owner-to-presentation
path and run one real Pulse panel with both axes, moving thumbs, wheel smoothing,
direct dragging, and a working click after movement. Establish pixel/geometry
assertions early. A prerequisite is complete only when its enabled behavior runs.

**2. Harden the same path.** Finish nested routing and gesture latching,
modality/capture cancellation, anchoring, in-flight target reclamp, hover
re-resolution, presentation snapping, reduced motion, rejection/retry, and
reconstruction. Use focused cases for distinct failure modes; do not force all
conditions into one lifetime.
Presentation must consume prepared results; reuse the existing compile-fail target
to prove incomplete preparation cannot present, with one valid counterpart.

**3. Qualify and delete.** Preserve equal/unchanged text reuse, border/corner,
damage, theme, Portal order, and locality evidence affected by the change. A local
scroll may visit its ownership chain and changed visible descendant commands,
not unrelated regions or all mounted instances. Track retained allocations and
active work; input bursts must not grow storage with historical event count.

Qualify scrolling with causal, platform-neutral progress and work evidence, not
display timestamps or a machine-specific refresh-rate threshold. On the real
native Pulse journey, deliver a wheel notch and a held thumb drag through the
platform input adapter. Independently captured pixels must match the declared
content translation and thumb geometry, and a click after each move must hit
the newly displayed row. The host's accepted-sample record must contain
successive presentation-only motion samples with advancing epochs, positive
damage/render/present work, and less rendered work per sample than one complete
physical client raster. Deterministic locality cases must additionally prove
that scroll invalidation visits the affected viewport, not unrelated regions.
Deterministic runtime cases must cover burst coalescing,
retargeting, cancellation, final settlement, refusal/retry, and locality; an
accepted counter alone cannot prove the pixels moved. Conversely, endpoint
pixels alone cannot prove that intermediate work remained bounded.

Elapsed wall-clock and input/capture timestamps may be recorded for diagnosis,
but are not pass/fail criteria for 3.16.1. No three-trace DXGI or equivalent
compositor capture is required. This qualification makes no display-cadence or
latency claim. Precision input still requires a real precision-device run for
its own device-specific claim; when hardware is absent, mark that claim
unverified rather than passing it from synthetic input.

Use existing runtime and certification targets; no new runner or progress ledger.
Keep semantic native/headless parity and selected native pixels rather than
duplicating every scenario on both hosts. Run affected tests and warning-free
default and demo-feature builds, formatting, dirty line caps, boundary-check,
and agent-context check. Investigate failures before broader reruns. Reviewers
must answer whether callers can skip preparation or commit state different from
the evidence that produced accepted pixels. Material findings block closure.

Revise only three continuing guides during implementation: `docs/appearance-and-themes.md`
for author policy/chrome examples and limitations; `docs/runtime-subsystems.md` for
Scroll/Motion/capture lifecycle; `docs/native-host-platform.md` for input precision,
accepted sampling, functional scroll qualification, and cleanup. Examples must compile/lower.
No additional appearance guide or closeout document is required.

3.16.2 may trust exact two-axis geometry, accepted displayed position, stable
anchors, chrome derivation, gesture latching, and bounded motion/capture
cancellation. Window resizing, workspace docking, virtualization, touch
rubber-banding, overlay autohide/fade chrome, scrollbar arrow buttons, held
track-click auto-repeat, middle-button autoscroll, and drag-select edge
autoscroll are separate and not claimed here.
