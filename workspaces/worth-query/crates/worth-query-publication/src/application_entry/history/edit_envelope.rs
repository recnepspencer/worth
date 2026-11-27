//! A multi-stage edit envelope recorded as one history step.
use super::*;
use crate::application_entry::mutation::envelope::{
    entry::CaptureEditStages, EditStageSource, NoEditStages, WithEditStage,
    WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeModeDenial,
    WorthQueryEditStageToken, WorthQueryRuntimeEditEnvelope,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionBinding, ApplicationInverseBinding},
    application_operation::{
        ApplicationMutationAdmissionBinding, ApplicationMutationBinding, ApplicationMutationInputScope,
        ApplicationMutationIntent, ApplicationMutationOutputPosture, ApplicationMutationScopeResolution,
    },
    application_history::ApplicationHistoryModelDomain,
    application_program::ApplicationProgramDefinition,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

mod outcome;
mod recording;
mod sealing;
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
    principal: &'request WorthQueryAuthenticatedExternalPrincipal<S>,
    scope: &'request WorthQueryRequestScope,
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
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
        WorthQueryHistoryEditEnvelope<'runtime, 'request, 'program, S, H, P, K, Payload, Outputs>,
        WorthQueryHistoryEditEnvelopeFailure<'runtime, S, H, Payload, Outputs>,
    >
    where
        P: ApplicationProgramDefinition<S>,
    {
        let refusal =
            DocumentRequestContext::bind_exact(&self.owner, request, program.runtime()).err();
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
            principal: request.principal,
            scope: request.scope,
            document: self,
        })
    }
}

impl<'runtime, S, H, Payload, Outputs>
    WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Begin a correction envelope recorded as this document's next step:
    /// stage 0 reverses the selected source, and stages pushed after it commit
    /// with it. The source must be a step this document recorded on its
    /// selected line, and its meaning names the stage it reverses, so any
    /// stage of a multi-stage step can be corrected (public #368). Reversing
    /// a creation cascades only to what the same source binding created
    /// (`co_retire_created`); a sibling another stage created stays, so the
    /// reversal refuses as a partial retirement while that sibling lives. A
    /// retirement in a stage after the corrective one is postponed on public
    /// #369 (it refuses `LaterStageRetirementPostponed`). A correction step
    /// cannot itself be corrected: a corrective binding declares no
    /// correction, so selecting it refuses `SourceNotCorrectable`. Reapply
    /// cannot reach a document at all: this takes only a reversal request
    /// (`corrections().reverse(..)`), and `corrections().reapply(..)` builds a
    /// different type, so a history reapplication does not compile (ruling
    /// 8a.2; history reapply is future work). The step's reversal fact follows navigation: a move
    /// retires the facts of the steps it leaves and restores those of the
    /// steps it enters. A correction needs an owned-model-graph history: any
    /// other model domain refuses `CorrectionRequiresOwnedModelGraph` before
    /// anything else, and navigation refuses again across a recorded
    /// correction (defense in depth). Only an authored `retire_created` or
    /// `restore_retired` records (ruling 8a.2); any other correction refuses
    /// `UnsupportedHistoryCorrection` at open.
    pub fn correction_envelope<'request, 'program, C, P, R>(
        self,
        program: &'program WorthQueryProgramApplicationRuntime<S, P>,
        correction: crate::application_entry::correction::WorthQueryApplicationCorrectionRequest<
            'request,
            'request,
            'request,
            'request,
            S,
        >,
    ) -> Result<
        WorthQueryHistoryEditEnvelope<
            'runtime,
            'request,
            'program,
            S,
            H,
            P,
            <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey,
            Payload,
            Outputs,
        >,
        WorthQueryHistoryEditEnvelopeFailure<'runtime, S, H, Payload, Outputs>,
    >
    where
        P: ApplicationProgramDefinition<S>,
        C: ApplicationCorrectionBinding<S>,
        ApplicationInverseBinding<S, C>: ApplicationMutationBinding<S> + ApplicationMutationAdmissionBinding<S>,
        <ApplicationInverseBinding<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, ApplicationInverseBinding<S, C>>,
        <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, ApplicationInverseBinding<S, C>>,
        <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Result: Send,
        <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Denial: Send,
        <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::PrincipalIdentity>,
        <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize + Send + Sync + 'static,
        R: crate::application_entry::mutation::envelope::entry::outputs::EditStageOutputs<S, ApplicationInverseBinding<S, C>>,
    {
        let runtime = self.owner.application();
        let corrections = correction.corrections();
        let (principal, scope) = (corrections.principal, corrections.scope);
        let domain = runtime
            .installed_schema()
            .installed_history::<H>()
            .map(|installed| installed.definition().parts().model_domain);
        let refusal = if let Err(cause) = domain {
            Some(WorthQueryApplicationHistoryDenial::Installation(cause))
        } else if domain != Ok(ApplicationHistoryModelDomain::OwnedModelGraph) {
            Some(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::CorrectionRequiresOwnedModelGraph,
            ))
        } else if !supported_history_correction::<S, C>() {
            Some(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::UnsupportedHistoryCorrection,
            ))
        } else {
            let request = WorthQueryApplicationRequest {
                application: corrections.application,
                principal,
                scope,
                branch: corrections.branch,
            };
            DocumentRequestContext::bind_exact(&self.owner, &request, program.runtime()).err()
        };
        if let Some(denial) = refusal {
            return Err(WorthQueryHistoryEditEnvelopeFailure {
                document: self,
                denial: WorthQueryHistoryEditEnvelopeDenial::History(denial),
            });
        }
        match correction.history_envelope::<C, P, R>(program) {
            Ok(envelope) => Ok(WorthQueryHistoryEditEnvelope {
                envelope,
                principal,
                scope,
                document: self,
            }),
            Err(denial) => Err(WorthQueryHistoryEditEnvelopeFailure {
                document: self,
                denial: WorthQueryHistoryEditEnvelopeDenial::Correction(denial),
            }),
        }
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

/// Ruling 8a.2: an authored `retire_created` (its source creates, cascades
/// included) or `restore_retired` (its source retires). A correction with no
/// authored identity, a live-owner member correspondence, or a preserving
/// source is not one.
fn supported_history_correction<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>() -> bool {
    C::authored_identity()
        .and_then(|authored| authored.declaration().ok())
        .is_some_and(|declared| {
            declared.live_member_owner().is_none()
                && matches!(
                    declared.parts().source_posture,
                    ApplicationMutationOutputPosture::Create | ApplicationMutationOutputPosture::Retire
                )
        })
}
