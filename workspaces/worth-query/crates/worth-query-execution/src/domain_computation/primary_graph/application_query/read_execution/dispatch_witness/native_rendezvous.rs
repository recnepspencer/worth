//! Two distinct roots hold a native read open using only observed events.
use super::*;
use crate::domain_computation::execution_runtime::product_world::ReadLockProbeForTest;
pub(super) struct NativeReadRendezvous {
    roots: [EntityId; 2],
    owner: ReadLockProbeForTest,
    state: Mutex<NativeState>,
    ready: Condvar,
}
#[derive(Default)]
struct NativeState {
    workers: std::collections::HashSet<std::thread::ThreadId>,
    entered: std::collections::BTreeSet<EntityId>,
    held: usize,
    peak: usize,
    owner_locked: usize,
    open: bool,
    watchdog_failed: bool,
    exits: std::collections::BTreeMap<EntityId, usize>,
}
pub(in crate::domain_computation::primary_graph) struct NativeReadRendezvousGuardForTest {
    witness: Arc<DispatchWitness>,
    gate: Arc<NativeReadRendezvous>,
    previous: Option<Arc<NativeReadRendezvous>>,
}
impl DispatchWitness {
    pub(in crate::domain_computation::primary_graph) fn hold_native_reads_for_test(
        self: &Arc<Self>,
        roots: [EntityId; 2],
        owner: ReadLockProbeForTest,
    ) -> NativeReadRendezvousGuardForTest {
        assert_ne!(roots[0], roots[1]);
        let gate = Arc::new(NativeReadRendezvous {
            roots,
            owner,
            state: Mutex::new(NativeState::default()),
            ready: Condvar::new(),
        });
        let previous = self.native_reads.lock().unwrap().replace(Arc::clone(&gate));
        NativeReadRendezvousGuardForTest {
            witness: Arc::clone(self),
            gate,
            previous,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn with_native_owner_lock_for_test<T>(
        &self,
        read: impl FnOnce() -> T,
    ) -> T {
        let gate = self.native_reads.lock().unwrap().clone();
        match gate {
            Some(gate) => gate.owner.with_owner_lock_for_test(read),
            None => read(),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn native_read_exit(
        &self,
        root: EntityId,
    ) {
        if let Some(gate) = self.native_reads.lock().unwrap().clone() {
            gate.exit(root);
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn native_read_enter(
        &self,
        root: EntityId,
    ) {
        let gate = self.native_reads.lock().unwrap().clone();
        if let Some(gate) = gate {
            gate.enter(root);
        }
    }
}
impl NativeReadRendezvous {
    fn exit(&self, root: EntityId) {
        let mut state = self.state.lock().unwrap();
        if self.roots.contains(&root) {
            *state.exits.entry(root).or_default() += 1;
            if self.owner.is_locked_for_test() {
                state.owner_locked += 1;
            }
        }
    }
    fn enter(&self, root: EntityId) {
        let mut state = self.state.lock().unwrap();
        if !self.roots.contains(&root) || state.entered.contains(&root) {
            return;
        }
        state.entered.insert(root);
        // This fails the old lock-spanning read without leaving a peer waiting.
        if self.owner.is_locked_for_test() {
            state.owner_locked += 1;
            state.open = true;
            self.ready.notify_all();
            return;
        }
        state.workers.insert(std::thread::current().id());
        state.held += 1;
        state.peak = state.peak.max(state.held);
        if state.entered.len() == self.roots.len() {
            state.open = true;
            self.ready.notify_all();
        }
        // Failure watchdog only: passing synchronization ends on the peer's event.
        let (mut state, timeout) = self
            .ready
            .wait_timeout_while(state, Duration::from_secs(60), |state| !state.open)
            .unwrap();
        state.watchdog_failed |= timeout.timed_out();
        if state.watchdog_failed {
            state.open = true;
            self.ready.notify_all();
        }
        state.held -= 1;
    }
}
impl NativeReadRendezvousGuardForTest {
    pub(in crate::domain_computation::primary_graph) fn assert_overlap_for_test(&self) {
        let state = self.gate.state.lock().unwrap();
        assert!(
            !state.watchdog_failed,
            "a peer missed the native read rendezvous"
        );
        assert_eq!(
            state.exits,
            std::collections::BTreeMap::from(self.gate.roots.map(|root| (root, 2))),
            "both sides of each rooted read must exit"
        );
        assert_eq!(
            state.owner_locked, 0,
            "a native read retained its owner's lock"
        );
        assert_eq!(
            state.entered,
            std::collections::BTreeSet::from(self.gate.roots)
        );
        assert_eq!(
            state.workers.len(),
            2,
            "the two reads must run on distinct workers"
        );
        assert_eq!(
            state.peak, 2,
            "both distinct roots must hold their native reads open together"
        );
    }
}
impl Drop for NativeReadRendezvousGuardForTest {
    fn drop(&mut self) {
        {
            let mut state = self
                .gate
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.open = true;
            self.gate.ready.notify_all();
        }
        *self
            .witness
            .native_reads
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = self.previous.take();
    }
}
