use std::path::Path;

use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalByteRange,
};

use super::super::families::bootstrap_catalog::read_bootstrap_catalog;
use super::super::{
    BoundedMediaWalk, OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason,
};
use super::damage;

pub(super) fn observe_bootstrap(
    root: &Path,
    store: Option<[u8; 16]>,
    walk: &mut BoundedMediaWalk,
) -> OfflineArtifactObservation {
    let relative = "families/records/bootstrap.catalog";
    let path = root.join(relative);
    let mut alias = None;
    let outcome = if let Some(reason) = walk.exhausted_reason() {
        Outcome::Indeterminate(reason)
    } else if !path.try_exists().unwrap_or(true) {
        walk.counters_mut().missing_artifacts += 1;
        damage(Cause::MissingArtifact, None, Blast::Artifact)
    } else {
        match walk.acquire(&path, 3) {
            Err(outcome) => outcome,
            Ok(acquired) => {
                alias = acquired
                    .physical_alias_of
                    .as_ref()
                    .map(|path| super::super::unknown_artifact::relative_path(root, path));
                if alias.is_some() {
                    Outcome::Unknown(OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected)
                } else {
                    match store {
                        None => {
                            Outcome::Unknown(OfflineUnknownPhysicalReason::StoreIdentityUnavailable)
                        }
                        Some(store) => {
                            read_bootstrap_catalog(&acquired.bytes, store, walk.counters_mut())
                                .map_or_else(|outcome| outcome, |()| Outcome::Intact)
                        }
                    }
                }
            }
        }
    };
    walk.record_outcome(&outcome);
    let observation = OfflineArtifactObservation::new(
        relative,
        Family::BootstrapCatalog.into(),
        PhysicalArtifactIdentity::new("bootstrap-catalog").unwrap(),
        PhysicalArtifactGeneration::NotEncoded,
        PhysicalByteRange::new(0, 82).ok(),
        outcome,
    );
    match alias {
        Some(first_path) => {
            observation.with_duplicate(OfflineArtifactDuplicateEvidence::PhysicalAlias {
                first_path: first_path.into(),
            })
        }
        None => observation,
    }
}
