use super::*;

impl WorthQueryRunningWorkflowRun {
    pub fn resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan
    {
        self.affinity.resources()
    }

    pub fn operation_resources(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan
    {
        self.affinity.operation_resources()
    }

    pub fn stage_resources_and_evidence(
        &self,
        stage_identity: &str,
    ) -> Option<(
        std::sync::Arc<worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan>,
        WorthQueryExecutionResourceAttemptEvidence,
    )>{
        self.affinity
            .managed_stage_resources_and_evidence(stage_identity)
    }

    pub fn provider_session_identity(&self) -> &str {
        self.affinity.provider_session_description()
    }

    pub fn bind_stage_commit_call(
        &self,
        stage_identity: &str,
        graph_authorities: &[&worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority],
        request: crate::domain_computation::WorthQueryGraphCommitCallRequest,
    ) -> Result<
        crate::domain_computation::WorthQueryGraphCommitCall,
        crate::domain_computation::WorthQueryGraphCallBindingDenial,
    > {
        let (resources, evidence) = self
            .affinity
            .managed_stage_resources_and_evidence(stage_identity)
            .ok_or(
                crate::domain_computation::WorthQueryGraphCallBindingDenial::ForeignResourceAttempt,
            )?;
        self.affinity.bind_graph_commit_call(
            graph_authorities,
            request,
            &evidence,
            resources.shared_envelope(),
        )
    }

    pub fn identity(&self) -> &str {
        self.affinity.attempt_identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.affinity.logical_identity()
    }

    pub fn evidence(&self) -> &WorthQueryExecutionResourceAttemptEvidence {
        self.affinity.evidence()
    }

    pub(in crate::domain_computation::managed_run) fn counters(
        &self,
    ) -> &WorthQueryManagedRunCounters {
        &self.counters
    }

    pub(in crate::domain_computation::managed_run) fn provider_session_description(&self) -> &str {
        self.affinity.provider_session_description()
    }

    pub(in crate::domain_computation::managed_run) fn retained_capacity_reservation_count(
        &self,
    ) -> usize {
        self.affinity.retained_capacity_reservation_count()
    }

    pub(in crate::domain_computation::managed_run) fn installation_is_current(&self) -> bool {
        self.affinity.installation_is_current()
    }

    pub(in crate::domain_computation::managed_run) fn yield_is_installed(&self) -> bool {
        self.affinity
            .operation_resources()
            .envelope()
            .yield_contract()
            .is_some()
    }

    pub(in crate::domain_computation::managed_run) fn artifact_registry_evidence(
        &self,
    ) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.artifacts.registry().evidence()
    }

    pub(in crate::domain_computation::managed_run) fn stage_artifact_context(
        &self,
        stage_identity: &str,
    ) -> Result<
        Option<
            crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepArtifactContext,
        >,
        crate::domain_computation::WorthQueryArtifactDenial,
    >{
        self.artifacts.production_authority(stage_identity).map(|authority| {
            authority.map(|authority| {
                crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryGraphProviderStepArtifactContext::new(
                    authority,
                    Arc::clone(&self.provider_artifact_occurrences),
                )
            })
        })
    }

    pub(in crate::domain_computation::managed_run) fn stage_graph_support_matches(
        &self,
        stage_identity: &str,
        role: &str,
        expected: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    ) -> bool {
        self.affinity
            .stage_graph_support_matches(stage_identity, role, expected)
    }

    pub(in crate::domain_computation::managed_run) fn admit_bridge_step_contract(
        &self,
        contract: worth_query_installation::facade::WorthQueryInstalledBoundedStepContract,
    ) -> Result<
        crate::domain_computation::managed_run::step_contract_admission::WorthQueryAdmittedManagedStepContract,
        crate::domain_computation::managed_run::step_contract_admission::WorthQueryManagedStepContractDenial,
    >{
        crate::domain_computation::managed_run::step_contract_admission::admit_managed_step_contract(
            contract,
            self.bridge_basis.step_contract(),
        )
    }

    pub(in crate::domain_computation::managed_run) fn bridge_basis_identity_projection(
        &self,
    ) -> &str {
        self.bridge_basis.identity().as_str()
    }

    pub(in crate::domain_computation::managed_run) fn bridge_request_identity_projection(
        &self,
    ) -> &str {
        self.bridge_basis.request().digest()
    }

    pub(in crate::domain_computation::managed_run) fn request_bridge_cancellation(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionCancellation,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.bridge_basis.request_cancellation(reason)
    }

    pub(in crate::domain_computation::managed_run) fn admit_bridge_ready_timeout(
        &self,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionTimeout,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.bridge_basis.admit_ready_timeout()
    }

    pub(in crate::domain_computation::managed_run) fn reject_bridge_execution(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionRejectionReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionRejection,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.bridge_basis.reject_execution(reason)
    }

    pub fn observe_safe_point(
        &self,
    ) -> Result<WorthQueryManagedSafePointObservation, WorthQueryManagedSafePointFailure> {
        crate::domain_computation::managed_run::safe_point_observation::observe_managed_run_safe_point(
            self.affinity.attempt_description_arc(),
            &self.bridge_basis,
        )
    }

    pub fn artifacts(&self) -> WorthQueryManagedWorkflowArtifactAuthority<'_> {
        WorthQueryManagedWorkflowArtifactAuthority::new(&self.artifacts)
    }

    pub fn begin_stage_graph_execution(
        self,
        stage_identity: &str,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<
        crate::domain_computation::managed_run::WorthQueryActiveWorkflowGraphExecution,
        crate::domain_computation::managed_run::WorthQueryWorkflowGraphExecutionStartFailure,
    > {
        crate::domain_computation::managed_run::workflow_graph_execution_start::begin(
            self,
            stage_identity,
            graph_authority,
            request,
        )
    }

    pub(in crate::domain_computation::managed_run) fn mint_stage_graph_provider_call(
        &self,
        stage_identity: &str,
        graph_authority: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<crate::domain_computation::WorthQueryGraphProviderCall, &'static str> {
        let (resources, evidence) = self
            .affinity
            .managed_stage_resources_and_evidence(stage_identity)
            .ok_or("workflow-stage-resources-unavailable")?;
        let request = WorthQueryGraphProviderCallRequest::workflow_stage(
            request.kind(),
            request.scope_identity(),
            stage_identity,
        )
        .with_managed_execution_snapshot(self.execution_snapshot_reference());
        let call = self
            .affinity
            .bind_graph_provider_call(
                graph_authority,
                request,
                &evidence,
                resources.shared_envelope(),
            )
            .map_err(|_| "workflow-provider-call-binding-denied")?;
        Ok(call)
    }

    pub(crate) fn execution_snapshot_reference(&self) -> String {
        let parts = self
            .bridge_basis
            .observation()
            .snapshot_identity()
            .relational_snapshot_parts()
            .expect("managed Relational workflow validates typed snapshot identity");
        format!(
            "worth-query-managed-snapshot|runtime={}|snapshot={}|version={}",
            self.relational_basis.identity().runtime_instance_id(),
            parts.snapshot_id(),
            parts.version_id(),
        )
    }

    pub(in crate::domain_computation::managed_run) fn completed_evidence_authority(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.affinity.completed_evidence_authority()
    }

    pub(in crate::domain_computation::managed_run) fn completed_evidence_session(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionProviderSession {
        self.affinity.completed_evidence_session()
    }
}
