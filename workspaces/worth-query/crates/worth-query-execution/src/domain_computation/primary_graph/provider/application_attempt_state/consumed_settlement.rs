//! Query half of R3 settlement: a consumer that was not re-run keeps its
//! consumption when the chain's later round publishes exactly what it read.
use super::*;

impl super::super::WorthQueryPrimaryGraphProvider {
    /// Whether `replacement`'s round settles `consumer`'s consumption of the
    /// same route; with `settle`, the consumption takes it as its replacement.
    pub(in crate::domain_computation::primary_graph) fn settle_consumed_output(
        &self,
        consumer: WorthQueryProviderSessionView<'_>,
        replacement: WorthQueryProviderSessionView<'_>,
        settle: bool,
    ) -> bool {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let replacement = attempts
            .attempt_state(replacement)
            .expect("actual replacement writer")
            .attempt();
        let consumer = attempts
            .attempt_state(consumer)
            .expect("actual consumer writer")
            .attempt();
        let Some(outputs) = replacement
            .coupled_preparation()
            .and_then(|preparation| preparation.outputs.as_ref())
        else {
            return false;
        };
        consumer
            .consumed_outputs()
            .iter()
            .filter_map(|consumed| consumed.applied())
            .find(|link| link.reads_route(&outputs.producer, &outputs.meaning))
            .is_some_and(|link| link.settles_to(replacement, settle))
    }
}
