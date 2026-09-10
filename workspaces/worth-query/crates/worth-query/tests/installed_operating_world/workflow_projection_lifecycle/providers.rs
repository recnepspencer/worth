use super::*;

pub(crate) struct LifecycleWorkflowCompute(pub(crate) Arc<AtomicU64>);

pub(crate) struct RequestedTrigger;

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

impl domain::WorthQueryConditionalNodeComputeProvider<GeometryDomain, WorkflowRead, ReadFamily>
    for LifecycleWorkflowCompute
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
        context: &domain::WorthQueryConditionalComputeContext,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        if context.workflow_run_identity().is_none() {
            return Err("workflow lifecycle condition lost its originating run".into());
        }
        let version = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                version,
            )]),
        ))
    }
}
