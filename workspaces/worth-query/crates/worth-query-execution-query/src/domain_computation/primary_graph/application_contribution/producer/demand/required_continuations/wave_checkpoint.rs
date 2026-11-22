//! A checkpoint turn inspects the original successor and retains its actual owner.
use super::*;
pub struct WaveCheckpoint<'a, Schema: ApplicationSchema> {
    progress: &'a mut RequiredFreshProgress<Schema>,
}
pub enum WaveCheckpointTurn<'a, Schema: ApplicationSchema> {
    Current(CheckpointReady),
    Awaiting(CheckpointStage<'a, Schema>),
}
pub struct CheckpointReady {
    ready: SelectedReadyReadmission,
}
impl CheckpointReady {
    pub fn ready(&self) -> &SelectedReadyReadmission {
        &self.ready
    }
    pub fn into_readmission(self) -> SelectedReadyReadmission {
        self.ready
    }
}
pub struct CheckpointStage<'a, Schema: ApplicationSchema> {
    progress: &'a mut RequiredFreshProgress<Schema>,
}
impl<Schema: ApplicationSchema + 'static> RequiredFreshProgress<Schema> {
    pub fn checkpoint(&mut self) -> WaveCheckpoint<'_, Schema> {
        WaveCheckpoint { progress: self }
    }
}
impl<'a, Schema: ApplicationSchema + 'static> WaveCheckpoint<'a, Schema> {
    pub fn inspect(
        self,
        registry: &WorthQueryOutputDemandRegistry,
    ) -> Result<WaveCheckpointTurn<'a, Schema>, WorthQueryOutputDemandDenial> {
        match registry.interest_ready_readmission(self.progress.interest())? {
            Some(ready) => Ok(WaveCheckpointTurn::Current(CheckpointReady { ready })),
            None => Ok(WaveCheckpointTurn::Awaiting(CheckpointStage {
                progress: self.progress,
            })),
        }
    }
}
impl<Schema: ApplicationSchema + 'static> CheckpointStage<'_, Schema> {
    pub fn advance(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.progress
            .successor
            .advance_checkpoint(phase, runtime, request, principal, invocation)
    }
}
