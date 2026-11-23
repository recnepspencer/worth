//! A multi-stage edit envelope recorded as one history step.
use super::*;
use crate::application_entry::mutation::envelope::{
    entry::CaptureEditStages, EditStageSource, NoEditStages, WithEditStage,
    WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeModeDenial,
    WorthQueryEditStageToken, WorthQueryRuntimeEditEnvelope,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    application_program::ApplicationProgramDefinition,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

mod outcome;
mod recording;
pub use outcome::{
    WorthQueryHistoryEditEnvelopeDenial, WorthQueryHistoryEditEnvelopeFailure,
    WorthQueryHistoryEditEnvelopeOutcome,
};

/// Move-only builder for one envelope that the document records as one step.
///
/// Stages are authored exactly as on a request envelope. Nothing is admitted,
/// reserved or forked until `execute_recorded`; `cancel` returns the document.
#[must_use = "execute or cancel the envelope to recover the history document"]
pub struct WorthQueryHistoryEditEnvelope<
    'runtime,
    'request,
    'program,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    K,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    envelope: WorthQueryRuntimeEditEnvelope<'request, 'program, S, P, K>,
    scope: &'request WorthQueryRequestScope,
}

impl<'runtime, S, H, Payload>
    WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, WorthQueryHistoryNoOutputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Begin an envelope whose stages commit together as this document's next
    /// step. The request must target this document's own runtime and head.
    pub fn edit_envelope<'request, 'program, P, K>(
        self,
        program: &'program WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'request, 'request, 'request, S>,
        key: K,
    ) -> Result<
        WorthQueryHistoryEditEnvelope<'runtime, 'request, 'program, S, H, P, K, Payload>,
        WorthQueryHistoryEditEnvelopeFailure<'runtime, S, H, Payload>,
    >
    where
        P: ApplicationProgramDefinition<S>,
    {
        let runtime = self.owner.application();
        let refusal = if !std::ptr::eq(runtime, request.application)
            || !std::ptr::eq(runtime, program.runtime())
        {
            Some(WorthQueryApplicationHistoryDenial::ForeignRuntime)
        } else if request.branch != self.owner.branch() {
            Some(WorthQueryApplicationHistoryDenial::RequestTargetMismatch)
        } else {
            None
        };
        if let Some(denial) = refusal {
            return Err(WorthQueryHistoryEditEnvelopeFailure {
                document: self,
                denial: WorthQueryHistoryEditEnvelopeDenial::History(denial),
            });
        }
        Ok(WorthQueryHistoryEditEnvelope {
            envelope: request
                .edit_envelope()
                .idempotency(key)
                .runtime_stages(program),
            scope: request.scope,
            document: self,
        })
    }
}

impl<'runtime, S, H, P, K, Payload, Outputs>
    WorthQueryHistoryEditEnvelope<'runtime, '_, '_, S, H, P, K, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Return the document untouched; nothing was admitted or reserved.
    pub fn cancel(self) -> WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs> {
        self.document
    }

    /// Select positioned semantics before the first stage is captured. A
    /// refusal returns this same builder.
    pub fn ordered_authored(mut self) -> Result<Self, (Self, WorthQueryEditEnvelopeModeDenial)> {
        match self.envelope.order_authored() {
            Ok(()) => Ok(self),
            Err(denial) => Err((self, denial)),
        }
    }

    pub fn push_stage<R, I>(
        &mut self,
        intent: I,
    ) -> Result<
        WorthQueryEditStageToken<
            <I::Binding as ApplicationMutationBinding<S>>::Result,
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
        >,
        WorthQueryEditEnvelopeExecutionDenial,
    >
    where
        I: ApplicationMutationIntent<S> + 'static,
        R: 'static,
        WithEditStage<NoEditStages, R, I>: CaptureEditStages<S, P, K>,
    {
        self.envelope.push_stage::<R, I>(intent)
    }

    /// Capture this stage with its genuine issued row or result-set expectation.
    pub fn push_stage_with_source<R, I, E>(
        &mut self,
        intent: I,
        source: E,
    ) -> Result<
        WorthQueryEditStageToken<
            <I::Binding as ApplicationMutationBinding<S>>::Result,
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
        >,
        WorthQueryEditEnvelopeExecutionDenial,
    >
    where
        I: ApplicationMutationIntent<S> + 'static,
        R: 'static,
        E: EditStageSource<S, I>,
        WithEditStage<NoEditStages, R, I, E>: CaptureEditStages<S, P, K>,
    {
        self.envelope.push_stage_with_source::<R, I, E>(intent, source)
    }
}

impl<S, H, P, K, Payload, Outputs> std::fmt::Debug
    for WorthQueryHistoryEditEnvelope<'_, '_, '_, S, H, P, K, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryEditEnvelope")
            .finish_non_exhaustive()
    }
}
