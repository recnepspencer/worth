//! Ordinary mutation retirement under current Native custody.
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::{
    branch::AdmittedRelationalBranchBasis, runtime::RelationalRuntime,
    transactions::data::EntityReference,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod current_retirement;

impl RelationalRuntime {
    /// Reconstruct the selected complete publication behind real ordinary Retire
    /// targets. No matching canonical ownership leaves genuine ordinary deletion
    /// ordinary; any matched ownership requires its complete exact roster.
    pub fn readmit_authored_output_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial(
                "foreign installed author for output retirement",
            ));
        }
        let current = self.current_retirement_custody(execution, installed, basis, roles)?;
        self.refuse_live_owned_retirement(execution, roles, current.as_deref())?;
        let selected =
            crate::history::data::select_authored_retirement(execution, self, basis, roles, false)
                .map_err(custody::cold_stop)?;
        if let Some(envelope) = &selected
            && envelope.origin.source_scope_owned
        {
            let origin = &envelope.origin;
            return Err(TransactionCommitError::publication(
                crate::publication::data::PublicationError::owned_publication(
                    crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                        publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                    }, origin.outputs[0], origin.meaning.source_scope,
                )));
        }
        self.refuse_retired_scope_owners(execution, roles, current.as_deref())?;
        let Some(envelope) = selected else {
            return Ok(None);
        };
        self.readmit_canonical_publication(execution, installed, basis, &envelope.origin)
            .map(Some)
    }

    /// Mixed ordinary candidates retain legacy behavior for independent outputs,
    /// but cannot hide an owned member or its live source beside Create/Preserve roles.
    pub fn refuse_owned_output_retirement(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<(), TransactionCommitError> {
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(origin_denial(
                "foreign installed author for owned retirement",
            ));
        }
        if installed.source_scope_ownership.is_empty() {
            return Ok(());
        }
        let current = self.current_retirement_custody(execution, installed, basis, roles)?;
        self.refuse_live_owned_retirement(execution, roles, current.as_deref())?;
        if let Some(selected) =
            crate::history::data::select_authored_retirement(execution, self, basis, roles, true)
                .map_err(custody::cold_stop)?
        {
            let origin = selected.origin;
            return Err(TransactionCommitError::publication(
                crate::publication::data::PublicationError::owned_publication(
                    crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                        publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                    }, origin.outputs[0], origin.meaning.source_scope,
                )));
        }
        self.refuse_retired_scope_owners(execution, roles, current.as_deref())
    }

    fn refuse_retired_scope_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        roles: &[RequiredOutputRoleTarget],
        current: Option<&crate::publication::ownership::CurrentPublicationRoster>,
    ) -> Result<(), TransactionCommitError> {
        let Some(current) = current else {
            return Ok(());
        };
        let selected = execution
            .run_native_with_checkpoint(|context| {
                context.checkpoint(roles.len() as u64).map_err(|stop| {
                    crate::history::data::CodecRetirementClosureStop::Request(stop.into())
                })?;
                let removed = roles
                    .iter()
                    .map(|role| match role.target {
                        EntityReference::Existing(id) => Ok(id),
                        _ => Err(crate::history::data::CodecRetirementClosureStop::from(
                            "retirement source must be an existing native generation",
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                crate::history::data::select_owned_scope_dependent(
                    crate::history::data::OwnedScopeDependentSelection::RemovedSources(&removed),
                    current,
                    context,
                )
                .map_err(|failure| failure.stop)
            })
            .map_err(custody::execution_stop)?
            .0
            .map_err(custody::cold_stop)?;
        consult_stage(execution, CommitStage::Admission)?;
        if let Some(origin) = selected {
            return Err(TransactionCommitError::publication(crate::publication::data::PublicationError::owned_publication(
                crate::publication::data::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade {
                    publication: origin.commit.commit_id, owner: origin.meaning.source_scope,
                }, origin.outputs[0], origin.meaning.source_scope)));
        }
        Ok(())
    }
}
