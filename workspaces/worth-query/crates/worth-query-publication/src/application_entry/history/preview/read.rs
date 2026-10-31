//! The live document supplies current authority; the holder supplies old truth.
use super::*;
use crate::domain_computation::publish_application_result;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryPreview<S, H>
where
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn read(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'_, S, H>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryPreviewResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        self.read_with(document, request, |_, _, read| Ok(read))
    }
    pub fn read_governed<C, O, I>(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'_, S, H>,
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
        self.read_with(document, request, |phase, runtime, read| {
            read.with_metadata_capability(runtime, phase, capability, operation, input)
        })
    }

    fn read_with(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'_, S, H>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        prepare: impl for<'read> FnOnce(
            &WorthQueryAdvancementPhase<'_>,
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<
            WorthQueryDocumentHistoryRead<'read, S, H>,
            WorthQueryDocumentHistoryDenial,
        >,
    ) -> Result<
        WorthQueryHistoryPreviewResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        let runtime = document.owner.application();
        if !std::ptr::eq(runtime, request.application) {
            return Err(WorthQueryApplicationHistoryDenial::ForeignRuntime);
        }
        let current = WorthQueryApplicationRequest {
            application: runtime,
            principal: request.principal,
            scope: request.scope,
            branch: document.owner.branch(),
        };
        runtime
            .with_application_advancement(request.scope, |phase| {
                let admission = super::super::metadata::with_metadata_read::<S, H, _>(
                    &phase,
                    &current,
                    document.access.clone(),
                    |runtime, _, read| {
                        let read = prepare(&phase, runtime, read)?;
                        runtime.prepare_document_history_preview_read(
                            &phase,
                            &program_publication_access(),
                            &document.owner,
                            &self.owner,
                            read,
                        )
                    },
                )?;
                let installed = runtime
                    .installed_schema()
                    .installed_history::<H>()
                    .map_err(WorthQueryApplicationHistoryDenial::Installation)?;
                current
                    .query(document.access.clone())
                    .with_admitted_preview_query(
                        &phase,
                        Some(&admission),
                        |runtime, binding, external, plan| {
                            let read = WorthQueryDocumentHistoryRead::new(
                                &installed, binding, external, plan,
                            );
                            Ok(runtime
                                .read_document_history_preview(
                                    &phase,
                                    &program_publication_access(),
                                    &document.owner,
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
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
    }
}
