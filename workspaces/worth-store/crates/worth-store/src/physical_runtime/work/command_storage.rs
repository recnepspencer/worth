use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{PhysicalWorkIdentity, PhysicalWorkIntent, PhysicalWorkTerminalStage};

mod release;
mod signal_registration;
pub(super) use release::PhysicalCommandRelease;
pub(super) use signal_registration::PhysicalCommandSignalRegistration;

const COMMAND_SHARDS: usize = 32;

pub(super) struct PhysicalCommandArena {
    capacity: usize,
    declared: Box<[Mutex<HashMap<PhysicalWorkIdentity, PhysicalCommandEntry>>]>,
    #[cfg(feature = "certification-test-authority")]
    certification_shard_gate: Mutex<Option<super::CertificationPhysicalSubmissionPauseGate>>,
}

struct PhysicalCommandEntry {
    identity: PhysicalWorkIdentity,
    operation: super::PhysicalWorkOperationFamily,
    pressure: super::PhysicalWorkPressureClass,
    intent: Option<PhysicalWorkIntent>,
    consumer: Option<super::PhysicalWorkConsumerHandle>,
    signal_route: Option<super::PhysicalSignalAspectBindingDigest>,
    scope_members: usize,
    semantic_bytes: usize,
    stage: PhysicalWorkTerminalStage,
    retry_pending: bool,
    release: Arc<PhysicalCommandRelease>,
}

pub(super) struct PhysicalCommandAdmission {
    pub(super) intent: PhysicalWorkIntent,
    pub(super) release: Arc<PhysicalCommandRelease>,
}

pub(super) struct ReleasedPhysicalCommand {
    pub(super) operation: super::PhysicalWorkOperationFamily,
    pub(super) pressure: super::PhysicalWorkPressureClass,
    pub(super) scope_members: usize,
    pub(super) semantic_bytes: usize,
    pub(super) stage: PhysicalWorkTerminalStage,
    pub(super) consumer_cancelled: bool,
    pub(super) consumer: Option<super::PhysicalWorkConsumerHandle>,
    pub(super) retry_pending: bool,
}

pub(super) struct DrainedPhysicalCommand {
    pub(super) identity: PhysicalWorkIdentity,
    pub(super) operation: super::PhysicalWorkOperationFamily,
    pub(super) pressure: super::PhysicalWorkPressureClass,
    pub(super) scope_members: usize,
    pub(super) semantic_bytes: usize,
    pub(super) stage: PhysicalWorkTerminalStage,
    pub(super) release: Arc<PhysicalCommandRelease>,
    pub(super) consumer: Option<super::PhysicalWorkConsumerHandle>,
}

