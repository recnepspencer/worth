use crate::mounting::{
    UiMountedFrameOutcome, UiMountedFrameReconciliationCandidate, UiMountedPresentationOutcome,
};

use super::{indeterminate_observation, mounted_host_authority, UiMountedPublicationTransition};

impl crate::mounting::session_state::WorthUiMountedSessionState {
    pub(crate) fn present_current_for_reconciliation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
            Option<&crate::runtime::appearance::UiActiveThemeBinding>,
        )
            -> Result<crate::mounting::UiMountedAppearanceDerivedInput, ()>,
    ) -> Result<UiMountedPublicationTransition, crate::mounting::UiMountedIdentityDenial> {
        if !self.pending_direct_scroll.is_empty() {
            return Err(crate::mounting::UiMountedIdentityDenial::ReconciliationBasisMismatch);
        }
        let capability_report = host.capability_report().clone();
        let frame = self.identity.prepare_current_reconciliation_frame(
            replacements,
            host.protocol(),
            &capability_report,
        )?;
        Ok(self.present_reconciliation_frame(
            host,
            frame,
            replacements,
            capability_report,
            appearance_inspection,
            deadline,
            now,
            prepare_overlays,
        ))
    }

    pub(crate) fn present_prepared_for_reconciliation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        frame: crate::mounting::UiPreparedMountedFrame,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
            Option<&crate::runtime::appearance::UiActiveThemeBinding>,
        )
            -> Result<crate::mounting::UiMountedAppearanceDerivedInput, ()>,
    ) -> Result<UiMountedPublicationTransition, crate::mounting::UiMountedIdentityDenial> {
        if replacements.is_empty() || self.identity.publication_receipt().is_none() {
            return Err(crate::mounting::UiMountedIdentityDenial::ReconciliationBasisMismatch);
        }
        let mut frame = frame;
        self.bind_pending_direct_scroll(&mut frame);
        let admitted = match self.identity.admit_prepared_frame_authority(frame) {
            Ok(admitted) => admitted,
            Err(rejection) => {
                return Ok(UiMountedPublicationTransition::new(
                    UiMountedFrameOutcome::AdmissionDenied(rejection),
                ));
            }
        };
        Ok(self.present_reconciliation_frame(
            host,
            admitted,
            replacements,
            host.capability_report().clone(),
            appearance_inspection,
            deadline,
            now,
            prepare_overlays,
        ))
    }

    fn present_reconciliation_frame(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
        frame: crate::mounting::UiAuthorityAdmittedMountedFrame,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        capability_report: worth_ui_host_contract::WorthUiHostCapabilityReport,
        _appearance_inspection: Option<
            &mut crate::runtime::appearance::UiAppearanceInspectionProducer,
        >,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        prepare_overlays: impl FnOnce(
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
            &[worth_ui_host_contract::UiSemanticSurfaceIdentity],
            Option<&crate::runtime::appearance::UiActiveThemeBinding>,
        )
            -> Result<crate::mounting::UiMountedAppearanceDerivedInput, ()>,
    ) -> UiMountedPublicationTransition {
        let current = self
            .identity
            .publication_receipt()
            .cloned()
            .expect("reconciliation retains a published predecessor");
        let retained = match self.retention.prepare_reconciliation(frame) {
            Ok(retained) => retained,
            Err(rejection) => {
                return UiMountedPublicationTransition::new(
                    UiMountedFrameOutcome::RetentionDenied(rejection),
                );
            }
        };
        let admission = match self.presentation.admit_reconciliation(
            retained,
            replacements,
            &capability_report,
            deadline,
            now,
        ) {
            Ok(admission) => admission,
            Err(rejection) => {
                return UiMountedPublicationTransition::new(
                    UiMountedFrameOutcome::AdmissionDenied(rejection),
                );
            }
        };
        let surfaces = admission
            .frame()
            .surfaces()
            .iter()
            .map(|surface| surface.requirement().semantic_surface())
            .collect::<Vec<_>>();
        let appearance = match prepare_overlays(
            admission.attempt(),
            &surfaces,
            admission.frame().prepared_theme_binding(),
        ) {
            Ok(derived) => admission
                .lower_appearance_with_overlays(capability_report.appearance_profile(), &derived),
            Err(()) => admission.deny_appearance_output(),
        };
        let (admission, appearance_batch) = match appearance.admit_appearance_retention() {
            Ok(admission) => admission,
            Err(rejected) => {
                let (rejection, appearance_batch) = *rejected;
                let frame = rejection.frame().canonical_core().frame();
                return UiMountedPublicationTransition::with_observation_and_appearance(
                    UiMountedFrameOutcome::AdmissionDenied(rejection),
                    super::UiMountedHostObservationTransition::NeverPresented(frame),
                    appearance_batch,
                );
            }
        };
        let reservation =
            UiMountedFrameReconciliationCandidate::reserve(&admission, &current, replacements);
        let attempt = admission.attempt();
        self.presentation
            .retain_appearance_attempt(attempt, appearance_batch);
        let replaced = self
            .reconciliation_reservations
            .insert(attempt, reservation);
        assert!(
            replaced.is_none(),
            "runtime-minted reconciliation attempts must be unique"
        );
        let outcome = self.presentation.present(
            admission,
            host.effect_port(),
            mounted_host_authority(host, &capability_report),
            now,
        );
        self.finish_presentation(outcome)
    }

    pub(super) fn finish_reconciliation(
        &mut self,
        outcome: UiMountedPresentationOutcome,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> UiMountedPublicationTransition {
        match outcome {
            UiMountedPresentationOutcome::Presented(presented) => {
                let reservation = self
                    .reconciliation_reservations
                    .remove(&attempt)
                    .expect("every reconciliation presentation retains a reservation");
                let replacements = reservation.replacements().to_vec();
                match reservation.commit_presented(presented, &mut self.identity) {
                    crate::mounting::UiMountedFramePublicationCommit::Current(receipt) => {
                        self.rebind_motion_sampling_after_publication(&receipt);
                        self.retention.refresh_presented_hit_motion(
                            &self.motion_sampling,
                            &self.motion_sampling.retained_targets(),
                        );
                        self.presentation
                            .commit_current_frame_reconciliation(&replacements);
                        UiMountedPublicationTransition::new(UiMountedFrameOutcome::Reconciled(
                            receipt,
                        ))
                    }
                    crate::mounting::UiMountedFramePublicationCommit::Superseded(frame) => {
                        UiMountedPublicationTransition::new(UiMountedFrameOutcome::Superseded(
                            frame,
                        ))
                    }
                }
            }
            UiMountedPresentationOutcome::RejectedBeforeEffects(rejected) => {
                self.remove_reconciliation_reservation(rejected.attempt());
                UiMountedPublicationTransition::new(UiMountedFrameOutcome::RejectedBeforeEffects(
                    rejected,
                ))
            }
            UiMountedPresentationOutcome::InFlight(in_flight) => {
                UiMountedPublicationTransition::new(UiMountedFrameOutcome::InFlight(in_flight))
            }
            UiMountedPresentationOutcome::Superseded(frame) => {
                self.remove_reconciliation_reservation(frame.attempt());
                UiMountedPublicationTransition::new(UiMountedFrameOutcome::Superseded(frame))
            }
            UiMountedPresentationOutcome::PresentationIndeterminate(indeterminate) => {
                self.remove_reconciliation_reservation(indeterminate.report().attempt());
                let observation = indeterminate_observation(&indeterminate);
                UiMountedPublicationTransition::with_observation(
                    UiMountedFrameOutcome::PresentationIndeterminate(indeterminate),
                    observation,
                )
            }
        }
    }

    fn remove_reconciliation_reservation(
        &mut self,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) {
        self.reconciliation_reservations
            .remove(&attempt)
            .expect("every admitted reconciliation has a reservation");
    }
}
