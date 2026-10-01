//! Rejoins a C.8 custody claim against one Store-admitted physical media
//! session before a recovered serving capability can be issued.

pub(super) mod control_frames;
pub(super) mod head_v2;
pub(super) mod no_release;
pub(super) mod pending_wal_release;
pub(super) mod release_heads;
pub(super) mod resident;
pub(in crate::physical_runtime) use resident::PhysicalRecoveryRejoinResidentDenial;
pub(super) mod root_checkpoint;
pub(super) mod tier;
pub(super) mod wal_fate;
mod wal_inventory;
pub(in crate::physical_runtime) use control_frames::SelectedControlMediaFingerprint;
pub(in crate::physical_runtime) use wal_inventory::SelectedWalMediaFingerprint;

use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, BoundedRecoveryFilesystemDiscovery,
    RecoveryFilesystemQualificationError,
};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordV1, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    VerifiedSelectedCheckpointCustody, VerifiedSelectedTierEpochCustody,
};

use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, PhysicalRecoveryCoordination,
    PhysicalRecoveryFreshnessPort,
};

const MAX_CHECKPOINT_BYTES: u64 = 256 << 20;
const MAX_CONTROL_FRAME_BYTES: u64 = 64 << 20;
const MAX_DISCOVERY_BYTES: u64 = 1 << 30;
const MAX_DISCOVERY_ENTRIES: u64 = 65_536;
const MAX_CLEANUP_SAMPLE_BYTES: u64 = 128 << 20;

/// The transcript is compared to qualified media, including exact selected
/// control payloads, complete bounded WAL admission, and final selected-slot
/// re-reads. The retained OS lease excludes a second compliant Store writer;
/// rereads detect in-flight out-of-band drift but cannot prevent a hostile
/// filesystem write after seal issuance. Serving revalidates this bounded
/// selected-media fingerprint when consuming the one-shot custody seal.
pub(super) fn observe_claim(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedSelectedCheckpointCustody,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedWalMediaFingerprint,
        SelectedControlMediaFingerprint,
    ),
    SelectedMediaRejoinDenial,
