use crate::adapter::{
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackRequest,
};
use crate::facade::tests::{
    runtime_with_custom_writeback_authority, runtime_with_writeback_authority,
};
use crate::facade::{
    AdmittedBridgeAsyncCompletion, AdmittedBridgeAsyncRequestIdentity,
    BridgeAsyncRequestSubscriptionInstance, BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncWritebackAdmissionRequest, BridgeRuntimePolicy, BridgeWritebackEffectClass,
    BridgeWritebackEffectIntent, BridgeWritebackFailureClass, BridgeWritebackOutcomeClass,
    RuntimeBridge,
};
use worth_foundational::facade::{AspectKey, AspectValue};
use worth_signal::facade::NodeId;

use super::{
    admit_request_response_completion, admit_request_response_identity,
    admit_subscription_backed_identity, preview_active_subscription_with_basis,
    subscription_backed_raw_completion,
};

#[derive(Clone)]
pub(crate) struct RejectingWritebackAuthority;

impl TruthWritebackAuthority for RejectingWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, TruthWritebackAuthorityError> {
        Ok(
            crate::adapter::TruthWritebackReceipt::new_with_failure_class(
                BridgeWritebackOutcomeClass::Rejected,
                Some(BridgeWritebackFailureClass::AuthorityDenied),
                &request,
            ),
        )
    }
}

pub(crate) fn runtime_with_authority() -> RuntimeBridge {
    runtime_with_writeback_authority(BridgeRuntimePolicy::development())
}

pub(crate) fn runtime_with_rejecting_authority() -> RuntimeBridge {
    runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::development(),
        RejectingWritebackAuthority,
    )
}

pub(crate) fn admitted_authoritative_request_response_completion(
    runtime: &RuntimeBridge,
    node: NodeId,
    suffix: &str,
) -> AdmittedBridgeAsyncCompletion {
    let report = admit_request_response_completion(
        runtime,
        node,
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture(format!("truth-main:{suffix}")),
            crate::truth_identity_fixtures::truth_commit_fixture(format!("commit:{suffix}")),
            crate::truth_identity_fixtures::truth_snapshot_fixture(format!("snapshot:{suffix}")),
        ),
        64,
    );
    report
        .admitted_completion()
        .expect("authoritative request-response completion should admit")
        .clone()
}

pub(crate) fn admitted_preview_subscription_backed_completion(
    runtime: &RuntimeBridge,
    node: NodeId,
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> AdmittedBridgeAsyncCompletion {
    let preview_active = preview_active_subscription_with_basis(
        runtime,
        suffix,
        crate::truth_identity_fixtures::truth_branch_fixture(format!("truth-preview:{suffix}")),
        crate::truth_identity_fixtures::truth_snapshot_fixture(format!(
            "snapshot-preview:{suffix}"
        )),
        resource_request,
    );
    let truth_basis = BridgeAsyncRequestTruthViewBasis::preview(&preview_active);
    let subscription_instance = BridgeAsyncRequestSubscriptionInstance::preview(&preview_active);
    let request_identity =
        admit_subscription_backed_identity(runtime, node, truth_basis, subscription_instance)
            .expect("preview subscription-backed identity should admit");
    let validated = runtime
        .validate_async_completion_envelope(
            &request_identity,
            subscription_backed_raw_completion(&request_identity, 32),
        )
        .expect("preview completion envelope should validate");
    runtime
        .admit_async_completion(&request_identity, &validated)
        .expect("preview completion should admit")
        .admitted_completion()
        .expect("preview completion should stay admitted before writeback admission")
        .clone()
}

pub(crate) fn newer_authoritative_request_identity(
    runtime: &RuntimeBridge,
    node: NodeId,
    suffix: &str,
) -> AdmittedBridgeAsyncRequestIdentity {
    admit_request_response_identity(
        runtime,
        node,
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture(format!("truth-main:{suffix}")),
            crate::truth_identity_fixtures::truth_commit_fixture(format!("commit:{suffix}")),
            crate::truth_identity_fixtures::truth_snapshot_fixture(format!("snapshot:{suffix}")),
        ),
    )
}

pub(crate) fn projected_state_diff_intent(marker: &str) -> BridgeWritebackEffectIntent {
    BridgeWritebackEffectIntent::validated_scalar_patch(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        AspectKey::new("bridge.async.writeback").expect("static aspect key should validate"),
        AspectValue::String(marker.to_owned().into()),
    )
    .expect("projected-state-diff effect intent should validate")
}

pub(crate) fn aspect_reconciliation_intent(marker: &str) -> BridgeWritebackEffectIntent {
    BridgeWritebackEffectIntent::validated_scalar_patch(
        BridgeWritebackEffectClass::AspectReconciliation,
        AspectKey::new("bridge.async.writeback").expect("static aspect key should validate"),
        AspectValue::String(marker.to_owned().into()),
    )
    .expect("aspect-reconciliation effect intent should validate")
}

pub(crate) fn integer_projected_state_diff_intent(marker: i64) -> BridgeWritebackEffectIntent {
    BridgeWritebackEffectIntent::validated_scalar_patch(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        AspectKey::new("bridge.async.writeback").expect("static aspect key should validate"),
        AspectValue::Int64(marker),
    )
    .expect("integer projected-state-diff effect intent should validate")
}

pub(crate) fn authoritative_writeback_request(
    completion: &AdmittedBridgeAsyncCompletion,
    marker: &str,
) -> BridgeAsyncWritebackAdmissionRequest {
    BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
        completion,
        projected_state_diff_intent(marker),
        completion
            .request_identity()
            .basis_binding()
            .truth_view_basis()
            .clone(),
    )
}
