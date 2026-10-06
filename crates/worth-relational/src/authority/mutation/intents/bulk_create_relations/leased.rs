use super::*;

pub(super) fn stage_leased_relation_rows(
    intent: &BulkRelationCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<Vec<ImportStagedRow>, TransactionCommitError> {
    let packet_count = coarse_preparation_packet_count(
        intent.endpoints.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET,
    );
    if packet_count == 0 {
        return Ok(Vec::new());
    }
    workspace.record_preparation_strategy(
        packet_count,
        intent.endpoints.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET.min(intent.endpoints.len()),
        1,
        strategy_for_parallel_packets(Some(lease), packet_count),
    );
    let packet_ceiling =
        lease.policy().budget().charged_memory_bytes() / (packet_count as u64).saturating_mul(8);
    let (inputs, _preparation_report) = crate::execution::prepare_borrowed_packets(
        lease,
        workspace.commit_work_budget(),
        |budget| {
            let mut inputs = Vec::new();
            for (packet_index, endpoints) in intent
                .endpoints
                .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
                .enumerate()
            {
                budget.claim_packet::<(usize, &[(EntityReference, EntityReference)])>()?;
                inputs
                    .try_reserve_exact(1)
                    .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
                inputs.push(crate::execution::ReadOnlyPacket {
                    identity: PartitionIdentity::new(
                        (packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET) as u64,
                    ),
                    value: (
                        packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET,
                        endpoints,
                    ),
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
        |(packet_index_floor, endpoints), context| {
            let mut stream = Vec::new();
            for (offset, (source, target)) in endpoints.iter().enumerate() {
                context.checkpoint(1)?;
                let row_index = packet_index_floor + offset;
                let reference_bytes = |reference: &EntityReference| match reference {
                    EntityReference::Existing(_) => 0,
                    EntityReference::Created(created) => {
                        created.client_key.owned_allocation_capacity_bytes()
                    }
                };
                let owned_bytes = intent
                    .client_keys
                    .get(row_index)
                    .map_or(0, |key| key.owned_allocation_capacity_bytes())
                    .saturating_add(reference_bytes(source))
                    .saturating_add(reference_bytes(target))
                    .saturating_add(
                        intent
                            .field_patches
                            .get(row_index)
                            .map_or(0, AspectFieldPatch::owned_allocation_capacity_bytes),
                    );
                context.claim_result(
                    (std::mem::size_of::<(ImportReductionKey, ImportStagedRow)>() as u64)
                        .saturating_add(owned_bytes),
                )?;
                stream
                    .try_reserve_exact(1)
                    .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
                stream.push((
                    ImportReductionKey::new(intent.partition_id, 1, row_index),
                    ImportStagedRow::Relation {
                        client_key: intent.client_keys.get(row_index).cloned(),
                        source: source.clone(),
                        target: target.clone(),
                        fields: intent
                            .field_patches
                            .get(row_index)
                            .cloned()
                            .unwrap_or_default(),
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
        .map(|(_, row)| row)
        .collect(),
    )
}
