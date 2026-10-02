//! One completed post-checkpoint V3 edge. Its controls are bound to that
//! edge's addressed candidate root, not required to survive the final root.

use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobRecordKind,
    BlobRecordV1, DropSetManifestV3, OriginalDropReservedV1, ReleaseCustodyHeadKeyV1,
    ReleasedDropWalFateWitnessV1,
};

use super::{verification::verify_fate, PendingWalReleaseCustodyDenial as Denial};
use crate::{
    ReconciledOperationFates, RecoveryOperationFate, VerifiedAddressedCheckpointReleaseBase,
    VerifiedAddressedReleasedControlFrame, VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
    VerifiedSelectedReleaseHeadCustodyV2,
};

#[derive(Debug)]
pub struct VerifiedOrderedPendingWalReleaseBatch {
    edge_index: usize,
    descriptor_frame: VerifiedAddressedReleasedControlFrame,
    reservation_frame: VerifiedAddressedReleasedControlFrame,
    manifest_frame: VerifiedAddressedReleasedControlFrame,
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: DropSetManifestV3,
    wal_fate: ReleasedDropWalFateWitnessV1,
    raw_fate: RecoveryOperationFate,
    operation_fate: RecoveryOperationFate,
    retained_bytes: u64,
}

impl VerifiedOrderedPendingWalReleaseBatch {
    /// Outer batch storage belongs to the containing roster.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.descriptor_frame
            .owned_heap_bytes()?
            .checked_add(self.reservation_frame.owned_heap_bytes()?)?
            .checked_add(self.manifest_frame.owned_heap_bytes()?)?
            .checked_add(
                u64::try_from(self.manifest.dropped().len())
                    .ok()?
                    .checked_mul(
                        u64::try_from(std::mem::size_of::<
                            worth_store_physical_format::PersistedRecordIdentity,
                        >())
                        .ok()?,
                    )?,
            )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        history: &VerifiedOrderedRootHistory,
        edge_index: usize,
        descriptor_frame: VerifiedAddressedReleasedControlFrame,
        reservation_frame: VerifiedAddressedReleasedControlFrame,
        manifest_frame: VerifiedAddressedReleasedControlFrame,
        wal_fate: ReleasedDropWalFateWitnessV1,
        fates: &ReconciledOperationFates,
        policy: [u8; 32],
        checkpoint_cutoff: u64,
        selected_base: Option<&VerifiedAddressedCheckpointReleaseBase>,
        selected_head_v2: Option<&VerifiedSelectedReleaseHeadCustodyV2>,
        prior: &[Self],
        remaining_retained_bytes: u64,
    ) -> Result<Self, Denial> {
        let Some(VerifiedOrderedRootEdge::Released(edge)) = history.edges().get(edge_index) else {
            return Err(Denial::SourceBinding);
        };
        let candidate = edge.result_root_frame_sha256();
        if descriptor_frame.kind() != BlobRecordKind::ReclaimDescriptorV3
            || reservation_frame.kind() != BlobRecordKind::OriginalDropReserved
            || manifest_frame.kind() != BlobRecordKind::DropSetManifestV3
            || descriptor_frame.candidate_root_frame_sha256() != candidate
            || reservation_frame.candidate_root_frame_sha256() != candidate
            || manifest_frame.candidate_root_frame_sha256() != candidate
            || descriptor_frame.record() != edge.descriptor_record()
            || descriptor_frame.payload_sha256() != edge.descriptor_frame_sha256()
            || wal_fate.lsn_start() != edge.lsn().start().get()
            || wal_fate.lsn_end_exclusive() != edge.lsn().end_exclusive().get()
        {
            return Err(Denial::ControlBinding);
        }
        let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
            decode_blob_record(descriptor_frame.bytes()).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(reservation_frame.bytes()).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let BlobRecordV1::DropSetManifestV3(manifest) =
            decode_blob_record(manifest_frame.bytes()).map_err(|_| Denial::ControlBinding)?
        else {
            return Err(Denial::ControlBinding);
        };
        let base = descriptor.base();
        let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
            return Err(Denial::ControlBinding);
        };
        if descriptor.encode() != descriptor_frame.bytes()
            || base.manifest_record() != manifest_frame.record()
            || base.manifest_frame_sha256() != manifest_frame.payload_sha256()
            || base.store() != manifest.store()
            || base.store() != reservation.store()
            || base.reclaim_attempt() != manifest.reclaim_attempt()
            || base.reclaim_attempt() != reservation.reclaim_attempt()
            || base.source_basis_digest() != manifest.source_basis_digest()
            || base.source_basis_digest() != reservation.source_basis_digest()
            || base.manifest_count() != manifest.count()
            || base.manifest_record() != reservation.manifest_record()
            || base.manifest_frame_sha256() != reservation.manifest_frame_sha256()
            || base.source_root_generation().checked_add(1)
                != Some(base.candidate_root_generation())
            || base.candidate_root_generation() != edge.candidate_root_generation()
            || descriptor.custody().source_root_frame_sha256() != edge.source_root_frame_sha256()
            || descriptor.custody().source_free_space_frame_sha256()
                != edge.source_free_space_frame_sha256()
            || descriptor.custody().request().idempotency() != edge.operation()
            || reservation.request() != descriptor.custody().request()
            || reservation.reserved_selected_generation() != base.source_root_generation()
        {
            return Err(Denial::ControlBinding);
        }
        let previous = prior.iter().rev().find(|batch| {
            matches!(batch.manifest.source_basis(),
                BlobReclaimSourceBasisV1::ReleasedGeneration(value)
                    if value.object() == source.object()
                        && value.generation() == source.generation())
        });
        match (base.predecessor(), previous) {
            (None, None)
                if manifest
                    .dropped()
                    .binary_search(&source.publication_record())
                    .is_ok()
                    && base.cumulative_dropped() == u64::from(manifest.count()) => {}
            (Some(predecessor), Some(previous))
                if !previous.descriptor.base().terminal()
                    && predecessor.descriptor_record() == previous.descriptor_frame.record()
                    && predecessor.descriptor_frame_sha256()
                        == previous.descriptor_frame.payload_sha256()
                    && previous.descriptor.base().source_basis_digest()
                        == base.source_basis_digest()
                    && previous
                        .descriptor
                        .base()
                        .cumulative_dropped()
                        .checked_add(u64::from(manifest.count()))
                        == Some(base.cumulative_dropped()) => {}
            (Some(_), None)
                if selected_base.is_some_and(|selected| {
                    selected.matches_predecessor(descriptor, &manifest)
                }) => {}
            (Some(predecessor), None)
                if selected_head_v2.is_some_and(|selected| {
                    let Some(key) =
                        ReleaseCustodyHeadKeyV1::new(source.object(), source.generation())
                    else {
                        return false;
                    };
                    let Ok(index) = selected
                        .selected_heads()
                        .binary_search_by_key(&key, |entry| entry.key())
                    else {
                        return false;
                    };
                    let prior = selected.selected_heads()[index];
                    !prior.terminal()
                        && prior.descriptor_record() == predecessor.descriptor_record()
                        && prior.descriptor_frame_sha256() == predecessor.descriptor_frame_sha256()
                        && prior.source_basis_digest() == base.source_basis_digest()
                        && prior
                            .cumulative_dropped()
                            .checked_add(u64::from(manifest.count()))
                            == Some(base.cumulative_dropped())
                }) => {}
            _ => {
                return Err(Denial::ControlBinding);
            }
        }
        let operation_fate = verify_fate(
            descriptor,
            wal_fate,
            fates,
            policy,
            edge.operation(),
            checkpoint_cutoff,
        )?;
        if edge.fate() == RecoveryOperationFate::ProvenNoEffect
            || !matches!(
                operation_fate,
                RecoveryOperationFate::AcknowledgedDurable
                    | RecoveryOperationFate::DurableUnacknowledged
                    | RecoveryOperationFate::Indeterminate
            )
        {
            return Err(Denial::DurableWalFate);
        }
        let retained_bytes = descriptor_frame
            .retained_bytes()
            .checked_add(reservation_frame.retained_bytes())
            .and_then(|value| value.checked_add(manifest_frame.retained_bytes()))
            .and_then(|value| value.checked_add(std::mem::size_of::<Self>() as u64))
            .and_then(|value| {
                value.checked_add(
                    (manifest.dropped().len() as u64)
                        .checked_mul(std::mem::size_of::<
                            worth_store_physical_format::PersistedRecordIdentity,
                        >() as u64)?,
                )
            })
            .ok_or(Denial::RetainedSizeOverflow)?;
        if retained_bytes > remaining_retained_bytes {
            return Err(Denial::RetainedBoundExceeded {
                required: retained_bytes,
                admitted: remaining_retained_bytes,
            });
        }
        Ok(Self {
            edge_index,
            descriptor_frame,
            reservation_frame,
            manifest_frame,
            descriptor,
            reservation,
            manifest,
            wal_fate,
            raw_fate: edge.fate(),
            operation_fate,
            retained_bytes,
        })
    }

    pub const fn edge_index(&self) -> usize {
        self.edge_index
    }
    pub const fn descriptor_frame(&self) -> &VerifiedAddressedReleasedControlFrame {
        &self.descriptor_frame
    }
    pub const fn reservation_frame(&self) -> &VerifiedAddressedReleasedControlFrame {
        &self.reservation_frame
    }
    pub const fn manifest_frame(&self) -> &VerifiedAddressedReleasedControlFrame {
        &self.manifest_frame
    }
    pub const fn descriptor(&self) -> BlobReclaimDescriptorV3 {
        self.descriptor
    }
    pub const fn reservation(&self) -> OriginalDropReservedV1 {
        self.reservation
    }
    pub const fn manifest(&self) -> &DropSetManifestV3 {
        &self.manifest
    }
    pub const fn wal_fate(&self) -> ReleasedDropWalFateWitnessV1 {
        self.wal_fate
    }
    pub const fn operation_fate(&self) -> RecoveryOperationFate {
        self.operation_fate
    }
    pub const fn raw_fate(&self) -> RecoveryOperationFate {
        self.raw_fate
    }
    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}
