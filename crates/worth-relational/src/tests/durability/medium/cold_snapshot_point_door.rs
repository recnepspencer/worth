//! Issued handles select an exact root but do not prove its region residency.
use super::*;
#[derive(Debug, Clone, Copy)]
pub(super) enum SnapshotPointDoor {
    KindStatus,
    Kind,
    AspectRevision,
}
impl SnapshotPointDoor {
    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        entity: EntityId,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        let snapshot =
            snapshot_for_owner_branch(runtime, &crate::history::data::BranchId("main".into()));
        let read = runtime.read_truth();
        let answer = (|| {
            match self {
                Self::KindStatus => assert_eq!(
                    read.read_exact_snapshot_live_entity_kind_status(execution, &snapshot, entity)?,
                    Some(Some(KindId(1)))
                ),
                Self::Kind => assert_eq!(
                    read.read_exact_snapshot_live_entity_kind(execution, &snapshot, entity)?,
                    Some(KindId(1))
                ),
                Self::AspectRevision => assert!(read
                    .read_exact_snapshot_entity_aspect_version(
                        execution,
                        &snapshot,
                        entity,
                        &aspect_key("name")
                    )?
                    .unwrap()
                    .is_some()),
            }
            Ok(())
        })();
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        answer
    }
}
