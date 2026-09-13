use sha2::{Digest, Sha256};

use super::observer_evidence::{
    RecoveryObserverCheckpointCoverageEvidence, RecoveryObserverManifestMembershipEvidence,
    RecoveryObserverPageLsnEvidence, RecoveryObserverSelectorEvidence,
    RecoveryObserverWalPrefixEvidence,
};
use super::observer_evidence_summary::RecoveryObserverEvidence;
use super::report::RecoveryObserverReport;
use super::report_protocol::{self, RecoveryObserverDecodeDenial};

pub(super) fn encode(report: RecoveryObserverReport) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(512);
    report_protocol::encode_header(&mut bytes);
    bytes.extend_from_slice(&report.artifact_count.to_le_bytes());
    bytes.extend_from_slice(&report.bytes_read.to_le_bytes());
    bytes.extend_from_slice(&report.artifact_set_digest);
    append_digest(&mut bytes, report.evidence.artifact_identities());
    append_digest(&mut bytes, report.evidence.generation_links());
    let selectors = report.evidence.durable_selectors();
    bytes.extend_from_slice(&selectors.selector_count().to_le_bytes());
    bytes.extend_from_slice(&selectors.linked_selector_count().to_le_bytes());
    bytes.extend_from_slice(&selectors.unpaired_link_count().to_le_bytes());
    append_optional_array(&mut bytes, selectors.store_identity());
    append_optional_u64(&mut bytes, selectors.current_root_generation());
    bytes.extend_from_slice(&selectors.digest());
    let checkpoint = report.evidence.checkpoint_coverage();
    bytes.extend_from_slice(&checkpoint.checkpoint_count().to_le_bytes());
    bytes.extend_from_slice(&checkpoint.page_count().to_le_bytes());
    append_optional_u64(&mut bytes, checkpoint.covered_lsn_start());
    append_optional_u64(&mut bytes, checkpoint.covered_lsn_end());
    append_optional_u64(&mut bytes, checkpoint.redo_lsn());
    append_optional_u64(&mut bytes, checkpoint.durable_checkpoint_lsn());
    bytes.extend_from_slice(&checkpoint.digest());
    let wal = report.evidence.valid_wal_prefix();
    bytes.extend_from_slice(&wal.segment_count().to_le_bytes());
    bytes.extend_from_slice(&wal.valid_prefix_bytes().to_le_bytes());
    bytes.extend_from_slice(&wal.observed_bytes().to_le_bytes());
    bytes.extend_from_slice(&wal.frame_count().to_le_bytes());
    append_optional_u64(&mut bytes, wal.first_lsn());
    append_optional_u64(&mut bytes, wal.last_lsn());
    bytes.extend_from_slice(&wal.digest());
    let page_lsns = report.evidence.page_lsns();
    bytes.extend_from_slice(&page_lsns.observation_count().to_le_bytes());
    append_optional_u64(&mut bytes, page_lsns.minimum());
    append_optional_u64(&mut bytes, page_lsns.maximum());
    bytes.extend_from_slice(&page_lsns.digest());
    let manifests = report.evidence.manifest_membership();
    bytes.extend_from_slice(&manifests.manifest_count().to_le_bytes());
    bytes.extend_from_slice(&manifests.member_count().to_le_bytes());
    bytes.extend_from_slice(&manifests.digest());
    let residue = report.evidence.residue();
    bytes.extend_from_slice(&residue.artifact_count().to_le_bytes());
    bytes.extend_from_slice(&residue.bytes().to_le_bytes());
    bytes.extend_from_slice(&residue.digest());
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    bytes
}

