//! One producer route's succession chain in the candidate (#119).
use super::local_keys::LocalKeys;
use super::succession::{ledger::ChainCreateLedger, segment::MemberSpan};
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use contracts::identity::data::primitives::PartitionId;
use contracts::transactions::data::primitives::RecordRef;
use std::{collections::BTreeSet, ops::Range};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_touches::ValidatedMutationTouch;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionMutationJournal;
use worth_relational_storage::internal_port::storage::substrate::SharedMap;

pub(super) type Read = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

/// One route's chain. Its live round's entries are `span`; every superseded
/// round was retracted to the head's image and left no entry behind.
pub struct Chain {
    /// The candidate read immediately before the chain's head appended (`B`).
    pub head_input: Read,
    /// The candidate's journal at the head: which records and anchors `B`
    /// already changed against the basis.
    pub head_journal: SharedMap<PartitionId, PartitionMutationJournal>,
    /// The head's append ordinal: later members are the chain's interveners.
    pub head_ordinal: u64,
    /// The head's local-key scope: a new key a later round introduces is
    /// framed in it (`local_keys::rescope`).
    pub scope: usize,
    /// Exact touches of every superseded round, captured before retraction.
    pub prior_touches: Vec<ValidatedMutationTouch>,
    /// Exact touches of the round the live round superseded, plus the kind
    /// touches retracting it implies (§3.7 H-Q), for the append touches only.
    pub last_superseded: Vec<ValidatedMutationTouch>,
    /// The live round's transaction batches.
    pub batches: Range<usize>,
    /// Superseded rounds' batches, withdrawn once at completion.
    pub superseded: Vec<Range<usize>>,
    /// The scoped coordinates of every key a round of the chain used.
    pub local_keys: LocalKeys,
    /// Every key's identity, bound or dormant.
    pub ledger: ChainCreateLedger,
    /// Every record any round of the chain changed; it only grows.
    pub touched: BTreeSet<RecordRef>,
    /// The live round's entries in the candidate's accumulators.
    pub span: MemberSpan,
    /// How many rounds the chain has applied.
    pub rounds: usize,
    /// The candidate-wide round ordinal of the live round.
    pub latest: usize,
}
