use crate::domain_computation::application_outcome_identity::WorthQueryApplicationOutcomeIdentity;

/// Runtime-assigned identity of one application commit outcome, read from a
/// receipt's `outcome_identity()`.
///
/// It correlates a committed attempt with the dispatch outbox records and
/// external-effect identities derived from it. It is a nonzero number that stays
/// unique across checkpoint restore. It names an outcome and grants nothing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryApplicationCommitOutcomeIdentity(WorthQueryApplicationOutcomeIdentity);

impl WorthQueryApplicationCommitOutcomeIdentity {
    pub(in crate::domain_computation::primary_graph) fn mint() -> Option<Self> {
        WorthQueryApplicationOutcomeIdentity::mint().map(Self)
    }

    pub(in crate::domain_computation::primary_graph) fn restore(value: u64) -> Option<Self> {
        WorthQueryApplicationOutcomeIdentity::restore(value).map(Self)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}
