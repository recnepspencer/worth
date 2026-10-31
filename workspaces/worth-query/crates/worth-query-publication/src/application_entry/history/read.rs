use super::*;
use crate::domain_computation::{publish_application_result, WorthQueryPublishedApplicationResult};

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryDocument<'_, S, H>
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
        runtime
            .with_application_advancement(request.scope, |phase| {
                current.query(self.access.clone()).with_admitted_query(
                    &phase,
                    |runtime, binding, external, plan| {
                        let read =
                            WorthQueryDocumentHistoryRead::new(&installed, binding, external, plan);
                        Ok(runtime
                            .read_document_history(
                                &phase,
                                &program_publication_access(),
                                &self.owner,
                                read,
                            )
                            .map(|read| publish_application_result(read.into_admitted_disclosed()))
                            .map_err(WorthQueryApplicationHistoryDenial::Document))
                    },
                )
            })
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
            .map_err(WorthQueryApplicationHistoryDenial::Query)?
    }
}
