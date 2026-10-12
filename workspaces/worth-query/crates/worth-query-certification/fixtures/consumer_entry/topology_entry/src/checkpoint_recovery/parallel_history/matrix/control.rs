//! Each installation owns its schedule, counters and failure receiver.
const HANG_GUARD: std::time::Duration = std::time::Duration::from_secs(60);
use super::{Partition, World};
use std::sync::{Condvar, Mutex};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Prepare,
    Compute,
    Complete,
}
#[derive(Default)]
pub struct State {
    pub armed: bool,
    pub failure: Option<(u64, Phase)>,
    pub entered: Vec<u64>,
    pub completed: Vec<u64>,
    pub kernel_entered: Vec<u64>,
    pub active: usize,
    pub peak: usize,
    pub witnesses: Vec<u64>,
    pub hang_guard_expired: bool,
}
pub struct Control {
    pub world: World,
    pub reuse: bool,
    pub overlap: bool,
    pub declaration: u64,
    pub declarations: Mutex<std::collections::BTreeMap<u64, Vec<u64>>>,
    pub state: Mutex<State>,
    wake: Condvar,
}
impl Control {
    pub fn new(world: World, reuse: bool, overlap: bool, declaration: u64) -> Self {
        Self {
            world,
            reuse,
            overlap,
            declaration,
            declarations: Mutex::default(),
            state: Mutex::new(State::default()),
            wake: Condvar::new(),
        }
    }
    pub fn input(&self, key: u64) -> (Vec<u64>, Vec<Partition>) {
        if key == 100 {
            let mut required = self
                .world
                .requested
                .iter()
                .map(|i| self.world.members[*i].key)
                .collect::<Vec<_>>();
            if self.declaration == 1 {
                required.reverse();
            }
            return (required, vec![]);
        }
        let member = self
            .world
            .members
            .iter()
            .find(|member| member.key == key)
            .unwrap();
        let mut upstream = member
            .upstream
            .iter()
            .map(|i| self.world.members[*i].key)
            .collect::<Vec<_>>();
        if self.declaration == 1 {
            upstream.reverse();
        }
        (upstream, member.partitions.clone())
    }
    pub fn declare_begin(&self, key: u64) {
        self.declarations.lock().unwrap().insert(key, Vec::new());
    }
    pub fn declare_edge(&self, key: u64, upstream: u64) {
        self.declarations
            .lock()
            .unwrap()
            .get_mut(&key)
            .unwrap()
            .push(upstream);
    }
    pub fn fails(&self, key: u64, phase: Phase) -> bool {
        let state = self.state.lock().unwrap();
        state.armed && state.failure == Some((key, phase))
    }
    pub fn completed(&self, key: u64) {
        let mut state = self.state.lock().unwrap();
        if state.armed {
            state.completed.push(key);
        }
    }
    pub fn dispatched(&self, key: u64) {
        let mut state = self.state.lock().unwrap();
        if state.armed {
            state.entered.push(key);
        }
    }
    pub fn enter_compute(&self, key: u64, part: u64, units: u64) -> u64 {
        let mut state = self.state.lock().unwrap();
        if !state.armed {
            return 0;
        }
        state.active += 1;
        state.peak = state.peak.max(state.active);
        if part == 0 {
            state.kernel_entered.push(key);
        }
        self.wake.notify_all();
        if self.overlap && part == 0 && state.witnesses.contains(&key) {
            let (next, hang_guard) = self
                .wake
                .wait_timeout_while(state, HANG_GUARD, |s| {
                    s.witnesses
                        .iter()
                        .any(|key| !s.kernel_entered.contains(key))
                })
                .unwrap();
            state = next;
            state.hang_guard_expired |= hang_guard.timed_out();
        }
        units
    }
    pub fn exit_compute(&self, _: u64, _: u64) {
        let mut state = self.state.lock().unwrap();
        if state.armed {
            state.active -= 1;
            self.wake.notify_all();
        }
    }
}
