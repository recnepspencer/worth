use worth_query::facade::domain;

use super::conditional_node_contract::{conditional_node_result, dependency, GeometryCondition};
use super::installed_operation_fixture::{
    conditional_installation_without_observation, conditional_workspace,
    conditional_workspace_with, DirectConditionalCompute,
};

#[test]
fn unsupported_eager_execution_is_denied_instead_of_degrading_to_lazy() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "unsupported-eager",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::EagerOnEligibleInvalidation,
    )
    .unwrap();

    let Err(error) = conditional_workspace(request_execution, "unsupported-eager", node) else {
        panic!("unsupported eager execution must not construct a runtime")
    };
    assert!(error.message().contains("UnsupportedMaintenancePosture"));
}

#[test]
fn unsupported_durable_artifact_is_denied_without_claiming_persistence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "unsupported-durable",
        domain::WorthQueryConditionalNodeRole::Computed,
    )
    .dependencies([dependency.clone()])
    .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
        projection_role: domain::WorthQueryOperationProjectionRole::new("vertex").unwrap(),
    }])
    .required_context([domain::WorthQueryConditionalNodeContext::Snapshot])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::OutputEquivalent,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::Durable,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::ContributesToOperationOutput)
    .finish()
    .unwrap();

    let Err(error) = conditional_workspace(request_execution, "unsupported-durable", node) else {
        panic!("unsupported durable artifacts must not construct a runtime")
    };
    assert!(error.message().contains("UnsupportedArtifactPosture"));
}

#[test]
fn partition_observation_requires_an_exact_observation_record() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let partition = worth_foundational::facade::TruthPartitionRole::new("model-main").unwrap();
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourcePartition(
        partition,
    ));
    let node = conditional_node_result(
        "unsupported-partition-observation",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap();

    conditional_workspace(
        request_execution,
        "supported-partition-observation",
        node.clone(),
    )
    .expect("partition invalidation with an exact observation record must install");

    let installation = conditional_installation_without_observation(&node);
    let Err(error) = conditional_workspace_with(
        request_execution,
        "missing-partition-observation",
        node,
        installation,
        DirectConditionalCompute,
    ) else {
        panic!("missing semantic observation identity must fail runtime construction")
    };
    assert!(error.message().contains("SnapshotAdmission"));
}
