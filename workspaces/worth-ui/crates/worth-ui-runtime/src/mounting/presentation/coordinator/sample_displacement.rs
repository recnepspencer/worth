//! Holding a tick back from a frame in flight that displaces its samples.
//!
//! Once a frame's work lands, the host draws each command the work touches or
//! re-samples, outside a group the frame places, where the frame was
//! prepared, whatever sample a tick showed it through beside the frame. A
//! tick sampling such a command would be undone when the frame lands, leaving
//! the command apart from the group it moves with and from hit testing. So
//! the tick waits for the frame, as it waits for a sample still in flight.

use worth_ui_host_contract::{UiMountedPresentationWorkView, UiSurfaceBindingGeneration};

use super::super::UiMountedPresentationWork;
use super::UiMountedPresentationCoordinator;

impl UiMountedPresentationCoordinator {
    /// Whether a frame still in flight on `binding` displaces the sample of
    /// a command `work` samples.
    pub(super) fn frame_in_flight_displaces(
        &self,
        binding: UiSurfaceBindingGeneration,
        work: &UiMountedPresentationWork,
    ) -> bool {
        let UiMountedPresentationWorkView::Sample(sample) = work.view() else {
            return false;
        };
        self.in_flight
            .values()
            .filter(|flight| {
                flight
                    .pending
                    .iter()
                    .any(|surface| surface.binding == binding)
            })
            .filter_map(|flight| flight.candidates.get(&binding))
            .any(|candidate| {
                sample
                    .changes()
                    .iter()
                    .any(|change| candidate.displaces_sample_of(change.command()))
            })
    }
}
