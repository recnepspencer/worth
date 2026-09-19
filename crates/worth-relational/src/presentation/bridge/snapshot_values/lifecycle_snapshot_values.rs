use worth_foundational::facade::AspectValue;

use crate::storage::data::RecordLifecycleState;

pub(super) fn lifecycle_aspect_value(lifecycle: RecordLifecycleState) -> AspectValue {
    AspectValue::String(lifecycle_snapshot_label(lifecycle).into())
}

fn lifecycle_snapshot_label(lifecycle: RecordLifecycleState) -> &'static str {
    match lifecycle {
        RecordLifecycleState::Live => "live",
        RecordLifecycleState::DeletedRetained => "deleted_retained",
        RecordLifecycleState::RetainedDanglingForAudit => "retained_dangling_for_audit",
        RecordLifecycleState::PinnedBySnapshot => "pinned_by_snapshot",
        RecordLifecycleState::PinnedByBranch => "pinned_by_branch",
        RecordLifecycleState::PinnedByReplayRetention => "pinned_by_replay_retention",
        RecordLifecycleState::Reclaimable => "reclaimable",
        RecordLifecycleState::Reusable => "reusable",
        RecordLifecycleState::MaterializationUnavailable => "materialization_unavailable",
    }
}
