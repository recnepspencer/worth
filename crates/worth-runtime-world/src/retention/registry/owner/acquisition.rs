use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

#[cfg(test)]
use worth_signal::facade::branch::SignalBranchRetentionReleaseOutcome;

use super::super::super::claim_counts::ComponentClaimCounts;
use super::super::super::component_obligation::RetentionControlSurface;
#[cfg(test)]
use super::super::super::component_obligation::{
    ComponentBasisPinObligation, RetentionReleaseDenial,
};
use super::super::super::unique_component_pin::{
    ComponentBasisPinClaim, ExactComponentBasis, ExactComponentBasisKey, ExactComponentPinRequest,
};
use super::super::super::ComponentClaimKind;
use super::super::RetentionObligationDenial;
#[cfg(test)]
use super::OwnerReleaseFailure;
use super::{
    ComponentOwnerLease, FlightCompletion, PinEntry, PinFlight, RuntimeWorldRetentionOwner,
};

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn retain_component(
        &self,
        component: ExactComponentBasis<'_>,
    ) -> Result<ComponentOwnerLease, RetentionObligationDenial> {
        #[cfg(test)]
        self.lock().costs.record_component_contact(component);
        let (relational, signal) = {
            let state = self.lock();
            (state.relational_port.clone(), state.signal_port.clone())
        };
        match component {
            ExactComponentBasis::Relational(basis) => relational
                .retain_component_basis(basis)
                .map(ComponentOwnerLease::Relational)
                .map_err(RetentionObligationDenial::Relational),
            ExactComponentBasis::Signal(basis) => signal
                .retain_exact(basis)
                .map(ComponentOwnerLease::Signal)
                .map_err(RetentionObligationDenial::Signal),
        }
    }

    #[cfg(test)]
    pub(super) fn release_component(
        &self,
        lease: ComponentOwnerLease,
    ) -> Result<(), OwnerReleaseFailure> {
        let (relational, signal) = {
            let state = self.lock();
            (state.relational_port.clone(), state.signal_port.clone())
        };
        match lease {
            ComponentOwnerLease::Relational(lease) => relational
                .release_component_basis(lease)
                .map(|_| ())
                .map_err(|denial| OwnerReleaseFailure {
                    reason: RetentionReleaseDenial::Relational(denial.denial().clone()),
                    lease: ComponentOwnerLease::Relational(denial.into_lease()),
                }),
            ComponentOwnerLease::Signal(lease) => match signal.release_exact(lease) {
                SignalBranchRetentionReleaseOutcome::Released(_) => Ok(()),
                SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
                    Err(OwnerReleaseFailure {
                        reason: RetentionReleaseDenial::Signal(denial),
                        lease: ComponentOwnerLease::Signal(lease),
                    })
                }
            },
        }
    }

    #[cfg(test)]
    pub(crate) fn issue_component(
        &self,
        request: ExactComponentPinRequest<'_>,
    ) -> Result<ComponentBasisPinObligation, RetentionObligationDenial> {
        let expected = self.lock().owner_identity;
        if request.owner() != expected {
            return Err(RetentionObligationDenial::ForeignOwner {
                expected,
                actual: request.owner(),
            });
        }
        let key = request.key();
        let dependency = request.dependency();
        let control: Arc<dyn RetentionControlSurface> = Arc::new(self.clone());
        self.acquire(request, key, dependency, control)
            .map(ComponentBasisPinObligation::new)
    }

    pub(super) fn acquire(
        &self,
        request: ExactComponentPinRequest<'_>,
        key: ExactComponentBasisKey,
        dependency: ComponentClaimKind,
        control: Arc<dyn RetentionControlSurface>,
    ) -> Result<ComponentBasisPinClaim, RetentionObligationDenial> {
        let (flight, identity) = loop {
            let mut state = self.lock();
            if let Some(flight) = state.flights.get(&key).cloned() {
                #[cfg(test)]
                {
                    state.costs.single_flight_joins =
                        state.costs.single_flight_joins.saturating_add(1);
                }
                drop(state);
                match flight.wait() {
                    FlightCompletion::AcquisitionDenied(denial) => return Err(denial),
                    FlightCompletion::Acquired | FlightCompletion::Released => continue,
                    FlightCompletion::Pending => unreachable!(),
                }
            }
            let live_identity = state.entries.get(&key).map(|entry| entry.lease_identity);
            if let Some(lease_identity) = live_identity {
                if state.active_obligations == usize::MAX {
                    return Err(RetentionObligationDenial::DependencyCountExhausted);
                }
                state
                    .entries
                    .get_mut(&key)
                    .expect("live entry remains installed")
                    .counts
                    .increment(dependency)
                    .ok_or(RetentionObligationDenial::DependencyCountExhausted)?;
                state.active_obligations += 1;
                return Ok(ComponentBasisPinClaim::new(
                    state.owner_identity,
                    key,
                    dependency,
                    lease_identity,
                    control,
                ));
            }
            if state.maximum_in_flight_reservations.is_some_and(|maximum| {
                state
                    .active_reservations
                    .saturating_add(state.reserved_in_flight_reservations)
                    >= maximum
            }) {
                return Err(
                    RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                        maximum_in_flight_reservations: state.maximum_in_flight_reservations,
                    },
                );
            }
            if state.maximum_unique_pins.is_some_and(|maximum| {
                state
                    .unique_slots()
                    .saturating_add(state.reserved_unique_slots)
                    >= maximum
            }) {
                return Err(RetentionObligationDenial::UniquePinCapacityExhausted {
                    maximum_unique_component_pins: state.maximum_unique_pins,
                });
            }
            if state.active_obligations == usize::MAX {
                return Err(RetentionObligationDenial::DependencyCountExhausted);
            }
            state
                .active_reservations
                .checked_add(1)
                .ok_or(RetentionObligationDenial::DependencyCountExhausted)?;
            state
                .unique_slots()
                .checked_add(1)
                .ok_or(RetentionObligationDenial::DependencyCountExhausted)?;
            let ordinal = state.next_lease_ordinal;
            state.next_lease_ordinal = ordinal
                .checked_add(1)
                .ok_or(RetentionObligationDenial::LeaseIdentityExhausted)?;
            let identity =
                super::super::super::unique_component_pin::ComponentBasisLeaseIdentity::issued(
                    state.owner_identity,
                    ordinal,
                );
            let flight = Arc::new(PinFlight::new(false));
            state.active_reservations += 1;
            state.active_obligations += 1;
            state.flights.insert(key.clone(), Arc::clone(&flight));
            break (flight, identity);
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.retain_component(request.component())
        }))
        .unwrap_or(Err(RetentionObligationDenial::OwnerOperationPanicked));
        let mut state = self.lock();
        state.flights.remove(&key);
        state.active_reservations -= 1;
        match result {
            Ok(lease) => {
                let mut counts = ComponentClaimCounts::zero();
                counts
                    .increment(dependency)
                    .expect("a new count starts below its bound");
                state.entries.insert(
                    key.clone(),
                    PinEntry {
                        owner_lease: lease,
                        counts,
                        lease_identity: identity,
                    },
                );
                flight.finish(FlightCompletion::Acquired);
                Ok(ComponentBasisPinClaim::new(
                    state.owner_identity,
                    key,
                    dependency,
                    identity,
                    control,
                ))
            }
            Err(denial) => {
                state.active_obligations -= 1;
                flight.finish(FlightCompletion::AcquisitionDenied(denial.clone()));
                Err(denial)
            }
        }
    }
}
