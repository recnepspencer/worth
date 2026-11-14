//! Typed correction fronts defer handler work until whole-envelope resolution.
use super::*;
use crate::application_entry::mutation::envelope::{Connection, Demand, Source};
use crate::application_entry::mutation::{
    captured_input::WorthQueryCapturedMutationInput,
    envelope::{
        WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeOutcome,
        WorthQueryRuntimeEditEnvelope,
    },
};
use crate::application_entry::WorthQueryApplicationRequestExt;
use worth_query_declaration::facade::{
    application_program::*, application_query::*,
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::WorthQueryApplicationProjection,
};

impl<'a, 'p, 's, 'h, S: ApplicationSchema>
    WorthQueryApplicationReapplicationRequest<'a, 'p, 's, 'h, S>
{
    /// Retains the genuine selected action; no handler runs until execute.
    pub fn edit_envelope<'e, 'program, C, P, R>(
        self, program: &'program WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<WorthQueryRuntimeEditEnvelope<'e, 'program, S, P, <C::Source as ApplicationMutationBinding<S>>::IdempotencyKey>, WorthQueryApplicationCorrectionDenial>
    where
        'a: 'e, 'p: 'e, 's: 'e, 'h: 'e,
        C: ApplicationCorrectionBinding<S>,

    P: ApplicationProgramDefinition<S>,
    C::Source: ApplicationMutationBinding<S> + ApplicationMutationAdmissionBinding<S> + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <C::Source as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, C::Source>,
    <C::Source as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
    <C::Source as ApplicationMutationBinding<S>>::Result: Send,
    <C::Source as ApplicationMutationBinding<S>>::Denial: Send,
    <C::Source as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    <C::Source as ApplicationMutationBinding<S>>::IdempotencyKey: serde::Serialize + Send + Sync + 'static,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone + Send + Sync,
    R::Dependents: worth_query_execution::publication_boundary::synchronous_outputs::SynchronousOutputEdges<S, Demand<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S, <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    <<Source<S, R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <Source<S, R> as ApplicationQueryBinding<S>>::Query> + Clone,
{
        use WorthQueryApplicationCorrectionDenial as Denial;
        let key = C::reapplication_key(&self.key).map_err(|cause| {
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
        Ok(runtime.with_correction_front(Box::new(move |context, program, phase, key, entries| {
            let principal = self.correction.request.principal;
            let scope = self.correction.request.scope;
            let selected_source = self.correction.alternative;
            let limits = self.limits;
            let selected = reapplication::PrepareReapplication { request: self, phase }.with_input::<C, _>(
                true,
                |target, access, command| target.derive_required_output_reapplication::<C, R>(
                    access, phase, principal, scope, selected_source, command,
                    limits.maximum_ancestors, limits.maximum_disclosure_results,
                ).map_err(Denial::Derivation),
                |request, target, _, _, selected, phase| {
                    context.correction = Some(selected.intent);
                    let input = selected.input;
                    let entry = crate::application_entry::mutation::envelope::entry::corrective::admit::<S, P, _, _, R>(
                        &request, &target, program, phase, key,
                        WorthQueryCapturedMutationInput::<S, C::Source>::new(input.clone()),
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
