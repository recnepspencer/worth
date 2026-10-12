//! Law 1, reach: recovery fault injection belongs to the sealed control file.
use super::WorthQueryCheckpointTransitionRecovery;
impl WorthQueryCheckpointTransitionRecovery {
    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(&self) {
        self.graph
            .with_runtime(|runtime| runtime.fail_next_durable_append_for_test());
    }
}
