//! Bounded journal pages always use fresh metadata authority and the original model READ.
//! A caller cannot rewrite a sealed continuation position.
//! ```compile_fail,E0451
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryPageCursor;
//! fn rewind(cursor: WorthQueryHistoryPageCursor) -> WorthQueryHistoryPageCursor {
//!     WorthQueryHistoryPageCursor { after: 0, ..cursor }
//! }
//! ```
//! Keep an issued cursor for a fresh admitted retry instead.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryHistoryPageCursor;
//! fn keep(cursor: &WorthQueryHistoryPageCursor) -> WorthQueryHistoryPageCursor { *cursor }
//! ```
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryDocument<'_, S, H>
where
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn history_page(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryPage, WorthQueryApplicationHistoryDenial> {
        self.history_page_with(request, limits, after, |_, _, read| Ok(read))
    }
    pub fn history_page_governed<C, O, I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.history_page_with(request, limits, after, |phase, runtime, read| {
            read.with_metadata_capability(runtime, phase, capability, operation, input)
        })
    }
    fn history_page_with(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
        prepare: impl for<'read> FnOnce(
            &WorthQueryAdvancementPhase<'_>,
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<
            WorthQueryDocumentHistoryRead<'read, S, H>,
            WorthQueryDocumentHistoryDenial,
        >,
    ) -> Result<WorthQueryHistoryPage, WorthQueryApplicationHistoryDenial> {
        let runtime = self.owner.application();
        if !std::ptr::eq(runtime, request.application) {
            return Err(WorthQueryApplicationHistoryDenial::ForeignRuntime);
        }
        let current = WorthQueryApplicationRequest {
            application: runtime,
            principal: request.principal,
            scope: request.scope,
            branch: self.owner.branch(),
        };
        runtime
            .with_application_advancement(request.scope, |phase| {
                super::metadata::with_metadata_read::<S, H, _>(
                    &phase,
                    &current,
                    self.access.clone(),
                    |runtime, _, read| {
                        let read = prepare(&phase, runtime, read)?;
                        runtime.read_document_history_page(
                            &phase,
                            &program_publication_access(),
                            &self.owner,
                            read,
                            limits,
                            after,
                        )
                    },
                )
            })
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
    }
}
