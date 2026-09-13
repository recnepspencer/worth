use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::retention::component_obligation::{IssuedComponentPinPair, RetentionControlSurface};
use crate::retention::unique_component_pin::{
    ComponentBasisLeaseIdentity, ComponentBasisPinClaim, ExactComponentBasisKey,
    ExactComponentPinRequest,
};
use crate::retention::ComponentBasisDependencyClass;

use super::super::RetentionObligationDenial;
use super::{PinFlight, RuntimeWorldRetentionOwner};

mod reservation;
mod settlement;

#[derive(Debug)]
enum PairReservation<'a> {
    Ready(ComponentBasisPinClaim),
    Joined {
        request: ExactComponentPinRequest<'a>,
        key: ExactComponentBasisKey,
        dependency: ComponentBasisDependencyClass,
        flight: Arc<PinFlight>,
    },
    Owned(OwnedReservation<'a>),
}

#[derive(Debug)]
struct OwnedReservation<'a> {
    request: ExactComponentPinRequest<'a>,
    key: ExactComponentBasisKey,
    dependency: ComponentBasisDependencyClass,
    lease_identity: ComponentBasisLeaseIdentity,
    flight: Arc<PinFlight>,
    new_slot: bool,
}

struct PairReservationSet<'a> {
    basis: &'a AdmittedCompositeRuntimeWorldBasis,
    dependency: ComponentBasisDependencyClass,
    control: Arc<dyn RetentionControlSurface>,
    relational: PairReservation<'a>,
    signal: PairReservation<'a>,
}

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn issue_pair(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
    ) -> Result<IssuedComponentPinPair, RetentionObligationDenial> {
        let reservation = match self.reserve_pair(basis, dependency) {
            Ok(reservation) => reservation,
            Err(denial) => {
                self.record_batch_denial();
                return Err(denial);
            }
        };
        match self.resolve_pair(reservation) {
            Ok(pair) => {
                self.record_batch_admission();
                Ok(pair)
            }
            Err(denial) => {
                self.record_batch_denial();
                Err(denial)
            }
        }
    }

    pub(super) fn issue_pair_with_reserved_capacity(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
    ) -> Result<IssuedComponentPinPair, RetentionObligationDenial> {
        let reservation = match self.reserve_pair_with_capacity(basis, dependency) {
            Ok(reservation) => reservation,
            Err(denial) => {
                self.record_batch_denial();
                return Err(denial);
            }
        };
        match self.resolve_pair(reservation) {
            Ok(pair) => {
                self.record_batch_admission();
                Ok(pair)
            }
            Err(denial) => {
                self.restore_consumed_pair_capacity();
                self.record_batch_denial();
                Err(denial)
            }
        }
    }

    fn record_batch_admission(&self) {
        let mut state = self.lock();
        state.costs.batch_admitted = state.costs.batch_admitted.saturating_add(1);
    }

    fn record_batch_denial(&self) {
        let mut state = self.lock();
        state.costs.batch_denied = state.costs.batch_denied.saturating_add(1);
    }
}
