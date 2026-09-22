/// Exact generation a held retirement claim may delete. Only the admission
/// owner can issue one, and only while that generation is claimed.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct RetirementRemovalPermit {
    segment_id: u64,
    generation: u64,
}

impl RetirementRemovalPermit {
    pub(super) const fn issued(segment_id: u64, generation: u64) -> Self {
        Self {
            segment_id,
            generation,
        }
    }

    pub(in crate::physical_runtime) const fn segment_id(self) -> u64 {
        self.segment_id
    }

    pub(in crate::physical_runtime) const fn generation(self) -> u64 {
        self.generation
    }
}

pub(in crate::physical_runtime) const RETIREMENT_DOMAIN: &[u8] = b"store.physical.retirement.v1";

pub(in crate::physical_runtime) const RETIREMENT_INTENT: u8 = 1;
pub(in crate::physical_runtime) const RETIREMENT_COMPLETION: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRetirementDenial {
    Absent,
    Protected,
    Unresolved,
    Retained,
    WalPlan,
    WalWrite,
    WalSync,
    WalFinish,
    /// The scheduler has not started the frame. The claim stays charged.
    Waiting,
    Delete,
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
    pub(in crate::physical_runtime) action: u8,
    pub(in crate::physical_runtime) source_root: u64,
    pub(in crate::physical_runtime) segment_id: u64,
    pub(in crate::physical_runtime) generation: u64,
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
    Some(RetirementRecord {
        action: *action,
        source_root: numbers[0],
        segment_id: numbers[1],
        generation: numbers[2],
        bytes: numbers[3],
    })
}

/// The newest record for each segment generation that has not reached completion.
pub(in crate::physical_runtime) fn unresolved_retirements(
    records: Vec<RetirementRecord>,
) -> Vec<RetirementRecord> {
    let mut latest = std::collections::BTreeMap::new();
    for record in records {
        latest.insert((record.segment_id, record.generation), record);
    }
    latest
        .into_values()
        .filter(|record| record.action == RETIREMENT_INTENT)
        .collect()
}

/// Live reclamation keeps these spans until a completion is the newest record.
pub(in crate::physical_runtime) fn unresolved_retirement_holds(
    records: Vec<(RetirementRecord, u64, u64)>,
) -> Vec<(u64, u64, u64, u64)> {
    let mut latest = std::collections::BTreeMap::new();
    for (record, start, end) in records {
        latest.insert(
            (record.segment_id, record.generation),
            (record.action, start, end),
        );
    }
    latest
        .into_iter()
        .filter(|(_, (action, _, _))| *action == RETIREMENT_INTENT)
        .map(|((segment_id, generation), (_, start, end))| (segment_id, generation, start, end))
        .collect()
}

pub(in crate::physical_runtime::durability) fn note_retirement_hold(
    holds: &mut Vec<(u64, u64, u64, u64)>,
    retirement: Option<(u8, u64, u64)>,
    start: u64,
    end: u64,
) {
    let Some((action, segment_id, generation)) = retirement else {
        return;
    };
    holds.retain(|hold| hold.0 != segment_id || hold.1 != generation);
    if action == RETIREMENT_INTENT {
        holds.push((segment_id, generation, start, end));
    }
}

pub(in crate::physical_runtime) fn encode_retirement(
    action: u8,
    source_root: u64,
    segment_id: u64,
    generation: u64,
    bytes: u64,
) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + RETIREMENT_DOMAIN.len() + 1 + 32);
    encoded.extend_from_slice(&(RETIREMENT_DOMAIN.len() as u64).to_le_bytes());
    encoded.extend_from_slice(RETIREMENT_DOMAIN);
    encoded.push(action);
    encoded.extend_from_slice(&source_root.to_le_bytes());
    encoded.extend_from_slice(&segment_id.to_le_bytes());
    encoded.extend_from_slice(&generation.to_le_bytes());
    encoded.extend_from_slice(&bytes.to_le_bytes());
    encoded
}
