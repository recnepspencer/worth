use std::collections::{BTreeSet, VecDeque};

use worth_execution::{MapKernelContext, MapKernelFailure};

use super::super::query_packetization::PacketizedQueryWork;
use super::leased_map::{QueryPacketBudget, QueryReadPacketDenial};
use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::query::data::{
    PlannedQueryPacket, QueryFragmentCounters, QueryOrderingContract, QueryWorkerFragment,
    TraversalEntityVisitKey, TraversalReductionBasis, TraversalRelationVisitKey,
};
use crate::runtime::{RelationalRuntime, VisibilityProjectionView};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::{adjacency_queries, AdjacencyDirection};
use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};

type PacketFailure = MapKernelFailure<QueryReadPacketDenial>;
type FrontierEntry = (EntityId, u32, EntityId, Option<RelationId>);

#[derive(Clone, Copy)]
enum Direction {
    Outgoing,
    Incoming,
}

pub(in crate::visibility::materialization::read_records::reader) fn execute_leased_traversal_fragment(
    runtime: &RelationalRuntime,
    basis: &VisibilitySnapshotBasis,
    state: &(dyn PartitionAccess + Sync),
    registry: &RelationalSchemaRegistry,
    version_id: VersionId,
    packet: &PlannedQueryPacket,
    work: &PacketizedQueryWork,
    ordinal: u64,
    ceiling: u64,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<QueryWorkerFragment, PacketFailure> {
    if packet.ordering != QueryOrderingContract::CanonicalTraversalOrder {
        return Err(MapKernelFailure::Domain(
            QueryReadPacketDenial::FragmentUnavailable,
        ));
    }
    let (seeds, kind_scope, direction, max_depth) = match work {
        PacketizedQueryWork::OutgoingNeighborhood {
            seeds,
            relation_kind_scope,
        } => (seeds, relation_kind_scope, Direction::Outgoing, Some(1)),
        PacketizedQueryWork::IncomingNeighborhood {
            seeds,
            relation_kind_scope,
        } => (seeds, relation_kind_scope, Direction::Incoming, Some(1)),
        PacketizedQueryWork::ConnectivityTraversal {
            seeds,
            relation_kind_scope,
            max_depth,
        } => (seeds, relation_kind_scope, Direction::Outgoing, *max_depth),
        _ => {
            return Err(MapKernelFailure::Domain(
                QueryReadPacketDenial::FragmentUnavailable,
            ))
        }
    };
    let mut budget = QueryPacketBudget::new(context, ceiling)?;
    let projection =
        VisibilityProjectionView::new(runtime, SnapshotStateBasis::Exact(basis.clone()));
    let mut visited_entities = BTreeSet::new();
    let mut emitted_relations = BTreeSet::new();
    let mut frontier = VecDeque::<FrontierEntry>::new();
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut entity_visit_keys = Vec::new();
    let mut relation_visit_keys = Vec::new();
    let mut touched_partitions = BTreeSet::new();

    for seed in seeds.iter().copied() {
        budget.checkpoint(1)?;
        if visited_entities.contains(&seed) {
            continue;
        }
        budget.claim_scratch(entity_scratch_bytes())?;
        visited_entities.insert(seed);
        frontier.push_back((seed, 0, seed, None));
    }

    while let Some((entity_id, depth, root_seed, via_relation)) = frontier.pop_front() {
        budget.checkpoint(1)?;
        budget.check_temporary_record(projection.candidate_entity_bytes_for_id(entity_id))?;
        let Some(entity_record) = runtime
            .read_truth()
            .authoritative_entity_record_for_id_at_version_with_registry(
                state, registry, entity_id, version_id,
            )
        else {
            continue;
        };
        budget.push_result(
            &mut entity_visit_keys,
            TraversalEntityVisitKey {
                depth,
                root_seed,
                via_relation,
                entity_id,
            },
        )?;
        budget.push_result(&mut entities, entity_record)?;
        if !touched_partitions.contains(&entity_id.partition_id) {
            budget.claim_scratch(partition_scratch_bytes())?;
            touched_partitions.insert(entity_id.partition_id);
        }
        if max_depth.is_some_and(|limit| depth >= limit) {
            continue;
        }

        let adjacency_direction = match direction {
            Direction::Outgoing => AdjacencyDirection::Outgoing,
            Direction::Incoming => AdjacencyDirection::Incoming,
        };
        let candidates = adjacency_queries::relation_candidates_from_state(
            state,
            entity_id,
            adjacency_direction,
        );
        for relation_id in candidates.iter().copied() {
            budget.checkpoint(1)?;
            budget
                .check_temporary_record(projection.candidate_relation_bytes_for_id(relation_id))?;
            let Some(relation_record) = runtime
                .read_truth()
                .authoritative_relation_record_for_id_at_version_with_registry(
                    state,
                    registry,
                    relation_id,
                    version_id,
                )
            else {
                continue;
            };
            if !kind_scope
                .as_ref()
                .is_none_or(|kinds| kind_is_in_scope(kinds, relation_record.kind.kind_id))
            {
                continue;
            }
            if !emitted_relations.contains(&relation_id) {
                budget.claim_scratch(relation_scratch_bytes())?;
                emitted_relations.insert(relation_id);
                budget.push_result(
                    &mut relation_visit_keys,
                    TraversalRelationVisitKey {
                        depth,
                        root_seed,
                        relation_id,
                    },
                )?;
                budget.push_cloned_result(&mut relations, &relation_record)?;
            }
            let neighbor = match direction {
                Direction::Outgoing => relation_record.target,
                Direction::Incoming => relation_record.source,
            };
            if !visited_entities.contains(&neighbor) {
                budget.claim_scratch(entity_scratch_bytes())?;
                visited_entities.insert(neighbor);
                frontier.push_back((neighbor, depth + 1, root_seed, Some(relation_id)));
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
            target_count: seeds.len(),
            authoritative_entity_records_emitted: entities.len(),
            authoritative_relation_records_emitted: relations.len(),
            touched_partitions: touched_partitions.len(),
        },
        entities,
        relations,
        traversal_basis: Some(TraversalReductionBasis {
            entity_visit_keys,
            relation_visit_keys,
        }),
    })
}

fn kind_is_in_scope(kinds: &[KindId], kind: KindId) -> bool {
    kinds.binary_search(&kind).is_ok()
}

fn entity_scratch_bytes() -> u64 {
    (std::mem::size_of::<EntityId>() * 16 + std::mem::size_of::<FrontierEntry>() * 8 + 128) as u64
}

fn relation_scratch_bytes() -> u64 {
    (std::mem::size_of::<RelationId>() * 16 + 128) as u64
}

fn partition_scratch_bytes() -> u64 {
    (std::mem::size_of::<crate::identity::data::PartitionId>() * 16 + 128) as u64
}
