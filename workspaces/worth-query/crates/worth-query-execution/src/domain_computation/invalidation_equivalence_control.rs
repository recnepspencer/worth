//! Law 2, scope: verification reads only its runtime's receiver-owned arm.
//! The thread-bound factory applies solely while a receiver is being constructed.
use super::execution_runtime::source_invalidation::{
    InvalidationRetentionLedger, WorthQueryInvalidationResources,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak as LocalWeak},
    sync::{atomic::Ordering, Arc},
};
#[derive(Default)]
// The guard retains accounting/recording cells, never a runtime, image or ticket.
// Counts include receivers whose actual runtime has already been retired.
struct ConstructionFactory {
    receivers: RefCell<Vec<Arc<InvalidationRetentionLedger>>>,
}
thread_local! {
 static FACTORIES: RefCell<Vec<LocalWeak<ConstructionFactory>>> = const { RefCell::new(Vec::new()) };
}
enum OwnedArm {
    Construction(Rc<ConstructionFactory>),
    Receiver(Arc<InvalidationRetentionLedger>),
}
/// Owns differential verification on the runtime receivers it armed.
/// This grants no execution, publication or installation authority.
#[must_use = "the differential oracle is armed only while the guard is held"]
pub struct WorthQueryInvalidationEquivalence {
    arm: OwnedArm,
}
impl WorthQueryInvalidationEquivalence {
    /// Arm receivers constructed on this thread until this factory guard drops.
    /// Once constructed, each receiver also shadows verification on its workers.
    pub fn arm_on_this_thread() -> Self {
        let factory = Rc::new(ConstructionFactory::default());
        FACTORIES.with(|factories| factories.borrow_mut().push(Rc::downgrade(&factory)));
        Self {
            arm: OwnedArm::Construction(factory),
        }
    }
    /// Arm one existing runtime's resources, including its verification workers.
    pub fn arm_for_invalidation_resources(resources: &WorthQueryInvalidationResources) -> Self {
        let receiver = Arc::clone(resources.retention_receiver());
        receiver.oracle_guards.fetch_add(1, Ordering::AcqRel);
        Self {
            arm: OwnedArm::Receiver(receiver),
        }
    }
    /// Actual shadow checks performed by the runtime receivers this guard armed.
    pub fn shadow_checks_for_test(&self) -> u64 {
        match &self.arm {
            OwnedArm::Receiver(receiver) => receiver.oracle_checks.load(Ordering::Relaxed),
            OwnedArm::Construction(factory) => factory
                .receivers
                .borrow()
                .iter()
                .map(|receiver| receiver.oracle_checks.load(Ordering::Relaxed))
                .sum(),
        }
    }
}
impl Drop for WorthQueryInvalidationEquivalence {
    fn drop(&mut self) {
        match &self.arm {
            OwnedArm::Receiver(receiver) => {
                receiver.oracle_guards.fetch_sub(1, Ordering::AcqRel);
            }
            OwnedArm::Construction(factory) => {
                FACTORIES.with(|factories| {
                    factories.borrow_mut().retain(|candidate| {
                        !candidate.ptr_eq(&Rc::downgrade(factory)) && candidate.strong_count() != 0
                    })
                });
                for receiver in factory.receivers.borrow_mut().drain(..) {
                    receiver.oracle_guards.fetch_sub(1, Ordering::AcqRel);
                }
            }
        }
    }
}
/// Cold construction only. No verification or row path reads thread ambient state.
pub(in crate::domain_computation) fn attach_constructed_receiver(
    receiver: &Arc<InvalidationRetentionLedger>,
) {
    FACTORIES.with(|factories| {
        let mut factories = factories.borrow_mut();
        factories.retain(|factory| factory.strong_count() != 0);
        for factory in factories.iter().filter_map(LocalWeak::upgrade) {
            let mut receivers = factory.receivers.borrow_mut();
            if !receivers
                .iter()
                .any(|existing| Arc::ptr_eq(existing, receiver))
            {
                receiver.oracle_guards.fetch_add(1, Ordering::AcqRel);
                receivers.push(Arc::clone(receiver));
            }
        }
    });
}
#[cfg(test)]
mod scope_tests {
    use super::*;
    fn resources() -> WorthQueryInvalidationResources {
        let resources = WorthQueryInvalidationResources::install(super::super::execution_runtime::source_invalidation::WorthQueryInvalidationResourceInstallation::bounded(1, 1, 17, 1)).unwrap();
        resources.attach_runtime_observations();
        resources
    }

    #[test]
    fn constructing_under_a_factory_covers_the_receivers_workers_and_ends_on_drop() {
        let ordinary = resources();
        let guard = WorthQueryInvalidationEquivalence::arm_on_this_thread();
        let controlled = resources();
        assert!(!ordinary.invalidation_equivalence_is_armed());
        let worker = controlled.clone();
        assert!(
            std::thread::spawn(move || worker.invalidation_equivalence_is_armed())
                .join()
                .unwrap()
        );
        drop(guard);
        assert!(!controlled.invalidation_equivalence_is_armed());
    }
    #[test]
    fn resources_installed_here_carry_the_arm_into_worker_construction() {
        let guard = WorthQueryInvalidationEquivalence::arm_on_this_thread();
        let resources = WorthQueryInvalidationResources::install(super::super::execution_runtime::source_invalidation::WorthQueryInvalidationResourceInstallation::bounded(1, 1, 17, 1)).unwrap();
        let worker = resources.clone();
        assert!(std::thread::spawn(move || {
            worker.attach_runtime_observations();
            worker.invalidation_equivalence_is_armed()
        })
        .join()
        .unwrap());
        drop(guard);
        assert!(!resources.invalidation_equivalence_is_armed());
    }
    #[test]
    fn unwinding_an_explicit_arm_clears_only_its_receiver() {
        let controlled = resources();
        let other = resources();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard =
                WorthQueryInvalidationEquivalence::arm_for_invalidation_resources(&controlled);
            assert!(controlled.invalidation_equivalence_is_armed());
            assert!(!other.invalidation_equivalence_is_armed());
            panic!("exercise receiver oracle cleanup");
        }))
        .is_err());
        assert!(!controlled.invalidation_equivalence_is_armed());
    }
}
