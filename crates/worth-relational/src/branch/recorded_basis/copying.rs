//! Quote the actual owner description before canonicalization copies its heap.
use super::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) struct CanonicalDescription<'a> {
    pub(super) receipt: &'a RelationalCommitReceipt,
    pub(super) position: PatchStreamPosition,
    pub(super) artifact_digest: [u8; 32],
    pub(super) roots: &'a RelationalBranchRootDescriptor,
}

pub(super) fn malformed() -> RelationalRecordedComponentRefusal {
    RelationalRecordedComponentRefusal::PayloadEncoding {
        detail: "recorded Native component description is malformed or exceeds allocation limits"
            .into(),
    }
}
pub(super) fn reserve(
    execution: &ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<Option<ExecutionMemoryReservation>, RelationalRecordedComponentRefusal> {
    execution
        .reserve_retained_memory(u64::try_from(bytes).map_err(|_| malformed())?)
        .map_err(|cause| RelationalRecordedComponentRefusal::Request(cause.into()))
}
pub(super) fn fixed_bytes() -> Result<usize, RelationalRecordedComponentRefusal> {
    std::mem::size_of::<RecordedComponentMeaning>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .ok_or_else(malformed)
}
impl RecordedRelationalComponentReference {
    pub(super) fn copy_original(
        execution: &ExecutionRequest<'_, '_>,
        descriptor: &RelationalBranchBasisDescriptor,
        target: Option<CanonicalDescription<'_>>,
    ) -> Result<Self, RelationalRecordedComponentRefusal> {
        crate::execution::check_request(execution)
            .map_err(RelationalRecordedComponentRefusal::Request)?;
        // canonical_observation uses the owner's one exactly sized String;
        // writing this prefix, decimal runtime and UTF-8 branch never grows it.
        let runtime_digits = super::super::canonical_runtime_frame::CANONICAL_RUNTIME_INSTANCE_ID
            .checked_ilog10()
            .map_or(1, |digits| digits as usize + 1);
        let reference_name = "relational//"
            .len()
            .checked_add(runtime_digits)
            .and_then(|bytes| bytes.checked_add(descriptor.branch_id().0.len()))
            .ok_or_else(malformed)?;
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let mut bytes = fixed_bytes()?
            .checked_add(descriptor.branch_id().0.len())
            .and_then(|bytes| bytes.checked_add(reference_name))
            .and_then(|bytes| {
                parents
                    .checked_mul(std::mem::size_of::<u64>())
                    .and_then(|parents| bytes.checked_add(parents))
            })
            .ok_or_else(malformed)?;
        if let Some(target) = &target {
            bytes = target
                .receipt
                .parents
                .len()
                .checked_mul(std::mem::size_of::<CommitId>())
                .and_then(|parents| bytes.checked_add(parents))
                .and_then(|bytes| bytes.checked_add(target.receipt.branch_id.0.len()))
                .ok_or_else(malformed)?;
        }
        let funding = reserve(execution, bytes)?;
        let descriptor = descriptor
            .in_canonical_frame()
            .map_err(RelationalRecordedComponentRefusal::Frame)?;
        let target = match target {
            None => RecordedRelationalComponentTarget::Empty,
            Some(target) => {
                RecordedRelationalComponentTarget::Canonical(RecordedRelationalCanonicalComponent {
                    receipt: target.receipt.clone(),
                    position: target.position,
                    artifact_digest: target.artifact_digest,
                    roots: target.roots.clone(),
                })
            }
        };
        crate::execution::check_request(execution)
            .map_err(RelationalRecordedComponentRefusal::Request)?;
        Ok(Self {
            meaning: Arc::new(RecordedComponentMeaning {
                descriptor,
                target,
                _funding: funding,
            }),
        })
    }
}
