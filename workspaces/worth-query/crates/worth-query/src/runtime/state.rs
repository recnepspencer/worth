use super::evidence_identities::{
    runtime_state_snapshot_basis_label_identity,
    runtime_state_snapshot_result_shape_batch_write_receipt_identity,
    runtime_state_snapshot_result_shape_facade_family_identity,
    runtime_state_snapshot_result_shape_write_receipt_identity,
};
use super::ordinary_runtime_posture::project_live_subscription_ordinary_runtime_posture;
use super::{
    WorthQueryAuthorityLane, WorthQueryBatchWriteReceipt, WorthQueryDerivedViewHandle,
    WorthQueryLiveArtifactTarget, WorthQueryLiveView, WorthQueryRuntime, WorthQueryRuntimeError,
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupportStatus,
    WorthQueryRuntimeStateKind, WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeSupportDenial,
    WorthQueryWriteReceipt,
};

pub trait WorthQueryRuntimeStateTarget {
    fn into_state_snapshot(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError>;
}

pub(crate) fn snapshot_live_view_name(
    runtime: &WorthQueryRuntime,
    view_name: &str,
) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
    let state = runtime
        .live_subscriptions
        .get(&WorthQueryLiveArtifactTarget::from_view_name(view_name))
        .ok_or_else(|| WorthQueryRuntimeError::MissingLiveSubscription(view_name.to_string()))?;
    let installation = &state.installation;
    let mixed_cause_delivery = state
        .last_delivery
        .as_ref()
        .map(|delivery| delivery.mixed_cause_delivery());
    let (result_shape_identity, explanation) = match (
        state.last_delivery.as_ref(),
        state.async_result_state.as_ref(),
        mixed_cause_delivery,
    ) {
        (Some(delivery), Some(async_result_state), Some(mixed_cause_delivery)) => (
            installation.view_shape_identity().clone(),
            format!(
                "sync runtime-backed live view `{}` is ready through retained subscription evidence; last delivery cause is `{}` with evidence `{}` at sequence {} and relational_patch={}; mixed-cause delivery is `{}` over ordered members `{}` with {} suppressed and {} denied causes; async result state is `{}` with causality `{}` over basis `{}` and generation `{}`",
                installation.view_name(),
                delivery.delivery_cause_kind().as_str(),
                delivery.delivery_cause_for_reporting(),
                delivery.sequence(),
                delivery.has_relational_patch(),
                mixed_cause_delivery.coalescing_kind().as_public_str(),
                mixed_cause_delivery
                    .ordered_member_kinds()
                    .iter()
                    .map(|kind| kind.as_public_str())
                    .collect::<Vec<_>>()
                    .join(","),
                mixed_cause_delivery.suppressed_cause_identities().len(),
                mixed_cause_delivery.denied_cause_identities().len(),
                async_result_state.kind().as_str(),
                async_result_state.causality_for_reporting(),
                async_result_state.basis_for_reporting(),
                async_result_state.checkpoint_for_reporting()
            ),
        ),
        (Some(delivery), None, Some(mixed_cause_delivery)) => (
            installation.view_shape_identity().clone(),
            format!(
                "sync runtime-backed live view `{}` is ready through retained subscription evidence; last delivery cause is `{}` with evidence `{}` at sequence {} and relational_patch={}; mixed-cause delivery is `{}` over ordered members `{}` with {} suppressed and {} denied causes",
                installation.view_name(),
                delivery.delivery_cause_kind().as_str(),
                delivery.delivery_cause_for_reporting(),
                delivery.sequence(),
                delivery.has_relational_patch(),
                mixed_cause_delivery.coalescing_kind().as_public_str(),
                mixed_cause_delivery
                    .ordered_member_kinds()
                    .iter()
                    .map(|kind| kind.as_public_str())
                    .collect::<Vec<_>>()
                    .join(","),
                mixed_cause_delivery.suppressed_cause_identities().len(),
                mixed_cause_delivery.denied_cause_identities().len()
            ),
        ),
        (None, Some(async_result_state), None) => (
            installation.view_shape_identity().clone(),
            format!(
                "sync runtime-backed live view `{}` is ready through retained subscription evidence; async result state is `{}` with causality `{}` over basis `{}` and generation `{}`",
                installation.view_name(),
                async_result_state.kind().as_str(),
                async_result_state.causality_for_reporting(),
                async_result_state.basis_for_reporting(),
                async_result_state.checkpoint_for_reporting()
            ),
        ),
        (None, None, None) => (
            installation.view_shape_identity().clone(),
            format!(
                "sync runtime-backed live view `{}` is ready through retained subscription evidence",
                installation.view_name()
            ),
        ),
        _ => (
            installation.view_shape_identity().clone(),
            format!(
                "sync runtime-backed live view `{}` is ready through retained subscription evidence",
                installation.view_name()
            ),
        ),
    };
    let mut snapshot = WorthQueryRuntimeStateSnapshot::ready(
        installation.basis_binding_identity().clone(),
        result_shape_identity,
        installation.authority_lane(),
        explanation,
    )
    .with_ordinary_runtime_posture(project_live_subscription_ordinary_runtime_posture(state));
    if let Some(async_result_state) = state.async_result_state.clone() {
        snapshot = snapshot.with_async_result_state(async_result_state);
    }
    if let Some(remask_posture) = state.remask_posture.clone() {
        snapshot = WorthQueryRuntimeStateSnapshot::deferred(
            remask_posture.disposition_kind().state_kind(),
            remask_posture.basis_identity().clone(),
            installation.view_shape_identity().clone(),
            installation.authority_lane(),
            format!(
                "sync runtime-backed live view `{}` is {} through retained remask posture `{}` over basis `{}`; policy `{}`, tenant truth `{}`, tenant schema `{}`, relationship proof `{}`, schema context `{}`",
                installation.view_name(),
                remask_posture.disposition_kind().as_str(),
                remask_posture.reason_kind().as_str(),
                remask_posture.basis_for_reporting(),
                remask_posture.policy_for_reporting(),
                remask_posture.tenant_truth_for_reporting(),
                remask_posture.tenant_schema_for_reporting(),
                remask_posture.relationship_proof_for_reporting(),
                remask_posture.schema_context_for_reporting()
            ),
        )
        .with_ordinary_runtime_posture(project_live_subscription_ordinary_runtime_posture(state));
        if let Some(async_result_state) = state.async_result_state.clone() {
            snapshot = snapshot.with_async_result_state(async_result_state);
        }
        snapshot = snapshot.with_remask_posture(remask_posture);
    }
    Ok(snapshot)
}

