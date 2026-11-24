//! Extend one unpublished canonical patch with the publication owner's consequences.
use super::*;
pub(crate) fn extend_patch(
    instrumentation: &RuntimeInstrumentation,
    work_budget: Option<&RequestWorkBudget>,
    patch: &mut CanonicalAuthoritativePatch,
    fragments: Vec<FoundationalPatchFragment>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError>
{
    let consequences = prepare_patch_fragments(instrumentation, work_budget, fragments, lease)?;
    let count = patch
        .authoritative_record_patches
        .len()
        .checked_add(consequences.len())
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "current ownership patch width overflow",
            )
        })?;
    // Ordering has the same key/record backing as ordinary direct assembly;
    // the record payloads move into this one merged packet.
    let bytes = count
        .checked_mul(
            std::mem::size_of::<PublishedAuthoritativeRecordPatch>()
                + std::mem::size_of::<
                    crate::authority::commit::preparation::reduction::keys::DiffReductionKey,
                >(),
        )
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "current ownership patch backing overflow",
            )
        })?;
    let _funding =
        worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, bytes as u64)
            .map_err(|cause| {
                crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
                    crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                            crate::execution::denial::lease_denial(cause),
                        ),
                        partition_identity: None,
                    },
                )
            })?;
    let mut records = std::mem::take(&mut patch.authoritative_record_patches);
    records.extend(consequences);
    patch.authoritative_record_patches = direct_diff_record_order(records);
    Ok(())
}
