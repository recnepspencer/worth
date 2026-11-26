//! Typed correction fronts defer handler work until whole-envelope resolution.
use super::*;
use crate::application_entry::mutation::envelope::entry::outputs::EditStageOutputs;
use crate::application_entry::mutation::{
    captured_input::WorthQueryCapturedMutationInput,
    envelope::{
        WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeOutcome,
        WorthQueryRuntimeEditEnvelope,
    },
};
use crate::application_entry::WorthQueryApplicationRequestExt;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

impl<'a, 'p, 's, 'h, S: ApplicationSchema>
    WorthQueryApplicationCorrectionRequest<'a, 'p, 's, 'h, S>
{
    /// Retains the genuine selected action; no handler runs until execute.
    pub fn edit_envelope<'e, 'program, C, P, R>(
        self, program: &'program WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<WorthQueryRuntimeEditEnvelope<'e, 'program, S, P, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey>, WorthQueryApplicationCorrectionDenial>
    where
        'a: 'e, 'p: 'e, 's: 'e, 'h: 'e,
        C: ApplicationCorrectionBinding<S>,

    P: ApplicationProgramDefinition<S>,
    ApplicationInverseBinding<S, C>: ApplicationMutationBinding<S> + ApplicationMutationAdmissionBinding<S>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, ApplicationInverseBinding<S, C>>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, ApplicationInverseBinding<S, C>>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Result: Send,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Denial: Send,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize + Send + Sync + 'static,
    R: EditStageOutputs<S, ApplicationInverseBinding<S, C>>,
{
        self.envelope_runtime::<C, P, R>(program, false)
    }

    /// The same envelope for a history document (plan §10.1): a read-only
    /// head pass derives the corrective stage's meaning at the document head,
    /// and the front derives and admits again on the private candidate, under
    /// a command key that belongs to that one reserved attempt.
    pub(in crate::application_entry) fn history_envelope<'e, 'program, C, P, R>(
        self, program: &'program WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<WorthQueryRuntimeEditEnvelope<'e, 'program, S, P, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey>, WorthQueryApplicationCorrectionDenial>
    where
        'a: 'e, 'p: 'e, 's: 'e, 'h: 'e,
        C: ApplicationCorrectionBinding<S>,

    P: ApplicationProgramDefinition<S>,
    ApplicationInverseBinding<S, C>: ApplicationMutationBinding<S> + ApplicationMutationAdmissionBinding<S>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, ApplicationInverseBinding<S, C>>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, ApplicationInverseBinding<S, C>>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Result: Send,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Denial: Send,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize + Send + Sync + 'static,
    R: EditStageOutputs<S, ApplicationInverseBinding<S, C>>,
{
        use crate::application_entry::mutation::envelope::runtime::{CorrectionHead, CorrectionSource};
        use WorthQueryApplicationCorrectionDenial as Denial;
        let corrections = &self.correction.request;
        let (application, principal, scope, branch) =
            (corrections.application, corrections.principal, corrections.scope, corrections.branch);
        let source = self.correction.source;
        let limits = self.limits;
        let runtime = self.envelope_runtime::<C, P, R>(program, true)?;
        Ok(runtime.with_correction_head(Box::new(move |_, phase| {
            let access = worth_query_execution::publication_boundary::program_publication_access();
            let target = application.on_branch(branch).select(phase).map_err(Denial::ProductSelection)?;
            let input = target
                .derive_correction_inverse_input::<C>(
                    &access, phase, principal, scope, source,
                    limits.maximum_ancestors, limits.maximum_disclosure_results,
                )
                .map_err(Denial::Derivation)?;
            Ok(CorrectionHead {
                intent: ApplicationEditStageIntent::new::<S, ApplicationInverseBinding<S, C>, R>(&input),
                operation: <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::descriptor(),
                source: CorrectionSource {
                    binding: C::IDENTITY,
                    commit: source.source_commit().clone(),
                    member: source.source_member_text(),
                },
            })
        })))
    }

    fn envelope_runtime<'e, 'program, C, P, R>(
        self, program: &'program WorthQueryProgramApplicationRuntime<S, P>, history: bool,
    ) -> Result<WorthQueryRuntimeEditEnvelope<'e, 'program, S, P, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey>, WorthQueryApplicationCorrectionDenial>
    where
        'a: 'e, 'p: 'e, 's: 'e, 'h: 'e,
        C: ApplicationCorrectionBinding<S>,

    P: ApplicationProgramDefinition<S>,
    ApplicationInverseBinding<S, C>: ApplicationMutationBinding<S> + ApplicationMutationAdmissionBinding<S>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, ApplicationInverseBinding<S, C>>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, ApplicationInverseBinding<S, C>>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Result: Send,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Denial: Send,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize + Send + Sync + 'static,
    R: EditStageOutputs<S, ApplicationInverseBinding<S, C>>,
{
        use WorthQueryApplicationCorrectionDenial as Denial;
        let key = C::inverse_key(&self.key).map_err(|cause| {
            Denial::Mutation(
                crate::application_entry::WorthQueryApplicationRequestMutationDenial::Identity(
                    ApplicationMutationIdentityDenial::Key(cause),
                ),
            )
        })?;
        let request = &self.correction.request;
        let runtime = request
            .application
            .request(request.principal, request.scope)
            .on_branch(request.branch)
            .edit_envelope()
            .idempotency(key)
            .runtime_stages(program);
        let mut this = self;
        Ok(runtime.with_correction_front::<ApplicationInverseBinding<S, C>>(Box::new(move |context, program, phase, key, entries, navigation| {
            let correction_denial = |denial| WorthQueryEditEnvelopeExecutionDenial::envelope(P::IDENTITY.as_str(),
                crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Correction(Box::new(denial)));
            let reselected;
            let mut source = this.correction.source;
            if history {
                // The candidate pass: the history owner retargeted `context` to
                // its private candidate, one fork edge below the head.
                this.correction.request.branch = context.branch;
                this.limits.maximum_ancestors =
                    this.limits.maximum_ancestors.map(|limit| limit.saturating_add(1));
                let attempt = context.correction_attempt.as_deref().ok_or_else(|| {
                    WorthQueryEditEnvelopeExecutionDenial::envelope(P::IDENTITY.as_str(),
                        crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Correction(
                            Box::new(Denial::RecordedOutcomeUnavailable)))
                })?;
                this.history_attempt = Some(attempt.to_owned());
                // The source was selected on the document branch; select it
                // again on the candidate, whose incarnation the disclosure
                // binds (plan §10.1 step 4).
                let corrections = &this.correction.request;
                let target = corrections.application.on_branch(corrections.branch).select(phase)
                    .map_err(|cause| correction_denial(Denial::ProductSelection(cause)))?;
                reselected = target
                    .select_recorded_correction_source(
                        phase, corrections.principal, corrections.scope, &source.recorded_source(),
                        this.limits.maximum_ancestors, this.limits.maximum_disclosure_results,
                    )
                    .map_err(|cause| correction_denial(Denial::Source(cause)))?;
                drop(target);
                source = &reselected;
            }
            let navigation = navigation.filter(|_| history);
            let request = WorthQueryApplicationCorrectionRequest {
                correction: super::super::inspection::WorthQueryApplicationCorrection {
                    request: this.correction.request,
                    source,
                },
                key: this.key,
                limits: this.limits,
                history_attempt: this.history_attempt,
            };
            let principal = request.correction.request.principal;
            let scope = request.correction.request.scope;
            let selected_source = request.correction.source;
            let limits = request.limits;
            let selected = Prepare { request, phase }.with_input::<C, _>(
                true,
                |target, _access, command| R::inverse::<C>(
                    target, phase, principal, scope, selected_source, command,
                    limits.maximum_ancestors, limits.maximum_disclosure_results,
                    navigation,
                ).map_err(Denial::Derivation),
                |request, target, _, _, selected, phase| {
                    context.correction = Some(selected.intent);
                    let input = selected.input;
                    let entry = crate::application_entry::mutation::envelope::entry::corrective::admit::<S, P, _, _, R>(
                        &request, &target, program, phase, key,
                        WorthQueryCapturedMutationInput::<S, ApplicationInverseBinding<S, C>>::new(input.clone()),
                        input, selected.source, selected.intent,
                    )?;
                    entries.push(entry);
                    Ok(())
                },
            ).map_err(|cause| WorthQueryEditEnvelopeExecutionDenial::envelope(P::IDENTITY.as_str(),
                crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Correction(Box::new(cause))))?;
            match selected {
                candidate::CorrectionFront::Selected(()) => Ok(None),
                candidate::CorrectionFront::IntentDrift => Ok(Some(WorthQueryEditEnvelopeOutcome::IdempotencyIntentDrift)),
                candidate::CorrectionFront::PublicationPending | candidate::CorrectionFront::ProductUnpublished(_) => Err(
                    WorthQueryEditEnvelopeExecutionDenial::envelope(P::IDENTITY.as_str(),
                        crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Correction(Box::new(Denial::RecordedOutcomeUnavailable)))
                ),
            }
        })))
    }
}
