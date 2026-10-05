use crate::authority::commit::preparation::planning::strategy::{
    coarse_preparation_packet_count, packet_width_is_profitable, MIN_PARALLEL_PACKET_WIDTH,
    TARGET_PREPARATION_ITEMS_PER_PACKET,
};
#[cfg(test)]
use crate::authority::commit::preparation::reduction::merge::OrderedReductionStream;
use crate::authority::mutation::FoundationalPatchFragment;
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PatchOrdering, PatchPublicationMode,
    PublishedAuthoritativeRecordPatch,
};
use crate::transactions::data::RecordRef;
use std::collections::BTreeSet;
use worth_execution::ExecutionResourceLease;
use worth_foundational::PartitionIdentity;

mod checked_fragments;

pub(super) fn assemble_patch(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    fragments: Vec<FoundationalPatchFragment>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<CanonicalAuthoritativePatch, crate::transactions::data::TransactionCommitError> {
    let records = prepare_patch_fragments(runtime, fragments, lease)?;
    Ok(CanonicalAuthoritativePatch {
        ordering: PatchOrdering::CanonicalCommitOrder,
        publication_mode: PatchPublicationMode::CommitNative,
        authoritative_record_patches: records,
    })
}

fn prepare_patch_fragments(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    fragments: Vec<FoundationalPatchFragment>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Vec<PublishedAuthoritativeRecordPatch>, crate::transactions::data::TransactionCommitError>
{
    if fragments.is_empty() {
        return Ok(Vec::new());
    }

    let packet_count =
        coarse_preparation_packet_count(fragments.len(), TARGET_PREPARATION_ITEMS_PER_PACKET);
    let use_parallel_packets =
        lease.is_some_and(|lease| {
            lease.resolved_posture() == worth_foundational::ExecutionPosture::Automatic
        }) && packet_width_is_profitable(packet_count, MIN_PARALLEL_PACKET_WIDTH);

    if lease.is_none() {
        record_patch_shape(
            runtime,
            fragments.len(),
            fragment_partition_count(&fragments),
        );
        runtime
            .performance_access()
            .count_preparation_serial_strategy();
        return Ok(direct_diff_record_order(
            fragments
                .into_iter()
                .map(FoundationalPatchFragment::into_published_record)
                .collect(),
        ));
    }

    let lease = lease.expect("serial path returns before packet preparation");
    let packet_ceiling =
        lease.policy().budget().charged_memory_bytes() / (packet_count as u64).saturating_mul(8);
    let mut partition_count = None;
    let (inputs, _preparation_report) = crate::execution::prepare_borrowed_packets(
        lease,
        runtime.commit_work_budget.as_ref(),
        |budget| {
            let mut partitions = BTreeSet::new();
            for fragment in &fragments {
                budget.checkpoint(1)?;
                let partition = fragment_partition(fragment);
                if !partitions.contains(&partition) {
                    budget.claim(
                        (std::mem::size_of::<crate::identity::data::PartitionId>()
                            + 4 * std::mem::size_of::<usize>()) as u64,
                    )?;
                    partitions.insert(partition);
                }
            }
            partition_count = Some(partitions.len());
            let mut inputs = Vec::new();
            for (packet_index, chunk) in fragments
                .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
                .enumerate()
            {
                budget.claim_packet::<(usize, &[FoundationalPatchFragment])>()?;
                inputs
                    .try_reserve_exact(1)
                    .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
                let packet_index_floor = packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET;
                inputs.push(crate::execution::ReadOnlyPacket {
                    identity: PartitionIdentity::new(packet_index_floor as u64),
                    input_bytes: 0,
                    kernel_scratch_bytes: packet_ceiling,
                    max_result_bytes: packet_ceiling,
                    value: (packet_index_floor, chunk),
                });
            }
            Ok(inputs)
        },
    )?;
    record_patch_shape(
        runtime,
        fragments.len(),
        partition_count.expect("checked partition fold"),
    );
    if use_parallel_packets {
        runtime
            .performance_access()
            .count_preparation_parallel_legal();
        runtime
            .performance_access()
            .count_preparation_parallel_profitable();
        runtime
            .performance_access()
            .count_preparation_staged_parallel_strategy();
    } else {
        runtime
            .performance_access()
            .count_preparation_serial_strategy();
    }
    let streams = crate::execution::execute_read_only_packets_with_budget(
        inputs,
        Some(lease),
        runtime.commit_work_budget.as_ref(),
        |(floor, records), context| {
            checked_fragments::diff_fragment_slice_stream(*floor, records, context)
        },
        |stream| {
            stream.owned_allocation_capacity_bytes(|_, record| {
                record.owned_allocation_capacity_bytes()
            })
        },
    )
    .map_err(|stop| crate::transactions::data::TransactionCommitError::execution(stop.into()))?;
    crate::authority::commit::preparation::reduction::merge::canonical_merge_values_checked(
        streams,
        lease,
        runtime.commit_work_budget.as_ref(),
        |_, record| record.owned_allocation_capacity_bytes(),
        |_| 0,
    )
    .map_err(Into::into)
}

fn record_patch_shape(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    record_count: usize,
    partition_count: usize,
) {
    runtime.performance_access().count_preparation_packet_shape(
        coarse_preparation_packet_count(record_count, TARGET_PREPARATION_ITEMS_PER_PACKET),
        record_count,
        record_count.min(TARGET_PREPARATION_ITEMS_PER_PACKET),
        partition_count,
    );
}

fn fragment_partition(fragment: &FoundationalPatchFragment) -> crate::identity::data::PartitionId {
    match fragment.target {
        RecordRef::Entity(entity_id) => entity_id.partition_id,
        RecordRef::Relation(relation_id) => relation_id.partition_id,
    }
}

fn fragment_partition_count(fragments: &[FoundationalPatchFragment]) -> usize {
    fragments
        .iter()
        .map(fragment_partition)
        .collect::<BTreeSet<_>>()
        .len()
}

fn direct_diff_record_order(
    authoritative_record_patches: Vec<PublishedAuthoritativeRecordPatch>,
) -> Vec<PublishedAuthoritativeRecordPatch> {
    use crate::authority::commit::preparation::packets::diff::DiffFragmentKind;
    use crate::authority::commit::preparation::reduction::keys::DiffReductionKey;

    let mut keyed_records = authoritative_record_patches
        .into_iter()
        .enumerate()
        .map(|(record_index, record)| {
            let canonical = record.into_canonicalized();
            let key = DiffReductionKey::new(
                canonical.target.clone(),
                diff_kind_order(DiffFragmentKind::from(canonical.structural_change)),
                record_index,
            );
            (key, canonical)
        })
        .collect::<Vec<_>>();
    keyed_records.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    keyed_records
        .into_iter()
        .map(|(_key, record)| record)
        .collect()
}

#[cfg(test)]
fn diff_packet_stream(
    packet: &crate::authority::commit::preparation::packets::diff::DiffPreparationPacket,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<
    OrderedReductionStream<
        crate::authority::commit::preparation::reduction::keys::DiffReductionKey,
        PublishedAuthoritativeRecordPatch,
    >,
    worth_execution::MapKernelFailure<crate::execution::PacketBudgetDenial>,
> {
    diff_slice_stream(
        packet.header.packet_index_floor,
        &packet.authoritative_record_patches,
        context,
    )
}

#[cfg(test)]
fn diff_slice_stream(
    packet_index_floor: usize,
    records: &[PublishedAuthoritativeRecordPatch],
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<
    OrderedReductionStream<
        crate::authority::commit::preparation::reduction::keys::DiffReductionKey,
        PublishedAuthoritativeRecordPatch,
    >,
    worth_execution::MapKernelFailure<crate::execution::PacketBudgetDenial>,
> {
    use crate::authority::commit::preparation::reduction::keys::DiffReductionKey;

    let mut stream = Vec::new();

    for (offset, record) in records.iter().enumerate() {
        context.checkpoint(
            1_u64
                .saturating_add(record.authoritative_patch.full_grammar_operation_count() as u64)
                .saturating_add(record.semantic_changes.len() as u64),
        )?;
        context.claim_result(
            (std::mem::size_of::<(DiffReductionKey, PublishedAuthoritativeRecordPatch)>() as u64)
                .saturating_add(record.owned_allocation_capacity_bytes()),
        )?;
        stream
            .try_reserve_exact(1)
            .map_err(|_| worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
        let canonical = record.canonicalized();
        let key = DiffReductionKey::new(
            canonical.target.clone(),
            diff_kind_order(
                crate::authority::commit::preparation::packets::diff::DiffFragmentKind::from(
                    canonical.structural_change,
                ),
            ),
            packet_index_floor + offset,
        );
        stream.push((key, canonical));
    }

    context.checkpoint((stream.len() as u64).saturating_mul(5))?;
    stream.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    Ok(OrderedReductionStream::new(stream))
}

fn diff_kind_order(
    kind: crate::authority::commit::preparation::packets::diff::DiffFragmentKind,
) -> u8 {
    match kind {
        crate::authority::commit::preparation::packets::diff::DiffFragmentKind::Created => 0,
        crate::authority::commit::preparation::packets::diff::DiffFragmentKind::Updated => 1,
        crate::authority::commit::preparation::packets::diff::DiffFragmentKind::Deleted => 2,
        crate::authority::commit::preparation::packets::diff::DiffFragmentKind::RetainedForAudit => 3,
        crate::authority::commit::preparation::packets::diff::DiffFragmentKind::MaterializationSuspended => 4,
        crate::authority::commit::preparation::packets::diff::DiffFragmentKind::Rematerialized => 5,
    }
}

pub(super) fn diagnostics_summary_artifact(
    config: &crate::config::data::RelationalRuntimeConfig,
    reserved_entries: Vec<RelationalDiagnosticsEntry>,
    entries: Vec<RelationalDiagnosticsEntry>,
) -> RelationalDiagnosticArtifact {
    let max_entries = config.diagnostics.profile.max_entries_per_artifact;
    // Reserved entries are proof-carrying summary surfaces for the authoritative lifecycle.
    // They must survive even when the optional diagnostics budget is exhausted or configured
    // below the reserved-entry count.
    let mut kept = reserved_entries;
    let remaining_capacity = max_entries.saturating_sub(kept.len());
    kept.extend(entries.into_iter().take(remaining_capacity));
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::Transaction,
        DiagnosticsArtifactKind::MinimalSummary,
        DeterminismExpectation::Required,
        kept,
    )
}

#[cfg(test)]
mod tests;
