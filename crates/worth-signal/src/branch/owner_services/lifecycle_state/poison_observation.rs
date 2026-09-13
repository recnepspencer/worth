use super::{Ordering, SignalOwnerLifecyclePoisonRecovery, SignalOwnerLifecycleState};

impl SignalOwnerLifecycleState {
    pub(crate) fn poison_recovery(&self) -> Option<SignalOwnerLifecyclePoisonRecovery> {
        drop(self.lock_transition_gate());
        self.recovered_poison
            .load(Ordering::Acquire)
            .then_some(SignalOwnerLifecyclePoisonRecovery::PreservedLifecycleStatus)
    }
}
