use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::InboundWorld;
use crate::domain_computation::primary_graph::tests::inbound_admission::schema::WideNotifyOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;

impl InboundWorld {
    pub fn commit_wide_dispatch(
        &self,
        seed: u64,
        text: &str,
    ) -> WorthQueryApplicationCommitReceipt {
        let operation = self
            .application
            .installed_schema()
            .installed_operation(WideNotifyOperation::reference())
            .unwrap();
        self.commit_nonselected_dispatch(operation, seed, text)
    }
}