impl<T> WorthQueryRuntimeStateTarget for &WorthQueryLiveView<T> {
    fn into_state_snapshot(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        snapshot_live_view_name(runtime, self.name())
    }
}

impl<T> WorthQueryRuntimeStateTarget for &WorthQueryDerivedViewHandle<T> {
    fn into_state_snapshot(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        let inspection = runtime.inspect_derived_view(execution, self)?;
        Ok(WorthQueryRuntimeStateSnapshot::ready(
            inspection.dependency_identity(),
            inspection.materialization_identity(),
            inspection.authority_lane(),
            format!(
                "sync runtime-backed computed view `{}` is ready through retained materialization evidence",
                inspection.name()
            ),
        ))
    }
}

impl WorthQueryRuntimeStateTarget for WorthQueryRuntimeFacadeFamily {
    fn into_state_snapshot(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        let contract = runtime.public_api_contract();
        let row = contract.family(self).ok_or_else(|| {
            WorthQueryRuntimeError::UnsupportedFacadeFamily(
                WorthQueryRuntimeSupportDenial::unsupported(
                    self,
                    "runtime public API contract does not declare this facade family",
                ),
            )
        })?;
        let explanation = row.reason().unwrap_or_else(|| match row.status() {
            WorthQueryRuntimeFamilySupportStatus::Supported => {
                "runtime-backed facade family is currently supported"
            }
            WorthQueryRuntimeFamilySupportStatus::DeferredDebt => {
                "runtime-backed facade family is deferred to its owning future milestone"
            }
            WorthQueryRuntimeFamilySupportStatus::Unsupported => {
                "runtime-backed facade family is unsupported by this runtime"
            }
        });
        let result_shape_identity =
            runtime_state_snapshot_result_shape_facade_family_identity(self);
        match row.status() {
            WorthQueryRuntimeFamilySupportStatus::Supported => {
                Ok(WorthQueryRuntimeStateSnapshot::ready(
                    runtime_state_snapshot_basis_label_identity(row.contract_identity()),
                    result_shape_identity,
                    row.authority_lanes()
                        .first()
                        .copied()
                        .unwrap_or(WorthQueryAuthorityLane::AuthoritativeTruth),
                    explanation,
                ))
            }
            WorthQueryRuntimeFamilySupportStatus::DeferredDebt => {
                Ok(WorthQueryRuntimeStateSnapshot::deferred(
                    WorthQueryRuntimeStateKind::Pending,
                    runtime_state_snapshot_basis_label_identity(row.contract_identity()),
                    result_shape_identity,
                    deferred_authority_lane(self),
                    explanation,
                ))
            }
            WorthQueryRuntimeFamilySupportStatus::Unsupported => {
                Ok(WorthQueryRuntimeStateSnapshot::deferred(
                    WorthQueryRuntimeStateKind::Unsupported,
                    runtime_state_snapshot_basis_label_identity(row.contract_identity()),
                    result_shape_identity,
                    deferred_authority_lane(self),
                    explanation,
                ))
            }
        }
    }
}

