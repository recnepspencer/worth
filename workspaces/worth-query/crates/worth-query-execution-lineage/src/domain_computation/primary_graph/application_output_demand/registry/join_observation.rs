//! Observer-only scheduling and identity evidence at the Stable join boundary.
use super::*;
use std::sync::{Arc, Condvar, Mutex, Weak};
type Root = worth_relational::facade::identity::EntityId;
type Commit = worth_runtime_world::facade::CompositeCommitIdentity;
thread_local! {
    static JOINED: std::cell::RefCell<Vec<Root>>=const { std::cell::RefCell::new(Vec::new()) };
    static REFRESH_STOPS: std::cell::RefCell<Vec<(Root,crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind)>>=const { std::cell::RefCell::new(Vec::new()) };
    static READS: std::cell::RefCell<Vec<(Root,Commit,Commit)>>=const { std::cell::RefCell::new(Vec::new()) };
}
#[derive(Default)]
pub struct PauseState {
    reached: bool,
    released: bool,
}
#[derive(Clone)]
#[doc(hidden)]
pub struct StableJoinPauseForTest {
    latch: Arc<(Mutex<PauseState>, Condvar)>,
    receiver: Weak<Mutex<DemandRegistryState>>,
}
// Law 2, scope: pending control is stored in the registry it pauses.
// Law 3: admission returns its latch beside the charged interest; the caller
// honors it after installation and registry locks end, without growing the record.
pub type JoinLatch = Arc<(Mutex<PauseState>, Condvar)>;
pub fn take_pause(state: &mut DemandRegistryState, root: Root) -> Option<JoinLatch> {
    if state
        .join_pause
        .as_ref()
        .is_some_and(|(wanted, _)| *wanted == root)
    {
        state.join_pause.take().map(|(_, latch)| latch)
    } else {
        None
    }
}
impl WorthQueryOutputDemandRegistry {
    pub fn arm_join_pause(&self, root: Root) -> StableJoinPauseForTest {
        let latch = Arc::new((Mutex::new(PauseState::default()), Condvar::new()));
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            state.join_pause.is_none(),
            "one stable-join pause per registry"
        );
        state.join_pause = Some((root, Arc::clone(&latch)));
        StableJoinPauseForTest {
            latch,
            receiver: Arc::downgrade(&self.state),
        }
    }
}
impl StableJoinPauseForTest {
    #[doc(hidden)]
    pub fn wait_until_joined(&self, timeout: std::time::Duration) -> bool {
        let (lock, cv) = &*self.latch;
        let state = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cv.wait_timeout_while(state, timeout, |state| !state.reached)
            .unwrap()
            .0
            .reached
    }
    #[doc(hidden)]
    pub fn release(&self) {
        let (lock, cv) = &*self.latch;
        lock.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .released = true;
        cv.notify_all();
    }
}
impl Drop for StableJoinPauseForTest {
    fn drop(&mut self) {
        if let Some(receiver) = self.receiver.upgrade() {
            let mut state = receiver
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state
                .join_pause
                .as_ref()
                .is_some_and(|(_, latch)| Arc::ptr_eq(latch, &self.latch))
            {
                state.join_pause = None;
            }
        }
        self.release();
    }
}
pub fn joined(root: Root) {
    if crate::domain_computation::execution_observation::is_installed() {
        if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
            crate::domain_computation::execution_observation::register_cleanup(clear_observations);
        }
        JOINED.with(|rows| rows.borrow_mut().push(root));
    }
}
pub fn observe_stable_join_admission(pause: Option<JoinLatch>) {
    let Some(latch) = pause else {
        return;
    };
    let (lock, cv) = &*latch;
    let mut state = lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.reached = true;
    cv.notify_all();
    while !state.released {
        state = cv
            .wait(state)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}
pub fn observe_retained_source_selection(root: Root, recorded: Commit, selected: Commit) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    READS.with(|reads| reads.borrow_mut().push((root, recorded, selected)));
}

pub fn observe_required_refresh_stop(
    root: Root,
    kind: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    REFRESH_STOPS.with(|stops| stops.borrow_mut().push((root, kind)));
}

thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    JOINED.with(|records| *records.borrow_mut() = Default::default());
    REFRESH_STOPS.with(|records| *records.borrow_mut() = Default::default());
    READS.with(|records| *records.borrow_mut() = Default::default());
}

#[cfg(test)]
mod scope_tests {
    use super::*;
    fn root() -> Root {
        Root::new(
            worth_relational::facade::identity::PartitionId::main(),
            29,
            1,
        )
    }
    #[test]
    fn another_registry_cannot_consume_the_same_roots_pause() {
        let owner = WorthQueryOutputDemandRegistry::default();
        let other = WorthQueryOutputDemandRegistry::default();
        let pause = owner.arm_join_pause(root());
        assert!(take_pause(&mut other.state.lock().unwrap(), root()).is_none());
        assert!(!pause.wait_until_joined(std::time::Duration::ZERO));
        assert!(take_pause(&mut owner.state.lock().unwrap(), root()).is_some());
    }
    #[test]
    fn unwinding_an_unreached_pause_disarms_its_receiver() {
        let owner = WorthQueryOutputDemandRegistry::default();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _pause = owner.arm_join_pause(root());
            panic!("exercise pause cleanup");
        }))
        .is_err());
        assert!(owner.state.lock().unwrap().join_pause.is_none());
        let _next = owner.arm_join_pause(root());
    }
}

pub fn take_joined_stable_roots_for_test() -> Vec<Root> {
    JOINED.with(|rows| std::mem::take(&mut *rows.borrow_mut()))
}

pub fn take_retained_source_selections_for_test() -> Vec<(Root, Commit, Commit)> {
    READS.with(|reads| std::mem::take(&mut *reads.borrow_mut()))
}

pub fn take_required_refresh_stops_for_test() -> Vec<(
    Root,
    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
)> {
    REFRESH_STOPS.with(|stops| std::mem::take(&mut *stops.borrow_mut()))
}
