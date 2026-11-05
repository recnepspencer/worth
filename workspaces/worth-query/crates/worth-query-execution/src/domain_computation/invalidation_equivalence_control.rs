//! Law 2, scope: verification reads only its runtime's receiver-owned arm.
//! The thread-bound factory applies solely while a receiver is being constructed.
use super::execution_runtime::source_invalidation::InvalidationOracleReceiver;
use super::execution_runtime::source_invalidation::WorthQueryInvalidationResources;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak as LocalWeak;
use std::sync::atomic::Ordering;
use std::sync::Arc;
#[derive(Default)]
// The guard retains accounting/recording cells, never a runtime, image or ticket.
// Counts include receivers whose actual runtime has already been retired.
struct ConstructionFactory {
    receivers: RefCell<Vec<Arc<InvalidationOracleReceiver>>>,
}
thread_local! {
 static FACTORIES: RefCell<Vec<LocalWeak<ConstructionFactory>>> = const { RefCell::new(Vec::new()) };
}
enum OwnedArm {
    Construction(Rc<ConstructionFactory>),
    Receiver(Arc<InvalidationOracleReceiver>),
}
/// Owns differential verification on the runtime receivers it armed.
/// This grants no execution, publication or installation authority.
#[must_use = "the differential oracle is armed only while the guard is held"]
pub struct WorthQueryInvalidationEquivalenceForTest {
    arm: OwnedArm,
}
impl WorthQueryInvalidationEquivalenceForTest {
    /// Arm receivers constructed on this thread until this factory guard drops.
    /// Once constructed, each receiver also shadows verification on its workers.
    pub fn arm_on_this_thread_for_test() -> Self {
        let factory = Rc::new(ConstructionFactory::default());
        FACTORIES.with(|factories| factories.borrow_mut().push(Rc::downgrade(&factory)));
        Self {
            arm: OwnedArm::Construction(factory),
        }
    }
    /// Arm one existing runtime's resources, including its verification workers.
    pub fn arm_for_invalidation_resources_for_test(
        resources: &WorthQueryInvalidationResources,
    ) -> Self {
        let receiver = Arc::clone(resources.oracle_receiver());
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
impl Drop for WorthQueryInvalidationEquivalenceForTest {
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
    receiver: &Arc<InvalidationOracleReceiver>,
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
