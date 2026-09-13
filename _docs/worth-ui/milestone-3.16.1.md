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
2. Scroll horizontally, diagonally with precision input, and with Shift+wheel.
   Press a moving thumb during wheel easing, then drag each thumb to both ends,
   including releasing outside the track/window. Capture must not cause a jump.
   Return to the first row and verify restored text and unchanged neighbors.
3. In a separate focused production-handoff test, reject an intermediate frame,
   retry, change content extent, remove/reincarnate the owner, and reconstruct.
   No rejected sample becomes physical truth; stale input cannot target the
   replacement owner. Shutdown leaves no capture, track, wake, or report lease.

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
- **Axes:** vertical-only regions ignore incidental inline movement. Both-axis
  regions preserve intentional diagonal input. Shift maps a vertical wheel to
  inline movement only when no horizontal delta is already supplied. Nested
  routing uses the declared chain and passes only lawful remainder; modal
  shielding and captured scrollbar gestures take precedence over pointer location.
- **Edges:** the dashboard clamps without bounce. Wheel/thumb input cannot leave
  legal bounds. OS momentum phases remain explicit. Apple-style rubber-banding
  is not required here and must not be imitated with an unowned renderer offset.
- **Chrome:** show a track and thumb for every overflowing enabled axis. Reserve
  a stable 12-point gutter; use a centered 6-point rounded thumb, minimum length
  24 points, a subtle track, and distinct normal/hover/drag appearance roles.
  These are Pulse's declared metrics, not hardcoded host defaults; the platform
  admits typed chrome metrics with nonnegative extents and a usable thumb range.
  The effective pointer target fills the gutter. Respect an admitted system
  always-show preference; Pulse itself keeps overflowing bars visible for testing.
  No-overflow axes have no active thumb or misleading drag target.
- **Thumb semantics:** length reflects viewport/content proportion subject to
  the minimum; position reflects accepted displayed offset. The thumb does not
  jump to the final target ahead of content. Track clicks page toward the pointer
  by one viewport minus one line, clamped at the target. Dragging is direct and
  preserves the initial pointer-to-thumb grab offset, with no easing lag.
- **Keyboard/accessibility:** focused scroll regions support arrows, Page Up/
  Down, Home/End and accessible axis/range/current-value semantics through existing
  owners. Home/End act on block when available, otherwise inline. Keyboard steps
  use a declared line extent (20 points in Pulse), never an incidental row height. Respect reduced
  motion by direct settlement; retain fine control and keyboard reachability.
- **Stability:** preserve a current stable item anchor and its viewport-relative
  position across content insertion/removal when that anchor survives. Otherwise
  clamp the existing offset. Empty content, owner removal, modality loss, focus
  loss, and capture cancellation settle explicitly and do not leave motion alive.

## Authority and publication contract

Mosaic owns viewport/content extents and exact region occurrences. Scroll owns
axis policy, bounds, anchoring, routing, and semantic target offsets. Interaction
owns capture and gesture lifecycle. Motion owns interpolation, retargeting,
reduced-motion settlement, and scheduling through the existing sampled lane.
Appearance owns track/thumb styling. Hosts execute admitted mechanics only.

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
  runtime/interaction/                          [extend scrollbar capture and keyboard routing]
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

**2. Harden the same path.** Finish nested routing, modality/capture cancellation,
anchoring, reduced motion, rejection/retry, and reconstruction. Use focused cases
for distinct failure modes; do not force all conditions into one lifetime.
Presentation must consume prepared results; reuse the existing compile-fail target
to prove incomplete preparation cannot present, with one valid counterpart.

**3. Qualify and delete.** Preserve equal/unchanged text reuse, border/corner,
damage, theme, Portal order, and locality evidence affected by the change. A local
scroll may visit its ownership chain and changed visible descendant commands,
not unrelated regions or all mounted instances. Track retained allocations and
active work; input bursts must not grow storage with historical event count.

For a warm optimized native Pulse at 1536x1024 on the recorded Windows 60 Hz
qualification machine, capture three 10-second active-scroll traces after warmup.
Require p95 input-to-first-visible-change <=50 ms, p95 accepted visible-frame gap
<=25 ms, p99 <=50 ms, and no gap >100 ms while motion/input requires progress.
Final wheel settlement is within 120 ms plus two display intervals of the last
input, absent an explicitly injected host refusal. Report hardware, scale, device,
raw intervals, and preparation/host costs; do not average away stalls. Precision
input requires a real precision-device run; synthetic pixel reports only prove
semantics. Missing hardware is unverified evidence, not a pass. These are the
qualified 60 Hz floor, not a universal or 120 Hz claim.

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
accepted sampling, timing qualification, and cleanup. Examples must compile/lower.
No additional appearance guide or closeout document is required.

3.16.2 may trust exact two-axis geometry, accepted displayed position, stable
anchors, chrome derivation, and bounded motion/capture cancellation. Window
resizing, workspace docking, virtualization, and touch rubber-banding are separate.
