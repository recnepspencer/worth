pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::Arc;

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{
    certification::{
        host_conditional_signal_for_certification,
        WorthQueryHostConditionalSignalDecisionForCertification as HostDecision,
    },
    domain, foundation,
};

pub use super::conditional_node_contract::{
    conditional_node_result, dependency, GeometryCondition, ManualRefresh,
};
pub use super::installed_operation_fixture::{
    conditional_installation, conditional_workspace_with, GeometryDomain, ReadExecutionInput,
    ReadFamily, ReadVertex,
};

#[path = "conditional_trigger_outcomes/domain_predicate_deferral_is_not_reported_as_temporal_or_on_dema.rs"]
pub mod domain_predicate_deferral_is_not_reported_as_temporal_or_on_dema;

pub fn execute_first(
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
                crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
                &*workspace
            )
            .unwrap()
            .execute(workspace),
        TransitionOutcome::Success(_)
    ));
}

pub fn execute_second(
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
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
    else {
        panic!("equivalent second compute should stop without a semantic change")
    };
    stopped
}

pub struct EligibleCondition;

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

pub struct DeferredCondition;

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

pub struct EligiblePredicate;

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

pub struct RequestedTrigger;

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

pub struct CountedCompute(pub Arc<AtomicUsize>);

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
        crate::facade::query::suite::installed_operation_fixture::execution_resource_support()
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
