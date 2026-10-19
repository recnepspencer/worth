//! Physical custody for a prepared candidate, before its writer can allocate.
use super::super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation};
use worth_query_declaration::facade::application_operation::ApplicationCandidateRequirements;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct CandidateFunding {
    _memory: Option<ExecutionMemoryReservation>,
}

impl CandidateFunding {
    pub(super) fn admit(
        requirements: ApplicationCandidateRequirements,
        sidecar: Option<ApplicationCandidateRequirements>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let bytes = quote(requirements)
            .and_then(|bytes| match sidecar {
                Some(sidecar) => bytes.checked_add(quote(sidecar)?),
                None => Some(bytes),
            })
            .ok_or_else(|| {
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
                    "candidate allocation layout is not representable",
                )
            })?;
        let reservation = policy.reserve_memory(bytes);
        let memory = reservation.map_err(|cause| {
            WorthQueryApplicationAttemptDenial::allocation_denied("prepared candidate", cause)
        })?;
        Ok(Self { _memory: memory })
    }
    pub(super) fn grow_correspondence(
        &mut self,
        bytes: u64,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some(memory) = &mut self._memory {
            let total = memory.bytes().checked_add(bytes).ok_or_else(|| {
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
                    "candidate correspondence layout overflow",
                )
            })?;
            memory.grow_for_allocation(total).map_err(|cause| {
                WorthQueryApplicationAttemptDenial::allocation_denied(
                    "candidate correspondence",
                    cause,
                )
            })?;
        }
        Ok(())
    }
}

fn quote(requirements: ApplicationCandidateRequirements) -> Option<u64> {
    let c = requirements.cardinality();
    let items = [
        c.maximum_creates(),
        c.maximum_deletes(),
        c.maximum_links(),
        c.maximum_unlinks(),
        c.maximum_writes(),
        c.maximum_emits(),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)?;
    // A candidate owns its funding/program Arc headers, sealed correspondence,
    // effect slots and field/key/write-position B-trees.
    // An insertion can own at most one full node per entry in each index;
    // quote the node's eleven entries and twelve edges, including alignment.
    let entry = std::mem::size_of::<(
        worth_foundational::facade::AspectFieldLocator,
        worth_foundational::facade::AspectValue,
    )>()
    .max(std::mem::size_of::<(String, usize)>());
    let node = 11usize
        .checked_mul(entry)?
        .checked_add(12 * std::mem::size_of::<usize>() + 4 * std::mem::size_of::<usize>())?;
    let slot = std::mem::size_of::<super::WorthQueryApplicationRealizedEffect>()
        .checked_add(3usize.checked_mul(node)?)?;
    let storage = items
        .checked_mul(slot)?
        .checked_add(std::mem::size_of::<CandidateFunding>() + 4 * std::mem::size_of::<usize>())?
        .checked_add(super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::sealed_storage_bound()?)?;
    // Native lowering can hold the old key/reference representation beside its
    // normalized replacement. Both remain funded until provider backing drops.
    let payload = requirements
        .resources()
        .maximum_retained_representation_bytes()
        .checked_mul(2)?;
    u64::try_from(storage.checked_add(payload)?).ok()
}

#[cfg(test)]
mod tests;
