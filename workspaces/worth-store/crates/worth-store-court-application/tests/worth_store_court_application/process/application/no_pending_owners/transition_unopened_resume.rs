//! A protected original intent resumes its absent open without repeating an effect.
use super::transition_open_original as original;
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::HomeProgramTransitionPlanDigest;
use worth_relational::facade::mvcc::{RelationalCallerAssociation, RelationalTransactionIntent};
use worth_runtime_world::facade::{
    NoEffectCause, RuntimeWorldCancellationSource, WorldHomeTransitionUnopenedDenial as Denial,
    WorldOpenAttemptOutcome, WorldOpenAttemptRefusal,
};

#[test]
fn an_intent_only_restart_starts_its_original_assignment_once_and_keeps_refused_execution_for_retry(
) {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let evaluated = Arc::new(AtomicUsize::new(0));
    let assigned = {
        let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        let source = original::commit_intent(&mut fixture, &contracts, &execution);
        let assigned = fixture
            .home
            .home_program_transition_intent()
            .unwrap()
            .assignments()[0]
            .attempt()
            .ordinal();
        assert_eq!(source.reference_generation().get(), 0);
        drop((source, contracts, fixture));
        assigned
    };
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    let association = {
        let fixture = original::reopen(directory.path(), &execution);
        let source = fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        let target = contracts.target_meaning(&execution, 0).unwrap();
        let mut grant = fixture
            .home
            .admit_unopened_home_program_transition_step(
                0,
                &source,
                HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
                1,
                target,
                &crate::batches::transition_body_roster(&execution, 1),
                &execution,
            )
            .unwrap();
        assert_eq!(grant.original().attempt().ordinal(), assigned);
        assert!(matches!(
            fixture.home.admit_unopened_home_program_transition_step(
                0,
                &source,
                HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
                1,
                target,
                &crate::batches::transition_body_roster(&execution, 1),
                &execution,
            ),
            Err(worth_store_application::ApplicationHomeTransitionIntentDenial::StepAlreadyClaimed)
        ));
        let stopped = RuntimeWorldCancellationSource::new();
        stopped.cancel();
        // The cheap World stop must win over a real reconstruction budget
        // refusal. This request cannot allocate a protected World image.
        let empty_budget = worth_execution::SerialMemoryBudget::new(Some(0));
        let empty_request = worth_execution::SerialRequest::from_memory(
            empty_budget.clone(),
            worth_execution::CancellationSource::new().token(),
            None,
        );
        let empty_execution = ExecutionRequest::serial(&empty_request);
        let permit = grant.take_execution().unwrap();
        assert!(grant.take_execution().is_none());
        let (permit, cause) = fixture
            .home
            .world()
            .publication_port()
            .prepare_readmitted_home_transition_step(
                &empty_execution,
                source.clone(),
                RelationalTransactionIntent::ordinary(),
                grant.plan(),
                0,
                grant.original(),
                grant.admission(),
                permit,
                &stopped.token(),
                None,
            )
            .unwrap_err();
        match cause {
            Denial::Open(WorldOpenAttemptRefusal::Reservation(refusal)) => {
                assert_eq!(refusal.cause(), NoEffectCause::CancelledBeforeEffect);
            }
            other => panic!("cancelled before reconstruction: {other:?}"),
        }
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
        let cancellation = RuntimeWorldCancellationSource::new();
        let (permit, cause) = fixture
            .home
            .world()
            .publication_port()
            .prepare_readmitted_home_transition_step(
                &empty_execution,
                source.clone(),
                RelationalTransactionIntent::ordinary(),
                grant.plan(),
                0,
                grant.original(),
                grant.admission(),
                permit,
                &cancellation.token(),
                Some(worth_runtime_world::facade::RuntimeWorldInstant::from_ticks(0)),
            )
            .unwrap_err();
        match cause {
            Denial::Open(WorldOpenAttemptRefusal::Reservation(refusal)) => {
                assert_eq!(refusal.cause(), NoEffectCause::DeadlineBeforeEffect);
            }
            other => panic!("expired before reconstruction: {other:?}"),
        }
        assert_eq!(empty_budget.charged_memory_bytes(), 0);
        let pending = fixture
            .home
            .world()
            .publication_port()
            .prepare_readmitted_home_transition_step(
                &execution,
                source.clone(),
                RelationalTransactionIntent::ordinary(),
                grant.plan(),
                0,
                grant.original(),
                grant.admission(),
                permit,
                &cancellation.token(),
                None,
            )
            .unwrap();
        let mut outcome = pending.advance(&execution);
        let opened = loop {
            execution.consult().unwrap();
            match outcome {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(&execution),
                other => panic!("same original assigned open must commit: {other:?}"),
            }
        };
        assert_eq!(opened.attempt_identity().ordinal(), assigned);
        let association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let native = fixture.home.relational().runtime();
        let mut transaction = native
            .begin_home_program_step(
                &execution,
                source.basis().relational_basis(),
                worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: None,
                },
                grant.plan(),
                worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                    grant.plan().identity(),
                    grant.plan().target_meaning().step(),
                ),
                grant.original(),
                &association,
                grant.admission(),
            )
            .unwrap();
        transaction
            .push_batch(crate::batches::create(0), execution.allocation_policy())
            .unwrap();
        let proposal = transaction.validate(&execution, native).unwrap();
        let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        assert_eq!(performed.attempt_identity().ordinal(), assigned);
        drop(performed.consume());
        assert!(evaluated.load(Ordering::SeqCst) > 0);
        drop((source, grant, contracts));
        close(fixture.home, &execution);
        association
    };
    let fixture = original::reopen(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(source.reference_generation().get(), 1);
    assert_eq!(source.publication_attempt().unwrap().ordinal(), assigned);
    let effects = fixture
        .home
        .relational()
        .runtime()
        .durability_authority()
        .discover_associated_effects(&execution, &association)
        .unwrap();
    assert_eq!(
        effects.len(),
        1,
        "the assigned Native effect is never repeated"
    );
    assert_eq!(
        fixture
            .home
            .world()
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    drop((effects, source));
    close(fixture.home, &execution);
}

#[test]
fn an_unopened_execution_witness_cannot_restart_a_durable_original_open() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let evaluated = Arc::new(AtomicUsize::new(0));
    let original_ids = {
        let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        let (grant, opened) = original::original_open(&mut fixture, &contracts, &execution);
        let ids = (
            opened.attempt_identity().ordinal(),
            opened.reserved_commit_identity().ordinal(),
        );
        drop(opened.retain());
        drop((grant, contracts, fixture));
        ids
    };
    let fixture = original::reopen(directory.path(), &execution);
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let contracts = original::contracts(&fixture, &execution, evaluated.clone());
    let target = contracts.target_meaning(&execution, 0).unwrap();
    let mut grant = fixture
        .home
        .admit_unopened_home_program_transition_step(
            0,
            &source,
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            1,
            target,
            &crate::batches::transition_body_roster(&execution, 1),
            &execution,
        )
        .unwrap();
    let permit = grant.take_execution().unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let (_permit, cause) = fixture
        .home
        .world()
        .publication_port()
        .prepare_readmitted_home_transition_step(
            &execution,
            source.clone(),
            RelationalTransactionIntent::ordinary(),
            grant.plan(),
            0,
            grant.original(),
            grant.admission(),
            permit,
            &cancellation.token(),
            None,
        )
        .unwrap_err();
    assert!(matches!(cause, Denial::OriginalOpenAlreadyDurable));
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let claim = fixture
        .home
        .world()
        .recovery_port()
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(claim.attempt_identity().ordinal(), original_ids.0);
    assert_eq!(claim.reserved_commit_identity().ordinal(), original_ids.1);
    assert!(claim.native_receipt().is_none());
    let recovered = fixture
        .home
        .admit_readmitted_home_program_transition_step(
            0,
            &claim,
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            1,
            target,
            &crate::batches::transition_body_roster(&execution, 1),
            &execution,
        )
        .unwrap();
    assert_eq!(recovered.original().attempt().ordinal(), original_ids.0);
    assert_eq!(
        fixture
            .home
            .world()
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        1
    );
    drop((claim, recovered, grant, source, contracts, page));
    close(fixture.home, &execution);
}
