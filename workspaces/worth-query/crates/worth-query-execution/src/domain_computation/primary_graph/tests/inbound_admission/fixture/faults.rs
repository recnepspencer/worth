//! Fault injection under the inbound fixture's open-owner assumption.

use super::InboundWorld;

impl InboundWorld {
    pub(in crate::domain_computation::primary_graph::tests::inbound_admission) fn fail_next_durable_append(
        &self,
    ) {
        self.application
            .fail_next_durable_append_for_test()
            .expect("the inbound fixture keeps its owner open");
    }
}
