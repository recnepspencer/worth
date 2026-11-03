use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::BridgeManagedExecutionIntentIdentity;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BridgeExecutionBasisReservationKey {
    intent: BridgeManagedExecutionIntentIdentity,
}

impl BridgeExecutionBasisReservationKey {
    pub(crate) fn new(intent: BridgeManagedExecutionIntentIdentity) -> Self {
        Self { intent }
    }
}

#[derive(Debug, Default)]
pub(crate) struct BridgeExecutionBasisReservationRegistry {
    reserved: Mutex<HashSet<BridgeExecutionBasisReservationKey>>,
}

impl BridgeExecutionBasisReservationRegistry {
    pub(crate) fn try_home_stop_guard(
        &self,
    ) -> Result<
        std::sync::MutexGuard<'_, HashSet<BridgeExecutionBasisReservationKey>>,
        crate::facade::BridgeHomeRetirementDenial,
    > {
        let guard = self
            .reserved
            .try_lock()
            .map_err(|_| crate::facade::BridgeHomeRetirementDenial::Busy)?;
        if guard.is_empty() {
            Ok(guard)
        } else {
            Err(crate::facade::BridgeHomeRetirementDenial::Busy)
        }
    }
    pub(crate) fn reserve(
        self: &Arc<Self>,
        key: BridgeExecutionBasisReservationKey,
    ) -> Option<BridgeExecutionBasisReservation> {
        let mut reserved = self
            .reserved
            .lock()
            .expect("bridge execution-basis reservation registry must remain available");
        if !reserved.insert(key.clone()) {
            return None;
        }
        Some(BridgeExecutionBasisReservation {
            registry: Arc::clone(self),
            key: Some(key.clone()),
        })
    }

    fn release(&self, key: BridgeExecutionBasisReservationKey) -> bool {
        self.reserved
            .lock()
            .expect("bridge execution-basis reservation registry must remain available")
            .remove(&key)
    }

    pub(super) fn contains(&self, key: &BridgeExecutionBasisReservationKey) -> bool {
        self.reserved
            .lock()
            .expect("bridge execution-basis reservation registry must remain available")
            .contains(key)
    }
}

pub(crate) struct BridgeExecutionBasisReservation {
    registry: Arc<BridgeExecutionBasisReservationRegistry>,
    key: Option<BridgeExecutionBasisReservationKey>,
}

impl BridgeExecutionBasisReservation {
    pub(crate) fn release(mut self) -> bool {
        self.key
            .take()
            .is_some_and(|key| self.registry.release(key))
    }

    pub(super) fn observer_parts(
        &self,
    ) -> (
        Arc<BridgeExecutionBasisReservationRegistry>,
        BridgeExecutionBasisReservationKey,
    ) {
        (
            Arc::clone(&self.registry),
            self.key
                .as_ref()
                .expect("active reservation retains its exact key")
                .clone(),
        )
    }
}

impl Drop for BridgeExecutionBasisReservation {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            let _ = self.registry.release(key);
        }
    }
}
