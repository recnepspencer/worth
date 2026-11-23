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
use coverage::{covered, entity_write, roles, units, Unit};
use std::collections::BTreeSet;
use std::ops::Range;

mod coverage;
mod fold;
mod relations;
pub(super) use fold::withdraw_superseded;

type Read = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

/// One route's chain in the candidate. Positions are physical: transaction
/// batches and patch fragments are only ever appended before completion.
pub(crate) struct Chain {
    /// The candidate read immediately before the chain's head appended.
    pub(crate) head_input: Read,
    /// Cumulative certified touches of superseded rounds of this chain.
    pub(crate) prior_touches: Vec<crate::mvcc::validation::ValidatedMutationTouch>,
    /// The latest round's transaction batches.
    pub(crate) batches: Range<usize>,
    /// Earlier rounds' batches, withdrawn once at completion.
    pub(crate) superseded: Vec<Range<usize>>,
    /// The chain's patch fragments, one per record it changed, each with the
    /// candidate read from before the chain first wrote that record.
    pub(crate) fragments: Vec<(usize, Read)>,
    /// The scoped coordinates the head chose; every round reuses them.
    pub(crate) local_keys: super::local_keys::LocalKeys,
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
        let same = match (
            publication.producing_meaning(),
            head.owned(),
            publication.owned(),
        ) {
            (Some(next), Some(prior), Some(owned)) => {
                next.same_route(meaning)
                    && publication
                        .lineage_attribution()
                        .map(|(producer, _)| producer)
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
        super::local_keys::repartition(
            execution,
            &self.parts[index].commands.plan().merged_intents,
            self.mutation.candidate_symbols().interner(),
            &mut batches,
            publication.roles_mut(),
        )?;
        self.prove_succession(execution, index, content, &publication, &batches)?;
        let first_record = self
            .mutation
            .outcome
            .effect
            .publication
            .changed_records
            .len();
        let first_batch = self.transaction.batches().len();
        let first_fragment = self
            .mutation
            .outcome
            .effect
            .publication
            .patch_fragments
            .len();
        let mut applied = self.apply_suffix(
            execution,
            part::SuffixWriter::Successor {
                index,
                source,
                predecessor: predecessor.as_deref(),
            },
            content,
            batches,
        )?;
        self.fold_records(
            execution,
            index,
            first_fragment,
            &applied.plan,
            &applied.read,
            &mut applied.delta_state,
        )?;
        let records_end = self
            .mutation
            .outcome
            .effect
            .publication
            .changed_records
            .len();
        let batches_end = self.transaction.batches().len();
        let prior_round_touches = crate::mvcc::validation::project_applied_intents(
            execution,
            &self.mutation.state,
            &self.parts[index].commands.plan().merged_intents,
        )?;
        let part = &mut self.parts[index];
        part.chain.prior_touches.extend(prior_round_touches);
        part.chain.prior_touches.sort_unstable();
        part.chain.prior_touches.dedup();
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
        let latest = appended_rounds(&self.parts);
        let part = &mut self.parts[index];
        let prior = std::mem::replace(&mut part.chain.batches, first_batch..batches_end);
        part.chain.latest = latest;
        part.chain.superseded.push(prior);
        part.chain.rounds.push(first_record..records_end);
        part.inputs = None;
        part.input = applied.read;
        part.publication = publication;
        part.commands = applied.plan;
        part.records = records;
        self.successor_rounds += 1;
        let input = self.parts[index].input.clone();
        self.advance_pin(execution, &applied.delta_state, &applied.records, &input)
    }

    /// Native judgment of the rescoped round against what its chain wrote.
    /// O(chain commands + round intents + roles + records applied since the chain).
    fn prove_succession(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        content: &crate::content::RelationalContentAttachments,
        publication: &RelationalCoupledProducerPosture,
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
        let symbols = self.mutation.candidate_symbols().interner();
        let changed = &self.mutation.outcome.effect.publication.changed_records;
        execution
            .run_native_with_checkpoint(|context| {
                let mut charge =
                    |units: u64| checkpoint(context, units).map_err(custody::execution_stop);
                // Identity: the round publishes the same roles over the same records.
                let head = roles(bindings, part.publication.roles(), &mut charge)?;
                if head.is_none() || head != roles(bindings, publication.roles(), &mut charge)? {
                    return Err(continuation::postponed(Reason::UncoveredWrite, source));
                }
                // Exclusive outputs: no other writer changed a record this
                // chain changed since its head.
                let own = part.records.iter().collect::<BTreeSet<_>>();
                if let Some(record) = part.chain.intervened(changed, &own, &mut charge)? {
                    return Err(continuation::postponed(
                        Reason::IntervenedOutput,
                        subject(record, source),
                    ));
                }
                let created = part.chain.created.iter().copied().collect::<BTreeSet<_>>();
                // Identity: each head relation is re-created over the same endpoints.
                let mut links = BTreeSet::new();
                for intent in &part.commands.plan().merged_intents {
                    if let Some(link) =
                        relations::relink(bindings, symbols, intent, source, &mut charge)?
                    {
                        links.insert(link);
                    }
                }
                let mut relinked = BTreeSet::new();
                let mut written = BTreeSet::<Unit>::new();
                let mut recreated = BTreeSet::new();
                // One write per record: a chain record folds as one round's one write.
                let mut targets = BTreeSet::new();
                for intent in batches.iter().flat_map(|batch| &batch.intents) {
                    charge(intent.owned_allocation_capacity_bytes().saturating_add(1))?;
                    if let Some(link) =
                        relations::relink(bindings, symbols, intent, source, &mut charge)?
                    {
                        if !links.contains(&link) || !relinked.insert(link) {
                            return Err(continuation::postponed(Reason::StructuralEffect, source));
                        }
                        continue;
                    }
                    let id = match intent {
                        MutationIntent::Create(CreateIntent::Entity(spec)) => {
                            let Some(id) = allocation(bindings, spec, &mut charge)?
                                .filter(|id| created.contains(id))
                            else {
                                return Err(continuation::postponed(
                                    Reason::StructuralEffect,
                                    source,
                                ));
                            };
                            recreated.insert(id);
                            id
                        }
                        MutationIntent::Entity(EntityMutationIntent::Delete(delete))
                            if created.contains(&delete.entity_id) =>
                        {
                            return Err(continuation::postponed(
                                Reason::FreshDelete,
                                delete.entity_id,
                            ));
                        }
                        _ => entity_write(intent).ok_or_else(|| {
                            continuation::postponed(Reason::StructuralEffect, source)
                        })?,
                    };
                    if !targets.insert(id) {
                        return Err(continuation::postponed(Reason::StructuralEffect, id));
                    }
                    units(id, intent, &mut written);
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
                    if let Some(link) =
                        relations::relink(bindings, symbols, intent, source, &mut charge)?
                    {
                        if !relinked.contains(&link) {
                            return Err(continuation::postponed(Reason::UncoveredWrite, source));
                        }
                        continue;
                    }
                    let id = match intent {
                        MutationIntent::Create(CreateIntent::Entity(spec)) => {
                            allocation(bindings, spec, &mut charge)?.ok_or_else(|| {
                                crate::mvcc::origin_denial("chain create has no actual allocation")
                            })?
                        }
                        _ => entity_write(intent).ok_or_else(|| {
                            continuation::postponed(Reason::UncoveredWrite, source)
                        })?,
                    };
                    let mut wrote = BTreeSet::new();
                    units(id, intent, &mut wrote);
                    for (id, key, path) in wrote {
                        charge(
                            path.as_ref()
                                .map_or(1, |path| path.fields().len() as u64 + 1),
                        )?;
                        if !covered(&written, id, &key, path.as_ref()) {
                            return Err(continuation::postponed(Reason::UncoveredWrite, id));
                        }
                    }
                }
                Ok(())
            })
            .map_err(custody::execution_stop)?
            .0
    }
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
    charge(
        bindings
            .resolve_work_bound(&reference)
            .ok_or_else(overflow)?,
    )?;
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

/// Every round appended to the candidate so far: the next round's ordinal.
pub(super) fn appended_rounds(parts: &[super::CoupledPublicationPart]) -> usize {
    parts.iter().map(|part| part.chain.rounds.len()).sum()
}
