use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::retention::component_obligation::RetentionControlSurface;
use crate::retention::unique_component_pin::{
    ComponentBasisLeaseIdentity, ExactComponentBasisKey, ExactComponentPinRequest,
};
use crate::retention::ComponentBasisDependencyClass;

use super::super::super::RetentionObligationDenial;
use super::super::{PinFlight, RegistryState, RuntimeWorldRetentionOwner};
use super::{OwnedReservation, PairReservation, PairReservationSet};

#[derive(Debug)]
enum ExistingComponent {
    Ready(ComponentBasisLeaseIdentity),
    Acquiring(Arc<PinFlight>),
    Vacant { new_slot: bool },
}

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn reserve_pair<'a>(
        &self,
        basis: &'a AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
    ) -> Result<PairReservationSet<'a>, RetentionObligationDenial> {
        self.reserve_pair_inner(basis, dependency, None)
    }

    pub(super) fn reserve_pair_with_capacity<'a>(
        &self,
        basis: &'a AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
    ) -> Result<PairReservationSet<'a>, RetentionObligationDenial> {
        self.reserve_pair_inner(basis, dependency, Some((2, 2)))
    }

    fn reserve_pair_inner<'a>(
        &self,
        basis: &'a AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentBasisDependencyClass,
        capacity_credit: Option<(usize, usize)>,
    ) -> Result<PairReservationSet<'a>, RetentionObligationDenial> {
        let expected = self.lock().owner_identity;
        let actual = basis.owner_identity();
        if actual != expected {
            return Err(RetentionObligationDenial::ForeignOwner { expected, actual });
        }

        let relational = ExactComponentPinRequest::relational(basis, dependency);
        let signal = ExactComponentPinRequest::signal(basis, dependency);
        if relational.owner() != signal.owner()
            || relational.owner() != actual
            || relational.dependency() != signal.dependency()
            || relational.key() == signal.key()
        {
            return Err(RetentionObligationDenial::InvalidComponentPair);
        }

        let control: Arc<dyn RetentionControlSurface> = Arc::new(self.clone());
        let mut state = self.lock();
        let relational_slot = classify(&state, &relational.key());
        let signal_slot = classify(&state, &signal.key());
        let additional_unique_entries = usize::from(is_new_slot(&relational_slot))
            .checked_add(usize::from(is_new_slot(&signal_slot)))
            .expect("fixed pair cardinality cannot overflow a usize");
        let additional_flights = usize::from(is_vacant(&relational_slot))
            .checked_add(usize::from(is_vacant(&signal_slot)))
            .expect("fixed pair cardinality cannot overflow a usize");
        let immediately_registered = usize::from(is_ready(&relational_slot))
            .checked_add(usize::from(is_ready(&signal_slot)))
            .expect("fixed pair cardinality cannot overflow a usize")
            .checked_add(additional_flights)
            .expect("fixed pair cardinality cannot overflow a usize");

        let reserved_unique =
            capacity_credit.map_or(Ok(state.reserved_unique_slots), |(unique, _)| {
                state.reserved_unique_slots.checked_sub(unique).ok_or(
                    RetentionObligationDenial::UniquePinCapacityExhausted {
                        maximum_unique_component_pins: state.maximum_unique_pins,
                    },
                )
            })?;
        let reserved_in_flight = capacity_credit.map_or(
            Ok(state.reserved_in_flight_reservations),
            |(_, in_flight)| {
                state
                    .reserved_in_flight_reservations
                    .checked_sub(in_flight)
                    .ok_or(
                        RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                            maximum_in_flight_reservations: state.maximum_in_flight_reservations,
                        },
                    )
            },
        )?;
        let unique_after = state
            .unique_slots
            .checked_add(reserved_unique)
            .and_then(|value| value.checked_add(additional_unique_entries))
            .ok_or(RetentionObligationDenial::UniquePinCapacityExhausted {
                maximum_unique_component_pins: state.maximum_unique_pins,
            })?;
        if unique_after > state.maximum_unique_pins {
            return Err(RetentionObligationDenial::UniquePinCapacityExhausted {
                maximum_unique_component_pins: state.maximum_unique_pins,
            });
        }

        let reservations_after = state
            .active_reservations
            .checked_add(reserved_in_flight)
            .and_then(|value| value.checked_add(additional_flights))
            .ok_or(
                RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                    maximum_in_flight_reservations: state.maximum_in_flight_reservations,
                },
            )?;
        if reservations_after > state.maximum_in_flight_reservations {
            return Err(
                RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                    maximum_in_flight_reservations: state.maximum_in_flight_reservations,
                },
            );
        }

        if state
            .active_obligations
            .checked_add(immediately_registered)
            .is_none()
        {
            return Err(RetentionObligationDenial::DependencyCountExhausted);
        }
        check_ready_dependency_capacity(&state, &relational_slot, &relational.key(), dependency)?;
        check_ready_dependency_capacity(&state, &signal_slot, &signal.key(), dependency)?;
        let ordinal_count = u64::try_from(additional_flights)
            .map_err(|_| RetentionObligationDenial::LeaseIdentityExhausted)?;
        state
            .next_lease_ordinal
            .checked_add(ordinal_count)
            .ok_or(RetentionObligationDenial::LeaseIdentityExhausted)?;

        let relational = install_reservation(
            &mut state,
            relational,
            relational_slot,
            dependency,
            Arc::clone(&control),
        );
        let signal =
            install_reservation(&mut state, signal, signal_slot, dependency, control.clone());
        if let Some((unique, in_flight)) = capacity_credit {
            state.reserved_unique_slots -= unique;
            state.reserved_in_flight_reservations -= in_flight;
        }
        drop(state);
        Ok(PairReservationSet {
            basis,
            dependency,
            control,
            relational,
            signal,
        })
    }
}

