use worth_ui_inspection::{
    UiIntentCausalTraceEvidence, UiIntentEvidenceLookup, UiIntentEvidenceReference,
    UiIntentEvidenceRetentionOmission, UiIntentEvidenceRetentionOutcome,
    UiIntentEvidenceRetirementCause, UiIntentEvidenceRetirementReport,
    UiIntentInteractionEvidenceInput, UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY,
};

#[cfg(any(test, feature = "certification-support"))]
use worth_ui_inspection::UiIntentInteractionEvidence;

use super::UiIntentEvidenceResourceSnapshot;

#[derive(Clone, Copy)]
struct UiIntentAdmissionEvidenceIndex {
    generation: u64,
    reference: UiIntentEvidenceReference,
}

pub(crate) struct UiIntentEvidenceRegistry {
    session: u64,
    slots: [Option<UiIntentCausalTraceEvidence>; UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY],
    input_index:
        std::collections::HashMap<UiIntentInteractionEvidenceInput, UiIntentEvidenceReference>,
    latest_reference: Option<UiIntentEvidenceReference>,
    admission_index: [Option<UiIntentAdmissionEvidenceIndex>;
        crate::runtime::intent_execution::UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS],
    next_slot: u8,
    next_generation: u64,
    retained: usize,
    replacements: u64,
    omissions: u64,
}

impl UiIntentEvidenceRegistry {
    pub(crate) fn new(session: u64) -> Self {
        Self {
            session,
            slots: [None; UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY],
            input_index: std::collections::HashMap::with_capacity(
                UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY,
            ),
            latest_reference: None,
            admission_index: [None;
                crate::runtime::intent_execution::UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS],
            next_slot: 0,
            next_generation: 1,
            retained: 0,
            replacements: 0,
            omissions: 0,
        }
    }

    pub(crate) fn retain_transitions(
        &mut self,
        transitions: &[crate::runtime::interaction::UiInteractionTransition],
    ) {
        for interaction in transitions
            .iter()
            .filter_map(|transition| match transition {
                crate::runtime::interaction::UiInteractionTransition::Semantic(interaction) => {
                    Some(interaction)
                }
                crate::runtime::interaction::UiInteractionTransition::PointerPressed(_)
                | crate::runtime::interaction::UiInteractionTransition::DraftMutation(_)
                | crate::runtime::interaction::UiInteractionTransition::DismissRequested(_)
                | crate::runtime::interaction::UiInteractionTransition::Stopped(_) => None,
            })
        {
            let input = crate::runtime::interaction::semantic_evidence_input(interaction);
            let _ = self.retain(input);
        }
    }

    pub(crate) fn retain_selection(
        &mut self,
        interaction: &crate::runtime::interaction::UiSelectionCommitInteraction,
    ) -> UiIntentEvidenceRetentionOutcome {
        self.retain(crate::runtime::interaction::selection_evidence_input(
            interaction,
        ))
    }

    pub(crate) fn retain_command_route(
        &mut self,
        input: UiIntentInteractionEvidenceInput,
    ) -> Option<UiIntentEvidenceReference> {
        if input.family() != worth_ui_inspection::UiIntentInteractionEvidenceFamily::CommandRoute {
            return None;
        }
        match self.retain(input) {
            UiIntentEvidenceRetentionOutcome::Retained(reference)
            | UiIntentEvidenceRetentionOutcome::Replaced {
                retained: reference,
                ..
            } => Some(reference),
            UiIntentEvidenceRetentionOutcome::Omitted(_) => None,
        }
    }

