//! The gate between a settled Relational effect and the Signal advance. Every
//! denial here stops before the Signal owner is contacted and retains exactly
//! the Relational effect that was already performed.

use std::thread;

use super::*;

/// A cancellation observed after the Relational effect and before the Signal
/// advance retains only that Relational effect, and the Signal owner is never
/// contacted.
#[test]
fn cancellation_between_the_relational_effect_and_the_signal_advance_retains_only_the_relational_effect(
) {
    let (fixture, owner, expected) = setup();
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let prepared = prepare_both_owners(
        &execution,
        &fixture,
        &owner,
        &expected,
        "cancel-between-owners",
    );
    let (rehearsal, reached) = arm_rehearsal(
        owner.as_ref(),
        ExecutionRehearsalBoundary::BetweenOwnerEffects,
    );
    let source = RuntimeWorldCancellationSource::new();
    let token = source.token();
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let source_ref = &source;
        let rehearsal_ref = &rehearsal;
        let canceller = scope.spawn(move || {
            await_boundary(&reached);
            source_ref.cancel();
            rehearsal_ref.release();
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            execution,
            prepared,
            &mut context,
            &token,
            |_| Ok(()),
        );
        canceller
            .join()
            .expect("the cancelling thread completes within the rehearsal budget");
        outcome
    });

    assert_eq!(
        rehearsal.signal_advance_entries(),
        0,
        "the Signal owner was never contacted"
    );
    let record = retained(outcome);
    assert_retains_only_the_relational_effect(
        &record,
        ProductUnpublishedCause::CancellationAfterEffect,
    );
    drop(record);
}

/// A deadline that expires after the Relational effect and before the Signal
/// advance retains only that Relational effect, and the Signal owner is never
/// contacted.
#[test]
fn deadline_between_the_relational_effect_and_the_signal_advance_retains_only_the_relational_effect(
) {
    let clock = MutableClock::new(0);
    let (fixture, owner, expected) =
        setup_with_clock(RuntimeWorldClock::from_source(clock.clone()));
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = RuntimeWorldPreparationService::prepare_publication(
        owner.as_ref(),
        expected.clone(),
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
            .with_prepared_relational_candidate(
                fixture.prepare_relational_owner_candidate(&execution, "deadline-between-owners"),
            ),
        &cancellation.token(),
        Some(crate::lifecycle::RuntimeWorldInstant::from_ticks(5)),
    )
    .expect("the future deadline admits the complete attempt reservation");
    let (rehearsal, reached) = arm_rehearsal(
        owner.as_ref(),
        ExecutionRehearsalBoundary::BetweenOwnerEffects,
    );
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let clock_ref = &clock;
        let rehearsal_ref = &rehearsal;
        let expirer = scope.spawn(move || {
            await_boundary(&reached);
            clock_ref.set(5);
            rehearsal_ref.release();
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            execution,
            prepared,
            &mut context,
            &cancellation.token(),
            |_| Ok(()),
        );
        expirer
            .join()
            .expect("the deadline thread completes within the rehearsal budget");
        outcome
    });

    assert_eq!(
        rehearsal.signal_advance_entries(),
        0,
        "the Signal owner was never contacted"
    );
    let record = retained(outcome);
    assert_retains_only_the_relational_effect(
        &record,
        ProductUnpublishedCause::DeadlineAfterEffect,
    );
    drop(record);
}

/// The third arm of the same gate, proved directly on the gate itself.
///
/// This is a gate-level unit proof, not an end-to-end race: any competing
/// publication that could replace the product head also moves a component
/// basis, so a racing attempt is denied at admission long before it reaches
/// this gate. The gate reports the cause only — it carries no observed head —
/// so the winner an attempt observes is proved where a caller is actually told
/// about it, on the no-effect surface in `failures.rs`.
#[test]
fn the_pre_advance_gate_denies_a_stale_product_head_before_the_signal_advance() {
    let (fixture, owner, expected) = setup_with_relational_source();
    let competing_ready = ready_relational_competitor(
        &fixture,
        owner.as_ref(),
        &expected,
        "stale-between-owners-competitor",
    );
    let (rehearsal, _reached) =
        arm_rehearsal(owner.as_ref(), ExecutionRehearsalBoundary::SignalAdvance);
    let cancellation = RuntimeWorldCancellationSource::new();
    assert!(
        owner
            .pre_advance_signal_gate(&expected, None, &cancellation.token())
            .is_ok(),
        "the gate admits the attempt while its exact head is current"
    );

    publish_ready_competing_head(owner.as_ref(), competing_ready, &expected);
    assert!(
        matches!(
            owner.pre_advance_signal_gate(&expected, None, &cancellation.token()),
            Err((
                ProductUnpublishedCause::StaleProductHead,
                crate::publication::NoEffectCause::StaleExpectedProductHead
            ))
        ),
        "a replaced product head stops the attempt before the Signal advance"
    );
    assert_eq!(
        rehearsal.signal_advance_entries(),
        0,
        "no outcome of this gate contacts the Signal owner"
    );
}

