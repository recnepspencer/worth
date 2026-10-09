use super::super::{
    durable_frame::read_u64,
    physical_fields::{scope, shape},
};
use super::{
    certificates::{CertificateState, ObservedCheckpointReleaseClaim},
    record::{family, read_record},
    source::{read_dirty, read_source, CheckpointSourceFacts},
};
use crate::integrity_observation::{
    record_walk::{damage, shift_outcome},
    sha256::Sha256,
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalBlastRadius as Blast, OfflinePhysicalDamageCause as Cause,
    OfflinePhysicalFormatField as Field,
};
use worth_foundational::PhysicalArtifactFamily;

pub(crate) struct CheckpointRecordObservation {
    pub(crate) family: PhysicalArtifactFamily,
    pub(crate) kind: u8,
    pub(crate) sequence: Option<u64>,
    pub(crate) offset: u64,
    pub(crate) length: u64,
    pub(crate) outcome: Outcome,
}

struct StreamState {
    schema: u8,
    source: Option<CheckpointSourceFacts>,
    dirty: Sha256,
    bindings: Sha256,
    dirty_count: u64,
    binding_count: u64,
    binding_bytes: u64,
    compaction: Option<(u64, u64, u64)>,
    certificates: CertificateState,
    certificates_started: bool,
}

pub(crate) struct CheckpointStreamObservation {
    pub(crate) records: Vec<CheckpointRecordObservation>,
    /// Present only after the complete stream and its footer aggregates pass.
    pub(crate) completed_source: Option<CheckpointSourceFacts>,
    pub(crate) completed_cutoff: Option<u64>,
    pub(crate) completed_release_claim: Option<ObservedCheckpointReleaseClaim>,
}

pub(crate) fn read_checkpoint(
    bytes: &[u8],
    store: [u8; 16],
    sequence: Option<u64>,
    maximum_records: u64,
    counters: &mut OfflineIntegrityObservationCounters,
) -> CheckpointStreamObservation {
    let mut state = StreamState {
        schema: 0,
        source: None,
        dirty: Sha256::new(),
        bindings: Sha256::new(),
        dirty_count: 0,
        binding_count: 0,
        binding_bytes: 0,
        compaction: None,
        certificates: CertificateState::new(),
        certificates_started: false,
    };
    let mut observations = Vec::new();
    let mut completed_source = None;
    let mut completed_cutoff = None;
    let mut completed_release_claim = None;
    let mut offset = 0;
    let mut stage = 1;
    loop {
        let kind = match stage {
            1 => 1,
            2 if bytes.get(offset + 9) == Some(&3) => 3,
            2 => 2,
            4 if bytes.get(offset + 9) == Some(&5) => 5,
            4 if state.schema == 3 && bytes.get(offset + 9) == Some(&6) => 6,
            4 if state.schema == 3 && bytes.get(offset + 9) == Some(&7) => 7,
            _ => 4,
        };
        let remaining = &bytes[offset..];
        if observations.len() as u64 >= maximum_records {
            observations.push(observation(
                kind,
                state.source.as_ref().map(|s| s.sequence),
                offset,
                0,
                Outcome::Indeterminate(
                    crate::OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
                ),
            ));
            break;
        }
        let parsed = read_record(
            remaining,
            kind,
            (stage != 1).then_some(state.schema),
            counters,
        );
        let (payload, length) = match parsed {
            Ok(parsed) => parsed,
            Err(outcome) => {
                observations.push(observation(
                    kind,
                    state.source.as_ref().map(|s| s.sequence),
                    offset,
                    remaining.len(),
                    shift_outcome(outcome, offset as u64),
                ));
                break;
            }
        };
        let result = match kind {
            1 => read_source(payload, store, sequence).map(|source| {
                state.schema = remaining[8];
                state.source = Some(source);
                stage = 2;
            }),
            2 => read_dirty(payload).map(|()| {
                state.dirty.update(&remaining[..length]);
                state.dirty_count += 1;
            }),
            3 => {
                let generation = read_u64(payload, 0);
                let cutoff = read_u64(payload, 8);
                scope(
                    generation != 0
                        && cutoff != 0
                        && state.source.as_ref().is_some_and(|s| cutoff <= s.wal_end),
                    16,
                    16,
                    Field::IdentityField,
                )
                .map(|()| {
                    state.compaction = Some((offset as u64, generation, cutoff));
                    stage = 4;
                })
            }
            4 => {
                if state.certificates_started {
                    observations.push(observation(
                        kind,
                        state.source.as_ref().map(|s| s.sequence),
                        offset,
                        length,
                        damage(Cause::ScopeMismatch, None, Blast::Artifact),
                    ));
                    break;
                }
                state.bindings.update(&remaining[..length]);
                state.binding_count += 1;
                state.binding_bytes += length as u64;
                Ok(())
            }
            6 | 7 => {
                state.certificates_started = true;
                state.certificates.include(
                    kind,
                    payload,
                    &remaining[..length],
                    state.source.expect("header admitted before certificate"),
                )
            }
            5 => {
                let sequence = state.source.as_ref().map(|s| s.sequence);
                let source = state.source;
                let cutoff = state.compaction.map(|value| value.2);
                let result = finish(state, payload, offset + length == bytes.len(), counters);
                if let Ok(release) = &result {
                    completed_source = source;
                    completed_cutoff = cutoff;
                    completed_release_claim = release.clone();
                }
                observations.push(observation(
                    kind,
                    sequence,
                    offset,
                    length,
                    result.map_or_else(
                        |outcome| shift_outcome(outcome, offset as u64),
                        |_| Outcome::Intact,
                    ),
                ));
                break;
            }
            _ => unreachable!(),
        };
        let outcome = result.map_or_else(
            |outcome| shift_outcome(outcome, offset as u64),
            |()| Outcome::Intact,
        );
        let rejected = outcome != Outcome::Intact;
        observations.push(observation(
            kind,
            state.source.as_ref().map(|s| s.sequence),
            offset,
            length,
            outcome,
        ));
        if rejected {
            break;
        }
        offset += length;
        if offset > bytes.len() {
            break;
        }
    }
    CheckpointStreamObservation {
        records: observations,
        completed_source,
        completed_cutoff,
        completed_release_claim,
    }
}

