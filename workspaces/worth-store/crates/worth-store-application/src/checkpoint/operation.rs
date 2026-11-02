//! One actual Home checkpoint owner serves manual and automatic advancement.
use super::{
    owner::HomeCheckpointOwner, ApplicationHomeCheckpoint as Summary,
    ApplicationHomeCheckpointDenial as Denial, ApplicationHomeCheckpointLimits,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::WorldCheckpointPort;
use worth_store::physical_runtime::ServingPhysicalRuntime;
use worth_store_relational::RelationalSectionCompleteCheckpointPreparation;

/// A Home-issued operation, never a Home lifecycle or Native service owner.
/// Actual pending work remains in the SAME Home checkpoint owner on refusal.
/// The Native and World preparation ports are weak; this value cannot reopen them.
pub struct ApplicationHomeCheckpointOperation {
    serving: Arc<ServingPhysicalRuntime>,
    owner: Arc<HomeCheckpointOwner>,
    world: WorldCheckpointPort,
    native: RelationalSectionCompleteCheckpointPreparation,
    limits: ApplicationHomeCheckpointLimits,
}
impl std::fmt::Debug for ApplicationHomeCheckpointOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ApplicationHomeCheckpointOperation")
    }
}
impl ApplicationHomeCheckpointOperation {
    pub(crate) fn new(
        serving: Arc<ServingPhysicalRuntime>,
        owner: Arc<HomeCheckpointOwner>,
        world: WorldCheckpointPort,
        native: RelationalSectionCompleteCheckpointPreparation,
        limits: ApplicationHomeCheckpointLimits,
    ) -> Self {
        Self {
            serving,
            owner,
            world,
            native,
            limits,
        }
    }

    pub fn checkpoint(&self, request: &ExecutionRequest<'_, '_>) -> Result<Summary, Denial> {
        super::journey::checkpoint(
            &self.serving,
            &self.owner,
            Some(&self.world),
            request,
            || self.native.prepare(request),
            None,
            None,
            None,
        )
    }
    /// The actual operation owns this passive observer through its selected
    /// boundary. It grants neither selection nor acknowledgement authority.
    pub fn checkpoint_with_native_ack_observer_for_test(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observer: Box<dyn Fn() + Send + Sync>,
    ) -> Result<Summary, Denial> {
        super::journey::checkpoint(
            &self.serving,
            &self.owner,
            Some(&self.world),
            request,
            || self.native.prepare(request),
            None,
            Some(observer.as_ref()),
            None,
        )
    }
    pub fn recover_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Summary, Denial> {
        super::journey::recover(&self.serving, &self.owner, request)
    }
    /// Run only actual due work, or resume the original accepted attempt.
    /// Failure is retained by that attempt or remains due in actual journal tails;
    /// it never becomes a separate error ledger or a replacement business result.
    pub fn after_advancement(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<Summary>, Denial> {
        request.consult().map_err(Denial::Execution)?;
        if self.owner.pending().is_some() {
            return self.recover_checkpoint(request).map(Some);
        }
        let observed = super::inspection::inspect(
            &self.serving,
            &self.owner,
            &self.world,
            self.limits,
            self.native
                .cold_storage_reads()
                .ok_or(Denial::Native(
                    worth_store_relational::RelationalSectionCheckpointDenial::OwnerUnavailable,
                ))?
                .commit_reads,
            request,
        )?;
        if !observed.due {
            return Ok(None);
        }
        self.checkpoint(request).map(Some)
    }
}
