//! Manual capture, recovery and inspection use the Directory's actual owner.
use super::runtime::WorthQueryApplicationHome;
use worth_execution::ExecutionRequest;
use worth_store_application::{
    ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial, ApplicationHomeCheckpointInspection,
};

#[derive(Debug)]
pub enum WorthQueryApplicationHomeCheckpointDenial {
    MemoryLifecycleRequired,
    Directory(ApplicationHomeCheckpointDenial),
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program> {
    pub fn checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, WorthQueryApplicationHomeCheckpointDenial> {
        self.directory
            .as_ref()
            .ok_or(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)?
            .checkpoint(request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::Directory)
    }

    pub fn recover_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, WorthQueryApplicationHomeCheckpointDenial> {
        self.directory
            .as_ref()
            .ok_or(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)?
            .recover_checkpoint(request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::Directory)
    }

    pub fn inspect_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpointInspection, WorthQueryApplicationHomeCheckpointDenial>
    {
        self.directory
            .as_ref()
            .ok_or(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)?
            .inspect_checkpoint(self.checkpoint_limits, request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::Directory)
    }
}
