use crate::diagnostics::history::BridgeHistoricalMaterializationPath;
use crate::snapshot::HistoricalEvaluationDeclaration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BridgeHistoricalEvaluationCounters {
    truth_view_selector_count: usize,
    historical_truth_view_count: usize,
    branch_truth_view_count: usize,
    planned_truth_view_packet_count: usize,
    resolved_truth_view_policy_count: usize,
    materialized_truth_view_count: usize,
    truth_view_unavailable_count: usize,
    truth_view_branch_mismatch_count: usize,
    truth_view_snapshot_mismatch_count: usize,
    historical_replay_mismatch_count: usize,
    branch_local_evaluation_count: usize,
    truth_view_decision_log_count: usize,
    selector_width: usize,
    branch_width: usize,
    direct_snapshot_materialization_count: usize,
    commit_envelope_materialization_count: usize,
    branch_head_materialization_count: usize,
}

impl BridgeHistoricalEvaluationCounters {
    pub(crate) fn from_successful_materialization(
        declaration: &HistoricalEvaluationDeclaration,
        materialization_path: BridgeHistoricalMaterializationPath,
    ) -> Self {
        let mut counters = Self::base_for(declaration);
        counters.planned_truth_view_packet_count = 1;
        counters.resolved_truth_view_policy_count = 1;
        counters.materialized_truth_view_count = 1;
        counters.truth_view_decision_log_count = 1;
        match materialization_path {
            BridgeHistoricalMaterializationPath::DirectSnapshotRead => {
                counters.direct_snapshot_materialization_count = 1;
            }
            BridgeHistoricalMaterializationPath::PinnedCommitObservation => {}
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot => {
                counters.commit_envelope_materialization_count = 1;
            }
            BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot => {
                counters.branch_head_materialization_count = 1;
            }
        }
        counters
    }

    pub(crate) fn from_failed_materialization(
        declaration: &HistoricalEvaluationDeclaration,
        materialization_path: BridgeHistoricalMaterializationPath,
    ) -> Self {
        let mut counters = Self::base_for(declaration);
        counters.planned_truth_view_packet_count = 1;
        counters.resolved_truth_view_policy_count = 1;
        counters.truth_view_decision_log_count = 1;
        match materialization_path {
            BridgeHistoricalMaterializationPath::DirectSnapshotRead => {
                counters.direct_snapshot_materialization_count = 1;
            }
            BridgeHistoricalMaterializationPath::PinnedCommitObservation => {}
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot => {
                counters.commit_envelope_materialization_count = 1;
            }
            BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot => {
                counters.branch_head_materialization_count = 1;
            }
        }
        counters
    }

    fn base_for(declaration: &HistoricalEvaluationDeclaration) -> Self {
        let mut counters = Self {
            truth_view_selector_count: 1,
            selector_width: 1,
            branch_width: 1,
            ..Self::default()
        };
        match declaration.selector().view_kind() {
            crate::snapshot::BridgeTruthViewKind::HistoricalCommit => {
                counters.historical_truth_view_count = 1;
            }
            crate::snapshot::BridgeTruthViewKind::BranchHead => {
                counters.branch_truth_view_count = 1;
                counters.branch_local_evaluation_count = 1;
            }
            crate::snapshot::BridgeTruthViewKind::BranchSnapshot => {
                counters.branch_truth_view_count = 1;
                counters.branch_local_evaluation_count = 1;
            }
            crate::snapshot::BridgeTruthViewKind::BranchCommit => {
                counters.historical_truth_view_count = 1;
                counters.branch_truth_view_count = 1;
                counters.branch_local_evaluation_count = 1;
            }
            crate::snapshot::BridgeTruthViewKind::CommittedSnapshot => {}
        }
        counters
    }

    pub fn truth_view_selector_count(&self) -> usize {
        self.truth_view_selector_count
    }

    pub fn historical_truth_view_count(&self) -> usize {
        self.historical_truth_view_count
    }

    pub fn branch_truth_view_count(&self) -> usize {
        self.branch_truth_view_count
    }

    pub fn planned_truth_view_packet_count(&self) -> usize {
        self.planned_truth_view_packet_count
    }

    pub fn resolved_truth_view_policy_count(&self) -> usize {
        self.resolved_truth_view_policy_count
    }

    pub fn materialized_truth_view_count(&self) -> usize {
        self.materialized_truth_view_count
    }

    pub fn truth_view_unavailable_count(&self) -> usize {
        self.truth_view_unavailable_count
    }

    pub fn truth_view_branch_mismatch_count(&self) -> usize {
        self.truth_view_branch_mismatch_count
    }

    pub fn truth_view_snapshot_mismatch_count(&self) -> usize {
        self.truth_view_snapshot_mismatch_count
    }

    pub fn historical_replay_mismatch_count(&self) -> usize {
        self.historical_replay_mismatch_count
    }

    pub fn branch_local_evaluation_count(&self) -> usize {
        self.branch_local_evaluation_count
    }

    pub fn truth_view_decision_log_count(&self) -> usize {
        self.truth_view_decision_log_count
    }

    pub fn selector_width(&self) -> usize {
        self.selector_width
    }

    pub fn branch_width(&self) -> usize {
        self.branch_width
    }

    pub fn direct_snapshot_materialization_count(&self) -> usize {
        self.direct_snapshot_materialization_count
    }

    pub fn commit_envelope_materialization_count(&self) -> usize {
        self.commit_envelope_materialization_count
    }

    pub fn branch_head_materialization_count(&self) -> usize {
        self.branch_head_materialization_count
    }

    pub(crate) fn with_unavailable_truth_view(mut self) -> Self {
        self.truth_view_unavailable_count += 1;
        self
    }

    pub(crate) fn with_branch_mismatch(mut self) -> Self {
        self.truth_view_branch_mismatch_count += 1;
        self
    }

    pub(crate) fn with_snapshot_mismatch(mut self) -> Self {
        self.truth_view_snapshot_mismatch_count += 1;
        self
    }

    pub(crate) fn with_historical_replay_mismatch(mut self) -> Self {
        self.historical_replay_mismatch_count += 1;
        self
    }
}
