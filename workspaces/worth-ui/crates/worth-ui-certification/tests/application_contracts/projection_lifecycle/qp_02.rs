use super::async_fixture::{admitted_async_completion_for_request, authoritative_async_basis};
use super::support::{unsupported_admission, ScalarLifecycleWorld};
use worth_runtime_bridge::facade::BridgeMixedCauseOrderingInput;
use worth_signal::facade::NodeId;
use worth_ui_query_binding::{
    UiPresentProjection, UiProjectionAvailability, UiProjectionRetainedActivityKind,
    UiProjectionTransitionPosture, UiProjectionUnavailableKind, UiScalarProjectionBindingAdmission,
    UiScalarProjectionFactReceipt,
};

#[test]
fn pending_current_stale_revalidating_current_preserve_exact_value_posture() {
    let (mut world, completion) = ScalarLifecycleWorld::standard(NodeId::new(3130, 0), "Ready");
    let pending = world.initial();
    assert_unavailable(pending.fact(), UiProjectionUnavailableKind::Pending);
    let (pending, _) = pending.into_fact_and_predecessor();

    let current = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    assert_current(current.fact(), "Ready");
    assert!(current.retained_predecessor().is_none());
    assert_eq!(current.work().native_indexed_accesses(), 1);
    let (current, _) = current.into_fact_and_predecessor();

    let revalidation = world
        .bridge
        .revalidate_async_request(
            &world.request,
            authoritative_async_basis("commit-updated", "snapshot-updated"),
        )
        .expect("Bridge must issue revalidation lineage");
    let next_request = revalidation.newer_request().clone();
    let revalidating = world.advance(
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
        Some(current),
    );
    assert_eq!(
        revalidating.posture_trace().postures(),
        [
            UiProjectionTransitionPosture::RetainedStale(UiProjectionRetainedActivityKind::Idle,),
            UiProjectionTransitionPosture::RetainedStale(
                UiProjectionRetainedActivityKind::Revalidating,
            ),
        ]
    );
    assert_retained_revalidating(revalidating.fact(), "Ready");
    assert_eq!(revalidating.work().native_indexed_accesses(), 0);
    let (revalidating, _) = revalidating.into_fact_and_predecessor();

    worth_ui_query_binding::certification::update_projection_status(
        &mut world.workspace,
        world.entity.clone(),
        "Updated",
    );
    let completion = admitted_async_completion_for_request(&world.bridge, &next_request, 72);
    let updated = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(revalidating),
    );
    assert_current(updated.fact(), "Updated");
    assert!(updated.retained_predecessor().is_none());
    assert_eq!(updated.work().native_indexed_accesses(), 1);
}

#[test]
fn failed_cancelled_retried_superseded_and_denied_remain_distinct() {
    assert_failed_preserves_pending();
    assert_cancelled_and_retried_preserve_pending();
    assert_superseded_preserves_pending();
    assert_denied_preserves_pending();
}

fn assert_failed_preserves_pending() {
    let (mut failed, _) = ScalarLifecycleWorld::standard(NodeId::new(3131, 0), "Failed");
    let pending = initial_fact(&mut failed);
    let rejected = worth_runtime_bridge::certification_for_test::reject_async_request_for_test(
        &failed.bridge,
        &failed.request,
    );
    let failed_receipt = failed.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(rejected),
        Some(pending),
    );
    assert_unavailable_preserves(
        &failed_receipt,
        UiProjectionUnavailableKind::Failed,
        UiProjectionUnavailableKind::Pending,
    );
}

fn assert_cancelled_and_retried_preserve_pending() {
    let mut retried = ScalarLifecycleWorld::retryable(NodeId::new(3132, 0), "Retry");
    let pending = initial_fact(&mut retried);
    let (cancelled, retry) =
        worth_runtime_bridge::certification_for_test::cancel_and_retry_async_request_for_test(
            &retried.bridge,
            &retried.request,
        );
    let cancelled = retried.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(cancelled),
        Some(pending),
    );
    assert_unavailable_preserves(
        &cancelled,
        UiProjectionUnavailableKind::Cancelled,
        UiProjectionUnavailableKind::Pending,
    );
    let (_, pending) = cancelled.into_fact_and_predecessor();
    let retry = retried.advance(
        BridgeMixedCauseOrderingInput::AsyncRetryLineage(retry),
        pending,
    );
    assert_unavailable_preserves(
        &retry,
        UiProjectionUnavailableKind::Retried,
        UiProjectionUnavailableKind::Pending,
    );
}

fn assert_superseded_preserves_pending() {
    let (mut superseded, _) = ScalarLifecycleWorld::standard(NodeId::new(3133, 0), "Superseded");
    let pending = initial_fact(&mut superseded);
    let (denied, _replacement) =
        worth_runtime_bridge::certification_for_test::supersede_async_request_for_test(
            &superseded.bridge,
            &superseded.request,
        );
    let superseded_receipt = superseded.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(denied),
        Some(pending),
    );
    assert_unavailable_preserves(
        &superseded_receipt,
        UiProjectionUnavailableKind::Superseded,
        UiProjectionUnavailableKind::Pending,
    );
}

fn assert_denied_preserves_pending() {
    let (mut denied, _) = ScalarLifecycleWorld::standard(NodeId::new(3134, 0), "Denied");
    let pending = initial_fact(&mut denied);
    let oversized =
        worth_runtime_bridge::certification_for_test::deny_oversized_async_completion_for_test(
            &denied.bridge,
            &denied.request,
        );
    let denied_receipt = denied.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(oversized),
        Some(pending),
    );
    assert_unavailable_preserves(
        &denied_receipt,
        UiProjectionUnavailableKind::Denied,
        UiProjectionUnavailableKind::Pending,
    );
}

