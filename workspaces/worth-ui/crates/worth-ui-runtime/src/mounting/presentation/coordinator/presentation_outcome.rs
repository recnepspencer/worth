use worth_ui_host_contract::{
    UiHostPresentationCompletionToken, UiHostSurfacePresentationDenial,
    UiHostSurfacePresentationOutcome, UiMountedSurfacePresentationCompletion,
};

use super::super::outcome::{
    UiMountedSurfacePresentationReceipt, UiMountedSurfacePresentationRejection,
};
use super::super::presented_surface::{UiIssuedSurfacePresentation, UiPresentedSurfaceWitness};
use super::super::terminal::UiIndeterminatePresentationEvidence;
use super::presentation_attempt::{UiMountedPresentationProgress, UiMountedPresentationStart};

pub(super) fn record(
    start: &UiMountedPresentationStart<'_, '_>,
    surface: &crate::mounting::UiMountedSurfaceReceipt,
    expected_effects: &[worth_ui_host_contract::UiMountedEffectFamily],
    progress: &mut UiMountedPresentationProgress,
    outcome: UiHostSurfacePresentationOutcome,
    text_candidate: Option<super::UiMountedTextPinCandidate>,
    semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    text_reuse: Option<
        crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate,
    >,
    text: &mut crate::native_platform::text_presentation::UiNativeMountedTextCoordinator,
    presentation_async: Option<
        &mut crate::native_platform::text_presentation::UiPresentationAsyncRuntime,
    >,
) -> Result<(), UiIndeterminatePresentationEvidence> {
    PresentationOutcomeSettlement {
        start,
        surface,
        expected_effects,
        progress,
        presentation_async,
        text,
    }
    .record(
        outcome,
        PresentationOutcomeEvidence {
            text_candidate,
            semantic_receipts,
            text_reuse,
        },
    )
}

struct PresentationOutcomeEvidence {
    text_candidate: Option<super::UiMountedTextPinCandidate>,
    semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
    text_reuse:
        Option<crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate>,
}

struct PresentationOutcomeSettlement<'a, 'host, 'authority> {
    start: &'a UiMountedPresentationStart<'host, 'authority>,
    surface: &'a crate::mounting::UiMountedSurfaceReceipt,
    expected_effects: &'a [worth_ui_host_contract::UiMountedEffectFamily],
    progress: &'a mut UiMountedPresentationProgress,
    presentation_async:
        Option<&'a mut crate::native_platform::text_presentation::UiPresentationAsyncRuntime>,
    text: &'a mut crate::native_platform::text_presentation::UiNativeMountedTextCoordinator,
}

