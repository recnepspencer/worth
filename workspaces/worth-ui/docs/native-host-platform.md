# Native Host Platform

## What This Feature Is

The native host platform is the framework-owned application and host boundary.
It binds one Worth application to one qualified native profile, runs its native
window and graphics lifecycle, and closes its resources without giving product
code a raw adapter, event-loop client, graphics object, or wake port.

Platform and application preparation remain effect-free. No window, event
loop, surface, device, queue, or physical-work worker exists until the
application returns a prepared definition. After that gate succeeds,
`run(...)` enters the qualified native host and may perform native effects.

## Stable Entry Points

- `WorthUiNativePlatform::prepare(UiNativePlatformProfile)`
- `UiPreparedNativePlatform::run(UiNativeApplicationDefinition)`
- `UiNativeApplicationPreparation::builder()`
- `UiNativeApplicationPreparation::complete()`
- `UiNativeApplicationPreparation::deny(...)`
- `UiNativeApplicationPreparationOutcome`
- `UiNativePlatformOutcome`

The platform privately issues `UiNativePlatformBindingGrant`. The grant,
preparation scope, prepared application, and denial payload are move-only and
have private fields. There is no parts conversion or host replacement route.

## Preparation Progression

```text
qualified UiNativePlatformProfile
-> WorthUiNativePlatform::prepare
-> UiPreparedNativePlatform with one application slot
-> run(application definition)
-> UiNativeApplicationPreparation with the platform-bound Worth builder
-> application registration through a borrowing builder view
-> Prepared(UiPreparedNativeApplication)
   | Denied(UiNativeApplicationPreparationDenial)
-> qualified native host and event-loop execution
-> Closed(UiNativePlatformCloseReceipt)
   | Stopped(UiNativePlatformStopReport)
```

Calling `builder()` borrows the internal builder. It cannot extract or freeze
it, replace the native host, or retain a second application lane. `complete`
consumes the whole preparation scope and is the only way to produce a prepared
native application.

## Small Example

```rust
use worth_ui_native_platform::{
    UiNativeApplicationDefinition, UiNativeApplicationPreparation,
    UiNativeApplicationPreparationOutcome, UiNativePlatformProfile,
    UiNativeWindowSpec, WorthUiNativePlatform,
};
use worth_ui::facade::rebind::UiChangeProfile;

struct Application;

impl UiNativeApplicationDefinition for Application {
    fn prepare(
        self,
        mut preparation: UiNativeApplicationPreparation,
    ) -> UiNativeApplicationPreparationOutcome {
        if let Err(cause) = preparation
            .builder()
            .with_change_profile(UiChangeProfile::platform_pulse())
        {
            return preparation.deny(cause);
        }
        preparation.complete()
    }
}

let profile = UiNativePlatformProfile::single_window(UiNativeWindowSpec::new(
    "WORTH UI",
    [960, 600],
));
let prepared_platform = WorthUiNativePlatform::prepare(profile)?;
let outcome = prepared_platform.run(Application);
```

An application-preparation denial still occurs before native effects or native
host construction. Once preparation succeeds, `run(...)` returns either a
close receipt or a typed stop report. Both expose the terminal resource census;
a stop with retained external obligations also exposes cleanup authority.

## Presentation Contract

Runtime owns presentation meaning and emits one sealed revision-8 work item:

- `Initial` carries every attributed command, stable total order, initial
  logical damage, auxiliary reconstruction state, and the surface-issued
  transparent baseline.
- `Delta` carries only command changes, affected order edits, owner-issued
  logical damage, and an auxiliary successor when that meaning changed.
- `Unchanged` carries exact predecessor/successor affinity and no command,
  order, damage, or native work.

Host mechanics retain commands by owner-issued identity. They may build
mechanical indexes for execution, but they do not receive the complete
projection on ordinary successor frames and do not rediscover semantic deltas.
Candidate retained state commits only after every required surface succeeds.

