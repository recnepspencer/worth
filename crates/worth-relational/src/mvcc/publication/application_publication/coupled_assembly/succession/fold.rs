//! Record-level folding and superseded-batch withdrawal for a coupled chain.
use super::*;
use std::collections::BTreeMap;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Each chain record the round re-derived publishes what one round of its
    /// write would over the read from before the chain first wrote it, so the
    /// chain publishes one delta and one fragment per record. A record new to
    /// the chain keeps the round's fragment and joins the chain.
    pub(super) fn fold_records(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        first: usize,
        round: &crate::mvcc::OwnedNormalizedPlan,
        read: &Read,
        delta_state: &mut crate::storage::overlay::WorkingState,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let bindings = &self.mutation.outcome.created_entities;
        let publication = &mut self.mutation.outcome.effect.publication;
        let chain = &mut self.parts[index].chain;
        if publication.canonical_deltas.len() != publication.patch_fragments.len() {
            return Err(
                crate::mvcc::origin_denial("publication deltas and fragments diverged").into(),
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
                                crate::mvcc::origin_denial("a re-create has no chain allocation")
                            })?
                        }
                        _ => entity_write(intent).ok_or_else(|| {
                            crate::mvcc::origin_denial("a proved round wrote no entity")
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
        let plans = self.prepared.schema_authority.aspect_plans();
        let version = self.identity.proposed_version_id();
        let state = &mut self.mutation.state;
        let symbols = &mut self.mutation.identities.symbols;
        let mut folded = BTreeSet::new();
        for (slot, entity, intent) in folds {
            let (position, base) = &chain.fragments[slot];
            let root = base.resident_basis().root();
            let (delta, fragment) = crate::authority::mutation::succession_record(
                execution,
                plans,
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
}

/// Withdraw every superseded round's batches before final composition, so the
/// final command set holds one round per chain. Reads stay in the footprint.
pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn withdraw_superseded(
    execution: &ExecutionRequest<'_, '_>,
    transaction: &mut crate::mvcc::BranchBoundRelationalTransaction,
    parts: &[CoupledPublicationPart],
) -> Result<(), TransactionCommitError> {
    let mut withdrawn = parts
        .iter()
        .flat_map(|part| part.chain.superseded.iter().cloned())
        .collect::<Vec<_>>();
    if withdrawn.is_empty() {
        return Ok(());
    }
    withdrawn.sort_by_key(|range| range.start);
    let batches = execution
        .run_native_with_checkpoint(|context| {
            let mut kept = Vec::new();
            let mut ranges = withdrawn.iter().peekable();
            for (position, batch) in transaction.batches().iter().enumerate() {
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
    transaction
        .restage_coupled_field_batches(batches, execution.allocation_policy())
        .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))
}
