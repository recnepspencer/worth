use super::*;

impl AdmittedRecoveryFilesystemMedia {
    /// A frame-sized addressed read; never allocates the containing arena.
    pub fn read_recovery_range_scheduled(
        &self,
        coordinate: RecordFrameCoordinate,
        maximum_bytes: u64,
        binding: BackendQueueExecutionPlanBinding,
    ) -> RecoveryReopenReadOutcome {
        let artifact = coordinate.artifact();
        if coordinate.length() == 0 || u64::from(coordinate.length()) > maximum_bytes {
            return denied_without_queue(artifact);
        }
        let physical = match super::super::discovery::record_artifact(artifact) {
            Ok(physical) => physical,
            Err(_) => return denied_without_queue(artifact),
        };
        let ticket = match crate::BackendQueueExecutionAuthority::store_owned().issue_ticket(
            binding,
            &self.parts.execution_capability,
            BackendQueueExecutionAdaptation::None,
        ) {
            Ok(ticket) => ticket,
            Err(_) => return denied_without_queue(artifact),
        };
        let mut bytes = vec![0; coordinate.length() as usize];
        let result = self
            .parts
            .artifact_tree()
            .read_exact_range(&physical, coordinate, &mut bytes);
        let queue = ticket.begin_completion().observe_queue_depth(1).complete();
        match result {
            ArtifactRangeReadOutcome::Completed(physical) => {
                RecoveryReopenReadOutcome::Completed(CompletedScheduledRecoveryReopenRead {
                    store: self.parts.store_identity,
                    artifact,
                    bytes: bytes.into_boxed_slice(),
                    physical,
                    queue,
                })
            }
            ArtifactRangeReadOutcome::DeniedBeforeEffect(failure) => {
                RecoveryReopenReadOutcome::Denied(DeniedScheduledRecoveryReopenRead {
                    artifact,
                    failure,
                    queue: Some(queue),
                })
            }
        }
    }
}
