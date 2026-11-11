//! One prepaid cold result slot follows the actual Home through release.
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial};

type CheckpointResult =
    Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial>;

struct Backing {
    result: Option<CheckpointResult>,
    native_retirement: Option<worth_store_relational::PreparedRelationalSectionClose>,
    memory: Option<ExecutionMemoryReservation>,
}

pub(crate) struct HomeCloseCheckpointResult {
    // Present throughout the live owner; Option permits taking the exact Box
    // for consuming destruction without allocating a replacement.
    backing: Option<Box<Backing>>,
}
impl HomeCloseCheckpointResult {
    pub(crate) fn prepare(execution: &ExecutionRequest<'_, '_>) -> Result<Self, WorkCeilingDenial> {
        let memory = execution.reserve_retained_memory(std::mem::size_of::<Backing>() as u64)?;
        let native_retirement =
            worth_store_relational::PreparedRelationalSectionClose::prepare(execution)?;
        Ok(Self {
            backing: Some(Box::new(Backing {
                result: None,
                native_retirement: Some(native_retirement),
                memory,
            })),
        })
    }
    pub(crate) fn take_native_retirement(
        &mut self,
    ) -> worth_store_relational::PreparedRelationalSectionClose {
        self.backing
            .as_mut()
            .expect("same prepaid Home backing")
            .native_retirement
            .take()
            .expect("original Native close storage moves exactly once into its section")
    }
    pub(crate) fn restore_native_retirement(
        &mut self,
        original: worth_store_relational::PreparedRelationalSectionClose,
    ) {
        let slot = &mut self
            .backing
            .as_mut()
            .expect("same prepaid Home backing")
            .native_retirement;
        assert!(
            slot.is_none(),
            "reversible refusal returns the original vacant preparation"
        );
        *slot = Some(original);
    }
    pub(crate) fn is_none(&self) -> bool {
        self.backing
            .as_ref()
            .expect("live result backing")
            .result
            .is_none()
    }
    pub(crate) fn store(&mut self, result: CheckpointResult) {
        self.backing.as_mut().expect("live result backing").result = Some(result);
    }
    pub(crate) fn result(&self) -> &CheckpointResult {
        self.backing
            .as_ref()
            .expect("live result backing")
            .result
            .as_ref()
            .expect("release custody owns its settled checkpoint result")
    }
    pub(crate) fn into_result(mut self) -> CheckpointResult {
        let mut backing = self.backing.take().expect("live result backing");
        let result = backing
            .result
            .take()
            .expect("release custody owns its settled checkpoint result");
        Self::release(backing);
        result
    }
    fn release(mut backing: Box<Backing>) {
        let memory = backing.memory.take();
        // Child owners drop with their result; the fixed Box is physically
        // deallocated before this backing's own reservation can be refunded.
        drop(backing);
        drop(memory);
    }
}
impl Drop for HomeCloseCheckpointResult {
    fn drop(&mut self) {
        if let Some(backing) = self.backing.take() {
            Self::release(backing);
        }
    }
}
#[cfg(test)]
mod tests;
