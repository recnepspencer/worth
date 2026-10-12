use super::native_strategy_fixtures::*;
use crate::authority::commit::pipeline::{stage_observation, CommitStage};
use crate::durability::data::RelationalRecoveryStop;
use crate::execution::RelationalExecutionDenialCause as Cause;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

const REFUSAL: Cause = Cause::PolicyMemoryExhausted {
    requested: 17,
    admitted: 16,
    ancestor: 0,
};
struct RefuseOnce {
    calls: Arc<AtomicUsize>,
    published: Arc<AtomicUsize>,
}
impl CommitStrategyExecutor for RefuseOnce {
    fn execute(
        &self,
        _: &worth_execution::ExecutionRequest<'_, '_>,
        request: &CanonicalStrategyCommitRequest,
        _: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure> {
        assert_eq!(
            self.published.load(Ordering::SeqCst),
            1,
            "executor must run after reconstruction's publication"
        );
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            Err(StrategyExecutorFailure::Request(REFUSAL))
        } else {
            Ok(execution_result(request))
        }
    }
}
#[test]
fn executor_memory_refusal_is_returned_by_value_without_a_verification_outcome() {
    let calls = Arc::new(AtomicUsize::new(0));
    let published = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &strategy_descriptor(),
            RefuseOnce {
                calls: calls.clone(),
                published: published.clone(),
            },
        ))
        .build();
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let canonical = canonical_request();
    let (input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(&runtime, None);
    let lowered = authority
        .lower_execution_with_input(&runtime, &canonical, &execution_draft(&canonical), input)
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
    let observed = published.clone();
    let result = stage_observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Publication {
                observed.fetch_add(1, Ordering::SeqCst);
            }
        }),
        || {
            runtime
                .replay_authority()
                .replay_commit(request.clone(), &execution)
        },
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(published.load(Ordering::SeqCst), 1);
    assert_eq!(result, Err(RelationalRecoveryStop::Request(REFUSAL)));
    // Retry is a new verification from the same retained bytes, after the one-shot refusal.
    published.store(0, Ordering::SeqCst);
    let observed = published.clone();
    let retry = stage_observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Publication {
                observed.fetch_add(1, Ordering::SeqCst);
            }
        }),
        || {
            runtime
                .replay_authority()
                .replay_commit(request, &execution)
        },
    )
    .unwrap();
    assert!(retry.failure.is_none(), "{retry:?}");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
