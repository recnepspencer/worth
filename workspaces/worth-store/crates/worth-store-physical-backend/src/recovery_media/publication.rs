use worth_store_physical_format::{RecordArtifactFile, RootSelectorRole};

use crate::filesystem_media::{
    ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeReplacement,
    ScheduledArtifactTreePublicationEffectOutcome,
};
use crate::{BackendQueueExecutionAdaptation, BackendQueueExecutionPlanBinding};

use super::AdmittedRecoveryFilesystemMedia;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryRootProtocolPublicationDenial {
    CatalogCandidateRequired,
}

/// Exact C4 inputs for the ordered previous/current/catalog replacement.
///
/// One catalog candidate determines the two selector candidates. Keeping the
/// publication identity in one validated value prevents recovery orchestration
/// from mixing candidates from different publication attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryRootProtocolPublicationPlan {
    publication: u64,
}

impl RecoveryRootProtocolPublicationPlan {
    pub const fn from_catalog_candidate(
        candidate: RecordArtifactFile,
    ) -> Result<Self, RecoveryRootProtocolPublicationDenial> {
        let RecordArtifactFile::CatalogCandidate { publication } = candidate else {
            return Err(RecoveryRootProtocolPublicationDenial::CatalogCandidateRequired);
        };
        Ok(Self { publication })
    }

    pub const fn publication(self) -> u64 {
        self.publication
    }

    pub const fn previous_candidate(self) -> RecordArtifactFile {
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Previous,
            publication: self.publication,
        }
    }

    pub const fn current_candidate(self) -> RecordArtifactFile {
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Current,
            publication: self.publication,
        }
    }

    pub const fn catalog_candidate(self) -> RecordArtifactFile {
        RecordArtifactFile::CatalogCandidate {
            publication: self.publication,
        }
    }
}

impl AdmittedRecoveryFilesystemMedia {
    /// Executes the exact ordered root-protocol replacement through the
    /// backend queue. The returned C4 receipt remains the effect authority;
    /// recovery orchestration must not replace it with attempted counters.
    pub fn replace_recovery_root_protocol_scheduled(
        &self,
        plan: RecoveryRootProtocolPublicationPlan,
        binding: BackendQueueExecutionPlanBinding,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        let previous = replacement(
            plan.previous_candidate(),
            RecordArtifactFile::PreviousRootSelector,
        );
        let current = replacement(
            plan.current_candidate(),
            RecordArtifactFile::CurrentRootSelector,
        );
        let catalog = replacement(
            plan.catalog_candidate(),
            RecordArtifactFile::BootstrapCatalog,
        );
        let (Ok(previous), Ok(current), Ok(catalog)) = (previous, current, catalog) else {
            return ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                structural_denial(),
            );
        };
        self.parts.artifact_tree().replace_root_protocol_scheduled(
            previous,
            current,
            catalog,
            binding,
            BackendQueueExecutionAdaptation::None,
        )
    }

    /// Reads one root-protocol artifact (the catalog, a selector or a root
    /// manifest), at most `maximum_bytes`. `None` when it is absent.
    pub fn read_recovery_root_protocol_artifact(
        &self,
        artifact: RecordArtifactFile,
        maximum_bytes: u64,
    ) -> Result<Option<Box<[u8]>>, ArtifactTreeFailure> {
        if !matches!(
            artifact,
            RecordArtifactFile::BootstrapCatalog
                | RecordArtifactFile::CurrentRootSelector
                | RecordArtifactFile::PreviousRootSelector
                | RecordArtifactFile::RootManifest { .. }
        ) {
            return Err(structural_denial());
        }
        let file = super::discovery::record_artifact(artifact).map_err(|_| structural_denial())?;
        match self
            .parts
            .artifact_tree()
            .read_bounded(&file, maximum_bytes)
        {
            Ok(bytes) => Ok(Some(bytes.into())),
            Err(failure)
                if failure.kind() == crate::filesystem_media::ArtifactTreeFailureKind::Absent =>
            {
                Ok(None)
            }
            Err(failure) => Err(failure),
        }
    }

    /// Makes the completed root-protocol replacement namespace durable. The
    /// records directory is the common parent of all three destinations.
    pub fn synchronize_recovery_record_namespace_scheduled(
        &self,
        binding: BackendQueueExecutionPlanBinding,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        let Ok(records) = ArtifactTreeDirectory::families().child("records") else {
            return ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                structural_denial(),
            );
        };
        self.parts.artifact_tree().synchronize_scheduled_directory(
            &records,
            binding,
            BackendQueueExecutionAdaptation::None,
        )
    }
}

fn replacement(
    source: RecordArtifactFile,
    destination: RecordArtifactFile,
) -> Result<ArtifactTreeReplacement, super::RecoveryDiscoveryFailure> {
    Ok(ArtifactTreeReplacement::new(
        super::discovery::record_artifact(source)?,
        super::discovery::record_artifact(destination)?,
    ))
}

fn structural_denial() -> ArtifactTreeFailure {
    ArtifactTreeFailure::recovery_denial()
}
