# Native Host Platform

## What This Feature Is

The native host platform opens a real Worth UI desktop window and runs a
host-neutral Worth UI application on the qualified Windows graphics path. Use
it when the application should be owned by Worth UI from preparation through
normal window close, instead of being embedded in another UI toolkit.

## Why You Use It

- Launch a Worth UI application in a native Windows window.
- Keep application declaration separate from window, event-loop, and GPU
  mechanics.
- Receive a typed close or stop outcome with presentation and resource-cleanup
  evidence.

## Stable Entry Points

- `WorthUiNativePlatform::prepare(profile)` validates the effect-free launch
  profile.
- `UiNativeApplicationDefinition::prepare(...)` declares the application while
  no window or graphics resource exists.
- `UiNativeApplicationPreparation::builder()` exposes the bounded Worth UI
  application builder.
- `UiPreparedNativePlatform::run(application)` consumes the prepared platform,
  opens native effects, and blocks until close or stop.
- `UiNativePlatformOutcome::{Closed, Stopped, ApplicationPreparationDenied}`
  reports the terminal posture.

Import these from `worth_ui_native_platform`. Application descriptors and
change profiles still come from the public `worth_ui` facade.

## Core Mental Model

Application meaning is frozen before the platform selects native mechanics.
The application callback may register Worth UI declarations, but it cannot
obtain a window, event loop, graphics device, or host-binding grant. When the
callback completes successfully, the platform privately binds the one
qualified native host and starts the event loop.

The platform owns the whole run. The caller holds no runnable native adapter
and cannot replace the host after preparation. A normal close returns a receipt
that includes the last presentation, the peak native-resource census, and a
terminal zero-resource census. A failure returns a stop report that preserves
whether effects may already have begun.

## How It Executes

1. Construct a `UiNativePlatformProfile` with one window specification.
2. Call `WorthUiNativePlatform::prepare`. Invalid platform, architecture,
   title, or extent is rejected before native effects.
3. Pass an application definition to `run`.
4. In `prepare`, install the application change profile and register its
   declarations and authored input.
5. Return `preparation.complete()` or an explicit typed denial.
6. The platform privately creates the native host, binds the application,
   presents admitted mounted work, and waits for events.
7. Inspect the consuming terminal outcome after normal close or failure.

## Small Example

This validates the current native launch profile without opening a window:

```rust
use worth_ui_native_platform::{
    UiNativePlatformProfile, UiNativeWindowSpec, WorthUiNativePlatform,
};

let window = UiNativeWindowSpec::new("My Worth UI app", [960, 640]);
let profile = UiNativePlatformProfile::single_window(window);
let prepared = WorthUiNativePlatform::prepare(profile)?;

assert_eq!(prepared.profile().window().initial_logical_size(), [960, 640]);
# Ok::<(), worth_ui_native_platform::UiNativePlatformPreparationDenial>(())
```

Preparing is intentionally effect-free. No OS window exists until the prepared
platform is consumed by `run`.

## Real Example

The Platform Pulse composition root uses its application definition—which
installs the change profile, theme token, component, and authored input—then
gives that definition to the native platform:

```rust
use worth_ui_native_platform::{
    UiNativePlatformOutcome, UiNativePlatformProfile, UiNativeWindowSpec,
    WorthUiNativePlatform,
};
use worth_ui_platform_pulse::PlatformPulseNativeSeedApplication;

let window = UiNativeWindowSpec::new("My Worth UI app", [960, 640]);
let platform = WorthUiNativePlatform::prepare(
    UiNativePlatformProfile::single_window(window),
)?;

match platform.run(PlatformPulseNativeSeedApplication::new()) {
    UiNativePlatformOutcome::Closed(receipt) => {
        assert!(receipt.terminal_census().is_zero());
    }
    UiNativePlatformOutcome::ApplicationPreparationDenied(denial) => {
        eprintln!("application preparation denied: {:?}", denial.cause());
    }
    UiNativePlatformOutcome::Stopped(report) => {
        eprintln!("native run stopped: {:?}", report.reason());
    }
}
# Ok::<(), worth_ui_native_platform::UiNativePlatformPreparationDenial>(())
```

A real application also registers components, tokens, surfaces, authored
input, and inspection policy through the borrowed preparation builder. The
Platform Pulse native seed is the smallest production example that does this
and presents one attributed filled rectangle.

## Native Input Settlement

Native observations carry their presentation affinity, event tick, coordinate
space, and unit from the event boundary into the retained batch. Pointer,
keyboard, scroll, and IME observations use the event-time profile; preedit,
commit, and cancel remain distinct phases. A pending successor does not
retarget observations, and a profile or scale transition admits new input only
after its profile evidence is complete.

The readiness signal is level-triggered by retained observations. The event
loop rearms it only after the runtime callback drains through the authorized
interaction owner. Normal close waits for that retained input to settle. The
close receipt preserves the input report and terminal census, while the
Platform Pulse phase-six evidence additionally records applied, duplicate,
quarantined, and denied ingress dispositions plus any drain denial.

## How It Relates To Other Features

- Use `worth_ui` for declarations, authored input, rebind profiles, and
  application semantics.
- Use `worth_ui_native_platform` only at the application composition root.
- The headless host consumes the same inert presentation contract for testing
  and recording, but it is not a substitute for native pixel evidence.
- Query, intent, text, capture, and the complete Pulse journey use this native
  platform. Later multi-window work extends the same owner.

## Inspection And Debugging

A successful close receipt exposes:

- the attributed presentation observation and retained-source pixel;
- physical client size and DPI scale;
- selected graphics qualification;
- presentation and readiness counters;
- peak and terminal resource censuses.

A stop report exposes the stop reason, effect posture, peak census, terminal
census, and whether client cleanup completed. Treat
`PresentationIndeterminate` as a real may-have-presented state; do not retry as
if the failed attempt were known to be effect-free.

## Anti-Patterns

- Do not depend on `worth-ui-host-native` to construct or run a host directly.
- Do not try to bind a `WorthUiHostNeutralApp` yourself; native binding is a
  private platform transition.
- Do not perform OS, thread, Query, filesystem, or graphics work inside the
  application preparation callback.
- Do not treat a compositor screenshot as retained-source readback, or a
  retained-source pixel as proof of the external client area.
- Do not infer cleanup from process exit; inspect the typed terminal census.

## Current Limits

- The qualified production lane is Windows 11 on x86-64 with DX12.
- Phase 2 supports one window and one initial attributed filled rectangle.
- The initial logical extent is bounded to `16_384 × 16_384`, and adapter
  qualification requires that full texture extent.
- Native text, retained deltas, input, public capture, recovery breadth, and
  multi-window ownership remain deferred to their ordered milestone phases.
- The platform owns the blocking event loop; embedding it into another native
  event loop is unsupported.

## Related Docs

- [Milestone 3.14.1 specification](./milestone-3.14.1.md)
- [Worth UI roadmap](./worth_ui_roadmap.md)
