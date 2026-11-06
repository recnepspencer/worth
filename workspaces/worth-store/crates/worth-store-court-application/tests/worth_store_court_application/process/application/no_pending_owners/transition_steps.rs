//! Original intent assignments enter the ordinary canonical publication path.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionTargetMeaning,
};
use worth_relational::facade::mvcc::{RelationalCallerAssociation, RelationalTransactionIntent};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::{
    ApplicationHomeTransitionIntentDenial as Denial,
    ApplicationHomeTransitionIntentOutcome as Outcome,
};

#[test]
fn a_transition_step_requires_the_original_committed_intent_and_can_be_claimed_only_once() {
    steps(false, false, false);
}

#[test]
fn positioned_transition_steps_use_the_original_world_attempts_and_inherit_target_contracts() {
    steps(true, false, false);
}

#[test]
fn an_original_transition_grant_refuses_its_successor_native_basis_before_execution() {
    steps(true, true, false);
}

pub(super) fn steps(execute: bool, check_successor_refusal: bool, refuse_compilation: bool) -> u64 {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let first = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let evaluated = Arc::new(AtomicUsize::new(0));
    if refuse_compilation {
        let zero = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(Some(0)),
            worth_execution::CancellationSource::new().token(),
            None,
        );
        let refused = native.prepare_home_program_contracts(
            &ExecutionRequest::serial(&zero),
            first.basis().relational_basis(),
            native.config().schema.registry.clone(),
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(refused, Err(worth_relational::facade::runtime::RelationalHomeProgramContractDenial::Execution(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted { admitted: 0, ancestor: 0, .. }))));
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
        assert_eq!(
            fixture
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .unwrap(),
            first
        );
    }
    let compilation_memory = worth_execution::SerialMemoryBudget::new(None);
    let compilation_request = worth_execution::SerialRequest::from_memory(
        compilation_memory.clone(),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let contracts = native
        .prepare_home_program_contracts(
            &ExecutionRequest::serial(&compilation_request),
            first.basis().relational_basis(),
            native.config().schema.registry.clone(),
            vec![
                worth_relational::facade::runtime::CustomInvariantRegistration::new(
                    super::transition_executor::TargetRule(evaluated.clone()),
                )
                .unwrap(),
            ],
            Vec::new(),
        )
        .unwrap();
    let compiled_charge = compilation_memory.charged_memory_bytes();
    assert!(compiled_charge > 0);
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            first.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            2,
            HomeProgramTransitionTargetMeaning::from_recorded(0, contracts.catalog_digest()),
            &crate::batches::transition_body_roster(&execution, 2),
            &execution,
        )
        .unwrap();
    let assignments = reserved.assignments().to_vec();
    fixture.home.stage_home_program_transition_intent(reserved,
        worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(),
        &execution,
    ).unwrap();
    assert!(matches!(
        fixture
            .home
            .admit_home_program_transition_step(0, &first, &execution),
        Err(Denial::IntentNotCommitted)
    ));
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(&execution)
            .unwrap()
        {
            Outcome::Committed => break,
            Outcome::Pending => std::thread::yield_now(),
            other => panic!("original intent must commit: {other:?}"),
        }
    }
    assert!(matches!(
        fixture
            .home
            .admit_home_program_transition_step(1, &first, &execution),
        Err(Denial::SourceMismatch)
    ));
    let mut expected = first;
    for step in 0..if execute { 2 } else { 1 } {
        let executions_before = evaluated.load(Ordering::SeqCst);
        let mut admitted = fixture
            .home
            .admit_home_program_transition_step(step, &expected, &execution)
            .unwrap();
        assert_eq!(admitted.original(), &assignments[step as usize]);
        assert!(matches!(
            fixture
                .home
                .admit_home_program_transition_step(step, &expected, &execution),
            Err(Denial::StepAlreadyClaimed)
        ));
        if !execute {
            break;
        }
        let cancellation = RuntimeWorldCancellationSource::new();
        let change = RelationalTransactionIntent::ordinary();
        let reservation = admitted.take_reservation().unwrap();
        let pending = fixture
            .home
            .world()
            .publication_port()
            .prepare_home_transition_step(
                &execution,
                expected.clone(),
                change.clone(),
                admitted.plan(),
                admitted.original(),
                reservation,
                admitted.admission(),
                &cancellation.token(),
                None,
            )
            .unwrap();
        let mut opening = pending.advance(&execution);
        let opened = loop {
            execution.consult().unwrap();
            match opening {
                WorldOpenAttemptOutcome::Opened(value) => break value,
                WorldOpenAttemptOutcome::Pending(value) => opening = value.advance(&execution),
                other => panic!("the assigned original open must commit: {other:?}"),
            }
        };
        assert_eq!(
            opened.attempt_identity().ordinal(),
            assignments[step as usize].attempt().ordinal()
        );
        let original_association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let native = fixture.home.relational().runtime();
        assert!(matches!(native.begin_branch_transaction(
            expected.basis().relational_basis(), RelationalTransactionIntent::ordinary()),
            Err(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::HomeProgramTransition(
                worth_relational::facade::durability::RelationalHomeProgramWriteDenial::TransitionRequired))));
        assert_eq!(evaluated.load(Ordering::SeqCst), executions_before);
        let mut transaction = if step == 0 {
            native
                .begin_home_program_step(
                    &execution,
                    expected.basis().relational_basis(),
                    worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
                        contracts: &contracts,
                        transition: None,
                    },
                    admitted.plan(),
                    worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                        admitted.plan().identity(),
                        admitted.plan().target_meaning().step(),
                    ),
                    admitted.original(),
                    &original_association,
                    admitted.admission(),
                )
                .unwrap()
        } else {
            native
                .begin_home_program_step(
                    &execution, expected.basis().relational_basis(),
                    worth_relational::facade::runtime::RelationalHomeProgramStepContracts::AfterTarget,
                    admitted.plan(),
                    worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(admitted.plan().identity(), step),
                    admitted.original(), &original_association, admitted.admission(),
                )
                .unwrap()
        };
        transaction
            .push_batch(
                crate::batches::create(step as usize),
                execution.allocation_policy(),
            )
            .unwrap();
        let proposal = transaction.validate(&execution, native).unwrap();
        let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        assert!(evaluated.load(Ordering::SeqCst) > executions_before,
            "each original step executes the actual target inventory, including inherited executors");
        assert_eq!(
            performed.attempt_identity().ordinal(),
            assignments[step as usize].attempt().ordinal()
        );
        let mut consumed = performed.consume();
        expected = consumed.take_successor_observation().unwrap();
        drop(consumed);
        if check_successor_refusal && step == 0 {
            let executions_before = evaluated.load(Ordering::SeqCst);
            let refused = fixture.home.relational().runtime().begin_home_program_step(
                &execution,
                expected.basis().relational_basis(),
                worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: None,
                },
                admitted.plan(),
                worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                    admitted.plan().identity(),
                    admitted.plan().target_meaning().step(),
                ),
                admitted.original(),
                &original_association,
                admitted.admission(),
            );
            assert!(matches!(refused, Err(
                worth_relational::facade::runtime::RelationalHomeProgramContractDenial::SourceBasisMismatch
            )));
            assert_eq!(evaluated.load(Ordering::SeqCst), executions_before);
            assert_eq!(
                fixture
                    .home
                    .world()
                    .observation_port()
                    .observe_root_product_branch()
                    .unwrap(),
                expected
            );
        }
    }
    let schema_owner = if execute {
        use worth_relational::facade::inspection::RelationalAuthoritativeAllocationKind as Kind;
        let native = fixture.home.relational().runtime();
        let inventory = native
            .inspect_owner_allocation_ledger(std::slice::from_ref(&native.main_branch_identity()))
            .unwrap();
        let owners = inventory
            .authoritative_allocations()
            .iter()
            .filter(|allocation| allocation.locator().kind() == Kind::RootSchemaAuthority)
            .map(|allocation| allocation.locator().owner_id())
            .collect::<Vec<_>>();
        assert_eq!(owners.len(), 1);
        owners[0]
    } else {
        0
    };
    // No final binding is claimed by this lower-owner court. Abandoning retains
    // the actual durable intent and effects for the eventual open continuation.
    drop(contracts);
    if execute {
        assert_eq!(compilation_memory.charged_memory_bytes(), compiled_charge);
    }
    drop(fixture);
    if execute {
        assert_eq!(compilation_memory.charged_memory_bytes(), compiled_charge);
    }
    drop(expected);
    assert_eq!(compilation_memory.charged_memory_bytes(), 0);
    schema_owner
}
