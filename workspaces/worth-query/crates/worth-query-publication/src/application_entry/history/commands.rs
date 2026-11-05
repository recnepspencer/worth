//! Borrowed protocol translation; each command still enters its original owner.
use super::*;
use worth_query_declaration::facade::application_history::{
    ApplicationHistoryBookmarkBinding, ApplicationHistoryNavigationBinding,
    ApplicationHistoryRecentSteps,
};

pub struct WorthQueryHistoryCommands<'document, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    document: &'document mut WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Checks description correspondence only. This admits no READ or effect.
    pub fn check_history_command_context(
        &self,
        context: &WorthQueryHistoryDescriptionContext,
    ) -> Result<(), WorthQueryHistoryCommandContextDenial> {
        self.owner
            .check_history_command_context(&program_publication_access(), context)
    }

    pub fn commands(
        &mut self,
        context: &WorthQueryHistoryDescriptionContext,
    ) -> Result<
        WorthQueryHistoryCommands<'_, 'runtime, S, H, Payload, Outputs>,
        WorthQueryHistoryCommandContextDenial,
    > {
        self.check_history_command_context(context)?;
        Ok(WorthQueryHistoryCommands { document: self })
    }
}

impl<'document, 'runtime, S, H, Payload, Outputs>
    WorthQueryHistoryCommands<'document, 'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    pub fn navigate<N: ApplicationHistoryNavigationBinding<S, H>>(
        self,
    ) -> WorthQueryHistoryNavigationBuilder<'document, 'runtime, S, H, N, Payload, Outputs> {
        let document: &'document WorthQueryApplicationHistoryDocument<
            'runtime,
            S,
            H,
            Payload,
            Outputs,
        > = self.document;
        document.navigate::<N>()
    }

    pub fn window(
        &mut self,
        target: ApplicationHistoryRecentSteps,
    ) -> WorthQueryHistoryWindowChange<'_, 'runtime, S, H, Payload, Outputs> {
        self.document.set_history_window(target)
    }
}

impl<S, H, Payload, Outputs> WorthQueryHistoryCommands<'_, '_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn preview(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<WorthQueryApplicationHistoryPreview<S, H>, WorthQueryApplicationHistoryDenial> {
        self.document.preview(request, node)
    }

    pub fn bookmark<N: ApplicationHistoryBookmarkBinding<S, H>>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryApplicationHistoryDenial> {
        self.document.set_history_bookmark::<N>(request, node, pin)
    }
}

mod governed;

impl<S, H, Payload, Outputs> WorthQueryHistoryCommands<'_, '_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
{
    pub fn maintain(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryReleaseOutcome, WorthQueryApplicationHistoryDenial> {
        self.document.maintain_history(request)
    }
}
