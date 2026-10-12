//! Receiver-owned, read-only evidence about the installed owner's lock.
use super::WorthQueryRelationalSourceOwner;
use std::sync::{Arc, Mutex};
use worth_relational::facade::runtime::RelationalRuntime;
#[derive(Clone)]
pub(in crate::domain_computation) struct ReadLockProbeForTest {
    runtime: Arc<Mutex<RelationalRuntime>>,
}
impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn read_lock_probe_for_test(&self) -> ReadLockProbeForTest {
        ReadLockProbeForTest {
            runtime: Arc::clone(&self.runtime),
        }
    }
}
impl ReadLockProbeForTest {
    pub(in crate::domain_computation) fn is_locked_for_test(&self) -> bool {
        self.runtime.try_lock().is_err()
    }
    pub(in crate::domain_computation) fn with_owner_lock_for_test<T>(
        &self,
        read: impl FnOnce() -> T,
    ) -> T {
        let _owner = self.runtime.lock().unwrap();
        read()
    }
}
