use super::*;

pub(super) struct StrategyReplayRecoveryInput {
    pub(super) runtime: RelationalRuntime,
    pub(super) recovered_root: std::sync::Arc<crate::tests::support::TestDurableMedium>,
    pub(super) entity: crate::facade::identity::EntityId,
    pub(super) feature_branch: BranchId,
    pub(super) aspect_overlap_branch: BranchId,
    pub(super) aspect_disjoint_branch: BranchId,
    pub(super) controller_sequence_branch: BranchId,
    pub(super) main_commit: crate::facade::transactions::CommitResult,
    pub(super) feature_commit: crate::facade::transactions::CommitResult,
    pub(super) controller_feature_idempotent_commit: crate::facade::transactions::CommitResult,
    pub(super) replacement_certification: ReplacementCertificationBundle,
}

#[path = "replay_recovery/certification.rs"]
mod certification;
pub(super) use certification::certify_replay_recovery;
