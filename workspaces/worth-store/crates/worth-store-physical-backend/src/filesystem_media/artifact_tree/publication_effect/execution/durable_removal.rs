use super::denied;
use crate::filesystem_media::artifact_tree::publication_effect::{
    ArtifactTreePublicationEffect, ArtifactTreePublicationEffectOutcome,
    IndeterminateArtifactTreePublicationEffect, ScheduledArtifactTreePublicationEffectOutcome,
};
use crate::filesystem_media::artifact_tree::{
    ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile, ArtifactTreeMedia,
};
use crate::filesystem_media::artifact_tree_effects::{begin_identified, synchronize_directory};
use crate::{
    filesystem_media::MediaOperationRole, BackendQueueExecutionAdaptation,
    BackendQueueExecutionPlanBinding,
};

impl ArtifactTreeMedia<'_> {
    pub fn remove_scheduled_file_durably(
        &self,
        artifact: &ArtifactTreeFile,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        self.schedule_effect(binding, adaptation, |media| {
            media.remove_file_durably_observed(artifact)
        })
    }

    pub(crate) fn remove_file_durably_observed(
        &self,
        artifact: &ArtifactTreeFile,
    ) -> ArtifactTreePublicationEffectOutcome {
        let effect = ArtifactTreePublicationEffect::DurableRemoval(artifact.clone());
        let _coordination = match self
            .owner
            .begin_artifact_namespace_mutation(vec![artifact.coordination_key()])
        {
            Ok(coordination) => coordination,
            Err(_) => return denied(),
        };
        let directory = match self.open_directory(&artifact.directory) {
            Ok(directory) => directory,
            Err(failure) => {
                return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure)
            }
        };
        let Some((operation, attempt)) =
            begin_identified(self.owner, MediaOperationRole::Delete, 0)
        else {
            return denied();
        };
        if let Some(error) = attempt.fail_before_error() {
            attempt.denied();
            return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error),
            );
        }
        match directory.remove_file(&artifact.file_name) {
            Ok(()) if attempt.effect_observation_is_indeterminate() => {
                attempt.indeterminate(0);
                self.indeterminate(effect, operation, None)
            }
            Ok(()) => {
                self.owner.boundary().counters().deletion();
                attempt.completed(0);
                self.directory_sync_after_unlink(effect, operation, &directory)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                attempt.completed(0);
                self.directory_sync_after_unlink(effect, operation, &directory)
            }
            Err(error) => {
                attempt.indeterminate(0);
                self.indeterminate(effect, operation, Some(&error))
            }
        }
    }

    fn directory_sync_after_unlink(
        &self,
        effect: ArtifactTreePublicationEffect,
        operation: crate::filesystem_media::MediaOperationIdentity,
        directory: &cap_std::fs::Dir,
    ) -> ArtifactTreePublicationEffectOutcome {
        #[cfg(feature = "certification-test-authority")]
        if self.owner.take_removal_directory_sync_failure() {
            return self.indeterminate(effect, operation, None);
        }
        match synchronize_directory(self.owner, directory) {
            Ok(()) => self.completed(effect, operation),
            Err(failure) => ArtifactTreePublicationEffectOutcome::Indeterminate(
                IndeterminateArtifactTreePublicationEffect {
                    failure,
                    owner: self.owner.identity(),
                    store: self.store,
                    operation,
                    effect,
                },
            ),
        }
    }
}
