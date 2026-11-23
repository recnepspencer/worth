//! Judge a rescoped producer round against its exact chain and applied records.
use super::super::super::custody;
use super::super::refusals::{overflow, postponed, reject};
use super::super::{
    producer_posture::RelationalCoupledProducerPosture, retained_assembly::CoupledPublicationPart,
};
use super::coverage::{covered, entity_write, roles, units, Unit};
use super::{redelete, relations};
use crate::execution::request_stage::checkpoint;
use crate::identity::data::EntityId;
use crate::publication::data::{
    AuthoredRetirementRefusal as Cause, CoupledProvenancePostponement as Reason,
};
use crate::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::{
    CreatedEntityRef, EntitySpec, RecordRef, WorkerIntentBatch,
};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;

/// Native judgment of the rescoped round against what its chain wrote.
/// Returns the entities the round provably deletes again.
/// O(chain commands + round intents + roles + records applied since the chain).
pub(super) fn prove_succession(
    execution: &ExecutionRequest<'_, '_>,
    part: &CoupledPublicationPart,
    outcome: &crate::authority::mutation::MutationApplyOutcome,
    symbols: &crate::symbols::data::StringInterner,
    content: &crate::content::RelationalContentAttachments,
    publication: &RelationalCoupledProducerPosture,
    batches: &[WorkerIntentBatch],
) -> Result<BTreeSet<EntityId>, TransactionCommitError> {
    let source = part
        .publication
        .producing_meaning()
        .expect("admitted producer route")
        .source_scope;
    if !content.is_empty() {
        return Err(postponed(Reason::StructuralEffect, source));
    }
    let bindings = &outcome.created_entities;
    let changed = &outcome.effect.publication.changed_records;
    execution
        .run_native_with_checkpoint(|context| {
            let mut charge =
                |units: u64| checkpoint(context, units).map_err(custody::execution_stop);
            // Identity: the round publishes the same roles over the same records.
            let head = roles(bindings, part.publication.roles(), &mut charge)?;
            if head.is_none() || head != roles(bindings, publication.roles(), &mut charge)? {
                return Err(postponed(Reason::UncoveredWrite, source));
            }
            // Exclusive outputs: no other writer changed a record this
            // chain changed since its head.
            let own = part.records.iter().collect::<BTreeSet<_>>();
            if let Some(record) = part.chain.intervened(changed, &own, &mut charge)? {
                return Err(postponed(
                    Reason::IntervenedOutput,
                    subject(record, source),
                ));
            }
            let created = part.chain.created.iter().copied().collect::<BTreeSet<_>>();
            let latest = &part.commands.plan().merged_intents;
            let mut relation_identity = relations::RelationIdentity::of_chain(
                bindings,
                symbols,
                latest,
                source,
                &mut charge,
            )?;
            let mut redeletes = redelete::Redeletes::of_chain(latest, &created, &mut charge)?;
            let mut written = BTreeSet::<Unit>::new();
            let mut recreated = BTreeSet::new();
            // One write per record: a chain record folds as one round's one write.
            let mut targets = BTreeSet::new();
            for intent in batches.iter().flat_map(|batch| &batch.intents) {
                charge(intent.owned_allocation_capacity_bytes().saturating_add(1))?;
                if relation_identity.admit(bindings, symbols, intent, source, &mut charge)?
                    || redeletes.admit(intent, &mut targets)?
                {
                    continue;
                }
                let id = match intent {
                    MutationIntent::Create(CreateIntent::Entity(spec)) => {
                        let Some(id) = allocation(bindings, spec, &mut charge)?
                            .filter(|id| created.contains(id))
                        else {
                            return Err(postponed(
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
                        return Err(postponed(
                            Reason::FreshDelete,
                            delete.entity_id,
                        ));
                    }
                    _ => entity_write(intent).ok_or_else(|| {
                        postponed(Reason::StructuralEffect, source)
                    })?,
                };
                if !targets.insert(id) {
                    return Err(postponed(Reason::StructuralEffect, id));
                }
                units(id, intent, &mut written);
            }
            // Covering: the round re-creates every record the head allocated
            // and re-writes every field the latest round wrote.
            for id in &part.chain.created {
                charge(1)?;
                if !recreated.contains(id) {
                    return Err(postponed(Reason::UncoveredWrite, *id));
                }
            }
            for intent in latest {
                charge(intent.owned_allocation_capacity_bytes().saturating_add(1))?;
                if relation_identity.covers(bindings, symbols, intent, source, &mut charge)?
                    || redeletes.covers(intent)?
                {
                    continue;
                }
                let id = match intent {
                    MutationIntent::Create(CreateIntent::Entity(spec)) => {
                        allocation(bindings, spec, &mut charge)?.ok_or_else(|| {
                            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("chain create has no actual allocation")
                        })?
                    }
                    _ => entity_write(intent).ok_or_else(|| {
                        postponed(Reason::UncoveredWrite, source)
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
                        return Err(postponed(Reason::UncoveredWrite, id));
                    }
                }
            }
            Ok(redeletes.into_proved())
        })
        .map_err(custody::execution_stop)?
        .0
}

/// The chain allocation a create's raw key resolves to, charged by the probe.
pub(super) fn allocation(
    bindings: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
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

pub(super) fn require_same_route(
    head: &RelationalCoupledProducerPosture,
    publication: &RelationalCoupledProducerPosture,
) -> Result<EntityId, TransactionCommitError> {
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
        return Err(reject(Cause::RepeatedCoupledOwner, source));
    }
    Ok(source)
}
