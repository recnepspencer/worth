//! Query half of R3 settlement: a consumer that was not re-run keeps its
//! consumption when the chain's later round publishes exactly what it read.
use super::*;

impl super::super::WorthQueryPrimaryGraphProvider {
    /// Whether `replacement`'s round settles `consumer`'s consumption of the
    /// same route; with `settle`, the consumption takes it as its replacement.
    pub fn settle_consumed_output(
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
        // A consumer of a committed (Recorded) output names its parent by
        // semantic source, not by this route's writer, and has no settlement
        // transition: it is reopened and reads the replacement again.
        if consumer
            .consumed_outputs()
            .iter()
            .any(|consumed| consumed.recorded_evidence())
        {
            return false;
        }
        // Every output the consumer's own derivation read from this route
        // must be exactly what the replacement publishes. A consumer that
        // named the route only as its prerequisite read none: nothing on the
        // Query side changes, and Native still requires exact equality of
        // what its retained input consumed.
        let mut links = consumer
            .consumed_outputs()
            .iter()
            .filter_map(|consumed| consumed.applied())
            .filter(|link| link.reads_route(&outputs.producer, &outputs.meaning));
        links.all(|link| link.settles_to(replacement, settle))
    }
}
