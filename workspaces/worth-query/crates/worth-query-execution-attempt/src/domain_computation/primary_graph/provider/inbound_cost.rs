use worth_query_installation::facade::InstalledInboundOccurrenceContract;

use super::WorthQueryPrimaryGraphProvider;

impl WorthQueryPrimaryGraphProvider {
    pub fn outstanding_dispatch_count_for_operation(
        &self,
        contract: &InstalledInboundOccurrenceContract,
    ) -> u64 {
        self.outstanding_dispatch.count_for_contract(contract)
    }
}
