//! One tick sampled from the tracks it was prepared with.

use super::super::{
    UiPresentationMotionSamplePosture, UiPresentationMotionSamplingReceipt,
    UiPresentationMotionTerminalRequest, UiPresentationReducedMotionPosture,
};
use super::track_table::UiMotionTickTracks;
use super::{same_surface_binding, UiPresentationMotionSamplingDenial};

impl UiMotionTickTracks {
    /// Samples every running track at `tick` for `presentation`.
    pub(super) fn sample(
        &mut self,
        tick: u64,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        reduced_motion: UiPresentationReducedMotionPosture,
    ) -> Result<UiPresentationMotionSamplingReceipt, UiPresentationMotionSamplingDenial> {
        let mut samples = Vec::new();
        let mut terminals = Vec::new();
        let mut considered = 0;
        for state in self.running_mut() {
            considered += 1;
            if !same_surface_binding(state.track.successor_presentation(), presentation) {
                state.settle();
                terminals.push(UiPresentationMotionTerminalRequest::new(
                    state.track.identity(),
                    crate::runtime::motion::UiMotionTerminalCause::ReboundAway,
                ));
                continue;
            }
            if reduced_motion == UiPresentationReducedMotionPosture::Reduce {
                let declaration = state.track.declaration();
                if declaration.settles_directly_under_reduced_motion() {
                    let sample = state
                        .snap_system_reduced_motion(tick, presentation)
                        .map_err(UiPresentationMotionSamplingDenial::InvalidSampleGeometry)?;
                    samples.push(sample);
                    terminals.push(UiPresentationMotionTerminalRequest::new(
                        sample.track(),
                        crate::runtime::motion::UiMotionTerminalCause::SnappedToTarget,
                    ));
                    continue;
                }
                if declaration.shortens_under_reduced_motion() {
                    state.shorten_system_reduced_motion();
                }
            }
            let Some(sampled) = state.sample(tick, presentation) else {
                continue;
            };
            let sample = match sampled {
                Ok(sample) => sample,
                Err(denial) => {
                    return Err(UiPresentationMotionSamplingDenial::InvalidSampleGeometry(
                        denial,
                    ));
                }
            };
            samples.push(sample);
            if sample.posture() == UiPresentationMotionSamplePosture::Terminal {
                state.settle();
                terminals.push(UiPresentationMotionTerminalRequest::new(
                    state.track.identity(),
                    crate::runtime::motion::UiMotionTerminalCause::Completed,
                ));
            }
        }
        Ok(UiPresentationMotionSamplingReceipt::new(
            samples, terminals, considered,
        ))
    }
}
