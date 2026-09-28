use worth_query_host::facade::admission::authentication_event::WorthQueryAuthenticationEventDenial;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationRequestMutationDenial,
    WorthQueryWorkflowAdvancePreparationDenial, WorthQueryWorkflowAssessmentAcceptanceDenial,
    WorthQueryWorkflowAssessmentDemandPreparationDenial,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    WorthQueryWorkflowInstancePreparationDenial, WorthQueryWorkflowOperationBindingDenial,
    WorthQueryWorkflowOperationOwnerAcceptanceDenial,
    WorthQueryWorkflowOperationRecoveryPreparationDenial,
    WorthQueryWorkflowProposalPreparationDenial,
};
use worth_query_host::facade::domain::WorthQueryApplicationWorkflowInstallationDenial;

use crate::ApprovedBusinessPaymentDefinitionDenial;

#[derive(Debug)]
pub enum BankApprovedPaymentWorkflowError {
    Authentication(WorthQueryAuthenticationEventDenial),
    Definition(ApprovedBusinessPaymentDefinitionDenial),
    DefinitionBinding(WorthQueryApplicationWorkflowInstallationDenial),
    DefinitionPublication(WorthQueryWorkflowDefinitionPublicationPreparationDenial),
    InstanceStart(WorthQueryWorkflowInstancePreparationDenial),
    InstanceCancellation(WorthQueryWorkflowInstancePreparationDenial),
    InstanceMigration(WorthQueryWorkflowInstancePreparationDenial),
    Proposal(WorthQueryWorkflowProposalPreparationDenial),
    Advance(WorthQueryWorkflowAdvancePreparationDenial),
    AssessmentPreparation(WorthQueryWorkflowAssessmentDemandPreparationDenial),
    AssessmentDemand(WorthQueryApplicationOutputDemandDenial),
    AssessmentAcceptance(WorthQueryWorkflowAssessmentAcceptanceDenial),
    OperationBinding(WorthQueryWorkflowOperationBindingDenial),
    OperationMutation(WorthQueryApplicationRequestMutationDenial),
    OperationOwnerAcceptance(WorthQueryWorkflowOperationOwnerAcceptanceDenial),
    OperationRecoveryPreparation(WorthQueryWorkflowOperationRecoveryPreparationDenial),
}

impl BankApprovedPaymentWorkflowError {
    pub const fn authentication_denial(&self) -> Option<WorthQueryAuthenticationEventDenial> {
        match self {
            Self::Authentication(denial) => Some(*denial),
            _ => None,
        }
    }
}

impl std::fmt::Display for BankApprovedPaymentWorkflowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authentication(denial) => denial.fmt(formatter),
            Self::Definition(denial) => denial.fmt(formatter),
            Self::OperationBinding(denial) => denial.fmt(formatter),
            Self::DefinitionBinding(denial) => denial.fmt(formatter),
            Self::DefinitionPublication(denial) => denial.fmt(formatter),
            Self::InstanceStart(denial) => denial.fmt(formatter),
            Self::InstanceCancellation(denial) => denial.fmt(formatter),
            Self::InstanceMigration(denial) => denial.fmt(formatter),
            Self::Proposal(denial) => denial.fmt(formatter),
            Self::Advance(denial) => denial.fmt(formatter),
            Self::AssessmentPreparation(denial) => denial.fmt(formatter),
            Self::AssessmentDemand(denial) => denial.fmt(formatter),
            Self::AssessmentAcceptance(denial) => denial.fmt(formatter),
            Self::OperationMutation(denial) => denial.fmt(formatter),
            Self::OperationOwnerAcceptance(denial) => denial.fmt(formatter),
            Self::OperationRecoveryPreparation(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankApprovedPaymentWorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authentication(denial) => Some(denial),
            Self::Definition(denial) => Some(denial),
            Self::OperationBinding(denial) => Some(denial),
            Self::DefinitionBinding(denial) => Some(denial),
            Self::DefinitionPublication(denial) => Some(denial),
            Self::InstanceStart(denial) => Some(denial),
            Self::InstanceCancellation(denial) => Some(denial),
            Self::InstanceMigration(denial) => Some(denial),
            Self::Proposal(denial) => Some(denial),
            Self::Advance(denial) => Some(denial),
            Self::AssessmentPreparation(denial) => Some(denial),
            Self::AssessmentDemand(denial) => Some(denial),
            Self::AssessmentAcceptance(denial) => Some(denial),
            Self::OperationMutation(denial) => Some(denial),
            Self::OperationOwnerAcceptance(denial) => Some(denial),
            Self::OperationRecoveryPreparation(denial) => Some(denial),
        }
    }
}
