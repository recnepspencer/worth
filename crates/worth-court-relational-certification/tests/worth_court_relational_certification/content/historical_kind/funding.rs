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

/// One real two-record/one-relation publication under an exact Work ceiling.
struct Trial {
    committed: bool,
    charged: u64,
    held_after_commit: u64,
}

fn trial(ceiling: u64) -> Trial {
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
    };
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
    let charged = report.charged_work();
    assert!(charged <= ceiling, "the original account never overdraws");
    let committed = match result {
        Err(TransactionCommitError::Execution { denial, .. }) => {
            assert_eq!(
                denial.kind,
                CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted),
                "{ceiling} original Work must stop at its typed cause"
            );
            assert_eq!(
                world
                    .runtime
                    .observe_branch(&world.runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before,
                "a refused publication leaves the head unchanged"
            );
            None
        }
        Err(other) => panic!("{ceiling} original Work: unexpected refusal {other:?}"),
        Ok(committed) => Some(committed),
    };
    let (committed, held_after_commit) = match committed {
        Some(committed) => {
            let held = request.memory().unwrap().charged_memory_bytes();
            world.release(&committed);
            drop(committed);
            drop(world);
            (true, held)
        }
        None => {
            drop(world);
            (false, 0)
        }
    };
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    Trial {
        committed,
        charged,
        held_after_commit,
    }
}

#[test]
fn relation_publication_preserves_original_work_stops_and_releases_owned_backing() {
    // The complete publication, including canonical artifact hashing, settles in
    // one original account. Its exact invoice comes from the funded run, so the
    // boundary is exact without pinning a frozen total.
    let funded = trial(8_000_000);
    assert!(funded.committed && funded.held_after_commit > 0);
    let required = funded.charged;
    assert!(
        required > 943,
        "historical-kind reconstruction alone already exceeded 943 original Work"
    );
    let exact = trial(required);
    assert!(exact.committed, "the exact original invoice commits");
    assert_eq!(exact.charged, required);
    assert_eq!(
        exact.held_after_commit, funded.held_after_commit,
        "the same publication retains the same owned backing"
    );
    for ceiling in [required - 1, 943, 0] {
        let stopped = trial(ceiling);
        assert!(
            !stopped.committed,
            "{ceiling} below the exact invoice refuses"
        );
    }
    assert_eq!(trial(0).charged, 0);
}
