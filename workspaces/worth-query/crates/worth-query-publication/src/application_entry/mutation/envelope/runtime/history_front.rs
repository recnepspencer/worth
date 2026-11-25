//! A history document sees the complete envelope intent before any stage is admitted.
use super::*;
use super::CorrectionSource;
use crate::application_entry::correction::WorthQueryApplicationCorrectionDenial;
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
    /// A correction envelope's corrective stage counts even before any
    /// caller stage is pushed (review note 1).
    pub(in crate::application_entry) fn has_stages(&self) -> bool {
        !self.envelope.envelope.stages.entries.is_empty() || self.front.is_some()
    }

    /// Execute once, letting the history owner decide first. The owner may
    /// retarget the envelope to its private candidate, or stop before any
    /// stage is admitted; a stop executes nothing.
    ///
    /// A correction envelope reads its corrective stage's meaning at the head
    /// first (`head`, read-only), so the complete intent is known before the
    /// owner reserves its record. Once the owner has retargeted the envelope
    /// to its candidate, the correction front derives again there and its
    /// stage leads the envelope; a different meaning there commits nothing.
    pub(in crate::application_entry) fn execute_with_history_front(
        self,
        allocation: ExecutionAllocationPolicy<'_, '_>,
        navigation: Option<&dyn worth_query_execution::facade::primary_graph::WorthQueryCorrectionNavigationLineage>,
        history: impl FnOnce(
            &mut EnvelopeContext<'_, S>,
            &WorthQueryAdvancementPhase<'_>,
            CapturedEnvelopeIntent,
            Option<Result<CorrectionSource, WorthQueryApplicationCorrectionDenial>>,
        ) -> ControlFlow<()>,
    ) -> Result<WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeExecutionDenial>
    where
        K: serde::Serialize + Send + Sync + 'static,
    {
        let Self {
            envelope:
                WorthQueryKeyedEditEnvelope {
                    envelope:
                        WorthQueryEditEnvelope {
                            context,
                            stages,
                            brand,
                        },
                    key,
                },
            program,
            front,
            head,
            ..
        } = self;
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
            program,
            allocation,
            AuthoredEnvelopeResolutionPurpose::Execute,
            move |context, program, phase, key, entries| {
                let intent_denial = |cause| {
                    WorthQueryEditEnvelopeExecutionDenial::envelope(
                        P::IDENTITY.as_str(),
                        crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
                            WorthQueryApplicationRequestMutationDenial::Identity(
                                ApplicationMutationIdentityDenial::Input(cause),
                            ),
                        ),
                    )
                };
                let head = head.map(|head| head(context, phase));
                stages.capture(entries)?;
                let mut intents = entries
                    .iter()
                    .map(|entry| entry.intent())
                    .collect::<Vec<_>>();
                let mut first_operation = entries.first().map(|entry| entry.descriptor());
                let head = head.map(|head| {
                    head.map(|corrective| {
                        intents.insert(0, corrective.intent);
                        first_operation = Some(corrective.operation);
                        corrective.source
                    })
                });
                let Some(first_operation) = first_operation else {
                    return Ok(Some(WorthQueryEditEnvelopeOutcome::Empty));
                };
                let intent = key.intent(context.order, &intents).map_err(intent_denial)?;
                let captured = CapturedEnvelopeIntent {
                    key: *key.identity(),
                    intent,
                    first_operation,
                };
                if let ControlFlow::Break(()) = history(context, phase, captured, head) {
                    // Nothing ran. The history owner holds the reason and reports it.
                    return Ok(Some(WorthQueryEditEnvelopeOutcome::Unseen));
                }
                let Some(front) = front else {
                    return Ok(None);
                };
                // The candidate pass: the owner retargeted `context` to its
                // candidate, where the correction derives and admits again.
                let mut corrective = Vec::new();
                if let Some(outcome) = front(context, program, phase, key, &mut corrective, navigation)? {
                    return Ok(Some(outcome));
                }
                entries.splice(0..0, corrective);
                let intents = entries
                    .iter()
                    .map(|entry| entry.intent())
                    .collect::<Vec<_>>();
                let candidate = key.intent(context.order, &intents).map_err(intent_denial)?;
                if candidate.identity() != intent.identity() {
                    return Err(WorthQueryEditEnvelopeExecutionDenial::envelope(
                        P::IDENTITY.as_str(),
                        crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Correction(
                            Box::new(WorthQueryApplicationCorrectionDenial::CandidateMeaningChanged),
                        ),
                    ));
                }
                Ok(None)
            },
        )
    }
}
