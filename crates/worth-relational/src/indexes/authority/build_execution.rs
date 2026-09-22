use rayon::prelude::*;

use crate::authority::commit::preparation::packets::index::IndexPreparationPacket;
use crate::authority::commit::preparation::planning::strategy::{
    ParallelLegality, ParallelProfitability, PreparationStrategy, PreparationStrategySelection,
};
use crate::indexes::data::{DerivedIndexEntries, DerivedIndexId, DerivedIndexKind};
use crate::runtime::RelationalRuntime;

use super::super::projected_field_values::{
    build_entity_aspect_field_index, build_related_entity_ordering_index,
    build_relation_aspect_field_index, build_relation_join_index, IndexProjectionSource,
    RelatedEntityOrderingProjection,
};

#[derive(Debug, Clone)]
pub(super) struct IndexPreparationResult {
    pub(super) index_id: DerivedIndexId,
    pub(super) entries: Option<DerivedIndexEntries>,
}

pub(super) fn record_index_preparation_strategy_counters(
    runtime: &RelationalRuntime,
    definition_count: usize,
    strategy: &PreparationStrategy,
) {
    runtime.performance_access().count_preparation_packet_shape(
        definition_count,
        definition_count,
        usize::from(definition_count != 0),
        usize::from(definition_count != 0),
    );

    match strategy.parallel_legality {
        ParallelLegality::ProvenParallel => runtime
            .performance_access()
            .count_preparation_parallel_legal(),
        ParallelLegality::RequiresSerial => {}
    }
    match strategy.parallel_profitability {
        ParallelProfitability::Profitable => runtime
            .performance_access()
            .count_preparation_parallel_profitable(),
        ParallelProfitability::NotProfitable => {}
    }
    match strategy.selected_mode {
        PreparationStrategySelection::Serial => runtime
            .performance_access()
            .count_preparation_serial_strategy(),
        PreparationStrategySelection::StagedParallel => runtime
            .performance_access()
            .count_preparation_staged_parallel_strategy(),
    }
}

