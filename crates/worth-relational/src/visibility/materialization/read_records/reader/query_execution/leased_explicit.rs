use std::collections::BTreeSet;

use worth_execution::{MapKernelContext, MapKernelFailure};

use super::super::query_packetization::PacketizedQueryWork;
use super::super::VisibilityReadContext;
use super::leased_map::{QueryPacketBudget, QueryReadPacketDenial};
use crate::identity::data::VersionId;
use crate::query::data::{PlannedQueryPacket, QueryFragmentCounters, QueryWorkerFragment};
use crate::runtime::VisibilityProjectionView;
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::PartitionAccess;
use crate::transactions::data::RecordRef;
use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};

pub(in crate::visibility::materialization::read_records::reader) fn execute_leased_explicit_fragment(
    reader: &VisibilityReadContext<'_>,
    basis: &VisibilitySnapshotBasis,
    state_access: &(dyn PartitionAccess + Sync),
    registry: &RelationalSchemaRegistry,
    version_id: VersionId,
    packet: &PlannedQueryPacket,
    work: &PacketizedQueryWork,
    ordinal: u64,
    ceiling: u64,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<QueryWorkerFragment, MapKernelFailure<QueryReadPacketDenial>> {
    let PacketizedQueryWork::ExplicitTargets(targets) = work else {
        return Err(MapKernelFailure::Domain(
            QueryReadPacketDenial::FragmentUnavailable,
        ));
    };
    let mut budget = QueryPacketBudget::new(context, ceiling)?;
    let projection =
        VisibilityProjectionView::new(reader.runtime(), SnapshotStateBasis::Exact(basis.clone()));
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut touched_partitions = BTreeSet::new();
    for target in targets {
        budget.checkpoint(1)?;
        let partition_id = match target {
            RecordRef::Entity(entity_id) => entity_id.partition_id,
            RecordRef::Relation(relation_id) => relation_id.partition_id,
        };
        if !touched_partitions.contains(&partition_id) {
            budget.claim_scratch(
                (std::mem::size_of::<crate::identity::data::PartitionId>() * 16 + 128) as u64,
            )?;
            touched_partitions.insert(partition_id);
        }
        let Some(partition) = basis.root().get_partition(partition_id) else {
            continue;
        };
        match target {
            RecordRef::Entity(entity_id) => {
                if partition
                    .entity_arena
                    .live_bitset
                    .count_ones_in_range(entity_id.slot_index(), entity_id.slot_index() + 1)
                    == 0
                {
                    continue;
                }
                budget
                    .check_temporary_record(projection.candidate_entity_bytes_for_id(*entity_id))?;
                if let Some(record) = reader
                    .authoritative_entity_record_for_id_at_version_with_registry(
                        state_access,
                        registry,
                        *entity_id,
                        version_id,
                    )
                {
                    budget.push_result(&mut entities, record)?;
                }
            }
            RecordRef::Relation(relation_id) => {
                if partition
                    .relation_arena
                    .live_bitset
                    .count_ones_in_range(relation_id.slot_index(), relation_id.slot_index() + 1)
                    == 0
                {
                    continue;
                }
                budget.check_temporary_record(
                    projection.candidate_relation_bytes_for_id(*relation_id),
                )?;
                if let Some(record) = reader
                    .authoritative_relation_record_for_id_at_version_with_registry(
                        state_access,
                        registry,
                        *relation_id,
                        version_id,
                    )
                {
                    budget.push_result(&mut relations, record)?;
                }
            }
        }
    }
    Ok(QueryWorkerFragment {
        plan_key: packet.plan_key,
        fragment_key: crate::query::data::deterministic_query_fragment_key(
            packet.plan_key,
            ordinal,
        ),
        ordering: packet.ordering,
        counters: QueryFragmentCounters {
            target_count: targets.len(),
            authoritative_entity_records_emitted: entities.len(),
            authoritative_relation_records_emitted: relations.len(),
            touched_partitions: touched_partitions.len(),
        },
        entities,
        relations,
        traversal_basis: None,
    })
}
