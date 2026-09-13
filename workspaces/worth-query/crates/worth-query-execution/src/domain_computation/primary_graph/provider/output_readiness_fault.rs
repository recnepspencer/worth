use super::{fault_port::WorthQueryPrimaryGraphFault, WorthQueryPrimaryGraphProvider};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn take_delayed_output_readiness_delivery(
        &self,
    ) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery)
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn delay_next_output_readiness_delivery_for_test(
        &self,
    ) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery));
    }
}
