//! Observer-only scheduling and identity evidence at the Stable join boundary.
use super::*;
use std::sync::{Arc, Condvar, Mutex};
type Root = worth_relational::facade::identity::EntityId;
type Commit = worth_runtime_world::facade::CompositeCommitIdentity;
thread_local! {
    static JOINED: std::cell::RefCell<Vec<Root>>=const { std::cell::RefCell::new(Vec::new()) };
    static REFRESH_STOPS: std::cell::RefCell<Vec<(Root,crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind)>>=const { std::cell::RefCell::new(Vec::new()) };
    static READS: std::cell::RefCell<Vec<(Root,Commit,Commit)>>=const { std::cell::RefCell::new(Vec::new()) };
    static LAST_JOIN: std::cell::Cell<Option<Root>>=const { std::cell::Cell::new(None) };
}
#[derive(Default)]
struct PauseState {
    reached: bool,
    released: bool,
}
#[derive(Clone)]
#[doc(hidden)]
pub struct StableJoinPause(Arc<(Mutex<PauseState>, Condvar)>);
impl StableJoinPause {
    #[doc(hidden)]
    pub fn wait_until_joined(&self, timeout: std::time::Duration) -> bool {
        let (lock, cv) = &*self.0;
        let state = lock.lock().unwrap();
        cv.wait_timeout_while(state, timeout, |state| !state.reached)
            .unwrap()
            .0
            .reached
    }
    #[doc(hidden)]
    pub fn release(&self) {
        let (lock, cv) = &*self.0;
        lock.lock().unwrap().released = true;
        cv.notify_all();
    }
}
impl Drop for StableJoinPause {
    fn drop(&mut self) {
        self.release();
    }
}
static PAUSE: Mutex<Option<(Root, StableJoinPause)>> = Mutex::new(None);
pub(in crate::domain_computation::primary_graph) fn joined(root: Root) {
    JOINED.with(|rows| rows.borrow_mut().push(root));
    LAST_JOIN.with(|last| last.set(Some(root)));
}
pub(in crate::domain_computation::primary_graph) fn observe_stable_join_admission() {
    let Some(root) = LAST_JOIN.with(|last| last.take()) else {
        return;
    };
    let pause = {
        let mut armed = PAUSE.lock().unwrap();
        if armed.as_ref().is_some_and(|(wanted, _)| *wanted == root) {
            armed.take().map(|(_, pause)| pause)
        } else {
            None
        }
    };
    if let Some(pause) = pause {
        let (lock, cv) = &*pause.0;
        let mut state = lock.lock().unwrap();
        state.reached = true;
        cv.notify_all();
        while !state.released {
            state = cv.wait(state).unwrap();
        }
    }
}
pub(in crate::domain_computation::primary_graph) fn observe_retained_source_selection(
    root: Root,
    recorded: Commit,
    selected: Commit,
) {
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
        let pause = StableJoinPause(Arc::new((
            Mutex::new(PauseState::default()),
            Condvar::new(),
        )));
        *PAUSE.lock().unwrap() = Some((root, pause.clone()));
        pause
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
