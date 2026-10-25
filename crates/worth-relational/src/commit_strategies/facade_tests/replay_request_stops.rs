use super::native_strategy_fixtures::*;
use crate::authority::commit::pipeline::{stage_observation, CommitStage};
use crate::durability::data::RelationalRecoveryStop;
use crate::execution::RelationalExecutionDenialCause as Cause;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn verification_memory_stop_has_no_serialized_mismatch_outcome() {
    let descriptor = strategy_descriptor();
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &descriptor,
            PlanningExecutor,
        ))
        .build();
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let canonical = canonical_request();
    let draft = execution_draft(&canonical);
    let (input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(&runtime, None);
    let lowered = authority
        .lower_execution_with_input(
            &worth_execution::ExecutionRequest::serial(
                &crate::test_execution_policy::serial_request(),
            ),
            &runtime,
            &canonical,
            &draft,
            input,
        )
        .unwrap();
    let validated = authority
        .validate_lowered_plan(&execution, &runtime, lowered)
        .unwrap();
    let committed = authority
        .execute_validated_commit(&execution, &runtime, validated)
        .unwrap();
    let request = RelationalReplayRequest {
        commit_id: committed.commit.commit_id,
        branch_id: BranchId("main".into()),
        execution_mode: ReplayExecutionMode::SerialDeterministic,
        verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
    };
    const MEMORY: u64 = 64 * 1024 * 1024;
    let lease = Arc::new(
        crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(MEMORY), None),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap(),
    );
    let observed_lease = lease.clone();
    let reached = Arc::new(AtomicUsize::new(0));
    let observed_reached = reached.clone();
    let mut validation = 0;
    let mut held = None;
    let outcome = stage_observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Validation {
                validation += 1;
                if validation == 2 {
                    // Reconstruction has completed. Occupy the measured remaining
                    // room immediately before strategy re-validation allocates.
                    let worth_execution::MemoryLimitDenial::Capacity { admitted: room, .. } =
                        observed_lease.reserve_memory(MEMORY + 1).unwrap_err()
                    else {
                        panic!("the finite policy must supply a capacity refusal")
                    };
                    held = Some(observed_lease.reserve_memory(room).unwrap());
                    observed_reached.fetch_add(1, Ordering::SeqCst);
                }
            }
            std::hint::black_box(&held);
        }),
        || {
            runtime
                .replay_authority()
                .replay_commit(request.clone(), &ExecutionRequest::leased(&lease))
        },
    );
    assert_eq!(
        reached.load(Ordering::SeqCst),
        1,
        "memory pressure never reached verification"
    );
    let stop = outcome.expect_err("request memory refusal must leave no serializable outcome");
    assert!(
        matches!(
            stop,
            RelationalRecoveryStop::Request(Cause::PolicyMemoryExhausted { admitted: 0, .. })
        ),
        "{stop:?}"
    );
    let retry = runtime
        .replay_authority()
        .replay_commit(request, &execution)
        .unwrap();
    assert!(
        retry.failure.is_none(),
        "same verification succeeds without pressure: {retry:?}"
    );
}
