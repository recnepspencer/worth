//! Unsupported persistence refuses through the real medium and recovery doors.
use super::occurrences::{fixture, perform, prepare};
use super::*;
use std::sync::Arc;
use worth_relational::facade::{
    durability::*, mvcc::RelationalRetainedEntitySelectionDenial as Denial,
    runtime::RelationalRuntimeApi, storage::RelationalStorageUnavailableCause,
};

#[test]
fn selection_raw_replay_refuses_before_installing_its_effects() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let current = update(execution, &runtime, [(model, "5")]);
    let selected = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &source, model),
    );
    let plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut restored = fixture().build_runtime();
    let RelationalRecoveryStop::Durability(cause) = restored
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err()
    else {
        panic!("raw selection replay must return its specific unsupported profile")
    };
    assert_eq!(
        cause.class,
        RecoveryFailureClass::UnsupportedRetainedEntitySelectionReplay
    );
    assert_ne!(
        restored
            .admit_branch_basis(&restored.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        Some(selected.commit.commit_id)
    );
    for commit in [&old, &current, &selected] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[test]
fn attached_medium_refuses_selection_before_any_medium_access() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = fixture().build_runtime();
    let old = create(execution, &runtime, "model", "3");
    let model = changed_entities(&old)[0];
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let durable = RelationalRuntimeApi::builder()
        .schema_registry(fixture().build_registry())
        .build_durable(Box::new(UnusedMedium))
        .unwrap();
    let transaction = begin_main_transaction(&durable);
    assert_eq!(
        durable
            .owner_component_services()
            .preparation_port()
            .admit_retained_entity_selection(transaction, &source, model)
            .unwrap_err(),
        Denial::DurabilityUnsupported
    );
    assert_eq!(
        durable
            .admit_branch_basis(&durable.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        None
    );
    let lawful = perform(
        execution,
        &runtime,
        prepare(execution, &runtime, &source, model),
    );
    assert_eq!(
        values(execution, &runtime, &lawful, [model, model]),
        ["3", "3"].map(string_aspect_value)
    );
    for commit in [&old, &lawful] {
        release_test_commit_snapshot(&runtime, commit);
    }
}

#[derive(Debug)]
struct UnusedMedium;
impl RelationalDurableColdSource for UnusedMedium {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        panic!("selection cannot acquire medium members")
    }
    fn read_member(
        &self,
        _: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        panic!("selection cannot read medium members")
    }
    fn read_commit(
        &self,
        _: worth_relational::facade::publication::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        panic!("selection cannot read medium commits")
    }
}
impl RelationalDurableMedium for UnusedMedium {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        Arc::new(UnusedMedium)
    }
    fn read_transition(
        &self,
        _: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        panic!("selection cannot read medium transitions")
    }
    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> RelationalDurablePublication {
        panic!("selection cannot publish medium commits")
    }
    fn publish_transition(
        &self,
        _: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        panic!("selection cannot publish medium transitions")
    }
}
