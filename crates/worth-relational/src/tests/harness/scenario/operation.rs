use crate::facade::identity::PartitionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScenarioOperation {
    CreateEntity {
        partition: PartitionId,
        name: String,
    },
    UpdateEntity {
        entity_slot: usize,
        name: String,
        branch_slot: usize,
    },
    ReplaceEntity {
        entity_slot: usize,
        name: String,
        branch_slot: usize,
        partition: PartitionId,
    },
    CreateRelation {
        source_slot: usize,
        target_slot: usize,
        client_key: String,
        partition: PartitionId,
    },
    DeleteEntity {
        entity_slot: usize,
        branch_slot: usize,
    },
    DeleteRelation {
        relation_slot: usize,
    },
    CreateBranch {
        branch_name: String,
        from_branch_slot: usize,
    },
    MergeBranchIntoMain {
        branch_slot: usize,
    },
    CaptureSnapshot,
    ReleaseSnapshot {
        snapshot_slot: usize,
    },
    RunRetentionPass,
    DurableCheckpoint,
}
