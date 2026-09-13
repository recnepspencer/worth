use std::sync::{Arc, OnceLock};

use super::{PhysicalWorkIdentity, PhysicalWorkOperationFamily};

mod accounting;
mod causal;
pub(in crate::physical_runtime::work) use accounting::{
    family_index, pressure_index, terminal_stage_index, PhysicalWorkAccounting,
};
pub(super) use causal::PhysicalWorkCausalLedger;
pub use causal::{PhysicalWorkCausalObservation, PhysicalWorkCausalRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkTerminalStage {
    Declared,
    Blocked,
    Ready,
    Queued,
    Dispatched,
    Settling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkTerminalDisposition {
    ClosedBeforeReadiness,
    AbortedBeforeReadiness,
    DroppedBeforeReadiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkCounterStage {
    Declared,
    Blocked,
    Ready,
    Queued,
    Dispatched,
    Settling,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkPressureClass {
    Unscheduled,
    ForegroundPointRead,
    ForegroundRangeRead,
    ForegroundInteractiveRead,
    ForegroundInternalRead,
    ForegroundMutation,
    BackgroundCheckpoint,
    BackgroundScrub,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalWorkCounterSnapshot {
    by_family_and_pressure: [[[u64; 7]; 8]; 9],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWorkTerminalObservation {
    identity: PhysicalWorkIdentity,
    stage: PhysicalWorkTerminalStage,
    disposition: PhysicalWorkTerminalDisposition,
}

impl PhysicalWorkTerminalObservation {
    pub(super) const fn active(
        identity: PhysicalWorkIdentity,
        stage: PhysicalWorkTerminalStage,
        disposition: PhysicalWorkTerminalDisposition,
    ) -> Self {
        Self {
            identity,
            stage,
            disposition,
        }
    }

    pub const fn identity(self) -> PhysicalWorkIdentity {
        self.identity
    }

    pub const fn stage(self) -> PhysicalWorkTerminalStage {
        self.stage
    }

    pub const fn disposition(self) -> PhysicalWorkTerminalDisposition {
        self.disposition
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalWorkShutdownObservation {
    declared: u64,
    ready: u64,
    blocked: u64,
    queued: u64,
    dispatched: u64,
    settling: u64,
    terminal: Box<[PhysicalWorkTerminalObservation]>,
    residual: u64,
    unaccounted_terminal: u64,
    drain: Option<super::PhysicalWorkDrainObservation>,
    cancellation_candidates: Box<[super::PhysicalWorkConsumerHandle]>,
}

#[derive(Clone)]
pub struct PhysicalWorkObservation {
    terminal: Arc<OnceLock<PhysicalWorkShutdownObservation>>,
    causal: PhysicalWorkCausalObservation,
}

pub(super) struct PhysicalWorkObservationOwner {
    terminal: Arc<OnceLock<PhysicalWorkShutdownObservation>>,
    causal: Arc<PhysicalWorkCausalLedger>,
}

impl PhysicalWorkObservationOwner {
    pub(super) fn new(causal_capacity: usize) -> Self {
        Self {
            terminal: Arc::new(OnceLock::new()),
            causal: PhysicalWorkCausalLedger::bounded(causal_capacity),
        }
    }

    pub(super) fn handle(&self) -> PhysicalWorkObservation {
        PhysicalWorkObservation {
            terminal: Arc::clone(&self.terminal),
            causal: PhysicalWorkCausalObservation::new(Arc::clone(&self.causal)),
        }
    }

    pub(super) fn publish(&self, observation: PhysicalWorkShutdownObservation) {
        let _ = self.terminal.set(observation);
    }

    pub(super) fn causal(&self) -> &PhysicalWorkCausalLedger {
        &self.causal
    }
}

impl PhysicalWorkObservation {
    pub fn terminal(&self) -> Option<&PhysicalWorkShutdownObservation> {
        self.terminal.get()
    }

    pub const fn causal(&self) -> &PhysicalWorkCausalObservation {
        &self.causal
    }
}

impl PhysicalWorkShutdownObservation {
    pub(super) fn from_active(
        declared_total: u64,
        completed_terminal: u64,
        active: impl IntoIterator<
            Item = (
                PhysicalWorkIdentity,
                PhysicalWorkTerminalStage,
                Option<super::PhysicalWorkConsumerHandle>,
            ),
        >,
        disposition: PhysicalWorkTerminalDisposition,
    ) -> Self {
        let active = active.into_iter().collect::<Vec<_>>();
        let cancellation_candidates = active
            .iter()
            .filter_map(|(_, _, consumer)| *consumer)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let terminal: Box<[_]> = active
            .into_iter()
            .map(|(identity, stage, _)| {
                PhysicalWorkTerminalObservation::active(identity, stage, disposition)
            })
            .collect();
        let terminal_total = terminal.len() as u64;
        let mut ready = 0;
        let mut blocked = 0;
        let mut queued = 0;
        let mut dispatched = 0;
        let mut settling = 0;
        for observation in &terminal {
            match observation.stage {
                PhysicalWorkTerminalStage::Declared => {}
                PhysicalWorkTerminalStage::Blocked => blocked += 1,
                PhysicalWorkTerminalStage::Ready => ready += 1,
                PhysicalWorkTerminalStage::Queued => queued += 1,
                PhysicalWorkTerminalStage::Dispatched => dispatched += 1,
                PhysicalWorkTerminalStage::Settling => settling += 1,
            }
        }
        let accounted = completed_terminal.saturating_add(terminal_total);
        Self {
            declared: declared_total,
            ready,
            blocked,
            queued,
            dispatched,
            settling,
            terminal,
            residual: declared_total.saturating_sub(accounted),
            unaccounted_terminal: accounted.saturating_sub(declared_total),
            drain: None,
            cancellation_candidates,
        }
    }

    pub(super) fn with_drain(mut self, drain: super::PhysicalWorkDrainObservation) -> Self {
        self.drain = Some(drain);
        self
    }

    pub(super) fn with_additional_cancellation_candidates(
        mut self,
        additional: impl IntoIterator<Item = super::PhysicalWorkConsumerHandle>,
    ) -> Self {
        let mut candidates = self.cancellation_candidates.into_vec();
        candidates.extend(additional);
        candidates.sort_by_key(|consumer| consumer.identity().operation().get());
        candidates.dedup_by_key(|consumer| consumer.identity());
        self.cancellation_candidates = candidates.into_boxed_slice();
        self
    }

    pub const fn declared(&self) -> u64 {
        self.declared
    }

    pub const fn ready(&self) -> u64 {
        self.ready
    }

    pub const fn blocked(&self) -> u64 {
        self.blocked
    }

    pub const fn queued(&self) -> u64 {
        self.queued
    }

    pub const fn dispatched(&self) -> u64 {
        self.dispatched
    }

    pub const fn settling(&self) -> u64 {
        self.settling
    }

    pub const fn terminal(&self) -> &[PhysicalWorkTerminalObservation] {
        &self.terminal
    }

    pub const fn residual(&self) -> u64 {
        self.residual
    }

    pub const fn unaccounted_terminal(&self) -> u64 {
        self.unaccounted_terminal
    }

    pub fn drain(&self) -> &super::PhysicalWorkDrainObservation {
        self.drain
            .as_ref()
            .expect("physical shutdown owner always attaches drain evidence")
    }
}

impl PhysicalWorkCounterSnapshot {
    pub(super) const fn from_counts(by_family_and_pressure: [[[u64; 7]; 8]; 9]) -> Self {
        Self {
            by_family_and_pressure,
        }
    }

    pub const fn count(
        self,
        family: PhysicalWorkOperationFamily,
        stage: PhysicalWorkCounterStage,
    ) -> u64 {
        let family = family_index(family);
        let stage = counter_stage_index(stage);
        let mut total = 0;
        let mut pressure = 0;
        while pressure < 8 {
            total += self.by_family_and_pressure[family][pressure][stage];
            pressure += 1;
        }
        total
    }

    pub const fn count_under_pressure(
        self,
        family: PhysicalWorkOperationFamily,
        pressure: PhysicalWorkPressureClass,
        stage: PhysicalWorkCounterStage,
    ) -> u64 {
        self.by_family_and_pressure[family_index(family)][pressure_index(pressure)]
            [counter_stage_index(stage)]
    }

    pub fn total(self, stage: PhysicalWorkCounterStage) -> u64 {
        self.by_family_and_pressure
            .iter()
            .flat_map(|family| family.iter())
            .map(|pressure| pressure[counter_stage_index(stage)])
            .sum()
    }
}

const fn counter_stage_index(stage: PhysicalWorkCounterStage) -> usize {
    match stage {
        PhysicalWorkCounterStage::Declared => 0,
        PhysicalWorkCounterStage::Blocked => 1,
        PhysicalWorkCounterStage::Ready => 2,
        PhysicalWorkCounterStage::Queued => 3,
        PhysicalWorkCounterStage::Dispatched => 4,
        PhysicalWorkCounterStage::Settling => 5,
        PhysicalWorkCounterStage::Terminal => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PhysicalWorkCounterSnapshot, PhysicalWorkCounterStage, PhysicalWorkOperationFamily,
    };

    #[test]
    fn metadata_and_range_read_counters_have_distinct_family_buckets() {
        let mut counts = [[[0_u64; 7]; 8]; 9];
        counts[0][0][6] = 2;
        counts[1][0][6] = 3;
        let snapshot = PhysicalWorkCounterSnapshot::from_counts(counts);

        assert_eq!(
            snapshot.count(
                PhysicalWorkOperationFamily::ArtifactMetadataRead,
                PhysicalWorkCounterStage::Terminal,
            ),
            2
        );
        assert_eq!(
            snapshot.count(
                PhysicalWorkOperationFamily::ArtifactRangeRead,
                PhysicalWorkCounterStage::Terminal,
            ),
            3
        );
    }
}
