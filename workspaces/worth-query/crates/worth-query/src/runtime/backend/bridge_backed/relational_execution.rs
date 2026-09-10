use crate::memory_workspace::{WorthQueryMutationReceipt, WorthQueryWorkspaceError};
use crate::runtime::{
    WorthQueryBackendAdmissibleMutation, WorthQueryBackendMergeAuthority,
    WorthQueryIntentDeclaration, WorthQueryIntentExecution, WorthQueryRuntimeError,
};

use super::WorthQueryBridgeBackedRuntimeBackend;

impl WorthQueryBridgeBackedRuntimeBackend {
    pub(super) fn surrender_unpublished_graph_runtime(
        &mut self,
    ) -> Result<crate::runtime::WorthQueryUnpublishedPrimaryGraphRuntime, WorthQueryWorkspaceError>
    {
        if !matches!(
            self.relational_runtime.as_ref(),
            Some(super::super::relational_owner::WorthQueryBackendRelationalOwner::Unpublished(_))
        ) {
            return Err(WorthQueryWorkspaceError::new(
                "bridge-backed runtime has no unpublished Relational runtime to surrender",
            ));
        }
        let Some(super::super::relational_owner::WorthQueryBackendRelationalOwner::Unpublished(
            runtime,
        )) = self.relational_runtime.take()
        else {
            unreachable!("the unpublished ownership stage was checked before transfer")
        };
        Ok(crate::runtime::WorthQueryUnpublishedPrimaryGraphRuntime::new(runtime))
    }

    pub(super) fn attach_published_graph_runtime(
        &mut self,
        runtime: crate::runtime::WorthQueryPrimaryGraphBackendHandle,
    ) -> Result<(), WorthQueryWorkspaceError> {
        if self.relational_runtime.is_some() {
            return Err(WorthQueryWorkspaceError::new(
                "bridge-backed runtime must surrender its Relational runtime before primary-graph attachment",
            ));
        }
        self.relational_runtime = Some(
            super::super::relational_owner::WorthQueryBackendRelationalOwner::PrimaryGraph(runtime),
        );
        Ok(())
    }

    pub(super) fn execute_backend_write(
        &mut self,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        let expected_mutation = mutation.clone();
        let write_execution = match self.relational_runtime.as_mut() {
            Some(owner) => owner
                .execute_mutation(|runtime| {
                    self.write_authority
                        .write(&self.runtime_bridge, Some(runtime), mutation)
                })
                .map_err(primary_graph_refresh_workspace_error)??,
            None => self
                .write_authority
                .write(&self.runtime_bridge, None, mutation)?,
        };
        if let Some(message) =
            write_execution.drift_from_backend_admissible_mutation(&expected_mutation)
        {
            return Err(WorthQueryWorkspaceError::new(message));
        }
        let receipt = write_execution.mutation_receipt().clone();
        let routed = self.signal_sink.route_write_receipt(&receipt)?;
        if let Some(message) = routed.drift_from_mutation_receipt(&receipt) {
            return Err(WorthQueryWorkspaceError::new(message));
        }
        Ok(receipt)
    }

    pub(super) fn execute_backend_write_batch(
        &mut self,
        mutations: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        let expected_mutations = mutations.clone();
        let write_executions = match self.relational_runtime.as_mut() {
            Some(owner) => owner
                .execute_mutation(|runtime| {
                    self.write_authority
                        .write_batch(&self.runtime_bridge, Some(runtime), mutations)
                })
                .map_err(primary_graph_refresh_workspace_error)??,
            None => self
                .write_authority
                .write_batch(&self.runtime_bridge, None, mutations)?,
        };
        for (mutation, execution) in expected_mutations.iter().zip(write_executions.iter()) {
            if let Some(message) = execution.drift_from_backend_admissible_mutation(mutation) {
                return Err(WorthQueryWorkspaceError::new(message));
            }
        }
        let receipts = write_executions
            .iter()
            .map(|execution| execution.mutation_receipt().clone())
            .collect::<Vec<_>>();
        let routed = self.signal_sink.route_write_batch(&receipts)?;
        if routed.len() != receipts.len() {
            return Err(WorthQueryWorkspaceError::new(format!(
                "signal invalidation routing batch width drifted from write batch: expected `{}`, found `{}`",
                receipts.len(),
                routed.len()
            )));
        }
        for (receipt, routed_receipt) in receipts.iter().zip(routed.iter()) {
            if let Some(message) = routed_receipt.drift_from_mutation_receipt(receipt) {
                return Err(WorthQueryWorkspaceError::new(message));
            }
        }
        Ok(receipts)
    }

