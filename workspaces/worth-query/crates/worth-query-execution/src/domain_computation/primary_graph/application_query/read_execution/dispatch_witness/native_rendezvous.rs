//! Two distinct roots hold a native read open using only observed events.
use super::*;
use crate::domain_computation::execution_runtime::product_world::ReadLockProbeForTest;
pub(super) struct NativeReadRendezvous {
    roots: [EntityId; 2],
    owner: ReadLockProbeForTest,
    state: Mutex<NativeState>,
    ready: Condvar,
    read_snapshot: Mutex<Vec<worth_relational::facade::snapshots::SnapshotHandle>>,
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
    initial_acquisitions: Option<u64>,
    exit_acquisitions: Vec<(EntityId, u64)>,
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
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> NativeReadRendezvousGuardForTest {
        assert_ne!(roots[0], roots[1]);
        let snapshots = vec![
            owner.retain_snapshot_for_test(request),
            owner.retain_snapshot_for_test(request),
        ];
        let gate = Arc::new(NativeReadRendezvous {
            roots,
            owner,
            state: Mutex::new(NativeState::default()),
            ready: Condvar::new(),
            read_snapshot: Mutex::new(snapshots),
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
        request: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce() -> T,
    ) -> T {
        let gate = self.native_reads.lock().unwrap().clone();
        match gate {
            Some(gate) => gate.owner.with_owner_lock_for_test(request, read),
            None => read(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn release_native_snapshot_for_test(&self) {
        if let Some(gate) = self.native_reads.lock().unwrap().clone()
            && let Some(snapshot) = gate.read_snapshot.lock().unwrap().pop()
        {
            gate.owner.release_snapshot_for_test(&snapshot);
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
            state
                .exit_acquisitions
                .push((root, self.owner.acquisitions_for_test()));
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
        state
            .initial_acquisitions
            .get_or_insert_with(|| self.owner.acquisitions_for_test());
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
        let initial = state
            .initial_acquisitions
            .expect("the first read sampled its owner");
        for &(root, acquisitions) in &state.exit_acquisitions {
            assert_eq!(
                acquisitions, initial,
                "owner acquired during rooted read {root:?}"
            );
        }
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
        if let Some(snapshot) = self.gate.read_snapshot.lock().unwrap().pop() {
            self.gate.owner.release_snapshot_for_test(&snapshot);
        }
        *self
            .witness
            .native_reads
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = self.previous.take();
    }
}