Revision 8 carries runtime-resolved surface fill, border and corner radii,
outline, text-range foreground, pointer affordance, Backdrop, and relational
overlay-order mechanics. Native and headless hosts translate that sealed work;
they do not select roles, theme cells, visual state, seams, or Portal order.
Motion arrives as the exact accepted command sample already composed into the
prepared appearance work.

Surface fill is either solid or a two-stop linear gradient. Gradient endpoints
are relative to the un-clipped allocation; sampling projects onto the physical
axis and interpolates premultiplied linear-sRGB colors. Native and headless
consumers preserve that geometry, clipping, and the composed presentation opacity.

Border and outline geometry follows half-open mounted visual bounds. A seam
omission is expressed in owner-surface local coordinates and applies only where
the owning region's shared edge coincides with that surface's exterior edge.
Native rasterization preserves border coverage through rounded corners and
includes qualified anti-alias fringes in damage.

## Runtime-Service Mechanics

Runtime services do not create a catch-all host protocol. The host contract
keeps observations, mounted presentation, measurement, and solicited effects
separate:

- `WindowFocus` reports activation for an exact host surface. It does not move
  or own semantic keyboard focus.
- `UiHostFocusPlacementRequest` is the one narrow solicited service effect. It
  binds the host session, surface, current presentation, mounted target, and
  host logical geometry. Native and headless adapters return a typed
  acknowledgement for that exact request.
- `ScrollDelta` reports source, phase, precision, high-resolution x/y delta,
  and exact coordinate, mounted-target, or current presented-surface affinity.
  The runtime Scroll owner resolves and updates the semantic owner and offset.
  Precision is part of the report, not a hint. A pixel delta is travel already
  and reaches the offset untouched, which is the path a precision device --
  touchpad, precision wheel -- takes. A line delta is a count of lines in
  thousandths, already multiplied by the platform's lines-per-notch reading, and
  the runtime turns it into travel with the region author's declared line
  extent; a region that declares none is denied rather than moved. The adapter
  never converts one precision into the other, and a synthetic pixel delta
  therefore proves the semantics of a precision device and not its timing.
- `Tick` and the existing level-triggered readiness path wake presentation
  sampling. They do not give the host a motion timeline or interpolation
  authority.
- Portal content is product-authored mounted overlay work. The host presents
  its geometry, layer, lifecycle, and input shielding, but does not own logical
  open/close state. An operating-system popup surface is unsupported in this
  contract.

A settling region is driven by the shell on the frames it already runs, not by
a timeline the host owns:

```text
prepare_motion_tick
-> present_prepared_motion_tick
-> settle_accepted_scroll_sample
```

The accepted offset is what the last presented sample actually placed, not what
the settle is aiming at. A host that presented a frame has moved the reader's
content; one that prepared a frame it never presented has not, and the accepted
offset must not claim otherwise. Because a region's owner box at rest is its
content box, the first frame after an input samples the rest pose and moves
nothing, so a settle declaring N ticks arrives on frame N + 1.

Focus placement preserves the physical-effect boundary:

```text
runtime prepares exact request
-> adapter issues or rejects before effect
-> exact acknowledgement settles the request
or unknown completion retains reconciliation authority
-> current WindowFocus / mounted host truth reconciles
```

A timeout or missing acknowledgement is never converted to success. Shutdown
cancels only before-effect work, retains what may have escaped, and releases
the focus-effect record only after settlement, typed abandonment, or
reconciliation. Neither Portal nor Motion may use this port as a generic
service-effect escape hatch.

## Physical Work Progression

Host-native owns physical resources and effects. The device owner retains the
adapter/device/queue generation, while the presentation-surface owner retains
the surface and current target. Concrete WGPU and Winit handles remain inside
the WGPU backend; the graphics port carries only associated backend mechanics.
The native adapter's observation of an external effect is the source of
physical completion truth.

A private physical Signal runtime inside host-native owns how that work
progresses. It admits bounded work, tracks the current physical attempt, emits
exact readiness wakes, and governs retry, timeout, cancellation, supersession,
recovery scheduling, and shutdown ordering. One runtime is retained for each
native host/device lifecycle; it is not created per presentation.

