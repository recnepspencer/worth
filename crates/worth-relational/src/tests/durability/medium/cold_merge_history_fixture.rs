//! A nonhead source change overlaps the target after another source commit.
use std::sync::Arc;

use crate::facade::history::BranchId;
use crate::tests::support::*;

pub(super) struct Fixture {
    pub runtime: crate::runtime::RelationalRuntime,
    pub medium: Arc<OwnedTestDurableMedium>,
    pub overlap: crate::identity::data::EntityId,
    pub intermediate: crate::history::data::RelationalCommitReceipt,
    pub position: crate::publication::patch::data::PatchStreamPosition,
}

pub(super) fn branches() -> (BranchId, BranchId) {
    (
        BranchId("main".into()),
        BranchId("intermediate-source".into()),
    )
}

impl Fixture {
    pub(super) fn reopened(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        let medium = OwnedTestDurableMedium::new();
        let producer = durable_runtime_for_medium(&medium);
        let a = create_entity(execution, &producer, "merge-a-base");
        let b = create_entity(execution, &producer, "merge-b-base");
        let (target, source_branch) = branches();
        create_branch_from_main(&producer, &source_branch.0);
        let first = update_entity_on_branch(
            execution,
            &producer,
            a,
            "merge-a-source",
            source_branch.clone(),
        );
        let intermediate = first.commit.clone();
        let position = first.patch_position();
        release_test_commit_snapshot(&producer, &first);
        let head =
            update_entity_on_branch(execution, &producer, b, "merge-b-source", source_branch);
        release_test_commit_snapshot(&producer, &head);
        let target_update =
            update_entity_on_branch(execution, &producer, a, "merge-a-target", target);
        release_test_commit_snapshot(&producer, &target_update);
        capture_and_file_checkpoint(execution, &producer, &medium);
        drop(producer);

        let mut runtime = durable_runtime_for_medium(&medium);
        recover_runtime_from_medium(execution, &mut runtime, &medium);
        assert!(runtime
            .history
            .commit_artifact(intermediate.commit_id)
            .unwrap()
            .resident_envelope()
            .is_none());
        medium.start_recording_commit_reads(position);
        Self {
            runtime,
            medium,
            overlap: a,
            intermediate,
            position,
        }
    }
}
