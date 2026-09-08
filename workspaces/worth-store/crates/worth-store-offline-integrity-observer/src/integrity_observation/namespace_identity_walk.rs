use std::path::Path;

use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity, PhysicalByteRange,
};

use super::families::read_namespace_identity;
use super::unknown_artifact::{relative_path, unknown_artifact};
use super::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIndeterminatePhysicalReason,
    OfflineIntegrityOutcome, OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause,
    OfflinePhysicalDamageLocalization,
};

const NAMESPACE_RELATIVE: &str = "namespace";
const IDENTITY_NAME: &str = "identity";

pub(crate) struct NamespaceIdentityWalk {
    pub(crate) expected_store_identity: Option<[u8; 16]>,
    pub(crate) observations: Vec<OfflineArtifactObservation>,
}

pub(crate) fn observe_namespace_identity(
    store_root: &Path,
    walk: &mut BoundedMediaWalk,
) -> Result<NamespaceIdentityWalk, ()> {
    let directory = store_root.join(NAMESPACE_RELATIVE);
    let scan = walk.scan_directory(&directory, 1).map_err(|_| ())?;
    let identity_path = scan
        .entries
        .iter()
        .find(|path| path.file_name().is_some_and(|name| name == IDENTITY_NAME));
    let (expected_store_identity, identity_observation) = match identity_path {
        Some(path) => observe_identity_file(store_root, path, walk),
        None => missing_identity(scan.incomplete_reason, walk),
    };
    let mut observations = vec![identity_observation];
    observations.extend(unknown_entries(store_root, &scan.entries, walk));
    observations.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
    Ok(NamespaceIdentityWalk {
        expected_store_identity,
        observations,
    })
}

fn observe_identity_file(
    store_root: &Path,
    path: &Path,
    walk: &mut BoundedMediaWalk,
) -> (Option<[u8; 16]>, OfflineArtifactObservation) {
    let relative = relative_path(store_root, path);
    match walk.acquire(path, 2) {
        Ok(acquired) if acquired.is_alias() => (
            None,
            observation(relative, "namespace-identity", duplicate_damage()),
        ),
        Ok(acquired) => match read_namespace_identity(&acquired.bytes, walk.counters_mut()) {
            Ok(facts) => (
                Some(facts.store_identity),
                observation(
                    relative,
                    &hex_bytes(&facts.store_identity),
                    OfflineIntegrityOutcome::Intact,
                ),
            ),
            Err(outcome) => {
                walk.record_outcome(&outcome);
                (None, observation(relative, "namespace-identity", outcome))
            }
        },
        Err(outcome) => {
            walk.record_outcome(&outcome);
            (None, observation(relative, "namespace-identity", outcome))
        }
    }
}

fn missing_identity(
    incomplete: Option<OfflineIndeterminatePhysicalReason>,
    walk: &mut BoundedMediaWalk,
) -> (Option<[u8; 16]>, OfflineArtifactObservation) {
    let outcome = match incomplete {
        Some(reason) => OfflineIntegrityOutcome::Indeterminate(reason),
        None => {
            walk.counters_mut().missing_artifacts += 1;
            OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
                OfflinePhysicalDamageCause::MissingArtifact,
                None,
                None,
                OfflinePhysicalBlastRadius::Artifact,
            ))
        }
    };
    (
        None,
        observation("namespace/identity".into(), "namespace-identity", outcome),
    )
}

fn unknown_entries(
    store_root: &Path,
    entries: &[std::path::PathBuf],
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    entries
        .iter()
        .filter(|path| path.file_name().is_none_or(|name| name != IDENTITY_NAME))
        .map(|path| unknown_artifact(store_root, path, 2, walk))
        .collect()
}

fn observation(
    relative: String,
    identity: &str,
    outcome: OfflineIntegrityOutcome,
) -> OfflineArtifactObservation {
    OfflineArtifactObservation::new(
        relative,
        PhysicalArtifactFamily::NamespaceIdentity.into(),
        PhysicalArtifactIdentity::new(identity).expect("namespace identity label"),
        PhysicalArtifactGeneration::NotEncoded,
        Some(PhysicalByteRange::new(0, 72).expect("fixed namespace identity declaration")),
        outcome,
    )
}

fn duplicate_damage() -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
        OfflinePhysicalDamageCause::DuplicateIdentity,
        None,
        None,
        OfflinePhysicalBlastRadius::Artifact,
    ))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}
