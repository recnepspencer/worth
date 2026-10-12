use super::{fault_port::WorthQueryPrimaryGraphFault, WorthQueryPrimaryGraphProvider};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn take_delayed_output_readiness_delivery(
        &self,
    ) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery)
    }

    pub(in crate::domain_computation::primary_graph) fn take_failed_output_readiness_evaluation(
        &self,
    ) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::FailedOutputReadinessEvaluation)
    }

    pub(in crate::domain_computation::primary_graph) fn take_ready_read_snapshot_pressure(
        &self,
    ) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::ReadyReadSnapshotPressure)
    }

    pub(in crate::domain_computation::primary_graph) fn take_readiness_snapshot_pressure(
        &self,
    ) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::ReadinessSnapshotPressure)
    }

    pub(in crate::domain_computation::primary_graph) fn delay_next_output_readiness_delivery_for_test(
        &self,
    ) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery));
    }

    pub(in crate::domain_computation::primary_graph) fn fail_next_output_readiness_evaluation_for_test(
        &self,
    ) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::FailedOutputReadinessEvaluation));
    }

    pub(in crate::domain_computation::primary_graph) fn press_next_ready_read_with_world_snapshots_for_test(
        &self,
    ) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::ReadyReadSnapshotPressure));
    }

    pub(in crate::domain_computation::primary_graph) fn press_next_readiness_with_world_snapshots_for_test(
        &self,
    ) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::ReadinessSnapshotPressure));
    }
}
