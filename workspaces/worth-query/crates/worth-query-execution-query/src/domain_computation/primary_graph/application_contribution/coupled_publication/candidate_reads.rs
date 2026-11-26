//! The name reads decisions of this assembly took on its unpublished
//! candidate: group schemes read from its committed basis (#361) and names
//! and births read from the envelope's own pending names (#426). Neither has
//! a Query fact key that a later member's append would change, so every
//! append is judged against both here, and one it falsifies refuses the
//! envelope with no effect.
use super::*;

mod group_reads;
mod pending_reads;
pub(super) use pending_reads::ReadOwner;

/// Every candidate name read this assembly's decisions retained.
#[derive(Default)]
pub(super) struct CandidateReads {
    pub(super) groups: group_reads::GroupReads,
    pub(super) pending: pending_reads::PendingReads,
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Refuse an append that falsifies a candidate name read a decision of
    /// this assembly retained.
    pub(super) fn refuse_stale_candidate_reads(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        self.refuse_claimed_group_reads(candidate)?;
        self.refuse_changed_pending_reads(execution, candidate)
    }
}
