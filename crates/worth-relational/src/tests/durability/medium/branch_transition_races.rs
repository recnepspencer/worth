//! D8 publishers obey branch-before-reservation ordering in both runtime modes.
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::slow_publication::{PublicationGate, SlowFirstPublication};
use crate::branch::{PositionedBranchTransition, RelationalBranchTransitionDurableStep};
use crate::facade::history::BranchId;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::runtime::RelationalRuntime;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

fn runtime() -> (
    RelationalRuntime,
    Arc<TestDurableMedium>,
    Arc<PublicationGate>,
) {
    let contents = TestDurableMedium::new();
    let gate = Arc::new(PublicationGate::default());
    let runtime = RelationalRuntimeApi::builder()
        .profile(crate::config::data::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .build_durable(Box::new(SlowFirstPublication {
            contents: contents.attachment(),
            gate: Arc::clone(&gate),
        }));
    let request = crate::test_execution_policy::serial_request();
    create_entity(&ExecutionRequest::serial(&request), &runtime, "source");
    create_branch_from_main(&runtime, "other");
    (runtime, contents, gate)
}

fn await_condition(condition: impl Fn() -> bool, gate: &PublicationGate) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !condition() && Instant::now() < deadline {
        std::thread::yield_now();
    }
    let reached = condition();
    gate.open(); // Always release the parked publisher, including on failure.
    assert!(
        reached,
        "the competing publisher did not reach its synchronization boundary"
    );
}

fn assert_streams(
    runtime: &RelationalRuntime,
    contents: &TestDurableMedium,
    commits: u64,
    transitions: u64,
) {
    use crate::branch::RelationalBranchTransitionSequence as Sequence;
    use crate::publication::patch::data::PatchStreamPosition as Position;
    assert_eq!(
        contents.positions(),
        (1..=commits).map(Position).collect::<Vec<_>>()
    );
    assert_eq!(
        contents.transition_sequences(),
        (1..=transitions).map(Sequence).collect::<Vec<_>>()
    );
    assert_eq!(runtime.patch_position_reservation_counters().deferrals, 0);
}

#[test]
fn durable_fork_and_source_commit_complete_without_deferral() {
    let (runtime, contents, gate) = runtime();
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("concurrent"), execution.allocation_policy())
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let cell = candidate.publication_cell_for_test();
    let waits = cell.coordination().wait_count();
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    gate.arm();
    std::thread::scope(|scope| {
        let fork =
            scope.spawn(|| runtime.fork_branch(execution, BranchId("target".into()), source));
        gate.await_publication();
        let commit = scope.spawn(|| {
            runtime
                .publication_port()
                .compare_and_publish(execution, candidate)
        });
        await_condition(|| cell.coordination().wait_count() > waits, &gate);
        fork.join().unwrap().unwrap();
        let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
            commit.join().unwrap()
        else {
            panic!("the source commit must perform without a retry");
        };
        runtime.settle_performed_publication(performed).unwrap();
    });
    assert_streams(&runtime, &contents, 2, 2);
}

#[test]
fn durable_fork_and_source_archive_complete_in_lock_order() {
    let (runtime, contents, gate) = runtime();
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let main = runtime.branch_identity(&BranchId("main".into())).unwrap();
    let cell = runtime
        .history
        .branch_cell(main.branch_id())
        .unwrap()
        .publication_cell();
    let waits = cell.coordination().wait_count();
    let (_, source) = runtime.observe_fork_source(main.branch_id()).unwrap();
    gate.arm();
    std::thread::scope(|scope| {
        let fork =
            scope.spawn(|| runtime.fork_branch(execution, BranchId("target".into()), source));
        gate.await_publication();
        let archive = scope.spawn(|| {
            runtime
                .owner_component_services()
                .lifecycle_port()
                .archive_branch(execution, &main)
        });
        await_condition(|| cell.coordination().wait_count() > waits, &gate);
        fork.join().unwrap().unwrap();
        archive.join().unwrap().unwrap();
    });
    assert_streams(&runtime, &contents, 1, 3);
}

#[test]
fn durable_fork_waits_for_another_branch_commit() {
    let (runtime, contents, gate) = runtime();
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    let contacts = runtime.patch_position_reservation_counters().contacts;
    gate.arm();
    std::thread::scope(|scope| {
        let commit = scope.spawn(|| {
            create_entity_outcome_on_branch(execution, &runtime, "other", BranchId("other".into()))
        });
        gate.await_publication();
        let fork =
            scope.spawn(|| runtime.fork_branch(execution, BranchId("target".into()), source));
        await_condition(
            || runtime.patch_position_reservation_counters().contacts == contacts + 2,
            &gate,
        );
        commit.join().unwrap();
        fork.join().unwrap().unwrap();
    });
    assert_streams(&runtime, &contents, 2, 2);
    assert_eq!(
        contents.offered_transitions().last().unwrap().follows().0,
        2
    );
}

struct ParkedMemoryTransition(Arc<PublicationGate>);
impl RelationalBranchTransitionDurableStep for ParkedMemoryTransition {
    type Stop = std::convert::Infallible;
    fn take(&self, _: &PositionedBranchTransition) -> Result<(), Self::Stop> {
        self.0.pass();
        Ok(())
    }
}

#[test]
fn memory_commit_waits_behind_a_transition_without_consuming_its_candidate() {
    let runtime = runtime_with_test_schema();
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    create_entity(execution, &runtime, "source");
    create_branch_from_main(&runtime, "other");
    let gate = Arc::new(PublicationGate::default());
    gate.arm();
    let step = ParkedMemoryTransition(Arc::clone(&gate));
    let contacts = runtime.patch_position_reservation_counters().contacts;
    std::thread::scope(|scope| {
        let fork = scope.spawn(|| {
            runtime
                .owner_component_services()
                .fork_port()
                .fork_branch_with(
                    execution,
                    &step,
                    BranchId("target".into()),
                    &BranchId("main".into()),
                )
        });
        gate.await_publication();
        let commit = scope.spawn(|| {
            create_entity_outcome_on_branch(execution, &runtime, "other", BranchId("other".into()))
        });
        await_condition(
            || runtime.patch_position_reservation_counters().contacts >= contacts + 2,
            &gate,
        );
        fork.join().unwrap().unwrap();
        assert_eq!(commit.join().unwrap().patch_position().0, 2);
    });
    assert_eq!(runtime.patch_position_reservation_counters().deferrals, 0);
}