```text
runtime presentation work
-> host-native reserves physical owners
-> private physical Signal runtime admits bounded work
-> Winit readiness transport wakes the native event thread
-> native adapter submits or polls the external effect
-> typed physical observation returns to Signal
-> Signal settles, retries, supersedes, or schedules recovery
-> host-native commits or releases the physical owners
```

Signal owns progression, not effects. It does not store raw WGPU handles,
allocate atlas storage, submit command buffers, poll the device, or decide that
an external consequence completed. Readiness is eligibility to progress work;
it is not completion evidence or effect authority. The Winit readiness
registry transports wakes only and does not provide a second retry or
currentness scheduler.

An observation must match the exact owner-issued physical runtime, work, and
attempt. Stale, duplicate, foreign, or superseded observations cannot settle
current work. A rejection known to occur before effects releases its owners.
An effects-indeterminate observation enters retained recovery until host-native
can reconcile or close the physical consequence.

Shutdown first stops new admissions, then drains retained completion and
recovery obligations. Signal state and native resource ownership are disposed
only after those obligations reach a terminal posture.

## Live Resize

A border drag reports far more client extents than frames can be prepared
for. Host-native records each positive extent as the newest pending extent and
requests a redraw. The next dispatch turn that can hand the client work, a
redraw, a posted wake, an input observation, or the idle turn, first prepares
one target for whatever extent is newest by then, and only then publishes that
viewport. Preparation cost follows turns, not raw resize events, and no
presentation reports an extent its pixels do not have. A zero extent is a
lifecycle transition, not a pending extent: it suspends presentation when
observed and discards the extent it supersedes. A scale change reads the
window's current extent itself, so it supersedes any pending extent.

While the border is held, Windows dispatches from its own modal loop. Winit
then emits no `NewEvents` or `AboutToWait`, and a `WaitUntil` deadline never
fires. Physical completion is polled on due ticks of the host clock, so an
in-flight frame would otherwise not settle, and no newer extent could present,
until release. A deadline watch thread holds the next physical-signal or
presentation-retry deadline and posts one application wake when the event loop
has not reached it within a small slack. It does not post that deadline again
while the wake is queued. Posted wakes are dispatched by any loop, and the
wake progresses timed work itself. The watch owns no host state and never
calls into it; in the ordinary loop, `WaitUntil` reaches each deadline first
and the watch stays silent.

## Presented-Source Readback

The Windows native host records one capture source only after presentation has
settled successfully. That source binds the host session, frame, presentation
attempt, surface, binding generation, presentation epoch, current client
transform, and the retained realized regions that produced the pixels. A draw
list, reconstructed image, or later compositor capture cannot substitute for
that identity.

Capture admission is deliberately two-step:

```text
request -> reserve one bounded logical slot and padded byte capacity (no GPU effect)
poll    -> revalidate the exact source, allocate owners, and submit the copy
poll    -> Pending | canonical tight RGBA8 pixels | completion indeterminate
```

The native owner permits at most four readback slots and 16 MiB of aggregate
padded capacity. WGPU's 256-byte row alignment remains internal; successful
results remove row padding and expose top-left, tightly packed RGBA8 bytes.
Resize, scale, binding, presentation, and derived-state changes invalidate
unsubmitted source reservations. Already submitted copies retain their exact
predecessor source until terminal observation. Cancelling the caller's request
does not cancel or forget physical work that may already have begun.

Cancellation before submission proves that readback did not begin.
Cancellation after submission can prove only that readback may have begun. A
device timeout remains pending. An unknown physical completion enters bounded
capture recovery and keeps its slot, padded bytes, readback buffer, and pending
submission owner charged. A map failure or malformed artifact observed after
GPU settlement is completion-indeterminate but physically releasable. Ordinary
admission and governed close retries progress retained recovery; host graphics
remain live until it settles. Runtime snapshot retention and disposal remain
runtime policy, not native policy.