    pub(crate) fn snapshot(&self) -> UiIntentEvidenceResourceSnapshot {
        UiIntentEvidenceResourceSnapshot::new(
            self.retained,
            self.retained * core::mem::size_of::<UiIntentCausalTraceEvidence>(),
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn lookup(
        &self,
        reference: UiIntentEvidenceReference,
    ) -> Option<UiIntentInteractionEvidence> {
        let slot = usize::from(reference.slot());
        let evidence = self.slots.get(slot).copied().flatten()?;
        (evidence.reference() == reference).then(|| evidence.interaction_evidence())
    }

    pub(crate) fn lookup_trace(
        &self,
        reference: UiIntentEvidenceReference,
    ) -> UiIntentEvidenceLookup {
        if reference.session_diagnostic_value() != self.session {
            return UiIntentEvidenceLookup::ForeignSession;
        }
        let slot = usize::from(reference.slot());
        match self.slots.get(slot).copied().flatten() {
            Some(evidence) if evidence.reference() == reference => {
                UiIntentEvidenceLookup::Found(evidence)
            }
            _ => UiIntentEvidenceLookup::Expired,
        }
    }

    pub(crate) fn reference_for_input(
        &self,
        input: UiIntentInteractionEvidenceInput,
    ) -> Option<UiIntentEvidenceReference> {
        self.input_index.get(&input).copied()
    }

    pub(crate) fn record_admission(
        &mut self,
        prefix: crate::runtime::intent::UiIntentCausalTraceAdmissionPrefix,
        slot: crate::runtime::intent::UiIntentAdmissionSlotIdentity,
        lineage: crate::runtime::intent::UiIntentAttemptLineage,
    ) -> bool {
        let Some(trace) = self.trace_mut(prefix.reference) else {
            return false;
        };
        trace.record_admission(
            prefix.route,
            prefix.payload,
            prefix.operability,
            worth_ui_inspection::UiIntentCausalTraceAdmissionEvidence::new(
                slot.slot(),
                slot.generation(),
                lineage.diagnostic_value(),
            ),
        );
        self.admission_index[usize::from(slot.slot())] = Some(UiIntentAdmissionEvidenceIndex {
            generation: slot.generation(),
            reference: prefix.reference,
        });
        true
    }

    pub(crate) fn record_dispatch(
        &mut self,
        admission: crate::runtime::intent::UiIntentAdmissionSlotIdentity,
        dispatch: crate::runtime::intent_execution::UiIntentExecutionDispatchReceipt,
    ) -> Option<UiIntentEvidenceReference> {
        let indexed = self
            .admission_index
            .get(usize::from(admission.slot()))
            .copied()
            .flatten()?;
        if indexed.generation != admission.generation()
            || dispatch.attempt().slot() != admission.slot()
            || dispatch.attempt().generation() != admission.generation()
        {
            return None;
        }
        let trace = self.trace_mut(indexed.reference)?;
        trace.record_attempt(attempt_evidence(
            dispatch.attempt(),
            dispatch.idempotency(),
            worth_ui_inspection::UiIntentCausalTraceAttemptPosture::Prepared,
        ));
        Some(indexed.reference)
    }

    pub(crate) fn record_transitions(
        &mut self,
        transitions: &[crate::runtime::intent_execution::UiIntentExecutionTransition],
    ) {
        for transition in transitions {
            self.record_transition(transition);
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn latest_reference(&self) -> Option<UiIntentEvidenceReference> {
        self.latest_reference
    }

    pub(crate) fn retire(
        &mut self,
        cause: UiIntentEvidenceRetirementCause,
    ) -> UiIntentEvidenceRetirementReport {
        let snapshot = self.snapshot();
        self.slots = [None; UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY];
        self.input_index.clear();
        self.latest_reference = None;
        self.admission_index =
            [None; crate::runtime::intent_execution::UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS];
        self.retained = 0;
        let report = UiIntentEvidenceRetirementReport::new(
            cause,
            snapshot.retained_references(),
            snapshot.retained_bytes(),
            self.replacements,
            self.omissions,
        );
        self.replacements = 0;
        self.omissions = 0;
        report
    }

    fn retain(
        &mut self,
        input: UiIntentInteractionEvidenceInput,
    ) -> UiIntentEvidenceRetentionOutcome {
        let generation = self.next_generation;
        let Some(next_generation) = generation.checked_add(1) else {
            self.omissions = self.omissions.saturating_add(1);
            return UiIntentEvidenceRetentionOutcome::Omitted(
                UiIntentEvidenceRetentionOmission::IdentityExhausted,
            );
        };
        self.next_generation = next_generation;

        let slot = self.next_slot;
        self.next_slot =
            u8::try_from((usize::from(slot) + 1) % UI_INTENT_INTERACTION_EVIDENCE_ENTRY_CAPACITY)
                .expect("the evidence ring holds fewer than 256 entries");
        let reference =
            UiIntentEvidenceReference::from_diagnostic_parts(self.session, slot, generation);
        let evidence = UiIntentCausalTraceEvidence::from_interaction(reference, input);
        let expired = self.slots[usize::from(slot)].replace(evidence);
        self.remove_expired_index(expired);
        self.input_index.insert(input, reference);
        self.latest_reference = Some(reference);
        match expired {
            Some(expired) => {
                self.replacements = self.replacements.saturating_add(1);
                UiIntentEvidenceRetentionOutcome::Replaced {
                    retained: reference,
                    expired: expired.reference(),
                }
            }
            None => {
                self.retained += 1;
                UiIntentEvidenceRetentionOutcome::Retained(reference)
            }
        }
    }

    fn remove_expired_index(&mut self, expired: Option<UiIntentCausalTraceEvidence>) {
        let Some(expired) = expired else {
            return;
        };
        let input = expired.interaction();
        if self.input_index.get(&input).copied() == Some(expired.reference()) {
            self.input_index.remove(&input);
        }
    }

    fn trace_mut(
        &mut self,
        reference: UiIntentEvidenceReference,
    ) -> Option<&mut UiIntentCausalTraceEvidence> {
        let trace = self
            .slots
            .get_mut(usize::from(reference.slot()))?
            .as_mut()?;
        (trace.reference() == reference).then_some(trace)
    }

    fn record_transition(
        &mut self,
        transition: &crate::runtime::intent_execution::UiIntentExecutionTransition,
    ) {
        let attempt = transition.attempt();
        let Some(indexed) = self
            .admission_index
            .get(usize::from(attempt.slot()))
            .copied()
            .flatten()
        else {
            return;
        };
        if indexed.generation != attempt.generation() {
            return;
        }
        let posture = trace_posture(transition.posture());
        let Some(trace) = self.trace_mut(indexed.reference) else {
            return;
        };
        trace.record_attempt(attempt_evidence(attempt, transition.idempotency(), posture));
        if let crate::runtime::intent_execution::UiIntentExecutionTransitionPosture::Completed {
            outcome,
        } = transition.posture()
        {
            trace.record_completion(
                worth_ui_inspection::UiIntentCausalTraceCompletionEvidence::new(
                    schema_digest(outcome),
                    true,
                ),
            );
        }
    }
}

fn attempt_evidence(
    attempt: crate::runtime::intent_execution::UiIntentExecutionAttemptIdentity,
    idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    posture: worth_ui_inspection::UiIntentCausalTraceAttemptPosture,
) -> worth_ui_inspection::UiIntentCausalTraceAttemptEvidence {
    worth_ui_inspection::UiIntentCausalTraceAttemptEvidence::new(
        attempt.slot(),
        attempt.generation(),
        idempotency.session(),
        idempotency.lineage(),
        posture,
    )
}

fn trace_posture(
    posture: crate::runtime::intent_execution::UiIntentExecutionTransitionPosture,
) -> worth_ui_inspection::UiIntentCausalTraceAttemptPosture {
    use crate::runtime::intent_execution::UiIntentExecutionTransitionPosture as Runtime;
    use worth_ui_inspection::UiIntentCausalTraceAttemptPosture as Trace;
    match posture {
        Runtime::Started => Trace::Started,
        Runtime::PendingBeforeEffect => Trace::PendingBeforeEffect,
        Runtime::PendingEffectMayHaveBegun => Trace::PendingEffectMayHaveBegun,
        Runtime::Completed { .. } => Trace::Completed,
        Runtime::RejectedBeforeEffect { .. }
        | Runtime::FailedBeforeEffect { .. }
        | Runtime::CancelledBeforeEffect { .. }
        | Runtime::TimedOutBeforeEffect { .. } => Trace::StoppedBeforeEffect,
        Runtime::Partial { .. } => Trace::Partial,
        Runtime::Indeterminate { .. } => Trace::Indeterminate,
    }
}

fn schema_digest(schema: crate::capability::UiIntentSchema) -> u64 {
    let digest = schema
        .stable_identity()
        .bytes()
        .fold(0xcbf29ce484222325, |mut digest, byte| {
            digest ^= u64::from(byte);
            digest.wrapping_mul(0x100000001b3)
        });
    digest ^ u64::from(schema.version())
}
