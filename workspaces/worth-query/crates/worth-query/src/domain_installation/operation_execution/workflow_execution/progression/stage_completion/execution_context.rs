use super::*;

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(super) fn stage_effect_workflow_binding(
        &self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    ) -> crate::workflow::WorkflowContextBinding {
        let effect_binding_scope = format!(
            "{}:{}:{}",
            self.bound.binding_identity(),
            self.identity,
            stage.identity()
        );
        crate::workflow::synthetic_runtime_workflow_binding_scoped_for_snapshot_identity(
            self.bound.definition().canonical_identity(),
            &effect_binding_scope,
            snapshot,
        )
    }

    pub(super) fn stage_execution_context<'a>(
        &'a self,
        stage: &'a worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        predecessor_receipts: &'a [&'a WorthQueryWorkflowStageReceipt],
        graph_receipts: &'a [WorthQueryBoundGraphExecutionReceipt],
        resources: &'a super::super::WorthQueryAdmittedExecutionResourcePlan,
        resource_evidence: &'a super::super::WorthQueryExecutionResourceAttemptEvidence,
        effect_workflow_binding: crate::workflow::WorkflowContextBinding,
    ) -> Result<WorthQueryWorkflowStageExecutionContext<'a>, WorthQueryWorkflowAdvanceDenial> {
        let artifact_production_authority = self
            .managed_run()
            .artifacts()
            .production_authority(stage.identity())
            .map_err(|denial| {
                WorthQueryWorkflowAdvanceDenial::new(
                    WorthQueryWorkflowAdvanceDenialKind::ArtifactCarriage(denial),
                    self.counters,
                )
            })?;
        let artifact_access_authority = self
            .managed_run()
            .artifacts()
            .access_authority(stage.identity())
            .map_err(|denial| {
                WorthQueryWorkflowAdvanceDenial::new(
                    WorthQueryWorkflowAdvanceDenialKind::ArtifactCarriage(denial),
                    self.counters,
                )
            })?;
        Ok(WorthQueryWorkflowStageExecutionContext::new(
            WorthQueryWorkflowStageExecutionScope {
                operation_identity: self.bound.definition().canonical_identity(),
                binding_identity: self.bound.binding_identity(),
                run_identity: &self.identity,
                stage,
                predecessor_receipts,
            },
            WorthQueryWorkflowStageExecutionAuthority {
                effect_workflow_binding,
                basis: self.bound.basis().normalized().family(),
                installed_read: self.executor.installed_read.as_ref(),
                operation_graph_reads: self
                    .bound
                    .definition()
                    .semantics()
                    .graph_reads
                    .domain_roles(),
                graph_receipts,
                resources,
                resource_evidence,
                provider_session_identity: self.provider_session_identity(),
                query_authority: self
                    .bound
                    .definition()
                    .semantics()
                    .canonical_query
                    .query()
                    .authority(),
                identity_evolution_basis_identity: self
                    .bound
                    .basis()
                    .capability_digest()
                    .to_owned(),
                artifact_access_authority,
                artifact_production_authority,
            },
        ))
    }
}
