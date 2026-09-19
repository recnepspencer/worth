# Application Lifecycle

## What This Feature Is

The application lifecycle prepares, launches, runs, inspects, and shuts down
one Worth UI application while the framework keeps graph, Query, host, mounted
publication, and generation state coherent. Application code holds one
`WorthUiActiveApplicationSession`.

## Why You Use It

- Launch Query-free or Query-backed UI through one path.
- Present headless and native-host frames with the same mounted contract.
- Preserve the last published frame when preparation or presentation denies.
- Handle in-flight or uncertain native effects through typed outcomes.
- Inspect the active generation without receiving execution authority.

## Stable Entry Points

- `worth_ui::facade::app::WorthUi::app()`
- `worth_ui_native_platform::WorthUiNativePlatform::prepare(...)`
- `UiNativeApplicationDefinition::prepare(...)`
- `UiNativeApplicationPreparation::builder()` and `complete()`
- `WorthUiApp::launch()`
- `WorthUiActiveApplicationSession::execute_mounted_frame(...)`
- `WorthUiNativeApplicationShell::begin_source_rebind(...)`
- `WorthUiActiveApplicationSession::inspect(...)`
- `WorthUiActiveApplicationSession::shutdown()`
- `UiMountedFrameOutcome`
- `WorthUiMountedFrameExecutionStop`
- `UiSourceRebindRequest`
- `WorthUiNativeManagedSourceRebindOutcome`
- `WorthUiNativeManagedRebindProgress`
- `WorthUiNativeSourceRebindDenial`

Mounted request, deadline, outcome, and recovery types are re-exported by
`worth_ui::facade::app`. Application code does not import a mounted runtime
module.

## Core Mental Model

The native preparation scope carries the only host-bound builder. An unbound
builder cannot freeze, and a bound builder cannot replace its host. `complete`
freezes one inseparable application generation: authored meaning,
capabilities, graph, Query bindings, host plan, and inspection indexes. Launch
consumes that prepared app and returns the only ordinary running owner.

`execute_mounted_frame` collects admitted inputs, advances the runtime,
assembles all required surfaces, presents through the host contract, and
publishes only a complete frame. A receipt reports what happened; it cannot be
used to execute or publish another frame.

After launch, `begin_source_rebind` is the ordinary bridge from one settled
filesystem snapshot to semantic classification, bounded consequence planning,
canonical host presentation, and atomic successor publication. It borrows the
same running shell and retains incomplete work inside that shell.

## How It Executes

```text
WorthUi::app()
-> framework-owned one-shot host binding
-> register capabilities, source, Query views, intents, and inspection policy
-> complete native preparation
-> launch
-> register/mount application-specific surfaces as required
-> execute_mounted_frame
-> Published | Unchanged | Reconciled
   | RejectedBeforeEffects | InFlight | PresentationIndeterminate
   | RetentionDenied | AdmissionDenied | CompletionDenied
-> inspect or continue through the returned typed outcome
-> for a settled edit, begin_source_rebind
-> Published | Pending | Stopped
-> progress_managed_rebind or retry_managed_rebind while Pending
-> Published | AwaitingProgress | Stopped
-> shutdown
```

A pre-effect denial leaves the predecessor publication current. If native
effects may have started, the shell retains semantic truth and the exact
completion, retry, or recovery authority until managed progression settles it.

## Small Example

Inside a function that returns the preparation error, a minimal Query-free
native definition has this shape:

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
let platform = WorthUiNativePlatform::prepare(profile)?;
let outcome = platform.run(Application);
```

Preparation is effect-free. After it succeeds, `run` enters the qualified
native lifecycle and returns an exhaustive closed, stopped, or
application-preparation-denied outcome. Query-free applications do not create
dummy Query work.

## Executable Fresh-Reader Contract

The following is the exact downstream program used by the compiler-contract
matrix and the application-contract runtime suite. It uses the ordinary
product facade plus the sealed certification headless transition, branches
over every mounted outcome and start-stop family, and executes the real
empty-application path.

<!-- compile-run:ordinary-mounted-frame -->
```rust
use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedFrameRequest,
    UiMountedFrameRetentionRejection, UiMountedHostMeasurementTransitionDenial,
    UiMountedHostMeasurementUnexpectedTransition, UiMountedIndeterminateFrame,
    UiMountedPresentationAdmissionRejection, UiMountedPresentationCompletionDenial,
    UiMountedPresentationInFlight, UiMountedRejectedFrame, UiMountedSupersededFrame,
    UiPresentationDeadline, WorthUi, WorthUiMountedFrameExecutionStop,
};