> {
    if claim.checkpoint().encoded_bytes() > MAX_CHECKPOINT_BYTES {
        return Err(SelectedMediaRejoinDenial::BoundExceeded);
    }
    let store = media.store_identity();
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(SelectedMediaRejoinDenial::Qualification)?;
    let selected = root_checkpoint::observe(&mut discovery, store, reopen.format(), claim)?;
    let occurrence = reopen.fresh_reopen_occurrence();
    if selected.root() != reopen.root()
        || selected.selector_bytes() != occurrence.selector().bytes()
        || selected.root_bytes() != occurrence.root().bytes()
    {
        return Err(SelectedMediaRejoinDenial::RootBinding);
    }
    let selected_tier = if let Some(tier) = tier_claim {
        if tier.selected_root() != claim.selected_root()
            || tier.selected_root_sha256() != claim.selected_root_sha256()
            || tier.checkpoint().source().identity() != claim.checkpoint().source().identity()
            || tier.checkpoint().encoded_bytes() != claim.checkpoint().encoded_bytes()
            || tier.checkpoint().encoded_digest() != claim.checkpoint().encoded_digest()
            || tier.checkpoint_source_root_sha256() != claim.source_root_sha256()
        {
            return Err(SelectedMediaRejoinDenial::CertificateRoster);
        }
        Some(tier::selection::observe(
            &mut discovery,
            store,
            reopen,
            tier,
        )?)
    } else {
        if selected.root().tier_epoch_anchor().is_some() {
            return Err(SelectedMediaRejoinDenial::RootBinding);
        }
        None
    };
    let selected_routes = if let Some(tier) = selected_tier.as_ref() {
        tier::routes::verify(
            &mut discovery,
            tier.root(),
            tier.free_header(),
            reopen.format(),
            None,
        )?
    } else {
        observe_unanchored_routes(&mut discovery, &selected, reopen.format())?
    };
    let partition = tier::released_partition::partition(
        &mut discovery,
        selected_routes.selected_routes(),
        reopen.format(),
    )?;
    let mut failed_ingest_slices = partition.reservation_slices().to_vec();
    tier::verify_failed_ingest_subset(
        &mut discovery,
        partition.failed_ingest(),
        reopen.format(),
        selected.root().generation(),
        claim.checkpoint().source().identity().sequence().get(),
        &mut failed_ingest_slices,
    )?;
    let controls = control_frames::observe(
        &mut discovery,
        &selected,
        claim,
        partition.released(),
        tier_claim.map(|tier| tier.tier_epoch_start()),
        MAX_CONTROL_FRAME_BYTES,
    )?;
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(controls.descriptor().bytes())
            .map_err(|_| SelectedMediaRejoinDenial::ControlFrame)?
    else {
        return Err(SelectedMediaRejoinDenial::ControlFrame);
    };
    let retained_wal = wal_inventory::admit_complete_inventory(&mut discovery, coordination)?;
    if let Some(tier) = tier_claim {
        tier::wal::verify(&retained_wal, tier)?;
    }
    let media = discovery.finish();
    // The C.8-installed binding basis is sealed to this C.9 stream's full
    // encoded digest and length. Sample actual C.9-admitted retained frames,
    // not an empty or caller-described WAL tail.
    let sample = PhysicalRecoveryFreshnessPort::sample_binding(
        coordination,
        &media,
        claim.checkpoint(),
        retained_wal.frames().iter(),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(|_| SelectedMediaRejoinDenial::WalFate)?;
    if !wal_fate::matches_selected_fate(
        store,
        descriptor.base().reclaim_attempt(),
        claim.accumulator().tip(),
        &sample,
        retained_wal.frames(),
        claim
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive(),
    ) {
        return Err(SelectedMediaRejoinDenial::WalFate);
    }
    for batch in claim.batches() {
        let attempt = controls
            .attempt_for_descriptor(batch.descriptor_record())
            .ok_or(SelectedMediaRejoinDenial::ControlFrame)?;
        if !wal_fate::matches_selected_fate(
            store,
            attempt,
            batch
                .tip_provenance()
                .map_err(|_| SelectedMediaRejoinDenial::CertificateRoster)?,
            &sample,
            retained_wal.frames(),
            claim
                .checkpoint()
                .compaction_cutover()
                .wal_cutoff_lsn_exclusive(),
        ) {
            return Err(SelectedMediaRejoinDenial::WalFate);
        }
    }
    if let Some(tier) = tier_claim {
        tier::wal::verify_sample(&sample, tier)?;
    }
    pause_before_final_reread();
    let mut final_discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(SelectedMediaRejoinDenial::Qualification)?;
    selected.matches_claim(&mut final_discovery, claim)?;
    let final_tier = if let Some(tier) = tier_claim {
        Some(tier::selection::observe(
            &mut final_discovery,
            store,
            reopen,
            tier,
        )?)
    } else {
        None
    };
    let final_routes = if let Some(tier) = final_tier.as_ref() {
        tier::routes::verify(
            &mut final_discovery,
            tier.root(),
            tier.free_header(),
            reopen.format(),
            None,
        )?
    } else {
        observe_unanchored_routes(&mut final_discovery, &selected, reopen.format())?
    };
    let final_partition = tier::released_partition::partition(
        &mut final_discovery,
        final_routes.selected_routes(),
        reopen.format(),
    )?;
    let mut final_failed_ingest_slices = final_partition.reservation_slices().to_vec();
    tier::verify_failed_ingest_subset(
        &mut final_discovery,
        final_partition.failed_ingest(),
        reopen.format(),
        selected.root().generation(),
        claim.checkpoint().source().identity().sequence().get(),
        &mut final_failed_ingest_slices,
    )?;
    let final_controls = control_frames::observe(
        &mut final_discovery,
        &selected,
        claim,
        final_partition.released(),
        tier_claim.map(|tier| tier.tier_epoch_start()),
        MAX_CONTROL_FRAME_BYTES,
    )?;
    if partition != final_partition
        || failed_ingest_slices != final_failed_ingest_slices
        || !controls.matches_reread(&final_controls)
    {
        return Err(SelectedMediaRejoinDenial::ControlFrame);
    }
    if !selected_tier.as_ref().is_none_or(|before| {
        final_tier
            .as_ref()
            .is_some_and(|after| before.matches_reread(after))
    }) || selected_routes != final_routes
    {
        return Err(SelectedMediaRejoinDenial::RoutingFrame);
    }
    let final_wal = wal_inventory::admit_complete_inventory(&mut final_discovery, coordination)?;
    if !retained_wal.matches_reread(&final_wal) {
        return Err(SelectedMediaRejoinDenial::WalFate);
    }
    if let Some(tier) = tier_claim {
        tier::wal::verify(&final_wal, tier)?;
    }
    root_checkpoint::revalidate_selected_checkpoint(&mut final_discovery, &selected)?;
    root_checkpoint::revalidate_selected_root(&mut final_discovery, &selected)?;
    let mut control_fingerprint = final_controls.fingerprint();
    control_fingerprint.extend(SelectedControlMediaFingerprint::observed(
        final_failed_ingest_slices,
    ));
    control_fingerprint.extend(final_routes.into_media_fingerprint());
    Ok((
        final_discovery.finish(),
        final_wal.fingerprint(),
        control_fingerprint,
    ))
}

fn observe_unanchored_routes(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &root_checkpoint::ObservedRootCheckpoint,
    format: PhysicalRecordFormatDeclaration,
) -> Result<tier::routes::NoReleaseControlProvenance, SelectedMediaRejoinDenial> {
    let bytes = discovery
        .read_free_space_manifest(
            selected.root().generation(),
            u64::from(format.page_size().bytes()),
        )
        .map_err(SelectedMediaRejoinDenial::Discovery)?;
    let bytes = bytes
        .bytes()
        .ok_or(SelectedMediaRejoinDenial::MissingFrame)?;
    let free = tier::selection::validate_selected_free_header(
        bytes,
        discovery.store_identity(),
        format,
        selected.root(),
    )?;
    if free.tier_epoch_start().is_some() {
        return Err(SelectedMediaRejoinDenial::RootBinding);
    }
    tier::routes::verify(discovery, selected.root(), &free, format, None)
}

#[derive(Debug)]
pub(super) enum SelectedMediaRejoinDenial {
    Resident(PhysicalRecoveryRejoinResidentDenial),
    ResidentBoundary {
        boundary: crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    WalResident {
        boundary: Option<crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary>,
        stage: crate::physical_runtime::PhysicalRecoveryWalResidentStage,
        artifact_count: usize,
        artifact_ordinal: Option<usize>,
        frame_offset: Option<u64>,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    ResidentRead {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    ReadBufferLengthMismatch {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        observed: usize,
    },
    Qualification(RecoveryFilesystemQualificationError),
    Discovery(crate::physical_runtime::RecoveryDiscoveryFailure),
    MissingSelector,
    MissingRoot,
    MissingCheckpoint,
    MissingRoute,
    MissingFrame,
    RootBinding,
    CheckpointBinding,
    CertificateRoster,
    RoutingFrame,
    UnsupportedSelectedPlacement,
    ControlFrame,
    BoundExceeded,
    WalFate,
}

impl SelectedMediaRejoinDenial {
    pub(super) fn at_resident_boundary(
        self,
        boundary: crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary,
    ) -> Self {
        match self {
            Self::Resident(cause) => Self::ResidentBoundary { boundary, cause },
            Self::WalResident {
                boundary: existing,
                stage,
                artifact_count,
                artifact_ordinal,
                frame_offset,
                cause,
            } => Self::WalResident {
                boundary: existing.or(Some(boundary)),
                stage,
                artifact_count,
                artifact_ordinal,
                frame_offset,
                cause,
            },
            other => other,
        }
    }
}