impl WorthQueryRuntimeStateTarget for &WorthQueryWriteReceipt {
    fn into_state_snapshot(
        self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        let result_shape_identity =
            runtime_state_snapshot_result_shape_write_receipt_identity(self);
        let commit_evidence_identity = self.commit_evidence_identity().clone();
        Ok(WorthQueryRuntimeStateSnapshot::ready(
            commit_evidence_identity,
            result_shape_identity,
            self.authority_lane(),
            format!(
                "mutation receipt `{}` is ready with `{}` family evidence over `{}` basis lane",
                self.commit_evidence_identity().as_str(),
                self.mutation_family(),
                self.basis_lane()
            ),
        ))
    }
}

impl WorthQueryRuntimeStateTarget for &WorthQueryBatchWriteReceipt {
    fn into_state_snapshot(
        self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _runtime: &WorthQueryRuntime,
    ) -> Result<WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        Ok(WorthQueryRuntimeStateSnapshot::ready(
            runtime_state_snapshot_basis_label_identity(self.batch_identity()),
            runtime_state_snapshot_result_shape_batch_write_receipt_identity(self),
            self.authority_lane(),
            format!(
                "batch write receipt `{}` is ready with {} component writes over `{}` basis lane",
                self.batch_identity().as_str(),
                self.write_count(),
                self.basis_lane()
            ),
        ))
    }
}

fn deferred_authority_lane(family: WorthQueryRuntimeFacadeFamily) -> WorthQueryAuthorityLane {
    match family {
        WorthQueryRuntimeFacadeFamily::Temporal => WorthQueryAuthorityLane::TemporalExecutionState,
        WorthQueryRuntimeFacadeFamily::AsyncResource => WorthQueryAuthorityLane::AsyncResourceState,
        WorthQueryRuntimeFacadeFamily::MixedCauseDelivery
        | WorthQueryRuntimeFacadeFamily::StoreBackedExecution
        | WorthQueryRuntimeFacadeFamily::DurableArtifacts => {
            WorthQueryAuthorityLane::BridgeExternalState
        }
        WorthQueryRuntimeFacadeFamily::Computed | WorthQueryRuntimeFacadeFamily::SharedRead => {
            WorthQueryAuthorityLane::DerivedRuntimeState
        }
        WorthQueryRuntimeFacadeFamily::Effect => WorthQueryAuthorityLane::EffectDeliveryState,
        WorthQueryRuntimeFacadeFamily::Intent | WorthQueryRuntimeFacadeFamily::Submission => {
            WorthQueryAuthorityLane::PendingWriteIntent
        }
        WorthQueryRuntimeFacadeFamily::BranchPreview => WorthQueryAuthorityLane::PreviewTruth,
        WorthQueryRuntimeFacadeFamily::Read
        | WorthQueryRuntimeFacadeFamily::Live
        | WorthQueryRuntimeFacadeFamily::Replay
        | WorthQueryRuntimeFacadeFamily::Write
        | WorthQueryRuntimeFacadeFamily::Inspect => WorthQueryAuthorityLane::AuthoritativeTruth,
    }
}
