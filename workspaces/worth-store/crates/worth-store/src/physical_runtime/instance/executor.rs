use worth_store_physical_backend::QualifiedFilesystemMedia;

use crate::physical_runtime::{PhysicalExecutorCommand, PhysicalExecutorDispatch};

mod checkpoint;
mod inspection_read;
mod metadata_read;
mod publication;
mod range_read;
mod range_write;
mod recovery_obligation;
mod residency_writeback;
mod wal_append;
mod wal_barrier;
mod wal_reclamation;
mod wal_segment_create;
#[cfg(feature = "certification-test-authority")]
mod yieldpoint;

#[cfg(feature = "certification-test-authority")]
pub use yieldpoint::{
    CertificationPhysicalExecutionCheckpoint, CertificationPhysicalExecutionPauseGate,
};

/// Sole owner of the qualified media route used by physical work.
pub(in crate::physical_runtime) struct PhysicalWorkExecutor {
    media: QualifiedFilesystemMedia,
    recovery: crate::physical_runtime::work::PhysicalEffectJournal,
    #[cfg(feature = "certification-test-authority")]
    certification_yieldpoints: yieldpoint::PhysicalExecutorYieldpointOwner,
}

impl PhysicalWorkExecutor {
    pub(super) fn new(media: QualifiedFilesystemMedia) -> Self {
        let recovery = crate::physical_runtime::work::PhysicalEffectJournal::new(&media);
        Self {
            media,
            recovery,
            #[cfg(feature = "certification-test-authority")]
            certification_yieldpoints: yieldpoint::PhysicalExecutorYieldpointOwner::new(),
        }
    }

    pub(super) fn inspect_recovery(
        media: &QualifiedFilesystemMedia,
        limit: usize,
    ) -> crate::physical_runtime::work::PhysicalEffectRecoveryInventory {
        crate::physical_runtime::work::PhysicalEffectJournal::inspect(media, limit)
    }

    pub(in crate::physical_runtime) const fn record_serving_media(
        &self,
    ) -> &QualifiedFilesystemMedia {
        &self.media
    }

    pub(in crate::physical_runtime) fn into_media(self) -> QualifiedFilesystemMedia {
        self.media
    }

    pub(in crate::physical_runtime) fn dispatch(
        &self,
        command: PhysicalExecutorCommand,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        #[cfg(feature = "certification-test-authority")]
        self.certification_yieldpoints
            .pause(CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch);
        match command {
            PhysicalExecutorCommand::Inspection(command) => self.dispatch_inspection(command),
            PhysicalExecutorCommand::Metadata(command) => self.dispatch_metadata(command),
            PhysicalExecutorCommand::Read(command) => self.dispatch_read(command),
            PhysicalExecutorCommand::ExactWrite(command) => self.dispatch_exact_write(command),
            PhysicalExecutorCommand::Publication(command) => {
                self.dispatch_publication_write(command)
            }
            PhysicalExecutorCommand::NewArtifact(command) => self.dispatch_new_artifact(command),
            PhysicalExecutorCommand::PublicationEffect(command) => {
                self.dispatch_publication_effect(command, false)
            }
            PhysicalExecutorCommand::RootPublicationEffect(command) => {
                self.dispatch_publication_effect(command, true)
            }
            PhysicalExecutorCommand::ResidencyWriteback(command) => {
                self.dispatch_residency_writeback(command)
            }
            PhysicalExecutorCommand::WalAppend(command) => self.dispatch_wal_append(command),
            PhysicalExecutorCommand::WalSegmentCreate(command) => {
                self.dispatch_wal_segment_create(command)
            }
            PhysicalExecutorCommand::WalBarrier(command) => self.dispatch_wal_barrier(command),
            PhysicalExecutorCommand::Checkpoint(command) => self.dispatch_checkpoint(command),
            PhysicalExecutorCommand::WalReclamation(command) => {
                self.dispatch_wal_reclamation(command)
            }
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pause_at_for_certification(
        &self,
        checkpoint: CertificationPhysicalExecutionCheckpoint,
    ) -> CertificationPhysicalExecutionPauseGate {
        self.certification_yieldpoints.install(checkpoint)
    }
}