pub(super) fn execute_index_packets(
    runtime: &RelationalRuntime,
    projection: &IndexProjectionSource<'_, '_>,
    packets: &[IndexPreparationPacket],
    selected_mode: PreparationStrategySelection,
) -> Vec<IndexPreparationResult> {
    let entity_packets = packets
        .iter()
        .filter_map(entity_field_packet)
        .collect::<Vec<_>>();
    let relation_packets = packets
        .iter()
        .filter_map(relation_field_packet)
        .collect::<Vec<_>>();
    let related_entity_packets = packets
        .iter()
        .filter_map(related_entity_ordering_packet)
        .collect::<Vec<_>>();
    let relation_join_packets = packets
        .iter()
        .filter_map(relation_join_packet)
        .collect::<Vec<_>>();

    let entity_streams = match selected_mode {
        PreparationStrategySelection::StagedParallel => entity_packets
            .par_iter()
            .map(|(reduction_key, index_id, field_locator)| {
                singleton_result_stream(
                    *reduction_key,
                    *index_id,
                    DerivedIndexEntries::EntityField(
                        build_entity_aspect_field_index(projection, field_locator).into(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
        PreparationStrategySelection::Serial => entity_packets
            .iter()
            .map(|(reduction_key, index_id, field_locator)| {
                singleton_result_stream(
                    *reduction_key,
                    *index_id,
                    DerivedIndexEntries::EntityField(
                        build_entity_aspect_field_index(projection, field_locator).into(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    };

    let relation_streams = match selected_mode {
        PreparationStrategySelection::StagedParallel => relation_packets
            .par_iter()
            .map(|(reduction_key, index_id, field_locator)| {
                singleton_result_stream(
                    *reduction_key,
                    *index_id,
                    DerivedIndexEntries::RelationField(
                        build_relation_aspect_field_index(projection, field_locator).into(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
        PreparationStrategySelection::Serial => relation_packets
            .iter()
            .map(|(reduction_key, index_id, field_locator)| {
                singleton_result_stream(
                    *reduction_key,
                    *index_id,
                    DerivedIndexEntries::RelationField(
                        build_relation_aspect_field_index(projection, field_locator).into(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    };
    let related_entity_streams = match selected_mode {
        PreparationStrategySelection::StagedParallel => related_entity_packets
            .par_iter()
            .map(
                |(
                    reduction_key,
                    index_id,
                    relation_kind,
                    parent_endpoint,
                    child_kind,
                    ordering,
                )| {
                    singleton_result_stream(
                        *reduction_key,
                        *index_id,
                        DerivedIndexEntries::RelatedEntityOrdering(
                            build_related_entity_ordering_index(
                                projection,
                                &RelatedEntityOrderingProjection::new(
                                    *relation_kind,
                                    *parent_endpoint,
                                    *child_kind,
                                    ordering,
                                ),
                            )
                            .into(),
                        ),
                    )
                },
            )
            .collect::<Vec<_>>(),
        PreparationStrategySelection::Serial => related_entity_packets
            .iter()
            .map(
                |(
                    reduction_key,
                    index_id,
                    relation_kind,
                    parent_endpoint,
                    child_kind,
                    ordering,
                )| {
                    singleton_result_stream(
                        *reduction_key,
                        *index_id,
                        DerivedIndexEntries::RelatedEntityOrdering(
                            build_related_entity_ordering_index(
                                projection,
                                &RelatedEntityOrderingProjection::new(
                                    *relation_kind,
                                    *parent_endpoint,
                                    *child_kind,
                                    ordering,
                                ),
                            )
                            .into(),
                        ),
                    )
                },
            )
            .collect::<Vec<_>>(),
    };
    let relation_join_streams = match selected_mode {
        PreparationStrategySelection::StagedParallel => relation_join_packets
            .par_iter()
            .map(|packet| relation_join_result(runtime, projection, packet))
            .collect::<Vec<_>>(),
        PreparationStrategySelection::Serial => relation_join_packets
            .iter()
            .map(|packet| relation_join_result(runtime, projection, packet))
            .collect::<Vec<_>>(),
    };

    crate::authority::commit::preparation::reduction::merge::canonical_merge_streams(
        entity_streams
            .into_iter()
            .chain(relation_streams)
            .chain(related_entity_streams)
            .chain(relation_join_streams)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|(_, result)| result)
    .collect()
}

fn entity_field_packet(
    packet: &IndexPreparationPacket,
) -> Option<(
    crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    DerivedIndexId,
    worth_foundational::facade::AspectFieldLocator,
)> {
    match &packet.definition.kind {
        DerivedIndexKind::EntityField { field_locator } => Some((
            packet.header.reduction_key,
            packet.header.identity.index_id,
            field_locator.clone(),
        )),
        DerivedIndexKind::RelationField { .. } => None,
        DerivedIndexKind::RelatedEntityOrdering { .. } => None,
        DerivedIndexKind::RelationJoin(_) => None,
    }
}

fn relation_field_packet(
    packet: &IndexPreparationPacket,
) -> Option<(
    crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    DerivedIndexId,
    worth_foundational::facade::AspectFieldLocator,
)> {
    match &packet.definition.kind {
        DerivedIndexKind::EntityField { .. } => None,
        DerivedIndexKind::RelationField { field_locator } => Some((
            packet.header.reduction_key,
            packet.header.identity.index_id,
            field_locator.clone(),
        )),
        DerivedIndexKind::RelatedEntityOrdering { .. } => None,
        DerivedIndexKind::RelationJoin(_) => None,
    }
}

#[allow(clippy::type_complexity)]
fn related_entity_ordering_packet(
    packet: &IndexPreparationPacket,
) -> Option<(
    crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    DerivedIndexId,
    crate::identity::data::KindId,
    crate::indexes::data::RelatedEntityEndpoint,
    crate::identity::data::KindId,
    Vec<crate::indexes::data::RelatedEntityOrderingField>,
)> {
    match &packet.definition.kind {
        DerivedIndexKind::RelatedEntityOrdering {
            relation_kind,
            parent_endpoint,
            child_kind,
            ordering,
        } => Some((
            packet.header.reduction_key,
            packet.header.identity.index_id,
            *relation_kind,
            *parent_endpoint,
            *child_kind,
            ordering.clone(),
        )),
        DerivedIndexKind::EntityField { .. }
        | DerivedIndexKind::RelationField { .. }
        | DerivedIndexKind::RelationJoin(_) => None,
    }
}

#[derive(Clone, Copy)]
struct RelationJoinPacket {
    reduction_key: crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    index_id: DerivedIndexId,
    definition: crate::indexes::data::RelationJoinDefinition,
}

fn relation_join_packet(packet: &IndexPreparationPacket) -> Option<RelationJoinPacket> {
    let DerivedIndexKind::RelationJoin(definition) = &packet.definition.kind else {
        return None;
    };
    Some(RelationJoinPacket {
        reduction_key: packet.header.reduction_key,
        index_id: packet.header.identity.index_id,
        definition: *definition,
    })
}

fn relation_join_result(
    _runtime: &RelationalRuntime,
    projection: &IndexProjectionSource<'_, '_>,
    packet: &RelationJoinPacket,
) -> crate::authority::commit::preparation::reduction::merge::OrderedReductionStream<
    crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    IndexPreparationResult,
> {
    singleton_result_stream(
        packet.reduction_key,
        packet.index_id,
        DerivedIndexEntries::RelationJoin(
            build_relation_join_index(projection, packet.definition).into(),
        ),
    )
}

fn singleton_result_stream(
    reduction_key: crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    index_id: DerivedIndexId,
    entries: DerivedIndexEntries,
) -> crate::authority::commit::preparation::reduction::merge::OrderedReductionStream<
    crate::authority::commit::preparation::reduction::keys::IndexReductionKey,
    IndexPreparationResult,
> {
    crate::authority::commit::preparation::reduction::merge::OrderedReductionStream::singleton(
        reduction_key,
        IndexPreparationResult {
            index_id,
            entries: Some(entries),
        },
    )
}
