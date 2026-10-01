//! Complete checkpoint-source Batch/Accumulator admission from witnessed media.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobReclaimSourceKind, BlobRecordKind,
    BlobRecordV1, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    ReleaseCheckpointBatchV1, ReleasedDropCumulativeEvidenceV1, ReleasedDropWalFateWitnessV1,
};

use super::{
    checks::{require_addressed_fate, require_frame, verify_lineage},
    prior::verify_prior_control,
    AddressedCheckpointBatchControl, AddressedReleaseLineage,
    VerifiedAddressedCheckpointReleaseBase,
};
use crate::source_precedence::release_custody::{
    roster, SelectedCustodyDenial, WitnessedSelectedControlFrame,
};
use crate::{
    PhysicalSourceSelection, ReconciledOperationFates, ReleasedInventoryView,
    VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
};

impl VerifiedAddressedCheckpointReleaseBase {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        selected: &PhysicalSourceSelection,
        history: &VerifiedOrderedRootHistory,
        source: ReleasedInventoryView<'_>,
        descriptor_frame: WitnessedSelectedControlFrame,
        reservation_frame: WitnessedSelectedControlFrame,
        manifest_frame: WitnessedSelectedControlFrame,
        prior_controls: Vec<AddressedCheckpointBatchControl>,
        fates: &ReconciledOperationFates,
        selected_wal_frames: &[ReleasedDropWalFateWitnessV1],
        selected_wal_members: &[([u8; 32], u64, u64)],
        policy: [u8; 32],
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        maximum_retained_bytes: u64,
    ) -> Result<Self, SelectedCustodyDenial> {
        let denial = SelectedCustodyDenial::ReleaseBinding;
        let checkpoint = selected
            .checkpoint()
            .ok_or(SelectedCustodyDenial::MissingCheckpoint)?;
        let stream = checkpoint.checkpoint();
        let checkpoint_root = source.root;
        let checkpoint_free = source.free;
        let root_basis = stream.source().root();
        let root_sha: [u8; 32] = Sha256::digest(checkpoint_root.encode(format)).into();
        let free_sha: [u8; 32] = Sha256::digest(checkpoint_free.encode(format)).into();
        let first_topology = match history.edges().first() {
            Some(VerifiedOrderedRootEdge::Ordinary(step)) => step.source_topology(),
            Some(VerifiedOrderedRootEdge::Released(step)) => step.transition().source_topology(),
            None => return Err(denial),
        };
        if checkpoint_root.generation() != root_basis.generation()
            || checkpoint_root.tree_identity() != root_basis.tree_identity()
            || root_sha != checkpoint.source_root_frame_sha256()
            || root_sha != history.checkpoint_root_frame_sha256()
            || !first_topology.matches_headers(checkpoint_root, checkpoint_free, format)
            || crate::source_precedence::released_v3_inventory_transition::transcript(
                source,
                format,
                maximum_entries,
            )
            .map_err(|_| denial)?
                != first_topology
        {
            return Err(denial);
        }
        let parsed = roster::parse(stream, root_sha)?;
        if prior_controls.len() != parsed.batches.len().saturating_sub(1) {
            return Err(denial);
        }
        let tip = parsed.accumulator.tip();
        require_frame(
            source.routes,
            &descriptor_frame,
            BlobRecordKind::ReclaimDescriptorV3,
        )?;
        require_frame(
            source.routes,
            &reservation_frame,
            BlobRecordKind::OriginalDropReserved,
        )?;
        require_frame(
            source.routes,
            &manifest_frame,
            BlobRecordKind::DropSetManifestV3,
        )?;
        let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
            decode_blob_record(descriptor_frame.bytes()).map_err(|_| denial)?
        else {
            return Err(denial);
        };
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(reservation_frame.bytes()).map_err(|_| denial)?
        else {
            return Err(denial);
        };
        let BlobRecordV1::DropSetManifestV3(manifest) =
            decode_blob_record(manifest_frame.bytes()).map_err(|_| denial)?
        else {
            return Err(denial);
        };
        let base = descriptor.base();
        let BlobReclaimSourceBasisV1::ReleasedGeneration(source_basis) = manifest.source_basis()
        else {
            return Err(denial);
        };
        if descriptor_frame.selected_placement().record() != tip.descriptor_record()
            || descriptor_frame.selected_payload_sha256() != tip.descriptor_frame_sha256()
            || reservation_frame.selected_placement().record() != tip.reservation_record()
            || reservation_frame.selected_payload_sha256() != tip.reservation_frame_sha256()
            || manifest_frame.selected_placement().record() != base.manifest_record()
            || manifest_frame.selected_payload_sha256() != base.manifest_frame_sha256()
            || base.store()
                != selected
                    .root()
                    .selected()
                    .selector()
                    .store_identity()
                    .bytes()
            || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || base.candidate_root_generation() != tip.candidate_root_generation()
            || descriptor.custody().request() != tip.request()
            || descriptor.custody_digest()
                != parsed
                    .batches
                    .last()
                    .map_or(descriptor.custody_digest(), |batch| batch.custody_digest())
            || base.terminal() != parsed.accumulator.terminal()
            || parsed.batches.last().is_some_and(|batch| {
                batch.cumulative_dropped() != parsed.accumulator.cumulative_dropped()
                    || batch.cumulative_digest() != parsed.accumulator.cumulative_digest()
                    || batch.predecessor() != base.predecessor()
            })
            || reservation.request() != tip.request()
            || reservation.store() != base.store()
            || reservation.reclaim_attempt() != base.reclaim_attempt()
            || reservation.manifest_record() != base.manifest_record()
            || reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
            || reservation.source_basis_digest() != base.source_basis_digest()
            || reservation.reserved_selected_generation() != base.source_root_generation()
            || manifest.store() != base.store()
            || manifest.reclaim_attempt() != base.reclaim_attempt()
            || manifest.source_basis_digest() != base.source_basis_digest()
            || manifest.count() != base.manifest_count()
            || manifest.never_reserved_slot_generation()
                != reservation.manifest_selected_generation()
            || (base.predecessor().is_none()
                && (base.cumulative_dropped() != u64::from(manifest.count())
                    || manifest
                        .dropped()
                        .binary_search(&source_basis.publication_record())
                        .is_err()))
        {
            return Err(denial);
        }
        require_addressed_fate(
            fates,
            selected_wal_frames,
            selected_wal_members,
            policy,
            tip.request(),
            base.store(),
            base.reclaim_attempt(),
            tip.fate(),
            stream.compaction_cutover().wal_cutoff_lsn_exclusive(),
        )?;
        let mut prior_count = parsed.accumulator.prior_cumulative_dropped();
        let mut prior_digest = parsed.accumulator.prior_cumulative_digest();
        let lineage_count = parsed.batches.len().max(1);
        let lineage_bytes = (lineage_count as u64)
            .checked_mul(std::mem::size_of::<AddressedReleaseLineage>() as u64)
            .ok_or(denial)?;
        if lineage_bytes > maximum_retained_bytes {
            return Err(denial);
        }
        let mut lineage = Vec::new();
        lineage
            .try_reserve_exact(lineage_count)
            .map_err(|_| denial)?;
        for (index, batch) in parsed.batches.iter().copied().enumerate() {
            let (batch_descriptor, batch_count, batch_source) = if index + 1 == parsed.batches.len()
            {
                (descriptor, manifest.count(), source_basis)
            } else {
                let controls = &prior_controls[index];
                let (descriptor, count, source) = verify_prior_control(
                    source.routes,
                    controls,
                    batch,
                    selected,
                    fates,
                    selected_wal_frames,
                    selected_wal_members,
                    policy,
                    stream.compaction_cutover().wal_cutoff_lsn_exclusive(),
                )?;
                (descriptor, count, source)
            };
            if batch_descriptor.base().predecessor() != batch.predecessor()
                || batch_descriptor.base().terminal() != batch.terminal()
                || batch_descriptor.custody_digest() != batch.custody_digest()
            {
                return Err(denial);
            }
            verify_lineage(
                &lineage,
                batch_descriptor,
                batch_source,
                batch_count,
                batch.predecessor(),
            )?;
            let step = ReleasedDropCumulativeEvidenceV1::new(
                batch.descriptor_record(),
                batch.descriptor_frame_sha256(),
                batch.custody_digest(),
                batch.reservation_record(),
                batch.reservation_frame_sha256(),
                batch.fate(),
                batch.candidate_root_generation(),
                batch.candidate_root_sha256(),
                batch.predecessor(),
                batch_count,
                batch.terminal(),
            )
            .map_err(|_| denial)?;
            (prior_count, prior_digest) = step
                .advance(prior_count, prior_digest)
                .map_err(|_| denial)?;
            if prior_count != batch.cumulative_dropped()
                || prior_digest != batch.cumulative_digest()
            {
                return Err(denial);
            }
            lineage.push(AddressedReleaseLineage {
                record: batch.descriptor_record(),
                sha256: batch.descriptor_frame_sha256(),
                descriptor: batch_descriptor,
                source: batch_source,
            });
        }
        if prior_count != parsed.accumulator.cumulative_dropped()
            || prior_digest != parsed.accumulator.cumulative_digest()
        {
            return Err(denial);
        }
        if parsed.batches.is_empty() {
            lineage.push(AddressedReleaseLineage {
                record: tip.descriptor_record(),
                sha256: tip.descriptor_frame_sha256(),
                descriptor,
                source: source_basis,
            });
        }
        let retained_bytes = [
            descriptor_frame.bytes().len(),
            reservation_frame.bytes().len(),
            manifest_frame.bytes().len(),
        ]
        .into_iter()
        .try_fold(std::mem::size_of::<Self>() as u64, |bytes, len| {
            bytes.checked_add(len as u64)
        })
        .and_then(|bytes| bytes.checked_add(lineage_bytes))
        .and_then(|bytes| {
            prior_controls.iter().try_fold(bytes, |bytes, controls| {
                bytes
                    .checked_add(std::mem::size_of::<AddressedCheckpointBatchControl>() as u64)?
                    .checked_add(controls.descriptor.bytes().len() as u64)?
                    .checked_add(controls.reservation.bytes().len() as u64)?
                    .checked_add(controls.manifest.bytes().len() as u64)
            })
        })
        .and_then(|bytes| {
            bytes.checked_add(
                (parsed.batches.len() as u64)
                    .checked_mul(std::mem::size_of::<ReleaseCheckpointBatchV1>() as u64)?,
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                (manifest.dropped().len() as u64)
                    .checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)?,
            )
        })
        .ok_or(denial)?;
        if retained_bytes > maximum_retained_bytes {
            return Err(denial);
        }
        Ok(Self {
            checkpoint: checkpoint.share_checkpoint(),
            checkpoint_root: checkpoint_root.clone(),
            checkpoint_root_frame_sha256: root_sha,
            checkpoint_free_space_frame_sha256: free_sha,
            batches: parsed.batches,
            prior_controls: prior_controls.into_boxed_slice(),
            lineage: lineage.into_boxed_slice(),
            accumulator: parsed.accumulator,
            tip_descriptor_frame: descriptor_frame,
            tip_reservation_frame: reservation_frame,
            tip_manifest_frame: manifest_frame,
            tip_descriptor: descriptor,
            tip_reservation: reservation,
            tip_manifest: manifest,
            retained_bytes,
        })
    }
}
