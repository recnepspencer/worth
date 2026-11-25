use worth_relational_contracts::internal_port as contracts;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatchOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::authority::commit::preparation::planning::strategy::coarse_preparation_packet_count;
use contracts::authority::commit::preparation::planning::strategy::packet_width_is_profitable;
use contracts::authority::commit::preparation::planning::strategy::MIN_PARALLEL_PACKET_WIDTH;
use contracts::authority::commit::preparation::planning::strategy::TARGET_PREPARATION_ITEMS_PER_PACKET;
#[cfg(test)]
use worth_relational_storage::internal_port::authority::commit::preparation::reduction::merge::OrderedReductionStream;
use worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::patch_fragments::FoundationalPatchFragment;
use contracts::diagnostics::data::artifacts::DeterminismExpectation;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::execution::request_work::RequestWorkBudget;
use contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use contracts::publication::patch::data::authoritative_patch_envelope::CanonicalAuthoritativePatch;
use contracts::publication::patch::data::patch_ordering::PatchOrdering;
use contracts::publication::patch::data::patch_ordering::PatchPublicationMode;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use contracts::transactions::data::primitives::RecordRef;
use std::collections::BTreeSet;
use worth_execution::ExecutionResourceLease;
use worth_foundational::PartitionIdentity;

pub mod checked_fragments;
pub mod consequence_patch;
pub(crate) use consequence_patch::extend_patch;

pub(super) fn assemble_patch(
    instrumentation: &RuntimeInstrumentation,
    work_budget: Option<&RequestWorkBudget>,
    fragments: Vec<FoundationalPatchFragment>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<
    CanonicalAuthoritativePatch,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    let records = prepare_patch_fragments(instrumentation, work_budget, fragments, lease)?;
    Ok(CanonicalAuthoritativePatch {
        ordering: PatchOrdering::CanonicalCommitOrder,
        publication_mode: PatchPublicationMode::CommitNative,
        authoritative_record_patches: records,
    })
}

