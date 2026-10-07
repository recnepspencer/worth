use super::denied;
use crate::filesystem_media::artifact_tree::publication_effect::{
    ArtifactTreePublicationEffect, ArtifactTreePublicationEffectOutcome,
    ScheduledArtifactTreePublicationEffectOutcome,
};
use crate::filesystem_media::artifact_tree::{
    ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile, ArtifactTreeMedia,
};
use crate::filesystem_media::artifact_tree_effects::begin_identified;
use crate::{
    filesystem_media::MediaOperationRole, BackendQueueExecutionAdaptation,
    BackendQueueExecutionPlanBinding,
};

impl ArtifactTreeMedia<'_> {
    pub fn synchronize_scheduled_file(
        &self,
        artifact: &ArtifactTreeFile,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        self.schedule_effect(binding, adaptation, |media| {
            media.synchronize_file_observed(artifact)
        })
    }

    pub fn synchronize_file_effect(
        &self,
        artifact: &ArtifactTreeFile,
    ) -> ArtifactTreePublicationEffectOutcome {
        self.synchronize_file_observed(artifact)
    }

    pub(in crate::filesystem_media::artifact_tree) fn synchronize_file_observed(
        &self,
        artifact: &ArtifactTreeFile,
    ) -> ArtifactTreePublicationEffectOutcome {
        let effect = ArtifactTreePublicationEffect::FileSynchronization(artifact.clone());
        let _coordination = match self
            .owner
            .begin_artifact_mutation(vec![artifact.coordination_key()])
        {
            Ok(coordination) => coordination,
            Err(_) => return denied(),
        };
        let directory = match self.open_directory(&artifact.directory) {
            Ok(directory) => directory,
            Err(failure) => {
                return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure);
            }
        };
        let file = match self.open_mutable_file(&directory, &artifact.file_name) {
            Ok(file) => file,
            Err(failure) => {
                return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure);
            }
        };
        let sequence_file = match file.try_clone().map(cap_std::fs::File::into_std) {
            Ok(file) => file,
            Err(error) => {
                return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error),
                );
            }
        };
        let sequence = match self.owner.mutation_sequence_for(&sequence_file) {
            Ok(sequence) => sequence,
            Err(error) => {
                return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                    ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error),
                );
            }
        };
        let _sequence = sequence.lock();
        let Some((operation, attempt)) =
            begin_identified(self.owner, MediaOperationRole::SynchronizeFileState, 0)
        else {
            return denied();
        };
        if let Some(error) = attempt
            .fail_before_error()
            .or_else(|| attempt.barrier_error())
        {
            attempt.denied();
            return ArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(
                ArtifactTreeFailure::io(ArtifactTreeFailureKind::DeniedBeforeEffect, &error),
            );
        }
        match crate::filesystem_media::os_synchronization::synchronize_state(&file) {
            Ok(()) if attempt.effect_observation_is_indeterminate() => {
                attempt.indeterminate(0);
                self.indeterminate(effect, operation, None)
            }
            Ok(()) => {
                self.owner.boundary().counters().file_sync();
                attempt.completed(0);
                self.completed(effect, operation)
            }
            Err(error) => {
                attempt.indeterminate(0);
                self.indeterminate(effect, operation, Some(&error))
            }
        }
    }
}
