use super::RetiredArtifact;

/// Exact generation a held retirement claim may delete. Only the admission
/// owner can issue one, and only while that generation is claimed.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct RetirementRemovalPermit {
    artifact: RetiredArtifact,
}

impl RetirementRemovalPermit {
    pub(super) const fn issued(artifact: RetiredArtifact) -> Self {
        Self { artifact }
    }

    /// Whether this permit names `file` as one of its generation's files.
    pub(in crate::physical_runtime) fn admits(
        self,
        file: worth_store_physical_format::RecordArtifactFile,
    ) -> bool {
        self.artifact.admits_removal(file)
    }
}

pub(in crate::physical_runtime) const RETIREMENT_DOMAIN: &[u8] = b"store.physical.retirement.v1";

pub(in crate::physical_runtime) const RETIREMENT_INTENT: u8 = 1;
pub(in crate::physical_runtime) const RETIREMENT_COMPLETION: u8 = 2;
pub(in crate::physical_runtime) const RETIREMENT_EXTENT_INTENT: u8 = 3;
pub(in crate::physical_runtime) const RETIREMENT_EXTENT_COMPLETION: u8 = 4;

/// Why `retire_displaced_segment` did not complete a retirement.
///
/// Claim denials (`Absent` through `Retained`) precede every effect. Stage
/// denials name the first stage that did not finish; a durable intent survives
/// them and a later call or fresh recovery resumes the same retirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRetirementDenial {
    /// No displaced generation awaits retirement.
    Absent,
    /// A live reader still protects the source root or the displaced generation.
    Protected,
    /// A root-changing publication is still pending.
    Unresolved,
    /// The current root still reads the generation, or the successor checkpoint
    /// did not move past its source root.
    Retained,
    /// The retirement WAL frame could not be planned.
    WalPlan,
    /// Writing the retirement WAL frame failed.
    WalWrite,
    /// Synchronizing the retirement WAL frame failed.
    WalSync,
    /// Settling the retirement WAL frame failed.
    WalFinish,
    /// Another retirement is running, or the scheduler has not started the
    /// frame. The claim stays charged.
    Waiting,
    /// Deleting the generation or synchronizing its namespace failed.
    Delete,
    /// The successor-root checkpoint did not complete.
    Checkpoint,
}

pub(in crate::physical_runtime) fn payload_is_retirement(payload: &[u8]) -> bool {
    let Some(encoded_len) = payload.get(..8) else {
        return false;
    };
    let Ok(encoded_len) = <[u8; 8]>::try_from(encoded_len) else {
        return false;
    };
    let length = u64::from_le_bytes(encoded_len);
    length == RETIREMENT_DOMAIN.len() as u64
        && payload.get(8..8 + RETIREMENT_DOMAIN.len()) == Some(RETIREMENT_DOMAIN)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct RetirementRecord {
    pub(in crate::physical_runtime) artifact: RetiredArtifact,
    pub(in crate::physical_runtime) completion: bool,
    pub(in crate::physical_runtime) source_root: u64,
    pub(in crate::physical_runtime) bytes: u64,
}

pub(in crate::physical_runtime) fn decode_retirement(payload: &[u8]) -> Option<RetirementRecord> {
    if !payload_is_retirement(payload) {
        return None;
    }
    let body = payload.get(8 + RETIREMENT_DOMAIN.len()..)?;
    let (action, body) = body.split_first()?;
    if body.len() != 32 {
        return None;
    }
    let mut numbers = [0u64; 4];
    for (index, number) in numbers.iter_mut().enumerate() {
        let start = index * 8;
        *number = u64::from_le_bytes(body[start..start + 8].try_into().ok()?);
    }
    let (artifact, completion) = RetiredArtifact::from_action(*action, numbers[1], numbers[2])?;
    Some(RetirementRecord {
        artifact,
        completion,
        source_root: numbers[0],
        bytes: numbers[3],
    })
}

/// The newest record for each retired generation that has not reached completion.
pub(in crate::physical_runtime) fn unresolved_retirements(
    records: Vec<RetirementRecord>,
) -> Vec<RetirementRecord> {
    let mut latest = std::collections::BTreeMap::new();
    for record in records {
        latest.insert(record.artifact, record);
    }
    latest
        .into_values()
        .filter(|record| !record.completion)
        .collect()
}

/// Live reclamation keeps these spans until a completion is the newest record.
pub(in crate::physical_runtime) fn unresolved_retirement_holds(
    records: Vec<(RetirementRecord, u64, u64)>,
) -> Vec<(RetiredArtifact, u64, u64)> {
    let mut latest = std::collections::BTreeMap::new();
    for (record, start, end) in records {
        latest.insert(record.artifact, (record.completion, start, end));
    }
    latest
        .into_iter()
        .filter(|(_, (completion, _, _))| !*completion)
        .map(|(artifact, (_, start, end))| (artifact, start, end))
        .collect()
}

pub(in crate::physical_runtime::durability) fn note_retirement_hold(
    holds: &mut Vec<(RetiredArtifact, u64, u64)>,
    retirement: Option<(RetiredArtifact, bool)>,
    start: u64,
    end: u64,
) {
    let Some((artifact, completion)) = retirement else {
        return;
    };
    holds.retain(|hold| hold.0 != artifact);
    if !completion {
        holds.push((artifact, start, end));
    }
}

pub(in crate::physical_runtime) fn encode_retirement(
    artifact: RetiredArtifact,
    completion: bool,
    source_root: u64,
    bytes: u64,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + RETIREMENT_DOMAIN.len() + 1 + 32);
    encoded.extend_from_slice(&(RETIREMENT_DOMAIN.len() as u64).to_le_bytes());
    encoded.extend_from_slice(RETIREMENT_DOMAIN);
    encoded.push(artifact.action_code(completion));
    encoded.extend_from_slice(&source_root.to_le_bytes());
    encoded.extend_from_slice(&artifact.id().to_le_bytes());
    encoded.extend_from_slice(&artifact.generation().to_le_bytes());
    encoded.extend_from_slice(&bytes.to_le_bytes());
    encoded
}
