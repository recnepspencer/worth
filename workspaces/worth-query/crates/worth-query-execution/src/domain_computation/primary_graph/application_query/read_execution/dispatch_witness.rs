//! Carried test probes: isolated per owner thread, bounded on every wait.
use std::{
    cell::RefCell,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};
use worth_relational::facade::identity::EntityId;
thread_local! { static ACTIVE: RefCell<Option<Arc<DispatchWitness>>> = const { RefCell::new(None) }; }
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct DispatchWitness {
    state: Mutex<State>,
    finished: Condvar,
}
#[derive(Default)]
struct State {
    entries: Vec<EntityId>,
    completions: Vec<EntityId>,
    later: Option<EntityId>,
    watchdog_failed: bool,
    applied: Vec<(EntityId, String)>,
    interrupt: Option<Interrupt>,
    expected_point: Option<(EntityId, Vec<EntityId>)>,
    point: Option<InterruptionPoint>,
    costs: std::collections::BTreeMap<EntityId, u64>,
    map_charge: u64,
    map_prefix: usize,
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct InterruptionPoint {
    pub(in crate::domain_computation::primary_graph) root: EntityId,
    pub(in crate::domain_computation::primary_graph) completed: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph) fresh: bool,
}
pub(in crate::domain_computation::primary_graph) enum Interrupt {
    Cancel {
        root: EntityId,
        earlier: Vec<EntityId>,
        source: worth_execution::CancellationSource,
    },
    Deadline {
        root: EntityId,
        earlier: Vec<EntityId>,
        deadline: Instant,
    },
    QueryDeadline {
        root: EntityId,
        earlier: Vec<EntityId>,
        deadline: Instant,
    },
    QueryCancel {
        root: EntityId,
        earlier: Vec<EntityId>,
        source:
            worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource,
    },
}
pub(in crate::domain_computation::primary_graph) struct WitnessScope(Option<Arc<DispatchWitness>>);
impl Drop for WitnessScope {
    fn drop(&mut self) {
        ACTIVE.set(self.0.take());
    }
}
impl DispatchWitness {
    pub(in crate::domain_computation::primary_graph) fn install(
        later: Option<EntityId>,
    ) -> (Arc<Self>, WitnessScope) {
        let witness = Arc::new(Self::default());
        witness.state.lock().unwrap().later = later;
        let previous = ACTIVE.replace(Some(Arc::clone(&witness)));
        (witness, WitnessScope(previous))
    }
    pub(in crate::domain_computation::primary_graph) fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.entries.clear();
        state.completions.clear();
        state.applied.clear();
        state.costs.clear();
        state.map_charge = 0;
        state.map_prefix = 0;
        state.watchdog_failed = false;
        state.point = None;
    }
    pub(in crate::domain_computation::primary_graph) fn interrupt(&self, interruption: Interrupt) {
        let (root, earlier) = match &interruption {
            Interrupt::Cancel { root, earlier, .. }
            | Interrupt::Deadline { root, earlier, .. }
            | Interrupt::QueryCancel { root, earlier, .. }
            | Interrupt::QueryDeadline { root, earlier, .. } => (*root, earlier.clone()),
        };
        let mut state = self.state.lock().unwrap();
        state.expected_point = Some((root, earlier));
        state.interrupt = Some(interruption);
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn carried(
    ) -> Option<Arc<Self>> {
        ACTIVE.with(|active| active.borrow().clone())
    }
    pub(super) fn enter(&self, root: EntityId) {
        self.state.lock().unwrap().entries.push(root);
    }
    pub(super) fn charged(&self, root: EntityId, units: u64) {
        *self.state.lock().unwrap().costs.entry(root).or_default() += units;
    }
    pub(super) fn before_second(&self, root: EntityId) {
        let mut state = self.state.lock().unwrap();
        let earlier = match &state.interrupt {
            Some(
                Interrupt::Cancel {
                    root: selected,
                    earlier,
                    ..
                }
                | Interrupt::Deadline {
                    root: selected,
                    earlier,
                    ..
                }
                | Interrupt::QueryCancel {
                    root: selected,
                    earlier,
                    ..
                }
                | Interrupt::QueryDeadline {
                    root: selected,
                    earlier,
                    ..
                },
            ) if *selected == root => earlier.clone(),
            None | Some(_) => return,
        };
        // Failure watchdog only. A passing wait ends on actual completion,
        // independently of elapsed throughput. The test thread reports failure.
        let (guard, timeout) = self
            .finished
            .wait_timeout_while(state, Duration::from_secs(60), |state| {
                earlier.iter().any(|root| !state.completions.contains(root))
            })
            .unwrap();
        state = guard;
        state.watchdog_failed |= timeout.timed_out();
        let interruption = state.interrupt.take().unwrap();
        let fresh = match &interruption {
            Interrupt::Cancel { source, .. } => !source.is_cancelled(),
            Interrupt::QueryCancel { source, .. } => !source.token().is_cancelled(),
            Interrupt::Deadline { deadline, .. } | Interrupt::QueryDeadline { deadline, .. } => {
                Instant::now() < *deadline
            }
        };
        state.point = Some(InterruptionPoint {
            root,
            completed: state.completions.clone(),
            fresh,
        });
        match interruption {
            Interrupt::Cancel { source, .. } => source.cancel(),
            Interrupt::QueryCancel { source, .. } => source.cancel(),
            Interrupt::Deadline { deadline, .. } | Interrupt::QueryDeadline { deadline, .. } => {
                // This is the deadline event, not a speed assertion. The next
                // request checkpoint must see it elapsed on any machine.
                while Instant::now() < deadline {
                    let duration = deadline.saturating_duration_since(Instant::now());
                    state = self.finished.wait_timeout(state, duration).unwrap().0;
                }
            }
        }
    }
    pub(super) fn complete(&self, root: EntityId) {
        let mut state = self.state.lock().unwrap();
        if state.later.is_some_and(|later| later != root) {
            let (guard, timeout) = self
                .finished
                .wait_timeout_while(state, Duration::from_secs(60), |state| {
                    state
                        .later
                        .is_some_and(|later| !state.completions.contains(&later))
                })
                .unwrap();
            state = guard;
            state.watchdog_failed |= timeout.timed_out();
        }
        state.completions.push(root);
        self.finished.notify_all();
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn applied(
        root: EntityId,
        dependencies: &impl std::fmt::Debug,
    ) {
        if let Some(witness) = Self::carried() {
            witness
                .state
                .lock()
                .unwrap()
                .applied
                .push((root, format!("{dependencies:?}")));
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn settled(
        charge: u64,
        prefix: usize,
    ) {
        if let Some(witness) = Self::carried() {
            let mut state = witness.state.lock().unwrap();
            state.map_charge = charge;
            state.map_prefix = prefix;
        }
    }
    pub(in crate::domain_computation::primary_graph) fn assert_interruption_point(&self) {
        let state = self.state.lock().unwrap();
        if let Some((root, earlier)) = &state.expected_point {
            let point = state.point.as_ref().expect(
                "selected root never reached its interruption point; interruption arrived before dispatch",
            );
            assert_eq!(
                point.root, *root,
                "the selected root must trigger interruption"
            );
            assert!(
                earlier.iter().all(|root| point.completed.contains(root)),
                "interruption preceded required completions: {point:?}; required: {earlier:?}"
            );
            assert!(
                point.fresh,
                "interruption already present at selected rendezvous: {point:?}"
            );
            assert!(
                !state.watchdog_failed,
                "predecessors missed the interruption rendezvous"
            );
        }
    }
    pub(in crate::domain_computation::primary_graph) fn interruption_point(
        &self,
    ) -> Option<InterruptionPoint> {
        self.state.lock().unwrap().point.clone()
    }
    pub(in crate::domain_computation::primary_graph) fn observations(
        &self,
    ) -> (Vec<EntityId>, Vec<EntityId>) {
        let state = self.state.lock().unwrap();
        assert!(
            !state.watchdog_failed,
            "an independent root did not reach the bounded rendezvous"
        );
        (state.entries.clone(), state.completions.clone())
    }
    pub(in crate::domain_computation::primary_graph) fn dependencies(
        &self,
    ) -> Vec<(EntityId, String)> {
        self.state.lock().unwrap().applied.clone()
    }
    pub(in crate::domain_computation::primary_graph) fn map_observation(
        &self,
    ) -> (u64, usize, std::collections::BTreeMap<EntityId, u64>) {
        let state = self.state.lock().unwrap();
        (state.map_charge, state.map_prefix, state.costs.clone())
    }
}
