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
fn historical_kind_reconstruction_preserves_literal_original_stops_and_releases_scratch() {
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
    };
    // SDK totals are independent of Native's proposal work account. These are
    // frozen invoices for this real two-record/one-relation transaction.
    for ceiling in [944, 1024] {
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
        if ceiling == 944 {
            assert!(
                matches!(result, Err(TransactionCommitError::Execution { denial, .. })
                if denial.kind == CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted)),
                "944 original Work must refuse: {result:?}"
            );
            assert_eq!(
                world
                    .runtime
                    .observe_branch(&world.runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
            assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
        } else {
            let committed = result.unwrap();
            assert_eq!(report.charged_work(), 945);
            assert_eq!(request.memory().unwrap().charged_memory_bytes(), 11_728);
            world.release(&committed);
        }
    }
}
