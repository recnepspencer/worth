//! The frame that lands a Portal's focus reveal.
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::mounted::UiMountedFrameRequest;
use worth_ui_test_support::{
    UiScrollRuntimeCertificationSnapshot, WorthUiActiveSessionCertificationExt,
    WorthUiServiceStateCertificationExt,
};

use super::super::world::PayloadWorld;

/// A focus reveal is a direct placement, like a wheel offset: Scroll holds
/// `predecessor` until the frame carrying the reveal is accepted. Publish that
/// frame and return Scroll as it lands.
pub(super) fn publish_focus_reveal_frame(
    world: &mut PayloadWorld,
    recorder: &WorthUiHeadlessRecorder,
    predecessor: i64,
) -> UiScrollRuntimeCertificationSnapshot {
    assert_eq!(
        world
            .interaction
            .session
            .inspect_scroll_runtime_for_certification()
            .owner_geometry()[0]
            .block_offset_subpixels(),
        predecessor,
        "a pending reveal leaves the accepted offset where the host displays it"
    );
    let _ = recorder.drain_transcripts();
    let frame = world
        .interaction
        .session
        .prepare_application_presentation_frame(UiMountedFrameRequest::all_bound_surfaces())
        .expect("the reveal frame prepares over the current geometry");
    world.interaction.publish_prepared_successor(frame);
    world
        .interaction
        .session
        .inspect_scroll_runtime_for_certification()
}
