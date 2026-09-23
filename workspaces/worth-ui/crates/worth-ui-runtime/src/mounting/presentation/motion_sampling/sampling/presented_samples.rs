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
        let mut matches = self.tracks.values().filter_map(|state| {
            considered += 1;
            // A contents-group target names a group placed inside the
            // instance, not the instance itself, so it can never answer a
            // lookup keyed by mounted instance. Portal content and Scroll
            // content are both resolved by target.
            if !state.presented || state.track.target().scope() != UiMotionTargetScope::Ordinary {
                return None;
            }
            let sample = state.current?;
            (sample.target().mounted_instance() == mounted_instance
                && same_surface_binding(sample.geometry()?.presentation_basis(), presentation))
            .then_some(sample)
        });
        let sample = matches.next();
        let unique = matches.next().is_none();
        (sample.filter(|_| unique), considered)
    }

    pub(crate) fn current_sample_for_target(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Option<super::super::UiPresentationMotionSampleReceipt> {
        let state = self.tracks.get(&target)?;
        if !state.presented {
            return None;
        }
        let sample = state.current?;
        same_surface_binding(sample.geometry()?.presentation_basis(), presentation)
            .then_some(sample)
    }
}
