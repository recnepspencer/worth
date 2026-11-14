//! Stage order and tokens are request-owned; only execute performs work.
use super::*;
pub struct NoEditStages;
pub struct WithEditStage<Previous, Root, Intent> {
    pub(super) previous: Previous,
    pub(super) intent: Intent,
    pub(super) marker: PhantomData<fn() -> Root>,
}
pub struct WorthQueryEditEnvelope<'a, S, Stages> {
    pub(super) context: EnvelopeContext<'a, S>,
    pub(super) stages: Stages,
    pub(super) brand: Arc<()>,
}
pub struct WorthQueryKeyedEditEnvelope<'a, S, Stages, Key> {
    pub(super) envelope: WorthQueryEditEnvelope<'a, S, Stages>,
    pub(super) key: Key,
}
impl<'a, S: ApplicationSchema> WorthQueryApplicationRequest<'a, 'a, 'a, S> {
    pub fn edit_envelope(&self) -> WorthQueryEditEnvelope<'a, S, NoEditStages> {
        WorthQueryEditEnvelope {
            context: EnvelopeContext {
                application: self.application,
                principal: self.principal,
                scope: self.scope,
                branch: self.branch,
                order: ApplicationEditEnvelopeOrder::DeclaredDependencies,
                correction: None,
            },
            stages: NoEditStages,
            brand: Arc::new(()),
        }
    }
}
impl<'a, S: ApplicationSchema, Stages> WorthQueryEditEnvelope<'a, S, Stages> {
    pub fn stage<Root, Intent>(
        self,
        intent: Intent,
    ) -> WorthQueryEditEnvelope<'a, S, WithEditStage<Stages, Root, Intent>>
    where
        Intent: ApplicationMutationIntent<S> + Clone + Send + Sync + 'static,
    {
        WorthQueryEditEnvelope {
            context: self.context,
            brand: self.brand,
            stages: WithEditStage {
                previous: self.stages,
                intent,
                marker: PhantomData,
            },
        }
    }
    pub fn idempotency<Key>(self, key: Key) -> WorthQueryKeyedEditEnvelope<'a, S, Stages, Key> {
        WorthQueryKeyedEditEnvelope {
            envelope: self,
            key,
        }
    }
}
impl<S, Stages: EditStageList<S>, Key> WorthQueryKeyedEditEnvelope<'_, S, Stages, Key>
where
    S: ApplicationSchema,
{
    pub fn stage_token<B: ApplicationMutationBinding<S>>(
        &self,
        ordinal: usize,
    ) -> Option<WorthQueryEditStageToken<B::Result, B::Denial>> {
        self.envelope
            .stages
            .binding(ordinal)
            .filter(|binding| *binding == std::any::TypeId::of::<B>())
            .map(|_| WorthQueryEditStageToken {
                brand: self.envelope.brand.clone(),
                ordinal,
                marker: PhantomData,
            })
    }
    pub fn execute_performed<Program>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, Program>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeExecutionDenial>
    where
        Program: ApplicationProgramDefinition<S>,
        Stages: entry::CaptureEditStages<S, Program, Key>,
        Key: serde::Serialize + Send + Sync + 'static,
    {
        self.advance_with_front(
            program,
            allocation,
            AuthoredEnvelopeResolutionPurpose::Execute,
            |_, _, _, _, _| Ok(None),
        )
    }

    pub(in crate::application_entry) fn advance_with_front<Program>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, Program>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
        purpose: AuthoredEnvelopeResolutionPurpose,
        front: impl FnOnce(
            &mut EnvelopeContext<'_, S>,
            &WorthQueryProgramApplicationRuntime<S, Program>,
            &WorthQueryAdvancementPhase<'_>,
            &Arc<ApplicationEditEnvelopeKey<Key>>,
            &mut Vec<Box<dyn entry::EditEntry<S, Program, Key>>>,
        ) -> Result<
            Option<WorthQueryEditEnvelopeOutcome>,
            WorthQueryEditEnvelopeExecutionDenial,
        >,
    ) -> Result<WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeExecutionDenial>
    where
        Program: ApplicationProgramDefinition<S>,
        Stages: entry::CaptureEditStages<S, Program, Key>,
        Key: serde::Serialize + Send + Sync + 'static,
    {
        let envelope_failure = |cause| {
            WorthQueryEditEnvelopeExecutionDenial::envelope(Program::IDENTITY.as_str(), cause)
        };
        let mut context = self.envelope.context;
        context.application.with_funded_application_advancement(
            context.scope,
            allocation,
            |phase| {
                let key = Arc::new(
                    ApplicationEditEnvelopeKey::encode(Program::IDENTITY.as_str(), self.key)
                        .map_err(|cause| crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                            WorthQueryApplicationRequestMutationDenial::Identity(
                                ApplicationMutationIdentityDenial::Key(cause),
                            ),
                        ))
                        .map_err(envelope_failure)?,
                );
                let mut entries = Vec::new();
                if let Some(outcome) = front(&mut context, program, &phase, &key, &mut entries)? {
                    return Ok(outcome);
                }
                self.envelope.stages.capture(&mut entries)?;
                if entries.is_empty() {
                    return Ok(WorthQueryEditEnvelopeOutcome::Empty);
                }
                let intents = entries.iter().map(|entry| entry.intent()).collect::<Vec<_>>();
                let intent = key.intent(context.order, &intents)
                    .map_err(|cause| crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                        WorthQueryApplicationRequestMutationDenial::Identity(
                            ApplicationMutationIdentityDenial::Input(cause),
                        ),
                    ))
                    .map_err(envelope_failure)?;
                let values = entries.iter().map(|_| Arc::new(Mutex::new(None))).collect::<Vec<StageValue>>();
                let mut captured = Vec::new();
                for (ordinal, entry) in entries.iter().enumerate() {
                    captured.push(
                        entry.secondary(&context, program, &phase, &key, ordinal, values[ordinal].clone())
                            .map_err(|cause| WorthQueryEditEnvelopeExecutionDenial::stage(
                                ordinal, entry.binding(), cause,
                            ))?,
                    );
                }
                if let Some(stage) = captured.iter().skip(1)
                    .find(|stage| !stage.same_admitted_basis(&captured[0]))
                {
                    return Ok(WorthQueryEditEnvelopeOutcome::AdmissionBasisChanged(stage.subject()));
                }
                let admissions = captured.clone();
                with_resolved_edit_envelope(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    &phase, context.application, &admissions, intent, purpose,
                    |resolution| {
                        let unseen = match resolution {
                            AuthoredEnvelopeResolution::Recorded(resolution) => return super::replay::recorded(resolution).map_err(envelope_failure),
                            AuthoredEnvelopeResolution::Unseen(unseen) => unseen,
                        };
                if purpose == AuthoredEnvelopeResolutionPurpose::ResolveOnly {
                    return Ok(WorthQueryEditEnvelopeOutcome::Unseen);
                }
                let order = worth_query_execution::publication_boundary::authored_edit::order_authored_stages(
                    &phase.checked_execution_request_for(context.application.product_runtime())
                        .map_err(|cause| crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                        ))
                        .map_err(envelope_failure)?,
                    &mut captured,
                    &phase, context.application, &unseen,
                );
                let order = match order {
                    Ok(order) => order,
                    Err(DeclaredEditOrderRefusal::Planning(cause)) => return Err(envelope_failure(crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)))))),
                    Err(DeclaredEditOrderRefusal::Execution(cause)) => return Err(envelope_failure(
                        crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                            WorthQueryApplicationRequestMutationDenial::Handler(
                                MutationHandlerExecutionDenial::Attempt(cause),
                            ),
                        ),
                    )),
                    Err(DeclaredEditOrderRefusal::Cycle(cycle)) => return Ok(WorthQueryEditEnvelopeOutcome::CyclicStages(cycle)),
                    Err(DeclaredEditOrderRefusal::StageAfterProducer(dependency)) => return Ok(WorthQueryEditEnvelopeOutcome::StageAfterProducer(dependency)),
                };
                let prefix = order[0];
                let secondary = order.iter().map(|ordinal| captured[*ordinal].clone()).collect();
                entries[prefix].execute_prefix(
                    &context, program, &phase, &key, prefix, &unseen, intent, secondary, values, self.envelope.brand,
                ).map_err(|cause| WorthQueryEditEnvelopeExecutionDenial::stage(
                    prefix, entries[prefix].binding(), cause,
                ))
                    },
                ).map_err(|cause| envelope_failure(crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(WorthQueryApplicationRequestMutationDenial::Idempotency(cause))))?
            },
        ).unwrap_or_else(|cause| Err(envelope_failure(
            crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
            ),
        )))
    }
}
