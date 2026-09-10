use worth_query::facade::consumer_kit::{
    in_memory_test_runtime, WorthQueryControlledTestWorkspace,
    WorthQueryInMemoryTestRuntimeBuilder, WorthQueryTestBackendSchema,
};
use worth_query::facade::{domain, foundation, runtime};

use super::workflow_parallel_providers::SerialParallelProvider;
use super::{
    canonical_bundle, identity_contract, semantic_closure, GeometryDomain, ReadFamily, WorkflowRead,
};

#[path = "artifact_workflow/contract.rs"]
mod contract;
#[path = "artifact_workflow/definitions.rs"]
mod definitions;
#[path = "artifact_workflow/executor.rs"]
mod executor;
#[path = "artifact_workflow/integrated_evidence.rs"]
mod integrated_evidence;
#[path = "artifact_workflow/native_consumer.rs"]
mod native_consumer;
#[path = "artifact_workflow/native_observation.rs"]
mod native_observation;
#[path = "artifact_workflow/native_provider.rs"]
mod native_provider;
#[path = "artifact_workflow/provider.rs"]
mod provider;

pub use definitions::ArtifactWorkflowKind;
pub use native_observation::{
    ArtifactNativeDenial, ArtifactNativeLane, ArtifactNativeObservation, ArtifactNativeSuccess,
    ArtifactNativeValues,
};
pub use provider::ArtifactProbe;

use contract::{artifact_support, candidate_contract};
use definitions::workflow_definition;
use executor::ArtifactWorkflowExecutor;

pub fn artifact_move_workspace(
    name: &str,
) -> Result<
    (runtime::WorthQueryWorkspace, ArtifactProbe),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    artifact_workspace(name, ArtifactWorkflowKind::Move, true)
}

pub fn artifact_integrated_workspace(
    name: &str,
) -> Result<
    (runtime::WorthQueryWorkspace, ArtifactProbe),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    artifact_workspace(name, ArtifactWorkflowKind::Integrated, true)
}

pub fn artifact_lease_workspace(
    name: &str,
) -> Result<
    (runtime::WorthQueryWorkspace, ArtifactProbe),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    artifact_workspace(name, ArtifactWorkflowKind::Lease, true)
}

pub fn artifact_workspace_without_support(
    name: &str,
) -> Result<
    (runtime::WorthQueryWorkspace, ArtifactProbe),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    artifact_workspace(name, ArtifactWorkflowKind::Move, false)
}

pub fn artifact_controlled_workspace(
    name: &str,
) -> Result<
    (WorthQueryControlledTestWorkspace, ArtifactProbe),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let probe = ArtifactProbe::default();
    let workspace = artifact_runtime_builder(ArtifactWorkflowKind::Move, true, &probe)
        .controlled_workspace(name)?;
    Ok((workspace, probe))
}

pub fn bind_artifact_workflow(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}

pub fn move_intent(mode: &str) -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("produce", Value::Text(mode.to_owned())),
        Stage::new("consume", Value::predecessor_artifact("produce")),
    ])
    .unwrap()
}

pub fn lease_intent() -> domain::WorthQueryNormalizedWorkflowIntent {
    lease_intent_with_mode("produce")
}

pub fn lease_intent_with_mode(mode: &str) -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("produce", Value::Text(mode.into())),
        Stage::new(
            "observe-a",
            Value::predecessor_artifact_lease("produce", "observer-a"),
        ),
        Stage::new(
            "observe-b",
            Value::predecessor_artifact_lease("produce", "observer-b"),
        ),
    ])
    .unwrap()
}

fn artifact_workspace(
    name: &str,
    kind: ArtifactWorkflowKind,
    admit_artifact_support: bool,
) -> Result<
    (runtime::WorthQueryWorkspace, ArtifactProbe),
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let probe = ArtifactProbe::default();
    let workspace =
        artifact_runtime_builder(kind, admit_artifact_support, &probe).workspace(name)?;
    Ok((workspace, probe))
}

fn artifact_runtime_builder(
    kind: ArtifactWorkflowKind,
    admit_artifact_support: bool,
    probe: &ArtifactProbe,
) -> WorthQueryInMemoryTestRuntimeBuilder {
    let contract = candidate_contract();
    let package = artifact_package(workflow_definition(&contract, kind), contract);
    let schema = WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(identity_contract())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    let builder = in_memory_test_runtime().with_schema(schema);
    let builder = if admit_artifact_support {
        builder.domain_package_with_artifact_support(package, artifact_support())
    } else {
        builder.domain_package(package)
    };
    let builder = builder.replayable_workflow_stage_executor(
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
        ArtifactWorkflowExecutor::new(probe.clone()),
    );
    match kind {
        ArtifactWorkflowKind::Move | ArtifactWorkflowKind::Integrated => builder,
        ArtifactWorkflowKind::Lease => builder.workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            SerialParallelProvider,
        ),
    }
}

fn artifact_package(
    workflow: domain::WorthQueryPortableWorkflowDefinition,
    contract: domain::WorthQueryPortableArtifactContract,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::NotRequired,
        false,
    );
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::NotRequired;
    semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(workflow);
    semantics.replay = domain::WorthQueryOperationReplayContract::CertReplayable {
        comparator: domain::WorthQueryOperationReplayComparatorContract::new(
            "artifact-workflow-exact-v1",
        )
        .expect("static replay comparator family is portable"),
    };
    semantics.lowering = domain::WorthQueryOperationLoweringContract {
        family: "artifact-workflow-v1".into(),
        deterministic: true,
    };
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("artifact-workflow", 1),
        semantics,
    );
    domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
    .artifact_contract(contract)
}
