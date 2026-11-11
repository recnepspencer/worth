//! Only independent merge header/string/parent storage; intrinsic leaves share.
use crate::history::data::{RelationalCommitReceipt, RelationalMergeBranchBasis};
use crate::merge::data::NormalizedRelationalMergeRequest;
use crate::transactions::data::MergeExecutionSummary;

pub(super) fn arc_slots<T>(count: usize) -> Option<usize> {
    Some(
        std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<T>(count).ok()?)
            .ok()?
            .0
            .pad_to_align()
            .size(),
    )
}
fn receipt(value: &RelationalCommitReceipt) -> Option<usize> {
    value
        .branch_id
        .0
        .len()
        .checked_add(super::super::normalized_plan::slots::<
            crate::history::data::CommitId,
        >(value.parents.len())?)
}
fn basis(value: &RelationalMergeBranchBasis) -> Option<usize> {
    // Supporting ancestor buffers already retain their own shared leaf grants.
    receipt(value.source_head())?
        .checked_add(receipt(value.target_head())?)?
        .checked_add(receipt(value.merge_base().commit())?)
}
fn request(value: &NormalizedRelationalMergeRequest) -> Option<usize> {
    value
        .target_branch()
        .0
        .len()
        .checked_add(value.source_branch().0.len())?
        .checked_add(value.request_digest().len())
}
pub(super) fn summary(value: &MergeExecutionSummary) -> Option<usize> {
    let correspondence = &value.correspondence_witness;
    let schema = &value.schema_reconciliation_witness;
    let strategy = &value.strategy_witness;
    let packet = &value.proof_packet;
    let bytes = request(&value.request)?
        .checked_add(basis(&value.branch_basis)?)?
        .checked_add(request(packet.request())?)?
        .checked_add(basis(packet.branch_basis())?)?;
    [
        correspondence.request_digest(),
        correspondence.branch_basis_digest(),
        correspondence.witness_digest(),
        schema.request_digest(),
        schema.branch_basis_digest(),
        schema.witness_digest(),
        strategy.request_digest(),
        strategy.branch_basis_digest(),
        strategy.witness_digest(),
        packet.correspondence_witness_digest(),
        packet.schema_reconciliation_witness_digest(),
        packet.strategy_witness_digest(),
        packet.foundational_request_lowering_digest(),
        packet.admitted_merge_surface_digest(),
        packet.planning_digest(),
        packet.execution_digest(),
        packet.packet_digest(),
        &value.diagnostics_digest,
        &value.execution_digest,
    ]
    .into_iter()
    .try_fold(bytes, |bytes, text| bytes.checked_add(text.len()))
}
