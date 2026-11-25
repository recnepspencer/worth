//! Manual capture, recovery and inspection use the Directory's actual owner.
use super::runtime::WorthQueryApplicationHome;
use worth_execution::ExecutionRequest;
use worth_store_application::{
    ApplicationHomeCheckpoint, ApplicationHomeCheckpointDenial, ApplicationHomeCheckpointInspection,
};

#[derive(Debug)]
pub enum WorthQueryApplicationHomeCheckpointDenial {
    MemoryLifecycleRequired,
    CoupledOutputRetention(
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial,
    ),
    Directory(ApplicationHomeCheckpointDenial),
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program> {
    pub fn checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeCheckpoint, WorthQueryApplicationHomeCheckpointDenial> {
        let directory = self
            .directory
            .as_ref()
            .ok_or(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)?;
        self.query
            .flush_pending_coupled_outputs(request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::CoupledOutputRetention)?;
        directory
            .checkpoint(request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::Directory)
    }

    pub fn checkpoint_in_advancement(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<ApplicationHomeCheckpoint, WorthQueryApplicationHomeCheckpointDenial> {
        self.checkpoint(&phase.execution_request())
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

    /// Receiver-owned observation of the actual selected attempt before Native ACK.
    pub fn checkpoint_with_native_ack_observer_for_test(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observer: Box<dyn Fn() + Send + Sync>,
    ) -> Result<ApplicationHomeCheckpoint, WorthQueryApplicationHomeCheckpointDenial> {
        let directory = self
            .directory
            .as_ref()
            .ok_or(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)?;
        self.query
            .flush_pending_coupled_outputs(request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::CoupledOutputRetention)?;
        directory
            .checkpoint_operation(self.checkpoint_limits)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::Directory)?
            .checkpoint_with_native_ack_observer_for_test(request, observer)
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

    /// D14 sealed certification door: discards every durable `RetentionClass::Derived`
    /// artifact held by this Directory Home so reopen reconstructs from authoritative
    /// journals only. Callers are recorded in `tools/boundary-check/config/control-callers.txt`.
    #[doc(hidden)]
    pub fn discard_derived_for_test(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, WorthQueryApplicationHomeCheckpointDenial> {
        self.directory
            .as_ref()
            .ok_or(WorthQueryApplicationHomeCheckpointDenial::MemoryLifecycleRequired)?
            .discard_derived_for_test(request)
            .map_err(WorthQueryApplicationHomeCheckpointDenial::Directory)
    }
}
