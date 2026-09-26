//! Finishing a presentation attempt the host did not fully present: every
//! binding it leaves uncertain is blocked until reconstruction proves it.

use worth_ui_host_contract::UiMountedPresentationAttemptIdentity;

use super::{UiMountedPresentationCoordinator, UiMountedPresentationSettlement};
use crate::mounting::presentation::outcome::{
    UiMountedIndeterminateFrame, UiMountedPresentationOutcome, UiPresentationIndeterminateReport,
};
use crate::mounting::presentation::terminal::{
    aggregate_affected, rejected_outcome, UiIndeterminatePresentationEvidence,
};

impl UiMountedPresentationCoordinator {
    pub(super) fn finish_rejected(
        &mut self,
        settlement: UiMountedPresentationSettlement<'_>,
    ) -> UiMountedPresentationOutcome {
        self.active.borrow_mut().remove(&settlement.attempt);
        for rejection in &settlement.rejected {
            if rejection.denial()
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::ReconstructionRequired
            {
                if let Some(requirement) = settlement
                    .frame
                    .surfaces()
                    .iter()
                    .find(|surface| surface.requirement().binding() == rejection.binding())
                    .map(|surface| surface.requirement())
                {
                    self.host_truth.block_presentation(requirement);
                }
                self.reconstruction_bindings.insert(rejection.binding());
            }
        }
        rejected_outcome(
            settlement.attempt,
            settlement.frame,
            settlement.retention,
            settlement.rejected,
        )
    }

    pub(super) fn finish_partially_presented(
        &mut self,
        settlement: UiMountedPresentationSettlement<'_>,
    ) -> UiMountedPresentationOutcome {
        let affected = aggregate_affected(&settlement.completed, &[], &settlement.rejected);
        self.indeterminate(
            settlement.frame,
            settlement.retention,
            settlement.attempt,
            UiIndeterminatePresentationEvidence::new(affected, settlement.completed),
        )
    }

    pub(super) fn indeterminate(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        retention: crate::mounting::retention::UiMountedRetentionReservation,
        attempt: UiMountedPresentationAttemptIdentity,
        evidence: UiIndeterminatePresentationEvidence,
    ) -> UiMountedPresentationOutcome {
        let (affected, cost, semantic_receipts, recovery_required, physical_recovery_bindings) =
            evidence.into_terminal_parts(frame.cost_report());
        self.retain_semantic_uncertainty(attempt, semantic_receipts);
        if !recovery_required.is_empty() {
            self.unresolved_semantic_receipts
                .entry(attempt)
                .or_default()
                .extend(recovery_required);
        }
        self.active.borrow_mut().remove(&attempt);
        for binding in &affected {
            // Retain accepted commands and Motion for reconstruction while host truth is blocked.
            self.reconstruction_bindings.insert(*binding);
            let requirement = frame
                .surfaces()
                .iter()
                .find(|surface| surface.requirement().binding() == *binding)
                .expect("affected binding belongs to the retained prepared frame")
                .requirement();
            self.host_truth.block_presentation(requirement);
        }
        drop(retention);
        let report =
            UiPresentationIndeterminateReport::new(attempt, affected, physical_recovery_bindings);
        UiMountedPresentationOutcome::PresentationIndeterminate(UiMountedIndeterminateFrame::new(
            frame, report, cost,
        ))
    }
}
