use std::collections::BTreeMap;

use crate::memory_workspace::WorthQuerySnapshotIdentity;

#[cfg(test)]
use super::{record_forbidden_fallback_seam_invocation, WorthQueryForbiddenFallbackSeam};
use super::{
    WorthQueryDerivedMaterializationResult, WorthQueryDerivedViewHandle,
    WorthQueryLiveArtifactBinding, WorthQueryLiveArtifactBundle, WorthQueryLiveArtifactTarget,
    WorthQueryLiveReadResult, WorthQueryPatchBatch, WorthQueryRuntimeError, WorthQueryWorkspace,
};

impl WorthQueryWorkspace {
    pub fn declare_bridge_async_live_view<T>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        request: super::DeclarativeLiveQueryRequest,
        schema_view: super::QuerySchemaView,
        bridge_request: &worth_runtime_bridge::facade::AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<super::WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.runtime.declare_bridge_async_live_view(
            request_execution,
            name,
            request,
            schema_view,
            bridge_request,
        )
    }

    pub fn declare_bridge_async_live_view_with_typed_identity<T>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        request: super::DeclarativeLiveQueryRequest,
        schema_view: super::QuerySchemaView,
        typed_request_identity: &[crate::application::WorthQueryAsyncRequestIdentityPart],
        bridge_request: &worth_runtime_bridge::facade::AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<super::WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.runtime
            .declare_bridge_async_live_view_with_typed_identity(
                request_execution,
                name,
                request,
                schema_view,
                typed_request_identity,
                bridge_request,
            )
    }

    pub fn admit_bridge_async_result_transitions<T>(
        &mut self,
        view: &super::WorthQueryLiveView<T>,
        ordering: &worth_runtime_bridge::facade::BridgeMixedCauseOrdering,
    ) -> Result<super::WorthQueryAsyncResultTransitionBatch, super::WorthQueryAsyncSourceBindingError>
    {
        self.runtime
            .admit_bridge_async_result_transitions(view, ordering)
    }

    pub fn take_bridge_async_initial_result<T>(
        &mut self,
        view: &super::WorthQueryLiveView<T>,
    ) -> Result<super::WorthQueryAsyncResultTransitionBatch, super::WorthQueryAsyncSourceBindingError>
    {
        self.runtime.take_bridge_async_initial_result(view)
    }

    pub(crate) fn managed_live_capability(
        &self,
    ) -> std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability> {
        self.runtime.managed_live_capability()
    }

    pub(crate) fn admit_managed_live_capability(
        &self,
        capability: &std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability>,
        resource_name: &str,
    ) -> Result<(), WorthQueryRuntimeError> {
        self.runtime
            .admit_managed_live_capability(capability, resource_name)
    }

    pub(crate) fn close_managed_live_view<T>(
        &mut self,
        view: &super::WorthQueryLiveView<T>,
        capability: &std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability>,
        cause: super::WorthQueryManagedLiveResourceCloseCause,
    ) -> Result<crate::subscription::SubscriptionLifecycleCloseout, WorthQueryRuntimeError> {
        self.admit_managed_live_capability(capability, view.name())?;
        self.runtime.reap_abandoned_managed_live_resources()?;
        self.runtime.close_managed_live_view(view, cause)
    }

    pub(crate) fn read_managed_live_view<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &super::WorthQueryLiveView<T>,
        capability: &std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability>,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.admit_managed_live_capability(capability, view.name())?;
        self.runtime.reap_abandoned_managed_live_resources()?;
        self.read_live_result(execution, view)
    }

    pub(crate) fn read_managed_live_view_for_granular_scope<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &super::WorthQueryLiveView<T>,
        capability: &std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability>,
        scope: &crate::live::WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.admit_managed_live_capability(capability, view.name())?;
        self.runtime.reap_abandoned_managed_live_resources()?;
        let review = self
            .runtime
            .review_runtime_live_read_execution(view.subscription_installation().clone())?;
        let handoff = self
            .runtime
            .resolve_reviewed_admitted_live_read_execution_handoff(review)?;
        let binding = self.runtime.prepare_live_read_execution_binding(handoff)?;
        self.runtime
            .execute_granular_live_read_execution_binding(execution, binding, scope, basis)
    }

    pub(crate) fn drain_managed_live_view<T>(
        &mut self,
        view: &super::WorthQueryLiveView<T>,
        capability: &std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability>,
    ) -> Result<super::WorthQueryManagedLiveRuntimeDelivery, WorthQueryRuntimeError> {
        self.admit_managed_live_capability(capability, view.name())?;
        self.runtime.reap_abandoned_managed_live_resources()?;
        let target = self.resolve_live_artifact_target(view.name())?;
        if target.subscription_installation() != Some(view.subscription_installation()) {
            return Err(WorthQueryRuntimeError::MissingLiveSubscription(
                view.name().to_string(),
            ));
        }
        Ok(super::WorthQueryManagedLiveRuntimeDelivery::new(
            view.name(),
            self.runtime.drain_live_delivery_batches(view),
        ))
    }

    pub(crate) fn observe_managed_live_view<T>(
        &mut self,
        view: &super::WorthQueryLiveView<T>,
        capability: &std::sync::Arc<super::WorthQueryManagedLiveWorkspaceCapability>,
    ) -> Result<super::WorthQueryManagedLiveLifecycleObservation, WorthQueryRuntimeError> {
        self.admit_managed_live_capability(capability, view.name())?;
        self.runtime.observe_managed_live_view(view)
    }

    pub fn resolve_live_artifact_target(
        &self,
        view_name: &str,
    ) -> Result<WorthQueryLiveArtifactTarget, WorthQueryRuntimeError> {
        self.runtime.resolve_live_artifact_target(view_name)
    }

    pub fn read<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &super::WorthQueryLiveView<T>,
    ) -> Vec<crate::memory_workspace::WorthQueryEntity> {
        self.read_live_result(execution, view)
            .expect("live view declaration admitted before workspace.read execution")
            .rows()
            .to_vec()
    }

    pub fn read_live_result<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &super::WorthQueryLiveView<T>,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        self.read_live_intent(view).execute(execution)
    }

    pub fn read_live_target(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        let resolved = self.resolve_live_artifact_target(target.terminal_view_name_projection())?;
        let installation = resolved
            .subscription_installation()
            .cloned()
            .ok_or_else(|| {
                WorthQueryRuntimeError::MissingLiveSubscription(
                    target.terminal_view_name_projection().to_string(),
                )
            })?;
        self.runtime
            .execute_live_read_for_installation(execution, installation)
    }

    pub fn state_live<T>(
        &self,
        view: &super::WorthQueryLiveView<T>,
    ) -> Result<super::WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        self.state_live_by_name(view.name())
    }

    pub fn state_live_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<super::WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        let resolved = self.resolve_live_artifact_target(target.terminal_view_name_projection())?;
        self.state_live_by_name(resolved.terminal_view_name_projection())
    }

    pub(crate) fn state_live_by_name(
        &self,
        view_name: &str,
    ) -> Result<super::WorthQueryRuntimeStateSnapshot, WorthQueryRuntimeError> {
        super::state::snapshot_live_view_name(&self.runtime, view_name)
    }

    pub fn read_live_artifact_binding(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        artifact_name: impl Into<String>,
        targets: impl IntoIterator<Item = WorthQueryLiveArtifactTarget>,
    ) -> Result<WorthQueryLiveArtifactBinding, WorthQueryRuntimeError> {
        #[cfg(test)]
        record_forbidden_fallback_seam_invocation(
            WorthQueryForbiddenFallbackSeam::ReadLiveArtifactBinding,
        );
        let retained_targets = targets.into_iter().collect::<Vec<_>>();
        self.read_live_artifact_bundle(execution, retained_targets.clone())?
            .bind_live_artifact(artifact_name, retained_targets)
    }

    pub fn read_live_artifact_bundle(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        targets: impl IntoIterator<Item = WorthQueryLiveArtifactTarget>,
    ) -> Result<WorthQueryLiveArtifactBundle, WorthQueryRuntimeError> {
        #[cfg(test)]
        record_forbidden_fallback_seam_invocation(
            WorthQueryForbiddenFallbackSeam::ReadLiveArtifactBundle,
        );
        let mut retained_targets = targets.into_iter().collect::<Vec<_>>();
        retained_targets.sort();
        retained_targets.dedup();

        let mut reads = BTreeMap::new();
        for target in retained_targets {
            let installation = target.subscription_installation().cloned().ok_or_else(|| {
                WorthQueryRuntimeError::MissingLiveView(
                    target.terminal_view_name_projection().to_string(),
                )
            })?;
            let result = self
                .runtime
                .execute_live_read_for_installation(execution, installation)?;
            reads.insert(target, result);
        }
        let snapshot_identity = live_bundle_snapshot_identity(&reads)?;
        Ok(WorthQueryLiveArtifactBundle::new(snapshot_identity, reads))
    }

    pub fn observe<T>(&mut self, view: &super::WorthQueryLiveView<T>) -> WorthQueryPatchBatch {
        self.runtime.drain_patches(view)
    }

    pub fn materialize_result<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> Result<WorthQueryDerivedMaterializationResult, WorthQueryRuntimeError> {
        self.runtime.read_derived_result(execution, view)
    }

    pub fn observe_computed<T>(
        &mut self,
        view: &WorthQueryDerivedViewHandle<T>,
    ) -> WorthQueryPatchBatch {
        self.runtime.drain_derived_patches(view)
    }

    pub fn subscription_basis_digest<T>(
        &self,
        view: &super::WorthQueryLiveView<T>,
    ) -> Result<String, WorthQueryRuntimeError> {
        self.subscription_basis_digest_by_name(view.name())
    }

    pub fn subscription_basis_digest_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<String, WorthQueryRuntimeError> {
        let resolved = self.resolve_live_artifact_target(target.terminal_view_name_projection())?;
        self.subscription_basis_digest_by_name(resolved.terminal_view_name_projection())
    }

    pub(crate) fn subscription_basis_digest_by_name(
        &self,
        view_name: &str,
    ) -> Result<String, WorthQueryRuntimeError> {
        Ok(self
            .runtime
            .inspect_live_view_name_installation(view_name)?
            .basis_binding_projection()
            .label()
            .to_string())
    }
}

