//! Original SDK observations certify publication admission independently.
use super::fixture::World;
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

pub(super) fn request(work: u64) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(work),
            ),
        ),
        CancellationSource::new().token(),
        None,
    )
}

fn seeded() -> World {
    let request = request(8_000_000);
    let execution = ExecutionRequest::serial(&request);
    let mut world = World::new();
    world.create(&execution, 7, "left");
    world.create(&execution, 7, "right");
    world
}

#[test]
fn kind_publication_preserves_original_stops_and_releases_scratch() {
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
    };
    // Measure the same real SDK operation with ample room. The original work
    // ceiling is tested against that observation, never a frozen invoice.
    let measured = request(8_000_000);
    let execution = ExecutionRequest::serial(&measured);
    let world = seeded();
    let (result, report) = execution
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            world.commit(&execution, world.relation_intent(0, 1, 7, "edge"))
        })
        .unwrap();
    let committed = result.unwrap();
    let required = report.charged_work();
    assert!(required > 0);
    world.release(&committed);
    drop(committed);
    drop(world);
    assert_eq!(measured.memory().unwrap().charged_memory_bytes(), 0);

    // A fresh identical state must refuse an insufficient original budget,
    // publish nothing and release all proposal scratch. A sufficient budget
    // must produce identical work, independent of its unused headroom.
    for (ceiling, succeeds) in [(required / 2, false), (required * 2, true)] {
        let world = seeded();
        let request = request(ceiling);
        let execution = ExecutionRequest::serial(&request);
        let before = world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap()
            .0;
        let (result, report) = execution
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                world.commit(&execution, world.relation_intent(0, 1, 7, "edge"))
            })
            .unwrap();
        if succeeds {
            let committed = result.unwrap();
            assert_eq!(
                report.charged_work(),
                required,
                "headroom cannot change the original work bill"
            );
            world.release(&committed);
            drop(committed);
            drop(world);
        } else {
            assert!(
                matches!(result, Err(TransactionCommitError::Execution { denial, .. })
                if denial.kind == CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted)),
                "the original work ceiling must refuse: {result:?}"
            );
            assert_eq!(
                world
                    .runtime
                    .observe_branch(&world.runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        }
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
}
