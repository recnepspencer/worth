//! Original pending, release and post-selection completion use the same World seat.
use super::*;
#[test]
fn pending_creation_keeps_its_original_seat_with_zero_owner_effects_and_resumes_after_caller_drop()
{
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    f.recorder.hold_publication(true);
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) =
        create(&f, &request, &parent, "pending", true, true)
    else {
        panic!("the actual medium holds the original before ACK")
    };
    assert_eq!(effects.owner_effect_count(), 0);
    assert!(effects
        .next_actions()
        .contains(&ProductUnpublishedNextAction::ContinueOriginalPublication));
    let original = effects.attempt_identity().clone();
    let handle = effects.recovery_handle();
    assert!(matches!(
        f.world.branch_port().resume_creation(
            &request,
            &handle,
            &RuntimeWorldCancellationSource::new().token()
        ),
        Err(RuntimeWorldRecoveryDenial::CallerCapabilityLive)
    ));
    drop(effects);
    f.recorder.hold_publication(false);
    let outcome = f
        .world
        .branch_port()
        .resume_creation(
            &request,
            &handle,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("same creation: {outcome:?}")
    };
    assert_eq!(child.reference_generation().get(), 0);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    let actual = f
        .signal
        .source_port()
        .inspect_associated_advances(&request, |effect, _| {
            std::ops::ControlFlow::Break(
                effect
                    .association()
                    .expect("actual fork association")
                    .attempt()
                    .ordinal(),
            )
        })
        .unwrap();
    assert_eq!(actual, std::ops::ControlFlow::Break(original.ordinal()));
    drop((child, parent));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request).len(), 2);
    f.finish(&request);
}

#[derive(Debug)]
struct InterruptedCompletion(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl RuntimeWorldBranchCreationCompletion for InterruptedCompletion {
    fn completed(&mut self, _: &ProductBranchObservation, _: &ProductBranchObservation) {
        if self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
            panic!("interrupt after real destination insertion");
        }
    }
}
#[test]
fn creation_completion_unwind_preserves_the_installed_original_without_repeating_owner_forks() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let intent = ProductBranchCreationIntent::from_source(
        "interrupted",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId("interrupted-native".into()),
            },
            SignalBranchCreationPlan::ForkExact {
                target: worth_signal::facade::branch::validate_signal_branch_name(
                    "interrupted-signal",
                )
                .unwrap(),
            },
        ),
    )
    .unwrap();
    let original = RuntimeWorldHomeBranchCreationRequest::new(
        parent.clone(),
        f.recorder.program_source(&parent),
        intent,
    )
    .with_completion(Box::new(InterruptedCompletion(calls.clone())));
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        f.world.branch_port().create_home_product_branch(
            &request,
            original,
            &RuntimeWorldCancellationSource::new().token(),
        )
    }));
    assert!(interrupted.is_err());
    assert_eq!(
        descriptions(&f, &request).len(),
        2,
        "selection installed the actual destination before interruption"
    );
    let native_before = f
        .native
        .owner_component_services()
        .fork_port()
        .observe_original_frontiers(&request)
        .unwrap();
    let signal_before = f.signal.published_advance_count().unwrap();
    let page = f
        .world
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let handle = page.rows()[0].handle().clone();
    let outcome = f
        .world
        .branch_port()
        .resume_creation(
            &request,
            &handle,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("original installed destination: {outcome:?}")
    };
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(
        f.native
            .owner_component_services()
            .fork_port()
            .observe_original_frontiers(&request)
            .unwrap(),
        native_before
    );
    assert_eq!(f.signal.published_advance_count().unwrap(), signal_before);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    drop((child, parent, page));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request).len(), 2);
    f.finish(&request);
}

#[test]
fn cancelling_an_original_after_intent_ack_records_release_without_executing_either_owner() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let native_before = f
        .native
        .owner_component_services()
        .fork_port()
        .observe_original_frontiers(&request)
        .unwrap();
    let signal_before = f.signal.published_advance_count().unwrap();
    f.recorder.hold_publication(true);
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) =
        create(&f, &request, &parent, "cancelled", true, true)
    else {
        panic!("actual intent remains pending")
    };
    let handle = effects.recovery_handle();
    let attempt = effects.attempt_identity().clone();
    drop(effects);
    let cancellation = RuntimeWorldCancellationSource::new();
    cancellation.cancel();
    f.recorder.hold_publication(false);
    let outcome = f
        .world
        .branch_port()
        .resume_creation(&request, &handle, &cancellation.token())
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::ReleasedOriginal(receipt) = outcome else {
        panic!("acknowledged original has a durable release: {outcome:?}")
    };
    assert_eq!(receipt.association().attempt().ordinal(), attempt.ordinal());
    assert_eq!(
        f.native
            .owner_component_services()
            .fork_port()
            .observe_original_frontiers(&request)
            .unwrap(),
        native_before
    );
    assert_eq!(f.signal.published_advance_count().unwrap(), signal_before);
    assert_eq!(descriptions(&f, &request).len(), 1);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    drop((parent, receipt));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request).len(), 1);
    f.finish(&request);
}

#[test]
fn pending_intent_that_proves_no_effect_completes_without_a_missing_recovery_row_or_reopen_claim() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = f.recorder.heads();
    let native_before = f
        .native
        .owner_component_services()
        .fork_port()
        .observe_original_frontiers(&request)
        .unwrap();
    let signal_before = f.signal.published_advance_count().unwrap();
    f.recorder.hold_publication(true);
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) =
        create(&f, &request, &parent, "refused", true, true)
    else {
        panic!("actual intent remains pending")
    };
    let attempt = effects.attempt_identity().clone();
    let handle = effects.recovery_handle();
    drop(effects);
    f.recorder.hold_publication(false);
    f.recorder.refuse_next_publication();
    let outcome = f
        .world
        .branch_port()
        .resume_creation(
            &request,
            &handle,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::NoEffect(receipt) = outcome else {
        panic!("the actual medium proves no World original effect: {outcome:?}")
    };
    assert_eq!(receipt.association().attempt().ordinal(), attempt.ordinal());
    assert!(matches!(
        receipt.cause(),
        WorldDurableMediumDenial::Unavailable
    ));
    assert_eq!(f.recorder.heads(), before);
    assert_eq!(
        f.native
            .owner_component_services()
            .fork_port()
            .observe_original_frontiers(&request)
            .unwrap(),
        native_before
    );
    assert_eq!(f.signal.published_advance_count().unwrap(), signal_before);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    assert_eq!(descriptions(&f, &request).len(), 1);
    drop((receipt, parent));
    f.finish(&request);
}
