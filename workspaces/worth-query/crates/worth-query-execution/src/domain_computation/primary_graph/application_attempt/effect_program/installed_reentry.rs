//! Installed admission authority at typed effect-program re-entry.
use super::*;
use crate::domain_computation::authorization::{
    BrandTag, WorthQueryAdmittedOperation, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

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

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[inline]
    pub(in crate::domain_computation::primary_graph) fn validate_commit_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        let (core, tag) = program.read_set.admission.reentry_parts();
        self.runtime.validate_commit_program(
            phase,
            core,
            tag,
            &self.installed_schema.binding_identity(),
        )
    }
}

impl WorthQueryApplicationRuntimeCore {
    fn validate_commit_program(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        core: &WorthQueryAdmittedOperation,
        tag: &BrandTag,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        if let Some(phase) = phase {
            phase
                .execution_request_for(&self.product_runtime)
                .map_err(|cause| {
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                        cause,
                    )
                    .into_commit_outcome()
                })?;
        }
        self.validate_commit_installation(core, tag, binding)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        core.validate_current_authority()
            .map_err(current_authority_outcome)
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
