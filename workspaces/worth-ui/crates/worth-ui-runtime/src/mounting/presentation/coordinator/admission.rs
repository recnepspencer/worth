use std::rc::Rc;

use worth_ui_host_contract::{
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiPresentationDeadline, WorthUiHostCapabilityReport,
};

use super::super::{
    UiMountedPresentationAdmission, UiMountedPresentationAdmissionDenial,
    UiMountedPresentationAdmissionRejection, UiMountedSurfaceReconciliationBinding,
};
use super::UiMountedPresentationCoordinator;

impl UiMountedPresentationCoordinator {
    pub(crate) fn admit_current(
        &mut self,
        frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
        capability_report: &WorthUiHostCapabilityReport,
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection> {
        self.admit(frame, capability_report, deadline, now)
    }

    fn admit(
        &mut self,
        frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
        capability_report: &WorthUiHostCapabilityReport,
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection> {
        self.admit_for(frame, capability_report, deadline, now, None)
    }

    pub(crate) fn admit_reconciliation(
        &mut self,
        frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
        replacements: &[UiMountedSurfaceReconciliationBinding],
        capability_report: &WorthUiHostCapabilityReport,
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> Result<UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection> {
        self.admit_for(frame, capability_report, deadline, now, Some(replacements))
    }

    fn admit_for(
        &mut self,
        frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
        capability_report: &WorthUiHostCapabilityReport,
        deadline: UiPresentationDeadline,
        now: u64,
        reconciliation: Option<&[UiMountedSurfaceReconciliationBinding]>,
    ) -> Result<UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection> {
        if self.shutting_down {
            return Err(rejected(
                frame,
                UiMountedPresentationAdmissionDenial::CoordinatorShuttingDown,
            ));
        }
        if deadline.expired_at(now) {
            return Err(rejected(
                frame,
                UiMountedPresentationAdmissionDenial::DeadlineExpired,
            ));
        }
        if self.active.borrow().len() >= self.in_flight_limit {
            return Err(rejected(
                frame,
                UiMountedPresentationAdmissionDenial::CapacityExceeded,
            ));
        }
        if !self.admits_binding_purpose(frame.frame(), reconciliation) {
            return Err(rejected(
                frame,
                UiMountedPresentationAdmissionDenial::ReconciliationBasisMismatch,
            ));
        }
        self.validate_surface_requirements(frame, capability_report, reconciliation, deadline)
    }

    fn validate_surface_requirements(
        &mut self,
        frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
        capability_report: &WorthUiHostCapabilityReport,
        reconciliation: Option<&[UiMountedSurfaceReconciliationBinding]>,
        deadline: UiPresentationDeadline,
    ) -> Result<UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection> {
        for surface in frame.frame().surfaces() {
            let requirement = surface.requirement();
            let denial = if reconciliation.is_none()
                && self
                    .host_truth
                    .surface_requires_reconciliation(requirement.semantic_surface())
            {
                Some(
                    UiMountedPresentationAdmissionDenial::BindingRequiresReconciliation(
                        requirement.binding(),
                    ),
                )
            } else if capability_report.observation_generation()
                != requirement.capability_generation()
            {
                Some(
                    UiMountedPresentationAdmissionDenial::CapabilityGenerationChanged(
                        requirement.binding(),
                    ),
                )
            } else if capability_report.profile_identity_digest()
                != requirement.capability_profile_digest()
            {
                Some(
                    UiMountedPresentationAdmissionDenial::CapabilityProfileChanged(
                        requirement.binding(),
                    ),
                )
            } else {
                baseline_requirement_denial(&self.host_truth, requirement)
            };
            if let Some(denial) = denial {
                return Err(rejected(frame, denial));
            }
        }
        self.reserve_attempt(frame, deadline)
    }

    fn reserve_attempt(
        &mut self,
        frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
        deadline: UiPresentationDeadline,
    ) -> Result<UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection> {
        let attempt = match UiMountedPresentationAttemptIdentity::mint_unbound() {
            Ok(attempt) => attempt,
            Err(_) => {
                return Err(rejected(
                    frame,
                    UiMountedPresentationAdmissionDenial::IdentityExhausted,
                ));
            }
        };
        let candidates = match super::UiPreparedFrameCandidates::prepare(
            frame.frame(),
            &self.presentation_states,
            &self.reconstruction_bindings,
        ) {
            Ok(candidates) => candidates,
            Err(denial) => {
                return Err(rejected(
                    frame,
                    UiMountedPresentationAdmissionDenial::CandidatePreparation(denial),
                ))
            }
        };
        self.active.borrow_mut().insert(attempt);
        Ok(UiMountedPresentationAdmission::new(
            frame,
            attempt,
            deadline,
            Rc::clone(&self.active),
            candidates,
        ))
    }

    fn admits_binding_purpose(
        &self,
        frame: &super::super::super::UiPreparedMountedFrame,
        reconciliation: Option<&[UiMountedSurfaceReconciliationBinding]>,
    ) -> bool {
        let Some(replacements) = reconciliation else {
            return true;
        };
        self.host_truth.reconciliation_covers(frame, replacements)
    }
}

fn baseline_requirement_denial(
    truth: &crate::mounting::UiMountedHostTruthCoordinator,
    requirement: UiMountedSurfaceBindingRequirement,
) -> Option<UiMountedPresentationAdmissionDenial> {
    (!truth.has_live_baseline(requirement.binding(), requirement.baseline())).then_some(
        UiMountedPresentationAdmissionDenial::BaselineReceiptUnavailable(requirement.binding()),
    )
}

fn rejected(
    frame: super::super::super::retention::UiRetentionPreparedMountedFrame,
    denial: UiMountedPresentationAdmissionDenial,
) -> UiMountedPresentationAdmissionRejection {
    let (frame, retention) = frame.into_parts();
    drop(retention);
    UiMountedPresentationAdmissionRejection::new(frame, denial)
}

#[cfg(test)]
mod tests {
    use super::baseline_requirement_denial;
    use crate::certification_support::UiCertificationBuilderHost;
    use crate::facade::prepared_application_authority::WorthUiHostSessionPlan;
    use crate::facade::WorthUiHostSessionAuthority;
    use crate::mounting::UiMountedHostTruthCoordinator;
    use worth_ui_host_contract::{
        UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiHostSurfaceRegistrationInput,
        UiHostSurfaceRegistrationRequest, UiMountedSurfaceBindingRequirement,
        UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    };

    #[test]
    fn actual_baseline_registration_gates_the_presentation_admission_transition() {
        let plan = WorthUiHostSessionPlan::prepare(UiCertificationBuilderHost);
        let session = WorthUiHostSessionAuthority::activate(&plan).unwrap();
        let report = session.capability_report();
        let request =
            UiHostSurfaceRegistrationRequest::from_runtime(UiHostSurfaceRegistrationInput {
                host_session_identity: session.identity().as_u64(),
                semantic_surface_identity: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
                host_surface_identity: UiHostSurfaceIdentity::mint_unbound().unwrap(),
                binding_generation: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
                protocol: session.protocol(),
                capability_generation: report.observation_generation(),
                capability_profile_digest: report.profile_identity_digest(),
                presentation_mode: UiHostSurfacePresentationMode::NativeDisplay,
            });
        let requirement = UiMountedSurfaceBindingRequirement::with_baseline(
            request.semantic_surface_identity(),
            request.host_surface_identity(),
            request.binding_generation(),
            report.observation_generation(),
            report.profile_identity_digest(),
            request.presentation_mode(),
            request.baseline_identity(),
        );
        assert_eq!(
            request.baseline_identity().transparent_rgba8(),
            [0, 0, 0, 0]
        );
        assert_eq!(requirement.baseline().transparent_rgba8(), [0, 0, 0, 0]);
        let mut truth = UiMountedHostTruthCoordinator::default();
        assert!(baseline_requirement_denial(&truth, requirement).is_some());
        truth
            .register_surface(session.effect_port(), request)
            .unwrap();
        let admitted = u64::from(baseline_requirement_denial(&truth, requirement).is_none());
        assert_eq!(admitted, 1);
        truth
            .deregister_surface(session.effect_port(), request)
            .unwrap();
        assert!(baseline_requirement_denial(&truth, requirement).is_some());
    }
}
