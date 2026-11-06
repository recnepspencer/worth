use std::io::Write;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::acquisition_record::DurableForensicSourceRecord;
use super::acquisition_session::{output_name, sync_directory};
use super::{
    ForensicAcquisitionDenial, ForensicAcquisitionPlan, ForensicBundle, ForensicBundleRange,
    ForensicCustodyRecord, ForensicEvidencePosture, ForensicRangePosture,
};

pub(super) fn finalize_bundle(
    plan: &ForensicAcquisitionPlan,
    records: &[DurableForensicSourceRecord],
    completed_at_tick: u64,
) -> Result<ForensicBundle, ForensicAcquisitionDenial> {
    let ranges = ranges_from_records(records);
    let custody = custody_record(plan, completed_at_tick);
    let bundle_identity = forensic_bundle_identity(&custody, &ranges);
    persist_manifest(plan.target_root(), bundle_identity, &custody, &ranges)?;
    Ok(ForensicBundle {
        root: plan.target_root().to_path_buf(),
        bundle_identity,
        ranges,
        custody,
    })
}

fn ranges_from_records(records: &[DurableForensicSourceRecord]) -> Vec<ForensicBundleRange> {
    let mut ranges = Vec::with_capacity(records.len().saturating_mul(2));
    for record in records {
        if record.acquired_prefix_bytes > 0 {
            ranges.push(ForensicBundleRange {
                source_index: record.source_index as usize,
                source_offset: 0,
                byte_length: record.acquired_prefix_bytes,
                output_name: Some(output_name(record.source_index as usize)),
                digest: Some(*record.acquired_digest.bytes()),
                posture: ForensicRangePosture::Acquired,
            });
        }
        if record.unreadable_bytes() > 0 {
            ranges.push(ForensicBundleRange {
                source_index: record.source_index as usize,
                source_offset: record.acquired_prefix_bytes,
                byte_length: record.unreadable_bytes(),
                output_name: None,
                digest: None,
                posture: ForensicRangePosture::Unreadable,
            });
        }
    }
    ranges
}

fn custody_record(plan: &ForensicAcquisitionPlan, completed_at_tick: u64) -> ForensicCustodyRecord {
    ForensicCustodyRecord {
        observer_identity: plan.observer_identity().to_owned(),
        acquisition_method: plan.acquisition_method().to_owned(),
        consistency_basis_identity: plan.consistency_basis_identity,
        source_media_fingerprints: plan
            .sources
            .iter()
            .map(|source| source.metadata_fingerprint)
            .collect(),
        clock_provenance: plan.clock_provenance().to_owned(),
        started_at_tick: plan.started_at_tick(),
        completed_at_tick,
        integrity_posture: ForensicEvidencePosture::UntrustedObservation,
        authenticity_posture: ForensicEvidencePosture::Unavailable,
        custody_posture: ForensicEvidencePosture::UntrustedObservation,
        transformation_identity: Sha256::digest(b"worth-store-byte-preserving-forensic-copy-v1")
            .into(),
    }
}

fn forensic_bundle_identity(
    custody: &ForensicCustodyRecord,
    ranges: &[ForensicBundleRange],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-forensic-bundle-v2");
    digest.update(custody.observer_identity.as_bytes());
    digest.update(custody.acquisition_method.as_bytes());
    digest.update(custody.clock_provenance.as_bytes());
    digest.update(custody.started_at_tick.to_be_bytes());
    digest.update(custody.completed_at_tick.to_be_bytes());
    digest.update(custody.consistency_basis_identity);
    digest.update(custody.transformation_identity);
    for fingerprint in &custody.source_media_fingerprints {
        digest.update(fingerprint);
    }
    for range in ranges {
        digest.update((range.source_index as u64).to_be_bytes());
        digest.update(range.source_offset.to_be_bytes());
        digest.update(range.byte_length.to_be_bytes());
        digest.update([range.posture as u8]);
        digest.update(range.digest.unwrap_or([0; 32]));
    }
    digest.finalize().into()
}

fn persist_manifest(
    root: &Path,
    identity: [u8; 32],
    custody: &ForensicCustodyRecord,
    ranges: &[ForensicBundleRange],
) -> Result<(), ForensicAcquisitionDenial> {
    let mut manifest = Vec::new();
    manifest.extend_from_slice(b"WORTHFORENSIC2\n");
    manifest.extend_from_slice(&identity);
    manifest.extend_from_slice(&custody.consistency_basis_identity);
    manifest.extend_from_slice(&custody.started_at_tick.to_be_bytes());
    manifest.extend_from_slice(&custody.completed_at_tick.to_be_bytes());
    manifest.extend_from_slice(&(ranges.len() as u64).to_be_bytes());
    for range in ranges {
        manifest.extend_from_slice(&(range.source_index as u64).to_be_bytes());
        manifest.extend_from_slice(&range.source_offset.to_be_bytes());
        manifest.extend_from_slice(&range.byte_length.to_be_bytes());
        manifest.push(range.posture as u8);
        manifest.extend_from_slice(&range.digest.unwrap_or([0; 32]));
    }
    let final_path = root.join("forensic.manifest");
    if final_path.exists() {
        if std::fs::read(&final_path)? == manifest {
            return Ok(());
        }
        return Err(ForensicAcquisitionDenial::TargetAlreadyContainsConflict);
    }
    let pending_path = root.join(".forensic.manifest.pending");
    if pending_path.exists() {
        std::fs::remove_file(&pending_path)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending_path)?;
    file.write_all(&manifest)?;
    file.sync_all()?;
    std::fs::rename(&pending_path, &final_path)?;
    sync_directory(root)?;
    Ok(())
}
