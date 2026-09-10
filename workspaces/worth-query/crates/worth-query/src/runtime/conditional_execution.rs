use super::WorthQueryRuntime;

impl WorthQueryRuntime {
    pub fn conditional_evaluation_resource_observation(
        &self,
    ) -> Option<super::WorthQueryConditionalEvaluationResourceObservation> {
        self.installed_product.conditional_resource_observation()
    }

    pub(crate) fn conditional_nodes<D: 'static, O: 'static, F: 'static>(
        &self,
    ) -> Vec<std::sync::Arc<crate::domain_installation::WorthQueryInstalledConditionalNode>> {
        self.conditional_execution_registry
            .operation_nodes::<D, O, F>()
    }

    pub fn rebuild_conditional_execution_index(
        &mut self,
    ) -> crate::domain_installation::WorthQueryConditionalExecutionIndexRebuildReport {
        self.conditional_execution_registry
            .destroy_and_rebuild_index()
    }

    pub(crate) fn execute_conditional(
        &self,
        selected: &std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        >,
        request: worth_runtime_bridge::facade::BridgeConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<
        super::WorthQueryExecutedConditional,
        (
            worth_runtime_bridge::facade::BridgeConditionalDenialKind,
            String,
            worth_signal::facade::SignalConditionalDecisionCounters,
            usize,
        ),
    > {
        self.installed_product
            .execute_conditional(selected, request, context)
    }
}
