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
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn workflow_settlement_denied(
        denial: &WorthQueryApplicationAttemptDenial,
    ) -> Self {
        let stage = match denial.kind() {
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
            | WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
            | WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded
            | WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable
            | WorthQueryApplicationAttemptDenialKind::WorkflowLineageCapacityUnavailable => {
                WorthQueryApplicationCommitDenialStage::ResourceAdmission
            }
            WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionSuperseded
            | WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionRetired => {
                WorthQueryApplicationCommitDenialStage::DecisionReadSet
            }
            _ => WorthQueryApplicationCommitDenialStage::ProposalBinding,
        };
        Self {
            kind: WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
                kind: denial.kind(),
            },
            stage,
            detail: Some(denial.to_string().into()),
            cause: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_capacity_workflow_settlement_reports_resource_admission() {
        let attempt = WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
            "workflow settlement",
        );
        let denied = WorthQueryApplicationCommitDenial::workflow_settlement_denied(&attempt);
        assert_eq!(
            denied.stage(),
            WorthQueryApplicationCommitDenialStage::ResourceAdmission
        );
        assert_eq!(
            denied.kind(),
            WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
                kind: WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
            }
        );
    }
}
