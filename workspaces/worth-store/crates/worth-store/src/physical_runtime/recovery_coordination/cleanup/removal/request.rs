use super::PhysicalRecoveryCleanupRemovalCommand;

pub(super) fn from_command(
    command: &PhysicalRecoveryCleanupRemovalCommand,
    admission: [u8; 32],
) -> Option<worth_store_physical_backend::BackendRecoveryCleanupRemovalRequest> {
    let checkpoint = worth_store_physical_backend::ArtifactTreeDirectory::families()
        .file("checkpoint.current")
        .ok()?;
    let artifact = worth_store_physical_backend::ArtifactTreeDirectory::families()
        .child("wal")
        .ok()?
        .file(&format!(
            "segment-{}-generation-{}.wal",
            command.artifact.segment().get(),
            command.artifact.generation().get()
        ))
        .ok()?;
    let inspection = command.admitted_wal.inspection();
    let checkpoint = worth_store_physical_backend::BackendRecoveryArtifactExpectation::new(
        checkpoint,
        command.checkpoint_stream.encoded_bytes(),
        command.checkpoint_stream.encoded_digest(),
    )?;
    let artifact = worth_store_physical_backend::BackendRecoveryArtifactExpectation::new(
        artifact,
        inspection.byte_count(),
        inspection.artifact_digest(),
    )?;
    worth_store_physical_backend::BackendRecoveryCleanupRemovalRequest::new(
        command.store,
        command.session,
        command.plan,
        checkpoint,
        artifact,
        admission,
    )
}
