use crate::authority::commit::preparation::planning::strategy::{
    coarse_preparation_packet_count, packet_width_is_profitable, MIN_PARALLEL_PACKET_WIDTH,
    TARGET_PREPARATION_ITEMS_PER_PACKET,
};
use crate::authority::commit::preparation::reduction::merge::{
    canonical_merge_streams, OrderedReductionStream,
};
use crate::authority::mutation::FoundationalPatchFragment;
use crate::config::data::RelationalExecutionModel;
use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::publication::patch::data::{
    CanonicalAuthoritativePatch, PatchOrdering, PatchPublicationMode,
    PublishedAuthoritativeRecordPatch,
};
use crate::transactions::data::RecordRef;
use rayon::prelude::*;
use std::collections::BTreeSet;

pub(super) fn assemble_patch(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    fragments: Vec<FoundationalPatchFragment>,
) -> CanonicalAuthoritativePatch {
    let records = prepare_patch_fragments(
        runtime,
        fragments
            .into_iter()
            .map(|fragment| fragment.published_record())
            .collect(),
    );
    CanonicalAuthoritativePatch {
        ordering: PatchOrdering::CanonicalCommitOrder,
        publication_mode: PatchPublicationMode::CommitNative,
        authoritative_record_patches: records,
    }
    .canonicalized()
}

fn prepare_patch_fragments(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    authoritative_record_patches: Vec<PublishedAuthoritativeRecordPatch>,
) -> Vec<PublishedAuthoritativeRecordPatch> {
    use crate::authority::commit::preparation::packets::diff::{
        DiffPreparationHeader, DiffPreparationPacket,
    };

    if authoritative_record_patches.is_empty() {
        return authoritative_record_patches;
    }

    let packet_count = coarse_preparation_packet_count(
        authoritative_record_patches.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET,
    );
    runtime.performance_access().count_preparation_packet_shape(
        packet_count,
        authoritative_record_patches.len(),
        authoritative_record_patches
            .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
            .map(|chunk| chunk.len())
            .max()
            .unwrap_or(0),
        authoritative_record_patches
            .iter()
            .map(|record| match record.target {
                RecordRef::Entity(entity_id) => entity_id.partition_id,
                RecordRef::Relation(relation_id) => relation_id.partition_id,
            })
            .collect::<BTreeSet<_>>()
            .len(),
    );

    let use_parallel_packets =
        matches!(
            runtime.config.execution.execution_model,
            RelationalExecutionModel::ParallelPreparation
        ) && packet_width_is_profitable(packet_count, MIN_PARALLEL_PACKET_WIDTH);

    if !use_parallel_packets {
        runtime
            .performance_access()
            .count_preparation_serial_strategy();
        return direct_diff_record_order(authoritative_record_patches);
    }

    runtime
        .performance_access()
        .count_preparation_parallel_legal();
    runtime
        .performance_access()
        .count_preparation_parallel_profitable();
    runtime
        .performance_access()
        .count_preparation_staged_parallel_strategy();

    let mut packets = Vec::with_capacity(packet_count);
    for (packet_index, chunk) in authoritative_record_patches
        .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
        .enumerate()
    {
        packets.push(DiffPreparationPacket {
            header: DiffPreparationHeader {
                packet_index_floor: packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET,
            },
            authoritative_record_patches: chunk.to_vec(),
        });
    }

    canonical_merge_streams(
        packets
            .par_iter()
            .map(diff_packet_stream)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|(_, record)| record)
    .collect()
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
            let canonical = record.canonicalized();
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

fn diff_packet_stream(
    packet: &crate::authority::commit::preparation::packets::diff::DiffPreparationPacket,
) -> OrderedReductionStream<
    crate::authority::commit::preparation::reduction::keys::DiffReductionKey,
    PublishedAuthoritativeRecordPatch,
> {
    use crate::authority::commit::preparation::reduction::keys::DiffReductionKey;

    let mut canonical_records = Vec::with_capacity(packet.authoritative_record_patches.len());
    let mut headers = Vec::with_capacity(packet.authoritative_record_patches.len());

    for (offset, record) in packet.authoritative_record_patches.iter().enumerate() {
        let canonical = record.canonicalized();
        let key = DiffReductionKey::new(
            canonical.target.clone(),
            diff_kind_order(
                crate::authority::commit::preparation::packets::diff::DiffFragmentKind::from(
                    canonical.structural_change,
                ),
            ),
            packet.header.packet_index_floor + offset,
        );
        headers.push((key, offset));
        canonical_records.push(canonical);
    }

    headers.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    let mut canonical_records = canonical_records.into_iter().map(Some).collect::<Vec<_>>();
    let mut stream = Vec::with_capacity(headers.len());
    for (key, record_index) in headers {
        let canonical = canonical_records[record_index]
            .take()
            .expect("diff packet header index must resolve exactly once");
        stream.push((key, canonical));
    }
    OrderedReductionStream::new(stream)
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
