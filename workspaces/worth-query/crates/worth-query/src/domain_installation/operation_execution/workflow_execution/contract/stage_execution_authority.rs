use super::{WorthQueryBoundGraphExecutionReceipt, WorthQueryWorkflowStageReceipt};

pub(crate) struct WorthQueryWorkflowStageExecutionAuthority<'a> {
    pub(crate) effect_workflow_binding: crate::workflow::WorkflowContextBinding,
    pub(crate) basis: crate::basis_lifecycle::BasisFamily,
    pub(crate) installed_read: Option<&'a crate::ordinary::read::WorthQueryReadDeclaration>,
    pub(crate) operation_graph_reads:
        &'a [worth_query_installation::facade::WorthQueryDomainOperationGraphReadRole],
    pub(crate) graph_receipts: &'a [WorthQueryBoundGraphExecutionReceipt],
    pub(crate) resources: &'a super::WorthQueryAdmittedExecutionResourcePlan,
    pub(crate) resource_evidence: &'a super::WorthQueryExecutionResourceAttemptEvidence,
    pub(crate) provider_session_identity: &'a str,
    pub(crate) query_authority: crate::identity_authority::QueryCanonicalAuthority,
    pub(crate) identity_evolution_basis_identity: String,
    pub(crate) artifact_access_authority:
        Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactAccessAuthority>>,
    pub(crate) artifact_production_authority:
        Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactProductionAuthority>>,
}

pub(crate) struct WorthQueryWorkflowStageExecutionScope<'a> {
    pub(crate) operation_identity: &'a str,
    pub(crate) binding_identity: &'a str,
    pub(crate) run_identity: &'a str,
    pub(crate) stage: &'a worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    pub(crate) predecessor_receipts: &'a [&'a WorthQueryWorkflowStageReceipt],
}
