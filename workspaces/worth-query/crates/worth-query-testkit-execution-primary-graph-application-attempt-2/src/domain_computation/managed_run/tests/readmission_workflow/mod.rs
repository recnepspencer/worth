use crate::domain_computation::managed_run::tests::ManagedGraph;
use worth_runtime_bridge::facade::RuntimeBridge;
use worth_query_admission::facade::resource_admission::WorthQueryAdmittedWorkflowResourcePlan;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowGraphStepOutcome;
use crate::domain_computation::provider_session::tests::admitted_plan_with_graph_support;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture;
use crate::domain_computation::managed_run::tests::provider_work;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::workflow_authority_with_stage_graph_and_output_artifact;

pub use crate::domain_computation::managed_run::tests::readmission_workflow::readmission_artifact::ReadmissionArtifact;
use crate::domain_computation::managed_run::tests::readmission_workflow::yielded_run::yielded_workflow_fixture;
use crate::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
use crate::domain_computation::managed_run::tests::*;
use std::collections::BTreeMap;
use std::sync::Arc;
pub fn yielded_workflow(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: YieldProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority>,
) {
    yielded_workflow_for_stage(execution, provider, "producer", resource_request)
}
pub fn yielded_workflow_with_retained_artifact(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: YieldProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority>,
    worth_query_execution::facade::domain_computation::WorthQueryMoveOnlyArtifactHandle,
) {
    let (yielded, bridge, runtime, producer, artifact) =
        yielded_workflow_fixture(execution, provider, "producer", true, resource_request);
    (
        yielded,
        bridge,
        runtime,
        producer,
        artifact.expect("retained workflow artifact fixture must register"),
    )
}
pub fn yielded_workflow_for_stage(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: YieldProvider,
    stage_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    Arc<worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority>,
) {
    let (yielded, bridge, runtime, producer, artifact) =
        yielded_workflow_fixture(execution, provider, stage_identity, false, resource_request);
    debug_assert!(artifact.is_none());
    (yielded, bridge, runtime, producer)
}

#[path = "readmission_artifact/mod.rs"]
pub mod readmission_artifact;
#[path = "yielded_run/mod.rs"]
pub mod yielded_run;
