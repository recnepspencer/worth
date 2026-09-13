use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

use super::WorthQueryOutputDemandSettlement;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingOutputDelivery {
    pub(in crate::domain_computation::primary_graph) receipt:
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) change:
        crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandKey {
    producer: String,
    source: [u8; 32],
}

impl WorthQueryOutputDemandKey {
    pub(in crate::domain_computation::primary_graph) fn new(
        producer: String,
        source: [u8; 32],
    ) -> Self {
        Self { producer, source }
    }
}

#[derive(Clone)]
pub struct WorthQueryOutputDemandNotifications {
    wake: Arc<DemandWake>,
}

impl WorthQueryOutputDemandNotifications {
    pub fn generation(&self) -> u64 {
        *self
            .wake
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn wait_after(&self, observed_generation: u64) -> u64 {
        let generation = self
            .wake
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let generation = self
            .wake
            .changed
            .wait_while(generation, |generation| *generation <= observed_generation)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *generation
    }
}

struct DemandWake {
    generation: Mutex<u64>,
    changed: Condvar,
}

impl DemandWake {
    fn notify(&self) {
        let mut generation = self
            .generation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *generation = generation.saturating_add(1);
        self.changed.notify_all();
    }
}

enum DemandState {
    Admitted,
    Scheduling,
    Scheduled,
    Running,
    Delivering,
    DeliveryPending(Option<WorthQueryPendingOutputDelivery>),
    Settled(Arc<WorthQueryOutputDemandSettlement>),
    Failed(WorthQueryOutputDemandDenial),
}

struct DemandRecord {
    interests: usize,
    state: DemandState,
    wake: Arc<DemandWake>,
}

#[derive(Default)]
struct DemandRegistryState {
    records: HashMap<WorthQueryOutputDemandKey, DemandRecord>,
}

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandRegistry {
    state: Arc<Mutex<DemandRegistryState>>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryOutputDemandInterest {
    key: WorthQueryOutputDemandKey,
    notifications: WorthQueryOutputDemandNotifications,
    owner: WorthQueryOutputDemandRegistry,
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryOutputDemandAdvanceAdmission {
    Schedule,
    Execute,
    Deliver(WorthQueryPendingOutputDelivery),
    Pending,
    Settled(Arc<WorthQueryOutputDemandSettlement>),
    Failed(WorthQueryOutputDemandDenial),
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn admit(
        &self,
        key: WorthQueryOutputDemandKey,
    ) -> WorthQueryOutputDemandInterest {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .entry(key.clone())
            .or_insert_with(|| DemandRecord {
                interests: 0,
                state: DemandState::Admitted,
                wake: Arc::new(DemandWake {
                    generation: Mutex::new(0),
                    changed: Condvar::new(),
                }),
            });
        record.interests = record.interests.saturating_add(1);
        WorthQueryOutputDemandInterest {
            key,
            notifications: WorthQueryOutputDemandNotifications {
                wake: Arc::clone(&record.wake),
            },
            owner: self.clone(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn begin(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> WorthQueryOutputDemandAdvanceAdmission {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("live demand interest retains its owner record");
        if let DemandState::DeliveryPending(pending) = &mut record.state {
            let pending = pending
                .take()
                .expect("pending delivery remains present outside delivery custody");
            record.state = DemandState::Delivering;
            return WorthQueryOutputDemandAdvanceAdmission::Deliver(pending);
        }
        match &record.state {
            DemandState::Admitted => {
                record.state = DemandState::Scheduling;
                WorthQueryOutputDemandAdvanceAdmission::Schedule
            }
            DemandState::Scheduled => {
                record.state = DemandState::Running;
                WorthQueryOutputDemandAdvanceAdmission::Execute
            }
            DemandState::Scheduling | DemandState::Running | DemandState::Delivering => {
                WorthQueryOutputDemandAdvanceAdmission::Pending
            }
            DemandState::DeliveryPending(_) => unreachable!("pending delivery handled above"),
            DemandState::Settled(receipt) => {
                WorthQueryOutputDemandAdvanceAdmission::Settled(receipt.clone())
            }
            DemandState::Failed(denial) => {
                WorthQueryOutputDemandAdvanceAdmission::Failed(denial.clone())
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn finish_scheduling(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        result: &Result<(), WorthQueryOutputDemandDenial>,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("scheduled demand retains its owner record");
        record.state = match result {
            Ok(()) => DemandState::Scheduled,
            Err(denial) => DemandState::Failed(denial.clone()),
        };
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn finish(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        result: &Result<Arc<WorthQueryOutputDemandSettlement>, WorthQueryOutputDemandDenial>,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("executing demand retains its owner record");
        record.state = match result {
            Ok(receipt) => DemandState::Settled(receipt.clone()),
            Err(denial) => DemandState::Failed(denial.clone()),
        };
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn finish_delivery_pending(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        pending: WorthQueryPendingOutputDelivery,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("pending delivery retains its owner record");
        record.state = DemandState::DeliveryPending(Some(pending));
        record.wake.notify();
    }

    pub(in crate::domain_computation::primary_graph) fn relinquish_execution(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("executing demand retains its owner record");
        if matches!(record.state, DemandState::Scheduling | DemandState::Running) {
            record.state = if matches!(record.state, DemandState::Scheduling) {
                DemandState::Admitted
            } else {
                DemandState::Scheduled
            };
            record.wake.notify();
        }
    }

    pub(in crate::domain_computation::primary_graph) fn release(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove = state.records.get_mut(&interest.key).is_some_and(|record| {
            record.interests = record.interests.saturating_sub(1);
            record.interests == 0
        });
        if remove {
            state.records.remove(&interest.key);
        }
    }
}

impl Drop for WorthQueryOutputDemandInterest {
    fn drop(&mut self) {
        self.owner.release(self);
    }
}

impl WorthQueryOutputDemandInterest {
    pub(in crate::domain_computation::primary_graph) fn notifications(
        &self,
    ) -> WorthQueryOutputDemandNotifications {
        self.notifications.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_owner_issued_interest_releases_its_registry_record() {
        let registry = WorthQueryOutputDemandRegistry::default();
        let interest = registry.admit(WorthQueryOutputDemandKey::new(
            "producer".to_owned(),
            [7; 32],
        ));
        assert_eq!(registry.state.lock().unwrap().records.len(), 1);
        drop(interest);
        assert!(registry.state.lock().unwrap().records.is_empty());
    }
}
