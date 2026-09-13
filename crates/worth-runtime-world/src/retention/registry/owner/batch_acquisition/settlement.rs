use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::retention::component_obligation::{
    ComponentBasisPinObligation, RetentionControlSurface,
};
use crate::retention::dependency_counts::ComponentBasisDependencyCounts;
use crate::retention::unique_component_pin::{
    ComponentBasisPinClaim, ExactComponentBasisKey, ExactComponentPinRequest,
};
use crate::retention::ComponentBasisDependencyClass;

use super::super::super::RetentionObligationDenial;
use super::super::{FlightCompletion, PinEntry, PinFlight, RuntimeWorldRetentionOwner};
use super::{OwnedReservation, PairReservation, PairReservationSet};

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn resolve_pair<'a>(
        &self,
        reservation: PairReservationSet<'a>,
    ) -> Result<
        crate::retention::component_obligation::IssuedComponentPinPair,
        RetentionObligationDenial,
    > {
        let PairReservationSet {
            basis,
            dependency,
            control,
            relational,
            signal,
        } = reservation;
        let relational = match self.resolve_component(relational, &control) {
            Ok(claim) => claim,
            Err(denial) => {
                self.rollback_pending(signal, denial.clone());
                return Err(denial);
            }
        };
        let signal = match self.resolve_component(signal, &control) {
            Ok(claim) => claim,
            Err(denial) => {
                self.rollback_claim(relational);
                return Err(denial);
            }
        };
        Ok(
            crate::retention::component_obligation::IssuedComponentPinPair::owner_issued(
                basis,
                dependency,
                ComponentBasisPinObligation::new(relational),
                ComponentBasisPinObligation::new(signal),
            ),
        )
    }

    fn resolve_component<'a>(
        &self,
        reservation: PairReservation<'a>,
        control: &Arc<dyn RetentionControlSurface>,
    ) -> Result<ComponentBasisPinClaim, RetentionObligationDenial> {
        match reservation {
            PairReservation::Ready(claim) => Ok(claim),
            PairReservation::Joined {
                request,
                key,
                dependency,
                flight,
            } => self.resolve_joined(request, key, dependency, flight, control),
            PairReservation::Owned(reservation) => self.resolve_owned(reservation, control),
        }
    }

    fn resolve_joined<'a>(
        &self,
        request: ExactComponentPinRequest<'a>,
        key: ExactComponentBasisKey,
        dependency: ComponentBasisDependencyClass,
        flight: Arc<PinFlight>,
        control: &Arc<dyn RetentionControlSurface>,
    ) -> Result<ComponentBasisPinClaim, RetentionObligationDenial> {
        let completion = wait_for_flight(&flight);
        match completion {
            FlightCompletion::AcquisitionDenied(denial) => return Err(denial),
            FlightCompletion::Released => {
                return self.acquire(request, key, dependency, Arc::clone(control));
            }
            FlightCompletion::Acquired => {}
            FlightCompletion::Pending => unreachable!(),
        }

        let mut state = self.lock();
        if state.flights.contains_key(&key) {
            drop(state);
            return self.acquire(request, key, dependency, Arc::clone(control));
        }
        let Some(lease_identity) = state
            .entries
            .get(&key)
            .and_then(|entry| entry.owner_lease.as_ref().map(|_| entry.lease_identity))
        else {
            drop(state);
            return self.acquire(request, key, dependency, Arc::clone(control));
        };
        if state.active_obligations == usize::MAX {
            return Err(RetentionObligationDenial::DependencyCountExhausted);
        }
        state
            .entries
            .get_mut(&key)
            .expect("the completed flight leaves its entry installed")
            .counts
            .increment(dependency)
            .ok_or(RetentionObligationDenial::DependencyCountExhausted)?;
        state.active_obligations += 1;
        state.costs.unique_pin_hits = state.costs.unique_pin_hits.saturating_add(1);
        state.costs.dependency_acquires = state.costs.dependency_acquires.saturating_add(1);
        Ok(ComponentBasisPinClaim::new(
            state.owner_identity,
            key,
            dependency,
            lease_identity,
            Arc::clone(control),
        ))
    }

    fn resolve_owned<'a>(
        &self,
        reservation: OwnedReservation<'a>,
        control: &Arc<dyn RetentionControlSurface>,
    ) -> Result<ComponentBasisPinClaim, RetentionObligationDenial> {
        let OwnedReservation {
            request,
            key,
            dependency,
            lease_identity,
            flight,
            new_slot,
        } = reservation;
        {
            let mut state = self.lock();
            state.costs.owner_acquisition_contacts =
                state.costs.owner_acquisition_contacts.saturating_add(1);
            state.costs.record_component_contact(request.component());
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.retain_component(request.component())
        }))
        .unwrap_or(Err(RetentionObligationDenial::OwnerOperationPanicked));
        let mut state = self.lock();
        let installed = state
            .flights
            .get(&key)
            .is_some_and(|current| Arc::ptr_eq(current, &flight));
        assert!(installed, "the batch claimant owns its installed flight");
        state.flights.remove(&key);
        state.active_reservations -= 1;
        state
            .costs
            .record_component_outcome(request.component(), result.is_ok());
        match result {
            Ok(lease) => {
                let mut counts = ComponentBasisDependencyCounts::zero();
                counts
                    .increment(dependency)
                    .expect("a new batch entry starts below its bound");
                state.entries.insert(
                    key.clone(),
                    PinEntry {
                        owner_lease: Some(lease),
                        counts,
                        lease_identity,
                    },
                );
                flight.finish(FlightCompletion::Acquired);
                Ok(ComponentBasisPinClaim::new(
                    state.owner_identity,
                    key,
                    dependency,
                    lease_identity,
                    Arc::clone(control),
                ))
            }
            Err(denial) => {
                state.active_obligations -= 1;
                if new_slot {
                    state.unique_slots -= 1;
                }
                state.costs.rollbacks = state.costs.rollbacks.saturating_add(1);
                flight.finish(FlightCompletion::AcquisitionDenied(denial.clone()));
                Err(denial)
            }
        }
    }

    fn rollback_pending<'a>(
        &self,
        reservation: PairReservation<'a>,
        denial: RetentionObligationDenial,
    ) {
        match reservation {
            PairReservation::Ready(claim) => self.rollback_claim(claim),
            PairReservation::Joined { .. } => {}
            PairReservation::Owned(reservation) => self.cancel_owned(reservation, denial),
        }
    }

    fn rollback_claim(&self, claim: ComponentBasisPinClaim) {
        let mut state = self.lock();
        state.costs.rollbacks = state.costs.rollbacks.saturating_add(1);
        drop(state);
        let control = Arc::clone(&claim.control);
        control.abandon_claim(claim);
    }

    fn cancel_owned<'a>(
        &self,
        reservation: OwnedReservation<'a>,
        denial: RetentionObligationDenial,
    ) {
        let OwnedReservation {
            key,
            flight,
            new_slot,
            ..
        } = reservation;
        let mut state = self.lock();
        let installed = state
            .flights
            .get(&key)
            .is_some_and(|current| Arc::ptr_eq(current, &flight));
        if !installed {
            return;
        }
        state.flights.remove(&key);
        state.active_reservations -= 1;
        state.active_obligations -= 1;
        if new_slot {
            state.unique_slots -= 1;
        }
        state.costs.rollbacks = state.costs.rollbacks.saturating_add(1);
        flight.finish(FlightCompletion::AcquisitionDenied(denial));
    }
}

fn wait_for_flight(flight: &PinFlight) -> FlightCompletion {
    let completion = flight
        .completion
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    flight
        .wake
        .wait_while(completion, |value| {
            matches!(*value, FlightCompletion::Pending)
        })
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}
