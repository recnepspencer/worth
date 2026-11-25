//! A successor round becomes its chain's live round, and superseded rounds'
//! batches are withdrawn once at completion (#119).
use super::super::super::custody;
use super::super::producer_posture::RelationalCoupledProducerPosture;
use super::super::retained_assembly::CoupledPublicationPart;
use super::ledger::ChainCreate;
use super::segment::MemberSpan;
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use contracts::execution::request_stage::checkpoint;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::{RecordRef, WorkerIntentBatch};
use std::ops::Range;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_touches::ValidatedMutationTouch;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_touches::ValidatedMutationTouch as Touch;
use worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments;

/// Withdraw every superseded round's batches before final composition, so the
/// final command set holds one round per chain. Reads stay in the footprint.
pub fn batches_after_withdrawal(
    execution: &ExecutionRequest<'_, '_>,
    batches: &[WorkerIntentBatch],
    parts: &[CoupledPublicationPart],
) -> Result<Option<Vec<WorkerIntentBatch>>, TransactionCommitError> {
    let mut withdrawn = parts
        .iter()
        .flat_map(|part| part.chain.superseded.iter().cloned())
        .collect::<Vec<_>>();
    if withdrawn.is_empty() {
        return Ok(None);
    }
    withdrawn.sort_by_key(|range| range.start);
    let batches = execution
        .run_native_with_checkpoint(|context| {
            let mut kept = Vec::new();
            let mut ranges = withdrawn.iter().peekable();
            for (position, batch) in batches.iter().enumerate() {
                checkpoint(context, batch.intents.len() as u64 + 1)
                    .map_err(custody::execution_stop)?;
                while ranges.peek().is_some_and(|range| range.end <= position) {
                    ranges.next();
                }
                if !ranges.peek().is_some_and(|range| range.contains(&position)) {
                    kept.push(batch.clone());
                }
            }
            Ok::<_, TransactionCommitError>(kept)
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(Some(batches))
}

/// Every round appended to the candidate so far: the next round's ordinal.
pub fn appended_rounds(parts: &[CoupledPublicationPart]) -> usize {
    parts.iter().map(|part| part.chain.rounds).sum()
}

/// A successor round as applied, ready to become its chain's live round.
pub struct AppliedRound {
    pub records: Vec<RecordRef>,
    pub input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub plan: OwnedNormalizedPlan,
    pub publication: RelationalCoupledProducerPosture,
    pub batches: Range<usize>,
    pub span: MemberSpan,
    pub content: RelationalContentAttachments,
    pub naming: Option<contracts::lineage::data::CommitNamingPlan>,
    pub creates: Vec<ChainCreate>,
    /// The exact touches of the round it supersedes, captured before retraction.
    pub superseded: Vec<ValidatedMutationTouch>,
}

/// Make `round` the live round of part `index`'s chain (§2.3 step 10).
pub fn adopt_round(parts: &mut [CoupledPublicationPart], index: usize, round: AppliedRound) {
    let latest = appended_rounds(parts);
    let part = &mut parts[index];
    let chain = &mut part.chain;
    chain.prior_touches.extend(round.superseded.iter().cloned());
    chain.prior_touches.sort_unstable();
    chain.prior_touches.dedup();
    chain.last_superseded = round.superseded;
    chain.touched.extend(round.records.iter().cloned());
    chain.ledger.bind_round(&round.creates);
    let prior = std::mem::replace(&mut chain.batches, round.batches);
    chain.superseded.push(prior);
    chain.span = round.span;
    chain.rounds += 1;
    chain.latest = latest;
    part.inputs = None;
    part.input = round.input;
    part.publication = round.publication;
    part.commands = round.plan;
    part.records = round.records;
    part.content = round.content;
    part.naming = round.naming;
}

/// The superseded round's exact touches, plus each kind-level inverse:
/// a dropped create deletes, a dropped link unlinks, a dropped unlink or
/// delete restores (a conservative superset of what retraction changes).
pub fn with_retraction_kinds(mut touches: Vec<Touch>) -> Vec<Touch> {
    let inverses = touches
        .iter()
        .filter_map(|touch| match touch {
            Touch::CreateEntity { kind } => Some(Touch::DeleteEntity { kind: *kind }),
            Touch::DeleteEntity { kind } => Some(Touch::CreateEntity { kind: *kind }),
            Touch::LinkRelation { kind } => Some(Touch::UnlinkRelation { kind: *kind }),
            Touch::UnlinkRelation { kind } => Some(Touch::LinkRelation { kind: *kind }),
            _ => None,
        })
        .collect::<Vec<_>>();
    touches.extend(inverses);
    touches.sort_unstable();
    touches.dedup();
    touches
}
