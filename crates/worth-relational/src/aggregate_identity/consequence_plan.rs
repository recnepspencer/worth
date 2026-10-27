//! Translate one identity transition into ordinary private graph mutations.
use super::cohorts::CollectedCohorts;
use super::preparation::{AggregateWork, BuildResult};
use super::projection::ChangedProjection;
use super::record::{lineage_fields, AggregateRecord};
use super::retention_extent as extent;
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::schema::data::AggregateIdentityContractDeclaration;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, DeleteRelationIntent, EntityMutationIntent,
    EntityReference, EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec,
    UpdateEntityFieldsIntent,
};
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::AggregateIdentity;

#[derive(Default)]
pub(crate) struct RecordDemands {
    pub(crate) entities: BTreeSet<EntityId>,
    pub(crate) relations: BTreeSet<RelationId>,
    pub(crate) partitions: BTreeSet<PartitionId>,
}
impl RecordDemands {
    pub(super) fn owned_bytes(&self) -> u64 {
        extent::demands(
            self.entities.len(),
            self.relations.len(),
            self.partitions.len(),
        )
    }
}
#[derive(Default)]
pub(super) struct ConsequencePlan {
    pub(super) intents: Vec<MutationIntent>,
    pub(super) demands: RecordDemands,
}
impl ConsequencePlan {
    fn push(
        &mut self,
        intent: MutationIntent,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        work.retain(intent.owned_allocation_capacity_bytes())?;
        self.intents.push(intent);
        Ok(())
    }
    fn endpoint(&mut self, reference: &EntityReference) {
        self.demands.partitions.insert(reference.partition_id());
    }
}

