use worth_runtime_bridge::facade::AdmittedBridgeAsyncRequestIdentity;

use super::async_source_binding::WorthQueryRuntimeAsyncSourceBinding;
use super::runtime_declarations::admit_live_view_declaration_receipt;
use super::{
    DeclarativeLiveQueryRequest, QuerySchemaView, WorthQueryLiveView, WorthQueryRuntime,
    WorthQueryRuntimeError, WorthQueryRuntimeFacadeFamily,
};

impl WorthQueryRuntime {
    pub fn declare_bridge_async_live_view<T>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
        bridge_request: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        let future_selection =
            crate::subscription::QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![
                    crate::subscription::QuerySubscriptionAsyncRequestIdentityPart::new(
                        "bridge-declaration",
                        bridge_request
                            .lowered()
                            .declaration_identity_for_reporting(),
                    ),
                    crate::subscription::QuerySubscriptionAsyncRequestIdentityPart::new(
                        "bridge-request",
                        bridge_request.request_identity_for_reporting(),
                    ),
                ],
            );
        self.declare_bridge_async_live_view_with_selection(
            request_execution,
            name,
            request,
            schema_view,
            future_selection,
            bridge_request,
        )
    }

    pub fn declare_bridge_async_live_view_with_typed_identity<T>(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
        typed_request_identity: &[crate::application::WorthQueryAsyncRequestIdentityPart],
        bridge_request: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        let future_selection = crate::subscription::QuerySubscriptionFutureSelection::
            async_resource_with_identity(
                true,
                typed_request_identity
                    .iter()
                    .map(
                        crate::subscription::QuerySubscriptionAsyncRequestIdentityPart::
                            from_async_identity_part,
                    )
                    .collect(),
            );
        self.declare_bridge_async_live_view_with_selection(
            request_execution,
            name,
            request,
            schema_view,
            future_selection,
            bridge_request,
        )
    }

    fn declare_bridge_async_live_view_with_selection<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
        future_selection: crate::subscription::QuerySubscriptionFutureSelection,
        bridge_request: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Live)?;
        let name = name.into();
        admit_live_view_declaration_receipt(&*self.backend, &name, &request, &schema_view)?;
        let activation = self.install_live_subscription_for_request(
            execution,
            &name,
            &request,
            schema_view.clone(),
            future_selection,
        )?;
        let binding = WorthQueryRuntimeAsyncSourceBinding::admit(&name, bridge_request);
        self.finish_live_view_declaration(name, request, schema_view, activation, Some(binding))
    }
}
