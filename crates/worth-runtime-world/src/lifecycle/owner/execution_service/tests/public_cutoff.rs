use super::*;
use crate::facade::{
    NoEffectCause, ProductBranchReferenceSnapshot, RuntimeWorldCancellationToken, RuntimeWorldOwner,
    RuntimeWorldPublicationOutcome,
};
use crate::lifecycle::RuntimeWorldInstant;
use std::sync::atomic::{AtomicBool, AtomicUsize};

struct PanickingClock;
impl RuntimeWorldClockSource for PanickingClock {
    fn now(&self) -> RuntimeWorldInstant { panic!("clock fails at exact-cell cutoff") }
}

struct CancellingClock(Arc<RuntimeWorldCancellationSource>);
impl RuntimeWorldClockSource for CancellingClock {
    fn now(&self) -> RuntimeWorldInstant {
        self.0.cancel();
        RuntimeWorldInstant::from_ticks(0)
    }
}

fn public_world_with_clock(
    clock: RuntimeWorldClock,
) -> (RealReferenceFixture, RuntimeWorldOwner<(), (), (), (), ()>, ProductBranchObservation) {
    let mut fixture = reference_test_fixture::real_fixture(12, 12);
    let inputs = fixture.owner_inputs(budgets(4), clock);
    let owner = RuntimeWorldOwner::builder()
        .with_execution_policy(inputs.execution.request_policy())
        .with_bridge_correspondence(inputs.bridge)
        .with_relational_services(inputs.relational)
        .with_signal_services(inputs.signal)
        .with_signal_definition_publication(inputs.signal_definition_publication)
        .with_budgets(inputs.budgets)
        .with_clock(inputs.clock)
        .build()
        .unwrap();
    let RuntimeWorldBootstrapOutcome::Performed(performed) =
        owner.lifecycle_port().bootstrap_root(fixture.bootstrap_intent()).unwrap()
    else { panic!("public bootstrap") };
    (fixture, owner, performed.product_branch().clone())
}

fn seed_public_head(
    fixture: &RealReferenceFixture,
    owner: &RuntimeWorldOwner<(), (), (), (), ()>,
    expected: ProductBranchObservation,
    label: &str,
) -> ProductBranchObservation {
    let host = crate::test_execution_policy::serial_request();
    let exec = worth_execution::ExecutionRequest::serial(&host);
    let cancel = RuntimeWorldCancellationSource::new();
    let port = owner.publication_port();
    let intent = CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
        .with_prepared_relational_candidate(fixture.prepare_relational_owner_candidate(&exec, label));
    let branch_id = expected.branch_identity().clone();
    let prepared = port.prepare_without_signal(expected, intent, &cancel.token(), None).unwrap();
    let RuntimeWorldPublicationOutcome::Performed(performed) =
        port.execute_without_signal(exec, prepared, &cancel.token())
    else { panic!("seed publication") };
    drop(performed.consume());
    owner.observation_port().observe_product_branch(&branch_id).unwrap()
}

fn history_marker(owner: &RuntimeWorldOwner<(), (), (), (), ()>) -> (usize, usize, u64, u64) {
    let snap = owner.inspection_port().history_snapshot().unwrap();
    (
        snap.installed_commits(),
        snap.reserved_commits(),
        snap.costs().metadata_promotions(),
        snap.costs().reachability_rows_installed(),
    )
}

fn assert_unchanged_public_no_effect(
    fixture: &RealReferenceFixture,
    owner: &RuntimeWorldOwner<(), (), (), (), ()>,
    seeded: &ProductBranchObservation,
    before_native: &worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    before_history: (usize, usize, u64, u64),
    before_product: &ProductBranchObservation,
    mem: &worth_execution::SerialMemoryBudget,
    outcome: RuntimeWorldPublicationOutcome,
    expected_cause: NoEffectCause,
) {
    let rt = fixture.relational_runtime();
    assert!(matches!(outcome, RuntimeWorldPublicationOutcome::NoEffect(r) if r.cause() == expected_cause));
    assert_eq!(rt.observe_branch(&rt.main_branch_identity()).unwrap().1.descriptor(), before_native);
    assert_eq!(history_marker(owner), before_history);
    assert_eq!(
        owner.observation_port().observe_product_branch(seeded.branch_identity()).unwrap().snapshot(),
        before_product.snapshot(),
    );
    assert_eq!(owner.inspection_port().recovery_snapshot().unwrap().installed(), 0);
    assert!(owner.root.recovery_handles().is_empty());
    assert_eq!(mem.charged_memory_bytes(), 0);
}

