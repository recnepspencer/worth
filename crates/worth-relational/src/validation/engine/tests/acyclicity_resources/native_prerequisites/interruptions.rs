//! Stops at the real native stage cannot enter the installed custom hook.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::CommitExecutionDenialKind;
use crate::validation::engine::InvariantRuntimeView;
use crate::validation::execution::InvariantStage;
use worth_execution::CancellationSource;

fn assert_stop(error: TransactionCommitError, cause: Cause) {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("original request stop required: {error:?}");
    };
    assert_eq!(denial.kind, CommitExecutionDenialKind::Cause(cause));
}

#[test]
fn original_precancellation_prevents_both_hooks_and_an_uninterrupted_twin_completes() {
    let (fixture, hooks) = fixture(2);
    let before = fixture.head();
    for leased in [false, true] {
        let cancellation = CancellationSource::new();
        let lease =
            super::super::fixture::request(8 * 1024 * 1024, 4_000_000, cancellation.token(), None);
        let serial =
            worth_execution::SerialRequest::from_policy(lease.policy(), cancellation.token(), None);
        cancellation.cancel();
        let execution = if leased {
            ExecutionRequest::leased(&lease)
        } else {
            ExecutionRequest::serial(&serial)
        };
        let error = fixture
            .validate(&execution, fixture.chain_batch(1))
            .unwrap_err();
        assert_stop(error, Cause::Cancelled);
        assert_eq!(hooks.observed(), (0, 0));
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
        assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
    }
    let funded =
        super::super::fixture::request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let candidate = fixture
        .validate(&ExecutionRequest::leased(&funded), fixture.chain_batch(1))
        .expect("same DAG and installed hooks complete without cancellation");
    assert_eq!(hooks.observed(), (1, 1));
    drop(candidate);
    assert_eq!(fixture.head(), before);
    assert_eq!(funded.charged_memory_bytes(), 0);
}

#[test]
fn the_checked_native_worker_exhausts_before_custom_preparation_and_the_funded_twin_completes() {
    let (fixture, hooks) = fixture(32);
    let before = fixture.head();
    let plan = fixture.merged_plan(fixture.chain_batch(3));
    let request = || {
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &fixture.runtime,
            InvariantObservation::committed(fixture.runtime.storage_access().current_edition()),
            fixture.runtime.current_version_id(),
            Some(&plan),
            None,
        )
    };
    let funded =
        super::super::fixture::request(8 * 1024 * 1024, 4_000_000, CancellationToken::new(), None);
    let view = InvariantRuntimeView::from_runtime(&fixture.runtime);
    let (stage, charges) = crate::execution::work_observation::capture(|| {
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
            crate::validation::data::InvariantVerdict::Pass
        )));
    assert_eq!(
        charges.len(),
        3,
        "planning, packet preparation and actual native worker"
    );
    assert!(charges[2] > 1, "native traversal performs checked work");
    let native_work: u64 = charges.iter().sum();
    drop(stage);
    let constrained = super::super::fixture::request(
        8 * 1024 * 1024,
        native_work - 1,
        CancellationToken::new(),
        None,
    );
    let (failure, stopped_charges) = crate::execution::work_observation::capture(|| {
        InvariantEngine::new(&fixture.runtime)
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
    let result = InvariantEngine::new(&fixture.runtime)
        .execute(&ExecutionRequest::leased(&funded), request())
        .unwrap();
    assert_eq!(result.results().len(), 2);
    assert_eq!(hooks.observed(), (1, 1));
    assert_eq!(fixture.head(), before);
    assert_eq!(funded.charged_memory_bytes(), 0);
}
