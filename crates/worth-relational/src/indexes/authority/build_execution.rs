use worth_execution::ExecutionResourceLease;
use worth_foundational::PartitionIdentity;

use crate::authority::commit::preparation::packets::index::IndexPreparationPacket;
use crate::authority::commit::preparation::planning::strategy::{
    ParallelLegality, ParallelProfitability, PreparationStrategy, PreparationStrategySelection,
};
use crate::authority::commit::preparation::reduction::keys::IndexReductionKey;
use crate::authority::commit::preparation::reduction::merge::{
    canonical_merge_streams, canonical_merge_streams_checked, OrderedReductionStream,
};
use crate::execution::{
    execute_read_only_packets_with_budget, PacketBudgetDenial, PacketExecutionStop,
    PacketKernelContext, ReadOnlyPacket, RequestWorkBudget,
};
use crate::indexes::data::{
    DerivedIndexEntries, DerivedIndexExecutionDenial, DerivedIndexId, DerivedIndexKind,
};
use crate::runtime::RelationalRuntime;

use super::super::projected_field_values::{
    build_entity_aspect_field_index, build_entity_aspect_field_index_checked,
    build_related_entity_ordering_index, build_related_entity_ordering_index_checked,
    build_relation_aspect_field_index, build_relation_aspect_field_index_checked,
    build_relation_join_index, build_relation_join_index_checked, checked_finish,
    IndexProjectionSource, RelatedEntityOrderingProjection,
};

#[derive(Debug, Clone)]
pub(super) struct IndexPreparationResult {
    pub(super) index_id: DerivedIndexId,
    pub(super) entries: Option<DerivedIndexEntries>,
}

type IndexResultStream = OrderedReductionStream<IndexReductionKey, IndexPreparationResult>;

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
    if strategy.parallel_legality == ParallelLegality::ProvenParallel {
        runtime
            .performance_access()
            .count_preparation_parallel_legal();
    }
    if strategy.parallel_profitability == ParallelProfitability::Profitable {
        runtime
            .performance_access()
            .count_preparation_parallel_profitable();
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
    projection: &IndexProjectionSource<'_, '_>,
    packets: Vec<IndexPreparationPacket>,
    lease: Option<&ExecutionResourceLease<'_>>,
    work_budget: Option<&RequestWorkBudget>,
) -> Result<Vec<(IndexReductionKey, IndexPreparationResult)>, DerivedIndexExecutionDenial> {
    // The definition key is the stable partition identity. Sorting here also
    // prevents caller request order from affecting worker assignment or merge.
    let mut ordered = packets;
    ordered.sort_by_key(|packet| (packet.definition.index_id.0, packet.header.packet_index));
    let packet_ceiling = lease.map_or(0, |lease| {
        lease.policy().budget().charged_memory_bytes()
            / (ordered.len().max(1) as u64).saturating_mul(8)
    });
    let inputs = ordered
        .into_iter()
        .enumerate()
        .map(|(ordinal, packet)| ReadOnlyPacket {
            identity: PartitionIdentity::new(ordinal as u64),
            input_bytes: packet.definition.owned_allocation_capacity_bytes(),
            kernel_scratch_bytes: packet_ceiling,
            max_result_bytes: packet_ceiling,
            value: packet,
        })
        .collect();
    execute_index_inputs(projection, inputs, lease, work_budget)
}