impl PresentationOutcomeSettlement<'_, '_, '_> {
    fn record(
        &mut self,
        outcome: UiHostSurfacePresentationOutcome,
        evidence: PresentationOutcomeEvidence,
    ) -> Result<(), UiIndeterminatePresentationEvidence> {
        let PresentationOutcomeEvidence {
            text_candidate,
            semantic_receipts,
            text_reuse,
        } = evidence;
        match outcome {
            UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial) => {
                self.record_rejected_before_effects(denial, semantic_receipts, text_reuse)
            }
            UiHostSurfacePresentationOutcome::InFlight(token) => {
                self.record_in_flight(token, text_candidate, semantic_receipts, text_reuse);
                Ok(())
            }
            UiHostSurfacePresentationOutcome::PresentationIndeterminate => {
                self.record_effects_indeterminate(semantic_receipts, text_reuse)
            }
            UiHostSurfacePresentationOutcome::Presented(completion) => {
                self.record_presented(completion, semantic_receipts, text_reuse)
            }
        }
    }

    fn record_rejected_before_effects(
        &mut self,
        denial: UiHostSurfacePresentationDenial,
        semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
        _text_reuse: Option<
            crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate,
        >,
    ) -> Result<(), UiIndeterminatePresentationEvidence> {
        if let Some(owner) = self.presentation_async.as_deref_mut() {
            if semantic_receipts
                .iter()
                .any(|receipt| owner.reject_recovery_before_effects(receipt).is_err())
            {
                let uncertainty =
                    super::surface_uncertainty::PresentationSurfaceUncertainty::semantic(
                        self.surface.requirement().binding(),
                        None,
                        semantic_receipts.into_vec(),
                    );
                return Err(self.terminalize(uncertainty));
            }
        }
        self.progress
            .rejected
            .push(UiMountedSurfacePresentationRejection::new(
                self.surface.requirement().binding(),
                denial,
            ));
        Ok(())
    }

    fn record_in_flight(
        &mut self,
        token: UiHostPresentationCompletionToken,
        text_candidate: Option<super::UiMountedTextPinCandidate>,
        semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
        text_reuse: Option<
            crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate,
        >,
    ) {
        self.progress
            .pending
            .push(super::super::state::UiPendingMountedSurface {
                binding: self.surface.requirement().binding(),
                token,
                expected_effects: self.expected_effects.to_vec().into_boxed_slice(),
                text_candidate,
                semantic_receipts,
                text_reuse,
            });
    }

    fn record_effects_indeterminate(
        &mut self,
        semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
        _text_reuse: Option<
            crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate,
        >,
    ) -> Result<(), UiIndeterminatePresentationEvidence> {
        let uncertainty =
            super::surface_uncertainty::PresentationSurfaceUncertainty::effects_indeterminate(
                self.surface.requirement().binding(),
                None,
                semantic_receipts,
                self.presentation_async.as_deref_mut(),
                true,
            );
        Err(self.terminalize(uncertainty))
    }

    fn record_presented(
        &mut self,
        completion: UiMountedSurfacePresentationCompletion,
        semantic_receipts: Box<[worth_ui_query_binding::WorthUiPresentationRecoveryReceipt]>,
        text_reuse: Option<
            crate::native_platform::text_presentation::UiMountedTextForegroundReuseUpdate,
        >,
    ) -> Result<(), UiIndeterminatePresentationEvidence> {
        let witness = match UiPresentedSurfaceWitness::admit(
            completion,
            self.start.host.authority(),
            UiIssuedSurfacePresentation {
                attempt: self.start.attempt,
                requirement: self.surface.requirement(),
                frame: self.start.frame.canonical_core().frame(),
                expected_effects: self.expected_effects,
            },
        ) {
            Ok(witness) => witness,
            Err(refused) => {
                let uncertainty =
                    super::surface_uncertainty::PresentationSurfaceUncertainty::effects_indeterminate(
                        self.surface.requirement().binding(),
                        Some(refused.cost()),
                        semantic_receipts,
                        self.presentation_async.as_deref_mut(),
                        false,
                    );
                return Err(self.terminalize(uncertainty));
            }
        };
        let posture = super::presented_semantic_settlement::settle(
            self.surface.requirement().binding(),
            witness.cost(),
            semantic_receipts,
            self.presentation_async.as_deref_mut(),
        )
        .map_err(|uncertainty| self.terminalize(uncertainty))?;
        self.progress.superseded |=
            posture == super::presented_semantic_settlement::PresentedSemanticPosture::Superseded;
        self.progress
            .completed
            .push(UiMountedSurfacePresentationReceipt::new(witness));
        if posture == super::presented_semantic_settlement::PresentedSemanticPosture::Current {
            if let Some(update) = text_reuse {
                self.text.commit_foreground_reuse(update);
            }
        }
        Ok(())
    }

    fn terminalize(
        &mut self,
        uncertainty: super::surface_uncertainty::PresentationSurfaceUncertainty,
    ) -> UiIndeterminatePresentationEvidence {
        super::surface_uncertainty::terminalize(
            self.progress,
            self.start.host,
            self.presentation_async.as_deref_mut(),
            uncertainty,
        )
    }
}
