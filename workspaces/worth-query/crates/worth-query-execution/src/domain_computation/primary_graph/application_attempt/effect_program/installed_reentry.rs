//! Installed admission authority at typed effect-program re-entry.
use super::*;
use crate::domain_computation::authorization::{
    BrandTag, WorthQueryAdmittedOperation, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationRuntimeCore;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use {
    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial,
    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
    crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[inline]
    pub(in crate::domain_computation::primary_graph) fn validate_current_authority(
        &self,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.read_set.admission.validate_current_authority()
    }
    #[inline]
    pub(in crate::domain_computation::primary_graph) fn validate_installed_entry(
        &self,
        runtime: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.read_set.admission.validate_installed_entry(
            runtime.authority_identity(),
            binding,
            runtime.installed_operations(),
        )
    }
}

/// The commit program's phase, installation and authority are still current for this runtime.
pub(in crate::domain_computation::primary_graph) fn validate_installed_commit_program<Schema>(
    host: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    phase: Option<&WorthQueryAdvancementPhase<'_>>,
    core: &WorthQueryAdmittedOperation,
    tag: &BrandTag,
    binding: &ApplicationSchemaBindingIdentity,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    validate_commit_phase(host, phase)?;
    core.validate_installed_entry_with_tag(
        tag,
        host.authority_identity(),
        binding,
        host.installed_operations(),
    )
    .map_err(installation_denial)
    .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
    core.validate_current_authority()
        .map_err(current_authority_outcome)
}

/// The admitted operation's authority has not lapsed inside this runtime's advancement.
pub(in crate::domain_computation::primary_graph) fn validate_commit_lapse<Schema>(
    host: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    core: &WorthQueryAdmittedOperation,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    validate_commit_phase(host, Some(phase))?;
    core.validate_current_authority()
        .map_err(current_authority_outcome)
}

fn validate_commit_phase<Schema>(
    host: &dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    phase: Option<&WorthQueryAdvancementPhase<'_>>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    if let Some(phase) = phase {
        phase
            .execution_request_for(host.product_runtime())
            .map_err(|cause| {
                crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from(
                    cause,
                )
                .into_commit_outcome()
            })?;
    }
    Ok(())
}

fn installation_denial(
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryApplicationCommitDenial {
    WorthQueryApplicationCommitDenial::request_authority_denied(
        WorthQueryApplicationCommitDenialStage::DecisionReadSet,
        &denial,
    )
}
fn current_authority_outcome(
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryApplicationCommitOutcome {
    super::super::provider_execution::commit_outcome_from_authorization_denial(
        denial,
        WorthQueryApplicationCommitDenialStage::DecisionReadSet,
    )
}

mod court_doors_for_test;
