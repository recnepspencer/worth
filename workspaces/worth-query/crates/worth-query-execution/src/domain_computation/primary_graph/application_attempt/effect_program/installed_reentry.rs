//! Installed admission authority at typed effect-program re-entry.
use super::*;
use crate::domain_computation::authorization::{
    BrandTag, WorthQueryAdmittedOperation, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use {crate::domain_computation::request_execution::WorthQueryAdvancementPhase, crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial, crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage, crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome};

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
        runtime: &WorthQueryApplicationRuntimeCore,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.read_set.admission.validate_installed_entry(
            runtime.authority_identity(),
            binding,
            &runtime.installed_operations,
        )
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub(in crate::domain_computation::primary_graph) fn validate_commit_program(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        core: &WorthQueryAdmittedOperation,
        tag: &BrandTag,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        self.validate_commit_phase(phase)?;
        self.validate_commit_installation(core, tag, binding)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        core.validate_current_authority()
            .map_err(current_authority_outcome)
    }
    pub(in crate::domain_computation::primary_graph) fn validate_commit_lapse_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        core: &WorthQueryAdmittedOperation,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        self.validate_commit_phase(Some(phase))?;
        core.validate_current_authority()
            .map_err(current_authority_outcome)
    }
    fn validate_commit_phase(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        if let Some(phase) = phase {
            phase
                .execution_request_for(&self.product_runtime)
                .map_err(|cause| {
                    crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from(
                        cause,
                    )
                    .into_commit_outcome()
                })?;
        }
        Ok(())
    }
    fn validate_commit_installation(
        &self,
        core: &WorthQueryAdmittedOperation,
        tag: &BrandTag,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryApplicationCommitDenial> {
        core.validate_installed_entry_with_tag(
            tag,
            self.authority_identity(),
            binding,
            &self.installed_operations,
        )
        .map_err(installation_denial)
    }
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
