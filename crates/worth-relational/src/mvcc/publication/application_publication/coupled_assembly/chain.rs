//! Recorded physical positions of one producer's succession chain.
use super::local_keys::LocalKeys;
use crate::identity::data::EntityId;
use crate::transactions::data::{
    outcomes::transaction_commit_error::TransactionCommitError, primitives::RecordRef,
};
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use std::{collections::BTreeSet, ops::Range};

pub(super) type Read = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

/// One route's chain in the candidate. Positions are physical: transaction
/// batches and patch fragments are only ever appended before completion.
pub(crate) struct Chain {
    /// The candidate read immediately before the chain's head appended.
    pub(crate) head_input: Read,
    /// Cumulative certified touches of superseded rounds of this chain.
    pub(crate) prior_touches:
        Vec<crate::mvcc::validation::proposal_touches::ValidatedMutationTouch>,
    /// The latest round's transaction batches.
    pub(crate) batches: Range<usize>,
    /// Earlier rounds' batches, withdrawn once at completion.
    pub(crate) superseded: Vec<Range<usize>>,
    /// The chain's patch fragments, one per record it changed, each with the
    /// candidate read from before the chain first wrote that record.
    pub(crate) fragments: Vec<(usize, Read)>,
    /// The scoped coordinates the head chose; every round reuses them.
    pub(crate) local_keys: LocalKeys,
    /// The records the head allocated; every round re-creates exactly these.
    pub(crate) created: Vec<EntityId>,
    /// Each round's span of the changed-record log, head first, ascending.
    pub(crate) rounds: Vec<Range<usize>>,
    /// The candidate-wide append ordinal of the latest round: rounds order by
    /// it even when one appended no batch.
    pub(crate) latest: usize,
}

impl Chain {
    /// The first record in `own` that a member outside this chain changed
    /// since the chain's head: every log entry from the head's span on,
    /// outside the chain's own rounds (§1.5 fold condition 5).
    pub(crate) fn intervened<'a>(
        &self,
        changed: &'a [RecordRef],
        own: &BTreeSet<&RecordRef>,
        charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
    ) -> Result<Option<&'a RecordRef>, TransactionCommitError> {
        let start = self.rounds.first().map_or(changed.len(), |head| head.start);
        let mut rounds = self.rounds.iter().peekable();
        for (index, record) in changed.iter().enumerate().skip(start) {
            charge(1)?;
            while rounds.peek().is_some_and(|round| round.end <= index) {
                rounds.next();
            }
            if rounds.peek().is_some_and(|round| round.contains(&index)) {
                continue;
            }
            if own.contains(record) {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }
}