fn prepare_public_cutover(
    fixture: &RealReferenceFixture,
    owner: &RuntimeWorldOwner<(), (), (), (), ()>,
    seeded: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
    label: &str,
) -> crate::publication::PreparedCompositePublicationWithoutSignal {
    let host = crate::test_execution_policy::serial_request();
    let exec = worth_execution::ExecutionRequest::serial(&host);
    let mut intent = CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
        .with_prepared_relational_candidate(fixture.prepare_relational_owner_candidate(&exec, label));
    intent.require_current_read(seeded.clone()).unwrap();
    owner
        .publication_port()
        .prepare_without_signal(seeded.clone(), intent, token, Some(RuntimeWorldInstant::from_ticks(10)))
        .unwrap()
}

fn publish_with_clock_cutoff(
    label: &str,
    token: &RuntimeWorldCancellationToken,
    clock: RuntimeWorldClock,
) -> (
    std::thread::Result<RuntimeWorldPublicationOutcome>,
    ProductBranchReferenceSnapshot,
    ProductBranchObservation,
    RelationalAttemptProgressPosture,
) {
    let (fixture, owner, expected) = setup();
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);
    let intent = CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
        .with_prepared_relational_candidate(fixture.prepare_relational_owner_candidate(&execution, label));
    let prepared = owner
        .prepare_publication(expected.clone(), intent, token, Some(RuntimeWorldInstant::from_ticks(10)))
        .unwrap();
    let settlement = settled(execute_without_signal(execution, &owner, prepared));
    let successor = settlement.successor_basis().unwrap().clone();
    let ready = settlement
        .ready(successor)
        .unwrap()
        .capture_memory_original(&execution, &owner.state.history, &owner.state.relational_services().basis_port())
        .unwrap();
    let cell = owner.state.branches.root_cell().unwrap();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ready.publish_controlled(&cell, token, &clock, &owner.state.operation_control)
    }));
    let snap = cell.atomic_snapshot();
    let posture = owner
        .recovery_handles()
        .pop()
        .map(|handle| owner.inspect_recovery(&handle).unwrap().progress().relational_posture())
        .unwrap_or_else(|| match &outcome {
            Ok(RuntimeWorldPublicationOutcome::ProductUnpublished(r)) => r.progress().relational_posture(),
            other => panic!("expected recoverable or unpublished effect: {other:?}"),
        });
    (outcome, snap, expected, posture)
}

#[test]
fn final_clock_panic_keeps_owner_effects_without_product_movement() {
    let cancellation = RuntimeWorldCancellationSource::new();
    let (outcome, snap, expected, posture) =
        publish_with_clock_cutoff("clock-panic", &cancellation.token(), RuntimeWorldClock::from_source(PanickingClock));
    assert!(outcome.is_err());
    assert_eq!(snap, *expected.snapshot());
    assert_eq!(posture, RelationalAttemptProgressPosture::Settled);
}

#[test]
fn cancellation_during_clock_check_prevents_product_movement() {
    let source = Arc::new(RuntimeWorldCancellationSource::new());
    let (outcome, snap, expected, posture) = publish_with_clock_cutoff(
        "late-cancellation",
        &source.token(),
        RuntimeWorldClock::from_source(CancellingClock(Arc::clone(&source))),
    );
    let Ok(RuntimeWorldPublicationOutcome::ProductUnpublished(record)) = outcome else {
        panic!("cancellation before final atomic cutoff")
    };
    assert_eq!(record.cause(), ProductUnpublishedCause::CancellationAfterEffect);
    assert_eq!(snap, *expected.snapshot());
    assert_eq!(posture, RelationalAttemptProgressPosture::Settled);
}

