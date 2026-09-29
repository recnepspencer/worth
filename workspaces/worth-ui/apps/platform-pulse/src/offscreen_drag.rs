//! Drives the product application offscreen through a drag of its client
//! area, so each step's presentation work is counted without a window or a
//! person.
//!
//! The run renders every frame into an offscreen target and never presents
//! one on screen: it measures the work each frame does, not its pacing. Each
//! step is an ordinary loop turn, not a platform's modal border drag.

use std::process::ExitCode;
use std::time::Duration;

use worth_ui_native_platform::{
    UiNativeOffscreenSettle, UiNativeOffscreenStart, UiNativePlatformOutcome,
    UiNativePlatformProfile, UiNativeSubmittedFrameWork, UiNativeWindowSpec,
    UiPresentationWorkStage, WorthUiNativePlatform,
};
use worth_ui_platform_pulse::visual_identity_pulse::{
    PLATFORM_PULSE_MINIMUM_LOGICAL_EXTENT, PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT,
};

use crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration;
use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

/// Runs the offscreen qualification drag instead of the product.
pub(crate) const FLAG: &str = "--worth-ui-offscreen-drag";

/// The most turns one step may take to settle. A step that needs more is
/// reported, not waited on.
const MAX_TURNS_PER_STEP: u32 = 10_000;

/// How long the started application must post no wake before the drag
/// begins. Its threads observe their sources at most 250 ms after they
/// open, so a quiet second means startup is over.
const STARTUP_QUIET: Duration = Duration::from_secs(1);

/// One step of a drag: the extent the client area was dragged to, and the
/// frames the host submitted before it settled.
pub(crate) struct PlatformPulseDragStep {
    pub(crate) extent: [u32; 2],
    pub(crate) settle: UiNativeOffscreenSettle,
    pub(crate) frames: Vec<UiNativeSubmittedFrameWork>,
}

/// How startup settled and the frames the application submitted before the
/// drag, then one step per extent of `path`.
pub(crate) struct PlatformPulseDrag {
    pub(crate) startup: UiNativeOffscreenSettle,
    pub(crate) settled: Vec<UiNativeSubmittedFrameWork>,
    pub(crate) steps: Vec<PlatformPulseDragStep>,
}

/// Opens the product application offscreen at its product extent and scale
/// 1, lets it settle until it posts nothing for [`STARTUP_QUIET`], then
/// drags its client area through `path`.
pub(crate) fn drag(
    launch: AdmittedPlatformPulseLaunchConfiguration,
    publisher: PlatformPulseObservationPublisher,
    path: &[[u32; 2]],
) -> Result<PlatformPulseDrag, String> {
    let profile = UiNativePlatformProfile::single_window(
        UiNativeWindowSpec::new(
            "WORTH UI Platform Pulse",
            PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT,
        )
        .with_minimum_logical_size(PLATFORM_PULSE_MINIMUM_LOGICAL_EXTENT),
    );
    let platform = WorthUiNativePlatform::prepare(profile)
        .map_err(|denial| format!("platform preparation failed: {denial:?}"))?;
    let application =
        crate::native_application::PlatformPulseApplication::offscreen(launch, publisher);
    let mut session =
        match platform.start_offscreen(application, PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT, 1.0) {
            UiNativeOffscreenStart::Started(session) => session,
            UiNativeOffscreenStart::Ended(outcome) => {
                return Err(format!("offscreen start ended: {outcome:?}"));
            }
        };
    let mut startup = session.run_until_idle(MAX_TURNS_PER_STEP);
    while matches!(startup, UiNativeOffscreenSettle::Idle { .. })
        && session.await_wake(STARTUP_QUIET)
    {
        startup = session.run_until_idle(MAX_TURNS_PER_STEP);
    }
    let settled = session.take_frame_work();
    let steps = path
        .iter()
        .map(|&extent| {
            session.resize(extent);
            let settle = session.run_until_idle(MAX_TURNS_PER_STEP);
            PlatformPulseDragStep {
                extent,
                settle,
                frames: session.take_frame_work(),
            }
        })
        .collect();
    match session.close(MAX_TURNS_PER_STEP) {
        UiNativePlatformOutcome::Closed(receipt) if receipt.terminal_census().is_zero() => {
            Ok(PlatformPulseDrag {
                startup,
                settled,
                steps,
            })
        }
        outcome => Err(format!("offscreen close ended: {outcome:?}")),
    }
}

/// The drag the `--worth-ui-offscreen-drag` run performs: narrower in 16
/// pixel steps, which rewraps, then shorter in 16 pixel steps at the
/// narrowest width.
fn qualification_path() -> Vec<[u32; 2]> {
    let [width, height] = PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT;
    let narrowest = width - 16 * 16;
    let narrower = (1..=16).map(|step| [width - 16 * step, height]);
    let shorter = (1..=16).map(|step| [narrowest, height - 16 * step]);
    narrower.chain(shorter).collect()
}

/// Runs the qualification drag with the launch options given beside the
/// flag, and prints each submitted frame's work: `startup <settle>`, then
/// `step <width> <height> <settle>` per step, each followed per frame by
/// `work <frame> <width> <height>`, the glyphs of each stage, digested
/// bytes, map inserts, and allocations (`-` when uncounted). The product's
/// own prefixed lifecycle observation lines share standard output.
pub(crate) fn run() -> ExitCode {
    let publisher = match PlatformPulseObservationPublisher::start() {
        Ok(publisher) => publisher,
        Err(denial) => {
            eprintln!("WORTH UI offscreen drag observation stream could not start: {denial:?}");
            return ExitCode::FAILURE;
        }
    };
    let options = std::env::args_os()
        .skip(1)
        .filter(|argument| argument != FLAG);
    let launch = match AdmittedPlatformPulseLaunchConfiguration::from_arguments(options) {
        Ok(launch) => launch,
        Err(denial) => {
            eprintln!("WORTH UI offscreen drag launch was denied: {denial:?}");
            return ExitCode::from(2);
        }
    };
    let drag = match drag(launch, publisher, &qualification_path()) {
        Ok(drag) => drag,
        Err(failure) => {
            eprintln!("WORTH UI offscreen drag failed: {failure}");
            return ExitCode::from(3);
        }
    };
    println!("startup {:?}", drag.startup);
    print_frames(&drag.settled);
    for step in &drag.steps {
        println!(
            "step {} {} {:?}",
            step.extent[0], step.extent[1], step.settle
        );
        print_frames(&step.frames);
    }
    ExitCode::SUCCESS
}

fn print_frames(frames: &[UiNativeSubmittedFrameWork]) {
    for frame in frames {
        let work = frame.work();
        let stages = UiPresentationWorkStage::ALL
            .iter()
            .map(|&stage| work.glyphs(stage).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let allocations = work
            .allocations()
            .map_or_else(|| "-".to_owned(), |count| count.to_string());
        println!(
            "work {} {} {} {stages} {} {} {allocations}",
            frame.frame(),
            frame.extent()[0],
            frame.extent()[1],
            work.digested_bytes(),
            work.map_inserts(),
        );
    }
}

#[cfg(test)]
#[path = "offscreen_drag_tests.rs"]
mod tests;
