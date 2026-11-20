//! One owned route's later round folds into its chain in place, or refuses
//! under #119 before anything applies. The chain stays one producer part, so
//! every first-match consumer of `parts` reads its latest round.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::publication::data::{
    AuthoredRetirementRefusal as Cause, CoupledProvenancePostponement as Reason,
};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, EntityMutationIntent, EntitySpec, MutationIntent, RecordRef,
    WorkerIntentBatch,
};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use worth_foundational::facade::AspectFieldLocator;

/// One route's chain in the candidate. Positions are physical: transaction
/// batches and patch fragments are only ever appended before completion.
pub(crate) struct Chain {
    /// The latest round's transaction batches.
    pub(crate) batches: Range<usize>,
    /// Earlier rounds' batches, withdrawn once at completion.
    pub(crate) superseded: Vec<Range<usize>>,
    /// The chain's patch fragments, one per record it changed.
    pub(crate) fragments: Vec<usize>,
    /// The scoped coordinates the head chose; every round reuses them.
    pub(crate) local_keys: super::local_keys::LocalKeys,
    /// The records the head allocated; every round re-creates exactly these.
    pub(crate) created: Vec<EntityId>,
    /// The changed-record log's length when the latest round applied.
    pub(crate) records_end: usize,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Fold a complete re-derivation of owned route `token`'s chain into it.
    /// Admitted only for the same route, producer and posture kind, and only
    /// when the round re-writes everything the chain wrote with no other kind
    /// of effect. Any other round refuses with its #119 reason before applying.
    pub fn append_successor_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        token: RelationalCoupledProducerContinuation,
        mut publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let index = self
            .continuation
            .consume(token, self.identity, self.applied_parts())?;
        let head = &self.parts[index].publication;
        let meaning = head.producing_meaning().expect("admitted producer route");
        let source = meaning.source_scope;
        let same = match (publication.producing_meaning(), head.owned(), publication.owned()) {
            (Some(next), Some(prior), Some(owned)) => {
                next.same_route(meaning)
                    && publication.lineage_attribution().map(|(producer, _)| producer)
                        == head.lineage_attribution().map(|(producer, _)| producer)
                    && std::mem::discriminant(prior) == std::mem::discriminant(owned)
            }
            _ => false,
        };
        if !same {
            return Err(part::reject(Cause::RepeatedCoupledOwner, source).into());
        }
        let predecessor =
            super::participant_admission::admit_posture(self, execution, &publication)?;
        let chain = &self.parts[index].chain;
        if super::local_keys::rescope(
            execution,
            &chain.local_keys,
            self.mutation.candidate_symbols().interner(),
            &mut batches,
            publication.roles_mut(),
        )?
        .is_err()
        {
            return Err(continuation::postponed(Reason::StructuralEffect, source).into());
        }
        self.prove_succession(execution, index, content, &batches)?;
        let first_batch = self.transaction.batches().len();
        let first_fragment = self.mutation.outcome.effect.publication.patch_fragments.len();
        let applied = self.apply_suffix(
            execution,
            part::SuffixWriter::Successor {
                index,
                source,
                predecessor: predecessor.as_deref(),
            },
            content,
            batches,
        )?;
        let ordinary_ids = publication
            .resolve_ordinary_roles(execution, &self.mutation.outcome.created_entities)?;
        self.fold_fragments(execution, index, first_fragment)?;
        let records_end = self.mutation.outcome.effect.publication.changed_records.len();
        let batches_end = self.transaction.batches().len();
        let part = &mut self.parts[index];
        let mut records = std::mem::take(&mut part.records);
        execution
            .run_native_with_checkpoint(|context| {
                let known = records.iter().cloned().collect::<BTreeSet<_>>();
                for record in &applied.records {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    if !known.contains(record) {
                        records.push(record.clone());
                    }
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(custody::execution_stop)?
            .0?;
        let prior = std::mem::replace(&mut part.chain.batches, first_batch..batches_end);
        part.chain.superseded.push(prior);
        part.chain.records_end = records_end;
        part.inputs = None;
        part.ordinary_ids = ordinary_ids;
        part.input = applied.read;
        part.publication = publication;
        part.commands = applied.plan;
        part.records = records;
        self.successor_rounds += 1;
        let input = self.parts[index].input.clone();
        self.advance_pin(execution, &applied.delta_state, &applied.records, &input)
    }

    /// Native judgment of the rescoped round against what its chain wrote.
    /// O(chain commands + round intents + records applied since the chain).
    fn prove_succession(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        content: &crate::content::RelationalContentAttachments,
        batches: &[WorkerIntentBatch],
    ) -> Result<(), TransactionCommitError> {
        let part = &self.parts[index];
        let source = part
            .publication
            .producing_meaning()
            .expect("admitted producer route")
            .source_scope;
        if !content.is_empty() {
            return Err(continuation::postponed(Reason::StructuralEffect, source));
        }
        let bindings = &self.mutation.outcome.created_entities;
        let changed = &self.mutation.outcome.effect.publication.changed_records;
        execution
            .run_native_with_checkpoint(|context| {
                let mut charge = |units: u64| checkpoint(context, units).map_err(custody::execution_stop);
                // Exclusive outputs: no later writer changed a record this chain changed.
                let own = part.records.iter().collect::<BTreeSet<_>>();
                for record in &changed[part.chain.records_end..] {
                    charge(1)?;
                    if own.contains(record) {
                        return Err(continuation::postponed(
                            Reason::IntervenedOutput,
                            subject(record, source),
                        ));
                    }
                }
                let created = part.chain.created.iter().copied().collect::<BTreeSet<_>>();
                let mut written = BTreeSet::<(EntityId, &AspectFieldLocator)>::new();
                let mut recreated = BTreeSet::new();
                for intent in batches.iter().flat_map(|batch| &batch.intents) {
                    charge(intent.owned_allocation_capacity_bytes().saturating_add(1))?;
                    match intent {
                        MutationIntent::Create(CreateIntent::Entity(spec)) => {
                            let Some(id) =
                                allocation(bindings, spec, &mut charge)?.filter(|id| created.contains(id))
                            else {
                                return Err(continuation::postponed(Reason::StructuralEffect, source));
                            };
                            recreated.insert(id);
                            written.extend(spec.fields.locators().map(|locator| (id, locator)));
                        }
                        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) => {
                            written.extend(
                                update.fields.locators().map(|locator| (update.entity_id, locator)),
                            );
                        }
                        MutationIntent::Entity(EntityMutationIntent::Delete(delete))
                            if created.contains(&delete.entity_id) =>
                        {
                            return Err(continuation::postponed(Reason::FreshDelete, delete.entity_id));
                        }
                        _ => return Err(continuation::postponed(Reason::StructuralEffect, source)),
                    }
                }
                // Covering: the round re-creates every record the head allocated
                // and re-writes every field the latest round wrote.
                for id in &part.chain.created {
                    charge(1)?;
                    if !recreated.contains(id) {
                        return Err(continuation::postponed(Reason::UncoveredWrite, *id));
                    }
                }
                for intent in &part.commands.plan().merged_intents {
                    charge(intent.owned_allocation_capacity_bytes().saturating_add(1))?;
                    let (id, fields) = match intent {
                        MutationIntent::Create(CreateIntent::Entity(spec)) => {
                            let id = allocation(bindings, spec, &mut charge)?.ok_or_else(|| {
                                crate::mvcc::origin_denial("chain create has no actual allocation")
                            })?;
                            (id, &spec.fields)
                        }
                        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) => {
                            (update.entity_id, &update.fields)
                        }
                        _ => return Err(continuation::postponed(Reason::UncoveredWrite, source)),
                    };
                    for locator in fields.locators() {
                        if !written.contains(&(id, locator)) {
                            return Err(continuation::postponed(Reason::UncoveredWrite, id));
                        }
                    }
                }
                Ok(())
            })
            .map_err(custody::execution_stop)?
            .0
    }

