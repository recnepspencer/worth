use worth_execution::ExecutionResourceLease;
use worth_foundational::PartitionIdentity;

use crate::authority::commit::preparation::packets::import::{
    ImportFragmentIdentity, ImportFragmentKind, ImportStagedRow, ImportStagingHeader,
    ImportStagingPacket,
};
use crate::authority::commit::preparation::planning::strategy::{
    coarse_preparation_packet_count, strategy_for_parallel_packets,
    TARGET_PREPARATION_ITEMS_PER_PACKET,
};
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationLocalityProof, PreparationPartitionScope, PreparationReadSetApproximation,
    PreparationRecordDomain, PreparationWriteExclusionClass,
};
use crate::authority::commit::preparation::reduction::keys::ImportReductionKey;
use crate::authority::commit::preparation::reduction::merge::{
    canonical_merge_streams, OrderedReductionStream,
};
use crate::authority::mutation::outcomes::{MutationOutcome, RecordMutation};
use crate::authority::mutation::record_changes::allocate_entity_with_extra;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    BulkEntityCreateIntent, BulkImportRowDomain, BulkImportStage, CommitConflict, CreatedEntityRef,
    RecordAspectPatchTarget, TransactionCommitError,
};
use crate::validation::data::InvariantGroupSet;
use worth_foundational::facade::{
    AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectState, PortablePatchReadmissionPurpose,
};

use super::field_authoring_candidate::FieldAuthoringDomain;
use super::record_aspect_patch;

pub(super) fn apply(
    intent: &BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<MutationOutcome, TransactionCommitError> {
    let version_id = workspace.version_id();
    let mut outcome = MutationOutcome::with_capacity(intent.field_patches.len(), 1);
    outcome.record_event(
        crate::authority::mutation::outcomes::MutationEvent::BulkEntitiesCreated {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            count: 0,
        },
    );
    let staged_rows = stage_bulk_entity_rows(intent, workspace, lease)?;
    let entity_aspect_plans = stage_bulk_entity_aspect_plans(intent, workspace, &staged_rows)?;
    for ((client_key, _fields), aspect_plan) in intent
        .client_keys
        .iter()
        .cloned()
        .zip(staged_rows.into_iter())
        .zip(entity_aspect_plans.into_iter())
    {
        let entity_id = workspace.with_context(|context| {
            let entity_id = allocate_entity_with_extra(
                context.state,
                context.record_allocations,
                version_id,
                intent.partition_id,
                intent.kind_id,
                crate::storage::substrate::EntityExtra {
                    authoritative_aspect_state: aspect_plan.1,
                    ..crate::storage::substrate::EntityExtra::default()
                },
            )?;
            context
                .state
                .mark_entity_slot_touched(entity_id.partition_id, entity_id.slot_index());
            Ok::<_, CommitConflict>(entity_id)
        })?;
        workspace.register_created_entity(
            CreatedEntityRef {
                partition_id: intent.partition_id,
                kind_id: intent.kind_id,
                client_key,
            },
            entity_id,
        );
        outcome.record_change(RecordMutation::EntityCreated {
            entity_id,
            kind_id: intent.kind_id,
            authoritative_patch: record_aspect_patch::published_patch(aspect_plan.0),
        });
    }
    outcome.set_last_event_count(intent.field_patches.len());
    Ok(outcome)
}

fn stage_bulk_entity_aspect_plans(
    intent: &BulkEntityCreateIntent,
    workspace: &MutationWorkspace<'_>,
    field_patches: &[crate::transactions::data::AspectFieldPatch],
) -> Result<
    Vec<(
        AuthoritativeRecordAspectPatch,
        Option<AuthoritativeRecordAspectState>,
    )>,
    CommitConflict,
> {
    let lowered_plan = workspace.entity_aspect_plan(intent.kind_id);
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.kind_id,
    };
    field_patches
        .iter()
        .map(|fields| {
            let patch = record_aspect_patch::readmit_field_authoring(
                fields,
                PortablePatchReadmissionPurpose::RecordCreation,
                lowered_plan,
                target,
                FieldAuthoringDomain::Entity,
            )?;
            let state = record_aspect_patch::apply(None, &patch, target)?;
            Ok((patch, state))
        })
        .collect()
}

