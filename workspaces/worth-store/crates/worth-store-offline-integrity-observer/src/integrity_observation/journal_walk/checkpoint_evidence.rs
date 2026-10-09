use std::path::Path;

use crate::integrity_observation::{
    families::{
        checkpoint::CheckpointStreamObservation,
        durable_frame::read_u64,
        root_manifest::{read_root_manifest, OfflineRootManifestFacts},
    },
    record_walk::damage,
    BoundedMediaWalk, OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason as Unknown,
};

/// Evidence for the canonical current checkpoint, never a staging candidate.
#[derive(Clone)]
pub(crate) enum SelectedCheckpointEvidence {
    Absent,
    Validated {
        sequence: u64,
        #[cfg(test)]
        release_claim: Option<
            crate::integrity_observation::families::checkpoint::ObservedCheckpointReleaseClaim,
        >,
    },
    Unavailable(Outcome),
}

pub(super) fn validate(
    root: &Path,
    selected: Option<&OfflineRootManifestFacts>,
    stream: &CheckpointStreamObservation,
    walk: &mut BoundedMediaWalk,
) -> SelectedCheckpointEvidence {
    let result = validate_source_root(root, selected, stream, walk);
    match result {
        Ok((sequence, source_root_sha)) => {
            if stream
                .completed_release_claim
                .as_ref()
                .is_some_and(|claim| claim.source_root_sha() != source_root_sha)
            {
                return SelectedCheckpointEvidence::Unavailable(damage(
                    Cause::ScopeMismatch,
                    None,
                    Blast::Artifact,
                ));
            }
            SelectedCheckpointEvidence::Validated {
                sequence,
                #[cfg(test)]
                release_claim: stream.completed_release_claim.clone(),
            }
        }
        Err(outcome) => SelectedCheckpointEvidence::Unavailable(outcome),
    }
}

fn validate_source_root(
    root: &Path,
    selected: Option<&OfflineRootManifestFacts>,
    stream: &CheckpointStreamObservation,
    walk: &mut BoundedMediaWalk,
) -> Result<(u64, [u8; 32]), Outcome> {
    // A valid header alone is not a completed checkpoint. A footer failure or
    // bounded/truncated stream retains its original uncertainty or damage.
    let source = stream.completed_source.ok_or_else(|| {
        stream
            .records
            .iter()
            .find(|record| record.outcome != Outcome::Intact)
            .map(|record| record.outcome.clone())
            .unwrap_or_else(|| damage(Cause::MissingArtifact, None, Blast::Artifact))
    })?;
    let selected = selected.ok_or(Outcome::Unknown(Unknown::SelectorUnavailable))?;
    let cutoff = stream
        .completed_cutoff
        .ok_or_else(|| damage(Cause::Pointer, None, Blast::Artifact))?;
    if cutoff < source.wal_begin || cutoff > source.wal_end {
        return Err(damage(Cause::Pointer, None, Blast::Artifact));
    }
    if source.root_generation == 0 || source.root_generation > selected.generation {
        return Err(damage(Cause::Pointer, None, Blast::Artifact));
    }
    let path = root.join(format!(
        "families/records/roots/root-{:016x}.manifest",
        source.root_generation,
    ));
    let acquired = walk.acquire(&path, 4)?;
    if acquired.is_alias() {
        return Err(Outcome::Unknown(Unknown::PhysicalAliasNotReinspected));
    }
    let basis = read_root_manifest(&acquired.bytes, walk.counters_mut())?;
    if basis.generation != source.root_generation
        || basis.format != selected.format
        || read_u64(&basis.payload, 8) != source.tree_identity
    {
        return Err(damage(Cause::Pointer, None, Blast::Artifact));
    }
    Ok((
        source.sequence,
        crate::integrity_observation::sha256::sha256(&acquired.bytes),
    ))
}

/// Keep stream-local parser damage where it originated. A failed source-root
/// dependency instead marks the checkpoint's reference, not the root's offsets.
pub(super) fn localize_dependency_failure(
    stream: &mut CheckpointStreamObservation,
    evidence: &SelectedCheckpointEvidence,
) {
    if stream.completed_source.is_none() {
        return;
    }
    let SelectedCheckpointEvidence::Unavailable(outcome) = evidence else {
        return;
    };
    if let Some(header) = stream.records.first_mut() {
        header.outcome = match outcome {
            Outcome::Damaged(_) => damage(Cause::Pointer, Some((56, 16)), Blast::Artifact),
            Outcome::Unsupported(_) => Outcome::Unknown(Unknown::ParentScopeUnavailable),
            other => other.clone(),
        };
    }
}

#[cfg(test)]
mod tests;
