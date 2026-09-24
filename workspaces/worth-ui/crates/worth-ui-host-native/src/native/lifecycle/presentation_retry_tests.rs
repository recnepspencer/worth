use super::{
    UiNativePresentationEffectPosture, UiNativePresentationRetryFinalization,
    UiNativePresentationRetryPolicy, UiNativePresentationRetryWake,
};
use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationOutcome,
    UiMountedPresentationAttemptIdentity,
};

#[test]
fn four_same_round_timeouts_charge_one_bounded_retry_attempt() {
    let mut policy = UiNativePresentationRetryPolicy::new();
    for round in 0..3 {
        observe_denials(
            &mut policy,
            attempt(),
            [UiHostSurfacePresentationDenial::ExternalTimeout; 4],
        );
        let UiNativePresentationRetryFinalization::Wake(UiNativePresentationRetryWake::Timeout(
            deadline,
        )) = policy.finalize_round(std::time::Instant::now())
        else {
            panic!("round {round} must schedule one timeout wake")
        };
        assert!(policy.consume_due_timeout(deadline));
    }
    observe_denials(
        &mut policy,
        attempt(),
        [UiHostSurfacePresentationDenial::ExternalTimeout; 4],
    );
    assert_eq!(
        policy.finalize_round(std::time::Instant::now()),
        UiNativePresentationRetryFinalization::DeadlineExpired
    );
}

#[test]
fn mixed_timeout_and_occlusion_orders_both_wait_for_visibility() {
    for denials in [
        [
            UiHostSurfacePresentationDenial::ExternalTimeout,
            UiHostSurfacePresentationDenial::SurfaceOccluded,
        ],
        [
            UiHostSurfacePresentationDenial::SurfaceOccluded,
            UiHostSurfacePresentationDenial::ExternalTimeout,
        ],
    ] {
        let mut policy = UiNativePresentationRetryPolicy::new();
        observe_denials(&mut policy, attempt(), denials);
        assert_eq!(
            policy.finalize_round(std::time::Instant::now()),
            UiNativePresentationRetryFinalization::Wake(UiNativePresentationRetryWake::Visibility)
        );
    }
}

#[test]
fn reconstruction_dominates_retry_without_leaving_a_wake() {
    for denials in [
        [
            UiHostSurfacePresentationDenial::ExternalTimeout,
            UiHostSurfacePresentationDenial::ReconstructionRequired,
        ],
        [
            UiHostSurfacePresentationDenial::ReconstructionRequired,
            UiHostSurfacePresentationDenial::SurfaceOccluded,
        ],
    ] {
        let mut policy = UiNativePresentationRetryPolicy::new();
        observe_denials(&mut policy, attempt(), denials);
        assert_eq!(
            policy.finalize_round(std::time::Instant::now()),
            UiNativePresentationRetryFinalization::Unchanged
        );
        assert_eq!(policy.wake(), None);
    }
}

#[test]
fn presented_binding_suppresses_sibling_retry_in_both_orders() {
    for timeout_first in [true, false] {
        let mut policy = UiNativePresentationRetryPolicy::new();
        let attempt = attempt();
        if timeout_first {
            policy.observe_outcome(
                attempt,
                &rejected(UiHostSurfacePresentationDenial::ExternalTimeout),
            );
        }
        policy.observe_outcome(attempt, &presented());
        if !timeout_first {
            policy.observe_outcome(
                attempt,
                &rejected(UiHostSurfacePresentationDenial::ExternalTimeout),
            );
        }
        assert_eq!(
            policy.finalize_round(std::time::Instant::now()),
            UiNativePresentationRetryFinalization::Unchanged
        );
        assert_eq!(policy.wake(), None);
    }
}

#[test]
fn in_flight_binding_suppresses_sibling_visibility_retry() {
    for in_flight_first in [true, false] {
        let mut policy = UiNativePresentationRetryPolicy::new();
        let attempt = attempt();
        policy.begin_attempt(attempt);
        if in_flight_first {
            policy.round.as_mut().unwrap().effect_posture =
                Some(UiNativePresentationEffectPosture::InFlight);
        }
        policy.observe_outcome(
            attempt,
            &rejected(UiHostSurfacePresentationDenial::SurfaceOccluded),
        );
        if !in_flight_first {
            policy.round.as_mut().unwrap().effect_posture =
                Some(UiNativePresentationEffectPosture::InFlight);
        }
        assert_eq!(
            policy.finalize_round(std::time::Instant::now()),
            UiNativePresentationRetryFinalization::Unchanged
        );
        assert_eq!(policy.wake(), None);
    }
}

#[test]
fn text_atlas_dominates_timeout_without_host_wake_or_timeout_charge() {
    for denials in [
        [
            UiHostSurfacePresentationDenial::ExternalTimeout,
            UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
        ],
        [
            UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred,
            UiHostSurfacePresentationDenial::ExternalTimeout,
        ],
    ] {
        let mut policy = UiNativePresentationRetryPolicy::new();
        for _ in 0..4 {
            observe_denials(&mut policy, attempt(), denials);
            assert_eq!(
                policy.finalize_round(std::time::Instant::now()),
                UiNativePresentationRetryFinalization::Unchanged
            );
            assert_eq!(policy.wake(), None);
        }
        observe_denials(
            &mut policy,
            attempt(),
            [UiHostSurfacePresentationDenial::ExternalTimeout],
        );
        assert!(matches!(
            policy.finalize_round(std::time::Instant::now()),
            UiNativePresentationRetryFinalization::Wake(UiNativePresentationRetryWake::Timeout(_))
        ));
    }
}

#[test]
fn close_clears_observed_and_scheduled_retry_authority() {
    let mut lifecycle = super::super::orchestrator::UiNativeLifecycleOrchestrator::new();
    lifecycle.observe_presentation_retry_outcome(
        attempt(),
        &rejected(UiHostSurfacePresentationDenial::SurfaceOccluded),
    );
    assert!(matches!(
        lifecycle.finalize_presentation_retry_round(std::time::Instant::now()),
        UiNativePresentationRetryFinalization::Wake(UiNativePresentationRetryWake::Visibility)
    ));
    let _ = lifecycle.request_close();
    assert_eq!(lifecycle.presentation_retry_wake(), None);
    assert!(!lifecycle.consume_presentation_visibility());
}

fn observe_denials(
    policy: &mut UiNativePresentationRetryPolicy,
    attempt: UiMountedPresentationAttemptIdentity,
    denials: impl IntoIterator<Item = UiHostSurfacePresentationDenial>,
) {
    for denial in denials {
        policy.observe_outcome(attempt, &rejected(denial));
    }
}

fn rejected(denial: UiHostSurfacePresentationDenial) -> UiHostSurfacePresentationOutcome {
    UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial)
}

fn presented() -> UiHostSurfacePresentationOutcome {
    let world = crate::native::presentation::DrawListWorld::new();
    let initial = world.initial(
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        [],
    );
    UiHostSurfacePresentationOutcome::Presented(
        world.consumption_view(&initial).acknowledge_presented(
            worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
            worth_ui_host_contract::UiMountedCompletedEffects::new(vec![
                worth_ui_host_contract::UiMountedEffectFamily::NativePaint,
            ]),
            worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
                worth_ui_host_contract::UiHostPresentationCostInput {
                    presented_surfaces: 1,
                    ..Default::default()
                },
            ),
        ),
    )
}

fn attempt() -> UiMountedPresentationAttemptIdentity {
    UiMountedPresentationAttemptIdentity::mint_unbound().expect("attempt identity")
}
