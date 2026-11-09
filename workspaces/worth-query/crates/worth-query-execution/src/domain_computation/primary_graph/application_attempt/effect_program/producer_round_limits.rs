//! Carry entry resource controls through the actual sealed source program.
use super::WorthQueryApplicationEffectProgram;
use std::num::NonZeroUsize;
impl<S, O, I, Scope> WorthQueryApplicationEffectProgram<S, O, I, Scope> {
    pub fn with_producer_round_limits(
        mut self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        per_route: NonZeroUsize,
        per_attempt: NonZeroUsize,
    ) -> Self {
        self.producer_round_limits = (per_route, per_attempt);
        self
    }
}
