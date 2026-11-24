//! Immutable descriptive effects of exactly the actual authored appends.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::patch_fragments::FoundationalPatchFragment;
use contracts::execution::request_stage::checkpoint;
use crate::mvcc::publication::application_publication::custody;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
struct Backing {
    patches: Vec<PublishedAuthoritativeRecordPatch>,
    // The original payer outlives every retained patch value.
    _memory: ExecutionMemoryReservation,
}
/// Descriptive writer attribution from the one Native candidate. It neither
/// grants publication custody nor reissues a source, settlement or capability.
#[derive(Clone, Debug)]
pub struct RelationalCoupledAuthoredSourceEffects(Arc<Backing>);
impl PartialEq for RelationalCoupledAuthoredSourceEffects {
    fn eq(&self, other: &Self) -> bool {
        self.0.patches == other.0.patches
    }
}
impl Eq for RelationalCoupledAuthoredSourceEffects {}
impl RelationalCoupledAuthoredSourceEffects {
    pub fn record_patches(&self) -> &[PublishedAuthoritativeRecordPatch] {
        &self.0.patches
    }
    pub(super) fn capture(
        execution: &ExecutionRequest<'_, '_>,
        fragments: &[FoundationalPatchFragment],
        ranges: impl Iterator<Item = std::ops::Range<usize>> + Clone,
    ) -> Result<Self, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                let overflow = || {
                    custody::execution_stop(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
                    )
                };
                let count = ranges.clone().map(|range| range.len()).sum::<usize>();
                let selected = || ranges.clone().flat_map(|range| fragments[range].iter());
                let mut bytes = (count as u64)
                    .checked_mul(size_of::<PublishedAuthoritativeRecordPatch>() as u64)
                    .and_then(|n| {
                        n.checked_add((size_of::<Backing>() + 2 * size_of::<usize>()) as u64)
                    })
                    .ok_or_else(overflow)?;
                for fragment in selected() {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    bytes = bytes
                        .checked_add(fragment.published_record_owned_allocation_capacity_bytes())
                        .ok_or_else(overflow)?;
                }
                let memory = ExecutionMemoryReservation::reserve_in_scope(
                    contracts::execution::request_custody::request_lease(execution),
                    bytes,
                )
                .map_err(|cause| {
                    custody::execution_stop(contracts::execution::denial::lease_denial(cause))
                })?;
                let mut patches = Vec::new();
                patches.try_reserve_exact(count).map_err(|_| {
                    custody::execution_stop(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                    )
                })?;
                for fragment in selected() {
                    checkpoint(
                        context,
                        fragment
                            .published_record_owned_allocation_capacity_bytes()
                            .checked_add(1)
                            .ok_or_else(overflow)?,
                    )
                    .map_err(custody::execution_stop)?;
                    patches.push(fragment.published_record());
                }
                Ok(Self(Arc::new(Backing {
                    patches,
                    _memory: memory,
                })))
            })
            .map_err(custody::execution_stop)?
            .0
    }
}
