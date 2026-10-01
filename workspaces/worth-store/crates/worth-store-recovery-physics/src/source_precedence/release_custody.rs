//! A selected-checkpoint custody claim for a retired V3 release. The
//! historical closure is Store-attested after retirement; Store must rejoin
//! these source-bound claims to its own observed selected media before it may
//! mint a Serving capability.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobReclaimSourceKind, BlobRecordKind,
    BlobRecordV1, CurrentPhysicalRecordPlacement, DropSetManifestV3, DurablePhysicalRootManifest,
    OriginalDropReservedV1, PhysicalRecordFormatDeclaration, ReleaseCheckpointAccumulatorV1,
    ReleaseCheckpointBatchV1, SelectedRecordContentClass,
};
use worth_store_physical_integrity::{
    IntegrityValidatedSelectedExtentPayload, VerifiedCheckpointStream,
};

use super::PhysicalSourceSelection;
use crate::operation_reconciliation::{ReconciledOperationFates, RecoveryOperationFate};

#[path = "release_custody/head_v2.rs"]
mod head_v2;
#[path = "release_custody/retained_storage.rs"]
mod retained_storage;
#[path = "release_custody/roster.rs"]
mod roster;
#[path = "release_custody/roster_v2.rs"]
mod roster_v2;
pub use head_v2::{
    AddressedReleaseHeadControlV2, VerifiedCheckpointReleaseHeadRosterV2,
    VerifiedSelectedReleaseHeadCustodyV2,
};
#[path = "release_custody/addressed_base.rs"]
mod addressed_base;
pub use addressed_base::{AddressedCheckpointBatchControl, VerifiedAddressedCheckpointReleaseBase};

const DROP_MATERIAL_DOMAIN: &[u8] = b"worth.store.blob.reclaim.mutation.v1";
const IDEMPOTENCY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCustodyDenial {
    MissingCheckpoint,
    CertificateRoster,
    SelectedControlFrame,
    ReleaseBinding,
    DurableFate,
    ResidentBoundExceeded { required: u64, admitted: u64 },
}

/// Bytes authenticated by every C.9 extent chunk under an exact selected
/// manifest placement. A raw Vec or caller-supplied SHA cannot mint this,
/// but this type alone does not establish actual Store media provenance.
#[derive(Debug)]
pub struct WitnessedSelectedControlFrame {
    bytes: Vec<u8>,
    witness: IntegrityValidatedSelectedExtentPayload,
}

impl WitnessedSelectedControlFrame {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn from_validated(
        bytes: Vec<u8>,
        witness: IntegrityValidatedSelectedExtentPayload,
    ) -> Result<Self, SelectedCustodyDenial> {
        if !witness.matches_frame(&bytes) {
            return Err(SelectedCustodyDenial::SelectedControlFrame);
        }
        Ok(Self { bytes, witness })
    }

    pub(crate) const fn selected_placement(&self) -> CurrentPhysicalRecordPlacement {
        CurrentPhysicalRecordPlacement::Extent(self.witness.placement())
    }

    pub(super) const fn selected_payload_sha256(&self) -> [u8; 32] {
        self.witness.payload_sha256()
    }

    pub(crate) fn selected<'a>(
        &'a self,
        selected: &PhysicalSourceSelection,
        kind: BlobRecordKind,
    ) -> Result<&'a [u8], SelectedCustodyDenial> {
        let placement = self.witness.placement();
        if !selected.page_facts().placements().iter().any(|route| {
            matches!(route,
                CurrentPhysicalRecordPlacement::Extent(value)
                    if *value == placement
                        && value.content_class() == SelectedRecordContentClass::Blob(kind))
        }) {
            return Err(SelectedCustodyDenial::SelectedControlFrame);
        }
        Ok(&self.bytes)
    }
}

/// Private-field claim handed to Store after the supplied checkpoint, control
/// bytes, and operation fate join. Store must re-read its selected media and
/// recheck the claim under owner lock; this public constructor is not an
/// ingress-owned media seal. Earlier retired closure edges remain selected
/// Store custody, not a new independent rewalk of absent media.
#[derive(Debug)]
pub struct VerifiedSelectedCheckpointCustody {
    checkpoint: Arc<VerifiedCheckpointStream>,
    source_root_sha256: [u8; 32],
    selected_root: DurablePhysicalRootManifest,
    selected_root_sha256: [u8; 32],
    batches: Box<[ReleaseCheckpointBatchV1]>,
    release_certificate_record_count: u16,
    release_certificate_encoded_bytes: u32,
    accumulator: ReleaseCheckpointAccumulatorV1,
    accumulator_payload_sha256: [u8; 32],
    tip_custody_digest: [u8; 32],
}