pub(super) fn execute_index_inputs(
    projection: &IndexProjectionSource<'_, '_>,
    inputs: Vec<ReadOnlyPacket<IndexPreparationPacket>>,
    lease: Option<&ExecutionResourceLease<'_>>,
    work_budget: Option<&RequestWorkBudget>,
) -> Result<Vec<(IndexReductionKey, IndexPreparationResult)>, DerivedIndexExecutionDenial> {
    let streams = execute_read_only_packets_with_budget(
        inputs,
        lease,
        work_budget,
        |packet, context| {
            if lease.is_some() {
                index_packet_result_checked(projection, packet, context)
            } else {
                Ok(index_packet_result(projection, packet))
            }
        },
        |stream: &IndexResultStream| {
            stream.owned_allocation_capacity_bytes(|_, result| {
                result
                    .entries
                    .as_ref()
                    .map_or(0, DerivedIndexEntries::owned_allocation_capacity_bytes)
            })
        },
    )
    .map_err(index_execution_denial)?;
    let merged = if let Some(lease) = lease {
        canonical_merge_streams_checked(
            streams,
            lease,
            work_budget,
            |_, result: &IndexPreparationResult| {
                result
                    .entries
                    .as_ref()
                    .map_or(0, DerivedIndexEntries::owned_allocation_capacity_bytes)
            },
            |_| 0,
        )
        .map_err(index_execution_denial)?
    } else {
        canonical_merge_streams(streams)
    };
    Ok(merged)
}

fn index_packet_result_checked(
    projection: &IndexProjectionSource<'_, '_>,
    packet: &IndexPreparationPacket,
    context: &mut PacketKernelContext<'_, '_, '_>,
) -> Result<IndexResultStream, worth_execution::MapKernelFailure<PacketBudgetDenial>> {
    let entries = match &packet.definition.kind {
        DerivedIndexKind::EntityField { field_locator } => {
            DerivedIndexEntries::EntityField(checked_finish(
                build_entity_aspect_field_index_checked(projection, field_locator, context)?,
                context,
            )?)
        }
        DerivedIndexKind::RelationField { field_locator } => {
            DerivedIndexEntries::RelationField(checked_finish(
                build_relation_aspect_field_index_checked(projection, field_locator, context)?,
                context,
            )?)
        }
        DerivedIndexKind::RelatedEntityOrdering {
            relation_kind,
            parent_endpoint,
            child_kind,
            ordering,
        } => {
            let contract = RelatedEntityOrderingProjection::new(
                *relation_kind,
                *parent_endpoint,
                *child_kind,
                ordering,
            );
            DerivedIndexEntries::RelatedEntityOrdering(checked_finish(
                build_related_entity_ordering_index_checked(projection, &contract, context)?,
                context,
            )?)
        }
        DerivedIndexKind::RelationJoin(definition) => {
            DerivedIndexEntries::RelationJoin(checked_finish(
                build_relation_join_index_checked(projection, *definition, context)?,
                context,
            )?)
        }
    };
    context.checkpoint(1)?;
    Ok(OrderedReductionStream::singleton(
        packet.header.reduction_key,
        IndexPreparationResult {
            index_id: packet.definition.index_id,
            entries: Some(entries),
        },
    ))
}

fn index_packet_result(
    projection: &IndexProjectionSource<'_, '_>,
    packet: &IndexPreparationPacket,
) -> IndexResultStream {
    let entries = match &packet.definition.kind {
        DerivedIndexKind::EntityField { field_locator } => DerivedIndexEntries::EntityField(
            build_entity_aspect_field_index(projection, field_locator).into(),
        ),
        DerivedIndexKind::RelationField { field_locator } => DerivedIndexEntries::RelationField(
            build_relation_aspect_field_index(projection, field_locator).into(),
        ),
        DerivedIndexKind::RelatedEntityOrdering {
            relation_kind,
            parent_endpoint,
            child_kind,
            ordering,
        } => DerivedIndexEntries::RelatedEntityOrdering(
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
        DerivedIndexKind::RelationJoin(definition) => DerivedIndexEntries::RelationJoin(
            build_relation_join_index(projection, *definition).into(),
        ),
    };
    OrderedReductionStream::singleton(
        packet.header.reduction_key,
        IndexPreparationResult {
            index_id: packet.definition.index_id,
            entries: Some(entries),
        },
    )
}

pub(super) fn index_execution_denial(stop: PacketExecutionStop) -> DerivedIndexExecutionDenial {
    stop.into()
}
