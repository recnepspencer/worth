use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use worth_signal::facade::{ResourceAttemptId, ResourceRequestHandle};
use worth_store_physical_backend::{
    BackendQueueExecutionPlanBinding, MediaOperationIdentity, MediaOperationRole,
};

use super::super::{
    PhysicalSignalAspectBindingDigest, PhysicalSignalSettlementOutcome,
    PhysicalWorkCounterSnapshot, PhysicalWorkEffectFate, PhysicalWorkIdentity,
    PhysicalWorkRecoveryDisposition, SettledPhysicalWork,
};

mod counter_patch;

use counter_patch::PhysicalWorkCounterPatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWorkCausalRecord {
    identity: PhysicalWorkIdentity,
    operation: super::super::PhysicalWorkOperationFamily,
    signal_request: ResourceRequestHandle,
    signal_predecessor: Option<ResourceRequestHandle>,
    signal_attempt: ResourceAttemptId,
    signal_family: super::super::PhysicalWorkSignalFamily,
    signal_binding: PhysicalSignalAspectBindingDigest,
    scheduler_binding: BackendQueueExecutionPlanBinding,
    backend_operation: Option<MediaOperationIdentity>,
    backend_role: Option<MediaOperationRole>,
    effect_fate: PhysicalWorkEffectFate,
    recovery: PhysicalWorkRecoveryDisposition,
    derived_completion: Option<PhysicalSignalSettlementOutcome>,
    counters: PhysicalWorkCounterSnapshot,
}

pub(in crate::physical_runtime) struct PhysicalWorkCausalLedger {
    capacity: usize,
    state: Mutex<PhysicalWorkCausalLedgerState>,
    overflow: AtomicU64,
}

struct PhysicalWorkCausalLedgerState {
    base_counters: PhysicalWorkCounterSnapshot,
    latest_counters: PhysicalWorkCounterSnapshot,
    records: VecDeque<RetainedPhysicalWorkCausalRecord>,
}

#[derive(Debug)]
struct RetainedPhysicalWorkCausalRecord {
    identity: PhysicalWorkIdentity,
    operation: super::super::PhysicalWorkOperationFamily,
    signal_request: ResourceRequestHandle,
    signal_predecessor: Option<ResourceRequestHandle>,
    signal_attempt: ResourceAttemptId,
    signal_family: super::super::PhysicalWorkSignalFamily,
    signal_binding: PhysicalSignalAspectBindingDigest,
    scheduler_binding: BackendQueueExecutionPlanBinding,
    backend_operation: Option<MediaOperationIdentity>,
    backend_role: Option<MediaOperationRole>,
    effect_fate: PhysicalWorkEffectFate,
    recovery: PhysicalWorkRecoveryDisposition,
    derived_completion: Option<PhysicalSignalSettlementOutcome>,
    counter_patch: PhysicalWorkCounterPatch,
}

#[derive(Clone)]
pub struct PhysicalWorkCausalObservation {
    ledger: Arc<PhysicalWorkCausalLedger>,
}

