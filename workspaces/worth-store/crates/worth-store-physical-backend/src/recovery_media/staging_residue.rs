//! Stale root-protocol candidates a killed runtime left in record staging.
//!
//! A catalog or selector candidate becomes authority only by the ordered
//! replacement that consumes it. After recovery has published its own root,
//! any candidate still in record staging belongs to an attempt that never
//! replaced anything, so it is removable. Any other staging entry is damage,
//! not residue, and is never removed here.

use worth_store_physical_format::{RecordArtifactFile, RootSelectorRole};

use crate::filesystem_media::{
    ArtifactTreeDirectory, ArtifactTreeFailure, ScheduledArtifactTreePublicationEffectOutcome,
};
use crate::{BackendQueueExecutionAdaptation, BackendQueueExecutionPlanBinding};

use super::AdmittedRecoveryFilesystemMedia;

impl AdmittedRecoveryFilesystemMedia {
    /// Lists record-staging candidates, at most `limit` of them. A name that is
    /// not a root-protocol candidate refuses as damaged evidence.
    pub fn list_recovery_record_staging_candidates(
        &self,
        limit: usize,
    ) -> Result<Box<[RecordArtifactFile]>, ArtifactTreeFailure> {
        let directory = record_staging()?;
        let tree = self.parts.artifact_tree();
        if !tree.directory_exists(&directory)? {
            return Ok(Box::new([]));
        }
        tree.list_file_names_bounded(&directory, limit)?
            .iter()
            .map(|name| parse_candidate(name).ok_or_else(ArtifactTreeFailure::recovery_damaged))
            .collect()
    }

    /// Durably removes one stale root-protocol candidate. An already absent
    /// candidate completes, so an interrupted removal can be repeated.
    pub fn remove_recovery_staging_candidate_scheduled(
        &self,
        candidate: RecordArtifactFile,
        binding: BackendQueueExecutionPlanBinding,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        let artifact = match candidate {
            RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::RootSelectorCandidate { .. } => {
                super::discovery::record_artifact(candidate)
            }
            _ => {
                return ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                    ArtifactTreeFailure::recovery_denial(),
                )
            }
        };
        let Ok(artifact) = artifact else {
            return ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                ArtifactTreeFailure::recovery_denial(),
            );
        };
        self.parts.artifact_tree().remove_scheduled_file_durably(
            &artifact,
            binding,
            BackendQueueExecutionAdaptation::None,
        )
    }
}

fn record_staging() -> Result<ArtifactTreeDirectory, ArtifactTreeFailure> {
    ArtifactTreeDirectory::staging()
        .child("records")
        .map_err(|_| ArtifactTreeFailure::recovery_denial())
}

/// The exact inverse of the candidates' canonical file names.
fn parse_candidate(name: &str) -> Option<RecordArtifactFile> {
    let candidate = if let Some(rest) = name.strip_prefix("bootstrap-") {
        RecordArtifactFile::CatalogCandidate {
            publication: publication(rest)?,
        }
    } else if let Some(rest) = name.strip_prefix("root-current-") {
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Current,
            publication: publication(rest)?,
        }
    } else if let Some(rest) = name.strip_prefix("root-previous-") {
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Previous,
            publication: publication(rest)?,
        }
    } else {
        return None;
    };
    (candidate.file_name() == name).then_some(candidate)
}

fn publication(rest: &str) -> Option<u64> {
    let digits = rest.strip_suffix(".candidate")?;
    (digits.len() == 16)
        .then(|| u64::from_str_radix(digits, 16).ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_canonical_candidate_names_parse() {
        for candidate in [
            RecordArtifactFile::CatalogCandidate { publication: 7 },
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Current,
                publication: u64::MAX,
            },
            RecordArtifactFile::RootSelectorCandidate {
                role: RootSelectorRole::Previous,
                publication: 1,
            },
        ] {
            assert_eq!(parse_candidate(&candidate.file_name()), Some(candidate));
        }
        for name in [
            "bootstrap.catalog",
            "bootstrap-7.candidate",
            "bootstrap-000000000000000G.candidate",
            "bootstrap-0000000000000007.candidate.tmp",
            "root-0000000000000002.manifest",
            "checkpoint-0000000000000002.candidate",
        ] {
            assert_eq!(parse_candidate(name), None, "{name}");
        }
    }
}
