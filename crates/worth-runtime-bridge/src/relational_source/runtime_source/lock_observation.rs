//! Observe the native owner lock acquired through this source and its clones.
use super::RuntimeBridgeRelationalSource;
impl RuntimeBridgeRelationalSource {
    /// The guard owns passive observation; it grants no runtime access.
    #[doc(hidden)]
    pub fn observe_runtime_locks_for_test(
        &self,
        acquired: impl Fn() + Send + Sync + 'static,
    ) -> worth_relational::facade::change_source::RelationalRuntimeLockObservationForTest {
        self.runtime.observe_locks_for_test(acquired)
    }
}
