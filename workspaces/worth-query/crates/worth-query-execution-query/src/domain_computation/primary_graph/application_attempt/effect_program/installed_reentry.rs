//! Installed admission authority at typed effect-program re-entry.
use super::*;
use crate::domain_computation::authorization::{
    BrandTag, WorthQueryAdmittedOperation, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
};
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// Opaque entry refusal. Its causes cannot contain a landed receipt or cleanup
/// outcome; only real entry validation and advancement construct this value.
#[derive(Debug)]
pub struct WorthQueryCommitEntryRefusal {
    cause: CommitEntryCause,
}
#[derive(Debug)]
enum CommitEntryCause {
    Installation(WorthQueryApplicationCommitDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    Advancement(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
}
impl WorthQueryCommitEntryRefusal {
    fn installation(cause: WorthQueryApplicationCommitDenial) -> Self {
        Self {
            cause: CommitEntryCause::Installation(cause),
        }
    }
    fn authorization(cause: WorthQueryOperationAuthorizationDenial) -> Self {
        Self {
            cause: CommitEntryCause::Authorization(cause),
        }
    }
    pub fn from_advancement(
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Self {
        Self {
            cause: CommitEntryCause::Advancement(cause),
        }
    }
    pub fn into_outcome(self) -> WorthQueryApplicationCommitOutcome {
        match self.cause {
            CommitEntryCause::Installation(cause) => {
                WorthQueryApplicationCommitOutcome::Denied(cause)
            }
            CommitEntryCause::Authorization(cause) => current_authority_outcome(cause),
            CommitEntryCause::Advancement(cause) => cause.into_commit_outcome(),
        }
    }
}

impl WorthQueryApplicationRuntimeCore {
    pub fn validate_commit_program(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        core: &WorthQueryAdmittedOperation,
        tag: &BrandTag,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryCommitEntryRefusal> {
        core.refuse_stale_home_authority(self.product_runtime.owner.owner_identity())
            .map_err(WorthQueryCommitEntryRefusal::authorization)?;
        self.validate_commit_phase(phase)?;
        self.validate_commit_installation(core, tag, binding)
            .map_err(WorthQueryCommitEntryRefusal::installation)?;
        core.validate_current_authority()
            .map_err(WorthQueryCommitEntryRefusal::authorization)
    }
    pub fn validate_commit_lapse_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        core: &WorthQueryAdmittedOperation,
    ) -> Result<(), WorthQueryCommitEntryRefusal> {
        core.refuse_stale_home_authority(self.product_runtime.owner.owner_identity())
            .map_err(WorthQueryCommitEntryRefusal::authorization)?;
        self.validate_commit_phase(Some(phase))?;
        core.validate_current_authority()
            .map_err(WorthQueryCommitEntryRefusal::authorization)
    }
    fn validate_commit_phase(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
    ) -> Result<(), WorthQueryCommitEntryRefusal> {
        if let Some(phase) = phase {
            phase
                .execution_request_for(&self.product_runtime)
                .map_err(|cause| {
                    WorthQueryCommitEntryRefusal::from_advancement(
                        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                            cause,
                        ),
                    )
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

pub trait EffectProgramInstalledEntryExt {
    fn validate_installed_entry(
        &self,
        runtime: &WorthQueryApplicationRuntimeCore,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>;
}

impl<Schema, Operation, Input, Scope> EffectProgramInstalledEntryExt
    for WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[inline]
    fn validate_installed_entry(
        &self,
        runtime: &WorthQueryApplicationRuntimeCore,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.read_set.admission.validate_installed_entry(
            runtime.authority_identity(),
            binding,
            runtime.installed_operations(),
        )
    }
}