fn finish(
    state: StreamState,
    payload: &[u8],
    terminal: bool,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<Option<ObservedCheckpointReleaseClaim>, Outcome> {
    shape(
        terminal && payload.len() == if state.schema == 3 { 184 } else { 136 },
        0,
        156,
    )?;
    let source = state.source.expect("header admitted before footer");
    scope(
        payload[..24] == source.identity,
        16,
        24,
        Field::IdentityField,
    )?;
    scope(
        read_u64(payload, 24) == state.dirty_count
            && read_u64(payload, 88) == state.binding_count
            && read_u64(payload, 96) == state.binding_bytes,
        40,
        80,
        Field::RecordLength,
    )?;
    let compaction = state.compaction.expect("compaction admitted before footer");
    scope(
        (
            read_u64(payload, 64),
            read_u64(payload, 72),
            read_u64(payload, 80),
        ) == compaction,
        80,
        24,
        Field::IdentityField,
    )?;
    counters.checksum_calculations += 2;
    if state.dirty.finish() != payload[32..64] {
        return Err(damage(
            Cause::ChecksumMismatch,
            Some((48, 32)),
            Blast::Artifact,
        ));
    }
    if state.bindings.finish() != payload[104..136] {
        return Err(damage(
            Cause::ChecksumMismatch,
            Some((120, 32)),
            Blast::Artifact,
        ));
    }
    if state.schema == 3 {
        state.certificates.finish(payload, counters)
    } else {
        Ok(None)
    }
}

fn observation(
    kind: u8,
    sequence: Option<u64>,
    offset: usize,
    length: usize,
    outcome: Outcome,
) -> CheckpointRecordObservation {
    CheckpointRecordObservation {
        family: family(kind),
        kind,
        sequence,
        offset: offset as u64,
        length: length as u64,
        outcome,
    }
}
