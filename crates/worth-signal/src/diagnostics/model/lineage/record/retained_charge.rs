use super::{LineageRecord, LineageRecordKind};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for LineageRecord {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            sequence: _,
            emitted_on_branch_id: _,
            kind,
        } = self;
        kind.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for LineageRecordKind {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::ArtifactTransition {
                node: _,
                artifact_id: _,
                parent_artifact_id: _,
                execution_record_id: _,
                semantic_segment_id: _,
                transition,
            } => transition.retained_heap_charge(work),
            Self::BranchFork {
                created_branch_id: _,
                parent_branch_id: _,
                created_branch_display_name,
                parent_branch_display_name,
            } => created_branch_display_name
                .retained_heap_charge(work)?
                .checked_add(parent_branch_display_name.retained_heap_charge(work)?),
            Self::BranchSwitch {
                from_branch_id: _,
                to_branch_id: _,
                from_branch_display_name,
                to_branch_display_name,
            } => from_branch_display_name
                .retained_heap_charge(work)?
                .checked_add(to_branch_display_name.retained_heap_charge(work)?),
            Self::SnapshotRestore {
                snapshot_id: _,
                node: _,
                artifact_id: _,
                restore_kind: _,
            } => Ok(Charge::ZERO),
            Self::Invalidation {
                node: _,
                artifact_id: _,
                cause,
            } => cause.retained_heap_charge(work),
        }
    }
}
#[cfg(test)]
#[path = "retained_variant_tests.rs"]
mod tests;
