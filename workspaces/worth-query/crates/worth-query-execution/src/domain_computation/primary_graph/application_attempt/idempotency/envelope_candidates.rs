//! Complete, individually admitted prefix meanings for one canonical envelope key.
use super::{WorthQueryApplicationIdempotencyBinding as Binding, WorthQueryRecordedIntentMatch};
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Denial;

pub(in crate::domain_computation::primary_graph) struct EnvelopeCandidates<'a> {
    bindings: &'a [Binding],
}
impl<'a> EnvelopeCandidates<'a> {
    pub(in crate::domain_computation::primary_graph) fn new(
        bindings: &'a [Binding],
    ) -> Result<Self, Denial> {
        let first = bindings.first().ok_or(Denial::ForeignAffinity)?;
        if bindings.iter().any(|binding| {
            binding.key_identity != first.key_identity
                || binding.envelope_intent != first.envelope_intent
        }) || (bindings.len() != 1 && first.envelope_intent.is_none())
        {
            return Err(Denial::ForeignAffinity);
        }
        Ok(Self { bindings })
    }
    pub(in crate::domain_computation::primary_graph) fn first(&self) -> Binding {
        self.bindings[0]
    }
    pub(in crate::domain_computation::primary_graph) fn matches(
        &self,
        recorded: Binding,
    ) -> Result<bool, Denial> {
        if self.first().envelope_intent.is_none() {
            return Ok(recorded == self.first());
        }
        self.match_text(&recorded.intent_text())
    }
    pub(in crate::domain_computation::primary_graph) fn match_text(
        &self,
        recorded: &str,
    ) -> Result<bool, Denial> {
        let mut matched = None;
        let mut unverifiable = false;
        for binding in self.bindings {
            match binding.match_recorded_intent(recorded)? {
                WorthQueryRecordedIntentMatch::Same => {
                    if matched.is_some() {
                        return Err(Denial::RecordedIntentUnverifiable);
                    }
                    matched = Some(*binding);
                }
                WorthQueryRecordedIntentMatch::Unverifiable => unverifiable = true,
                WorthQueryRecordedIntentMatch::Drift => {}
            }
        }
        if matched.is_some() {
            return Ok(true);
        }
        let original = Binding::observe_recorded_intent(recorded)?;
        // Equal envelope DATA never replaces an original nominal prefix proof.
        if unverifiable
            || (self.first().envelope_intent.is_some()
                && (original.envelope.is_none()
                    || original.envelope == self.first().envelope_intent))
        {
            return Err(Denial::RecordedIntentUnverifiable);
        }
        Ok(false)
    }
}
