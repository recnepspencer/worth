//! Whole-record graph selection touches use actual before/after structural views.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelectionOwnerPort as _;
use super::*;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::transactions::data::primitives::RecordRef;
pub(super) fn collect(
    selection: &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelection,
    after: &InvariantStateView<'_>,
    before: Option<&InvariantStateView<'_>>,
    access: &crate::contracts::validation::data::descriptor::CustomInvariantAccessContract,
    work: &super::super::CustomInvariantWorkMeter,
    entities: &mut BTreeSet<EntityId>,
    relations: &mut BTreeSet<RelationId>,
    partitions: &mut BTreeSet<PartitionId>,
    entity_deletes: &mut Vec<EntityId>,
    relation_deletes: &mut Vec<RelationId>,
) {
    for record in selection.targets() {
        if !work.try_charge(1) {
            return;
        }
        match record {
            RecordRef::Entity(id) => {
                partitions.insert(id.partition_id);
                include_affected_entity(entities, id, after, before, access);
            }
            RecordRef::Relation(id) => {
                partitions.insert(id.partition_id);
                include_affected_relation(entities, relations, id, after, before, access);
            }
        }
    }
    for id in selection.retired_entities() {
        if !work.try_charge(1) {
            return;
        }
        if include_affected_entity(entities, id, after, before, access) {
            entity_deletes.push(id);
        }
    }
    for id in selection.retired_relations() {
        if !work.try_charge(1) {
            return;
        }
        if include_affected_relation(entities, relations, id, after, before, access) {
            relation_deletes.push(id);
        }
    }
}
