//! Complete, individually admitted prefix meanings for one canonical envelope key.
use super::{WorthQueryApplicationIdempotencyBinding as Binding, WorthQueryRecordedIntentMatch};
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Denial;

pub struct EnvelopeCandidates<'a> {
    bindings: &'a [Binding],
}
impl<'a> EnvelopeCandidates<'a> {
    pub fn new(
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
    pub fn first(&self) -> Binding {
        self.bindings[0]
    }
    pub fn matches(
        &self,
        recorded: Binding,
    ) -> Result<bool, Denial> {
        if self.first().envelope_intent.is_none() {
            return Ok(recorded == self.first());
        }
        self.match_text(&recorded.intent_text())
    }
    pub fn match_text(
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

    /// The sealed Native/World receipt owns the already performed handler's
    /// definition. Reopening never reruns that handler to obtain its binding.
    /// Fresh request slots and an explicitly supplied definition still compare
    /// exactly, as in live managed recovery's bind_original_definition.
    pub fn match_performed(
        &self,
        receipt: &crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt,
    ) -> Result<bool, Denial> {
        self.match_performed_intent(receipt.recorded_intent())
    }

    fn match_performed_intent(&self, recorded: &str) -> Result<bool, Denial> {
        let original = Binding::observe_recorded_intent(recorded)?;
        let bindings = self
            .bindings
            .iter()
            .map(|binding| {
                if binding.conditional_definition_identity().is_none() {
                    binding.bind_conditional_definition(original.conditional.as_ref())
                } else {
                    *binding
                }
            })
            .collect::<Vec<_>>();
        EnvelopeCandidates::new(&bindings)?.match_text(recorded)
    }
}

#[cfg(test)]
mod tests;
