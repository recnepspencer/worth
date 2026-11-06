use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use worth_foundational::CanonicalDigestId;

use worth_store_offline_verifier::OperationalTruthRegion;

use super::FreshProcessOfflineTruthDenial;
use crate::certification_child_process::publish_new_synced;

const MAGIC: &[u8; 8] = b"WS10TRU1";
const REPORT_BYTES: usize = 8 + 32 + 4 + 32 + 32 + 32 + 1 + 8 + 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum TruthRegionKind {
    TrustedAuthority = 1,
    DegradedDerived = 2,
    Rebuildable = 3,
    Quarantined = 4,
    UnrecoverableAuthority = 5,
    Indeterminate = 6,
    AliasGroup = 7,
    OverlapConflict = 8,
}

pub(super) struct FreshProcessTruthReport {
    pub(super) challenge: CanonicalDigestId,
    pub(super) observer_process_id: u32,
    pub(super) source_inspection_identity: CanonicalDigestId,
    pub(super) truth_evidence_identity: CanonicalDigestId,
    pub(super) observed_content_digest: CanonicalDigestId,
    pub(super) region_kind: TruthRegionKind,
    pub(super) start: u64,
    pub(super) end: u64,
}

impl TruthRegionKind {
    pub(super) const fn from_region(region: &OperationalTruthRegion) -> Self {
        match region {
            OperationalTruthRegion::TrustedAuthorityRegion(_) => Self::TrustedAuthority,
            OperationalTruthRegion::DegradedDerivedRegion(_) => Self::DegradedDerived,
            OperationalTruthRegion::RebuildableRegion(_) => Self::Rebuildable,
            OperationalTruthRegion::QuarantinedRegion(_) => Self::Quarantined,
            OperationalTruthRegion::UnrecoverableAuthorityRegion(_) => Self::UnrecoverableAuthority,
            OperationalTruthRegion::IndeterminateTruthRegion(_) => Self::Indeterminate,
            OperationalTruthRegion::AliasGroup { .. } => Self::AliasGroup,
            OperationalTruthRegion::OverlapConflict { .. } => Self::OverlapConflict,
        }
    }

    fn decode(value: u8) -> Option<Self> {
        Some(match value {
            1 => Self::TrustedAuthority,
            2 => Self::DegradedDerived,
            3 => Self::Rebuildable,
            4 => Self::Quarantined,
            5 => Self::UnrecoverableAuthority,
            6 => Self::Indeterminate,
            7 => Self::AliasGroup,
            8 => Self::OverlapConflict,
            _ => return None,
        })
    }
}

pub(super) fn write_report(
    path: &Path,
    report: &FreshProcessTruthReport,
) -> Result<(), FreshProcessOfflineTruthDenial> {
    let mut bytes = Vec::with_capacity(REPORT_BYTES);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(report.challenge.bytes());
    bytes.extend_from_slice(&report.observer_process_id.to_be_bytes());
    bytes.extend_from_slice(report.source_inspection_identity.bytes());
    bytes.extend_from_slice(report.truth_evidence_identity.bytes());
    bytes.extend_from_slice(report.observed_content_digest.bytes());
    bytes.push(report.region_kind as u8);
    bytes.extend_from_slice(&report.start.to_be_bytes());
    bytes.extend_from_slice(&report.end.to_be_bytes());
    publish_new_synced(path, &bytes)?;
    Ok(())
}

pub(super) fn read_report(
    path: &Path,
) -> Result<FreshProcessTruthReport, FreshProcessOfflineTruthDenial> {
    let mut bytes = Vec::with_capacity(REPORT_BYTES);
    OpenOptions::new()
        .read(true)
        .open(path)
        .and_then(|mut file| file.read_to_end(&mut bytes))?;
    if bytes.len() != REPORT_BYTES {
        return Err(FreshProcessOfflineTruthDenial::MissingOrMalformedReport);
    }
    let mut cursor = Cursor::new(&bytes);
    if cursor.take(8) != Some(MAGIC.as_slice()) {
        return Err(FreshProcessOfflineTruthDenial::MissingOrMalformedReport);
    }
    let report = FreshProcessTruthReport {
        challenge: CanonicalDigestId::new(cursor.array()?),
        observer_process_id: u32::from_be_bytes(cursor.array()?),
        source_inspection_identity: CanonicalDigestId::new(cursor.array()?),
        truth_evidence_identity: CanonicalDigestId::new(cursor.array()?),
        observed_content_digest: CanonicalDigestId::new(cursor.array()?),
        region_kind: TruthRegionKind::decode(cursor.byte()?)
            .ok_or(FreshProcessOfflineTruthDenial::MissingOrMalformedReport)?,
        start: u64::from_be_bytes(cursor.array()?),
        end: u64::from_be_bytes(cursor.array()?),
    };
    if !cursor.done() {
        return Err(FreshProcessOfflineTruthDenial::MissingOrMalformedReport);
    }
    Ok(report)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let end = self.offset.checked_add(length)?;
        let value = self.bytes.get(self.offset..end)?;
        self.offset = end;
        Some(value)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], FreshProcessOfflineTruthDenial> {
        self.take(N)
            .and_then(|value| value.try_into().ok())
            .ok_or(FreshProcessOfflineTruthDenial::MissingOrMalformedReport)
    }
    fn byte(&mut self) -> Result<u8, FreshProcessOfflineTruthDenial> {
        self.take(1)
            .map(|value| value[0])
            .ok_or(FreshProcessOfflineTruthDenial::MissingOrMalformedReport)
    }
    const fn done(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
