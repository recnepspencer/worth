use super::InboundWorld;
use crate::domain_computation::primary_graph::tests::inbound_admission::schema::WideNotifyOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

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
