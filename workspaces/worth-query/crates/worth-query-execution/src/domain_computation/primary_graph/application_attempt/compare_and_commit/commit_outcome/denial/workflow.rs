use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation) const fn workflow_authority_required() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::WorkflowAuthorityRequired,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn workflow_settlement_denied(
        denial: &WorthQueryApplicationAttemptDenial,
    ) -> Self {
        let stage = match denial.kind() {
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
            | WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
            | WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded => {
                WorthQueryApplicationCommitDenialStage::ResourceAdmission
            }
            _ => WorthQueryApplicationCommitDenialStage::ProposalBinding,
        };
        Self {
            kind: WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
                kind: denial.kind(),
            },
            stage,
            detail: Some(denial.to_string().into()),
            custom_invariant: None,
        }
    }
}
