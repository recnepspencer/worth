//! Per-frame work budgets for the product application's border drag, counted
//! offscreen. The counts are deterministic, so each budget is a ceiling at
//! the work the host does today: a change that does more fails, and a change
//! that does less lowers the ceiling with it.
//!
//! Digested bytes also count work the presenting thread does for the
//! application's own threads, such as a late startup Query observation
//! (see `worth_ui_host_contract::presentation_work`). Startup settles on a
//! quiet window, which a loaded machine can outlast, so that one-off work can
//! land in a measured step. The drag therefore repeats both transitions, and
//! the byte ceiling holds the lesser of the two identical frames: one-off
//! work lands in at most one of them, while work every frame does stays in
//! both. Glyph and insert counts are charged on the glyph path alone and are
//! held on the first frame.
//!
//! Milestone 3.16.2 lowers these ceilings. Block-level retained text takes a
//! move-only frame to zero glyphs in every text stage, and a rewrap frame to
//! the glyphs of the blocks it rewraps.

use std::sync::OnceLock;

use worth_ui_native_platform::{
    UiNativeOffscreenSettle, UiNativeSubmittedFrameWork, UiPresentationWorkStage,
};
use worth_ui_platform_pulse::visual_identity_pulse::PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT;

use super::{drag, PlatformPulseDragStep};
use crate::launch_configuration::AdmittedPlatformPulseLaunchConfiguration;
use crate::lifecycle_observation_publication::PlatformPulseObservationPublisher;

const PRODUCT: [u32; 2] = PLATFORM_PULSE_PRODUCT_LOGICAL_EXTENT;

/// Shorter by 16 pixels, which moves text without rewrapping it, then
/// narrower by 16 pixels, which rewraps it.
const MOVE_ONLY: [u32; 2] = [PRODUCT[0], PRODUCT[1] - 16];
const REWRAP: [u32; 2] = [PRODUCT[0] - 16, PRODUCT[1] - 16];

/// Both transitions, back to the product extent, then both again.
const PATH: [[u32; 2]; 5] = [MOVE_ONLY, REWRAP, PRODUCT, MOVE_ONLY, REWRAP];

const STAGES: usize = UiPresentationWorkStage::ALL.len();

/// One frame's work: glyphs per stage in [`UiPresentationWorkStage::ALL`]
/// order, then digested bytes, then map inserts.
type Counts = [u64; STAGES + 2];

const MOVE_ONLY_CEILING: Counts = [
    3087, 1474, 1474, 1084, 1084, 1379, 2948, 1694, 5628, 2151, 2_134_726, 8502,
];

const REWRAP_CEILING: Counts = [
    3002, 1434, 1434, 1045, 1045, 1379, 2868, 2560, 5999, 2113, 2_104_168, 8303,
];

struct Step {
    settle: UiNativeOffscreenSettle,
    frames: Vec<Counts>,
}

/// The drag's steps along [`PATH`], run once for every budget.
fn steps() -> &'static [Step; 5] {
    static STEPS: OnceLock<[Step; 5]> = OnceLock::new();
    STEPS.get_or_init(|| {
        let publisher = PlatformPulseObservationPublisher::start().expect("observation stream");
        let launch = AdmittedPlatformPulseLaunchConfiguration::from_arguments(std::iter::empty())
            .expect("default launch");
        let drag = drag(launch, publisher, &PATH).expect("offscreen drag");
        assert!(
            matches!(drag.startup, UiNativeOffscreenSettle::Idle { .. }),
            "startup settles: {:?}",
            drag.startup
        );
        let extents = drag.steps.iter().map(|step| step.extent);
        assert!(extents.eq(PATH), "one step per extent");
        <[Step; 5]>::try_from(drag.steps.iter().map(step).collect::<Vec<_>>())
            .unwrap_or_else(|_| panic!("one step per extent"))
    })
}

fn step(step: &PlatformPulseDragStep) -> Step {
    Step {
        settle: step.settle,
        frames: step.frames.iter().map(counts).collect(),
    }
}

fn counts(frame: &UiNativeSubmittedFrameWork) -> Counts {
    let work = frame.work();
    let mut counts = [0; STAGES + 2];
    for (count, stage) in counts.iter_mut().zip(UiPresentationWorkStage::ALL) {
        *count = work.glyphs(stage);
    }
    counts[STAGES] = work.digested_bytes();
    counts[STAGES + 1] = work.map_inserts();
    counts
}

fn only_frame(step: &Step) -> &Counts {
    assert!(
        matches!(step.settle, UiNativeOffscreenSettle::Idle { .. }),
        "the step settles: {:?}",
        step.settle
    );
    let [frame] = step.frames.as_slice() else {
        panic!("one frame per step: {:?}", step.frames);
    };
    frame
}

/// Holds `first` to `ceiling`, except digested bytes, which hold the lesser
/// of `first` and `repeat`, the same transition made again.
fn assert_within(first: &Step, repeat: &Step, ceiling: &Counts) {
    let frame = only_frame(first);
    let repeated = only_frame(repeat);
    let mut counts = *frame;
    counts[STAGES] = frame[STAGES].min(repeated[STAGES]);
    let names = UiPresentationWorkStage::ALL
        .iter()
        .map(|stage| stage.name())
        .chain(["digested bytes", "map inserts"]);
    for ((name, count), ceiling) in names.zip(&counts).zip(ceiling) {
        assert!(
            count <= ceiling,
            "{name}: {count} exceeds the ceiling {ceiling}; frame {frame:?}, repeat {repeated:?}"
        );
    }
}

#[test]
fn a_move_only_frame_stays_within_its_budget() {
    let steps = steps();
    assert_within(&steps[0], &steps[3], &MOVE_ONLY_CEILING);
}

#[test]
fn a_rewrap_frame_stays_within_its_budget() {
    let steps = steps();
    assert_within(&steps[1], &steps[4], &REWRAP_CEILING);
}
