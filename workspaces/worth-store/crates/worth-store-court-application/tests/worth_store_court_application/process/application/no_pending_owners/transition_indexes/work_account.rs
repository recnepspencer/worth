//! Exact Native preparation ceilings include the original Home-step body.
use super::target;
use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling, LeaseRequest,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::{home_program_transition_body_digest, RelationalHomeProgramStepContracts},
    transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
    },
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::ApplicationHomeTransitionIntentOutcome;

const FUNDED_WORK: u64 = 8_000_000;
#[test]
fn home_step_body_and_candidate_share_the_original_work_ceiling() {
    let (completed, required) = trial(FUNDED_WORK, true);
    assert!(completed);
    let required = required.unwrap();
    assert!(required > 0 && required < FUNDED_WORK);
    for wrapped in [false, true] {
        assert!(
            trial(required, wrapped).0,
            "the exact actual SDK invoice must complete"
        );
        assert!(
            !trial(required - 1, wrapped).0,
            "one below the complete original invoice must refuse"
        );
    }
}

fn trial(ceiling: u64, wrapped: bool) -> (bool, Option<u64>) {
    let setup_request = crate::workload::serial_request();
    let execution = &ExecutionRequest::serial(&setup_request);
    let directory = tempfile::tempdir().unwrap();
    let mut fixture =
        super::super::transition_opening::open_transition(directory.path(), execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let contracts = native
        .prepare_home_program_contracts(
            execution,
            source.basis().relational_basis(),
            target::registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
    let proposed = target::proposed();
    let intent =
        RelationalTransactionIntent::ordinary().with_schema_transition(proposed.clone(), None);
    let body = target::body();
    let digest =
        home_program_transition_body_digest(execution, &intent, std::slice::from_ref(&body))
            .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([83; 32]),
            1,
            contracts.target_meaning(execution, 0).unwrap(),
            &[digest],
            execution,
        )
        .unwrap();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), execution).unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(execution)
            .unwrap()
        {
            ApplicationHomeTransitionIntentOutcome::Committed => break,
            ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
            other => panic!("real intent refused: {other:?}"),
        }
    }
    let mut grant = fixture
        .home
        .admit_home_program_transition_step(0, &source, execution)
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let reservation = grant.take_reservation().unwrap();
    let pending = fixture
        .home
        .world()
        .publication_port()
        .prepare_home_transition_step(
            execution,
            source.clone(),
            fixture.admit_initial_program_source(&source),
            intent,
            grant.plan(),
            grant.original(),
            reservation,
            grant.admission(),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let mut outcome = pending.advance(execution);
    let opened = loop {
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(execution),
            other => panic!("original World open refused: {other:?}"),
        }
    };
    let native = fixture.home.relational().runtime();
    let association =
        RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
    let mut transaction = native
        .begin_home_program_step(
            execution,
            source.basis().relational_basis(),
            RelationalHomeProgramStepContracts::Target {
                contracts: &contracts,
                transition: Some((proposed, None)),
            },
            grant.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
            grant.original(),
            &association,
            grant.admission(),
        )
        .unwrap();
    transaction
        .push_batch(body, execution.allocation_policy())
        .unwrap();

    let before = native
        .observe_branch(&native.main_branch_identity())
        .unwrap()
        .0;
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(ceiling)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let request = ExecutionRequest::leased(&lease);
    let prepare = || {
        let proposal = transaction
            .validate(&request, native)?
            .bind_caller_association(association)
            .unwrap();
        native.prepare_validated_proposal(&request, proposal)
    };
    let (outcome, charged) = if wrapped {
        let mut outcome = None;
        let report = request.run(ExecutionWorkCeiling::new(None), |_| {
            outcome = Some(prepare())
        });
        let outcome = outcome.expect("the positive admitted original request entered preparation");
        let charged = match (&outcome, report) {
            (Ok(_), Ok(((), report))) => Some(report.charged_work()),
            (Err(_), _) => None,
            (Ok(_), Err(cause)) => {
                panic!("completed preparation lost its original report: {cause:?}")
            }
        };
        (outcome, charged)
    } else {
        (prepare(), None)
    };
    let completed = match outcome {
        Ok(candidate) => {
            native
                .preparation_port()
                .discard_prepared_candidate(candidate)
                .unwrap();
            true
        }
        Err(TransactionCommitError::Execution { denial, .. }) => {
            assert_eq!(
                denial.kind,
                CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted)
            );
            false
        }
        Err(cause) => panic!("the original Work ceiling is the only refusal: {cause:?}"),
    };
    assert_eq!(
        native
            .observe_branch(&native.main_branch_identity())
            .unwrap()
            .0,
        before,
        "preparation and original Work refusal publish no Native successor"
    );
    // This court deliberately stops at preparation and abandons its actual
    // Home/open owners. It makes no completed-close or recovered-fate claim.
    drop((opened, grant, contracts, source));
    drop(fixture);
    (completed, charged)
}
