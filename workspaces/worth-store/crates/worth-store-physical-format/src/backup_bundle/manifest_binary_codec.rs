use std::io::Read;

use super::manifest_binary_cursor::{FallibleEncoder, ManifestDecoder};
use super::manifest_binary_tags::{family_from_tag, family_tag, format_from_tag, format_tag};
use super::{
    BackupBundleArtifactCoverage, BackupBundleArtifactManifestRow, BackupBundleFormatDenial,
    BackupBundleManifest, BackupBundleManifestDeclaration, BackupBundleManifestIdentity,
    BackupBundlePhysicalOwner, BackupBundleRecoveryCoordinates,
};
use crate::{
    PhysicalCellReuseDomain, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordSlot, PhysicalRootReference, PhysicalSegmentId,
};

const MANIFEST_MAGIC: [u8; 8] = *b"WSBMF001";

pub(super) fn encode_manifest(
    manifest: &BackupBundleManifest,
) -> Result<Vec<u8>, BackupBundleFormatDenial> {
    let mut output = FallibleEncoder::new();
    output.bytes(&MANIFEST_MAGIC)?;
    output.bytes(&manifest.cut_identity())?;
    output.string(manifest.store_lineage())?;
    output.u64(manifest.root_generation())?;
    output.u64(manifest.manifest_generation())?;
    output.string(manifest.checkpoint_identity())?;
    output.u64(manifest.durable_checkpoint_lsn())?;
    let (wal_start, wal_end) = manifest.wal_half_open_interval();
    output.u64(wal_start)?;
    output.u64(wal_end)?;
    output.u64(manifest.acknowledged_frontier())?;
    output.u64(manifest.security_scope_fingerprint())?;
    output.u32(
        u32::try_from(manifest.artifacts().len())
            .map_err(|_| BackupBundleFormatDenial::InvalidManifest)?,
    )?;
    for row in manifest.artifacts() {
        encode_row(&mut output, row)?;
    }
    output.bytes(&manifest.artifact_closure_digest())?;
    Ok(output.finish())
}

pub(super) fn decode_manifest(
    reader: impl Read,
    maximum_artifacts: u64,
    encoded_bytes: u64,
    maximum_owned_allocation_bytes: u64,
) -> Result<BackupBundleManifest, BackupBundleFormatDenial> {
    let mut input = ManifestDecoder::new(reader, encoded_bytes, maximum_owned_allocation_bytes);
    if input.array::<8>()? != MANIFEST_MAGIC {
        return Err(BackupBundleFormatDenial::InvalidManifest);
    }
    let cut_identity = input.array::<32>()?;
    let store_lineage = input.string()?;
    let root_generation = input.u64()?;
    let manifest_generation = input.u64()?;
    let checkpoint_identity = input.string()?;
    let durable_checkpoint_lsn = input.u64()?;
    let wal_start = input.u64()?;
    let wal_end = input.u64()?;
    let acknowledged_frontier = input.u64()?;
    let security_scope_fingerprint = input.u64()?;
    let artifacts = decode_artifacts(&mut input, maximum_artifacts)?;
    let encoded_closure_digest = input.array::<32>()?;
    input.require_eof()?;
    let manifest = BackupBundleManifest::from_decoded_parts(
        BackupBundleManifestDeclaration::new(
            BackupBundleManifestIdentity {
                cut_identity,
                store_lineage,
                root_generation,
                manifest_generation,
            },
            BackupBundleRecoveryCoordinates {
                checkpoint_identity,
                durable_checkpoint_lsn,
                wal_half_open_interval: (wal_start, wal_end),
                acknowledged_frontier,
            },
            security_scope_fingerprint,
            artifacts,
        ),
        encoded_closure_digest,
    );
    Ok(manifest)
}

fn decode_artifacts<R: Read>(
    input: &mut ManifestDecoder<R>,
    maximum_artifacts: u64,
) -> Result<Vec<BackupBundleArtifactManifestRow>, BackupBundleFormatDenial> {
    let artifact_count = u64::from(input.u32()?);
    if artifact_count > maximum_artifacts {
        return Err(BackupBundleFormatDenial::ManifestArtifactLimitExceeded {
            artifacts: artifact_count,
            maximum_artifacts,
        });
    }
    let artifact_count = usize::try_from(artifact_count)
        .map_err(|_| BackupBundleFormatDenial::ManifestAllocationCountOverflow)?;
    let mut artifacts = Vec::new();
    input.charge_owned_allocation(
        u64::try_from(artifact_count)
            .ok()
            .and_then(|count| {
                count.checked_mul(std::mem::size_of::<BackupBundleArtifactManifestRow>() as u64)
            })
            .ok_or(BackupBundleFormatDenial::ManifestAllocationCountOverflow)?,
    )?;
    artifacts
        .try_reserve_exact(artifact_count)
        .map_err(|_| BackupBundleFormatDenial::ManifestAllocationFailed)?;
    for _ in 0..artifact_count {
        artifacts.push(decode_row(input)?);
    }
    Ok(artifacts)
}

