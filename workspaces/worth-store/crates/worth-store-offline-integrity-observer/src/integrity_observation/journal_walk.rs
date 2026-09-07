use super::{
    families::{
        checkpoint::read_checkpoint, physical_work::read_physical_work, wal::read_wal_segment,
    },
    unknown_artifact::{relative_path, unknown_artifact},
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIntegrityOutcome as Outcome,
    OfflineUnknownPhysicalReason,
};
use std::path::Path;
use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};

pub(crate) fn observe_journals(
    root: &Path,
    store: Option<[u8; 16]>,
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    let mut observations = Vec::new();
    if walk.exhausted_reason().is_some() {
        return observations;
    }
    let pending = root.join("families/physical-work");
    if let Ok(scan) = walk.scan_directory(&pending, 2) {
        for path in scan.entries {
            let Some(identity) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(pending_identity)
            else {
                observations.push(unknown_artifact(root, &path, 3, walk));
                continue;
            };
            let relative = relative_path(root, &path);
            let mut length = 0;
            let outcome = match walk.acquire(&path, 3) {
                Err(outcome) => outcome,
                Ok(acquired) => {
                    length = acquired.byte_length;
                    match store {
                        Some(store) => read_physical_work(
                            &acquired.bytes,
                            store,
                            identity,
                            walk.counters_mut(),
                        )
                        .map_or_else(|outcome| outcome, |()| Outcome::Intact),
                        None => {
                            Outcome::Unknown(OfflineUnknownPhysicalReason::StoreIdentityUnavailable)
                        }
                    }
                }
            };
            walk.record_outcome(&outcome);
            observations.push(project(
                &relative,
                Family::PhysicalWorkObligation,
                format!(
                    "operation:{:016x}:{:016x}:{:016x}",
                    identity.0, identity.1, identity.2
                ),
                Some(identity.1),
                0,
                length as u64,
                outcome,
            ));
        }
    }
    let wal = root.join("families/wal");
    if let Ok(scan) = walk.scan_directory(&wal, 2) {
        for path in scan.entries {
            let Some((segment, generation)) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(wal_identity)
            else {
                observations.push(unknown_artifact(root, &path, 3, walk));
                continue;
            };
            let relative = relative_path(root, &path);
            match walk.acquire(&path, 3) {
                Err(outcome) => {
                    walk.record_outcome(&outcome);
                    observations.push(project(
                        &relative,
                        Family::WalFrame,
                        format!("wal:{segment}:{generation}:0"),
                        Some(generation),
                        0,
                        0,
                        outcome,
                    ));
                }
                Ok(acquired) => {
                    let maximum = walk.maximum_entries();
                    let frames = read_wal_segment(
                        &acquired.bytes,
                        segment,
                        generation,
                        maximum,
                        walk.counters_mut(),
                    );
                    for frame in frames {
                        walk.record_outcome(&frame.outcome);
                        observations.push(project(
                            &relative,
                            Family::WalFrame,
                            format!("wal:{segment}:{generation}:{}", frame.offset),
                            Some(generation),
                            frame.offset,
                            frame.length,
                            frame.outcome,
                        ));
                    }
                }
            }
        }
    }
    let checkpoint = root.join("families/checkpoint.current");
    if checkpoint.try_exists().unwrap_or(true) {
        observe_checkpoint(root, &checkpoint, store, None, walk, &mut observations);
    }
    let staging = root.join("staging");
    if let Ok(scan) = walk.scan_directory(&staging, 1) {
        for path in scan.entries {
            let sequence = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_prefix("checkpoint-")?.strip_suffix(".candidate"))
                .and_then(hex_word);
            if let Some(sequence) = sequence {
                observe_checkpoint(root, &path, store, Some(sequence), walk, &mut observations);
            }
        }
    }
    observations
}

fn observe_checkpoint(
    root: &Path,
    path: &Path,
    store: Option<[u8; 16]>,
    sequence: Option<u64>,
    walk: &mut BoundedMediaWalk,
    observations: &mut Vec<OfflineArtifactObservation>,
) {
    let relative = relative_path(root, path);
    let acquired = walk.acquire(path, 2);
    match (acquired, store) {
        (Err(outcome), _) => {
            walk.record_outcome(&outcome);
            observations.push(project(
                &relative,
                Family::CheckpointStreamHeader,
                "checkpoint-header".into(),
                sequence,
                0,
                0,
                outcome,
            ));
        }
        (Ok(acquired), None) => observations.push(project(
            &relative,
            Family::CheckpointStreamHeader,
            "checkpoint-header".into(),
            sequence,
            0,
            acquired.byte_length as u64,
            Outcome::Unknown(OfflineUnknownPhysicalReason::StoreIdentityUnavailable),
        )),
        (Ok(acquired), Some(store)) => {
            let maximum = walk.maximum_entries();
            for record in read_checkpoint(
                &acquired.bytes,
                store,
                sequence,
                maximum,
                walk.counters_mut(),
            ) {
                walk.record_outcome(&record.outcome);
                observations.push(project(
                    &relative,
                    record.family,
                    format!(
                        "checkpoint:{}:{}:{}",
                        record.sequence.unwrap_or(0),
                        record.kind,
                        record.offset
                    ),
                    record.sequence,
                    record.offset,
                    record.length,
                    record.outcome,
                ));
            }
        }
    }
}

fn pending_identity(name: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<_> = name
        .strip_prefix("effect-")?
        .strip_suffix(".pending")?
        .split('-')
        .collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        hex_word(parts[0])?,
        hex_word(parts[1])?,
        hex_word(parts[2])?,
    ))
}
fn hex_word(word: &str) -> Option<u64> {
    let value = u64::from_str_radix(word, 16).ok()?;
    (value != 0 && format!("{value:016x}") == word).then_some(value)
}
fn wal_identity(name: &str) -> Option<(u64, u64)> {
    let (segment, generation) = name
        .strip_prefix("segment-")?
        .strip_suffix(".wal")?
        .split_once("-generation-")?;
    let segment: u64 = segment.parse().ok()?;
    let generation: u64 = generation.parse().ok()?;
    (segment != 0
        && generation != 0
        && format!("segment-{segment}-generation-{generation}.wal") == name)
        .then_some((segment, generation))
}
fn project(
    path: &str,
    family: Family,
    identity: String,
    generation: Option<u64>,
    offset: u64,
    length: u64,
    outcome: Outcome,
) -> OfflineArtifactObservation {
    OfflineArtifactObservation::new(
        path,
        family.into(),
        PhysicalArtifactIdentity::new(identity).unwrap(),
        generation
            .and_then(PhysicalArtifactGeneration::encoded)
            .unwrap_or(PhysicalArtifactGeneration::NotEncoded),
        PhysicalByteRange::new(offset, length).ok(),
        outcome,
    )
}
