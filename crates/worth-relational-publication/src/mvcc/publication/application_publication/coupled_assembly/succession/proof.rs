//! Judge a rescoped producer round against its exact chain and applied records.
use super::super::super::custody;
use super::super::refusals::{overflow, postponed, reject};
use super::super::{
    producer_posture::RelationalCoupledProducerPosture, retained_assembly::CoupledPublicationPart,
};
use super::coverage::{covered, entity_write, roles, units, Unit};
use super::relations;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::authored_retirement_refusal::CoupledProvenancePostponement as Reason;
use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntentOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::CreatedEntityRef;
use contracts::transactions::data::primitives::EntitySpec;
use contracts::transactions::data::primitives::RecordRef;
use contracts::transactions::data::primitives::WorkerIntentBatch;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;

/// Native judgment of the rescoped round against what its chain wrote.
/// O(chain commands + round intents + roles + records applied since the chain).
pub fn prove_succession(
    execution: &ExecutionRequest<'_, '_>,
    part: &CoupledPublicationPart,
    outcome: &worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
    content: &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
    publication: &RelationalCoupledProducerPosture,
    batches: &[WorkerIntentBatch],
) -> Result<(), TransactionCommitError> {
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
                        return Err(postponed(Reason::StructuralEffect, source));
                    }
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
            for intent in &part.commands.plan().merged_intents {
                charge(intent.owned_allocation_capacity_bytes().saturating_add(1))?;
                if let Some(link) =
                    relations::relink(bindings, symbols, intent, source, &mut charge)?
                {
                    if !relinked.contains(&link) {
                        return Err(postponed(Reason::UncoveredWrite, source));
                    }
                    continue;
                }
                let id = match intent {
                    MutationIntent::Create(CreateIntent::Entity(spec)) => {
                        allocation(bindings, spec, &mut charge)?.ok_or_else(|| {
                            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("chain create has no actual allocation")
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
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}

/// The chain allocation a create's raw key resolves to, charged by the probe.
pub(super) fn allocation(
    bindings: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
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

pub fn require_same_route(
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
