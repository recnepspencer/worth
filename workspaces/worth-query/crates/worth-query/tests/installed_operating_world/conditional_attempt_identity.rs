use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use worth_query::facade::{domain, foundation, read};

use super::conditional_node_contract::{conditional_node_result, dependency, ManualRefresh};
use super::installed_operation_fixture::{
    conditional_controlled_workspace, conditional_installation, conditional_workspace_with,
    GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

#[test]
fn conditional_authority_keeps_query_and_installation_runtime_owners_distinct() {
    let _unrelated_installation_runtime =
        worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity::fresh();
    let node = super::conditional_node_contract::node(
        "independent-runtime-authorities",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let mut workspace =
        conditional_controlled_workspace("independent-runtime-authorities", node).unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    let executed = bind(&workspace, &installed)
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
}

#[test]
fn fresh_bound_capabilities_mint_distinct_signal_attempt_identities() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "distinct-direct-attempts",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let versions = Arc::new(AtomicU64::new(0));
    let mut workspace = conditional_workspace_with(
        "distinct-direct-attempts",
        node,
        installation,
        AdvancingCompute(Arc::clone(&versions)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    let first = bind(&workspace, &installed)
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    let second = bind(&workspace, &installed)
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();

    assert_eq!(
        first.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_eq!(
        second.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_ne!(
        first.conditional_provenance()[0].signal_projection(),
        second.conditional_provenance()[0].signal_projection(),
        "two consumed bound capabilities are two Signal evaluation attempts"
    );
}

#[test]
fn live_promotion_mints_a_fresh_signal_decision_after_settlement() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "promotion-fresh-decision",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let versions = Arc::new(AtomicU64::new(0));
    let mut workspace = conditional_workspace_with(
        "promotion-fresh-decision",
        node,
        installation,
        AdvancingCompute(Arc::clone(&versions)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = bind(&workspace, &installed);
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
    let execution_signal = executed.conditional_provenance()[0]
        .signal_projection()
        .label()
        .to_string();
    let settled = executed
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap();

    let live = match settled.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("conditional projection did not promote"),
    };
    assert_eq!(live.receipt().counters().lifecycle_attempts, 1);
    assert_eq!(live.receipt().counters().fresh_conditional_decisions, 1);
    assert_ne!(
        live.conditional_provenance()[0]
            .signal_projection()
            .label()
            .as_ref(),
        execution_signal.as_str(),
        "promotion must not reuse settlement's Signal decision"
    );
}

#[test]
fn replacement_mints_a_fresh_signal_decision_for_the_successor() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "replacement-fresh-decision",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::on_demand(),
        domain::WorthQueryConditionalTrigger::on_demand::<ManualRefresh>(),
        domain::WorthQueryMaintenancePosture::OnDemandOnly,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().trigger(RequestedTrigger);
    let versions = Arc::new(AtomicU64::new(0));
    let mut workspace = conditional_workspace_with(
        "replacement-fresh-decision",
        node,
        installation,
        AdvancingCompute(Arc::clone(&versions)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let first = settle_bound(bind(&workspace, &installed), &mut workspace);
    let live = match first.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("conditional predecessor did not promote"),
    };
    let predecessor_signal = live.conditional_provenance()[0]
        .signal_projection()
        .label()
        .to_string();
    let candidate = settle_bound(bind(&workspace, &installed), &mut workspace).into_lifecycle();
    let witness = live.replacement_witness_for(&candidate).unwrap();
    let replaced = match live.replace_with(candidate, witness, &mut workspace) {
        domain::WorthQueryProjectionReplacementOutcome::Replaced(replaced) => replaced,
        _ => panic!("conditional replacement did not converge"),
    };

    assert_eq!(
        replaced
            .transition_work()
            .candidate()
            .fresh_conditional_decisions,
        1
    );
    assert_ne!(
        replaced.conditional_provenance()[0]
            .signal_projection()
            .label()
            .as_ref(),
        predecessor_signal.as_str(),
        "replacement must mint a new Signal attempt"
    );
}

#[test]
fn stale_conditional_installation_stops_before_lowering_or_signal_work() {
    let node = super::conditional_node_contract::node(
        "promotion-stale-conditional",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let mut workspace =
        conditional_controlled_workspace("promotion-stale-conditional", node).unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = bind(&workspace, &installed);
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
    workspace.advance_domain_installation_generation().unwrap();

    let stale = match settled.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Stale(stale) => stale,
        _ => panic!("stale conditional installation did not produce stale proof"),
    };
    assert_eq!(stale.counters().conditional_lowerings_checked, 0);
    assert_eq!(stale.counters().lifecycle_attempts, 0);
    assert_eq!(stale.counters().fresh_conditional_decisions, 0);
    assert_eq!(stale.counters().planning_attempts, 0);
}

fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(installed, ReadVertex)
        .unwrap()
}

fn settle_bound(
    bound: domain::WorthQueryBoundDomainOperation<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
        foundation::ObservationLaneWitness,
    >,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap()
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

struct AdvancingCompute(Arc<AtomicU64>);

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, ReadVertex, ReadFamily>
    for AdvancingCompute
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
        let version = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                version,
            )]),
        ))
    }
}
