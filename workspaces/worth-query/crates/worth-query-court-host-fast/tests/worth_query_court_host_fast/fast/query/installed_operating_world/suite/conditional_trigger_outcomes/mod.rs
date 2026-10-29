pub(crate) use worth_query_worlds::facade::query::suite::conditional_trigger_outcomes::*;

pub(crate) mod domain_predicate_deferral_is_not_reported_as_temporal_or_on_dema;

#[test]
fn eligible_temporal_wake_is_checked_even_when_dependencies_are_unchanged() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "temporal-eligible",
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
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().wake(EligibleCondition);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        request_execution,
        "temporal-eligible",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    execute_first(&mut workspace, &installed);
    let stopped = execute_second(&mut workspace, &installed);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedRevertedClean
    );
    assert_eq!(
        host_conditional_signal_for_certification(&stopped.conditional_provenance()[0]),
        HostDecision::RevertedClean
    );
    assert_eq!(stopped.counters().conditional_condition_checks, 1);
    assert_eq!(stopped.counters().conditional_compute_contacts, 1);
    assert_eq!(stopped.counters().conditional_reverted_clean_outcomes, 1);
    assert_eq!(stopped.counters().conditional_semantic_changes, 0);
    assert_eq!(stopped.counters().conditional_decisions_delivered, 1);
}

#[test]
fn requested_on_demand_trigger_is_not_short_circuited_by_unchanged_dependencies() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "on-demand-requested",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        request_execution,
        "on-demand-requested",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    execute_first(&mut workspace, &installed);
    let stopped = execute_second(&mut workspace, &installed);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedRevertedClean
    );
    assert_eq!(stopped.counters().conditional_reverted_clean_outcomes, 1);
    assert_eq!(stopped.counters().conditional_on_demand_deferrals, 0);
    assert_eq!(stopped.counters().conditional_decisions_delivered, 1);
}

#[test]
fn unchanged_domain_predicate_stops_before_resolver_and_semantic_reads() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "predicate-unchanged",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(EligiblePredicate);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        request_execution,
        "predicate-unchanged",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    execute_first(&mut workspace, &installed);
    let stopped = execute_second(&mut workspace, &installed);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DependencyUnchanged
    );
    assert_eq!(stopped.counters().conditional_condition_checks, 0);
    assert_eq!(stopped.counters().conditional_semantic_reads, 0);
}
