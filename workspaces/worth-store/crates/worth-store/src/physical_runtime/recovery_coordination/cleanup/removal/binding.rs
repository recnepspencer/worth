use sha2::{Digest, Sha256};

use crate::physical_runtime::PhysicalRecoveryCoordination;

use super::PhysicalRecoveryCleanupRemovalCommand;

#[derive(Clone, Copy)]
pub(super) struct CleanupEffectBinding {
    store: [u8; 16],
    session: [u8; 16],
    plan: [u8; 32],
    published_generation: u64,
    checkpoint_store: [u8; 16],
    checkpoint_sequence: u64,
    artifact_segment: u64,
    artifact_generation: u64,
    lsn_start: u64,
    lsn_end_exclusive: u64,
    byte_count: u64,
    artifact_digest: [u8; 32],
    work_runtime: u64,
    work_generation: u64,
    work_operation: u64,
}

pub(super) fn effect(
    _coordination: &PhysicalRecoveryCoordination,
    command: &PhysicalRecoveryCleanupRemovalCommand,
    work: crate::physical_runtime::PhysicalWorkIdentity,
) -> CleanupEffectBinding {
    let inspection = command.admitted_wal.inspection();
    #[cfg(feature = "certification-test-authority")]
    let artifact_generation =
        if _coordination.take_certification_cleanup_authorization_substitution() {
            command.artifact.generation().get().saturating_add(1)
        } else {
            command.artifact.generation().get()
        };
    #[cfg(not(feature = "certification-test-authority"))]
    let artifact_generation = command.artifact.generation().get();
    CleanupEffectBinding {
        store: command.store.bytes(),
        session: command.session,
        plan: command.plan,
        published_generation: command.published_generation,
        checkpoint_store: command.checkpoint.store_identity().bytes(),
        checkpoint_sequence: command.checkpoint.sequence().get(),
        artifact_segment: command.artifact.segment().get(),
        artifact_generation,
        lsn_start: command.lsn_range.start().get(),
        lsn_end_exclusive: command.lsn_range.end_exclusive().get(),
        byte_count: command.byte_count,
        artifact_digest: inspection.artifact_digest(),
        work_runtime: work.runtime().get(),
        work_generation: work.generation().lifecycle().get(),
        work_operation: work.operation().get(),
    }
}

pub(super) fn admit_selector(
    command: &PhysicalRecoveryCleanupRemovalCommand,
) -> Result<
    worth_store_physical_format::DurableRootSelector,
    crate::physical_runtime::RootProtocolAdmissionDenial,
> {
    let selector = super::super::super::source_admission::admit_scheduled_current_selector(
        &command.selector_read,
        command.store,
        command.format,
    )?;
    selector.project()
}

pub(super) fn admit_root_manifest(
    command: &PhysicalRecoveryCleanupRemovalCommand,
) -> Result<
    worth_store_physical_format::DurablePhysicalRootManifest,
    crate::physical_runtime::RootProtocolAdmissionDenial,
> {
    let root = super::super::super::source_admission::admit_scheduled_root_manifest(
        &command.root_read,
        command.store,
        command.format,
        command.published_generation,
    )?;
    root.project()
}

pub(super) fn matches(
    command: &PhysicalRecoveryCleanupRemovalCommand,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    binding: CleanupEffectBinding,
    selector: worth_store_physical_format::DurableRootSelector,
    root: worth_store_physical_format::DurablePhysicalRootManifest,
) -> bool {
    let inspection = command.admitted_wal.inspection();
    binding.store == command.store.bytes()
        && selector.store_identity() == command.store
        && selector.root_generation() == command.published_generation
        && root.generation() == command.published_generation
        && root.tree_identity() == command.checkpoint_stream.source().root().tree_identity()
        && binding.artifact_segment == command.artifact.segment().get()
        && binding.artifact_generation == command.artifact.generation().get()
        && binding.byte_count == command.byte_count
        && binding.artifact_digest == inspection.artifact_digest()
        && binding.work_runtime == work.runtime().get()
        && binding.work_generation == work.generation().lifecycle().get()
        && binding.work_operation == work.operation().get()
}

impl CleanupEffectBinding {
    pub(super) fn identity(self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.store.recovery.cleanup-effect-admission.v3");
        digest.update(self.store);
        digest.update(self.session);
        digest.update(self.plan);
        digest.update(self.published_generation.to_le_bytes());
        digest.update(self.checkpoint_store);
        digest.update(self.checkpoint_sequence.to_le_bytes());
        digest.update(self.artifact_segment.to_le_bytes());
        digest.update(self.artifact_generation.to_le_bytes());
        digest.update(self.lsn_start.to_le_bytes());
        digest.update(self.lsn_end_exclusive.to_le_bytes());
        digest.update(self.byte_count.to_le_bytes());
        digest.update(self.artifact_digest);
        digest.update(self.work_runtime.to_le_bytes());
        digest.update(self.work_generation.to_le_bytes());
        digest.update(self.work_operation.to_le_bytes());
        digest.finalize().into()
    }
}
