//! D1: a medium is owned by exactly one runtime. A fork never inherits it,
//! recovery moves it into the rebuilt runtime, and an attachment it moved out
//! of publishes nothing. (That a medium value cannot be handed to two runtimes
//! is a compile-time guarantee, proven in `tests/ui/durable_medium`.)

use crate::durability::data::RecoveryVerificationMode;
use crate::durability::medium::{
    RelationalDurableMediumBinding, RelationalDurablePublicationRefusal, RelationalDurableStepStop,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::tests::support::*;

#[test]
fn a_forked_runtime_never_inherits_the_medium() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = durable_runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &runtime, "before-fork");
    release_test_commit_snapshot(&runtime, &committed);
    let contents = attached_test_medium(&runtime);

    let forked = runtime.fork().expect("a settled runtime forks");
    assert!(attached_durable_medium(&forked).is_none());
    let speculative = create_entity_outcome(execution, &forked, "on-the-fork");
    release_test_commit_snapshot(&forked, &speculative);

    assert_eq!(contents.positions(), vec![PatchStreamPosition(1)]);
    assert_eq!(contents.offered().len(), 1, "the fork offered nothing");
}

#[test]
fn recovery_moves_the_medium_into_the_rebuilt_runtime() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let mut runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    release_test_commit_snapshot(&runtime, &first);
    let contents = attached_test_medium(&runtime);
    let plan = recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .expect("the medium's history plans");
    let replaced = runtime.runtime_instance_id();

    runtime
        .durability_recovery()
        .recover(execution, plan)
        .expect("the runtime recovers in place");

    assert_ne!(runtime.runtime_instance_id(), replaced);
    let next = create_entity_outcome(execution, &runtime, "after-recovery");
    assert_eq!(next.patch_position(), PatchStreamPosition(2));
    assert_eq!(contents.head(), PatchStreamPosition(2));
    release_test_commit_snapshot(&runtime, &next);
}

#[test]
fn an_attachment_the_medium_moved_out_of_publishes_nothing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let source = runtime_with_test_schema();
    let committed = create_entity_outcome(execution, &source, "positioned");
    release_test_commit_snapshot(&source, &committed);
    let positioned = source.history.positioned_canonical_commits_snapshot()[0].clone();
    let contents = TestDurableMedium::new();
    let binding = RelationalDurableMediumBinding::new(contents.attachment());

    assert!(binding.detach().is_some(), "the medium moves out once");
    assert!(binding.detach().is_none(), "and only once");
    let refused = binding.publish(
        &positioned,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );

    assert!(matches!(
        refused,
        Err(RelationalDurableStepStop::Refused(
            RelationalDurablePublicationRefusal::MediumMoved
        ))
    ));
    assert!(contents.offered().is_empty());
}
