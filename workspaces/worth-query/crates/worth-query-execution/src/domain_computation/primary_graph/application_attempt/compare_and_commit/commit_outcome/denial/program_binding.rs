use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryProgramActivationUnresolved,
};

impl WorthQueryApplicationCommitDenial {
    /// Refuses activation this host cannot attribute to an admitted program.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_activation_unresolved(
        unresolved: WorthQueryProgramActivationUnresolved,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramActivationUnresolved,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(unresolved.detail())),
            custom_invariant: None,
        }
    }

    /// Refuses a rostered program different from the occurrence's activation.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_not_active_on_occurrence(
        presented: &ApplicationProgramIdentity,
        active: &ApplicationProgramIdentity,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(format!(
                "presented program {} is not active on this occurrence: {} is",
                presented.as_str(),
                active.as_str()
            ))),
            custom_invariant: None,
        }
    }

    /// Refuses a rostered revision whose support this host is retiring or has
    /// retired.
    pub(in crate::domain_computation::primary_graph) fn program_support_not_active(
        revision: &ApplicationProgramRevision,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(format!(
                "program revision {revision} is no longer active on this host"
            ))),
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_revision_not_active_on_occurrence(
        presented_identity: &ApplicationProgramIdentity,
        presented_revision: &ApplicationProgramRevision,
        active_identity: &ApplicationProgramIdentity,
        active_revision: &ApplicationProgramRevision,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(format!(
                "presented program {} revision {} is not active on this occurrence: {} revision {} is",
                presented_identity.as_str(),
                presented_revision,
                active_identity.as_str(),
                active_revision,
            ))),
            custom_invariant: None,
        }
    }
}
