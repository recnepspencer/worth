use worth_query::facade::installed::operation::WorthQueryBoundExecutionDenial;
use worth_query::facade::runtime;

use crate::{
    operation_live::open_operation_live_resource, WorthUiInstalledQueryBindingReference,
    WorthUiInstalledQueryDomain, WorthUiInstalledQueryView, WorthUiOperationLiveOpenError,
    WorthUiOperationLiveOpenRequest, WorthUiOperationLiveResource, WorthUiQueryViewDefinition,
    WorthUiQueryViewLifecycle,
};

/// Installed live view. Query-owned managed-resource operations are added on
/// this lifecycle type rather than on the registration envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiInstalledLiveQueryView {
    registration: WorthUiInstalledQueryView,
}

impl WorthUiInstalledLiveQueryView {
    pub(super) fn from_registration(registration: WorthUiInstalledQueryView) -> Self {
        debug_assert_eq!(
            registration.definition().lifecycle(),
            WorthUiQueryViewLifecycle::Live
        );
        Self { registration }
    }

    pub fn definition(&self) -> &WorthUiQueryViewDefinition {
        self.registration.definition()
    }

    pub fn installed_domain(&self) -> &WorthUiInstalledQueryDomain {
        self.registration.installed_domain()
    }

    pub fn open_operation(
        &self,
        request: WorthUiOperationLiveOpenRequest,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<WorthUiOperationLiveResource, WorthUiOperationLiveOpenError> {
        let reference = WorthUiInstalledQueryBindingReference::new(
            self.installed_domain().clone(),
            self.definition().clone(),
        );
        workspace
            .with_advancement_mut(|phase, workspace| {
                open_operation_live_resource(&phase, reference, request, workspace)
            })
            .unwrap_or_else(|cause| {
                Err(WorthUiOperationLiveOpenError::Execution(Box::new(
                    WorthQueryBoundExecutionDenial::from_advancement_denial(cause),
                )))
            })
    }
}

impl From<WorthUiInstalledLiveQueryView> for WorthUiInstalledQueryView {
    fn from(view: WorthUiInstalledLiveQueryView) -> Self {
        view.registration
    }
}
