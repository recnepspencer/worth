//! Finite producer-start policy carried by one mutation request.
use std::num::NonZeroUsize;
/// Bounds actual synchronous producer starts, including verification runs.
/// These limits confer no permission and do not change the mutation identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationProducerRoundLimits {
    per_route: NonZeroUsize,
    per_attempt: NonZeroUsize,
}
impl ApplicationProducerRoundLimits {
    pub const fn new(per_route: NonZeroUsize, per_attempt: NonZeroUsize) -> Self {
        Self {
            per_route,
            per_attempt,
        }
    }
    pub const fn per_route(self) -> NonZeroUsize {
        self.per_route
    }
    pub const fn per_attempt(self) -> NonZeroUsize {
        self.per_attempt
    }
}
impl Default for ApplicationProducerRoundLimits {
    fn default() -> Self {
        let (per_route, per_attempt) = worth_query_execution::publication_boundary::synchronous_outputs::default_producer_round_bounds();
        Self::new(per_route, per_attempt)
    }
}