fn classify<D, I, T>(
    state: &RegistryState<D, I, T>,
    key: &ExactComponentBasisKey,
) -> ExistingComponent
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if let Some(flight) = state.flights.get(key) {
        return ExistingComponent::Acquiring(Arc::clone(flight));
    }
    state
        .entries
        .get(key)
        .and_then(|entry| entry.owner_lease.as_ref().map(|_| entry.lease_identity))
        .map_or(
            ExistingComponent::Vacant {
                new_slot: !state.entries.contains_key(key),
            },
            ExistingComponent::Ready,
        )
}

fn is_ready(slot: &ExistingComponent) -> bool {
    matches!(slot, ExistingComponent::Ready(_))
}

fn is_vacant(slot: &ExistingComponent) -> bool {
    matches!(slot, ExistingComponent::Vacant { .. })
}

fn is_new_slot(slot: &ExistingComponent) -> bool {
    matches!(slot, ExistingComponent::Vacant { new_slot: true })
}

fn check_ready_dependency_capacity<D, I, T>(
    state: &RegistryState<D, I, T>,
    slot: &ExistingComponent,
    key: &ExactComponentBasisKey,
    dependency: ComponentBasisDependencyClass,
) -> Result<(), RetentionObligationDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if !is_ready(slot) {
        return Ok(());
    }
    state
        .entries
        .get(key)
        .and_then(|entry| entry.counts.get(dependency).checked_add(1))
        .map(|_| ())
        .ok_or(RetentionObligationDenial::DependencyCountExhausted)
}

fn install_reservation<'a, D, I, T>(
    state: &mut RegistryState<D, I, T>,
    request: ExactComponentPinRequest<'a>,
    slot: ExistingComponent,
    dependency: ComponentBasisDependencyClass,
    control: Arc<dyn RetentionControlSurface>,
) -> PairReservation<'a>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    match slot {
        ExistingComponent::Ready(lease_identity) => {
            state
                .entries
                .get_mut(&request.key())
                .expect("classified ready entry remains installed")
                .counts
                .increment(dependency)
                .expect("ready dependency count was checked before mutation");
            state.active_obligations += 1;
            state.costs.unique_pin_hits = state.costs.unique_pin_hits.saturating_add(1);
            state.costs.dependency_acquires = state.costs.dependency_acquires.saturating_add(1);
            PairReservation::Ready(
                crate::retention::unique_component_pin::ComponentBasisPinClaim::new(
                    state.owner_identity,
                    request.key(),
                    dependency,
                    lease_identity,
                    control,
                ),
            )
        }
        ExistingComponent::Acquiring(flight) => {
            state.costs.single_flight_joins = state.costs.single_flight_joins.saturating_add(1);
            PairReservation::Joined {
                key: request.key(),
                request,
                dependency,
                flight,
            }
        }
        ExistingComponent::Vacant { new_slot } => {
            let ordinal = state.next_lease_ordinal;
            state.next_lease_ordinal = ordinal
                .checked_add(1)
                .expect("batch lease ordinal was checked before mutation");
            let lease_identity = ComponentBasisLeaseIdentity::issued(state.owner_identity, ordinal);
            let flight = Arc::new(PinFlight::new());
            if new_slot {
                state.unique_slots += 1;
            }
            state.active_reservations += 1;
            state.active_obligations += 1;
            state.costs.flights_started = state.costs.flights_started.saturating_add(1);
            state.flights.insert(request.key(), Arc::clone(&flight));
            PairReservation::Owned(OwnedReservation {
                request,
                key: request.key(),
                dependency,
                lease_identity,
                flight,
                new_slot,
            })
        }
    }
}
