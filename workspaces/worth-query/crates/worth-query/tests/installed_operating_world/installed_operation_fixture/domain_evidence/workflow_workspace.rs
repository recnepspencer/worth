use worth_query::facade::consumer_kit::{
    in_memory_test_runtime, WorthQueryTestBackendError, WorthQueryTestBackendSchema,
};
use worth_query::facade::{domain, runtime};

use super::super::workflow_parallel_providers::WorkflowParallelProvider;
use super::super::{identity_contract, GeometryDomain, ReadFamily, WorkflowRead};
use super::contract::artifact_support;
use super::graph_receipt::{evidence_graph_definition, EvidenceGraph, EvidenceGraphProvider};
use super::workflow_contract::{workflow_graph_package, workflow_package};
use super::workflow_executor::{
    EvidenceGraphWorkflowExecutor, EvidenceWorkflowExecutor, EvidenceWorkflowProbe,
};

pub fn evidence_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<(runtime::WorthQueryWorkspace, EvidenceWorkflowProbe), WorthQueryTestBackendError> {
    let schema = WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(identity_contract())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    let (executor, probe) = EvidenceWorkflowExecutor::new();
    let workspace = in_memory_test_runtime()
        .with_schema(schema)
        .domain_package_with_artifact_support(
            workflow_package(domain::WorthQueryArtifactRedactionPosture::NotRequired),
            artifact_support(),
        )
        .replayable_workflow_stage_executor(GeometryDomain, WorkflowRead, ReadFamily, executor)
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)?;
    Ok((workspace, probe))
}

pub fn evidence_graph_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<(runtime::WorthQueryWorkspace, EvidenceWorkflowProbe), WorthQueryTestBackendError> {
    let schema = WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(identity_contract())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    let (executor, probe) = EvidenceGraphWorkflowExecutor::new();
    let workspace = in_memory_test_runtime()
        .with_schema(schema)
        .domain_package_with_artifact_support(
            workflow_graph_package(domain::WorthQueryArtifactRedactionPosture::NotRequired),
            artifact_support(),
        )
        .graph_participation(evidence_graph_definition())
        .graph_participation_provider(EvidenceGraph, EvidenceGraphProvider)
        .replayable_workflow_stage_executor(GeometryDomain, WorkflowRead, ReadFamily, executor)
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .workspace(request_execution, name)?;
    Ok((workspace, probe))
}

pub fn evidence_workflow_intent() -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("start", Value::NotRequired),
        Stage::new("right", Value::Text("start".into())),
        Stage::new("left", Value::Text("start".into())),
        Stage::new("publish", Value::Text("join".into())),
    ])
    .unwrap()
}
