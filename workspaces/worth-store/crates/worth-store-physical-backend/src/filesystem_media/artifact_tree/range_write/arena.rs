use super::*;
use worth_store_physical_format::RecordArtifactFile;

impl ArtifactTreeMedia<'_> {
    /// Writes an admitted arena range, preserving all bytes outside it. The
    /// arena must already exist; its first frame uses the new-artifact effect.
    pub fn write_arena_range_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ArtifactRangeWriteOutcome {
        let Some(request) = arena_request(artifact, coordinate, bytes, durability) else {
            return ArtifactRangeWriteOutcome::DeniedBeforeEffect(ArtifactTreeFailure::structural(
                ArtifactTreeFailureKind::AccessLimitExceeded,
            ));
        };
        self.write_exact(request)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn write_arena_scheduled_foreground_exact_at(
        &self,
        artifact: &ArtifactTreeFile,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        let Some(request) = arena_request(artifact, coordinate, bytes, durability) else {
            return ScheduledArtifactRangeWriteOutcome::DeniedBeforeEffect(
                ArtifactTreeFailure::structural(ArtifactTreeFailureKind::AccessLimitExceeded),
            );
        };
        self.write_scheduled(
            request,
            ScheduledArtifactRangeWriteContext {
                binding,
                adaptation,
                writeback_scope: None,
            },
        )
    }
}

fn arena_request<'a>(
    artifact: &'a ArtifactTreeFile,
    coordinate: RecordFrameCoordinate,
    bytes: &'a [u8],
    durability: ArtifactRangeWriteDurabilityRequirement,
) -> Option<ArtifactRangeWriteRequest<'a>> {
    matches!(
        coordinate.artifact(),
        RecordArtifactFile::ExtentArena { .. }
    )
    .then_some(ArtifactRangeWriteRequest {
        artifact,
        coordinate,
        bytes,
        durability,
        posture: ArtifactRangeWritePosture::ArenaRange,
    })
}
