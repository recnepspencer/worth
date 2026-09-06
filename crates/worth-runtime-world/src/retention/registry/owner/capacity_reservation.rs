use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::retention::component_obligation::PublicationRetentionObligation;
use crate::retention::ComponentBasisDependencyClass;

use super::super::RetentionObligationDenial;
use super::RuntimeWorldRetentionOwner;

trait ComponentPinPairCapacityControl: Send + Sync {
    fn bind_publication(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<PublicationRetentionObligation, RetentionObligationDenial>;

    fn release_pair_capacity(&self);
}

/// Worst-case pair capacity held before either component owner is contacted.
/// The token is the only authority that can bind that capacity to an exact
/// successor basis.
#[must_use = "reserved component capacity must be bound or dropped"]
pub(crate) struct ReservedComponentPinPairCapacity {
    control: std::sync::Arc<dyn ComponentPinPairCapacityControl>,
    armed: bool,
}

impl std::fmt::Debug for ReservedComponentPinPairCapacity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedComponentPinPairCapacity")
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl Drop for ReservedComponentPinPairCapacity {
    fn drop(&mut self) {
        if self.armed {
            self.control.release_pair_capacity();
            self.armed = false;
        }
    }
}

impl ReservedComponentPinPairCapacity {
    fn issued(control: std::sync::Arc<dyn ComponentPinPairCapacityControl>) -> Self {
        Self {
            control,
            armed: true,
        }
    }

    /// Bind the reserved pair to the exact basis selected by the operation.
    /// On denial, the owner restores consumed credit before returning this
    /// still-armed token so recovery can retry or release it exactly once.
    #[cfg(test)]
    pub(crate) fn bind_publication(
        mut self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<PublicationRetentionObligation, (Self, RetentionObligationDenial)> {
        match self.try_bind_publication(basis) {
            Ok(pair) => Ok(pair),
            Err(denial) => Err((self, denial)),
        }
    }

    /// The owner record retains the token across an acquisition unwind. The
    /// underlying acquisition restores consumed credit before unwinding.
    pub(crate) fn try_bind_publication(
        &mut self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<PublicationRetentionObligation, RetentionObligationDenial> {
        assert!(self.armed, "a reserved pair binds exactly once");
        let pair = self.control.bind_publication(basis)?;
        self.armed = false;
        Ok(pair)
    }
}

impl<D, I, T> RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn reserve_product_publication_pair(
        &self,
    ) -> Result<ReservedComponentPinPairCapacity, RetentionObligationDenial> {
        let mut state = self.lock();
        let unique_after = state
            .unique_slots
            .checked_add(state.reserved_unique_slots)
            .and_then(|value| value.checked_add(2))
            .ok_or(RetentionObligationDenial::UniquePinCapacityExhausted {
                maximum_unique_component_pins: state.maximum_unique_pins,
            })?;
        if unique_after > state.maximum_unique_pins {
            return Err(RetentionObligationDenial::UniquePinCapacityExhausted {
                maximum_unique_component_pins: state.maximum_unique_pins,
            });
        }
        let in_flight_after = state
            .active_reservations
            .checked_add(state.reserved_in_flight_reservations)
            .and_then(|value| value.checked_add(2))
            .ok_or(
                RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                    maximum_in_flight_reservations: state.maximum_in_flight_reservations,
                },
            )?;
        if in_flight_after > state.maximum_in_flight_reservations {
            return Err(
                RetentionObligationDenial::InFlightAcquisitionCapacityExhausted {
                    maximum_in_flight_reservations: state.maximum_in_flight_reservations,
                },
            );
        }
        state.reserved_unique_slots += 2;
        state.reserved_in_flight_reservations += 2;
        let control: std::sync::Arc<dyn ComponentPinPairCapacityControl> =
            std::sync::Arc::new(self.clone());
        Ok(ReservedComponentPinPairCapacity::issued(control))
    }

    pub(super) fn release_reserved_pair_capacity(&self) {
        let mut state = self.lock();
        state.reserved_unique_slots = state
            .reserved_unique_slots
            .checked_sub(2)
            .expect("a live pair reservation owns two unique capacity units");
        state.reserved_in_flight_reservations = state
            .reserved_in_flight_reservations
            .checked_sub(2)
            .expect("a live pair reservation owns two in-flight capacity units");
    }

    pub(super) fn restore_consumed_pair_capacity(&self) {
        let mut state = self.lock();
        state.reserved_unique_slots = state
            .reserved_unique_slots
            .checked_add(2)
            .expect("fixed pair capacity restoration cannot overflow");
        state.reserved_in_flight_reservations = state
            .reserved_in_flight_reservations
            .checked_add(2)
            .expect("fixed pair capacity restoration cannot overflow");
        assert!(state.reserved_unique_slots <= state.maximum_unique_pins);
        assert!(state.reserved_in_flight_reservations <= state.maximum_in_flight_reservations);
    }
}

impl<D, I, T> ComponentPinPairCapacityControl for RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn bind_publication(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<PublicationRetentionObligation, RetentionObligationDenial> {
        let pair = self.issue_pair_with_reserved_capacity(
            basis,
            ComponentBasisDependencyClass::ActivePublicationAttempt,
        );
        match pair {
            Ok(pair) => Ok(PublicationRetentionObligation::owner_issued(pair)),
            Err(denial) => Err(denial),
        }
    }

    fn release_pair_capacity(&self) {
        self.release_reserved_pair_capacity();
    }
}
