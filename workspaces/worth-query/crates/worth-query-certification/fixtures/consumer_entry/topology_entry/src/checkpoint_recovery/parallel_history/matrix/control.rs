//! Each installation owns its schedule, counters and failure receiver.
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
    pub kernel_active: usize,
    pub kernel_peak: usize,
    pub member_rendezvous: Vec<Vec<u64>>,
    pub wave_peaks: Vec<usize>,
    pub witnesses: Vec<u64>,
}
pub struct NestedMeter {
    pub member: u64,
    pub cost: Option<(u128, u128)>,
}
pub struct Control {
    pub world: World,
    pub reuse: bool,
    pub overlap: bool,
    pub declaration: u64,
    pub nested: bool,
    pub nested_meters: Mutex<Vec<NestedMeter>>,
    pub partition_declarations: Mutex<Vec<(u64, Vec<u64>)>>,
    pub declarations: Mutex<std::collections::BTreeMap<u64, Vec<u64>>>,
    pub state: Mutex<State>,
    wake: Condvar,
}
impl Control {
    pub fn new(world: World, reuse: bool, overlap: bool, declaration: u64, nested: bool) -> Self {
        Self {
            world,
            reuse,
            overlap,
            declaration,
            nested,
            nested_meters: Mutex::default(),
            partition_declarations: Mutex::default(),
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
            state.active += 1;
            state.peak = state.peak.max(state.active);
            // Fill the admitted member slots before any one member can enter
            // a nested map and let the scheduler reuse a physical worker.
            if let Some(wave) = state
                .member_rendezvous
                .iter()
                .position(|keys| keys.contains(&key))
            {
                let keys = state.member_rendezvous[wave].clone();
                let in_flight = keys
                    .iter()
                    .filter(|key| state.entered.contains(key))
                    .count();
                state.wave_peaks[wave] = state.wave_peaks[wave].max(in_flight);
                self.wake.notify_all();
                drop(
                    self.wake
                        .wait_while(state, |state| {
                            keys.iter().any(|key| !state.entered.contains(key))
                        })
                        .unwrap(),
                );
            }
        }
    }
    pub fn nested_meter(&self, member: u64, cost: Option<(u128, u128)>) {
        self.nested_meters
            .lock()
            .unwrap()
            .push(NestedMeter { member, cost });
    }
    pub fn declared_partitions(&self, key: u64, parts: Vec<u64>) {
        if self.state.lock().unwrap().armed {
            self.partition_declarations
                .lock()
                .unwrap()
                .push((key, parts));
        }
    }
    pub fn enter_compute(&self, key: u64, part: u64, units: u64) -> u64 {
        let mut state = self.state.lock().unwrap();
        if !state.armed {
            return 0;
        }
        state.kernel_active += 1;
        state.kernel_peak = state.kernel_peak.max(state.kernel_active);
        if part == 0 {
            state.kernel_entered.push(key);
        }
        self.wake.notify_all();
        if self.overlap && part == 0 && state.witnesses.contains(&key) {
            drop(
                self.wake
                    .wait_while(state, |s| {
                        s.witnesses
                            .iter()
                            .any(|key| !s.kernel_entered.contains(key))
                    })
                    .unwrap(),
            );
        }
        units
    }
    pub fn exit_compute(&self) {
        let mut state = self.state.lock().unwrap();
        if state.armed {
            state.kernel_active -= 1;
        }
    }
    pub fn exit_member(&self) {
        let mut state = self.state.lock().unwrap();
        if state.armed {
            state.active -= 1;
            self.wake.notify_all();
        }
    }
}

pub struct KernelFlight<'a>(pub &'a Control);
impl Drop for KernelFlight<'_> {
    fn drop(&mut self) {
        self.0.exit_compute();
    }
}
