//! Finite availability changes keep their unfinished frontier in the document.
use super::*;
use worth_query_declaration::facade::application_history::{
    ApplicationHistoryBookmarkBinding, ApplicationHistoryRecentSteps,
    ApplicationHistoryWindowBinding,
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[must_use = "advance the change or resume its registered target through the same document"]
pub struct WorthQueryHistoryWindowChange<'doc, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    document: &'doc mut WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    target: ApplicationHistoryRecentSteps,
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    pub fn set_history_window(
        &mut self,
        target: ApplicationHistoryRecentSteps,
    ) -> WorthQueryHistoryWindowChange<'_, 'runtime, S, H, Payload, Outputs> {
        WorthQueryHistoryWindowChange {
            document: self,
            target,
        }
    }
}

impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    fn retention_read<T>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        change: impl for<'read> FnOnce(
            &WorthQueryAdvancementPhase<'_>,
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<T, WorthQueryDocumentHistoryDenial>,
    ) -> Result<T, WorthQueryApplicationHistoryDenial> {
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
                    &self.payload,
                    |runtime, _, read| change(&phase, runtime, read),
                )
            })
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
    }

    pub fn history_retention_status(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.retention_read(request, |phase, runtime, read| {
            runtime.read_document_history_retention(
                phase,
                &program_publication_access(),
                &self.owner,
                read,
            )
        })
    }

    pub fn set_history_bookmark<N: ApplicationHistoryBookmarkBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.retention_read(request, |phase, runtime, read| {
            runtime.change_document_history_bookmark::<H, N>(
                phase,
                &program_publication_access(),
                &self.owner,
                &self.access,
                read,
                node,
                pin,
            )
        })
    }
}

impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
{
    /// One bounded actual owner release; no model payload is disclosed.
    pub fn maintain_history(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryReleaseOutcome, WorthQueryApplicationHistoryDenial> {
        let runtime = self.owner.application();
        if !std::ptr::eq(runtime, request.application) {
            return Err(WorthQueryApplicationHistoryDenial::ForeignRuntime);
        }
        runtime
            .with_application_advancement(request.scope, |phase| {
                self.owner
                    .release_history_owner_after_output_cleanup(
                        &phase,
                        &program_publication_access(),
                        |branch| self.outputs.release_for_branch(&phase, branch),
                    )
                    .map_err(WorthQueryApplicationHistoryDenial::Document)?
                    .map_err(WorthQueryApplicationHistoryDenial::Outputs)
            })
            .map_err(WorthQueryApplicationHistoryDenial::ExecutionRequest)?
    }
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryHistoryWindowChange<'_, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Each call freshly admits the window capability and excludes at most one
    /// old step. Dropping this borrow leaves its unfinished target registered.
    pub fn advance<N: ApplicationHistoryWindowBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.document.maintain_history(request)?;
        self.document
            .retention_read(request, |phase, runtime, read| {
                runtime.change_document_history_window::<H, N>(
                    phase,
                    &program_publication_access(),
                    &self.document.owner,
                    &self.document.access,
                    read,
                    self.target,
                )
            })
    }
}