pub(super) fn append(
    plan: &mut ConsequencePlan,
    contract: AggregateIdentityContractDeclaration,
    occurrence: CommitId,
    cohorts: CollectedCohorts,
    after: &InvariantStateView<'_>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<ChangedProjection> {
    let members = cohorts
        .prior
        .iter()
        .map(|old| old.members.len())
        .sum::<usize>()
        .saturating_add(cohorts.proposed.iter().map(BTreeSet::len).sum::<usize>());
    // Transition maps, fragments, assignments and lineage coexist with inputs.
    work.retain((members as u64).saturating_mul(extent::transition_member()))?;
    let result = super::transition(
        contract.definition,
        occurrence,
        &cohorts.prior,
        cohorts.proposed,
        &mut |units| work.checkpoint(units),
    )?;
    let maximum_intents = result
        .assignments
        .len()
        .checked_add(result.retired.len())
        .and_then(|n| n.checked_add(cohorts.memberships.len()))
        .and_then(|n| {
            n.checked_add(
                result
                    .assignments
                    .iter()
                    .map(|group| group.members.len())
                    .sum::<usize>(),
            )
        })
        .and_then(|n| n.checked_add(result.lineage.len()))
        .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    work.retain(
        (maximum_intents as u64)
            .checked_mul(extent::vector_entry::<MutationIntent>())
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    plan.intents
        .try_reserve(maximum_intents)
        .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    let mut targets = BTreeMap::new();
    for (&identity, old) in &cohorts.rows {
        work.retain(extent::identity_target())?;
        targets.insert(identity, EntityReference::Existing(old.entity));
    }
    let mut new_membership = BTreeMap::new();
    for (assignment_index, assignment) in result.assignments.iter().enumerate() {
        let anchor = *assignment
            .members
            .first()
            .expect("complete nonempty aggregate");
        let row = if assignment.born {
            AggregateRecord {
                identity: assignment.identity,
                definition: contract.definition,
                born: occurrence,
                seed: anchor,
                current_anchor: Some(anchor),
                retired: None,
            }
        } else {
            let mut row = cohorts.rows[&assignment.identity].record.clone();
            row.current_anchor = Some(anchor);
            row
        };
        work.retain(extent::row_mutation())?;
        if assignment.born {
            work.retain(extent::identity_target())?;
            let reference = create_row(plan, contract, anchor.partition_id, row, work)?;
            targets.insert(assignment.identity, reference);
        } else if row != cohorts.rows[&assignment.identity].record {
            update_row(plan, cohorts.rows[&assignment.identity].entity, row, work)?;
        }
        for &member in &assignment.members {
            work.retain(extent::assignment_member())?;
            new_membership.insert(member, assignment_index);
        }
    }
    for identity in &result.retired {
        let old = &cohorts.rows[identity];
        let mut row = old.record.clone();
        row.current_anchor = None;
        row.retired = Some(occurrence);
        work.retain(extent::row_mutation())?;
        update_row(plan, old.entity, row, work)?;
    }
    for (&member, &(relation, old)) in &cohorts.memberships {
        work.checkpoint(1)?;
        if new_membership
            .get(&member)
            .map(|&index| result.assignments[index].identity)
            == Some(old)
        {
            continue;
        }
        // Member deletion may already have retired this private edge for audit.
        // Never restore that business consequence from the before image.
        if after.relation_metadata(relation).is_some() {
            work.retain(extent::demand())?;
            plan.demands.relations.insert(relation);
            plan.demands.partitions.insert(relation.partition_id);
            plan.endpoint(&EntityReference::Existing(member));
            plan.endpoint(&targets[&old]);
            plan.push(
                MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                })),
                work,
            )?;
        }
    }
    for (&member, &assignment_index) in &new_membership {
        let identity = result.assignments[assignment_index].identity;
        work.checkpoint(1)?;
        if cohorts
            .memberships
            .get(&member)
            .is_some_and(|(_, old)| *old == identity)
        {
            continue;
        }
        work.retain(extent::membership_mutation())?;
        let source = EntityReference::Existing(member);
        let target = targets[&identity].clone();
        plan.endpoint(&source);
        plan.endpoint(&target);
        plan.push(
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: member.partition_id,
                kind_id: contract.membership_kind,
                client_key: ClientKey::raw(format!(
                    "aggregate-member:{}:{}:{}:{}",
                    contract.membership_kind.0,
                    member.partition_value(),
                    member.local_slot_value(),
                    member.generation_value()
                )),
                source,
                target,
                fields: Default::default(),
            })),
            work,
        )?;
    }
    for event in &result.lineage {
        work.retain(extent::lineage_mutation())?;
        let source = targets[&event.from].clone();
        let target = targets[&event.to].clone();
        plan.endpoint(&source);
        plan.endpoint(&target);
        plan.push(
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: source.partition_id(),
                kind_id: contract.storage.lineage_kind,
                client_key: ClientKey::raw(format!(
                    "aggregate-lineage:{}:{:?}:{}:{}",
                    occurrence.0,
                    event.kind,
                    identity_key(event.from),
                    identity_key(event.to)
                )),
                source,
                target,
                fields: AspectFieldPatch::new(lineage_fields(occurrence, event.kind)),
            })),
            work,
        )?;
    }
    Ok(ChangedProjection {
        transition: result,
        member_assignments: new_membership,
        prior_memberships: cohorts.memberships,
    })
}

fn create_row(
    plan: &mut ConsequencePlan,
    contract: AggregateIdentityContractDeclaration,
    partition: PartitionId,
    row: AggregateRecord,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<EntityReference> {
    let key = ClientKey::raw(identity_key(row.identity));
    let reference = EntityReference::Created(CreatedEntityRef {
        partition_id: partition,
        kind_id: contract.storage.aggregate_kind,
        client_key: key.clone(),
    });
    plan.demands.partitions.insert(partition);
    plan.push(
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: partition,
            kind_id: contract.storage.aggregate_kind,
            client_key: key,
            fields: AspectFieldPatch::new(row.fields()),
        })),
        work,
    )?;
    Ok(reference)
}
fn update_row(
    plan: &mut ConsequencePlan,
    entity: EntityId,
    row: AggregateRecord,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    plan.demands.entities.insert(entity);
    plan.demands.partitions.insert(entity.partition_id);
    plan.push(
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::new(row.fields()),
            },
        )),
        work,
    )
}
fn identity_key(identity: AggregateIdentity) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut key = String::with_capacity(64);
    for byte in identity.bytes() {
        key.push(HEX[(byte >> 4) as usize] as char);
        key.push(HEX[(byte & 15) as usize] as char);
    }
    key
}
