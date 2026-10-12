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
    pub active: usize,
    pub peak: usize,
    pub witnesses: Vec<u64>,
    pub hang_guard_expired: bool,
}
pub struct Control {
    pub world: World,
    pub reuse: bool,
    pub overlap: bool,
    pub state: Mutex<State>,
    wake: Condvar,
}
impl Control {
    pub fn new(world: World, reuse: bool, overlap: bool) -> Self {
        Self {
            world,
            reuse,
            overlap,
            state: Mutex::new(State::default()),
            wake: Condvar::new(),
        }
    }
    pub fn input(&self, key: u64) -> (Vec<u64>, Vec<Partition>) {
        if key == 100 {
            return (
                self.world
                    .requested
                    .iter()
                    .map(|i| self.world.members[*i].key)
                    .collect(),
                vec![],
            );
        }
        let member = self
            .world
            .members
            .iter()
            .find(|member| member.key == key)
            .unwrap();
        (
            member
                .upstream
                .iter()
                .map(|i| self.world.members[*i].key)
                .collect(),
            member.partitions.clone(),
        )
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
    pub fn enter_compute(&self, key: u64, part: u64, units: u64) -> u64 {
        let mut state = self.state.lock().unwrap();
        if !state.armed {
            return 0;
        }
        state.active += 1;
        state.peak = state.peak.max(state.active);
        if part == 0 {
            state.entered.push(key);
        }
        self.wake.notify_all();
        if self.overlap && part == 0 && state.witnesses.contains(&key) {
            let (next, hang_guard) = self
                .wake
                .wait_timeout_while(state, HANG_GUARD, |s| {
                    s.witnesses.iter().any(|key| !s.entered.contains(key))
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