impl VerifiedSelectedCheckpointCustody {
    /// Updates only the selected-root observation after C8's completed
    /// publication and exact fresh reopen. Certificate, tip, and fate facts
    /// stay bound to the original selected checkpoint and controls.
    pub fn rebind_published_root(
        &mut self,
        selected: &PhysicalSourceSelection,
        published: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
    ) -> Result<(), SelectedCustodyDenial> {
        if self.selected_root != *selected.root().selected().manifest()
            || format != selected.root().selected().selector().format()
            || published.generation() < self.selected_root.generation()
            || published.tree_identity() != self.selected_root.tree_identity()
            || published.generation() < self.checkpoint.source().root().generation()
        {
            return Err(SelectedCustodyDenial::ReleaseBinding);
        }
        self.selected_root_sha256 = Sha256::digest(published.encode(format)).into();
        self.selected_root = published.clone();
        Ok(())
    }

    pub fn admit_selected_release(
        selected: &PhysicalSourceSelection,
        descriptor: &WitnessedSelectedControlFrame,
        reservation: &WitnessedSelectedControlFrame,
        manifest: &WitnessedSelectedControlFrame,
        fates: &ReconciledOperationFates,
        policy_identity: [u8; 32],
    ) -> Result<Self, SelectedCustodyDenial> {
        let checkpoint = selected
            .checkpoint()
            .ok_or(SelectedCustodyDenial::MissingCheckpoint)?;
        let stream = checkpoint.checkpoint();
        let root_sha = checkpoint.source_root_frame_sha256();
        let roster = roster::parse(stream, root_sha)?;
        let accumulator = roster.accumulator;
        let tip = accumulator.tip();
        let descriptor_bytes =
            descriptor.selected(selected, BlobRecordKind::ReclaimDescriptorV3)?;
        let reservation_bytes =
            reservation.selected(selected, BlobRecordKind::OriginalDropReserved)?;
        let manifest_bytes = manifest.selected(selected, BlobRecordKind::DropSetManifestV3)?;
        let Ok(BlobRecordV1::ReclaimDescriptorV3(drop)) = decode_blob_record(descriptor_bytes)
        else {
            return Err(SelectedCustodyDenial::SelectedControlFrame);
        };
        let Ok(BlobRecordV1::OriginalDropReserved(reserved)) =
            decode_blob_record(reservation_bytes)
        else {
            return Err(SelectedCustodyDenial::SelectedControlFrame);
        };
        let Ok(BlobRecordV1::DropSetManifestV3(dropped)) = decode_blob_record(manifest_bytes)
        else {
            return Err(SelectedCustodyDenial::SelectedControlFrame);
        };
        require_binding(
            selected,
            tip,
            drop,
            reserved,
            &dropped,
            descriptor.witness,
            reservation.witness,
            manifest.witness,
        )?;
        require_fate(
            fates,
            policy_identity,
            tip.request(),
            drop.base().store(),
            drop.base().reclaim_attempt(),
            tip.fate(),
            stream.compaction_cutover().wal_cutoff_lsn_exclusive(),
        )?;
        // Root ingress admits only canonical bytes, so re-encoding the selected
        // projection reproduces the exact C.9-inspected current root frame.
        let selected_root = selected.root().selected().manifest().clone();
        let selected_root_sha256 =
            Sha256::digest(selected_root.encode(selected.root().selected().selector().format()))
                .into();
        Ok(Self {
            checkpoint: checkpoint.share_checkpoint(),
            source_root_sha256: root_sha,
            selected_root,
            selected_root_sha256,
            batches: roster.batches,
            release_certificate_record_count: roster.record_count,
            release_certificate_encoded_bytes: roster.encoded_bytes,
            accumulator,
            accumulator_payload_sha256: Sha256::digest(accumulator.encode()).into(),
            tip_custody_digest: drop.custody_digest(),
        })
    }

    pub fn checkpoint(&self) -> &VerifiedCheckpointStream {
        &self.checkpoint
    }

    pub const fn source_root_sha256(&self) -> [u8; 32] {
        self.source_root_sha256
    }

    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }

    pub const fn selected_root_sha256(&self) -> [u8; 32] {
        self.selected_root_sha256
    }

    pub const fn accumulator(&self) -> ReleaseCheckpointAccumulatorV1 {
        self.accumulator
    }

    pub fn batches(&self) -> &[ReleaseCheckpointBatchV1] {
        &self.batches
    }

    pub const fn release_certificate_record_count(&self) -> u16 {
        self.release_certificate_record_count
    }

    pub const fn release_certificate_encoded_bytes(&self) -> u32 {
        self.release_certificate_encoded_bytes
    }

    pub const fn accumulator_payload_sha256(&self) -> [u8; 32] {
        self.accumulator_payload_sha256
    }

    pub const fn tip_custody_digest(&self) -> [u8; 32] {
        self.tip_custody_digest
    }
}

