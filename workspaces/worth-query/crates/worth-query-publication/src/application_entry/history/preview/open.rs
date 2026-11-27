//! Every opening freshly reads current metadata before resolving a completed node.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn preview(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<WorthQueryApplicationHistoryPreview<S, H>, WorthQueryApplicationHistoryDenial> {
        self.preview_in(request, node, Ungoverned)
    }
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
        self.preview_in(request, node, Governed { capability, operation, input })
    }

    fn preview_in(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<WorthQueryApplicationHistoryPreview<S, H>, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.with_metadata(
            self.access.clone(),
            &self.payload,
            authority,
            |phase, runtime, read| {
                runtime
                    .open_document_history_preview(
                        phase,
                        &program_publication_access(),
                        &self.owner,
                        read,
                        node,
                    )
                    .map(|owner| WorthQueryApplicationHistoryPreview {
                        owner,
                        marker: std::marker::PhantomData,
                    })
            },
        )
    }
}
