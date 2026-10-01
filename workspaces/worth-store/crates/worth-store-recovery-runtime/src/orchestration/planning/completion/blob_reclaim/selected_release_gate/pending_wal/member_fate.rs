//! One V3 descriptor's unique selected WAL member and retained frame fate.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobRecordV1, ReleasedDropWalFateWitnessV1,
};
use worth_store_recovery_physics::{PhysicalRedoProjection, RecoveryOperationFate};

use super::{PlanningContext, ResolvedPlanningBasis};

pub(super) struct PendingMemberFate {
    pub(super) descriptor: BlobReclaimDescriptorV3,
    pub(super) wal_fate: ReleasedDropWalFateWitnessV1,
    pub(super) canonical_redo_digest: [u8; 32],
}

pub(super) fn witness(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
    projection: &PhysicalRedoProjection,
) -> Option<PendingMemberFate> {
    let descriptor_bytes = basis
        .redo
        .blob_semantic_record_bytes(projection.operation())?;
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(descriptor_bytes).ok()?
    else {
        return None;
    };
    let request = descriptor.custody().request();
    let mut members = basis
        .sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == request.idempotency());
    let member = members.next()?;
    if members.next().is_some()
        || member.operation_identity() != projection.operation()
        || !matches!(
            projection.fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
                | RecoveryOperationFate::Indeterminate
        )
    {
        return None;
    }
    let retained = context
        .integrity
        .admitted_wal()
        .recoverable_frame_iter(context.selection.wal_tail());
    let mut frames = retained.filter(|frame| {
        frame.lsn_start() == member.lsn_range().start().get()
            && frame.lsn_end() == member.lsn_range().end_exclusive().get()
    });
    let frame = frames.next()?;
    let wal_fate = ReleasedDropWalFateWitnessV1::new(
        frame.lsn_start(),
        frame.lsn_end(),
        frame.identity_digest(),
        frame.payload_digest(),
    )
    .ok()?;
    if frames.next().is_some() {
        return None;
    }
    Some(PendingMemberFate {
        descriptor,
        wal_fate,
        canonical_redo_digest: Sha256::digest(member.canonical_redo()).into(),
    })
}
