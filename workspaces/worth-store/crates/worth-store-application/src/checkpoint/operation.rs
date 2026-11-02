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
/// Original operation metadata after its actual serving alias is discharged.
pub struct RetiredApplicationHomeCheckpointOperation {
    _owner: Arc<HomeCheckpointOwner>,
    _world: WorldCheckpointPort,
    _native: RelationalSectionCompleteCheckpointPreparation,
    _limits: ApplicationHomeCheckpointLimits,
}
impl std::fmt::Debug for RetiredApplicationHomeCheckpointOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RetiredApplicationHomeCheckpointOperation")
    }
}
impl std::fmt::Debug for ApplicationHomeCheckpointOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ApplicationHomeCheckpointOperation")
    }
}
impl ApplicationHomeCheckpointOperation {
    pub(crate) fn retire_for_home(
        self,
        owner: &Arc<ServingPhysicalRuntime>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetiredApplicationHomeCheckpointOperation, (Self, crate::ApplicationHomeCloseDenial)>
    {
        if !Arc::ptr_eq(&self.serving, owner) {
            return Err((
                self,
                crate::ApplicationHomeCloseDenial::ForeignCheckpointOperation,
            ));
        }
        if let Err(cause) = request.consult() {
            return Err((self, crate::ApplicationHomeCloseDenial::Execution(cause)));
        }
        let Self {
            serving,
            owner,
            world,
            native,
            limits,
        } = self;
        // The matching borrowed Home Arc guarantees nonfinal local release.
        drop(serving);
        Ok(RetiredApplicationHomeCheckpointOperation {
            _owner: owner,
            _world: world,
            _native: native,
            _limits: limits,
        })
    }
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
            request,
        )?;
        if !observed.due {
            return Ok(None);
        }
        self.checkpoint(request).map(Some)
    }
}
