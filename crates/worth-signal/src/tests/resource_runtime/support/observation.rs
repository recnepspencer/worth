use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::tests::resource_runtime) struct ResourceObservationRecord {
    pub(in crate::tests::resource_runtime) observer_id: u64,
    pub(in crate::tests::resource_runtime) handle_id: u64,
    pub(in crate::tests::resource_runtime) matched_node_count: usize,
    pub(in crate::tests::resource_runtime) visited: bool,
    pub(in crate::tests::resource_runtime) recomputed: bool,
    pub(in crate::tests::resource_runtime) meaningful_change: bool,
    pub(in crate::tests::resource_runtime) trigger_matched: bool,
}

pub(in crate::tests::resource_runtime) struct ResourceObservationListener {
    pub(in crate::tests::resource_runtime) calls: Arc<Mutex<Vec<ResourceObservationRecord>>>,
}

impl ObservationListener<(), (), (), (), ()> for ResourceObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        notice: &ObservationNotice<'_>,
    ) {
        self.calls
            .lock()
            .expect("resource observation mutex poisoned")
            .push(ResourceObservationRecord {
                observer_id: notice.observer_id().get(),
                handle_id: notice.handle_id().get(),
                matched_node_count: notice.matched_nodes().len(),
                visited: notice.visited(),
                recomputed: notice.recomputed(),
                meaningful_change: notice.meaningful_change(),
                trigger_matched: notice.trigger_matched(),
            });
    }
}

#[derive(Debug, Clone)]
pub(in crate::tests::resource_runtime) struct ResourceBranchReplayWorkloadBranchState {
    pub(in crate::tests::resource_runtime) branch_id: SignalBranchId,
    pub(in crate::tests::resource_runtime) head_snapshot_before_restore: Option<SignalSnapshotId>,
    pub(in crate::tests::resource_runtime) head_snapshot_after_restore: Option<SignalSnapshotId>,
    pub(in crate::tests::resource_runtime) replay_before_restore:
        ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) replay_after_snapshot_drift:
        ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) replay_after_restore: ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) replay_history_before_restore: ReplaySlice,
    pub(in crate::tests::resource_runtime) replay_history_after_restore: ReplaySlice,
    pub(in crate::tests::resource_runtime) diagnostics_after_restore: ResourceDiagnosticsSummary,
    pub(in crate::tests::resource_runtime) restore_report: ResourceBranchRestoreReport,
}

#[derive(Debug, Clone)]
pub(in crate::tests::resource_runtime) struct ResourceBranchReplayWorkloadOutcome {
    pub(in crate::tests::resource_runtime) feature: ResourceBranchReplayWorkloadBranchState,
    pub(in crate::tests::resource_runtime) sibling: ResourceBranchReplayWorkloadBranchState,
}

#[derive(Debug)]
pub(in crate::tests::resource_runtime) struct ResourceAsyncLifecycleRollbackWorkloadOutcome {
    pub(in crate::tests::resource_runtime) pre_rollback_replay: ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) post_rollback_replay: ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) control_path_replay: ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) diagnostics_after_rollback: ResourceDiagnosticsSummary,
    pub(in crate::tests::resource_runtime) rollback_report: ResourceCompletionRollbackReport,
    pub(in crate::tests::resource_runtime) rollback_observation: ResourceObservationBatchReport,
    pub(in crate::tests::resource_runtime) control_commit_observation:
        ResourceObservationBatchReport,
    pub(in crate::tests::resource_runtime) delivered_observations_after_rollback:
        Vec<ResourceObservationRecord>,
    pub(in crate::tests::resource_runtime) delivered_observations_after_control_commit:
        Vec<ResourceObservationRecord>,
}

#[derive(Debug)]
pub(in crate::tests::resource_runtime) struct ResourceAsyncInflightPressureWorkloadOutcome {
    pub(in crate::tests::resource_runtime) runtime_summary: ResourceRuntimeSummary,
    pub(in crate::tests::resource_runtime) replay_after_restore: ResourceReplayReconstructionReport,
    pub(in crate::tests::resource_runtime) telemetry: crate::data::telemetry::ResourceTelemetry,
    pub(in crate::tests::resource_runtime) pressure_performance:
        ResourceBoundaryPerformanceEnvelope,
    pub(in crate::tests::resource_runtime) pressure_batch: ResourceCompletionBatchAdmissionReport,
    pub(in crate::tests::resource_runtime) branch_restore_report: ResourceBranchRestoreReport,
    pub(in crate::tests::resource_runtime) drifted_branch_handle_live_after_restore: bool,
    pub(in crate::tests::resource_runtime) zombie_completion_after_restore:
        ResourceCompletionAdmissionReport,
    pub(in crate::tests::resource_runtime) pre_restore_completion_after_restore:
        ResourceCompletionAdmissionReport,
}