fn encode_row(
    output: &mut FallibleEncoder,
    row: &BackupBundleArtifactManifestRow,
) -> Result<(), BackupBundleFormatDenial> {
    output.u8(family_tag(row.family()))?;
    output.u8(format_tag(row.format()))?;
    output.string(row.identity())?;
    output.string(row.output_name())?;
    output.u64(row.generation())?;
    output.u64(row.bytes())?;
    output.bytes(&row.content_digest())?;
    encode_coverage(output, row.coverage())?;
    encode_owner(output, row.reclaim_owner())
}

fn decode_row(
    input: &mut ManifestDecoder<impl Read>,
) -> Result<BackupBundleArtifactManifestRow, BackupBundleFormatDenial> {
    let family = family_from_tag(input.u8()?).ok_or(BackupBundleFormatDenial::InvalidManifest)?;
    let format = format_from_tag(input.u8()?).ok_or(BackupBundleFormatDenial::InvalidManifest)?;
    let identity = input.string()?;
    let output_name = input.string()?;
    let generation = input.u64()?;
    let bytes = input.u64()?;
    let content_digest = input.array::<32>()?;
    let coverage = decode_coverage(input)?;
    let owner = decode_owner(input)?;
    BackupBundleArtifactManifestRow::new(
        family,
        format,
        identity,
        output_name,
        generation,
        bytes,
        content_digest,
        coverage,
        owner,
    )
    .ok_or(BackupBundleFormatDenial::InvalidManifest)
}

fn encode_coverage(
    output: &mut FallibleEncoder,
    coverage: &BackupBundleArtifactCoverage,
) -> Result<(), BackupBundleFormatDenial> {
    match coverage {
        BackupBundleArtifactCoverage::RootManifest { root_generation } => {
            output.u8(1)?;
            output.u64(*root_generation)
        }
        BackupBundleArtifactCoverage::CheckpointManifest {
            checkpoint_identity,
            manifest_generation,
            durable_checkpoint_lsn,
            authority_fingerprint,
            frontier_digest,
        } => {
            output.u8(2)?;
            output.string(checkpoint_identity)?;
            output.u64(*manifest_generation)?;
            output.u64(*durable_checkpoint_lsn)?;
            output.bytes(authority_fingerprint)?;
            output.bytes(frontier_digest)
        }
        BackupBundleArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } => {
            output.u8(3)?;
            output.u64(*start_lsn)?;
            output.u64(*end_exclusive_lsn)
        }
        BackupBundleArtifactCoverage::PhysicalReachability => output.u8(4),
        BackupBundleArtifactCoverage::SecondaryRoot { root_generation } => {
            output.u8(5)?;
            output.u64(*root_generation)
        }
    }
}

fn decode_coverage(
    input: &mut ManifestDecoder<impl Read>,
) -> Result<BackupBundleArtifactCoverage, BackupBundleFormatDenial> {
    Ok(match input.u8()? {
        1 => BackupBundleArtifactCoverage::RootManifest {
            root_generation: input.u64()?,
        },
        2 => BackupBundleArtifactCoverage::CheckpointManifest {
            checkpoint_identity: input.string()?,
            manifest_generation: input.u64()?,
            durable_checkpoint_lsn: input.u64()?,
            authority_fingerprint: input.array()?,
            frontier_digest: input.array()?,
        },
        3 => BackupBundleArtifactCoverage::WalSegment {
            start_lsn: input.u64()?,
            end_exclusive_lsn: input.u64()?,
        },
        4 => BackupBundleArtifactCoverage::PhysicalReachability,
        5 => BackupBundleArtifactCoverage::SecondaryRoot {
            root_generation: input.u64()?,
        },
        _ => return Err(BackupBundleFormatDenial::InvalidManifest),
    })
}

