#[path = "publication/settlement.rs"]
mod settlement;

use super::WorthUiMountedSessionState;
use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFramePublicationCandidate, UiMountedFramePublicationReceipt,
    UiMountedPresentationInFlight, UiMountedPresentationOutcome,
};
mod host_authority;
#[path = "publication/reconciliation.rs"]
mod reconciliation;
mod transition;
pub(super) use host_authority::mounted_host_authority;

#[derive(Clone, Copy)]
pub(crate) struct UiMountedObservationValidationBasis<'session> {
    retention: &'session crate::mounting::UiMountedFrameRetentionCoordinator,
    presentation: &'session crate::mounting::UiMountedPresentationCoordinator,
}

pub(crate) struct UiMountedPublicationTransition {
    outcome: UiMountedFrameOutcome,
    observation: Option<UiMountedHostObservationTransition>,
    appearance: Option<crate::runtime::appearance::UiAppearanceInspectionAttemptBatch>,
    hit_transition: Option<crate::mounting::UiCommittedPresentedHitTransition>,
}
pub(crate) enum UiMountedHostObservationTransition {
    NeverPresented(worth_ui_host_contract::UiMountedFrameIdentity),
    Rejected(worth_ui_host_contract::UiMountedFrameIdentity),
    Indeterminate {
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        bindings: Box<[worth_ui_host_contract::UiSurfaceBindingGeneration]>,
    },
}

