//! Record-level folding and superseded-batch withdrawal for a coupled chain.
use super::super::super::custody;
use super::super::chain::{Chain, Read};
use super::super::producer_posture::RelationalCoupledProducerPosture;
use super::super::{
    assembly_failure::RelationalCoupledAssemblyFailure, retained_assembly::CoupledPublicationPart,
};
use super::coverage::entity_write;
use super::proof::allocation;
use crate::authority::mutation::MutationApplyOutcome;
use crate::execution::request_stage::checkpoint;
use crate::identity::data::VersionId;
use crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan;
use crate::mvcc::validation::proposal_touches::ValidatedMutationTouch;
use crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings;
use crate::schema::data::AspectContractPlanCatalog;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::intents::mutation_intent::{CreateIntent, MutationIntent};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::{RecordRef, WorkerIntentBatch};
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use worth_execution::ExecutionRequest;

pub(super) struct FoldInputs<'a> {
    pub(super) schema: &'a AspectContractPlanCatalog,
    pub(super) version: VersionId,
    pub(super) outcome: &'a mut MutationApplyOutcome,
    pub(super) chain: &'a mut Chain,
    pub(super) state: &'a mut WorkingState,
    pub(super) symbols: &'a mut PendingSymbolMeanings,
}

/// Each chain record the round re-derived publishes what one round of its
/// write would over the read from before the chain first wrote it, so the
/// chain publishes one delta and one fragment per record. A record new to
/// the chain keeps the round's fragment and joins the chain.
pub(super) fn fold_records(
    execution: &ExecutionRequest<'_, '_>,
    inputs: FoldInputs<'_>,
    first: usize,
    round: &crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    read: &Read,
    delta_state: &mut crate::storage::overlay::WorkingState,
) -> Result<(), RelationalCoupledAssemblyFailure> {
    let FoldInputs {
        schema,
        version,
        outcome,
        chain,
        state,
        symbols,
    } = inputs;
    let bindings = &outcome.created_entities;
    let publication = &mut outcome.effect.publication;
    if publication.canonical_deltas.len() != publication.patch_fragments.len() {
        return Err(
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "publication deltas and fragments diverged",
            )
            .into(),
        );
    }
    let (folds, tail, deltas) = execution
        .run_native_with_checkpoint(|context| {
            let mut charge =
                |units: u64| checkpoint(context, units).map_err(custody::execution_stop);
            let tail = publication.patch_fragments.split_off(first);
            let deltas = publication.canonical_deltas.split_off(first);
            let mut slots = BTreeMap::new();
            for (slot, (position, _)) in chain.fragments.iter().enumerate() {
                charge(1)?;
                slots.insert(publication.patch_fragments[*position].target.clone(), slot);
            }
            let mut folds = Vec::new();
            for intent in &round.plan().merged_intents {
                charge(1)?;
                let entity = match intent {
                    MutationIntent::Create(CreateIntent::Entity(spec)) => {
                        allocation(bindings, spec, &mut charge)?.ok_or_else(|| {
                            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("a re-create has no chain allocation")
                        })?
                    }
                    // A proved relation re-create keeps the head's relation.
                    MutationIntent::Create(CreateIntent::Relation(_)) => continue,
                    _ => entity_write(intent).ok_or_else(|| {
                        crate::transactions::data::outcomes::transaction_commit_error::origin_denial("a proved round wrote no entity")
                    })?,
                };
                if let Some(&slot) = slots.get(&RecordRef::Entity(entity)) {
                    folds.push((slot, entity, intent));
                }
            }
            Ok::<_, TransactionCommitError>((folds, tail, deltas))
        })
        .map_err(custody::execution_stop)?
        .0?;
    let mut folded = BTreeSet::new();
    for (slot, entity, intent) in folds {
        let (position, base) = &chain.fragments[slot];
        let root = base.resident_basis().root();
        let (delta, fragment) = crate::authority::mutation::succession_record(
            execution,
            schema,
            root.as_ref(),
            entity,
            intent,
        )?;
        crate::authority::mutation::restamp_succession_entity(
            execution,
            state,
            delta_state,
            symbols,
            root.as_ref(),
            &delta,
            entity,
            version,
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        publication.patch_fragments[*position] = fragment;
        publication.canonical_deltas[*position] = delta;
        folded.insert(RecordRef::Entity(entity));
    }
    execution
        .run_native_with_checkpoint(|context| {
            for (fragment, delta) in tail.into_iter().zip(deltas) {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                if folded.contains(&fragment.target) {
                    continue;
                }
                chain
                    .fragments
                    .push((publication.patch_fragments.len(), read.clone()));
                publication.patch_fragments.push(fragment);
                publication.canonical_deltas.push(delta);
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0
        .map_err(Into::into)
}

/// Withdraw every superseded round's batches before final composition, so the
/// final command set holds one round per chain. Reads stay in the footprint.
pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn batches_after_withdrawal(
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
pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn appended_rounds(
    parts: &[CoupledPublicationPart],
) -> usize {
    parts.iter().map(|part| part.chain.rounds.len()).sum()
}

pub(super) struct AppliedRound<'a> {
    pub(super) records: &'a [RecordRef],
    pub(super) input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub(super) plan: OwnedNormalizedPlan,
    pub(super) publication: RelationalCoupledProducerPosture,
    pub(super) batches: Range<usize>,
    pub(super) record_range: Range<usize>,
}

pub(super) fn adopt_round(
    execution: &ExecutionRequest<'_, '_>,
    parts: &mut [CoupledPublicationPart],
    index: usize,
    prior_round_touches: Vec<ValidatedMutationTouch>,
    round: AppliedRound<'_>,
) -> Result<(), TransactionCommitError> {
    let part = &mut parts[index];
    part.chain.prior_touches.extend(prior_round_touches);
    part.chain.prior_touches.sort_unstable();
    part.chain.prior_touches.dedup();
    let mut records = std::mem::take(&mut part.records);
    execution
        .run_native_with_checkpoint(|context| {
            let known = records.iter().cloned().collect::<BTreeSet<_>>();
            for record in round.records {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                if !known.contains(record) {
                    records.push(record.clone());
                }
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0?;
    let latest = appended_rounds(parts);
    let part = &mut parts[index];
    let prior = std::mem::replace(&mut part.chain.batches, round.batches);
    part.chain.latest = latest;
    part.chain.superseded.push(prior);
    part.chain.rounds.push(round.record_range);
    part.inputs = None;
    part.input = round.input;
    part.publication = round.publication;
    part.commands = round.plan;
    part.records = records;
    Ok(())
}