    /// Combine the round's fragments into its chain's: one fragment per record,
    /// keeping the chain's first structural change. New records stay at the tail.
    fn fold_fragments(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        first: usize,
    ) -> Result<(), TransactionCommitError> {
        let fragments = &mut self.mutation.outcome.effect.publication.patch_fragments;
        let chain = &mut self.parts[index].chain;
        execution
            .run_native_with_checkpoint(|context| {
                let tail = fragments.split_off(first);
                let mut positions = BTreeMap::new();
                for &position in &chain.fragments {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    positions.insert(fragments[position].target.clone(), position);
                }
                for fragment in tail {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    match positions.get(&fragment.target) {
                        Some(&position) => crate::authority::mutation::fold_successor_fragment(
                            context,
                            &mut fragments[position],
                            fragment,
                        )?,
                        None => {
                            chain.fragments.push(fragments.len());
                            fragments.push(fragment);
                        }
                    }
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(custody::execution_stop)?
            .0
    }
}

/// Withdraw every superseded round's batches before final composition, so the
/// final command set holds one round per chain. Reads stay in the footprint.
pub(super) fn withdraw_superseded(
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

/// The chain allocation a create's raw key resolves to, charged by the probe.
fn allocation(
    bindings: &crate::transactions::data::CommitCreatedEntityBindings,
    spec: &EntitySpec,
    charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
) -> Result<Option<EntityId>, TransactionCommitError> {
    let reference = CreatedEntityRef {
        partition_id: spec.partition_id,
        kind_id: spec.kind_id,
        client_key: spec.client_key.clone(),
    };
    charge(bindings.resolve_work_bound(&reference).ok_or_else(overflow)?)?;
    Ok(bindings.resolve(&reference))
}

fn subject(record: &RecordRef, source: EntityId) -> EntityId {
    match record {
        RecordRef::Entity(entity) => *entity,
        RecordRef::Relation(_) => source,
    }
}

fn overflow() -> TransactionCommitError {
    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