fn main() {
    run();
}

pub fn run() {
    let app = WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .map(
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_headless,
        )
        .expect("empty application preparation should succeed");
    let mut session = app.launch().expect("empty application should launch");
    let outcome = match session.execute_mounted_frame(
        UiMountedFrameRequest::all_bound_surfaces(),
        UiPresentationDeadline::at_tick(1),
        0,
        |_| {},
    ) {
        Ok(outcome) => outcome,
        Err(stop) => {
            observe_stop(&stop);
            return;
        }
    };

    match outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => {
            observe_publication(&receipt);
        }
        UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
            observe_rejection(&rejection);
        }
        UiMountedFrameOutcome::InFlight(in_flight) => {
            observe_in_flight(&in_flight);
        }
        UiMountedFrameOutcome::PresentationIndeterminate(indeterminate) => {
            observe_indeterminate(&indeterminate);
        }
        UiMountedFrameOutcome::RetentionDenied(rejection) => {
            observe_retention_denial(&rejection);
        }
        UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            observe_admission_denial(&rejection);
        }
        UiMountedFrameOutcome::CompletionDenied(denial) => observe_completion_denial(&denial),
        UiMountedFrameOutcome::Superseded(superseded) => observe_superseded(&superseded),
    }
}

fn observe_stop(stop: &WorthUiMountedFrameExecutionStop<'_>) {
    match stop {
        WorthUiMountedFrameExecutionStop::PublicationLease(_) => {}
        WorthUiMountedFrameExecutionStop::HostMeasurement(_) => {}
        WorthUiMountedFrameExecutionStop::HostMeasurementTransition(denial) => {
            observe_host_measurement_transition(denial)
        }
        WorthUiMountedFrameExecutionStop::FrameworkTransition(transition) => {
            let _ = transition.generation_identity();
        }
        WorthUiMountedFrameExecutionStop::Preparation(_) => {}
    }
}

fn observe_publication(receipt: &UiMountedFramePublicationReceipt) {
    let _ = receipt.cost_report();
}

fn observe_rejection(rejection: &UiMountedRejectedFrame) {
    let _ = rejection.cost_report();
}

fn observe_in_flight(in_flight: &UiMountedPresentationInFlight) {
    let _ = in_flight.cost_report();
}

fn observe_indeterminate(indeterminate: &UiMountedIndeterminateFrame) {
    let _ = indeterminate.cost_report();
}

fn observe_retention_denial(rejection: &UiMountedFrameRetentionRejection) {
    let _ = rejection.frame().cost_report();
}

fn observe_admission_denial(rejection: &UiMountedPresentationAdmissionRejection) {
    let _ = rejection.frame().cost_report();
}

fn observe_completion_denial(_denial: &UiMountedPresentationCompletionDenial) {}

fn observe_superseded(superseded: &UiMountedSupersededFrame) {
    let _ = superseded.cost_report();
}

fn observe_host_measurement_transition(denial: &UiMountedHostMeasurementTransitionDenial) {
    use UiMountedHostMeasurementTransitionDenial as Denial;
    use UiMountedHostMeasurementUnexpectedTransition as Unexpected;

    match denial {
        Denial::AllocationReplanDenied(_)
        | Denial::ViewportResizeDenied(_)
        | Denial::AllocationReplanSelectionDenied(_)
        | Denial::AllocationFrameResolutionDenied(_)
        | Denial::AllocationInvalidationNarrowingDenied(_)
        | Denial::FrameworkTransitionPlanningDenied(_)
        | Denial::FrameworkTransitionExecutionDenied(_)
        | Denial::DispatcherDenied { .. } => {}
        Denial::UnexpectedSuccessfulTransition(unexpected) => match unexpected {
            Unexpected::ReadyToExecute
            | Unexpected::ResizePreviewPublished
            | Unexpected::DurableResizeCommitted
            | Unexpected::DragResizePreviewPending => {}
        },
    }
}
```

## Real Example

This integration fragment assumes an active `session`, application input
collector, clock, and product-specific outcome handlers:

```rust
use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline,
    WorthUiMountedFrameExecutionStop,
};

