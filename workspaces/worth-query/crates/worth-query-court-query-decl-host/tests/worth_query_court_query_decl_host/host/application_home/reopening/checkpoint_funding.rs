//! Actual Host upkeep refusal cannot replace business or explicit-call funding.
use super::*;
use std::num::NonZeroU64;
use worth_query_host::facade::WorthQueryApplicationHomeCheckpointLimits;

#[test]
fn automatic_checkpoint_exhaustion_keeps_business_results_and_explicit_capture_uses_its_own_allowance(
) {
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let request = fixture::request(
        fixture::location(),
        build,
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    )
    .with_checkpoint_limits(
        WorthQueryApplicationHomeCheckpointLimits::new(
            NonZeroU64::MIN,
            NonZeroU64::new(64 << 20).unwrap(),
        )
        .with_maintenance_allowance(
            NonZeroU64::MIN,
            NonZeroU64::new(256 << 20).unwrap(),
            Duration::from_secs(60),
        ),
    );
    let bootstrap = checkpoint_capture::lease(256 << 20);
    let opening = worth_execution::ExecutionRequest::leased(&bootstrap);
    let home = with_bootstrap_advancement_from_request(opening, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("funding Home", &refused));
    for value in [8, 9] {
        let (_, live) = scope();
        assert!(matches!(
            set(home.runtime(), &live, 0x46554E44 + value, value).unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert_eq!(balance(home.runtime(), &live), value);
        let observed = home.inspect_checkpoint(&opening).unwrap();
        assert!(
            observed.due,
            "one Host work unit cannot capture this actual state"
        );
        assert!(observed.pending.is_none());
    }
    let stopping = explicit_caller(0);
    assert!(home
        .checkpoint(&worth_execution::ExecutionRequest::leased(&stopping))
        .is_err());
    let caller = explicit_caller(64 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&caller);
    let selected = home.checkpoint(&execution).unwrap();
    assert_eq!(
        selected.covered().0,
        home.inspect_checkpoint(&execution).unwrap().journals[0].head
    );
    assert!(!home.inspect_checkpoint(&execution).unwrap().due);
    home.close(&execution)
        .unwrap_or_else(|refused| opening_failed("funding close", &refused));
}

fn explicit_caller(work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(256 << 20),
                Some(work),
            ),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

#[test]
fn automatic_checkpoint_progresses_after_caller_cancellation_without_replacing_the_body_value() {
    let request = fixture::request(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    )
    .with_checkpoint_limits(WorthQueryApplicationHomeCheckpointLimits::new(
        NonZeroU64::MIN,
        NonZeroU64::new(64 << 20).unwrap(),
    ));
    let bootstrap = checkpoint_capture::lease(256 << 20);
    let opening = worth_execution::ExecutionRequest::leased(&bootstrap);
    let home = with_bootstrap_advancement_from_request(opening, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("caller-stop funding Home", &refused));
    let before = home.inspect_checkpoint(&opening).unwrap();
    assert!(before.due && before.journals[0].covered == 0);
    let (cancellation, stopped) = scope();
    let result = home
        .runtime()
        .product_runtime()
        .with_request_advancement(&stopped, |_| {
            cancellation.cancel();
            "the completed business value".to_owned()
        })
        .unwrap();
    assert_eq!(result, "the completed business value");
    let after = home.inspect_checkpoint(&opening).unwrap();
    assert!(after.pending.is_none());
    assert_eq!(after.journals[0].covered, before.journals[0].head);
    assert!(!after.due);
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), 7);
    home.close(&opening)
        .unwrap_or_else(|refused| opening_failed("caller-stop funding close", &refused));
}
