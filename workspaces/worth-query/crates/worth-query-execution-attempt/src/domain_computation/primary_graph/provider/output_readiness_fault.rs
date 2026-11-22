use super::{fault_port::WorthQueryPrimaryGraphFault, WorthQueryPrimaryGraphProvider};

impl WorthQueryPrimaryGraphProvider {
    pub fn output_comparison_completed(&self) {
        self.fault_port.output_comparison_completed();
    }
    pub fn cancel_after_next_output_comparison_for_test(
        &self,
        source: worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
    ) -> bool {
        self.fault_port
            .cancel_after_next_output_comparison_for_test(source)
    }

    pub fn take_delayed_output_readiness_delivery(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery)
    }

    pub fn take_failed_output_readiness_evaluation(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::FailedOutputReadinessEvaluation)
    }

    pub fn take_ready_read_snapshot_pressure(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::ReadyReadSnapshotPressure)
    }

    pub fn take_readiness_snapshot_pressure(&self) -> bool {
        self.take_fault(WorthQueryPrimaryGraphFault::ReadinessSnapshotPressure)
    }

    pub fn delay_next_output_readiness_delivery_for_test(&self) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::DelayedOutputReadinessDelivery));
    }

    pub fn fail_next_output_readiness_evaluation_for_test(&self) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::FailedOutputReadinessEvaluation));
    }

    pub fn press_next_ready_read_with_world_snapshots_for_test(&self) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::ReadyReadSnapshotPressure));
    }

    pub fn press_next_readiness_with_world_snapshots_for_test(&self) {
        assert!(self
            .fault_port
            .schedule_for_test(WorthQueryPrimaryGraphFault::ReadinessSnapshotPressure));
    }
}
