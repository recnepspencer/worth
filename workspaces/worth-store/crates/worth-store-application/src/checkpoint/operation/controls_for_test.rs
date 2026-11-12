//! Observe the original coupled operation, without a worldless checkpoint path.
use super::*;
impl ApplicationHomeCheckpointOperation {
    pub fn checkpoint_with_step_observer_for_test(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observer: impl Fn(&worth_store::physical_runtime::AnchoredRecordStepRegistration) + Send + Sync,
    ) -> Result<Summary, Denial> {
        super::super::journey::checkpoint(
            &self.serving,
            &self.owner,
            Some(&self.world),
            Some(&self.signal),
            request,
            |home, through, digest| self.native.prepare_home(home, through, digest, request),
            Some(&observer),
            None,
            None,
        )
    }
    pub fn checkpoint_with_acknowledgement_observer_for_test(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observer: impl Fn() + Send + Sync,
    ) -> Result<Summary, Denial> {
        super::super::journey::checkpoint(
            &self.serving,
            &self.owner,
            Some(&self.world),
            Some(&self.signal),
            request,
            |home, through, digest| self.native.prepare_home(home, through, digest, request),
            None,
            None,
            Some(&observer),
        )
    }
    #[cfg(feature = "certification-test-authority")]
    pub fn checkpoint_input_memory_bytes_for_test(&self) -> Option<u64> {
        self.owner.input_memory_bytes_for_test()
    }
    pub fn checkpoint_observation_for_test(
        &self,
    ) -> Option<super::super::ApplicationHomeCheckpointRecovery> {
        self.owner.pending().map(|attempt| attempt.observation())
    }
    pub fn physical_serving_for_test(&self) -> &ServingPhysicalRuntime {
        &self.serving
    }
}
