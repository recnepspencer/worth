//! Terminal stops that retain the consequence handoff for inspection and retry.
use super::{
    restore_query_from_facts, UiIntentConsequenceHandoff, UiIntentConsequencePublicationOutcome,
    UiIntentConsequenceStopReason, WorthUiActiveApplicationSession,
};

impl WorthUiActiveApplicationSession {
    pub(super) fn stop_intent_consequence(
        &mut self,
        handoff: UiIntentConsequenceHandoff,
        reason: UiIntentConsequenceStopReason,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        UiIntentConsequencePublicationOutcome::Stopped(
            self.intent_execution
                .retain_consequence_handoff(handoff, reason),
        )
    }

    pub(super) fn stop_intent_consequence_from_facts(
        &mut self,
        mut handoff: UiIntentConsequenceHandoff,
        reason: UiIntentConsequenceStopReason,
        facts: Box<[crate::fact_contract::UiProducedFact]>,
    ) -> UiIntentConsequencePublicationOutcome<'_> {
        restore_query_from_facts(&mut handoff, facts);
        self.stop_intent_consequence(handoff, reason)
    }
}
