use worth_query::facade::domain;

use super::conditional_node_contract::dependency;
use super::installed_operation_fixture::{
    conditional_workflow_workspace, ConditionalModelGraph, GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn workflow_stage_dependency_resolves_only_through_its_exact_stage_location() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let stage_node = domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "publish-when-changed",
        domain::WorthQueryConditionalNodeRole::WorkflowStage,
    )
    .dependencies([dependency.clone()])
    .outputs([
        domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput {
            contract: domain::WorthQueryWorkflowValueContract::Projection,
        },
    ])
    .required_context([domain::WorthQueryConditionalNodeContext::WorkflowRun])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsWorkflowStageOutput)
    .finish()
    .unwrap();
    let workspace = conditional_workflow_workspace("stage-authority", stage_node).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let operating_world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let operation = operating_world
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap();
    let graph = workspace
        .graph_participation(ConditionalModelGraph)
        .unwrap();
    let stage_location = domain::WorthQueryConditionalNodeLocation::workflow_stage(
        "publish",
        "publish-when-changed",
    )
    .unwrap();
    let mut signal_graph = worth_signal::facade::SignalGraph::new();
    let signal_node = signal_graph.node().build();
    let worth_proof::TransitionOutcome::Success(signal_node) =
        signal_graph.admit_installed_node(signal_node)
    else {
        panic!("installed Signal node capability")
    };
    let target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::allocate(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "conditional-identity",
        ),
        worth_signal::facade::PartitionToken::new("geometry-signal"),
        signal_node,
    );
    let registration = operation
        .semantic_correspondence_registration(
            stage_location.clone(),
            0,
            &graph,
            Some(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(0, 0, 1),
            ),
            vec![target.clone()],
        )
        .expect("exact workflow-stage dependency should resolve");
    assert_eq!(
        registration.dependency().source_stage_identity(),
        Some("publish")
    );
    assert_eq!(
        registration.dependency().source_node_identity(),
        "publish-when-changed"
    );

    let operation_location =
        domain::WorthQueryConditionalNodeLocation::operation("publish-when-changed").unwrap();
    assert!(matches!(
        operation.semantic_correspondence_registration(
            operation_location,
            0,
            &graph,
            Some(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                    0, 0, 1,
                ),
            ),
            vec![target],
        ),
        Err(denial)
            if denial.kind()
                == worth_runtime_bridge::facade::BridgeCorrespondenceDenialKind::PortableDependencyNotOwnedByOperation
    ));
}
