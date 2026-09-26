use crate::runtime::{
    WorthUiCertifiedFrameExecutionReceipt, WorthUiSteadyFrameFoundationalBridge,
    WorthUiSteadyFrameFoundationalEvidence,
};
use worth_proof::TransitionOutcome;

use super::denial::WorthUiLaneFrameCostCertificationDenialReason;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiLaneFrameCostFoundationalReadiness {
    foundational_evidence: WorthUiSteadyFrameFoundationalEvidence,
    certified_foundational_receipt_count: usize,
    readiness_required: bool,
    readiness_satisfied: bool,
}

impl WorthUiLaneFrameCostFoundationalReadiness {
    pub(crate) fn certify(
        certified: &WorthUiCertifiedFrameExecutionReceipt,
        readiness_required: bool,
    ) -> Result<Self, WorthUiLaneFrameCostCertificationDenialReason> {
        if !readiness_required {
            return Err(
                WorthUiLaneFrameCostCertificationDenialReason::FoundationalReadinessNotRequested,
            );
        }
        let foundational_evidence = WorthUiSteadyFrameFoundationalBridge::lower_counter_receipts(
            certified,
        )
        .map_err(|denial| {
            WorthUiLaneFrameCostCertificationDenialReason::UncertifiedFrameReceipt(denial.reason())
        })?;
        if foundational_evidence.receipt_count() == 0 {
            return Err(
                WorthUiLaneFrameCostCertificationDenialReason::FoundationalReadinessWithoutWorthUiEvidence,
            );
        }
        let certified_foundational_receipt_count =
            certify_foundational_counter_backed_receipts(&foundational_evidence)?;
        let readiness = worth_foundational::performance_api::stronger_lane::readiness::
            certify_foundational_performance_milestone8_production_test_readiness();
        let readiness_satisfied = worth_foundational::performance_api::stronger_lane::readiness::
            require_foundational_performance_milestone8_production_test_readiness(&readiness)
            .passes_readiness_checklist();
        if !readiness_satisfied {
            return Err(WorthUiLaneFrameCostCertificationDenialReason::FoundationalReadinessDenied);
        }
        Ok(Self {
            foundational_evidence,
            certified_foundational_receipt_count,
            readiness_required,
            readiness_satisfied,
        })
    }

    pub fn foundational_evidence(&self) -> &WorthUiSteadyFrameFoundationalEvidence {
        &self.foundational_evidence
    }

    pub fn certified_foundational_receipt_count(&self) -> usize {
        self.certified_foundational_receipt_count
    }

    pub fn is_required_and_satisfied(&self) -> bool {
        self.readiness_required && self.readiness_satisfied
    }
}

fn certify_foundational_counter_backed_receipts(
    evidence: &WorthUiSteadyFrameFoundationalEvidence,
) -> Result<usize, WorthUiLaneFrameCostCertificationDenialReason> {
    let mut certified_count = 0;
    for receipt_evidence in evidence.evidence() {
        let authority = worth_foundational::performance_api::stronger_lane::certified::
            foundational_performance_certified_attachment_authority();
        match worth_foundational::performance_api::stronger_lane::certified::
            certify_hot_path_counter_backed_performance_receipt(
                receipt_evidence.counter_backed_receipt().clone(),
                authority,
            ) {
            TransitionOutcome::Success(_) => certified_count += 1,
            TransitionOutcome::Denied(_)
            | TransitionOutcome::Deferred(_)
            | TransitionOutcome::Stale(_)
            | TransitionOutcome::RebindRequired(_)
            | TransitionOutcome::Failed(_) => {
                return Err(
                    WorthUiLaneFrameCostCertificationDenialReason::FoundationalCertificationDenied,
                )
            }
        }
    }
    Ok(certified_count)
}
