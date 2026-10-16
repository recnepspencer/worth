//! One native event resumes the actual retained Query owner using fresh request authority.
use super::{PlatformPulseApplicationRuntime, PlatformPulseTerminalError};
use crate::query_source::{PlatformPulseQueryContinuation, PlatformPulseQueryLifecycleDenial};
mod closing;
impl PlatformPulseApplicationRuntime {
    pub(super) fn advance_query_after_native_tick(&mut self) {
        if self
            .query_lifecycle
            .as_ref()
            .is_some_and(|query| query.has_pending_mutation())
        {
            self.advance_native_product_turn();
        }
    }

    pub(super) fn resume_query_mutation(&mut self) -> bool {
        if self.pending_query_request.as_ref().is_some_and(|original| {
            original.before_effect()
                && (original.cancellation_requested() || self.external_close_requested)
        }) {
            return self.cancel_retained_query_request();
        }
        // After begin_effect, cancellation cannot promise abort. The existing provider
        // explicitly accepts a genuine completion from the SAME in-flight request.
        let Some(query) = self.query_lifecycle.as_mut() else {
            return false;
        };
        if !query.has_pending_mutation() {
            return self.pending_query_request.is_none();
        }
        #[cfg(test)]
        if let Some(probe) = &self.query_probe {
            probe.borrow_mut().attempts += 1;
        }
        let result = query.continue_mutation(std::num::NonZeroUsize::MAX);
        match result {
            Err(PlatformPulseQueryLifecycleDenial::MutationPending) => false,
            Err(denial) => {
                self.fail(PlatformPulseTerminalError::QueryLifecycle(denial), Ok(()));
                false
            }
            Ok(PlatformPulseQueryContinuation::Source(observation)) => {
                self.publish_query_observation(observation, false);
                true
            }
            Ok(PlatformPulseQueryContinuation::Action(outcome)) => {
                let Some(original) = self.pending_query_request.take() else {
                    self.fail_intent_settlement(
                        "resumed Query action lost its original Intent request",
                    );
                    return false;
                };
                self.settle_query_action(original, Ok(outcome))
            }
        }
    }
}
