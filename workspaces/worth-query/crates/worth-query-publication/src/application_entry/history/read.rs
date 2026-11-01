use super::*;
use crate::domain_computation::{publish_application_result, WorthQueryPublishedApplicationResult};

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
    /// Read this exact model under current authorization. An externally changed
    /// head or a recreated model refuses instead of silently rebinding the document.
    pub fn read(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        let runtime = self.owner.application();
        if !std::ptr::eq(runtime, request.application) {
            return Err(WorthQueryApplicationHistoryDenial::ForeignRuntime);
        }
        runtime
            .with_application_advancement(request.scope, |phase| {
                self.read_in_advancement(&phase, request)
            })
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
    }

    pub(super) fn read_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        let runtime = self.owner.application();
        let installed = runtime
            .installed_schema()
            .installed_history::<H>()
            .map_err(WorthQueryApplicationHistoryDenial::Installation)?;
        let current = WorthQueryApplicationRequest {
            application: runtime,
            principal: request.principal,
            scope: request.scope,
            branch: self.owner.branch(),
        };
        self.payload
            .with_read(
                current.query(self.access.clone()),
                phase,
                None,
                |runtime, binding, external, plan| {
                    let read =
                        WorthQueryDocumentHistoryRead::new(&installed, binding, external, plan);
                    Ok(runtime
                        .read_document_history(
                            phase,
                            &program_publication_access(),
                            &self.owner,
                            read,
                        )
                        .map(|read| publish_application_result(read.into_admitted_disclosed()))
                        .map_err(WorthQueryApplicationHistoryDenial::Document))
                },
            )
            .map_err(WorthQueryApplicationHistoryDenial::Query)?
    }
}