#[test]
fn unsupported_remasked_basis_and_generation_drift_open_no_native_access() {
    match unsupported_admission() {
        UiScalarProjectionBindingAdmission::Unavailable(receipt) => {
            assert_eq!(receipt.kind(), UiProjectionUnavailableKind::Unsupported)
        }
        other => panic!("unsupported Query support must be unavailable, got {other:?}"),
    }

    let (mut remasked, completion) = ScalarLifecycleWorld::remasked(NodeId::new(3135, 0), "Hidden");
    let remasked_predecessor = remasked.initial();
    assert_unavailable(
        remasked_predecessor.fact(),
        UiProjectionUnavailableKind::Remasked,
    );
    let remasked_predecessor = remasked_predecessor.into_fact_and_predecessor().0;
    let remask = remasked.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(remasked_predecessor),
    );
    assert_unavailable_preserves(
        &remask,
        UiProjectionUnavailableKind::Remasked,
        UiProjectionUnavailableKind::Remasked,
    );

    assert_basis_drift();
    assert_generation_drift();
}

fn assert_basis_drift() {
    let (mut world, completion) = ScalarLifecycleWorld::standard(NodeId::new(3136, 0), "Basis");
    let pending = initial_fact(&mut world);
    let current = world.advance(
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
        Some(pending),
    );
    assert_current(current.fact(), "Basis");
    let (current, _) = current.into_fact_and_predecessor();
    let revalidation = world
        .bridge
        .revalidate_async_request(
            &world.request,
            authoritative_async_basis("commit-drifted", "snapshot-drifted"),
        )
        .expect("Bridge must issue basis-drift revalidation");
    let revalidating = world.advance(
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
        Some(current),
    );
    let (revalidating, _) = revalidating.into_fact_and_predecessor();
    let late = worth_runtime_bridge::certification_for_test::observe_late_async_completion_for_test(
        &world.bridge,
        &world.request,
    );
    let drift = world.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
        Some(revalidating),
    );
    assert_unavailable(drift.fact(), UiProjectionUnavailableKind::BasisDrift);
    assert_retained_revalidating(
        drift
            .retained_predecessor()
            .expect("basis drift must return predecessor truth"),
        "Basis",
    );
    assert_eq!(drift.work().native_indexed_accesses(), 0);
}

fn assert_generation_drift() {
    let mut world = ScalarLifecycleWorld::retryable(NodeId::new(3137, 0), "Generation");
    let pending = initial_fact(&mut world);
    let (cancelled, retry) =
        worth_runtime_bridge::certification_for_test::cancel_and_retry_async_request_for_test(
            &world.bridge,
            &world.request,
        );
    let cancelled = world.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(cancelled),
        Some(pending),
    );
    assert_unavailable_preserves(
        &cancelled,
        UiProjectionUnavailableKind::Cancelled,
        UiProjectionUnavailableKind::Pending,
    );
    let (_, pending) = cancelled.into_fact_and_predecessor();
    let retried = world.advance(
        BridgeMixedCauseOrderingInput::AsyncRetryLineage(retry),
        pending,
    );
    assert_unavailable_preserves(
        &retried,
        UiProjectionUnavailableKind::Retried,
        UiProjectionUnavailableKind::Pending,
    );
    let (_, pending) = retried.into_fact_and_predecessor();
    let late = worth_runtime_bridge::certification_for_test::observe_late_async_completion_for_test(
        &world.bridge,
        &world.request,
    );
    let drift = world.advance(
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
        pending,
    );
    assert_unavailable_preserves(
        &drift,
        UiProjectionUnavailableKind::GenerationDrift,
        UiProjectionUnavailableKind::Pending,
    );
}

fn initial_fact(world: &mut ScalarLifecycleWorld) -> UiScalarProjectionFactReceipt {
    let pending = world.initial();
    assert_unavailable(pending.fact(), UiProjectionUnavailableKind::Pending);
    pending.into_fact_and_predecessor().0
}

fn assert_unavailable_preserves(
    receipt: &worth_ui_query_binding::UiScalarProjectionTransitionReceipt,
    expected: UiProjectionUnavailableKind,
    predecessor: UiProjectionUnavailableKind,
) {
    assert_unavailable(receipt.fact(), expected);
    assert_eq!(
        receipt.posture_trace().postures(),
        [UiProjectionTransitionPosture::Unavailable(expected)]
    );
    assert_unavailable(
        receipt
            .retained_predecessor()
            .expect("unavailable transition must return predecessor truth"),
        predecessor,
    );
    assert_eq!(receipt.work().native_indexed_accesses(), 0);
}

fn assert_unavailable(fact: &UiScalarProjectionFactReceipt, expected: UiProjectionUnavailableKind) {
    match fact.availability() {
        UiProjectionAvailability::Unavailable(receipt) => assert_eq!(receipt.kind(), expected),
        other => panic!("expected unavailable {expected:?}, got {other:?}"),
    }
}

fn assert_current(fact: &UiScalarProjectionFactReceipt, expected: &str) {
    match fact.availability() {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => {
            assert_eq!(value.as_str(), expected)
        }
        other => panic!("expected current `{expected}`, got {other:?}"),
    }
}

fn assert_retained_revalidating(fact: &UiScalarProjectionFactReceipt, expected: &str) {
    match fact.availability() {
        UiProjectionAvailability::Present(UiPresentProjection::RetainedStale {
            value,
            activity,
        }) => {
            assert_eq!(value.as_str(), expected);
            assert_eq!(
                activity.kind(),
                UiProjectionRetainedActivityKind::Revalidating
            );
        }
        other => panic!("expected retained revalidating `{expected}`, got {other:?}"),
    }
}
