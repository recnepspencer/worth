use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_proof::TransitionOutcome;
use worth_query::facade::{
    certification::{
        host_conditional_signal_for_certification,
        WorthQueryHostConditionalSignalDecisionForCertification as HostDecision,
    },
    domain, read,
};

mod installation_denials;
mod provider_fixtures;

use super::conditional_node_contract::{conditional_node_result, dependency, node, ManualRefresh};
use super::installed_operation_fixture::conditional_workspace::shared_signal_node_workspace;
use super::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};
use provider_fixtures::{CountedCompute, DeferredWake, DetachedCompute, UnrequestedTrigger};

#[test]
fn unchanged_correspondence_versions_stop_before_condition_and_compute() {
    let node = node(
        "dependency-versioned",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&node);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "dependency-versioned",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let bound = world
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let first = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(first.counters().conditional_compute_contacts, 1);
    assert_eq!(first.counters().conditional_semantic_changes, 1);
    assert_eq!(first.counters().conditional_decisions_delivered, 1);
    assert_eq!(
        host_conditional_signal_for_certification(&first.conditional_provenance()[0]),
        HostDecision::Eligible
    );
    drop(first);

    let world = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap();
    let bound = world
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let TransitionOutcome::Deferred(second) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("unchanged semantic dependency versions must defer Query work")
    };

    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(
        second.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DependencyUnchanged
    );
    assert_eq!(
        host_conditional_signal_for_certification(&second.conditional_provenance()[0]),
        HostDecision::DependencyUnchanged
    );
    assert!(second.conditional_provenance()[0].artifact_reuse_admitted());
    assert_eq!(second.counters().conditional_dependency_checks, 1);
    assert_eq!(second.counters().conditional_comparator_checks, 1);
    assert_eq!(second.counters().conditional_condition_checks, 0);
    assert_eq!(second.counters().conditional_condition_deferrals, 0);
    assert_eq!(second.counters().conditional_temporal_deferrals, 0);
    assert_eq!(second.counters().conditional_on_demand_deferrals, 0);
    assert_eq!(second.counters().conditional_reuse_checks, 1);
    assert_eq!(second.counters().conditional_decisions_delivered, 1);
    assert_eq!(second.counters().graph_provider_contacts, 0);
    assert_eq!(second.counters().executor_contacts, 0);
}

#[test]
fn unchanged_dependency_opens_live_continuity_without_new_semantic_output() {
    let node = node(
        "promotion-dependency-unchanged",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&node);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "promotion-dependency-unchanged",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    let settled = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap();

    let live = match settled.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("fresh unchanged evidence did not admit live continuity"),
    };
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(live.receipt().counters().lifecycle_attempts, 1);
    assert_eq!(live.receipt().counters().fresh_conditional_decisions, 1);
    assert_eq!(live.receipt().counters().conditional_compute_contacts, 0);
    assert_eq!(live.receipt().counters().conditional_semantic_changes, 0);
    assert_eq!(
        live.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DependencyUnchanged
    );
    let refresh = live.refresh(&mut workspace).unwrap();
    assert!(refresh.delivery().is_empty());
    assert_eq!(refresh.work().delivery_batches(), 0);
}

#[test]
fn unrequested_on_demand_node_defers_without_compute_or_query_work() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "on-demand-deferred",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .trigger(UnrequestedTrigger);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "on-demand-deferred",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
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
        panic!("an unrequested on-demand node must remain deferred")
    };

    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(
        deferred.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DeferredOnDemand
    );
    assert_eq!(
        host_conditional_signal_for_certification(&deferred.conditional_provenance()[0]),
        HostDecision::Deferred
    );
    assert_eq!(deferred.counters().conditional_compute_contacts, 0);
    assert_eq!(deferred.counters().conditional_condition_checks, 1);
    assert_eq!(deferred.counters().conditional_on_demand_deferrals, 1);
    assert_eq!(deferred.counters().conditional_temporal_deferrals, 0);
    assert_eq!(deferred.counters().conditional_decisions_delivered, 1);
    assert_eq!(deferred.counters().graph_provider_contacts, 0);
    assert_eq!(deferred.counters().executor_contacts, 0);
}

#[test]
fn temporal_wake_defers_without_compute_or_query_work() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "temporal-deferred",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::temporal(
            domain::WorthQueryTemporalCondition::IntervalNanoseconds(1_000),
        ),
        domain::WorthQueryConditionalTrigger::Temporal(
            domain::WorthQueryTemporalWake::MonotonicClock,
        ),
        domain::WorthQueryMaintenancePosture::Temporal,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().wake(DeferredWake);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "temporal-deferred",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
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
        panic!("an unready temporal wake must remain deferred")
    };

    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(
        deferred.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DeferredTemporal
    );
    assert_eq!(
        host_conditional_signal_for_certification(&deferred.conditional_provenance()[0]),
        HostDecision::Deferred
    );
    assert_eq!(deferred.counters().conditional_compute_contacts, 0);
    assert_eq!(deferred.counters().conditional_condition_checks, 1);
    assert_eq!(deferred.counters().conditional_temporal_deferrals, 1);
    assert_eq!(deferred.counters().conditional_on_demand_deferrals, 0);
    assert_eq!(deferred.counters().conditional_decisions_delivered, 1);
    assert_eq!(deferred.counters().graph_provider_contacts, 0);
    assert_eq!(deferred.counters().executor_contacts, 0);
}
