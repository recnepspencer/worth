use super::*;

pub(super) fn stage_leased_entity_rows(
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
    let packet_ceiling = lease
        .policy()
        .budget()
        .charged_memory_bytes()
        .map(|limit| limit / (packet_count as u64).saturating_mul(8));
    let (inputs, _preparation_report) = crate::execution::prepare_borrowed_packets(
        lease,
        workspace.commit_work_budget(),
        crate::execution::packet_preparation_bytes::<(usize, &[AspectFieldPatch])>(packet_count)?,
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
                let packet_ceiling = packet_ceiling.map_or_else(
                    || crate::authority::commit::preparation::packets::import::import_result_capacity_bytes(
                        rows.iter().map(AspectFieldPatch::owned_allocation_capacity_bytes),
                    ).ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded),
                    Ok,
                )?;
                inputs.push(crate::execution::ReadOnlyPacket {
                    identity: PartitionIdentity::new(
                        (packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET) as u64,
                    ),
                    value: (packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET, rows),
                    input_bytes: 0,
                    memory: crate::execution::PacketMemory::Declared {
                        scratch: packet_ceiling,
                        result: packet_ceiling,
                    },
                });
            }
            Ok(inputs)
        },
    )?;
    let streams = crate::execution::execute_read_only_packets_with_budget(
        inputs,
        lease,
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
