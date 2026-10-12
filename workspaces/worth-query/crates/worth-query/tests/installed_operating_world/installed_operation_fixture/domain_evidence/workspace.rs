use worth_query::facade::consumer_kit::{
    in_memory_test_runtime, WorthQueryTestBackendError, WorthQueryTestBackendSchema,
};
use worth_query::facade::{domain, runtime};

use super::super::{identity_contract, GeometryDomain};
use super::contract::{
    artifact_support, direct_graph_package, direct_package, direct_package_with_governance,
    EvidenceGovernance,
};
use super::direct_executor::{EvidenceDirectExecutor, EvidenceGraphDirectExecutor};
use super::graph_receipt::{evidence_graph_definition, EvidenceGraph, EvidenceGraphProvider};
use super::{EvidenceFamily, EvidenceRead, EvidenceScenario};

pub fn evidence_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    scenario: EvidenceScenario,
    redaction: domain::WorthQueryArtifactRedactionPosture,
) -> Result<runtime::WorthQueryWorkspace, WorthQueryTestBackendError> {
    build_workspace(request_execution, name, scenario, direct_package(redaction))
}

pub fn evidence_workspace_with_governance(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    scenario: EvidenceScenario,
    governance: EvidenceGovernance,
) -> Result<runtime::WorthQueryWorkspace, WorthQueryTestBackendError> {
    build_workspace(
        request_execution,
        name,
        scenario,
        direct_package_with_governance(governance),
    )
}

pub fn evidence_graph_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> Result<runtime::WorthQueryWorkspace, WorthQueryTestBackendError> {
    in_memory_test_runtime()
        .with_schema(evidence_schema())
        .domain_package_with_artifact_support(
            direct_graph_package(domain::WorthQueryArtifactRedactionPosture::NotRequired),
            artifact_support(),
        )
        .graph_participation(evidence_graph_definition())
        .graph_participation_provider(EvidenceGraph, EvidenceGraphProvider)
        .domain_operation_executor(
            GeometryDomain,
            EvidenceRead,
            EvidenceFamily,
            EvidenceGraphDirectExecutor::honest(),
        )
        .workspace(request_execution, name)
}

fn build_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    scenario: EvidenceScenario,
    package: domain::WorthQueryDomainPackage<GeometryDomain>,
) -> Result<runtime::WorthQueryWorkspace, WorthQueryTestBackendError> {
    let schema = evidence_schema();
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package_with_artifact_support(package, artifact_support())
        .domain_operation_executor(
            GeometryDomain,
            EvidenceRead,
            EvidenceFamily,
            EvidenceDirectExecutor::new(scenario),
        )
        .workspace(request_execution, name)
}

fn evidence_schema() -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(identity_contract())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
}
