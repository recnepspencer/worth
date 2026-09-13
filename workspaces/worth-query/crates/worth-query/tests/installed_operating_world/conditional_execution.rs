use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use worth_proof::TransitionOutcome;
use worth_query::facade::{
    certification::{
        host_conditional_signal_for_certification,
        WorthQueryHostConditionalSignalDecisionForCertification as HostDecision,
    },
    domain, read,
};

use super::conditional_node_contract::{conditional_node_result, dependency, GeometryCondition};
use super::installed_operation_fixture::conditional_workspace::conditional_workspace_without_lowering;
use super::installed_operation_fixture::{
    conditional_installation, conditional_workflow_workspace, conditional_workspace,
    conditional_workspace_with, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
    WorkflowRead,
};

mod providers;
use providers::{domain_condition_node, CapturingCompute, CountedCompute, StaticCondition};

#[test]
fn changed_signal_decision_reenters_before_the_ordinary_executor() {
    let node = super::conditional_node_contract::node(
        "geometry",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let mut workspace = conditional_workspace("conditional-changed", node).unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();

    let executed = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();

    assert_eq!(executed.conditional_provenance().len(), 1);
    assert_eq!(
        executed.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_eq!(executed.counters().conditional_compute_contacts, 1);
    assert_eq!(executed.counters().conditional_semantic_changes, 1);
    assert_eq!(executed.counters().executor_contacts, 1);
    let settled = executed
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap();
    assert_eq!(settled.conditional_provenance().len(), 1);
    assert_eq!(
        settled.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
}

#[test]
fn extra_condition_provider_is_rejected_during_runtime_construction() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "conditional-extra-provider",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::always_eligible(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(StaticCondition(
            worth_signal::facade::InstalledSignalConditionDecision::Eligible,
        ));
    let contacts = Arc::new(AtomicUsize::new(0));
    let result = conditional_workspace_with(
        "conditional-extra-provider",
        node,
        installation,
        CountedCompute::new(Arc::clone(&contacts), 1),
    );

    assert!(matches!(&result, Err(error) if error.message().contains("ExtraConditionProvider")));
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

#[test]
fn missing_conditional_lowering_is_rejected_during_runtime_construction() {
    let node = super::conditional_node_contract::node(
        "missing-lowering",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );

    let result = conditional_workspace_without_lowering("conditional-missing-lowering", node);

    assert!(
        matches!(&result, Err(error) if error.message().contains("require 1 conditional registrations, found 0"))
    );
}

#[test]
fn compute_receives_the_exact_bound_query_context() {
    let node = super::conditional_node_contract::node(
        "conditional-context",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&node);
    let captured = Arc::new(Mutex::new(None));
    let mut workspace = conditional_workspace_with(
        "conditional-context",
        node,
        installation,
        CapturingCompute(Arc::clone(&captured)),
    )
    .unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();
    let expected = (
        bound.definition().canonical_identity().to_string(),
        bound.binding_identity().to_string(),
        bound.basis_identity().to_string(),
    );

    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();

    let actual = captured.lock().unwrap().take().unwrap();
    assert_eq!((actual.0, actual.1, actual.2), expected);
    assert!(actual.3.is_none());
    assert!(!actual.4.is_empty());
    assert_eq!(actual.5, 1);
}

#[test]
fn workflow_stage_retains_the_same_signal_decision_in_its_receipt() {
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
    let mut workspace =
        conditional_workflow_workspace("conditional-workflow-execution", stage_node).unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap();
    let run = run
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance(
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "right",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap();

    let receipt = run.receipts().last().unwrap();
    assert_eq!(receipt.stage_identity(), "publish");
    assert_eq!(receipt.conditional_provenance().len(), 1);
    assert_eq!(
        receipt.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_eq!(receipt.counters().conditional_compute_contacts, 1);
}

#[test]
fn suppressed_decision_runs_no_query_graph_or_domain_work() {
    let node = domain_condition_node("conditional-suppressed");
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(StaticCondition(
            worth_signal::facade::InstalledSignalConditionDecision::Suppressed,
        ));
    let compute_contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "conditional-suppressed",
        node,
        installation,
        CountedCompute::new(Arc::clone(&compute_contacts), 1),
    )
    .unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();

    let TransitionOutcome::Deferred(deferred) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("suppressed Signal decision must remain a typed Query deferral")
    };

    assert_eq!(compute_contacts.load(Ordering::SeqCst), 0);
    assert_eq!(
        deferred.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::Suppressed
    );
    assert_eq!(
        host_conditional_signal_for_certification(&deferred.conditional_provenance()[0]),
        HostDecision::Suppressed
    );
    assert_eq!(deferred.counters().conditional_compute_contacts, 0);
    assert_eq!(deferred.counters().graph_provider_contacts, 0);
    assert_eq!(deferred.counters().executor_contacts, 0);
}

#[test]
fn reverted_clean_retains_compute_cost_but_mints_no_query_consequence() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "conditional-reverted-clean",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::always_eligible(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap();
    let installation = conditional_installation(&node);
    let compute_contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "conditional-reverted-clean",
        node,
        installation,
        CountedCompute::new(Arc::clone(&compute_contacts), 0),
    )
    .unwrap();
    let domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&domain, ReadVertex)
        .unwrap();

    let TransitionOutcome::Deferred(deferred) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("reverted-clean Signal decision must not invent Query output")
    };

    assert_eq!(compute_contacts.load(Ordering::SeqCst), 1);
    assert_eq!(
        deferred.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedRevertedClean
    );
    assert_eq!(deferred.counters().conditional_compute_contacts, 1);
    assert_eq!(deferred.counters().conditional_semantic_changes, 0);
    assert_eq!(deferred.counters().graph_provider_contacts, 0);
    assert_eq!(deferred.counters().executor_contacts, 0);
}