fn live_bundle_snapshot_identity(
    reads: &BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryLiveReadResult>,
) -> Result<WorthQuerySnapshotIdentity, WorthQueryRuntimeError> {
    let snapshot_identities = reads
        .iter()
        .map(|(target, result)| {
            (
                target.terminal_view_name_projection().to_string(),
                result.receipt().snapshot_identity().clone(),
            )
        })
        .collect::<Vec<_>>();
    let shared_snapshot_identity = snapshot_identities
        .first()
        .map(|(_, snapshot_identity)| snapshot_identity);
    let has_single_snapshot_identity = shared_snapshot_identity
        .map(|expected| {
            snapshot_identities.iter().all(|(_, snapshot_identity)| {
                expected.is_same_current_identity_as(snapshot_identity)
            })
        })
        .unwrap_or(true);
    match snapshot_identities.as_slice() {
        [] => Ok(WorthQuerySnapshotIdentity::empty_relational_state()),
        [(_, snapshot_identity)] if has_single_snapshot_identity => Ok(snapshot_identity.clone()),
        _ if has_single_snapshot_identity => Ok(snapshot_identities[0].1.clone()),
        _ => Err(WorthQueryRuntimeError::ReadCompositionDenied(
            super::WorthQueryReadDenial::new(
                super::WorthQueryReadDenialKind::ExecutionDenied,
                format!(
                    "live artifact bundle materialized multiple snapshot identities: {}",
                    snapshot_identities
                        .iter()
                        .map(|(view_name, snapshot_identity)| {
                            format!(
                                "{view_name}:{}",
                                snapshot_identity
                                    .evidence_identity()
                                    .terminal_projection_for_reporting()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
        )),
    }
}