## Relationship To Query Invalidation

The physical Signal runtime is separate from the Query-semantic Signal graph.
The physical runtime schedules native work; Query tracks which
application-visible presentation meaning is pending, current, stale, failed,
cancelled, superseded, or unresolved. Typed physical completion evidence may
later be admitted to the installed Query correspondence, but Query does not
submit, poll, retry, wake, recover, or release WGPU work.

The two graphs do not share runtime identities, aspect slots, request handles,
completion envelopes, capacities, or shutdown receipts. Runtime imports
neither Signal nor Query, and no separate manual physical scheduler runs beside
host-native's private Signal owner.

## Qualification

The checked-in qualified identities are:

- text profile: `worth-ui-body-default-v1`;
- native profile: `worth-ui-windows-dx12-v2`;
- host protocol: revision 9;
- mounted frame and presentation schemas: revision 8;
- text schema: revision 4;
- observation schema: revision 7;
- measurement schema: revision 5;
- solicited-effect schema: revision 1;
- native dependency versions: `winit 0.30.13`, `wgpu 29.0.4`,
  `rustybuzz 0.20.1`, and `swash 0.2.10`.

The manifest digests are fixed by the milestone specification and verified by
the native host qualification tests. The profile is not selected from ambient
system fonts, an environment variable, or an adapter default.

Protocol negotiation compares the protocol identity and every schema family
before effects. Older or newer observation, measurement, presentation, or
solicited-effect revisions produce a typed incompatibility. There is no alias
that interprets the former ambiguous `Focus` observation as `WindowFocus`, and
no mixed-revision fallback that silently drops scroll affinity or focus
settlement.

Scroll qualification is deliberately independent of refresh rate and compositor
capture APIs. The executable native journey sends a real wheel notch and held
thumb drag, then checks independently captured content and chrome pixels against
authored geometry and clicks the row that should now be under the pointer. The
accepted host record must show advancing presentation-only motion samples,
changing thumb geometry, positive damage/render/present work, and per-sample
rendered work below one complete physical client raster. Runtime tests separately
protect burst coalescing, retargeting, cancellation, settlement, refusal/retry,
and viewport-local invalidation. These sources complement each other: accepted
samples are not proof of compositor visibility, while a correct endpoint image
does not establish bounded intermediate work.

The executable-world lane uses the current 1536-by-1024 dashboard for native
Query/review interaction, imported-source replacement and denial/recovery, and
wheel/thumb journeys. Five older 960-by-600 Pulse live-journey tests remain
explicitly ignored: their snapshot publication and control coordinates describe
the predecessor product, so they cannot certify the current dashboard. The
current journeys provide the 3.16.1 acceptance evidence through independent
native pixels and causal input, including modal focus/close, predecessor pixels
after compilation denial, and post-scroll row hits. This does not claim that
every historical 3.15 scenario has been ported.

The former three ten-second DXGI timestamp gate and its p95/p99 deadlines have
been retired from 3.16.1. GDI and X11 captures remain functional pixel oracles,
not frame-pacing clocks. On this Windows machine, DXGI desktop duplication
returned `DXGI_ERROR_UNSUPPORTED`; that adapter failure no longer blocks this
functional qualification and is not converted into a timing pass. Elapsed times
may be retained for troubleshooting, but this milestone claims neither a
latency percentile nor display cadence on Windows or Linux.

Precision input is qualified by a run on a real precision device. A synthetic
pixel-delta report exercises the same semantic path and proves nothing about
timing, and absent hardware leaves the claim unverified rather than passed.
For this closeout, the user explicitly deferred the real precision-device run
on September 22 because no device is available. Its device-specific claim
remains unverified; this does not waive semantic acceptance/rejection tests.

## Cost And Failure Posture