let outcome = match session.execute_mounted_frame(
    UiMountedFrameRequest::all_bound_surfaces(),
    UiPresentationDeadline::at_tick(clock.deadline()),
    clock.now(),
    |sources| collect_application_inputs(sources),
) {
    Ok(outcome) => outcome,
    Err(WorthUiMountedFrameExecutionStop::PublicationLease(denial)) => {
        return retry_after_publication_lease(denial);
    }
    Err(WorthUiMountedFrameExecutionStop::HostMeasurement(denial)) => {
        return retry_after_host_measurement(denial);
    }
    Err(WorthUiMountedFrameExecutionStop::HostMeasurementTransition(denial)) => {
        return preserve_predecessor(denial);
    }
    Err(WorthUiMountedFrameExecutionStop::FrameworkTransition(stop)) => {
        return report_generation_stop(stop.generation_identity());
    }
    Err(WorthUiMountedFrameExecutionStop::Preparation(denial)) => {
        return preserve_predecessor(denial);
    }
};

match outcome {
    UiMountedFrameOutcome::Published(receipt)
    | UiMountedFrameOutcome::Unchanged(receipt)
    | UiMountedFrameOutcome::Reconciled(receipt) => publish_observation(receipt),
    UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
        preserve_predecessor(rejection)
    }
    UiMountedFrameOutcome::InFlight(handle) => retain_in_flight(handle),
    UiMountedFrameOutcome::PresentationIndeterminate(handle) => {
        require_typed_reconciliation(handle)
    }
    UiMountedFrameOutcome::Superseded(frame) => preserve_predecessor(frame),
    UiMountedFrameOutcome::RetentionDenied(denial) => preserve_predecessor(denial),
    UiMountedFrameOutcome::AdmissionDenied(denial) => preserve_predecessor(denial),
    UiMountedFrameOutcome::CompletionDenied(denial) => preserve_predecessor(denial),
}
```

The advanced branches begin with typed values returned by the ordinary call.
The app may retain or route those values, but it cannot fabricate them from an
identity, host result, or inspection receipt.

## How It Relates To Other Features

- Add file or Rust composition as described in
  [Authored composition](./authored-composition.md).
- Register installed scalar or collection projections before `freeze`; submit
  Query-issued observations through `begin_projection_rebind(...)`.
- Use [Application inspection](./inspection.md) on the prepared app or active
  session.
- Host adapters consume only the sealed mounted mechanics prepared by runtime.

## Inspection And Debugging

Use `generation_identity()` to correlate lifecycle outcomes. Use `inspect(...)`
for generation-bound read-only evidence. Publication and denial cost reports
describe work already performed; materialize rich reports only when needed.

## Anti-Patterns

- Calling a raw frame transition or lane executor.
- Importing runtime, mounting, or publication internals.
- Treating an identity, digest, or receipt as executable authority.
- Assuming every non-success is safe to retry the same way.
- Rebuilding Query or host state inside application callbacks.

## Current Limits

Surface registration, mounting, and allocation setup depend on the
application’s declared capabilities. The empty example is useful for lifecycle
shape; it is not a substitute for a real application’s graph and host setup.

## Platform Pulse

Platform Pulse is the permanent human-visible application for the real Worth UI
lifecycle. The Milestone 3.15 source-level composition is a restrained desktop
workbench with a 960-by-600 logical client area, a 24-pixel outer gutter, an
eight-point spacing rhythm, and these reference regions:

| Region | Default logical rectangle |
| --- | --- |
| identity masthead | 912 by 56 at `[24, 24]` |
| evidence rail | 216 by 424 at `[24, 104]` |
| primary service stage | 672 by 424 at `[264, 104]` |
| truthful status band | 912 by 24 at `[24, 552]` |

The checked-in source declares the Mosaic regions, Query-backed status,
application action, portal open/close routes, typed command routes, and
Pulse-private palette roles. The evidence rail presents product facts; it is
not a framework inspector. The host paints and reports mechanics but does not
own portal, focus, motion, command, scroll, selection, Query, or application
meaning.

### Launch

From the repository root:

```powershell
cargo run --manifest-path workspaces/worth-ui/Cargo.toml -p worth-ui-platform-pulse
```

The no-argument workflow uses the checked-in source and sample product inputs.
To use another isolated installation, supply existing absolute roots:

```powershell
$sourceRoot = (Resolve-Path workspaces/worth-ui/apps/platform-pulse/app).Path
$queryRoot = (Resolve-Path workspaces/worth-ui/apps/platform-pulse/query_samples).Path
$intentRoot = (Resolve-Path workspaces/worth-ui/apps/platform-pulse/intent_samples).Path
cargo run --manifest-path workspaces/worth-ui/Cargo.toml -p worth-ui-platform-pulse -- `
  --source-root $sourceRoot --query-source-root $queryRoot --intent-source-root $intentRoot
```

