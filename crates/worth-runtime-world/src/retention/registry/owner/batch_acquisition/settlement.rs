use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use crate::retention::claim_counts::ComponentClaimCounts;
use crate::retention::component_obligation::{
    ComponentBasisPinObligation, RetentionControlSurface,
};
use crate::retention::unique_component_pin::{
    ComponentBasisPinClaim, ExactComponentBasisKey, ExactComponentPinRequest,
};
use crate::retention::ComponentClaimKind;

use super::super::super::RetentionObligationDenial;
use super::super::{
    ComponentOwnerLease, FlightCompletion, PinEntry, PinFlight, RegistryState,
    RuntimeWorldRetentionOwner,
};
use super::{OwnedReservation, PairReservation, PairReservationSet};

// A lease drawn against reserved credit stays behind its flight until both legs settle.
enum ResolvedComponent<'a> {
    Ready(ComponentBasisPinClaim),
    Provisional(OwnedReservation<'a>, ComponentOwnerLease),
}

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
            unused_credit,
            relational,
            signal,
        } = reservation;
        let reserved = unused_credit.is_some();
        let relational = match self.resolve_component(relational, &control, reserved) {
            Ok(claim) => claim,
            Err(denial) => {
                self.rollback_pending(signal, denial.clone(), reserved);
                return Err(denial);
            }
        };
        let signal = match self.resolve_component(signal, &control, reserved) {
            Ok(claim) => claim,
            Err(denial) => {
                self.rollback_resolved(relational);
                return Err(denial);
            }
        };
        let (relational, signal) = if let Some(unused_credit) = unused_credit {
            let mut state = self.lock();
            let relational = Self::settle_component(&mut state, relational, &control);
            let signal = Self::settle_component(&mut state, signal, &control);
            state.reserved_unique_slots -= unused_credit;
            state.reserved_in_flight_reservations -= unused_credit;
            (relational, signal)
        } else {
            let (ResolvedComponent::Ready(relational), ResolvedComponent::Ready(signal)) =
                (relational, signal)
            else {
                unreachable!("ordinary acquisitions settle immediately")
            };
            (relational, signal)
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
        reserved: bool,
    ) -> Result<ResolvedComponent<'a>, RetentionObligationDenial> {
        match reservation {
            PairReservation::Ready(claim) => Ok(ResolvedComponent::Ready(claim)),
            PairReservation::Joined {
                request,
                key,
                dependency,
                flight,
            } => self
                .resolve_joined(request, key, dependency, flight, control)
                .map(ResolvedComponent::Ready),
            PairReservation::Owned(reservation) => {
                self.resolve_owned(reservation, control, reserved)
            }
        }
    }

    fn resolve_joined<'a>(
        &self,
        request: ExactComponentPinRequest<'a>,
        key: ExactComponentBasisKey,
        dependency: ComponentClaimKind,
        flight: Arc<PinFlight>,
        control: &Arc<dyn RetentionControlSurface>,
    ) -> Result<ComponentBasisPinClaim, RetentionObligationDenial> {
        if let FlightCompletion::AcquisitionDenied(denial) = flight.wait() {
            return Err(denial);
        }
        self.acquire(request, key, dependency, Arc::clone(control))
    }

    fn resolve_owned<'a>(
        &self,
        reservation: OwnedReservation<'a>,
        control: &Arc<dyn RetentionControlSurface>,
        reserved: bool,
    ) -> Result<ResolvedComponent<'a>, RetentionObligationDenial> {
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.retain_component(reservation.request.component())
        }))
        .unwrap_or(Err(RetentionObligationDenial::OwnerOperationPanicked));
        match result {
            Ok(lease) if reserved => Ok(ResolvedComponent::Provisional(reservation, lease)),
            Ok(lease) => Ok(ResolvedComponent::Ready(Self::settle_owned(
                &mut self.lock(),
                reservation,
                lease,
                control,
            ))),
            Err(denial) => {
                self.cancel_owned(
                    reservation,
                    FlightCompletion::AcquisitionDenied(denial.clone()),
                    reserved,
                );
                Err(denial)
            }
        }
    }

    fn settle_component(
        state: &mut RegistryState<D, I, T>,
        component: ResolvedComponent<'_>,
        control: &Arc<dyn RetentionControlSurface>,
    ) -> ComponentBasisPinClaim {
        match component {
            ResolvedComponent::Ready(claim) => claim,
            ResolvedComponent::Provisional(reservation, lease) => {
                Self::settle_owned(state, reservation, lease, control)
            }
        }
    }

    fn settle_owned(
        state: &mut RegistryState<D, I, T>,
        reservation: OwnedReservation<'_>,
        lease: ComponentOwnerLease,
        control: &Arc<dyn RetentionControlSurface>,
    ) -> ComponentBasisPinClaim {
        let OwnedReservation {
            key,
            dependency,
            lease_identity,
            flight,
            ..
        } = reservation;
        Self::remove_acquisition_flight(state, &key, &flight);
        let mut counts = ComponentClaimCounts::zero();
        counts
            .increment(dependency)
            .expect("a new entry starts below its bound");
        state.entries.insert(
            key.clone(),
            PinEntry {
                owner_lease: lease,
                counts,
                lease_identity,
            },
        );
        flight.finish(FlightCompletion::Acquired);
        ComponentBasisPinClaim::new(
            state.owner_identity,
            key,
            dependency,
            lease_identity,
            Arc::clone(control),
        )
    }

    fn rollback_pending<'a>(
        &self,
        reservation: PairReservation<'a>,
        denial: RetentionObligationDenial,
        reserved: bool,
    ) {
        match reservation {
            PairReservation::Ready(claim) => self.rollback_claim(claim),
            PairReservation::Joined { .. } => {}
            PairReservation::Owned(reservation) => self.cancel_owned(
                reservation,
                FlightCompletion::AcquisitionDenied(denial),
                reserved,
            ),
        }
    }

    fn rollback_resolved(&self, component: ResolvedComponent<'_>) {
        match component {
            ResolvedComponent::Ready(claim) => self.rollback_claim(claim),
            ResolvedComponent::Provisional(reservation, lease) => {
                // Keep the flight installed through the owner release. Freeing
                // its slot and restoring credit are one registry mutation.
                drop(lease);
                self.cancel_owned(reservation, FlightCompletion::Released, true);
            }
        }
    }

    fn rollback_claim(&self, claim: ComponentBasisPinClaim) {
        let control = Arc::clone(&claim.control);
        control.abandon_claim(claim);
    }

    fn cancel_owned(
        &self,
        reservation: OwnedReservation<'_>,
        completion: FlightCompletion,
        restore_credit: bool,
    ) {
        let OwnedReservation { key, flight, .. } = reservation;
        let mut state = self.lock();
        Self::remove_acquisition_flight(&mut state, &key, &flight);
        state.active_obligations -= 1;
        if restore_credit {
            state.reserved_unique_slots += 1;
            state.reserved_in_flight_reservations += 1;
        }
        flight.finish(completion);
    }

    fn remove_acquisition_flight(
        state: &mut RegistryState<D, I, T>,
        key: &ExactComponentBasisKey,
        flight: &Arc<PinFlight>,
    ) {
        let installed = state.flights.remove(key).expect("installed batch flight");
        assert!(
            Arc::ptr_eq(&installed, flight),
            "the batch owns its installed flight"
        );
        state.active_reservations -= 1;
    }
}
