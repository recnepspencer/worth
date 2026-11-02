//! Protocol context never substitutes for current metadata disclosure.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

impl<S, H, Payload, Outputs> WorthQueryHistoryCommands<'_, '_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn preview_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryApplicationHistoryPreview<S, H>, WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.document
            .preview_governed(request, node, capability, operation, input)
    }

    pub fn bookmark_governed<N, C, O, I>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial>
    where
        N: ApplicationHistoryBookmarkBinding<S, H>,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.document.set_history_bookmark_governed::<N, C, O, I>(
            request, node, pin, capability, operation, input,
        )
    }
}
