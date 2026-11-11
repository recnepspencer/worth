//! Typed C admission and D execution use the existing mutation entry owners.
use super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission, preparation::CandidatePreparation,
    WorthQueryApplicationMutationRequest,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
type Failure = crate::application_entry::WorthQueryPerformedMutationExecutionDenial;
type InputBinding<S, I> =
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::InputBinding;
impl<S, P, K, Prev, I, R> CaptureEditStages<S, P, K> for WithEditStage<Prev, R, I>
where Prev: CaptureEditStages<S, P, K>,

    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: ApplicationMutationIntent<S> + Clone + Send + Sync + 'static,
    I::Binding: ApplicationMutationBinding<S, IdempotencyKey = K, SourceExpectation = NoApplicationMutationSource> + ApplicationMutationAdmissionBinding<S> + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
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
    fn capture(&self, context: &EnvelopeContext<'_, S>, program: &WorthQueryProgramApplicationRuntime<S, P>, key: &Arc<ApplicationEditEnvelopeKey<K>>, entries: &mut Vec<Box<dyn EditEntry<S, P, K>>>) -> Result<(), WorthQueryEditEnvelopeExecutionDenial> {
        self.previous.capture(context, program, key, entries)?;
        let input = ApplicationEncodedInput::encode(self.intent.input().clone())
            .map_err(|cause| WorthQueryEditEnvelopeExecutionDenial::stage(
                self.previous.len(),
                I::Binding::IDENTITY,
                Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(
                    ApplicationMutationIdentityDenial::Input(cause),
                )),
            ))?;
        entries.push(Box::new(TypedEntry::<I, R, InputBinding<S, I>> { intent: self.intent.clone(), input, marker: PhantomData }));
        Ok(())
    }
}
impl<S, P, K, I, R> EditEntry<S, P, K> for TypedEntry<I, R, InputBinding<S, I>>
where

    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: ApplicationMutationIntent<S> + Clone + Send + Sync + 'static,
    I::Binding: ApplicationMutationBinding<S, IdempotencyKey = K, SourceExpectation = NoApplicationMutationSource> + ApplicationMutationAdmissionBinding<S> + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
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
    fn binding(&self) -> &'static str { I::Binding::IDENTITY }
    fn intent(&self) -> ApplicationEditStageIntent {
        ApplicationEditStageIntent::new::<S, I::Binding, R>(&self.input)
    }
    fn secondary(&self, context: &EnvelopeContext<'_, S>, program: &WorthQueryProgramApplicationRuntime<S, P>, phase: &WorthQueryAdvancementPhase<'_>, key: &Arc<ApplicationEditEnvelopeKey<K>>, ordinal: usize, value: StageValue) -> Result<CapturedAuthoredEditStage<S>, Failure> {
        let request = WorthQueryApplicationMutationRequest::new(context.application, context.principal, context.scope, context.branch, self.intent.clone()).without_source().idempotency(key.key());
        let (_, owner) = request.select_performed_owner::<P, R>(phase, program)?;
        if !owner.contains_action::<I::Binding>() { return Err(Failure::Mutation(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)); }
        let order = owner.connection_order().clone();
        let demand = <I::Binding as WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>>::demand_from_source(self.input.input()).map_err(Failure::Connection)?;
        let program = program.clone();
        let principal = context.principal.clone();
        let scope = context.scope.clone();
        let branch = context.branch;
        let intent = self.intent.clone();
        let input = self.input.clone();
        let envelope_key = key.clone();
        CapturedAuthoredEditStage::capture::<I::Binding>(
            &worth_query_execution::publication_boundary::program_publication_access(), phase, context.application, key.clone(), self.input.clone(), ordinal, context.principal.clone(),
            move |context| {
                let identities = envelope_key.stage_with_encoded_input::<S, I::Binding>(&input, ordinal).map_err(|cause| stage_admission_refusal::<S, I::Binding>(&worth_query_execution::publication_boundary::program_publication_access(), ordinal, format!("stage identity: {cause:?}")))?;
                let mut request = WorthQueryApplicationMutationRequest::new(context.application(), &principal, &scope, branch, intent.clone()).without_source().idempotency(envelope_key.key());
                let prepared = (|| {
                    request.require_workflow_transition().map_err(Failure::Mutation)?;
                    let (selected, owner) = request.select_performed_owner::<P, R>(context.phase(), &program)?;
                    if !owner.contains_action::<I::Binding>() { return Err(Failure::Mutation(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)); }
                    let staged = request.stage().map_err(Failure::Mutation)?;
                    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(context.phase(), &request, &identities, staged, &selected).map_err(Failure::Mutation)
                })().map_err(|cause| stage_admission_refusal::<S, I::Binding>(&worth_query_execution::publication_boundary::program_publication_access(), ordinal, format!("stage admission: {cause:?}")))?;
                Ok(AuthoredEditStageAdmission::new(prepared.admission, prepared.principal_identity))
            },
            move |execution, program| program.with_synchronous_required_outputs::<Connection<S, R>, R::Dependents>(&worth_query_execution::publication_boundary::program_publication_access(), execution, demand.clone(), R::RootConnection::declaration(), &order),
            value,
        ).and_then(|stage| stage.with_required_output_dependencies::<I::Binding, Demand<S, R>, R::Dependents>(context.application)).map_err(|cause| Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)))))
    }
    fn execute_prefix(
        &self,
        context: &EnvelopeContext<'_, S>,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &Arc<ApplicationEditEnvelopeKey<K>>,
        ordinal: usize,
        envelope_intent: ApplicationEditIntentIdentity,
        stages: Vec<CapturedAuthoredEditStage<S>>,
        values: Vec<StageValue>,
        brand: Arc<()>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, Failure> {
        let mut request = WorthQueryApplicationMutationRequest::new(context.application, context.principal, context.scope, context.branch, self.intent.clone()).without_source().idempotency(key.key());
        let (selected, owner) = request.select_performed_owner::<P, R>(phase, program)?;
        let identities = key.stage_with_encoded_input::<S, I::Binding>(&self.input, ordinal).map_err(|cause| Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(cause)))?;
        let stage_identity = *identities.edit_stage_identity().expect("canonical prefix");
        let mut work = WorthQueryMutationHandlerWork::NotStarted;
        let prepared = request.prepare_owned_candidate(phase, identities, |request, identities, staged| {
            let mut prepared = <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(phase, request, identities, staged, &selected)?;
            prepared.extension.envelope = Some(envelope_intent);
            Ok(prepared)
        }, &mut work, phase.checked_execution_request_for(context.application.product_runtime()).map_err(|cause| Failure::Mutation(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?.allocation_policy()).map_err(Failure::Mutation)?;
        let candidate = match prepared {
            CandidatePreparation::Prepared(candidate) => candidate,
            CandidatePreparation::Settled(outcome) => return Ok(settled(ordinal, stage_identity, I::Binding::IDENTITY, brand, outcome)),
        };
        let execution = phase.checked_execution_request_for(context.application.product_runtime()).map_err(|cause| Failure::Mutation(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?;
        let demand = <I::Binding as WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>>::demand_from_source(self.input.input()).map_err(Failure::Connection)?;
        let captured = candidate.program.with_authored_edit_stages(&worth_query_execution::publication_boundary::program_publication_access(), stages)
            .with_synchronous_required_outputs::<Connection<S, R>, R::Dependents>(&worth_query_execution::publication_boundary::program_publication_access(), &execution, demand.clone(), R::RootConnection::declaration(), owner.connection_order());
        let binding = candidate.extension.apply(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&candidate.identities));
        let commit = program.compare_and_commit_selected_edit_envelope::<R, I::Binding>(phase, &worth_query_execution::publication_boundary::program_publication_access(), &owner, captured, binding, execution.allocation_policy());
        match commit.landed() {
            Ok((receipt, true)) => Ok(WorthQueryEditEnvelopeOutcome::AlreadyCommitted(receipt)),
            Ok((receipt, false)) => {
                *values[ordinal].lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Box::new(RetainedAuthoredStageResult { result: candidate.result, identity: stage_identity, receipt: Some(receipt.clone()) }));
                let results = values.into_iter().map(|value| value.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take().expect("every committed authored stage retained its result")).collect();
                Ok(WorthQueryEditEnvelopeOutcome::Committed(WorthQueryEditEnvelopeReceipt { receipt, results, brand }))
            }
            Err(outcome) => {
                for slot in values {
                    let mut slot = slot.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    if let Some(value) = slot.take() {
                        match value.downcast::<AuthoredStageDomainDenial>() {
                            Ok(failure) => return Ok(WorthQueryEditEnvelopeOutcome::DomainDenied(WorthQueryEditStageDomainDenial { failure: *failure, brand, outcome: Some(outcome) })),
                            Err(value) => match value.downcast::<AuthoredStageExecutionDenial>() {
                                Ok(failure) => return Ok(WorthQueryEditEnvelopeOutcome::StageExecutionDenied(WorthQueryEditStageExecutionDenial { failure: *failure, outcome })),
                                Err(value) => *slot = Some(value),
                            },
                        }
                    }
                }
                Ok(WorthQueryEditEnvelopeOutcome::Uncommitted(outcome))
            }
        }
    }
}
fn settled<D: Send + 'static, R: 'static>(
    stage: usize,
    identity: ApplicationEditStageIdentity,
    binding: &'static str,
    brand: Arc<()>,
    outcome: crate::application_entry::mutation::WorthQueryApplicationMutationOutcome<D, R>,
) -> WorthQueryEditEnvelopeOutcome {
    use crate::application_entry::mutation::WorthQueryApplicationMutationOutcome as Outcome;
    match outcome {
        Outcome::AlreadyCommitted(receipt) => {
            WorthQueryEditEnvelopeOutcome::AlreadyCommitted(receipt)
        }
        Outcome::IdempotencyIntentDrift => WorthQueryEditEnvelopeOutcome::IdempotencyIntentDrift,
        Outcome::PreviouslyCommitted(receipt) => {
            WorthQueryEditEnvelopeOutcome::PreviouslyCommitted(receipt)
        }
        Outcome::Commit(cause) => WorthQueryEditEnvelopeOutcome::Uncommitted(cause),
        Outcome::Cancelled => WorthQueryEditEnvelopeOutcome::Cancelled { stage },
        Outcome::DeadlineExceeded => WorthQueryEditEnvelopeOutcome::DeadlineExceeded { stage },
        Outcome::DomainDenied(denial) => {
            WorthQueryEditEnvelopeOutcome::DomainDenied(WorthQueryEditStageDomainDenial {
                failure: AuthoredStageDomainDenial {
                    identity,
                    binding,
                    denial: Box::new(denial),
                },
                brand,
                outcome: None,
            })
        }
        Outcome::Committed { .. } => unreachable!("preparation never publishes a candidate"),
    }
}