    pub(super) fn execute_backend_intent(
        &mut self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        let Some(intent_authority) = self.intent_authority.as_mut() else {
            return Err(WorthQueryRuntimeError::MissingIntentAuthority);
        };
        match self.relational_runtime.as_mut() {
            Some(owner) => owner
                .execute_mutation(|runtime| {
                    intent_authority.execute_intent(
                        &self.runtime_bridge,
                        Some(runtime),
                        declaration,
                    )
                })
                .map_err(primary_graph_refresh_runtime_error)?
                .map_err(WorthQueryRuntimeError::Workspace),
            None => intent_authority
                .execute_intent(&self.runtime_bridge, None, declaration)
                .map_err(WorthQueryRuntimeError::Workspace),
        }
    }

    pub(super) fn capture_backend_merge_authority(
        &self,
        target_branch: &crate::runtime::WorthQueryAdmittedBranchName,
        source_branch: &crate::runtime::WorthQueryAdmittedBranchName,
    ) -> Result<WorthQueryBackendMergeAuthority, WorthQueryWorkspaceError> {
        let owner = self.relational_runtime.as_ref().ok_or_else(|| {
            WorthQueryWorkspaceError::new(
                "bridge-backed runtime has no configured relational merge authority",
            )
        })?;
        owner.with_runtime(|runtime| {
            WorthQueryBackendMergeAuthority::capture(runtime, target_branch, source_branch)
        })
    }

    pub(super) fn validate_backend_merge_authority(
        &self,
        authority: &WorthQueryBackendMergeAuthority,
    ) -> Result<(), WorthQueryWorkspaceError> {
        let owner = self.relational_runtime.as_ref().ok_or_else(|| {
            WorthQueryWorkspaceError::new(
                "bridge-backed runtime has no configured relational merge authority",
            )
        })?;
        owner.with_runtime(|runtime| authority.validate_against(runtime))
    }

    pub(super) fn execute_backend_merge(
        &mut self,
        authority: &WorthQueryBackendMergeAuthority,
        declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    ) -> Result<
        worth_relational::facade::transactions::MergeExecutionOutcome,
        crate::effect_lifecycle::RelationalEffectExecutionFailure,
    > {
        if declaration.merge_request().target_branch() != authority.target_branch()
            || declaration.merge_request().source_branch() != authority.source_branch()
        {
            return Err((
                crate::effect_lifecycle::EffectExecutionDenialKind::AuthorityOverrideRejected,
                "lowered merge request does not match the captured branch authority".to_string(),
            )
                .into());
        }
        let owner = self.relational_runtime.as_mut().ok_or_else(|| {
            (
                crate::effect_lifecycle::EffectExecutionDenialKind::MissingRelationalAuthority,
                "bridge-backed runtime has no configured relational merge authority".to_string(),
            )
        })?;
        owner
            .execute_mutation(|runtime| {
                crate::effect_lifecycle::execute_lowered_merge(runtime, declaration)
            })
            .map_err(primary_graph_refresh_merge_error)?
    }

    pub(super) fn release_backend_merge_snapshot(
        &mut self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.relational_runtime
            .as_mut()
            .expect("a backend that executed a merge retains its relational runtime")
            .execute_mutation(|runtime| {
                runtime
                    .snapshots()
                    .release_snapshot(snapshot)
                    .expect("ordinary merge closes its exact relational snapshot once");
                Ok::<_, std::convert::Infallible>(())
            })
            .expect("snapshot closeout does not invalidate primary-graph indexes")
            .expect("snapshot closeout is infallible");
    }
}

fn primary_graph_refresh_workspace_error(
    denial: worth_query_execution::facade::integration::WorthQueryPrimaryGraphIndexRefreshDenial,
) -> WorthQueryWorkspaceError {
    WorthQueryWorkspaceError::new(format!(
        "authoritative graph mutation advanced but primary identity indexes could not be refreshed; retry is indeterminate: {denial}"
    ))
}

fn primary_graph_refresh_runtime_error(
    denial: worth_query_execution::facade::integration::WorthQueryPrimaryGraphIndexRefreshDenial,
) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::InvariantRegistration {
        stage: "primary_graph_index_refresh_after_commit",
        message: format!(
            "authoritative graph mutation advanced but primary identity indexes could not be refreshed; retry is indeterminate: {denial}"
        ),
    }
}

fn primary_graph_refresh_merge_error(
    denial: worth_query_execution::facade::integration::WorthQueryPrimaryGraphIndexRefreshDenial,
) -> crate::effect_lifecycle::RelationalEffectExecutionFailure {
    (
        crate::effect_lifecycle::EffectExecutionDenialKind::MergeExecutionFailed,
        format!(
            "merge committed but primary identity indexes could not be refreshed; retry is indeterminate: {denial}"
        ),
    )
        .into()
}
