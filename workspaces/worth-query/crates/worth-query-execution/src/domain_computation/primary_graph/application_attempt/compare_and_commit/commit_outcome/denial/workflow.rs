use {super::WorthQueryApplicationCommitDenial, crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage, crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialKind};
use {
    crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial,
    crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind,
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

    pub fn workflow_settlement_denied(denial: &WorthQueryApplicationAttemptDenial) -> Self {
        let stage = match denial.kind() {
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
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
