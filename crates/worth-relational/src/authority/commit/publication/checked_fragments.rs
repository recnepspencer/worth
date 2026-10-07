use crate::authority::commit::preparation::reduction::keys::DiffReductionKey;
use crate::authority::commit::preparation::reduction::merge::OrderedReductionStream;
use crate::authority::mutation::FoundationalPatchFragment;
use crate::execution::{PacketBudgetDenial, PacketKernelContext};
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use worth_execution::MapKernelFailure;

pub(super) fn diff_fragment_slice_stream(
    packet_index_floor: usize,
    fragments: &[FoundationalPatchFragment],
    context: &mut PacketKernelContext<'_, '_, '_>,
) -> Result<
    OrderedReductionStream<DiffReductionKey, PublishedAuthoritativeRecordPatch>,
    MapKernelFailure<PacketBudgetDenial>,
> {
    let mut stream = Vec::new();
    for (offset, fragment) in fragments.iter().enumerate() {
        let retained_bytes = fragment.published_record_owned_allocation_capacity_bytes();
        let operation_count = fragment.published_patch.full_grammar_operation_count() as u64;
        let semantic_count = fragment.semantic_changes.len() as u64;
        context.checkpoint(
            1_u64
                .saturating_add(operation_count)
                .saturating_add(semantic_count.saturating_mul(8)),
        )?;
        context.claim_result(
            (std::mem::size_of::<(DiffReductionKey, PublishedAuthoritativeRecordPatch)>() as u64)
                .saturating_add(retained_bytes),
        )?;
        // Canonical semantic-key comparison creates two temporary strings.
        // Their borrowed fields are included in retained_bytes; numeric labels
        // and vector headers have an additional fixed allowance.
        context.check_scratch_peak(retained_bytes.saturating_mul(2).saturating_add(1024))?;
        stream
            .try_reserve_exact(1)
            .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
        let canonical = fragment.published_record().into_canonicalized();
        let key = DiffReductionKey::new(
            canonical.target.clone(),
            super::diff_kind_order(
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
