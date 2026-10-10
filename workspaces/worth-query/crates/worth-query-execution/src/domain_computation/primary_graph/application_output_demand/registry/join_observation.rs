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
pub(in crate::domain_computation::primary_graph) struct PauseState {
    reached: bool,
    released: bool,
}
#[derive(Clone)]
#[doc(hidden)]
pub struct StableJoinPause {
    latch: Arc<(Mutex<PauseState>, Condvar)>,
    receiver: Weak<Mutex<DemandRegistryState>>,
}
// Law 2, scope: pending control is stored in the registry it pauses.
// Law 3: admission returns its latch beside the charged interest; the caller
// honors it after installation and registry locks end, without growing the record.
pub(in crate::domain_computation::primary_graph) type JoinLatch = Arc<(Mutex<PauseState>, Condvar)>;
pub(super) fn take_pause(state: &mut DemandRegistryState, root: Root) -> Option<JoinLatch> {
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
    fn arm_join_pause(&self, root: Root) -> StableJoinPause {
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
        StableJoinPause {
            latch,
            receiver: Arc::downgrade(&self.state),
        }
    }
}
impl StableJoinPause {
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
impl Drop for StableJoinPause {
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
pub(in crate::domain_computation::primary_graph) fn joined(root: Root) {
    if crate::domain_computation::execution_observation::is_installed() {
        if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
            crate::domain_computation::execution_observation::register_cleanup(clear_observations);
        }
        JOINED.with(|rows| rows.borrow_mut().push(root));
    }
}
pub(in crate::domain_computation::primary_graph) fn observe_stable_join_admission(
    pause: Option<JoinLatch>,
) {
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
pub(in crate::domain_computation::primary_graph) fn observe_retained_source_selection(
    root: Root,
    recorded: Commit,
    selected: Commit,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    READS.with(|reads| reads.borrow_mut().push((root, recorded, selected)));
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn joined_stable_roots_for_test(&self) -> Vec<Root> {
        JOINED.with(|rows| std::mem::take(&mut *rows.borrow_mut()))
    }
    #[doc(hidden)]
    pub fn retained_source_selections_for_test(&self) -> Vec<(Root, Commit, Commit)> {
        READS.with(|reads| std::mem::take(&mut *reads.borrow_mut()))
    }
    #[doc(hidden)]
    pub fn pause_stable_join_for_test(&self, root: Root) -> StableJoinPause {
        self.output_demands.arm_join_pause(root)
    }
    #[doc(hidden)]
    pub fn registry_row_keys_for_test(&self) -> Vec<(String, Root, u64, [u8; 32])> {
        self.output_demands
            .state
            .lock()
            .unwrap()
            .records
            .iter()
            .map(|(key, _)| {
                (
                    key.producer.clone(),
                    key.source.root_entity_for_test(),
                    key.source.observation_generation(),
                    key.source.runtime_idempotency_identity(),
                )
            })
            .collect()
    }
    #[doc(hidden)]
    pub fn registry_successor_claims_for_test(&self) -> usize {
        self.output_demands
            .state
            .lock()
            .unwrap()
            .records
            .values()
            .filter(|row| row.successor_of.is_some())
            .count()
    }
}

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn stable_ready_sources_for_test(&self) -> Vec<(Root, Commit, [u8; 32])> {
        self.output_demands
            .state
            .lock()
            .unwrap()
            .records
            .iter()
            .filter_map(|(key, row)| {
                let DemandState::Output(output) = &row.state else {
                    return None;
                };
                let Some(WorthQueryOutputCheckpoint::Ready(ready)) = &output.checkpoint else {
                    return None;
                };
                let WorthQueryAcceptedOutputAuthority::Stable(stable) = &ready.authority else {
                    return None;
                };
                Some((
                    key.source.root_entity_for_test(),
                    stable.observation().selected_commit().clone(),
                    stable.idempotency_key_identity(),
                ))
            })
            .collect()
    }
}

pub(in crate::domain_computation::primary_graph) fn observe_required_refresh_stop(
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
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn required_refresh_stops_for_test(
        &self,
    ) -> Vec<(
        Root,
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
    )> {
        REFRESH_STOPS.with(|stops| std::mem::take(&mut *stops.borrow_mut()))
    }
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
