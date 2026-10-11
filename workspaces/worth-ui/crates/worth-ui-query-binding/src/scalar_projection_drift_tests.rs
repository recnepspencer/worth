use worth_runtime_bridge::facade::{
    BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingRequest,
};
use worth_signal::facade::NodeId;

use crate::{
    scalar_projection_async_fixture::{
        admitted_async_request_and_completion, async_ordering, authoritative_async_basis,
        projection_bridge, scalar_async_view,
    },
    scalar_text_projection_fixture::{
        insert_status, projection_workspace, remasked_projection_workspace,
    },
    UiProjectionAvailability, UiProjectionConsumptionBudget, UiProjectionFieldRequirement,
    UiProjectionUnavailableKind, UiScalarProjectionBatchOutcome, UiScalarProjectionBinding,
    UiScalarProjectionBindingAdmission, UiScalarProjectionFactReceipt,
    UiScalarProjectionRegistration, WorthUiQueryWorkspaceExt,
};

#[test]
fn query_remask_posture_maps_to_unavailable_before_native_access() {
    let bridge = projection_bridge();
    let (request, completion) = admitted_async_request_and_completion(
        &bridge,
        NodeId::new(318, 0),
        authoritative_async_basis("commit-remask", "snapshot-remask"),
        64,
    );
    let mut workspace = remasked_projection_workspace();
    insert_status(&mut workspace, "Hidden");
    let view = scalar_async_view(&mut workspace, &request);
    let mut binding = scalar_binding(&workspace);
    let batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
    );
    assert!(batch.remask_posture().is_some());

    let remasked = advanced(binding.consume_async_result_batch(
        &mut workspace,
        batch,
        None,
        UiProjectionConsumptionBudget::platform_pulse(),
    ));

    assert_unavailable(remasked.fact(), UiProjectionUnavailableKind::Remasked);
    assert_eq!(remasked.work().native_indexed_accesses(), 0);
}

#[test]
fn bridge_late_result_maps_to_basis_drift_without_reusing_the_predecessor_value() {
    let bridge = projection_bridge();
    let basis_a = authoritative_async_basis("commit-drift-a", "snapshot-drift-a");
    let basis_b = authoritative_async_basis("commit-drift-b", "snapshot-drift-b");
    let (request, completion) =
        admitted_async_request_and_completion(&bridge, NodeId::new(316, 0), basis_a, 64);
    let mut workspace = projection_workspace(true);
    insert_status(&mut workspace, "Ready");
    let view = scalar_async_view(&mut workspace, &request);
    let mut binding = scalar_binding(&workspace);

    let current_batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncCompletion(completion),
    );
    let current = advanced(binding.consume_async_result_batch(
        &mut workspace,
        current_batch,
        None,
        UiProjectionConsumptionBudget::platform_pulse(),
    ));
    let (current_fact, _) = current.into_fact_and_predecessor();

    let revalidation = bridge
        .revalidate_async_request(&request, basis_b)
        .expect("Bridge must issue basis revalidation");
    let revalidating_batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(revalidation),
    );
    let revalidating = advanced(binding.consume_async_result_batch(
        &mut workspace,
        revalidating_batch,
        Some(current_fact),
        UiProjectionConsumptionBudget::platform_pulse(),
    ));
    let (revalidating_fact, _) = revalidating.into_fact_and_predecessor();

    let late = worth_runtime_bridge::certification_for_test::observe_late_async_completion_for_test(
        &bridge, &request,
    );
    let late_batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
    );
    let drift = advanced(binding.consume_async_result_batch(
        &mut workspace,
        late_batch,
        Some(revalidating_fact),
        UiProjectionConsumptionBudget::platform_pulse(),
    ));

    assert_unavailable(drift.fact(), UiProjectionUnavailableKind::BasisDrift);
    assert!(drift.retained_predecessor().is_some());
    assert_eq!(drift.work().native_indexed_accesses(), 0);
}

