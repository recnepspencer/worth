use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_proof::TransitionOutcome;
use worth_query::facade::{
    certification::{
        host_conditional_signal_for_certification,
        WorthQueryHostConditionalSignalDecisionForCertification as HostDecision,
    },
    domain, foundation,
};

use super::conditional_node_contract::{
    conditional_node_result, dependency, GeometryCondition, ManualRefresh,
};
use super::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};

#[test]
fn eligible_temporal_wake_is_checked_even_when_dependencies_are_unchanged() {
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
fn domain_predicate_deferral_is_not_reported_as_temporal_or_on_demand() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "predicate-deferred",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(DeferredCondition);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        "predicate-deferred",
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

    let TransitionOutcome::Deferred(stopped) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("the installed predicate should defer")
    };
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DeferredByCondition
    );
    assert_eq!(
        host_conditional_signal_for_certification(&stopped.conditional_provenance()[0]),
        HostDecision::Deferred
    );
    assert_eq!(stopped.counters().conditional_condition_deferrals, 1);
    assert_eq!(stopped.counters().conditional_temporal_deferrals, 0);
    assert_eq!(stopped.counters().conditional_on_demand_deferrals, 0);
    assert_eq!(stopped.counters().conditional_compute_contacts, 0);
    assert_eq!(stopped.counters().conditional_decisions_delivered, 1);
}

#[test]
fn unchanged_domain_predicate_stops_before_resolver_and_semantic_reads() {
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

fn execute_first(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) {
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(installed, ReadVertex)
        .unwrap();
    assert!(matches!(
        bound
            .admit_execution_resources(
                ReadExecutionInput::default(),
                crate::suite::installed_operation_fixture::execution_resource_request(),
                &*workspace
            )
            .unwrap()
            .execute(workspace),
        TransitionOutcome::Success(_)
    ));
}

fn execute_second(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> domain::WorthQueryDeferredDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(installed, ReadVertex)
        .unwrap();
    let TransitionOutcome::Deferred(stopped) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
    else {
        panic!("equivalent second compute should stop without a semantic change")
    };
    stopped
}

struct EligibleCondition;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for EligibleCondition {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalWakeProvider for EligibleCondition {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

struct DeferredCondition;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for DeferredCondition {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalConditionProvider for DeferredCondition {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Deferred)
    }
}

struct EligiblePredicate;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for EligiblePredicate {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalConditionProvider for EligiblePredicate {
    fn resolve(
        &self,
        _context: worth_runtime_bridge::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

struct RequestedTrigger;

impl worth_runtime_bridge::facade::BridgeConditionalProviderSemantics for RequestedTrigger {
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl worth_runtime_bridge::facade::BridgeConditionalTriggerProvider for RequestedTrigger {
    fn requested(&self) -> bool {
        true
    }
}

struct CountedCompute(Arc<AtomicUsize>);

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for CountedCompute
{
    type SemanticContract = ();

    fn semantic_contract(&self) -> Self::SemanticContract {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention,
        worth_runtime_bridge::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(worth_runtime_bridge::facade::BridgeConditionalProviderHeapRetention::none())
    }

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        crate::suite::installed_operation_fixture::execution_resource_support()
    }

    fn compute(
        &self,
        _context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}
