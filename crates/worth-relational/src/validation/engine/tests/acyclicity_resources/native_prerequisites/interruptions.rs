//! Stops at the real native stage cannot enter the installed custom hook.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;
use crate::validation::execution::InvariantStage;

fn assert_stop(error: TransactionCommitError, cause: Cause) {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("original request stop required: {error:?}");
    };
    assert_eq!(denial.kind, CommitExecutionDenialKind::Cause(cause));
}

#[test]
fn the_checked_native_worker_exhausts_before_custom_preparation_and_the_funded_twin_completes() {
    let (fixture, hooks) = fixture(32);
    let before = fixture.head();
    let plan = fixture.plan(3);
    let request = || {
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &fixture.view(),
            fixture.observation(),
            fixture.version(),
            Some(&plan),
            None,
        )
    };
    let funded = paid_dag_request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let view = fixture.view();
    let (stage, charges) = crate::execution::request_work::account_observation::observe(|| {
        crate::validation::engine::stage_execution::execute_stage(
            &view,
            ExecutionRequest::leased(&funded),
            &request(),
            Some(&funded),
            InvariantStage::Native,
        )
    });
    let stage = stage.unwrap().unwrap();
    assert!(stage
        .envelopes
        .iter()
        .flat_map(|envelope| &envelope.results)
        .all(|worker| matches!(
            worker.result.verdict,
            crate::validation::data::execution::InvariantVerdict::Pass
        )));
    assert_eq!(
        charges.len(),
        3,
        "planning, packet preparation and actual native worker"
    );
    assert!(charges[2] > 1, "native traversal performs checked work");
    let native_work: u64 = charges.iter().sum();
    drop(stage);
    let constrained = paid_dag_request(
        8 * 1024 * 1024,
        native_work - 1,
        CancellationToken::new(),
        None,
    );
    let (failure, stopped_charges) =
        crate::execution::request_work::account_observation::observe(|| {
            InvariantEngine::from_view(&fixture.view())
                .execute(&ExecutionRequest::leased(&constrained), request())
        });
    assert_stop(failure.unwrap_err(), Cause::WorkExhausted);
    assert_eq!(
        stopped_charges.len(),
        3,
        "the stop reaches the native worker, before reduction/custom"
    );
    assert_eq!(&stopped_charges[..2], &charges[..2]);
    assert!(stopped_charges[2] > 0);
    assert_eq!(hooks.observed(), (0, 0));
    assert_eq!(constrained.charged_memory_bytes(), 0);
    let result = InvariantEngine::from_view(&fixture.view())
        .execute(&ExecutionRequest::leased(&funded), request())
        .unwrap();
    assert_eq!(result.results().len(), 2);
    assert_eq!(hooks.observed(), (1, 1));
    assert_eq!(fixture.head(), before);
    assert_eq!(funded.charged_memory_bytes(), 0);
}
