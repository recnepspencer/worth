//! Receiver-owned evidence about every closure-scoped installed-owner acquisition.
use super::WorthQueryRelationalSourceOwner;
#[derive(Clone)]
pub(in crate::domain_computation) struct ReadLockProbeForTest {
    owner: WorthQueryRelationalSourceOwner,
}
impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn read_lock_probe_for_test(&self) -> ReadLockProbeForTest {
        ReadLockProbeForTest {
            owner: self.clone(),
        }
    }
}
impl ReadLockProbeForTest {
    pub(in crate::domain_computation) fn is_locked_for_test(&self) -> bool {
        self.owner.runtime.try_lock().is_err()
    }
    pub(in crate::domain_computation) fn acquisitions_for_test(&self) -> u64 {
        self.owner
            .read_acquisitions
            .load(std::sync::atomic::Ordering::SeqCst)
    }
    pub(in crate::domain_computation) fn with_owner_lock_for_test<T>(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce() -> T,
    ) -> T {
        self.owner.with_runtime(request, |_| read())
    }
}
