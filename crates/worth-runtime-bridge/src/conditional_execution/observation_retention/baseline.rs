use super::super::retention::{arc_charge, BridgeRetentionLedger, BridgeRetentionReservation};
use super::super::{BridgeConditionalDenial, BridgeConditionalSemanticObservation};
use std::sync::{Arc, Mutex};

/// Cloning this observation collection shares both immutable backing and custody.
#[derive(Clone, Debug, Default)]
pub(in crate::conditional_execution) struct BridgeRetainedObservations(Option<Arc<Backing>>);

#[derive(Debug)]
pub(super) struct Backing {
    observations: Vec<BridgeConditionalSemanticObservation>,
    _reservation: BridgeRetentionReservation,
}

impl BridgeRetainedObservations {
    pub(super) fn new(
        observations: Vec<BridgeConditionalSemanticObservation>,
        reservation: BridgeRetentionReservation,
    ) -> Self {
        Self(Some(Arc::new(Backing {
            observations,
            _reservation: reservation,
        })))
    }

    pub(in crate::conditional_execution) fn current(
        &self,
        ordinal: usize,
    ) -> Option<&worth_foundational::facade::ContractValidatedAspectArtifact> {
        self.binary_search_by_key(
            &ordinal,
            BridgeConditionalSemanticObservation::dependency_ordinal,
        )
        .ok()
        .and_then(|position| self[position].current())
    }
}

impl std::ops::Deref for BridgeRetainedObservations {
    type Target = [BridgeConditionalSemanticObservation];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().map_or(&[], |backing| &backing.observations)
    }
}

/// A session/intent owns this publication point. Its lock never spans callbacks.
#[derive(Debug)]
pub(in crate::conditional_execution) struct BridgeObservationBaselines {
    pub(in crate::conditional_execution) ledger: Arc<BridgeRetentionLedger>,
    current: Mutex<BridgeRetainedObservations>,
    _reservation: BridgeRetentionReservation,
}

impl BridgeObservationBaselines {
    pub(in crate::conditional_execution) fn new(
        ledger: &Arc<BridgeRetentionLedger>,
    ) -> Result<Arc<Self>, BridgeConditionalDenial> {
        let charge = arc_charge::<Self>().map_err(super::retention_denial)?;
        let reservation = ledger
            .reserve(0, 0, charge)
            .map_err(super::retention_denial)?;
        Ok(Arc::new(Self {
            ledger: Arc::clone(ledger),
            current: Mutex::new(Default::default()),
            _reservation: reservation,
        }))
    }

    pub(in crate::conditional_execution) fn snapshot(
        &self,
    ) -> Result<BridgeRetainedObservations, BridgeConditionalDenial> {
        self.current.lock().map(|value| value.clone()).map_err(|_| {
            super::retention_denial(super::super::retention::BridgeRetentionDenial::Quarantined)
        })
    }

    pub(in crate::conditional_execution) fn publish(
        &self,
        observations: BridgeRetainedObservations,
    ) {
        let displaced = std::mem::replace(
            &mut *self
                .current
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            observations,
        );
        drop(displaced);
    }
}