fn stage_bulk_entity_rows(
    intent: &BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Vec<crate::transactions::data::AspectFieldPatch>, TransactionCommitError> {
    if let Some(lease) = lease {
        return stage_leased_entity_rows(intent, workspace, lease);
    }
    let packet_count = coarse_preparation_packet_count(
        intent.field_patches.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET,
    );
    let mut packets = Vec::with_capacity(packet_count);
    for (packet_index, field_patches) in intent
        .field_patches
        .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
        .enumerate()
    {
        let packet_index_floor = packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET;
        packets.push(ImportStagingPacket {
            header: ImportStagingHeader {
                packet_index_floor,
                identity: ImportFragmentIdentity {
                    partition_id: intent.partition_id,
                    kind_id: intent.kind_id,
                    fragment_kind: ImportFragmentKind::EntityCreate,
                    packet_index: packet_index_floor,
                },
                proof_kind: PreparationProofKind::FragmentIdentityDisjoint,
                locality: PreparationLocalityProof {
                    observation_scope:
                        crate::validation::engine::InvariantObservationKind::Speculative,
                    record_domain: PreparationRecordDomain::Entity,
                    partition_scope: PreparationPartitionScope::TouchedPartitions(
                        vec![intent.partition_id].into(),
                    ),
                    invariant_group_scope: InvariantGroupSet::empty(),
                    read_set_approximation: PreparationReadSetApproximation::TouchedOnly,
                    write_exclusion: PreparationWriteExclusionClass::PublicationExcluded,
                },
            },
            rows: field_patches
                .iter()
                .cloned()
                .map(|fields| ImportStagedRow::Entity { fields })
                .collect(),
        });
    }

    stage_import_packets(workspace, packets, lease)?
        .into_iter()
        .map(|row| match row {
            ImportStagedRow::Entity { fields } => Ok(fields),
            ImportStagedRow::Relation { .. } => {
                Err(TransactionCommitError::conflict(CommitConflict::new(
                    crate::transactions::data::ConflictClass::BulkImportDomainMismatch {
                        expected: BulkImportRowDomain::Entity,
                        actual: BulkImportRowDomain::Relation,
                        stage: BulkImportStage::EntityCreate,
                    },
                )))
            }
        })
        .collect()
}

fn stage_leased_entity_rows(
    intent: &BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<Vec<crate::transactions::data::AspectFieldPatch>, TransactionCommitError> {
    use crate::transactions::data::AspectFieldPatch;
    let packet_count = coarse_preparation_packet_count(
        intent.field_patches.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET,
    );
    if packet_count == 0 {
        return Ok(Vec::new());
    }
    workspace.record_preparation_strategy(
        packet_count,
        intent.field_patches.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET.min(intent.field_patches.len()),
        1,
        record_import_strategy(Some(lease), packet_count),
    );
    let packet_ceiling =
        lease.policy().budget().charged_memory_bytes() / (packet_count as u64).saturating_mul(8);
    let (inputs, _preparation_report) = crate::execution::prepare_borrowed_packets(
        lease,
        workspace.commit_work_budget(),
        |budget| {
            let mut inputs = Vec::new();
            for (packet_index, rows) in intent
                .field_patches
                .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
                .enumerate()
            {
                budget.claim_packet::<(usize, &[AspectFieldPatch])>()?;
                inputs
                    .try_reserve_exact(1)
                    .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
                inputs.push(crate::execution::ReadOnlyPacket {
                    identity: PartitionIdentity::new(
                        (packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET) as u64,
                    ),
                    value: (packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET, rows),
                    input_bytes: 0,
                    kernel_scratch_bytes: packet_ceiling,
                    max_result_bytes: packet_ceiling,
                });
            }
            Ok(inputs)
        },
    )?;
    let streams = crate::execution::execute_read_only_packets_with_budget(
        inputs,
        Some(lease),
        workspace.commit_work_budget(),
        |(packet_index_floor, rows), context| {
            let mut stream = Vec::new();
            for (offset, fields) in rows.iter().enumerate() {
                context.checkpoint(1)?;
                context.claim_result(
                    (std::mem::size_of::<(ImportReductionKey, ImportStagedRow)>() as u64)
                        .saturating_add(fields.owned_allocation_capacity_bytes()),
                )?;
                stream
                    .try_reserve_exact(1)
                    .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
                stream.push((
                    ImportReductionKey::new(intent.partition_id, 0, packet_index_floor + offset),
                    ImportStagedRow::Entity {
                        fields: fields.clone(),
                    },
                ));
            }
            Ok(OrderedReductionStream::new(stream))
        },
        |stream| {
            stream.owned_allocation_capacity_bytes(|_, row| row.owned_allocation_capacity_bytes())
        },
    )?;
    Ok(
        crate::authority::commit::preparation::reduction::merge::canonical_merge_streams_checked(
            streams,
            lease,
            workspace.commit_work_budget(),
            |_, row| row.owned_allocation_capacity_bytes(),
            |_| 0,
        )?
        .into_iter()
        .map(|(_, row)| match row {
            ImportStagedRow::Entity { fields } => fields,
            ImportStagedRow::Relation { .. } => unreachable!("entity packet only"),
        })
        .collect(),
    )
}

fn stage_import_packets(
    workspace: &mut MutationWorkspace<'_>,
    packets: Vec<ImportStagingPacket>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Vec<ImportStagedRow>, TransactionCommitError> {
    if packets.is_empty() {
        return Ok(Vec::new());
    }

    let packet_item_count = packets.iter().map(|packet| packet.rows.len()).sum();
    let packet_max_width = packets
        .iter()
        .map(|packet| packet.rows.len())
        .max()
        .unwrap_or(0);
    let strategy = record_import_strategy(lease, packets.len());
    workspace.record_preparation_strategy(
        packets.len(),
        packet_item_count,
        packet_max_width,
        1,
        strategy,
    );
    if lease.is_none() {
        return Ok(packets.into_iter().flat_map(|packet| packet.rows).collect());
    }
    let packet_ceiling = lease.map_or(0, |lease| {
        lease.policy().budget().charged_memory_bytes() / (packets.len() as u64).saturating_mul(8)
    });
    let inputs = packets
        .into_iter()
        .map(|packet| crate::execution::ReadOnlyPacket {
            identity: PartitionIdentity::new(packet.header.packet_index_floor as u64),
            input_bytes: (packet.rows.capacity() as u64)
                .saturating_mul(std::mem::size_of::<ImportStagedRow>() as u64)
                .saturating_add(
                    packet
                        .rows
                        .iter()
                        .map(ImportStagedRow::owned_allocation_capacity_bytes)
                        .sum::<u64>(),
                ),
            kernel_scratch_bytes: packet_ceiling,
            max_result_bytes: packet_ceiling,
            value: packet,
        })
        .collect();
    let streams = crate::execution::execute_read_only_packets(
        inputs,
        lease,
        import_packet_stream,
        |stream| {
            stream.owned_allocation_capacity_bytes(|_, row| row.owned_allocation_capacity_bytes())
        },
    )?;
    Ok(canonical_merge_streams(streams)
        .into_iter()
        .map(|(_, row)| row)
        .collect())
}

fn record_import_strategy(
    lease: Option<&ExecutionResourceLease<'_>>,
    packet_count: usize,
) -> crate::authority::commit::preparation::planning::strategy::PreparationStrategy {
    strategy_for_parallel_packets(lease, packet_count)
}

fn import_packet_stream(
    packet: &ImportStagingPacket,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<
    OrderedReductionStream<ImportReductionKey, ImportStagedRow>,
    worth_execution::MapKernelFailure<crate::execution::PacketBudgetDenial>,
> {
    let mut stream = Vec::new();
    for (offset, row) in packet.rows.iter().enumerate() {
        context.checkpoint(1)?;
        context.claim_result(
            (std::mem::size_of::<(ImportReductionKey, ImportStagedRow)>() as u64)
                .saturating_add(row.owned_allocation_capacity_bytes()),
        )?;
        stream
            .try_reserve_exact(1)
            .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
        stream.push((
            ImportReductionKey::new(
                packet.header.identity.partition_id,
                0,
                packet.header.packet_index_floor + offset,
            ),
            row.clone(),
        ));
    }
    Ok(OrderedReductionStream::new(stream))
}