impl WorthUiMountedSessionState {
    pub(crate) fn observation_validation_basis(&self) -> UiMountedObservationValidationBasis<'_> {
        UiMountedObservationValidationBasis {
            retention: &self.retention,
            presentation: &self.presentation,
        }
    }

    pub(crate) fn present_prepared_frame(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        frame: crate::mounting::UiPreparedMountedFrame,
        appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedPublicationTransition {
        self.present_prepared_frame_with_overlays(
            host,
            frame,
            appearance_inspection,
            deadline,
            now,
            |_, _| Ok(Vec::new()),
        )
    }

    pub(crate) fn present_prepared_frame_with_overlays(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        frame: crate::mounting::UiPreparedMountedFrame,
        appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        ) -> Result<
            Vec<crate::mounting::UiMountedAppearanceSurfaceOverlayInput>,
            (),
        >,
    ) -> UiMountedPublicationTransition {
        let capability_report = host.capability_report().clone();
        let admitted = match self.identity.admit_prepared_frame_authority(frame) {
            Ok(admitted) => admitted,
            Err(rejection) => {
                let frame = rejection.frame().canonical_core().frame();
                return UiMountedPublicationTransition::with_observation(
                    UiMountedFrameOutcome::AdmissionDenied(rejection),
                    UiMountedHostObservationTransition::NeverPresented(frame),
                );
            }
        };
        let retained = match self.retention.prepare_publication(admitted) {
            Ok(retained) => retained,
            Err(rejection) => {
                let frame = rejection.frame().canonical_core().frame();
                return UiMountedPublicationTransition::with_observation(
                    UiMountedFrameOutcome::RetentionDenied(rejection),
                    UiMountedHostObservationTransition::NeverPresented(frame),
                );
            }
        };
        self.present_retained_frame(
            host,
            retained,
            capability_report,
            self.identity.view().current_frame(),
            appearance_inspection,
            deadline,
            now,
            prepare_overlays,
        )
    }

    pub(crate) fn present_prepared_superseding_frame(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        frame: crate::mounting::UiPreparedMountedFrame,
        predecessor: crate::mounting::UiMountedSupersedingPresentationBasis,
        appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> UiMountedPublicationTransition {
        self.present_prepared_superseding_frame_with_overlays(
            host,
            frame,
            predecessor,
            appearance_inspection,
            deadline,
            now,
            |_, _| Ok(Vec::new()),
        )
    }

    pub(crate) fn present_prepared_superseding_frame_with_overlays(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        frame: crate::mounting::UiPreparedMountedFrame,
        predecessor: crate::mounting::UiMountedSupersedingPresentationBasis,
        appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        ) -> Result<
            Vec<crate::mounting::UiMountedAppearanceSurfaceOverlayInput>,
            (),
        >,
    ) -> UiMountedPublicationTransition {
        if !self
            .presentation
            .admits_superseding_predecessor(predecessor)
        {
            let identity = frame.canonical_core().frame();
            let rejection = crate::mounting::UiMountedPresentationAdmissionRejection::new(
                frame,
                crate::mounting::UiMountedPresentationAdmissionDenial::SupersedingPredecessorUnavailable,
            );
            return UiMountedPublicationTransition::with_observation(
                UiMountedFrameOutcome::AdmissionDenied(rejection),
                UiMountedHostObservationTransition::NeverPresented(identity),
            );
        }
        let capability_report = host.capability_report().clone();
        let admitted = match self.identity.admit_prepared_frame_authority(frame) {
            Ok(admitted) => admitted,
            Err(rejection) => {
                let frame = rejection.frame().canonical_core().frame();
                return UiMountedPublicationTransition::with_observation(
                    UiMountedFrameOutcome::AdmissionDenied(rejection),
                    UiMountedHostObservationTransition::NeverPresented(frame),
                );
            }
        };
        let retained = match self
            .retention
            .prepare_superseding_publication(admitted, predecessor.retention())
        {
            Ok(retained) => retained,
            Err(rejection) => {
                let frame = rejection.frame().canonical_core().frame();
                return UiMountedPublicationTransition::with_observation(
                    UiMountedFrameOutcome::RetentionDenied(rejection),
                    UiMountedHostObservationTransition::NeverPresented(frame),
                );
            }
        };
        self.present_retained_frame(
            host,
            retained,
            capability_report,
            Some(predecessor.frame()),
            appearance_inspection,
            deadline,
            now,
            prepare_overlays,
        )
    }

    fn present_retained_frame(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        retained: crate::mounting::retention::UiRetentionPreparedMountedFrame,
        capability_report: worth_ui_host_contract::WorthUiHostCapabilityReport,
        publication_predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
        _appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
        ) -> Result<
            Vec<crate::mounting::UiMountedAppearanceSurfaceOverlayInput>,
            (),
        >,
    ) -> UiMountedPublicationTransition {
        let mut admission =
            match self
                .presentation
                .admit_current(retained, &capability_report, deadline, now)
            {
                Ok(admission) => admission,
                Err(rejection) => {
                    let frame = rejection.frame().canonical_core().frame();
                    return UiMountedPublicationTransition::with_observation(
                        UiMountedFrameOutcome::AdmissionDenied(rejection),
                        UiMountedHostObservationTransition::NeverPresented(frame),
                    );
                }
            };
        let surfaces = admission
            .frame()
            .surfaces()
            .iter()
            .map(|surface| surface.requirement().semantic_surface())
            .collect::<Vec<_>>();
        let appearance_batch = match prepare_overlays(admission.attempt(), &surfaces) {
            Ok(overlays) => admission
                .lower_appearance_with_overlays(capability_report.appearance_profile(), &overlays),
            Err(()) => admission.deny_appearance_output(),
        };
        if !admission.appearance_output_available() {
            let frame = admission.frame().canonical_core().frame();
            let rejection = admission.reject_appearance_output();
            return UiMountedPublicationTransition::with_observation_and_appearance(
                UiMountedFrameOutcome::AdmissionDenied(rejection),
                UiMountedHostObservationTransition::NeverPresented(frame),
                appearance_batch,
            );
        }
        let reservation =
            UiMountedFramePublicationCandidate::reserve(&admission, publication_predecessor);
        let attempt = admission.attempt();
        self.presentation
            .retain_appearance_attempt(attempt, appearance_batch);
        let replaced = self.publication_reservations.insert(attempt, reservation);
        assert!(
            replaced.is_none(),
            "runtime-minted presentation attempts must be unique"
        );
        let outcome = self.presentation.present(
            admission.into_attempt(),
            host.effect_port(),
            mounted_host_authority(host, &capability_report),
            now,
        );
        self.finish_presentation(outcome)
    }

    pub(crate) fn current_publication(&self) -> Option<&UiMountedFramePublicationReceipt> {
        self.identity.publication_receipt()
    }

    pub(crate) fn reconcile_presentation(
        &mut self,
        reconciliation: crate::mounting::UiHostPresentationReconciliation,
    ) -> bool {
        self.presentation
            .reconcile(reconciliation, self.identity.view().current_frame())
    }

    fn remove_publication_reservation(
        &mut self,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) {
        self.publication_reservations
            .remove(&attempt)
            .expect("every admitted attempt has a publication reservation");
    }
}

impl UiMountedObservationValidationBasis<'_> {
    pub(crate) fn retention(&self) -> &crate::mounting::UiMountedFrameRetentionCoordinator {
        self.retention
    }

    pub(crate) fn binding_requires_reconciliation(
        self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> bool {
        self.presentation.binding_requires_reconciliation(binding)
    }
}

fn presentation_attempt(
    outcome: &UiMountedPresentationOutcome,
) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
    match outcome {
        UiMountedPresentationOutcome::Presented(frame) => frame.receipt().attempt(),
        UiMountedPresentationOutcome::Superseded(frame) => frame.attempt(),
        UiMountedPresentationOutcome::RejectedBeforeEffects(frame) => frame.attempt(),
        UiMountedPresentationOutcome::InFlight(frame) => frame.attempt(),
        UiMountedPresentationOutcome::PresentationIndeterminate(frame) => frame.report().attempt(),
    }
}

pub(super) fn indeterminate_observation(
    frame: &crate::mounting::UiMountedIndeterminateFrame,
) -> UiMountedHostObservationTransition {
    UiMountedHostObservationTransition::Indeterminate {
        frame: frame.frame().canonical_core().frame(),
        bindings: frame
            .report()
            .affected_bindings()
            .to_vec()
            .into_boxed_slice(),
    }
}
