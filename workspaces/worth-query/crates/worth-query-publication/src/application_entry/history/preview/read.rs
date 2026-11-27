//! The live document supplies current authority; the holder supplies old truth.
use super::*;
use crate::domain_computation::publish_application_result;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryPreview<S, H>
where
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn read(
        &self,
        document: &WorthQueryApplicationHistoryDocument<
            '_,
            S,
            H,
            impl WorthQueryHistoryPayloadMode<S, H>,
            impl Sized,
        >,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryPreviewResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        self.read_in(
            &document.owner,
            &document.access,
            &document.payload,
            request,
            Ungoverned,
        )
    }
    pub fn read_governed<C, O, I>(
        &self,
        document: &WorthQueryApplicationHistoryDocument<
            '_,
            S,
            H,
            impl WorthQueryHistoryPayloadMode<S, H>,
            impl Sized,
        >,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<
        WorthQueryHistoryPreviewResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    >
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.read_in(
            &document.owner,
            &document.access,
            &document.payload,
            request,
            Governed {
                capability,
                operation,
                input,
            },
        )
    }

    fn read_in(
        &self,
        owner: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        payload: &impl WorthQueryHistoryPayloadMode<S, H>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: impl MetadataAuthority<S, H>,
    ) -> Result<
        WorthQueryHistoryPreviewResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        let context = DocumentRequestContext::bind(owner, request)?;
        context.advance(|phase| {
            let admission = context.admit_metadata(
                phase,
                history_access.clone(),
                payload,
                authority,
                |runtime, _, read| {
                    runtime.prepare_document_history_preview_read(
                        phase,
                        &program_publication_access(),
                        owner,
                        &self.owner,
                        read,
                    )
                },
            )?;
            let installed = context
                .request()
                .application
                .installed_schema()
                .installed_history::<H>()
                .map_err(WorthQueryApplicationHistoryDenial::Installation)?;
            payload
                .with_read(
                    context.request().query(history_access.clone()),
                    phase,
                    Some(&admission),
                    |runtime, binding, external, plan| {
                        let read =
                            WorthQueryDocumentHistoryRead::new(&installed, binding, external, plan);
                        Ok(runtime
                            .read_document_history_preview(
                                phase,
                                &program_publication_access(),
                                owner,
                                &admission,
                                read,
                            )
                            .map(|result| WorthQueryHistoryPreviewResult {
                                original: publish_application_result(
                                    result.into_admitted_disclosed(),
                                ),
                                source: self.step().endpoint().clone(),
                                root_posture: self
                                    .owner
                                    .required_root_posture(&program_publication_access()),
                            })
                            .map_err(WorthQueryApplicationHistoryDenial::Document))
                    },
                )
                .map_err(WorthQueryApplicationHistoryDenial::Query)?
        })
    }
}