/// Retaining a Memory publication after Native settles must release the
/// attempt's unsealed memory-original admission immediately, so a resumed
/// adoption on a budget sized for one original fits without double billing.
#[test]
fn retained_attempt_releases_unsealed_original_admission_for_adoption() {
    use crate::lifecycle::RuntimeWorldRecoveryService;
    use crate::publication::RuntimeWorldPublicationOutcome;

    let measure_budget = worth_execution::SerialMemoryBudget::new(None);
    let measure_request = worth_execution::SerialRequest::from_memory(
        measure_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let measure_exec = worth_execution::ExecutionRequest::serial(&measure_request);
    let (m_fixture, m_owner, m_expected) = setup();
    let m_prepared =
        prepare_both_owners(&measure_exec, &m_fixture, &m_owner, &m_expected, "measure");
    let m_source = RuntimeWorldCancellationSource::new();
    let mut m_ctx = ();
    let m_outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
        m_owner.as_ref(),
        measure_exec,
        m_prepared,
        &mut m_ctx,
        &m_source.token(),
        |_| Ok(()),
    );
    let m_published = m_owner.finish_publication(&measure_exec, m_outcome, &m_source.token());
    assert!(matches!(
        m_published,
        RuntimeWorldPublicationOutcome::Performed(_)
    ));
    let one_publication_bytes = measure_budget.charged_memory_bytes();
    assert!(one_publication_bytes > 0);
    drop(m_published);

    let (fixture, owner, expected) = setup();
    let setup_host = crate::test_execution_policy::serial_request();
    let setup_exec = worth_execution::ExecutionRequest::serial(&setup_host);
    let prepared = prepare_both_owners(
        &setup_exec,
        &fixture,
        &owner,
        &expected,
        "cancel-and-adopt-one-original",
    );
    let (rehearsal, reached) = arm_rehearsal(
        owner.as_ref(),
        ExecutionRehearsalBoundary::BetweenOwnerEffects,
    );
    let source = RuntimeWorldCancellationSource::new();
    let token = source.token();
    // Room for one performed publication (plus small headroom below two originals).
    let shared_budget = worth_execution::SerialMemoryBudget::new(Some(one_publication_bytes));
    let shared_request = worth_execution::SerialRequest::from_memory(
        shared_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let shared_exec = worth_execution::ExecutionRequest::serial(&shared_request);
    let mut context = ();

    let outcome = thread::scope(|scope| {
        let source_ref = &source;
        let rehearsal_ref = &rehearsal;
        let canceller = scope.spawn(move || {
            await_boundary(&reached);
            source_ref.cancel();
            rehearsal_ref.release();
        });
        let outcome = RuntimeWorldOwnerExecutionService::execute_with_signal(
            owner.as_ref(),
            shared_exec,
            prepared,
            &mut context,
            &token,
            |_| Ok(()),
        );
        canceller
            .join()
            .expect("the cancelling thread completes within the rehearsal budget");
        outcome
    });
    let retained = retained(outcome);
    assert!(
        shared_budget.charged_memory_bytes() < one_publication_bytes,
        "retaining after Native settles must release unsealed memory-original funding (remaining={}, full_pub={})",
        shared_budget.charged_memory_bytes(),
        one_publication_bytes
    );
    assert_eq!(owner.state.history.committed_memory_original_count_for_test(), 0);
    assert_eq!(owner.state.history.memory_original_capacity_for_test(), 0);

    let resume_source = RuntimeWorldCancellationSource::new();
    let adopted = RuntimeWorldRecoveryService::prepare_settled_relational_adoption(
        owner.as_ref(),
        &retained,
        &resume_source.token(),
        None,
    )
    .expect("settled relational adoption prepares cleanly");
    let resume_outcome = RuntimeWorldOwnerExecutionService::execute_without_signal(
        owner.as_ref(),
        shared_exec,
        adopted,
        &resume_source.token(),
    );
    let finished = owner.finish_publication(&shared_exec, resume_outcome, &resume_source.token());
    assert!(
        matches!(finished, RuntimeWorldPublicationOutcome::Performed(_)),
        "adoption on the same one-original budget must succeed: {finished:?}"
    );
}
