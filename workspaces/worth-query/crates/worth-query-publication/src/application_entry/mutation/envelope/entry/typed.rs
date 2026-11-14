//! Typed C admission and D execution use the existing mutation entry owners.
use super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission, preparation::CandidatePreparation,
    WorthQueryApplicationMutationRequest,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
use worth_query_execution::publication_boundary::authored_edit::AuthoredEditStageContext;
type Failure = crate::application_entry::WorthQueryPerformedMutationExecutionDenial;
impl<S, P, K, I, R> EditEntry<S, P, K> for TypedEntry<S, I, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: ApplicationMutationIntent<S> + Clone + Send + Sync + 'static,
    I::Binding:
        ApplicationMutationBinding<S, IdempotencyKey = K> + ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
            S,
            I::Binding,
        >,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::Result: Send,
    <I::Binding as ApplicationMutationBinding<S>>::Denial: Send,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
    K: serde::Serialize + Send + Sync + 'static,
    R: EditStageOutputs<S, I::Binding>,
{
    fn binding(&self) -> &'static str {
        I::Binding::IDENTITY
    }
    fn intent(&self) -> ApplicationEditStageIntent {
        ApplicationEditStageIntent::new::<S, I::Binding, R>(&self.input)
    }
    fn secondary(
        &self,
        context: &EnvelopeContext<'_, S>,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &Arc<ApplicationEditEnvelopeKey<K>>,
        ordinal: usize,
        value: StageValue,
    ) -> Result<CapturedAuthoredEditStage<S>, Failure> {
        let request = WorthQueryApplicationMutationRequest::new(
            context.application,
            context.principal,
            context.scope,
            context.branch,
            self.intent.clone(),
        )
        .idempotency(key.key());
        let (_, owner) = request.select_envelope_owner::<P>(phase, program)?;
        if !owner.contains_action::<I::Binding>() {
            return Err(Failure::Mutation(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
            ));
        }
        R::require(program, &owner)?;
        let program = program.clone();
        let principal = context.principal.clone();
        let scope = context.scope.clone();
        let branch = context.branch;
        let intent = self.intent.clone();
        let input = self.input.clone();
        let envelope_key = key.clone();
        let extension = self.commit_extension.clone();
        let admit = move |context: &AuthoredEditStageContext<'_, '_, '_, S>| {
            let identities = envelope_key
                .stage_with_encoded_input::<S, I::Binding>(&input, ordinal)
                .map_err(|cause| {
                    stage_admission_refusal::<S, I::Binding>(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        ordinal,
                        format!("stage identity: {cause:?}"),
                    )
                })?;
            let mut request = WorthQueryApplicationMutationRequest::new(
                context.application(),
                &principal,
                &scope,
                branch,
                intent.clone(),
            )
            .idempotency(envelope_key.key());
            let prepared = (|| {
                request
                    .require_workflow_transition()
                    .map_err(Failure::Mutation)?;
                let (selected, owner) =
                    request.select_envelope_owner::<P>(context.phase(), &program)?;
                if !owner.contains_action::<I::Binding>() {
                    return Err(Failure::Mutation(
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
                    ));
                }
                R::require(&program, &owner)?;
                let staged = request.stage().map_err(Failure::Mutation)?;
                <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(
                    context.phase(),
                    &request,
                    &identities,
                    staged,
                    &selected,
                )
                .map_err(Failure::Mutation)
            })()
            .map_err(|cause| {
                stage_admission_refusal::<S, I::Binding>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    ordinal,
                    format!("stage admission: {cause:?}"),
                )
            })?;
            extension
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_or_insert(prepared.extension);
            Ok(AuthoredEditStageAdmission::new(
                prepared.admission,
                prepared.principal_identity,
            ))
        };
        let access = worth_query_execution::publication_boundary::program_publication_access();
        let original = self
            .original
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let stage = match original {
            Some(original) => CapturedAuthoredEditStage::capture_admitted::<I::Binding>(
                &access,
                context.application,
                key.clone(),
                self.input.clone(),
                ordinal,
                context.principal.clone(),
                original,
                admit,
                value,
            ),
            None => CapturedAuthoredEditStage::capture::<I::Binding>(
                &access,
                phase,
                context.application,
                key.clone(),
                self.input.clone(),
                ordinal,
                context.principal.clone(),
                admit,
                value,
            ),
        };
        stage
            .map(|stage| match context.correction {
                Some(intent) => stage.with_correction_intent(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    intent,
                ),
                None => stage,
            })
            .map_err(|cause| {
                Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)),
                ))
            })
            .and_then(|stage| R::dependencies(stage, context.application, &self.input, &owner))
    }
    fn execute_prefix(
        &self,
        context: &EnvelopeContext<'_, S>,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        key: &Arc<ApplicationEditEnvelopeKey<K>>,
        ordinal: usize,
        unseen: &UnseenAuthoredEnvelope<'_>,
        envelope_intent: ApplicationEditIntentIdentity,
        stages: Vec<CapturedAuthoredEditStage<S>>,
        values: Vec<StageValue>,
        brand: Arc<()>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, Failure> {
        if self.corrective && self.correction_source.is_none() {
            return Err(Failure::Correction(Box::new(crate::application_entry::WorthQueryApplicationCorrectionDenial::RecordedOutcomeUnavailable)));
        }
        if context.correction.is_some() && !self.corrective {
            return Err(Failure::Correction(Box::new(crate::application_entry::WorthQueryApplicationCorrectionDenial::SourceBindingUnavailable)));
        }
        let request = WorthQueryApplicationMutationRequest::new(
            context.application,
            context.principal,
            context.scope,
            context.branch,
            self.intent.clone(),
        )
        .idempotency(key.key());
        let (_selected, owner) = request.select_envelope_owner::<P>(phase, program)?;
        R::require(program, &owner)?;
        let identities = key
            .stage_with_encoded_input::<S, I::Binding>(&self.input, ordinal)
            .map_err(|cause| {
                Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(cause))
            })?;
        let stage_identity = *identities.edit_stage_identity().expect("canonical prefix");
        request
            .require_workflow_transition()
            .map_err(Failure::Mutation)?;
        let mut extension = self
            .commit_extension
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ok_or(Failure::ForeignProgram)?;
        extension.envelope = Some(envelope_intent);
        extension.correction = context.correction;
        let capture = stages.first().expect("admitted nonempty prefix");
        let (completed, work) = capture
            .prepare_prefix_candidate::<I::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                phase,
                context.application,
                unseen,
            )
            .map_err(|cause| {
                Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)),
                ))
            })?;
        let execution = phase
            .checked_execution_request_for(context.application.product_runtime())
            .map_err(|cause| {
                Failure::Mutation(
                    WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                )
            })?;
        let prepared = request
            .prepare_resolved_envelope_candidate(
                phase,
                identities,
                extension,
                completed,
                work,
                execution.allocation_policy(),
            )
            .map_err(Failure::Mutation)?;
        let candidate = match prepared {
            CandidatePreparation::Prepared(candidate) => candidate,
            CandidatePreparation::Settled(outcome) => {
                return Ok(super::super::outcome::settled(
                    ordinal,
                    stage_identity,
                    I::Binding::IDENTITY,
                    brand,
                    outcome,
                ))
            }
        };
        let execution = phase
            .checked_execution_request_for(context.application.product_runtime())
            .map_err(|cause| {
                Failure::Mutation(
                    WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                )
            })?;
        let captured = R::capture(
            candidate.program.with_authored_edit_stages(
                &worth_query_execution::publication_boundary::program_publication_access(),
                stages,
            ),
            &execution,
            &self.input,
            &owner,
        )?;
        let binding = candidate.extension.apply(
            WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&candidate.identities),
        );
        let (commit, result) = match &self.correction_source {
            Some(source) => {
                let action =
                    crate::application_entry::correction::publication::PreparedCorrectionAction {
                        application: context.application,
                        owner,
                        program: captured,
                        identities: candidate.identities,
                        result: candidate.result,
                        source: source.clone(),
                    };
                action.commit_envelope::<P, R>(
                    program,
                    phase,
                    binding,
                    unseen,
                    execution.allocation_policy(),
                )
            }
            None => (
                R::commit(
                    program,
                    phase,
                    &owner,
                    captured,
                    &candidate.identities,
                    None,
                    binding,
                    unseen,
                    execution.allocation_policy(),
                ),
                candidate.result,
            ),
        };
        match commit.landed() {
            Ok(worth_query_execution::facade::primary_graph::WorthQueryApplicationLandedCommit::Recovered(receipt)) => Ok(WorthQueryEditEnvelopeOutcome::RecoveredCommitted(receipt)),
            Ok(worth_query_execution::facade::primary_graph::WorthQueryApplicationLandedCommit::Live { receipt }) => {
                *values[ordinal].lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Box::new(RetainedAuthoredStageResult { result, identity: stage_identity, receipt: Some(receipt.clone()) }));
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
