use worth_query_installation::facade::InstalledInboundOccurrenceContract;

use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn outstanding_dispatch_count_for_operation(
        &self,
        contract: &InstalledInboundOccurrenceContract,
    ) -> u64 {
        self.outstanding_dispatch.count_for_contract(contract)
    }
}
