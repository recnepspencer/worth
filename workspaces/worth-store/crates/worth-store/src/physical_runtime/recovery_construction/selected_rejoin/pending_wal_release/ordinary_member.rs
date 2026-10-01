//! One ordinary root edge is tied to a unique, independently sampled C.9
//! member and selected WAL frame before its checked transition is rewalked.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_canonical_redo_v3, PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_recovery_physics::{
    ObservedOrdinaryRootMember, PhysicalRedoGroupBinding, RecoveryOperationFate,
    VerifiedOrdinaryRootStep,
};

use super::super::SelectedMediaRejoinDenial as Denial;
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
    StoreRecoveryOperationFate,
};

pub(super) struct MatchedOrdinaryMember {
    projection: PersistedPhysicalRecoveryProjection,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    redo_sha256: [u8; 32],
    lsn_range: worth_store_wal::WalLsnRange,
    retained_scratch_bytes: u64,
}

impl MatchedOrdinaryMember {
    pub(super) const fn retained_scratch_bytes(&self) -> u64 {
        self.retained_scratch_bytes
    }

    pub(super) fn observed(&self) -> ObservedOrdinaryRootMember<'_> {
        ObservedOrdinaryRootMember {
            operation: self.operation,
            group: self.group,
            fate: self.fate,
            canonical_redo_sha256: self.redo_sha256,
            lsn_range: self.lsn_range,
            projection: &self.projection,
        }
    }
}

pub(super) fn match_step(
    step: &VerifiedOrdinaryRootStep,
    sample: &StoreRecoveryBindingFreshnessSample,
    selected_wal: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    maximum_decode_scratch_bytes: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<MatchedOrdinaryMember, Denial> {
    let range = step.lsn_range().ok_or(Denial::WalFate)?;
    if range.start().get() < cutoff {
        return Err(Denial::WalFate);
    }
    let mut members = sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == step.operation());
    let member = members.next().ok_or(Denial::WalFate)?;
    let group = PhysicalRedoGroupBinding::new(
        member.group_identity(),
        member.group_member_identity(),
        member.group_member_ordinal(),
        member.group_member_count(),
        member.group_membership_digest(),
    )
    .ok_or(Denial::WalFate)?;
    let digest: [u8; 32] = Sha256::digest(member.canonical_redo()).into();
    if members.next().is_some()
        || member.lsn_range() != range
        || group != step.group()
        || digest != step.redo_sha256()
    {
        return Err(Denial::WalFate);
    }
    let mut operations = sample
        .operations()
        .iter()
        .filter(|operation| operation.idempotency_identity() == step.operation());
    let operation = operations.next().ok_or(Denial::WalFate)?;
    let fate = match operation.fate() {
        StoreRecoveryOperationFate::AcknowledgedDurable => {
            RecoveryOperationFate::AcknowledgedDurable
        }
        StoreRecoveryOperationFate::DurableUnacknowledged => {
            RecoveryOperationFate::DurableUnacknowledged
        }
        StoreRecoveryOperationFate::Indeterminate => RecoveryOperationFate::Indeterminate,
        StoreRecoveryOperationFate::ProvenNoEffect => return Err(Denial::WalFate),
    };
    if operations.next().is_some() || fate != step.fate() {
        return Err(Denial::WalFate);
    }
    // `sample` was built from this exact independently inventoried frame set;
    // its canonical member bytes came from the sole frame in this interval.
    // The caller compares the complete inventory to a final reread before
    // seal, so this join is to the same media rather than an LSN-only claim.
    let mut frames = selected_wal.iter().filter(|frame| {
        frame.lsn_start() < range.end_exclusive().get() && range.start().get() < frame.lsn_end()
    });
    let frame = frames.next().ok_or(Denial::WalFate)?;
    if frames.next().is_some()
        || frame.lsn_start() != range.start().get()
        || frame.lsn_end() != range.end_exclusive().get()
    {
        return Err(Denial::WalFate);
    }
    let bound = member.canonical_redo().len() as u64;
    // The sampled canonical bytes remain live while decode allocates frames,
    // placements, manifests and inline state. Reserve a conservative peak
    // before asking the decoder to allocate any of those owned vectors.
    let retained_scratch_bytes = bound.checked_mul(4).ok_or(Denial::BoundExceeded)?;
    if retained_scratch_bytes > maximum_decode_scratch_bytes {
        return Err(Denial::BoundExceeded);
    }
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: bound,
        record_identities: bound,
        placements: bound,
        segment_updates: bound,
        manifests: bound,
        total_entries: bound.saturating_mul(3),
        inline_allocations: bound,
    };
    let (_, projection) = decode_canonical_redo_v3(
        member.canonical_redo(),
        range.start().get(),
        range.end_exclusive().get(),
        bound,
        None,
        limits,
        format,
    )
    .map_err(|_| Denial::WalFate)?;
    Ok(MatchedOrdinaryMember {
        projection,
        operation: step.operation(),
        group,
        fate,
        redo_sha256: digest,
        lsn_range: range,
        retained_scratch_bytes,
    })
}
