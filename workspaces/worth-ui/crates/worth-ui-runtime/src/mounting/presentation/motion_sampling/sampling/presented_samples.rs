use super::{same_surface_binding, UiMountedMotionSampler};
use crate::runtime::motion::UiMotionTargetScope;

impl UiMountedMotionSampler {
    pub(in crate::mounting) fn current_sample_for_with_work(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> (
        Option<super::super::UiPresentationMotionSampleReceipt>,
        usize,
    ) {
        let mut considered = 0;
        let mut matches = self.tracks.states().filter_map(|state| {
            considered += 1;
            // A contents-group target names a group placed inside the
            // instance, not the instance itself, so it can never answer a
            // lookup keyed by mounted instance. Portal content and Scroll
            // content are both resolved by target.
            if state.track.target().scope() != UiMotionTargetScope::Ordinary {
                return None;
            }
            let sample = state.on_screen()?;
            (sample.target().mounted_instance() == mounted_instance
                && same_surface_binding(sample.geometry()?.presentation_basis(), presentation))
            .then_some(sample)
        });
        let sample = matches.next();
        let unique = matches.next().is_none();
        (sample.filter(|_| unique), considered)
    }

    /// Whether a sample of `target` reached the screen after `presentation`,
    /// so a reader acting on that presentation saw the target elsewhere.
    pub(crate) fn target_presented_after(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> bool {
        self.tracks
            .get(&target)
            .and_then(|state| state.on_screen())
            .and_then(|sample| sample.geometry())
            .is_some_and(|geometry| {
                let shown = geometry.presentation_basis();
                !same_surface_binding(shown, presentation) || shown.epoch() > presentation.epoch()
            })
    }

    pub(crate) fn current_sample_for_target(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Option<super::super::UiPresentationMotionSampleReceipt> {
        let sample = self.tracks.get(&target)?.on_screen()?;
        same_surface_binding(sample.geometry()?.presentation_basis(), presentation)
            .then_some(sample)
    }
}
