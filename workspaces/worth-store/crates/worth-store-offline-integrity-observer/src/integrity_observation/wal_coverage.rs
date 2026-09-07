//! Reachability facts from adjacent independently checksum-admitted WAL ranges.
use super::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIntegrityOutcome,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization,
    OfflinePhysicalFormatField,
};
use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
};

pub(crate) fn observe_missing_coverage(
    mut ranges: Vec<(u64, u64)>,
    walk: &mut BoundedMediaWalk,
) -> Vec<OfflineArtifactObservation> {
    ranges.sort_unstable();
    let mut observations = Vec::new();
    let mut covered_end: Option<u64> = None;
    for (start, end) in ranges {
        if let Some(prior) = covered_end {
            if start > prior {
                walk.counters_mut().missing_artifacts += 1;
                let outcome =
                    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
                        OfflinePhysicalDamageCause::MissingArtifact,
                        None,
                        Some(OfflinePhysicalFormatField::WalLsn),
                        OfflinePhysicalBlastRadius::Artifact,
                    ));
                walk.record_outcome(&outcome);
                observations.push(OfflineArtifactObservation::new(
                    "families/wal",
                    PhysicalArtifactFamily::WalFrame.into(),
                    PhysicalArtifactIdentity::new(format!("wal-required-lsn:{prior}:{start}"))
                        .unwrap(),
                    PhysicalArtifactGeneration::NotEncoded,
                    None,
                    outcome,
                ));
            }
        }
        covered_end = Some(covered_end.unwrap_or(end).max(end));
    }
    observations
}
