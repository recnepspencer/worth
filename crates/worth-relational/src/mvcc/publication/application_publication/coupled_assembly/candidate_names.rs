//! A later envelope member reads the names earlier members give their
//! records (#426): the composed plan so far, previewed by the commit's own
//! naming authority over the assembly's applied state.
use super::*;
use crate::lineage::data::CommitNamingPlan;
use crate::validation::custom_rule::CustomInvariantNameSource;
use worth_relational_publication::internal_port::validation::invariant_names::{
    ProposalNameCarry, ProposalNameSource,
};

/// The composed plan and its settled preview, kept for the one pin they
/// describe: a later pin is another applied state and composes afresh.
#[derive(Default)]
pub(crate) struct CandidateNameCache(
    Option<(
        RelationalAppliedCandidateReadIdentity,
        Option<CommitNamingPlan>,
        ProposalNameCarry,
    )>,
);

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Runs `read` over the names a member reading `candidate` sees the
    /// envelope give its records: the authored prefix's plan, then each
    /// applied stage's, then each chain's live round (a superseded round's
    /// plan is gone), composed as completion composes them. `None` unless
    /// `candidate` is exactly this assembly's current pin.
    ///
    /// Complexity: O(composed plan) on a pin's first read, which also runs
    /// the preview; O(1) to reopen it on every later read of that pin.
    pub fn with_candidate_names<T>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &RelationalAppliedCandidateReadIdentity,
        read: impl FnOnce(&dyn CustomInvariantNameSource) -> T,
    ) -> Result<Option<T>, TransactionCommitError> {
        let current = self.read.borrow();
        if current
            .as_ref()
            .map(RelationalPinnedRead::candidate_identity)
            != Some(candidate)
        {
            return Ok(None);
        }
        let mut cache = self.candidate_names.borrow_mut();
        if cache.0.as_ref().is_none_or(|(kept, ..)| kept != candidate) {
            let stages = self.stages.iter().map(|stage| stage.naming.clone());
            let parts = self.parts.iter().map(|part| part.naming.clone());
            let root = self.transaction.naming_plan.as_deref();
            let plan = part_naming::compose(execution, root, stages.chain(parts))?;
            cache.0 = Some((candidate.clone(), plan, ProposalNameCarry::default()));
        }
        let (_, plan, kept) = cache.0.as_mut().expect("composed for this pin above");
        let names = ProposalNameSource::for_coupled_candidate(
            self.runtime.lineage_preparation_authority(),
            self.binding.owners.before(),
            self.mutation,
            self.prepared.plan.view(),
            self.identity,
            plan.as_ref(),
            std::mem::take(kept),
        );
        let answer = read(&names);
        *kept = names.settle(Ok::<_, TransactionCommitError>(())).1;
        Ok(Some(answer))
    }
}
