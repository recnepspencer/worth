//! Publication and recovery keep the original medium's exact acknowledged stream.
use super::*;
use worth_relational::facade::publication::PatchStreamPosition as Position;

fn held_positions(medium: &TestDurableMedium) -> Vec<Position> {
    medium
        .committed_artifacts()
        .unwrap()
        .iter()
        .map(|artifact| artifact.position())
        .collect()
}

#[test]
fn every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    for name in ["first", "second", "third"] {
        let committed = create_entity_outcome(request, &runtime, name);
        let held = medium
            .artifact_bytes(committed.patch_position())
            .expect("an acknowledged commit is already durable");
        let readmitted =
            Artifact::readmit(&held, request).expect("the medium holds Relational's own artifact");
        assert_eq!(readmitted.commit_id(), committed.commit.commit_id);
        assert_eq!(readmitted.position(), committed.patch_position());
        release_test_commit_snapshot(&runtime, &committed);
    }
    assert_eq!(
        held_positions(&medium),
        vec![Position(1), Position(2), Position(3)]
    );
}

#[test]
fn a_memory_only_runtime_publishes_without_a_medium() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let committed = create_entity_outcome(request, &runtime, "memory-only");
    assert!(!runtime.durability().has_bound_durable_medium());
    assert_eq!(runtime.durability().durable_commit_count(), 1);
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn a_forked_runtime_never_inherits_the_medium() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    assert!(runtime.durability().has_bound_durable_medium());
    let committed = create_entity_outcome(request, &runtime, "before-fork");
    release_test_commit_snapshot(&runtime, &committed);
    let forked = runtime.fork().expect("a settled runtime forks");
    assert!(!forked.durability().has_bound_durable_medium());
    let speculative = create_entity_outcome(request, &forked, "on-the-fork");
    release_test_commit_snapshot(&forked, &speculative);
    assert_eq!(held_positions(&medium), vec![Position(1)]);
    assert_eq!(medium.offered().len(), 1, "the fork offered nothing");
}

#[test]
fn recovery_moves_the_medium_into_the_rebuilt_runtime() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let first = create_entity_outcome(request, &runtime, "first");
    release_test_commit_snapshot(&runtime, &first);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let replaced = runtime.runtime_instance_id();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .expect("the runtime recovers in place");
    assert_ne!(runtime.runtime_instance_id(), replaced);
    let next = create_entity_outcome(request, &runtime, "after-recovery");
    assert_eq!(next.patch_position(), Position(2));
    assert_eq!(held_positions(&medium).last(), Some(&Position(2)));
    release_test_commit_snapshot(&runtime, &next);
}