#[test]
fn bridge_late_result_maps_to_generation_drift_when_basis_is_stable() {
    let bridge = projection_bridge();
    let request = worth_runtime_bridge::certification_for_test::retryable_async_request_for_test(
        &bridge,
        NodeId::new(317, 0),
        authoritative_async_basis("commit-stable", "snapshot-stable"),
    );
    let mut workspace = projection_workspace(true);
    insert_status(&mut workspace, "Ready");
    let view = scalar_async_view(&mut workspace, &request);
    let mut binding = scalar_binding(&workspace);
    let (cancelled, retry) =
        worth_runtime_bridge::certification_for_test::cancel_and_retry_async_request_for_test(
            &bridge, &request,
        );
    let cancelled_batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(cancelled),
    );
    let cancelled = advanced(binding.consume_async_result_batch(
        &mut workspace,
        cancelled_batch,
        None,
        UiProjectionConsumptionBudget::platform_pulse(),
    ));
    assert_unavailable(cancelled.fact(), UiProjectionUnavailableKind::Cancelled);
    let retry_batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncRetryLineage(retry),
    );
    let retried = advanced(binding.consume_async_result_batch(
        &mut workspace,
        retry_batch,
        None,
        UiProjectionConsumptionBudget::platform_pulse(),
    ));
    assert_unavailable(retried.fact(), UiProjectionUnavailableKind::Retried);

    let late = worth_runtime_bridge::certification_for_test::observe_late_async_completion_for_test(
        &bridge, &request,
    );
    let late_batch = admit_input(
        &bridge,
        &mut workspace,
        &view,
        BridgeMixedCauseOrderingInput::AsyncDeniedCompletion(late),
    );
    let drift = advanced(binding.consume_async_result_batch(
        &mut workspace,
        late_batch,
        None,
        UiProjectionConsumptionBudget::platform_pulse(),
    ));

    assert_unavailable(drift.fact(), UiProjectionUnavailableKind::GenerationDrift);
    assert_eq!(drift.work().native_indexed_accesses(), 0);
}

fn scalar_binding(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> UiScalarProjectionBinding {
    let view = workspace
        .worth_ui()
        .expect("Worth UI domain installed")
        .projection_view("platform.pulse.status")
        .expect("valid Platform Pulse view");
    let registration = UiScalarProjectionRegistration::text(
        view,
        UiProjectionFieldRequirement::declared("status").expect("valid selected field"),
    );
    match registration.admit(workspace) {
        UiScalarProjectionBindingAdmission::Ready(binding) => binding,
        UiScalarProjectionBindingAdmission::Unavailable(unavailable) => {
            panic!("scalar binding must be supported: {unavailable:?}")
        }
        UiScalarProjectionBindingAdmission::Stopped(stop) => {
            panic!("scalar binding must admit: {}", stop.summary())
        }
    }
}

fn admit_input(
    bridge: &worth_runtime_bridge::facade::RuntimeBridge,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    view: &worth_query::facade::runtime::WorthQueryLiveView<
        worth_query::facade::runtime::WorthQueryUnrefinedLiveShape,
    >,
    input: BridgeMixedCauseOrderingInput,
) -> worth_query::facade::runtime::WorthQueryAsyncResultTransitionBatch {
    let request: BridgeMixedCauseOrderingRequest = async_ordering(input);
    let ordering = bridge.order_mixed_causes(&request);
    workspace
        .admit_bridge_async_result_transitions(view, &ordering)
        .expect("Bridge-issued transition must reach Query")
}

fn advanced(outcome: UiScalarProjectionBatchOutcome) -> crate::UiScalarProjectionTransitionReceipt {
    match outcome {
        UiScalarProjectionBatchOutcome::Advanced(receipt) => receipt,
        UiScalarProjectionBatchOutcome::Unchanged(_) => panic!("transition must advance"),
    }
}

fn assert_unavailable(fact: &UiScalarProjectionFactReceipt, expected: UiProjectionUnavailableKind) {
    match fact.availability() {
        UiProjectionAvailability::Unavailable(receipt) => assert_eq!(receipt.kind(), expected),
        other => panic!("expected unavailable {expected:?}, got {other:?}"),
    }
}