The versioned `WORTH_UI_PLATFORM_PULSE_EVENT ` stream reports bounded product
observations such as `FirstFramePublished`, `IntentInputAdmitted`,
`IntentExecutorStarted`, `QueryAction`, `IntentCausalTrace`,
`QueryProjectionIssued`, `QueryProjectionPublished`, `RebindPublished`,
`RebindDeniedPreserving`, `VisualComparison`, and `ShutdownCompleted`.
These envelopes are reporting data. They cannot construct authority or certify
their own pixels.

### Milestone 3.15 Journey And Evidence Boundary

The required native journey opens an anchored portal through an admitted
action, places focus and samples entrance motion, resolves one typed shortcut
in two contexts, preserves the separate Query-backed action boundary, resizes to
1120 by 700, hot-rebinds while the portal is open, dismisses only the topmost
portal, restores lawful focus, and shuts down with exact-zero service and host
resources.

The closing native lane proved externally observed pixels and structure at
both sizes, real operating-system input, resize/rebind behavior, text
containment and contrast, exact cleanup, and the reference visual contract.
The structural checks remain independent from the recorded product/design
review that accepted the whole real-runtime composition against the
contemporary Linear-or-Notion quality bar. A synthetic raster, in-process
reenactment, direct service call, adapter injection, or successful compile
still cannot substitute for either evidence class.

The Pulse remains an ordinary bounded product world: at most 128 mounted nodes,
including portal descendants and exit retention. Large-scale service
amplification belongs only to the scheduled scale courtroom. Full-frame capture
above device scale 4 is unsupported; the largest admitted 1120-by-700 scale-4
RGBA8 capture is bounded to 50,176,000 bytes.

### Appearance And Theme Lifecycle

The prepared application admits appearance roles, a typed theme-slot catalog,
complete theme definitions, explicit component attachments, and authored
Backdrop/Portal relations. Launch selects the bundle's initial theme. Mounted
preparation resolves role cells from the current theme and coherent UI-owned
state, then composes accepted Motion samples and relational overlay order before
the host can receive the frame.

A live theme switch is an application operation with exact predecessor and
surface authority. It prepares only affected consumers, publishes through the
ordinary mounted boundary, and commits the selected theme only after host
acceptance. Source replacement follows the same rule: the owner-state
succession, text publication, appearance, Motion, overlays, and retention used
to derive successor pixels are the results committed on acceptance. Rejection
preserves predecessor paint and state; retry consumes the returned evidence.

UI owners remain authoritative for hover, pressed, selection, focus,
operability, validation, Portal state, and Motion tracks. Appearance reads
their sealed facts and cannot mutate interaction or service state. See
[Appearance and themes](./appearance-and-themes.md) for authoring and limits.

### Inspection And Cleanup

Use the active session's bounded runtime-service `why_*` methods and
`runtime_service_resource_census()` alongside the existing lifecycle,
mounted, Query, and visual receipts. No single projection is an independent
oracle for another production projection.

Normal window close consumes the active application shutdown path. The public
Pulse terminal observation reports joined source watchers and its Query,
intent, visual, capture, and host cleanup fields. The internal application
shutdown receipt separately requires the runtime-service census—including
family records, proposal state, command prefixes, motion tracks, and portal
retention—to be empty. Physical focus placement is not a census row: the
runtime's separate `UiFocusHostPlacementShutdownReport` records any abandoned
indeterminate host request. A nonzero census row or abandoned request is a
lifecycle failure, not a logging detail to suppress. The closing native
evidence projected and observed both forms of cleanup externally.

The historical documentation-lane baseline and its then-remaining ownership
are recorded in
[Milestone 3.15 documentation closeout](../../../_docs/worth-ui/milestone-3.15-documentation-closeout.md).

## Related Docs

- [Worth UI architecture](./architecture.md)
- [Authored composition](./authored-composition.md)
- [Interaction and intents](./interaction-and-intents.md)
- [Runtime services](./runtime-services.md)
- [Hot rebind](./hot-rebind.md)
- [Application inspection](./inspection.md)
- [Query-backed UI views](./query-binding.md)
