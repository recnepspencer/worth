//! Receiver-owned differential verification controls.
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[derive(Debug, Default)]
pub(in crate::domain_computation) struct InvalidationOracleReceiver {
    pub(in crate::domain_computation) oracle_guards: AtomicU64,
    pub(in crate::domain_computation) oracle_checks: AtomicU64,
}
#[derive(Clone, Debug)]
pub struct WorthQueryInvalidationResources {
    oracle: Arc<InvalidationOracleReceiver>,
}
impl WorthQueryInvalidationResources {
    pub fn new() -> Self {
        let resources = Self {
            oracle: Arc::default(),
        };
        resources.attach_runtime_observations();
        resources
    }
    pub(in crate::domain_computation) fn attach_runtime_observations(&self) {
        crate::domain_computation::invalidation_equivalence_control::attach_constructed_receiver(
            &self.oracle,
        );
    }
    pub(in crate::domain_computation) fn oracle_receiver(
        &self,
    ) -> &Arc<InvalidationOracleReceiver> {
        &self.oracle
    }
    pub(in crate::domain_computation) fn invalidation_equivalence_is_armed(&self) -> bool {
        self.oracle
            .oracle_guards
            .load(std::sync::atomic::Ordering::Acquire)
            != 0
    }
    pub(in crate::domain_computation) fn record_invalidation_shadow_check(&self) {
        self.oracle
            .oracle_checks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for WorthQueryInvalidationResources {
    fn default() -> Self {
        Self::new()
    }
}
