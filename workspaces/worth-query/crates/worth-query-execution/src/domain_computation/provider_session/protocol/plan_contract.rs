use std::sync::Arc;

use super::declared_closure::WorthQueryProviderPlanDeclarations;
use super::execution_plan::WorthQueryValidatedProviderPlan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderOperationScope {
    Direct,
    WorkflowStage(Arc<str>),
}

impl WorthQueryProviderOperationScope {
    pub fn workflow_stage(identity: impl Into<Arc<str>>) -> Self {
        Self::WorkflowStage(identity.into())
    }

    pub fn stage_identity(&self) -> Option<&str> {
        match self {
            Self::Direct => None,
            Self::WorkflowStage(identity) => Some(identity),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderExecutionPlanContract {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    identity: Arc<str>,
    operation_identity: Arc<str>,
    binding_identity: Arc<str>,
    scope: WorthQueryProviderOperationScope,
    provider_role: Arc<str>,
    provider_identity: Arc<str>,
    provider_generation: u64,
    graph_authority_identity: Arc<str>,
    managed_run_identity: Arc<str>,
    execution_basis_identity: Arc<str>,
    admitted_session_identity: Arc<str>,
    resource_attempt_identity: Arc<str>,
    basis_identity: Arc<str>,
    snapshot_identity: Arc<str>,
    resource_envelope_identity: Arc<str>,
    read_closure: Arc<[String]>,
    touch_closure: Arc<[String]>,
    application_graph_reads:
        Option<worth_query_installation::facade::WorthQueryOperationGraphReadContract>,
    application_touches: Option<worth_query_installation::facade::WorthQueryOperationTouchContract>,
    application_read_touch_overlap:
        Option<worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex>,
    effect_closure: Arc<[String]>,
    invariant_closure: Arc<[String]>,
    artifact_closure: Arc<[String]>,
    decision_fact_families: Arc<[worth_query_installation::facade::WorthQueryDecisionFactFamily]>,
    invariant_requirements:
        Arc<[worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement]>,
    transaction_posture: Arc<str>,
    reconciliation_posture: Arc<str>,
    graph_work_session: Option<u64>,
    graph_work_managed_run: Option<u64>,
    application_operation_attempt:
        Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>,
    application_operation_slot: Option<Arc<str>>,
    application_schema_binding:
        Option<worth_query_installation::facade::ApplicationSchemaBindingIdentity>,
    application_snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    application_product_observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

impl WorthQueryProviderExecutionPlanContract {
    pub(crate) fn bind(
        execution: WorthQueryValidatedProviderPlan<'_>,
        declarations: &WorthQueryProviderPlanDeclarations,
        artifact_closure: Vec<String>,
    ) -> Option<Self> {
        let operation = execution.operation();
        let stage_identity = execution.stage_identity();
        let declared_closure = declarations.closure(stage_identity, execution.graph().role())?;
        let invariant_requirements =
            declarations.invariant_requirements_for(stage_identity, execution.graph().role());
        let mut closure = declared_closure.clone();
        closure.invariant = invariant_requirements
            .iter()
            .map(|requirement| requirement.slot().to_owned())
            .collect();
        let scope = stage_identity.map_or(WorthQueryProviderOperationScope::Direct, |identity| {
            WorthQueryProviderOperationScope::workflow_stage(identity.to_owned())
        });
        let transaction_posture = operation.commit_posture().as_str();
        let reconciliation_posture = declarations.reconciliation_posture();
        let graph_work = operation.graph_work_affinity();
        Some(Self {
            runtime_authority: operation.runtime_authority(),
            identity: execution.resource_attempt_identity().into(),
            operation_identity: operation.operation_identity().into(),
            binding_identity: operation.binding_identity().into(),
            scope,
            provider_role: execution.graph().role().into(),
            provider_identity: execution.provider_identity().into(),
            provider_generation: execution.provider_generation(),
            graph_authority_identity: execution.graph().authority_identity().into(),
            managed_run_identity: execution.managed_run_identity().into(),
            execution_basis_identity: execution.execution_basis_identity().into(),
            admitted_session_identity: execution.admitted_session_identity().into(),
            resource_attempt_identity: execution.resource_attempt_identity().into(),
            basis_identity: operation.basis_identity().into(),
            snapshot_identity: execution.snapshot_identity().into(),
            resource_envelope_identity: execution.resource_envelope_identity().into(),
            read_closure: closure.read.into(),
            touch_closure: closure.touch.into(),
            application_graph_reads: declarations.application_graph_reads().cloned(),
            application_touches: declarations.application_touches().cloned(),
            application_read_touch_overlap: declarations.application_read_touch_overlap().cloned(),
            effect_closure: closure.effect.into(),
            invariant_closure: closure.invariant.into(),
            artifact_closure: artifact_closure.into(),
            decision_fact_families: declarations.decision_fact_families().into(),
            invariant_requirements: invariant_requirements.into(),
            transaction_posture: transaction_posture.into(),
            reconciliation_posture: reconciliation_posture.into(),
            graph_work_session: graph_work.map(|affinity| affinity.session.as_u64()),
            graph_work_managed_run: graph_work.map(|affinity| affinity.managed_run.as_u64()),
            application_operation_attempt: operation.application_operation_attempt(),
            application_operation_slot: operation.application_operation_slot().cloned(),
            application_schema_binding: operation.application_schema_binding().cloned(),
            application_snapshot: operation.application_snapshot().cloned(),
            application_product_observation: operation.application_product_observation().cloned(),
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn runtime_authority(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub fn operation_identity(&self) -> &str {
        &self.operation_identity
    }

    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub fn scope(&self) -> &WorthQueryProviderOperationScope {
        &self.scope
    }

    pub fn provider_role(&self) -> &str {
        &self.provider_role
    }

    pub fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub fn provider_generation(&self) -> u64 {
        self.provider_generation
    }

    pub fn graph_authority_identity(&self) -> &str {
        &self.graph_authority_identity
    }

    pub fn managed_run_identity(&self) -> &str {
        &self.managed_run_identity
    }

    pub fn execution_basis_identity(&self) -> &str {
        &self.execution_basis_identity
    }

    pub fn admitted_session_identity(&self) -> &str {
        &self.admitted_session_identity
    }

    pub fn resource_attempt_identity(&self) -> &str {
        &self.resource_attempt_identity
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub fn snapshot_identity(&self) -> &str {
        &self.snapshot_identity
    }

    pub(crate) const fn application_operation_attempt(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>
    {
        self.application_operation_attempt
    }

    pub(crate) const fn application_schema_binding(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity> {
        self.application_schema_binding.as_ref()
    }

    pub(crate) fn application_operation_slot(&self) -> Option<&str> {
        self.application_operation_slot.as_deref()
    }

    pub(crate) const fn application_snapshot(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        self.application_snapshot.as_ref()
    }

    pub(crate) const fn application_product_observation(
        &self,
    ) -> Option<&worth_runtime_world::facade::ProductBranchObservation> {
        self.application_product_observation.as_ref()
    }

    pub fn resource_envelope_identity(&self) -> &str {
        &self.resource_envelope_identity
    }

    pub fn read_closure(&self) -> &[String] {
        &self.read_closure
    }

    pub fn touch_closure(&self) -> &[String] {
        &self.touch_closure
    }

    pub const fn application_graph_reads(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationGraphReadContract> {
        self.application_graph_reads.as_ref()
    }

    pub const fn application_touches(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationTouchContract> {
        self.application_touches.as_ref()
    }

    pub const fn application_read_touch_overlap(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex> {
        self.application_read_touch_overlap.as_ref()
    }

    pub fn effect_closure(&self) -> &[String] {
        &self.effect_closure
    }

    pub fn invariant_closure(&self) -> &[String] {
        &self.invariant_closure
    }

    pub fn artifact_closure(&self) -> &[String] {
        &self.artifact_closure
    }

    pub fn decision_fact_families(
        &self,
    ) -> &[worth_query_installation::facade::WorthQueryDecisionFactFamily] {
        &self.decision_fact_families
    }

    pub fn invariant_requirements(
        &self,
    ) -> &[worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement] {
        &self.invariant_requirements
    }

    pub fn transaction_posture(&self) -> &str {
        &self.transaction_posture
    }

    pub fn reconciliation_posture(&self) -> &str {
        &self.reconciliation_posture
    }

    pub const fn graph_work_session_identity(&self) -> Option<u64> {
        self.graph_work_session
    }

    pub const fn graph_work_managed_run_identity(&self) -> Option<u64> {
        self.graph_work_managed_run
    }

    pub(super) fn closure_width(&self) -> usize {
        let application_read_width = self.application_graph_reads.as_ref().map_or(0, |reads| {
            reads
                .roles()
                .iter()
                .map(|role| role.read_scopes().len())
                .sum()
        });
        let application_touch_width = self
            .application_touches
            .as_ref()
            .map_or(0, |touches| touches.scopes().len());
        self.read_closure.len()
            + self.touch_closure.len()
            + application_read_width
            + application_touch_width
            + self.effect_closure.len()
            + self.invariant_closure.len()
            + self.artifact_closure.len()
            + self.decision_fact_families.len()
            + self.invariant_requirements.len()
    }
}
