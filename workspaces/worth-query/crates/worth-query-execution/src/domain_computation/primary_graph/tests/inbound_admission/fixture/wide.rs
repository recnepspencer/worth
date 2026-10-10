use super::InboundWorld;
use crate::domain_computation::primary_graph::tests::inbound_admission::schema::WideNotifyOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

impl InboundWorld {
    pub fn commit_wide_dispatch(
        &self,
        phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        seed: u64,
        text: &str,
    ) -> WorthQueryApplicationCommitReceipt {
        let operation = self
            .application
            .installed_schema()
            .installed_operation(WideNotifyOperation::reference())
            .unwrap();
        self.commit_nonselected_dispatch(
            phase_custody,
            &phase_custody.borrowed_request_for_test(),
            operation,
            seed,
            text,
        )
    }
}
