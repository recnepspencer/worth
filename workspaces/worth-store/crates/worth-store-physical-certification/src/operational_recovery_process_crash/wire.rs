use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use worth_foundational::CanonicalDigestId;

use worth_store_authority::ControlStoreSelectionCoordinates;
use worth_store_operations::OperationalControlSessionObservation;

use super::OperationalRecoveryProcessCrashDenial;
use crate::certification_child_process::publish_new_synced;
use crate::OperationalRecoveryYieldpoint;

const MAGIC: &[u8; 8] = b"WS10CUT2";
const MAX_OPERATIONS: usize = 128;
const MAX_OPERATION_BYTES: usize = 256;

#[derive(Debug)]
pub(super) struct ProcessObservationReport {
    pub(super) challenge: CanonicalDigestId,
    pub(super) yieldpoint: OperationalRecoveryYieldpoint,
    pub(super) observation: OperationalControlSessionObservation,
    pub(super) trace_identity: CanonicalDigestId,
    pub(super) operations: Vec<String>,
}

pub(super) fn write_report(path: &Path, report: &ProcessObservationReport) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(report.challenge.bytes());
    let point = OperationalRecoveryYieldpoint::ALL
        .iter()
        .position(|candidate| *candidate == report.yieldpoint)
        .expect("declared yieldpoint") as u16;
    bytes.extend_from_slice(&point.to_be_bytes());
    append_observation(&mut bytes, report.observation);
    bytes.extend_from_slice(report.trace_identity.bytes());
    assert!(report.operations.len() <= MAX_OPERATIONS);
    bytes.extend_from_slice(&(report.operations.len() as u16).to_be_bytes());
    for operation in &report.operations {
        assert!(operation.len() <= MAX_OPERATION_BYTES);
        bytes.extend_from_slice(&(operation.len() as u16).to_be_bytes());
        bytes.extend_from_slice(operation.as_bytes());
    }
    publish_new_synced(path, &bytes)
}

pub(super) fn read_report(
    path: &Path,
) -> Result<ProcessObservationReport, OperationalRecoveryProcessCrashDenial> {
    let mut bytes = Vec::new();
    OpenOptions::new()
        .read(true)
        .open(path)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .map_err(|_| OperationalRecoveryProcessCrashDenial::MissingOrMalformedReport)?;
    decode_report(&bytes).ok_or(OperationalRecoveryProcessCrashDenial::MissingOrMalformedReport)
}

fn append_observation(bytes: &mut Vec<u8>, observation: OperationalControlSessionObservation) {
    bytes.extend_from_slice(&observation.process().fingerprint());
    bytes.extend_from_slice(&observation.session().fingerprint());
    bytes.extend_from_slice(&observation.media_identity_fingerprint());
    match observation.coordinates() {
        Some(coordinates) => {
            bytes.push(1);
            bytes.extend_from_slice(&coordinates.media_identity_fingerprint());
            bytes.extend_from_slice(&coordinates.generation().get().to_be_bytes());
            bytes.extend_from_slice(&coordinates.prefix_digest());
        }
        None => bytes.push(0),
    }
}

fn decode_report(bytes: &[u8]) -> Option<ProcessObservationReport> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(8)? != MAGIC {
        return None;
    }
    let challenge = cursor.array()?;
    let point = u16::from_be_bytes(cursor.array()?) as usize;
    let yieldpoint = *OperationalRecoveryYieldpoint::ALL.get(point)?;
    let process = cursor.array()?;
    let session = cursor.array()?;
    let media = cursor.array()?;
    let coordinates = match cursor.byte()? {
        0 => None,
        1 => Some(ControlStoreSelectionCoordinates::new(
            cursor.array()?,
            worth_store_authority::ControlStoreGeneration::from_raw(u64::from_be_bytes(
                cursor.array()?,
            ))?,
            cursor.array()?,
        )),
        _ => return None,
    };
    let trace_identity = cursor.array()?;
    let count = u16::from_be_bytes(cursor.array()?) as usize;
    if count > MAX_OPERATIONS {
        return None;
    }
    let mut operations = Vec::with_capacity(count);
    for _ in 0..count {
        let len = u16::from_be_bytes(cursor.array()?) as usize;
        if len > MAX_OPERATION_BYTES {
            return None;
        }
        operations.push(std::str::from_utf8(cursor.take(len)?).ok()?.to_owned());
    }
    if !cursor.done() {
        return None;
    }
    Some(ProcessObservationReport {
        challenge: CanonicalDigestId::new(challenge),
        yieldpoint,
        observation:
            worth_store_operations::OperationalControlSessionObservation::from_untrusted_certification_report(
                process, session, media, coordinates,
            ),
        trace_identity: CanonicalDigestId::new(trace_identity),
        operations,
    })
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.offset.checked_add(len)?;
        let value = self.bytes.get(self.offset..end)?;
        self.offset = end;
        Some(value)
    }
    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }
    fn byte(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn done(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
