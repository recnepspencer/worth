use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryProgramActivationUnresolved,
};

impl WorthQueryApplicationCommitDenial {
    /// The attempt reached the direct commit entry for a program-owned operation.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_lane_required<
        Operation,
    >() -> Self {
        Self::program_operation_required(format!(
            "operation {} requires the program commit lane",
            std::any::type_name::<Operation>(),
        ))
    }

    /// A producer presented an operation outside the installed conditional inventory.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn conditional_operation_not_installed<
        Operation,
    >() -> Self {
        Self::program_operation_required(format!(
            "operation {} is not installed as a conditional operation",
            std::any::type_name::<Operation>(),
        ))
    }

    /// The occurrence was resolved, but its actual program does not declare this route.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn operation_not_declared_by_active_program<
        Operation,
    >(
        identity: &ApplicationProgramIdentity,
        revision: &ApplicationProgramRevision,
    ) -> Self {
        Self::program_operation_required(format!(
            "operation {} is not declared by active program {} revision {}",
            std::any::type_name::<Operation>(),
            identity.as_str(),
            revision,
        ))
    }

    fn program_operation_required(detail: String) -> Self {
        let mut denial = Self::application_program_required();
        denial.detail = Some(detail.into());
        denial
    }

    /// Refuses activation this host cannot attribute to an admitted program.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_activation_unresolved(
        unresolved: WorthQueryProgramActivationUnresolved,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramActivationUnresolved,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(unresolved.detail())),
            cause: None,
        }
    }

    /// Refuses a rostered program different from the occurrence's activation.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_not_active_on_occurrence(
        presented: &ApplicationProgramIdentity,
        active_identity: &ApplicationProgramIdentity,
        active_revision: &ApplicationProgramRevision,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence {
                active: *active_revision,
            },
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(format!(
                "presented program {} is not active on this occurrence: {} is",
                presented.as_str(),
                active_identity.as_str()
            ))),
            cause: None,
        }
    }

    /// Refuses a rostered revision whose support this host is retiring or has
    /// retired.
    pub(in crate::domain_computation::primary_graph) fn program_support_retired(
        revision: &ApplicationProgramRevision,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramSupportRetired,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(format!(
                "program revision {revision} is no longer active on this host"
            ))),
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn program_revision_not_active_on_occurrence(
        presented_identity: &ApplicationProgramIdentity,
        presented_revision: &ApplicationProgramRevision,
        active_identity: &ApplicationProgramIdentity,
        active_revision: &ApplicationProgramRevision,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProgramNotActiveOnOccurrence {
                active: *active_revision,
            },
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: Some(std::sync::Arc::from(format!(
                "presented program {} revision {} is not active on this occurrence: {} revision {} is",
                presented_identity.as_str(),
                presented_revision,
                active_identity.as_str(),
                active_revision,
            ))),
            cause: None,
        }
    }
}