#[test]
fn cutover_callback_world_cancellation_and_deadline_stop_before_native_effect() {
    use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
    let run_case = |clock: RuntimeWorldClock,
                    cancel: Arc<RuntimeWorldCancellationSource>,
                    label: &str,
                    expected_cause: NoEffectCause,
                    on_cutover: &dyn Fn()| {
        let (fixture, owner, expected) = public_world_with_clock(clock);
        let seeded = seed_public_head(&fixture, &owner, expected, "cutover-seed");
        let rt = fixture.relational_runtime();
        let before_native = rt.observe_branch(&rt.main_branch_identity()).unwrap().1.descriptor().clone();
        let before_history = history_marker(&owner);
        let before_product = owner.observation_port().observe_product_branch(seeded.branch_identity()).unwrap();
        let prepared = prepare_public_cutover(&fixture, &owner, &seeded, &cancel.token(), label);
        let mem = SerialMemoryBudget::from_policy(&owner.root.state.execution.request_policy());
        let req = SerialRequest::from_memory(mem.clone(), CancellationToken::new(), None);
        let mut callback_ran = false;
        let (_, outcome) = owner.publication_port().execute_without_signal_with_memory_cutover(
            ExecutionRequest::serial(&req),
            prepared,
            &cancel.token(),
            |cutover| { callback_ran = true; on_cutover(); cutover.perform(); },
        );
        assert!(callback_ran);
        assert_unchanged_public_no_effect(&fixture, &owner, &seeded, &before_native, before_history, &before_product, &mem, outcome, expected_cause);
    };

    let cancel = Arc::new(RuntimeWorldCancellationSource::new());
    run_case(RuntimeWorldClock::from_source(MutableClock::new(0)), Arc::clone(&cancel), "cutover-cancel", NoEffectCause::CancelledBeforeEffect, &|| cancel.cancel());

    let clock = MutableClock::new(0);
    run_case(RuntimeWorldClock::from_source(clock.clone()), Arc::new(RuntimeWorldCancellationSource::new()), "cutover-deadline", NoEffectCause::DeadlineBeforeEffect, &|| clock.set(10));

    struct ArmedCancellingClock { armed: Arc<AtomicBool>, fired: Arc<AtomicBool>, cancel: Arc<RuntimeWorldCancellationSource> }
    impl RuntimeWorldClockSource for ArmedCancellingClock {
        fn now(&self) -> RuntimeWorldInstant {
            if self.armed.load(Ordering::Acquire) { self.cancel.cancel(); self.fired.store(true, Ordering::Release); }
            RuntimeWorldInstant::from_ticks(0)
        }
    }
    let (armed, fired) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));
    let cancel = Arc::new(RuntimeWorldCancellationSource::new());
    run_case(
        RuntimeWorldClock::from_source(ArmedCancellingClock { armed: Arc::clone(&armed), fired: Arc::clone(&fired), cancel: Arc::clone(&cancel) }),
        Arc::clone(&cancel),
        "cutover-clock-cancel",
        NoEffectCause::CancelledBeforeEffect,
        &|| armed.store(true, Ordering::Release),
    );
    assert!(fired.load(Ordering::Acquire));
}

#[test]
fn cutover_boundary_samples_world_cancellation_and_deadline_stop_before_native_effect() {
    use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
    // Boundary sample checks across all three internal Native cutoff gates
    // (post-coordination call 2, post-reservation call 3, post-frame call 4).
    // Real-wait contention under Native's transition reservation lives in
    // worth-court-runtime-world's authorized memory_handoff::deadline court.
    struct CountdownClock { armed: Arc<AtomicBool>, seen: Arc<AtomicUsize>, target: usize, trigger_cancel: bool, cancel: Arc<RuntimeWorldCancellationSource> }
    impl RuntimeWorldClockSource for CountdownClock {
        fn now(&self) -> RuntimeWorldInstant {
            if self.armed.load(Ordering::Acquire) && self.seen.fetch_add(1, Ordering::AcqRel) + 1 >= self.target {
                if self.trigger_cancel { self.cancel.cancel(); return RuntimeWorldInstant::from_ticks(0); }
                return RuntimeWorldInstant::from_ticks(10);
            }
            RuntimeWorldInstant::from_ticks(0)
        }
    }
    for target in [2, 3, 4] {
        for trigger_cancel in [true, false] {
            let (armed, seen) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicUsize::new(0)));
            let cancel = Arc::new(RuntimeWorldCancellationSource::new());
            let (fixture, owner, expected) = public_world_with_clock(RuntimeWorldClock::from_source(CountdownClock {
                armed: Arc::clone(&armed), seen: Arc::clone(&seen), target, trigger_cancel, cancel: Arc::clone(&cancel),
            }));
            let seeded = seed_public_head(&fixture, &owner, expected, "boundary-seed");
            let rt = fixture.relational_runtime();
            let before_native = rt.observe_branch(&rt.main_branch_identity()).unwrap().1.descriptor().clone();
            let before_history = history_marker(&owner);
            let before_product = owner.observation_port().observe_product_branch(seeded.branch_identity()).unwrap();
            let prepared = prepare_public_cutover(&fixture, &owner, &seeded, &cancel.token(), "boundary-check");
            let mem = SerialMemoryBudget::from_policy(&owner.root.state.execution.request_policy());
            let req = SerialRequest::from_memory(mem.clone(), CancellationToken::new(), None);
            let (_, outcome) = owner.publication_port().execute_without_signal_with_memory_cutover(
                ExecutionRequest::serial(&req),
                prepared,
                &cancel.token(),
                |cutover| { armed.store(true, Ordering::Release); cutover.perform(); },
            );
            assert_eq!(seen.load(Ordering::Acquire), target, "must stop at boundary {target}");
            let expected_cause = if trigger_cancel { NoEffectCause::CancelledBeforeEffect } else { NoEffectCause::DeadlineBeforeEffect };
            assert_unchanged_public_no_effect(&fixture, &owner, &seeded, &before_native, before_history, &before_product, &mem, outcome, expected_cause);
        }
    }
}