fn encode_owner(
    output: &mut FallibleEncoder,
    owner: BackupBundlePhysicalOwner,
) -> Result<(), BackupBundleFormatDenial> {
    let owner = owner
        .generation_owner()
        .ok_or(BackupBundleFormatDenial::InvalidManifest)?;
    output.u8(owner_domain_tag(owner.domain()).ok_or(BackupBundleFormatDenial::InvalidManifest)?)?;
    output.u64(owner.segment_id().map_or(0, PhysicalSegmentId::get))?;
    output.u64(owner.page_id().map_or(0, PhysicalPageId::get))?;
    output.u64(owner.extent_id().map_or(0, PhysicalExtentId::get))?;
    output.u16(owner.slot().map_or(0, PhysicalRecordSlot::get))?;
    output.u64(owner.root_reference().map_or(0, PhysicalRootReference::get))?;
    output.u64(owner.generation().get())
}

fn decode_owner(
    input: &mut ManifestDecoder<impl Read>,
) -> Result<BackupBundlePhysicalOwner, BackupBundleFormatDenial> {
    let domain = input.u8()?;
    let segment = input.u64()?;
    let page = input.u64()?;
    let extent = input.u64()?;
    let slot = input.u16()?;
    let root = input.u64()?;
    let generation = PhysicalGeneration::from_raw(input.u64()?)
        .map_err(|_| BackupBundleFormatDenial::InvalidManifest)?;
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let owner = match domain {
        1 if extent == 0 && root == 0 => authority
            .slot_cell(segment_id(segment)?, page_id(page)?, record_slot(slot)?)
            .with_slot_generation(generation)
            .owner(),
        2 if page == 0 && slot == 0 && root == 0 => authority
            .extent_cell(segment_id(segment)?, extent_id(extent)?)
            .with_extent_generation(generation)
            .owner(),
        7 if segment == 0 && page == 0 && slot == 0 && root == 0 => authority
            .record_extent_cell(extent_id(extent)?)
            .with_extent_generation(generation)
            .owner(),
        4 if segment == 0 && page == 0 && extent == 0 && slot == 0 => authority
            .root_publication_cell(root_reference(root)?)
            .with_root_publication_generation(generation)
            .owner(),
        5 if extent == 0 && slot == 0 && root == 0 => authority
            .page_cell(segment_id(segment)?, page_id(page)?)
            .with_page_generation(generation)
            .owner(),
        6 if page == 0 && extent == 0 && slot == 0 && root == 0 => authority
            .segment_cell(segment_id(segment)?)
            .with_segment_generation(generation)
            .owner(),
        _ => return Err(BackupBundleFormatDenial::InvalidManifest),
    };
    Ok(BackupBundlePhysicalOwner::from_generation_owner(owner))
}

const fn owner_domain_tag(domain: PhysicalCellReuseDomain) -> Option<u8> {
    match domain {
        PhysicalCellReuseDomain::SlotAllocation => Some(1),
        PhysicalCellReuseDomain::ExtentAllocation => Some(2),
        PhysicalCellReuseDomain::RecordExtentAllocation => Some(7),
        PhysicalCellReuseDomain::RootPublication => Some(4),
        PhysicalCellReuseDomain::Page => Some(5),
        PhysicalCellReuseDomain::Segment => Some(6),
        PhysicalCellReuseDomain::FreeSpaceReuse => None,
    }
}

fn segment_id(value: u64) -> Result<PhysicalSegmentId, BackupBundleFormatDenial> {
    PhysicalSegmentId::from_raw(value).map_err(|_| BackupBundleFormatDenial::InvalidManifest)
}
fn page_id(value: u64) -> Result<PhysicalPageId, BackupBundleFormatDenial> {
    PhysicalPageId::from_raw(value).map_err(|_| BackupBundleFormatDenial::InvalidManifest)
}
fn extent_id(value: u64) -> Result<PhysicalExtentId, BackupBundleFormatDenial> {
    PhysicalExtentId::from_raw(value).map_err(|_| BackupBundleFormatDenial::InvalidManifest)
}
fn record_slot(value: u16) -> Result<PhysicalRecordSlot, BackupBundleFormatDenial> {
    PhysicalRecordSlot::from_raw(value).map_err(|_| BackupBundleFormatDenial::InvalidManifest)
}
fn root_reference(value: u64) -> Result<PhysicalRootReference, BackupBundleFormatDenial> {
    PhysicalRootReference::from_raw(value).map_err(|_| BackupBundleFormatDenial::InvalidManifest)
}