pub(super) fn decode(
    encoded: &[u8],
) -> Result<RecoveryObserverReport, RecoveryObserverDecodeDenial> {
    if encoded.len() < 32 {
        return Err(RecoveryObserverDecodeDenial::Malformed);
    }
    let (payload, digest) = encoded.split_at(encoded.len() - 32);
    let expected: [u8; 32] = Sha256::digest(payload).into();
    if digest != expected {
        return Err(RecoveryObserverDecodeDenial::DigestMismatch);
    }
    let mut bytes = payload;
    report_protocol::admit_header(&mut bytes)?;
    let artifact_count = report_protocol::u64_value(&mut bytes)?;
    let bytes_read = report_protocol::u64_value(&mut bytes)?;
    let artifact_set_digest = report_protocol::array(&mut bytes)?;
    let artifact_identities = decode_digest(&mut bytes)?;
    let generation_links = decode_digest(&mut bytes)?;
    let selectors = RecoveryObserverSelectorEvidence::from_parts(
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        decode_optional_array(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        report_protocol::array(&mut bytes)?,
    );
    let checkpoint = RecoveryObserverCheckpointCoverageEvidence::from_parts(
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        report_protocol::array(&mut bytes)?,
    );
    let wal = RecoveryObserverWalPrefixEvidence::from_parts(
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        report_protocol::array(&mut bytes)?,
    );
    let page_lsns = RecoveryObserverPageLsnEvidence::from_parts(
        report_protocol::u64_value(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        decode_optional_u64(&mut bytes)?,
        report_protocol::array(&mut bytes)?,
    );
    let manifests = RecoveryObserverManifestMembershipEvidence::from_parts(
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::array(&mut bytes)?,
    );
    let residue = super::observer_evidence::RecoveryObserverResidueEvidence::from_parts(
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::u64_value(&mut bytes)?,
        report_protocol::array(&mut bytes)?,
    );
    if !bytes.is_empty() {
        return Err(RecoveryObserverDecodeDenial::Malformed);
    }
    Ok(RecoveryObserverReport {
        artifact_count,
        bytes_read,
        artifact_set_digest,
        evidence: RecoveryObserverEvidence::from_parts(
            artifact_identities,
            generation_links,
            selectors,
            checkpoint,
            wal,
            page_lsns,
            manifests,
            residue,
        ),
    })
}

fn append_digest(
    bytes: &mut Vec<u8>,
    digest: super::observer_evidence::RecoveryObserverEvidenceDigest,
) {
    bytes.extend_from_slice(&digest.observations().to_le_bytes());
    bytes.extend_from_slice(&digest.digest());
}

fn decode_digest(
    bytes: &mut &[u8],
) -> Result<super::observer_evidence::RecoveryObserverEvidenceDigest, RecoveryObserverDecodeDenial>
{
    Ok(
        super::observer_evidence::RecoveryObserverEvidenceDigest::from_parts(
            report_protocol::u64_value(bytes)?,
            report_protocol::array(bytes)?,
        ),
    )
}

fn append_optional_u64(bytes: &mut Vec<u8>, value: Option<u64>) {
    bytes.push(u8::from(value.is_some()));
    if let Some(value) = value {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn append_optional_array(bytes: &mut Vec<u8>, value: Option<[u8; 16]>) {
    match value {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value);
        }
        None => bytes.push(0),
    }
}

fn decode_optional_array(
    bytes: &mut &[u8],
) -> Result<Option<[u8; 16]>, RecoveryObserverDecodeDenial> {
    match bytes.first().copied() {
        Some(0) => {
            *bytes = &bytes[1..];
            Ok(None)
        }
        Some(1) => {
            *bytes = &bytes[1..];
            Ok(Some(report_protocol::array(bytes)?))
        }
        _ => Err(RecoveryObserverDecodeDenial::Malformed),
    }
}

fn decode_optional_u64(bytes: &mut &[u8]) -> Result<Option<u64>, RecoveryObserverDecodeDenial> {
    match bytes.first().copied() {
        Some(0) => {
            *bytes = &bytes[1..];
            Ok(None)
        }
        Some(1) => {
            *bytes = &bytes[1..];
            Ok(Some(report_protocol::u64_value(bytes)?))
        }
        _ => Err(RecoveryObserverDecodeDenial::Malformed),
    }
}
