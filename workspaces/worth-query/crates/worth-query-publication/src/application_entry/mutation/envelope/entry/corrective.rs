//! An already admitted correction enters the same typed entry and prefix executor.
use super::*;
use crate::application_entry::correction::{
    WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrections,
};
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission, WorthQueryApplicationMutationRequest,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
use worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_execution::publication_boundary::recorded_correction::{
    WorthQueryCorrectionIntent, WorthQueryPreparedCorrectionSource,
};

pub(in crate::application_entry) fn admit<S, P, K, I, R>(
    request: &WorthQueryApplicationCorrections<'_, '_, '_, S>,
    target: &WorthQuerySelectedProductOperation<'_, S>,
    program: &WorthQueryProgramApplicationRuntime<S, P>,
    phase: &WorthQueryAdvancementPhase<'_>,
    key: &Arc<ApplicationEditEnvelopeKey<K>>,
    intent: I,
    input: ApplicationEncodedInput<<I::Binding as ApplicationMutationBinding<S>>::InputBinding>,
    source: Option<Arc<WorthQueryPreparedCorrectionSource>>,
    correction: WorthQueryCorrectionIntent,
) -> Result<Box<dyn EditEntry<S, P, K>>, WorthQueryApplicationCorrectionDenial>
where

    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: ApplicationMutationIntent<S> + Clone + Send + Sync + 'static,
    I::Binding: ApplicationMutationBinding<S, IdempotencyKey = K> + ApplicationMutationAdmissionBinding<S> + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission: crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::Result: Send,
    <I::Binding as ApplicationMutationBinding<S>>::Denial: Send,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    K: serde::Serialize + Send + Sync + 'static,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone + Send + Sync,
    R::Dependents: worth_query_execution::publication_boundary::synchronous_outputs::SynchronousOutputEdges<S, Demand<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S, <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    <<Source<S, R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <Source<S, R> as ApplicationQueryBinding<S>>::Query> + Clone,
{
    use WorthQueryApplicationCorrectionDenial as Denial;
    let identities = key
        .stage_with_encoded_input::<S, I::Binding>(&input, 0)
        .map_err(|cause| {
            Denial::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(cause))
        })?;
    let mut mutation = WorthQueryApplicationMutationRequest::new(
        request.application,
        request.principal,
        request.scope,
        request.branch,
        intent.clone(),
    )
    .idempotency(key.key());
    let (_, owner) = mutation
        .select_performed_owner::<P, R>(phase, program)
        .map_err(Denial::RequiredOutputs)?;
    if !owner.contains_action::<I::Binding>() {
        return Err(Denial::SourceBindingUnavailable);
    }
    mutation
        .require_workflow_transition()
        .map_err(Denial::Mutation)?;
    let staged = mutation.stage().map_err(Denial::Mutation)?;
    let mut prepared =
        <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(
            phase,
            &mutation,
            &identities,
            staged,
            target,
        )
        .map_err(Denial::Mutation)?;
    prepared.extension.correction = Some(correction);
    Ok(Box::new(TypedEntry::<S, I, R> {
        intent,
        input,
        marker: PhantomData,
        correction_source: source,
        corrective: true,
        original: Mutex::new(Some(AuthoredEditStageAdmission::new(
            prepared.admission,
            prepared.principal_identity,
        ))),
        commit_extension: Arc::new(Mutex::new(Some(prepared.extension))),
    }))
}
