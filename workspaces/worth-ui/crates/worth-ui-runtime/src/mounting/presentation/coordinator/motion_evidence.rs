use super::{UiMotionSamplePresentationOutcome, UiMountedPresentationCoordinator};
use crate::mounting::UiPresentedFrameBasisDenial;
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiMountedPaintCommandIdentity, UiSurfaceBindingGeneration,
};

impl UiMountedPresentationCoordinator {
    pub(in crate::mounting) fn motion_appearance_instances(
        &self,
        binding: UiSurfaceBindingGeneration,
        targets: &[crate::runtime::motion::UiMotionTargetIdentity],
    ) -> Vec<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let state = self
            .presentation_states
            .get(&binding)
            .expect("accepted Motion retains its current command owner");
        let mut instances = Vec::new();
        for target in targets {
            if let Some(group) = state.portal_motion_group(*target) {
                instances.extend(group.commands().map(|command| command.mounted_instance()));
            } else {
                instances.extend(
                    state
                        .command_identities_for_instance(target.mounted_instance())
                        .map(|command| command.mounted_instance()),
                );
            }
        }
        instances.sort_unstable();
        instances.dedup();
        instances
    }

    pub(super) fn accept_motion_evidence(
        &mut self,
        settled: UiMotionSamplePresentationOutcome,
        acceptance: super::super::work_producer::UiPreparedCommandMotionAcceptance,
        binding: UiSurfaceBindingGeneration,
    ) -> UiMotionSamplePresentationOutcome {
        let failed = match &settled {
            UiMotionSamplePresentationOutcome::Presented { presentation, .. } => {
                self.reconstruction_bindings.contains(&binding)
                    || self
                        .presentation_states
                        .get(&binding)
                        .is_none_or(|state| acceptance.accept(state, *presentation).is_err())
            }
            UiMotionSamplePresentationOutcome::PresentationIndeterminate => true,
            _ => false,
        };
        if failed {
            self.reconstruction_bindings.insert(binding);
            return UiMotionSamplePresentationOutcome::PresentationIndeterminate;
        }
        settled
    }

    /// Reads live physical evidence through the current command owner only.
    /// The returned receipt records when the sample was accepted; unchanged
    /// commands can retain it across subsequent physical epochs and frames.
    pub(in crate::mounting) fn accepted_motion_for_command(
        &self,
        presentation: UiHostObservationPresentationBasis,
        command: UiMountedPaintCommandIdentity,
    ) -> Result<
        Option<super::super::motion_sampling::UiPresentationMotionSampleReceipt>,
        UiPresentedFrameBasisDenial,
    > {
        if self
            .reconstruction_bindings
            .contains(&presentation.binding())
        {
            return Err(UiPresentedFrameBasisDenial::PresentationTruthUnavailable);
        }
        let state = self
            .presentation_states
            .get(&presentation.binding())
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        let requirement = state.motion_sample_requirement();
        if self
            .host_truth
            .surface_requires_reconciliation(requirement.semantic_surface())
        {
            return Err(UiPresentedFrameBasisDenial::PresentationTruthUnavailable);
        }
        if state.frame() != presentation.frame() {
            return Err(UiPresentedFrameBasisDenial::Expired);
        }
        if requirement.host_surface() != presentation.host_surface() {
            return Err(UiPresentedFrameBasisDenial::BindingNotPresented);
        }
        state
            .motion_for_command(command)
            .ok_or(UiPresentedFrameBasisDenial::Unknown)
    }
}