impl PhysicalWorkCausalLedger {
    pub(in crate::physical_runtime) fn bounded(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            capacity,
            state: Mutex::new(PhysicalWorkCausalLedgerState {
                base_counters: PhysicalWorkCounterSnapshot::default(),
                latest_counters: PhysicalWorkCounterSnapshot::default(),
                records: VecDeque::new(),
            }),
            overflow: AtomicU64::new(0),
        })
    }

    pub(in crate::physical_runtime) fn record_settlement(
        &self,
        settled: &SettledPhysicalWork,
        counters: PhysicalWorkCounterSnapshot,
    ) {
        if self.capacity == 0 {
            self.record_overflow();
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let counter_patch = PhysicalWorkCounterPatch::between(state.latest_counters, counters);
        state.latest_counters = counters;
        if state.records.len() == self.capacity {
            let evicted = state
                .records
                .pop_front()
                .expect("a full causal ledger has a record to evict");
            evicted.counter_patch.apply_to(&mut state.base_counters);
            self.record_overflow();
        }
        state
            .records
            .push_back(RetainedPhysicalWorkCausalRecord::new(
                settled,
                counter_patch,
            ));
    }

    pub(in crate::physical_runtime) fn record_derived_completion(
        &self,
        identity: PhysicalWorkIdentity,
        outcome: PhysicalSignalSettlementOutcome,
    ) {
        if let Some(record) = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .records
            .iter_mut()
            .rev()
            .find(|record| record.identity == identity)
        {
            record.derived_completion = Some(outcome);
        }
    }

    fn record_overflow(&self) {
        let _ = {
            let update_counter = &self.overflow;
            let mut value = update_counter.load(Ordering::Relaxed);
            loop {
                let next_value = Some(value.saturating_add(1));
                let Some(updated_value) = next_value else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        };
    }
}

impl PhysicalWorkCausalObservation {
    pub(super) fn new(ledger: Arc<PhysicalWorkCausalLedger>) -> Self {
        Self { ledger }
    }

    pub fn records(&self) -> Box<[PhysicalWorkCausalRecord]> {
        let state = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut counters = state.base_counters;
        let mut records = Vec::with_capacity(state.records.len());
        for record in &state.records {
            record.counter_patch.apply_to(&mut counters);
            records.push(record.materialize(counters));
        }
        records.into_boxed_slice()
    }

    pub fn overflow(&self) -> u64 {
        self.ledger.overflow.load(Ordering::Acquire)
    }
}

impl RetainedPhysicalWorkCausalRecord {
    fn new(settled: &SettledPhysicalWork, counter_patch: PhysicalWorkCounterPatch) -> Self {
        Self {
            identity: settled.intent().identity(),
            operation: settled.intent().operation(),
            signal_request: settled.signal_request(),
            signal_predecessor: settled.signal_evidence().replaces,
            signal_attempt: settled.request_attempt(),
            signal_family: settled.signal_family(),
            signal_binding: settled.signal_binding(),
            scheduler_binding: settled.scheduler_binding(),
            backend_operation: settled
                .effect_identity()
                .map(|identity| identity.backend_operation()),
            backend_role: settled.evidence().backend_role(),
            effect_fate: settled.evidence().fate(),
            recovery: settled.recovery_disposition(),
            derived_completion: None,
            counter_patch,
        }
    }

    fn materialize(&self, counters: PhysicalWorkCounterSnapshot) -> PhysicalWorkCausalRecord {
        PhysicalWorkCausalRecord {
            identity: self.identity,
            operation: self.operation,
            signal_request: self.signal_request,
            signal_predecessor: self.signal_predecessor,
            signal_attempt: self.signal_attempt,
            signal_family: self.signal_family,
            signal_binding: self.signal_binding,
            scheduler_binding: self.scheduler_binding,
            backend_operation: self.backend_operation,
            backend_role: self.backend_role,
            effect_fate: self.effect_fate,
            recovery: self.recovery,
            derived_completion: self.derived_completion,
            counters,
        }
    }
}

impl PhysicalWorkCausalRecord {
    pub const fn identity(self) -> PhysicalWorkIdentity {
        self.identity
    }

    pub const fn operation(self) -> super::super::PhysicalWorkOperationFamily {
        self.operation
    }

    pub const fn signal_request(self) -> ResourceRequestHandle {
        self.signal_request
    }

    pub const fn signal_predecessor(self) -> Option<ResourceRequestHandle> {
        self.signal_predecessor
    }

    pub const fn signal_attempt(self) -> ResourceAttemptId {
        self.signal_attempt
    }

    pub const fn signal_family(self) -> super::super::PhysicalWorkSignalFamily {
        self.signal_family
    }

    pub const fn signal_binding(self) -> PhysicalSignalAspectBindingDigest {
        self.signal_binding
    }

    pub const fn scheduler_binding(self) -> BackendQueueExecutionPlanBinding {
        self.scheduler_binding
    }

    pub const fn backend_operation(self) -> Option<MediaOperationIdentity> {
        self.backend_operation
    }

    pub const fn backend_role(self) -> Option<MediaOperationRole> {
        self.backend_role
    }

    pub const fn effect_fate(self) -> PhysicalWorkEffectFate {
        self.effect_fate
    }

    pub const fn recovery(self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }

    pub const fn derived_completion(self) -> Option<PhysicalSignalSettlementOutcome> {
        self.derived_completion
    }

    pub const fn counters(self) -> PhysicalWorkCounterSnapshot {
        self.counters
    }
}
