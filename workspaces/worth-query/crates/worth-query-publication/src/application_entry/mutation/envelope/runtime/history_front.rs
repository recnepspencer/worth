//! A history document sees the complete envelope intent before any stage is admitted.
use super::*;
use std::ops::ControlFlow;
use worth_query_declaration::facade::application_operation::ApplicationMutationBindingDescriptor;

/// What the whole envelope means: its caller key, its complete ordered intent,
/// and the first stage's operation.
pub(in crate::application_entry) struct CapturedEnvelopeIntent {
    pub(in crate::application_entry) key: [u8; 32],
    pub(in crate::application_entry) intent: ApplicationEditIntentIdentity,
    pub(in crate::application_entry) first_operation: ApplicationMutationBindingDescriptor,
}

impl<S, P, K> WorthQueryRuntimeEditEnvelope<'_, '_, S, P, K>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) fn has_stages(&self) -> bool {
        !self.envelope.envelope.stages.entries.is_empty()
    }

    /// Execute once, letting the history owner decide first. The owner may
    /// retarget the envelope to its private candidate, or stop before any
    /// stage is admitted; a stop executes nothing.
    pub(in crate::application_entry) fn execute_with_history_front(
        self,
        allocation: ExecutionAllocationPolicy<'_, '_>,
        history: impl FnOnce(
            &mut EnvelopeContext<'_, S>,
            &WorthQueryAdvancementPhase<'_>,
            CapturedEnvelopeIntent,
        ) -> ControlFlow<()>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeExecutionDenial>
    where
        K: serde::Serialize + Send + Sync + 'static,
    {
        let WorthQueryKeyedEditEnvelope {
            envelope:
                WorthQueryEditEnvelope {
                    context,
                    stages,
                    brand,
                },
            key,
        } = self.envelope;
        // The authored stages are captured inside the front, so the intent is
        // complete before the history owner reserves its one record.
        let keyed = WorthQueryKeyedEditEnvelope {
            key,
            envelope: WorthQueryEditEnvelope {
                context,
                brand,
                stages: RuntimeEditStages {
                    entries: Vec::new(),
                },
            },
        };
        keyed.advance_with_front(
            self.program,
            allocation,
            AuthoredEnvelopeResolutionPurpose::Execute,
            move |context, _, phase, key, entries| {
                stages.capture(entries)?;
                let Some(first_operation) = entries.first().map(|entry| entry.descriptor()) else {
                    return Ok(Some(WorthQueryEditEnvelopeOutcome::Empty));
                };
                let intents = entries
                    .iter()
                    .map(|entry| entry.intent())
                    .collect::<Vec<_>>();
                let intent = key.intent(context.order, &intents).map_err(|cause| {
                    WorthQueryEditEnvelopeExecutionDenial::envelope(
                        P::IDENTITY.as_str(),
                        crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                            WorthQueryApplicationRequestMutationDenial::Identity(
                                ApplicationMutationIdentityDenial::Input(cause),
                            ),
                        ),
                    )
                })?;
                let captured = CapturedEnvelopeIntent {
                    key: *key.identity(),
                    intent,
                    first_operation,
                };
                match history(context, phase, captured) {
                    ControlFlow::Continue(()) => Ok(None),
                    // Nothing ran. The history owner holds the reason and reports it.
                    ControlFlow::Break(()) => Ok(Some(WorthQueryEditEnvelopeOutcome::Unseen)),
                }
            },
        )
    }
}
