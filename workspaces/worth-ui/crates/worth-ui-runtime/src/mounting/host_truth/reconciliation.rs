use worth_ui_host_contract::{UiMountedFrameIdentity, UiSurfaceBindingGeneration};

use super::binding_truth::UiMountedHostTruthCoordinator;
use crate::mounting::{
    UiHostPresentationReconciliation, UiMountedSurfaceReconciliationBinding, UiPreparedMountedFrame,
};

impl UiMountedHostTruthCoordinator {
    pub(crate) fn reconcile_presentation(
        &mut self,
        reconciliation: UiHostPresentationReconciliation,
        current_frame: Option<UiMountedFrameIdentity>,
    ) -> bool {
        let binding = reconciliation.affected_binding();
        let Some(requirement) = self.blocked_presentation_requirement(binding) else {
            return false;
        };
        if !reconciliation.proves(requirement, current_frame) {
            return false;
        }
        let surface = requirement.semantic_surface();
        self.clear_presentations_for_surface(surface);
        true
    }

    pub(crate) fn reconcile_candidate_only_deregistration(
        &mut self,
        binding: UiSurfaceBindingGeneration,
    ) {
        // Retiring an unpublished candidate does not reconstruct the published
        // predecessor on the same surface. Only discharge this binding's paint.
        if let Some(blocked) = self.blocked.get_mut(&binding) {
            blocked.clear_presentation();
            if blocked.is_empty() {
                self.blocked.remove(&binding);
            }
        }
    }

    pub(crate) fn reconciliation_covers(
        &self,
        frame: &UiPreparedMountedFrame,
        replacements: &[UiMountedSurfaceReconciliationBinding],
    ) -> bool {
        !replacements.is_empty()
            && replacements
                .iter()
                .all(|replacement| self.replacement_is_valid(frame, *replacement))
            && frame.surfaces().iter().all(|surface| {
                !replacements
                    .iter()
                    .any(|replacement| surface.requirement().binding() == replacement.affected())
            })
            && self
                .blocked
                .values()
                .all(|blocked| blocked.native_lifecycle_obligation().is_none())
            && self.blocked.values().all(|blocked| {
                blocked.presentation_requirement().is_none()
                    || replacements.iter().any(|replacement| {
                        self.blocked_presentation_requirement(replacement.affected())
                            .is_some_and(|covered| {
                                covered.semantic_surface() == blocked.semantic_surface()
                            })
                    })
            })
    }

    pub(crate) fn commit_current_frame_reconciliation(
        &mut self,
        replacements: &[UiMountedSurfaceReconciliationBinding],
    ) {
        for replacement in replacements {
            let Some(surface) = self
                .blocked_presentation_requirement(replacement.affected())
                .map(|blocked| blocked.semantic_surface())
            else {
                continue;
            };
            self.clear_presentations_for_surface(surface);
        }
    }

    fn replacement_is_valid(
        &self,
        frame: &UiPreparedMountedFrame,
        replacement: UiMountedSurfaceReconciliationBinding,
    ) -> bool {
        let Some(blocked) = self.blocked_presentation_requirement(replacement.affected()) else {
            return false;
        };
        frame.surfaces().iter().any(|surface| {
            let requirement = surface.requirement();
            requirement.binding() == replacement.replacement()
                && requirement.semantic_surface() == blocked.semantic_surface()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiHostSurfaceRegistrationInput,
        UiHostSurfaceRegistrationRequest, UiMountedSurfaceBindingRequirement,
        UiSemanticSurfaceIdentity,
    };

    #[test]
    fn retiring_candidate_preserves_published_surface_reconstruction_obligation() {
        let plan = crate::facade::prepared_application_authority::WorthUiHostSessionPlan::prepare(
            crate::certification_support::UiCertificationBuilderHost,
        );
        let session = crate::facade::WorthUiHostSessionAuthority::activate(&plan).unwrap();
        let report = session.capability_report();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let host_surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
        let requirement = || {
            let request =
                UiHostSurfaceRegistrationRequest::from_runtime(UiHostSurfaceRegistrationInput {
                    host_session_identity: session.identity().as_u64(),
                    semantic_surface_identity: surface,
                    host_surface_identity: host_surface,
                    binding_generation: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
                    protocol: session.protocol(),
                    capability_generation: report.observation_generation(),
                    capability_profile_digest: report.profile_identity_digest(),
                    presentation_mode: UiHostSurfacePresentationMode::NativeDisplay,
                });
            UiMountedSurfaceBindingRequirement::with_baseline(
                surface,
                host_surface,
                request.binding_generation(),
                report.observation_generation(),
                report.profile_identity_digest(),
                request.presentation_mode(),
                request.baseline_identity(),
            )
        };
        let published = requirement();
        let candidate = requirement();
        let mut truth = UiMountedHostTruthCoordinator::default();
        truth.block_presentation(published);
        truth.block_presentation(candidate);

        truth.reconcile_candidate_only_deregistration(candidate.binding());

        assert_eq!(
            truth.blocked_presentation_requirement(published.binding()),
            Some(published)
        );
        assert_eq!(
            truth.blocked_presentation_requirement(candidate.binding()),
            None
        );
        assert!(truth.surface_requires_reconciliation(surface));
    }
}
