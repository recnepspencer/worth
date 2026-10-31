//! Certification interruption is delivered through the actual request cancellation source.
use super::WorthQueryPrimaryGraphProvider;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn inspection_materialization_prepared(&self) {
        self.fault_port.inspection_materialization_prepared();
    }
    pub(in crate::domain_computation::primary_graph) fn cancel_after_next_inspection_materialization_for_test(
        &self,
        source:worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
    ) -> bool {
        self.fault_port
            .cancel_after_next_inspection_materialization_for_test(source)
    }
}