impl PhysicalCommandArena {
    pub(super) fn bounded(capacity: usize) -> Self {
        Self {
            capacity,
            declared: (0..COMMAND_SHARDS)
                .map(|_| Mutex::new(HashMap::new()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            #[cfg(feature = "certification-test-authority")]
            certification_shard_gate: Mutex::new(None),
        }
    }

    pub(super) const fn capacity(&self) -> usize {
        self.capacity
    }

    pub(super) fn push_declared(
        &self,
        intent: PhysicalWorkIntent,
        scope_members: usize,
        semantic_bytes: usize,
    ) {
        let shard = intent.identity().operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        #[cfg(feature = "certification-test-authority")]
        self.pause_after_shard_lock();
        let identity = intent.identity();
        let previous = declared.insert(
            identity,
            PhysicalCommandEntry {
                identity: intent.identity(),
                operation: intent.operation(),
                pressure: super::PhysicalWorkPressureClass::Unscheduled,
                intent: Some(intent),
                consumer: None,
                signal_route: None,
                scope_members,
                semantic_bytes,
                stage: PhysicalWorkTerminalStage::Declared,
                retry_pending: false,
                release: Arc::new(PhysicalCommandRelease::new()),
            },
        );
        debug_assert!(previous.is_none(), "owner identities are monotonic");
    }

    pub(super) fn admit_declared(
        &self,
        identity: PhysicalWorkIdentity,
    ) -> Option<PhysicalCommandAdmission> {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = declared.get_mut(&identity)?;
        let intent = entry.intent.take()?;
        Some(PhysicalCommandAdmission {
            intent,
            release: Arc::clone(&entry.release),
        })
    }

    pub(super) fn mark_stage(
        &self,
        identity: PhysicalWorkIdentity,
        stage: PhysicalWorkTerminalStage,
    ) {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = declared.get_mut(&identity) {
            entry.stage = stage;
        }
    }

    pub(super) fn begin_dispatch(&self, identity: PhysicalWorkIdentity) -> bool {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(entry) = declared.get_mut(&identity) else {
            return false;
        };
        if entry.release.is_cancelled() {
            return false;
        }
        entry.stage = PhysicalWorkTerminalStage::Dispatched;
        entry.retry_pending = false;
        true
    }

    pub(super) fn mark_retry_pending(&self, identity: PhysicalWorkIdentity) {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = declared.get_mut(&identity) {
            entry.stage = PhysicalWorkTerminalStage::Settling;
            entry.retry_pending = true;
        }
    }

    pub(super) fn mark_pressure(
        &self,
        identity: PhysicalWorkIdentity,
        pressure: super::PhysicalWorkPressureClass,
    ) -> bool {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(entry) = declared.get_mut(&identity) else {
            return false;
        };
        if !matches!(
            entry.stage,
            PhysicalWorkTerminalStage::Ready | PhysicalWorkTerminalStage::Queued
        ) {
            return false;
        }
        entry.pressure = pressure;
        true
    }

    pub(super) fn release(
        &self,
        identity: PhysicalWorkIdentity,
    ) -> Option<ReleasedPhysicalCommand> {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = declared.remove(&identity)?;
        Some(ReleasedPhysicalCommand {
            operation: entry.operation,
            pressure: entry.pressure,
            scope_members: entry.scope_members,
            semantic_bytes: entry.semantic_bytes,
            stage: entry.stage,
            consumer_cancelled: entry.release.consumer_cancelled(),
            consumer: entry.consumer,
            retry_pending: entry.retry_pending,
        })
    }

    pub(super) fn cancel_before_dispatch(
        &self,
        identity: PhysicalWorkIdentity,
    ) -> Option<ReleasedPhysicalCommand> {
        let shard = identity.operation().get() as usize % self.declared.len();
        let mut declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let cancellable = declared.get(&identity).is_some_and(|entry| {
            !matches!(
                entry.stage,
                PhysicalWorkTerminalStage::Dispatched | PhysicalWorkTerminalStage::Settling
            )
        });
        if !cancellable {
            return None;
        }
        let entry = declared.get(&identity)?;
        entry.release.cancel();
        if !entry.release.claim_release() {
            return None;
        }
        let entry = declared
            .remove(&identity)
            .expect("release ownership was claimed for a locked command entry");
        Some(ReleasedPhysicalCommand {
            operation: entry.operation,
            pressure: entry.pressure,
            scope_members: entry.scope_members,
            semantic_bytes: entry.semantic_bytes,
            stage: entry.stage,
            consumer_cancelled: true,
            consumer: entry.consumer,
            retry_pending: entry.retry_pending,
        })
    }

    pub(super) fn mark_consumer_cancelled(&self, identity: PhysicalWorkIdentity) -> bool {
        let shard = identity.operation().get() as usize % self.declared.len();
        let declared = self.declared[shard]
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(entry) = declared.get(&identity) else {
            return false;
        };
        entry.release.mark_consumer_cancelled();
        true
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn pause_after_shard_lock_for_certification(
        &self,
    ) -> super::CertificationPhysicalSubmissionPauseGate {
        let gate = super::CertificationPhysicalSubmissionPauseGate::new();
        *self
            .certification_shard_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(gate.clone());
        gate
    }

    #[cfg(feature = "certification-test-authority")]
    fn pause_after_shard_lock(&self) {
        let gate = self
            .certification_shard_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(gate) = gate {
            gate.arrive_and_wait();
        }
    }

    pub(super) fn drain_before_dispatch(&self) -> Vec<DrainedPhysicalCommand> {
        let mut drained = Vec::new();
        for shard in &self.declared {
            let mut entries = shard
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let identities = entries
                .iter()
                .filter_map(|(identity, entry)| {
                    (entry.retry_pending
                        || !matches!(
                            entry.stage,
                            PhysicalWorkTerminalStage::Dispatched
                                | PhysicalWorkTerminalStage::Settling
                        ))
                    .then_some(*identity)
                })
                .collect::<Vec<_>>();
            drained.extend(identities.into_iter().filter_map(|identity| {
                entries
                    .remove(&identity)
                    .map(|entry| DrainedPhysicalCommand {
                        identity: entry.identity,
                        operation: entry.operation,
                        pressure: entry.pressure,
                        scope_members: entry.scope_members,
                        semantic_bytes: entry.semantic_bytes,
                        stage: entry.stage,
                        release: entry.release,
                        consumer: entry.consumer,
                    })
            }));
        }
        drained.sort_by_key(|entry| entry.identity.operation().get());
        drained
    }

    pub(super) fn active_stages(&self) -> Vec<(PhysicalWorkIdentity, PhysicalWorkTerminalStage)> {
        let mut active = Vec::new();
        for shard in &self.declared {
            active.extend(
                shard
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .values()
                    .map(|entry| (entry.identity, entry.stage)),
            );
        }
        active.sort_by_key(|(identity, _)| identity.operation().get());
        active
    }

    pub(super) fn active_counters(
        &self,
        terminal_by_family_and_pressure: [[u64; 8]; 9],
    ) -> super::PhysicalWorkCounterSnapshot {
        let mut counts = [[[0_u64; 7]; 8]; 9];
        for shard in &self.declared {
            let entries = shard
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for entry in entries.values() {
                counts[super::observation::family_index(entry.operation)]
                    [super::observation::pressure_index(entry.pressure)]
                    [super::observation::terminal_stage_index(entry.stage)] += 1;
            }
        }
        for (family, pressures) in terminal_by_family_and_pressure.into_iter().enumerate() {
            for (pressure, terminal) in pressures.into_iter().enumerate() {
                counts[family][pressure][6] = terminal;
            }
        }
        super::PhysicalWorkCounterSnapshot::from_counts(counts)
    }
}