fn require_binding(
    selected: &PhysicalSourceSelection,
    tip: worth_store_physical_format::ReleasedDropTipProvenanceV1,
    drop: worth_store_physical_format::BlobReclaimDescriptorV3,
    reserved: OriginalDropReservedV1,
    manifest: &DropSetManifestV3,
    descriptor: IntegrityValidatedSelectedExtentPayload,
    reservation: IntegrityValidatedSelectedExtentPayload,
    manifest_frame: IntegrityValidatedSelectedExtentPayload,
) -> Result<(), SelectedCustodyDenial> {
    let base = drop.base();
    let store = selected
        .root()
        .selected()
        .selector()
        .store_identity()
        .bytes();
    let expected = SelectedCustodyDenial::ReleaseBinding;
    if descriptor.placement().record() != tip.descriptor_record()
        || descriptor.payload_sha256() != tip.descriptor_frame_sha256()
        || reservation.placement().record() != tip.reservation_record()
        || reservation.payload_sha256() != tip.reservation_frame_sha256()
        || manifest_frame.placement().record() != base.manifest_record()
        || manifest_frame.payload_sha256() != base.manifest_frame_sha256()
        || base.store() != store
        || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
        || base.candidate_root_generation() != tip.candidate_root_generation()
        || drop.custody().request() != tip.request()
        || reserved.store() != store
        || reserved.reclaim_attempt() != base.reclaim_attempt()
        || reserved.manifest_record() != base.manifest_record()
        || reserved.manifest_frame_sha256() != base.manifest_frame_sha256()
        || reserved.source_basis_digest() != base.source_basis_digest()
        || reserved.reserved_selected_generation() != base.source_root_generation()
        || reserved.request() != tip.request()
        || manifest.store() != store
        || manifest.reclaim_attempt() != base.reclaim_attempt()
        || manifest.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
        || manifest.source_basis_digest() != base.source_basis_digest()
        || manifest.count() != base.manifest_count()
        || manifest.never_reserved_slot_generation() != reserved.manifest_selected_generation()
    {
        return Err(expected);
    }
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(expected);
    };
    if base.predecessor().is_none()
        && (base.cumulative_dropped() != u64::from(manifest.count())
            || manifest
                .dropped()
                .binary_search(&source.publication_record())
                .is_err())
    {
        return Err(expected);
    }
    Ok(())
}

fn require_fate(
    fates: &ReconciledOperationFates,
    policy: [u8; 32],
    request: worth_store_physical_format::OriginalDropReservationRequestV1,
    store: [u8; 16],
    attempt: [u8; 16],
    witness: worth_store_physical_format::ReleasedDropWalFateWitnessV1,
    cutoff: u64,
) -> Result<(), SelectedCustodyDenial> {
    let denial = SelectedCustodyDenial::DurableFate;
    if witness.lsn_end_exclusive() > cutoff {
        return Err(denial);
    }
    let mut matching = fates
        .operations()
        .iter()
        .filter(|operation| operation.identity().idempotency() == request.idempotency());
    let operation = matching.next().ok_or(denial)?;
    if matching.next().is_some()
        || operation.identity().store() != store
        || !matches!(
            operation.fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
        )
        || operation.request_fingerprint() != request.fingerprint()
        || operation.lease_issuance_generation() != request.lease_issuance_generation()
        || operation.lease_expiry_generation() != request.lease_expiry_generation()
        || expected_drop_key(store, attempt, policy, request) != request.idempotency()
    {
        return Err(denial);
    }
    Ok(())
}

pub(super) fn expected_drop_key(
    store: [u8; 16],
    attempt: [u8; 16],
    policy: [u8; 32],
    request: worth_store_physical_format::OriginalDropReservationRequestV1,
) -> [u8; 32] {
    let mut material = Sha256::new();
    material.update(DROP_MATERIAL_DOMAIN);
    material.update(store);
    material.update(attempt);
    material.update([2]);
    let mut key = Sha256::new();
    key.update((IDEMPOTENCY_DOMAIN.len() as u64).to_le_bytes());
    key.update(IDEMPOTENCY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(request.lease_issuance_generation().to_le_bytes());
    key.update(request.lease_expiry_generation().to_le_bytes());
    key.update(material.finalize());
    key.finalize().into()
}