Glyph diagnostic rows describe the ordinary frame's qualified base runs, not
their current sampled positions. One paired alpha/intrinsic derivation is
cached by exact frame, physical extent and atlas committed-content revision;
same-basis samples share immutable rows without rescanning unrelated text.
Reservations alone do not change committed atlas metadata or grant rendering
permission. Sampled chrome observations and external pixels separately prove
the displayed scroll pose.

Presentation reports structural and physical amplification separately:
delta rows, draw-list and order mutations, damage regions, index probes,
intersections, replay, cleared/rendered/presented pixels, GPU writes, passes,
copies, acquisitions, submissions, and presents. Unchanged reports exact zero
for all of them.

A pre-effect denial preserves the current publication. If an effect may have
started, the affected binding becomes indeterminate and must be reconciled or
closed. A successful surface in a partially failed multi-surface attempt does
not promote the candidate frame.

## Recovery And Hostile Close

Timeout and occlusion do not spend reconstruction authority in a local retry
loop. The runtime retains the exact mounted frame and reconstruction authority,
then admits another attempt only after a later qualified redraw or visibility
readiness generation. Validation rejection remains pre-effect. Surface
outdated, surface loss, device loss, and indeterminate completion enter the
typed recovery registry.

Physical graphics recovery is global to the native host generation, not local
to one semantic binding. One pending recovery epoch prepares and commits one
generation-bearing device/surface fact. Every affected binding must consume
that same fact through its own semantic reconstruction before the epoch can be
released. A stronger device-loss requirement supersedes a previously prepared
surface-only epoch instead of opening a parallel recovery lane.

Zero-sized Windows surfaces suspend presentation. Restoring a nonzero client
area publishes a later readiness generation and a monotonic successor-barrier
ordinal; retained reconstruction resumes from current mounted authority and
presents a distinct product frame. A later minimize/restore cycle cannot reuse
the earlier title/barrier observation. The Windows lifecycle courtroom verifies
each successor presentation before it accepts compositor pixels.

A settle is the tail of a gesture somebody is watching. A window that stops
being the reader's, an unmounted subject, content that empties or shrinks to
fit, and a pointer that takes a thumb each end the track explicitly, at the
sample the reader last saw, rather than finishing on their own and moving
content behind the reader's back. No motion survives the thing it was moving,
so the census a close waits on is reached rather than approached.

Close uses one ordered shutdown progression: stop admission, settle external
presentation and readback obligations, release derived/recovery state, release
native resources, then publish `Closed` only when the complete census is zero.
Close during preparation, any presentation stage, readback, queued readiness,
or a held application attempt uses this same progression. Certification
observes the production readiness registry and runtime-issued mounted attempt;
it does not mint census-only stand-ins.

## Current Limits

The physical Signal runtime is private host-native machinery, not an
application-facing scheduling API. Application-visible async presentation
posture and the native-completion-to-Query correspondence are still being
stabilized and are not public control surfaces. Product code cannot select a
Signal runtime, supply raw physical handles, or replace native recovery policy.

## Anti-Patterns

- Do not call a product-facing host selector or keep a default hidden host.
- Do not import runtime internals from a host or platform crate.
- Do not pass a complete projection to a successor-frame host operation.
- Do not treat a profile digest, baseline identity, or cost report as authority.
- Do not infer success after an uncertain or partially completed native effect.
- Do not run a manual retry, timeout, currentness, or wake scheduler beside the
  private physical Signal runtime.
- Do not route physical WGPU progression through Query or treat semantic
  invalidation as permission to perform native effects.
- Do not give Signal raw WGPU handles, atlas storage ownership, or effect
  authority.
- Do not treat `WindowFocus` as semantic keyboard focus or use focus placement
  as a generic service command.
- Do not create native popup state for a same-surface Portal or interpolate a
  Motion track in the adapter.
- Do not discard scroll source, phase, precision, target affinity, or
  presentation basis before runtime admission.

## Related Docs

- [Runtime services](./runtime-services.md)
- [Interaction and intents](./interaction-and-intents.md)
- [Application lifecycle](./application-lifecycle.md)
- [Worth UI architecture](./architecture.md)
