use crate::ordinary::read::{
    WorthQueryProjectionDeclaration, WorthQueryProjectionOutcome, WorthQueryProjectionViolation,
    WorthQueryReadProjectionBinding,
};
use crate::runtime::{
    WorthQueryLiveReadResult, WorthQueryLiveView, WorthQueryManagedLiveWorkspaceCapability,
    WorthQueryRuntimeError, WorthQueryUnrefinedLiveShape, WorthQueryWorkspace,
};
use std::sync::Arc;

use super::WorthQueryManagedLiveDelivery;

#[derive(Debug)]
#[must_use = "managed live resources remain active until the handle is explicitly closed"]
pub struct WorthQueryManagedLiveHandle {
    view: Option<WorthQueryLiveView<WorthQueryUnrefinedLiveShape>>,
    workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
    projection_binding: Option<WorthQueryReadProjectionBinding>,
}

impl WorthQueryManagedLiveHandle {
    pub fn name(&self) -> &str {
        self.view().name()
    }

    pub(crate) fn resource_identity(&self) -> &crate::WorthQueryEvidenceIdentity {
        self.view()
            .subscription_installation()
            .installation_identity()
    }

    pub fn read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        workspace.read_managed_live_view(execution, self.view(), &self.workspace_capability)
    }

    pub(crate) fn read_granular_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        workspace: &mut WorthQueryWorkspace,
        scope: &crate::live::WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<WorthQueryLiveReadResult, WorthQueryRuntimeError> {
        workspace.read_managed_live_view_for_granular_scope(
            execution,
            self.view(),
            &self.workspace_capability,
            scope,
            basis,
        )
    }

    pub fn project(
        &self,
        read: &WorthQueryLiveReadResult,
        declaration: WorthQueryProjectionDeclaration,
    ) -> WorthQueryProjectionOutcome {
        let expected = self
            .view()
            .subscription_installation()
            .installation_identity();
        let actual = read.receipt().installation_identity();
        if actual != expected {
            return WorthQueryProjectionOutcome::Violation(
                WorthQueryProjectionViolation::LiveInstallationMismatch {
                    expected: expected.clone(),
                    actual: actual.clone(),
                },
            );
        }
        self.projection_binding().consume_live(read, declaration)
    }

    pub(crate) fn project_contract(
        &self,
        read: &WorthQueryLiveReadResult,
        contract: crate::projection_consumption::ProjectionAuthorityContract,
    ) -> WorthQueryProjectionOutcome {
        self.projection_binding()
            .consume_live_contract(read, contract)
    }

    pub fn drain(
        &self,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryManagedLiveDelivery, WorthQueryRuntimeError> {
        workspace
            .drain_managed_live_view(self.view(), &self.workspace_capability)
            .map(WorthQueryManagedLiveDelivery::from_runtime)
    }

    pub(crate) fn view(&self) -> &WorthQueryLiveView<WorthQueryUnrefinedLiveShape> {
        self.view
            .as_ref()
            .expect("active managed live handle must retain its resource view")
    }

    pub(crate) fn workspace_capability(&self) -> &Arc<WorthQueryManagedLiveWorkspaceCapability> {
        &self.workspace_capability
    }

    pub(crate) fn new(
        view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
        workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
        projection_binding: WorthQueryReadProjectionBinding,
    ) -> Self {
        Self {
            view: Some(view),
            workspace_capability,
            projection_binding: Some(projection_binding),
        }
    }

    pub(crate) fn into_resource_parts(
        mut self,
    ) -> (
        WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
        Arc<WorthQueryManagedLiveWorkspaceCapability>,
        WorthQueryReadProjectionBinding,
    ) {
        let view = self
            .view
            .take()
            .expect("transferred managed live handle must retain its resource view");
        let projection_binding = self
            .projection_binding
            .take()
            .expect("transferred managed live handle must retain its projection binding");
        (
            view,
            Arc::clone(&self.workspace_capability),
            projection_binding,
        )
    }

    fn projection_binding(&self) -> &WorthQueryReadProjectionBinding {
        self.projection_binding
            .as_ref()
            .expect("active managed live handle must retain its projection binding")
    }

    pub(crate) fn disarm(&mut self) {
        self.view = None;
    }
}

impl Drop for WorthQueryManagedLiveHandle {
    fn drop(&mut self) {
        if let Some(view) = self.view.take() {
            self.workspace_capability.abandon(view);
        }
    }
}