fn prepare_patch_fragments(
    instrumentation: &RuntimeInstrumentation,
    work_budget: Option<&RequestWorkBudget>,
    fragments: Vec<FoundationalPatchFragment>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<
    Vec<PublishedAuthoritativeRecordPatch>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
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
            instrumentation,
            fragments.len(),
            fragment_partition_count(&fragments),
        );
        instrumentation.count(RuntimeComplexityCounters::count_preparation_serial_strategy);
        return Ok(direct_diff_record_order(
            fragments
                .into_iter()
                .map(FoundationalPatchFragment::into_published_record)
                .collect(),
        ));
    }

    let lease = lease.expect(/* Proof: The no-lease serial branch returns above; only the Some(lease) branch reaches packet preparation. */ "serial path returns before packet preparation");
    let preparation_bytes = contracts::execution::packet_preparation::packet_preparation_bytes::<(
        usize,
        &[FoundationalPatchFragment],
    )>(packet_count)?
    .checked_add(
        (fragments.len() as u64)
            .checked_mul(
                (std::mem::size_of::<contracts::identity::data::primitives::PartitionId>()
                    + 4 * std::mem::size_of::<usize>()) as u64,
            )
            .ok_or(
                contracts::execution::read_only_packets::PacketExecutionStop::Admission(
                    contracts::execution::read_only_packets::PacketAdmissionDenial::MemoryOverflow,
                ),
            )?,
    )
    .ok_or(
        contracts::execution::read_only_packets::PacketExecutionStop::Admission(
            contracts::execution::read_only_packets::PacketAdmissionDenial::MemoryOverflow,
        ),
    )?;
    let mut partition_count = None;
    let (inputs, _preparation_report) =
        contracts::execution::packet_preparation::prepare_borrowed_packets(
            lease,
            work_budget,
            preparation_bytes,
            |budget| {
                let mut partitions = BTreeSet::new();
                for fragment in &fragments {
                    budget.checkpoint(1)?;
                    let partition = fragment_partition(fragment);
                    if !partitions.contains(&partition) {
                        budget.claim(
                        (std::mem::size_of::<contracts::identity::data::primitives::PartitionId>()
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
                    let (scratch_bytes, result_bytes) = checked_fragments::memory_bound(chunk);
                    inputs.push(contracts::execution::read_only_packets::ReadOnlyPacket {
                        identity: PartitionIdentity::new(packet_index_floor as u64),
                        input_bytes: 0,
                        memory: contracts::execution::packet_memory::PacketMemory::Declared {
                            scratch: scratch_bytes,
                            result: result_bytes,
                        },
                        value: (packet_index_floor, chunk),
                    });
                }
                Ok(inputs)
            },
        )?;
    record_patch_shape(
        instrumentation,
        fragments.len(),
        partition_count.expect(/* Proof: The checked fold fills this output on success; every stopped fold returns before this extraction. */ "checked partition fold"),
    );
    if use_parallel_packets {
        instrumentation.count(RuntimeComplexityCounters::count_preparation_parallel_legal);
        instrumentation.count(RuntimeComplexityCounters::count_preparation_parallel_profitable);
        instrumentation
            .count(RuntimeComplexityCounters::count_preparation_staged_parallel_strategy);
    } else {
        instrumentation.count(RuntimeComplexityCounters::count_preparation_serial_strategy);
    }
    let streams = contracts::execution::read_only_packets::execute_read_only_packets_with_budget(
        inputs,
        lease,
        work_budget,
        |(floor, records), context| {
            checked_fragments::diff_fragment_slice_stream(*floor, records, context)
        },
        |stream| {
            stream.owned_allocation_capacity_bytes(|_, record| {
                record.owned_allocation_capacity_bytes()
            })
        },
    )
    .map_err(|stop| contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(stop.into()))?;
    worth_relational_storage::internal_port::authority::commit::preparation::reduction::merge::checked_engine::canonical_merge_values_checked(
        streams,
        lease,
        work_budget,
        |_, record| record.owned_allocation_capacity_bytes(),
        |_| 0,
    )
    .map_err(Into::into)
}

fn record_patch_shape(
    instrumentation: &RuntimeInstrumentation,
    record_count: usize,
    partition_count: usize,
) {
    instrumentation.count(|counters| {
        counters.count_preparation_packet_shape(
            coarse_preparation_packet_count(record_count, TARGET_PREPARATION_ITEMS_PER_PACKET),
            record_count,
            record_count.min(TARGET_PREPARATION_ITEMS_PER_PACKET),
            partition_count,
        );
    });
}

fn fragment_partition(
    fragment: &FoundationalPatchFragment,
) -> contracts::identity::data::primitives::PartitionId {
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
    use worth_relational_mutation::internal_port::authority::commit::preparation::reduction::keys::DiffReductionKey;

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
    context: &mut contracts::execution::packet_memory::PacketKernelContext<'_, '_, '_>,
) -> Result<
    OrderedReductionStream<
        worth_relational_mutation::internal_port::authority::commit::preparation::reduction::keys::DiffReductionKey,
        PublishedAuthoritativeRecordPatch,
    >,
    worth_execution::MapKernelFailure<contracts::execution::read_only_packets::PacketBudgetDenial>,
>{
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
    context: &mut contracts::execution::packet_memory::PacketKernelContext<'_, '_, '_>,
) -> Result<
    OrderedReductionStream<
        worth_relational_mutation::internal_port::authority::commit::preparation::reduction::keys::DiffReductionKey,
        PublishedAuthoritativeRecordPatch,
    >,
    worth_execution::MapKernelFailure<contracts::execution::read_only_packets::PacketBudgetDenial>,
>{
    use worth_relational_mutation::internal_port::authority::commit::preparation::reduction::keys::DiffReductionKey;

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
    max_entries: usize,
    reserved_entries: Vec<RelationalDiagnosticsEntry>,
    entries: Vec<RelationalDiagnosticsEntry>,
) -> RelationalDiagnosticArtifact {
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
pub mod tests;
