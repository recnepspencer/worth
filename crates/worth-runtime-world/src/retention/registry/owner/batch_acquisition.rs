use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::retention::component_obligation::{IssuedComponentPinPair, RetentionControlSurface};
use crate::retention::unique_component_pin::{
    ComponentBasisLeaseIdentity, ComponentBasisPinClaim, ExactComponentBasisKey,
    ExactComponentPinRequest,
};
use crate::retention::ComponentClaimKind;

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
        dependency: ComponentClaimKind,
        flight: Arc<PinFlight>,
    },
    Owned(OwnedReservation<'a>),
}

#[derive(Debug)]
struct OwnedReservation<'a> {
    request: ExactComponentPinRequest<'a>,
    key: ExactComponentBasisKey,
    dependency: ComponentClaimKind,
    lease_identity: ComponentBasisLeaseIdentity,
    flight: Arc<PinFlight>,
}

struct PairReservationSet<'a> {
    basis: &'a AdmittedCompositeRuntimeWorldBasis,
    dependency: ComponentClaimKind,
    control: Arc<dyn RetentionControlSurface>,
    unused_credit: Option<usize>,
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
        dependency: ComponentClaimKind,
    ) -> Result<IssuedComponentPinPair, RetentionObligationDenial> {
        let reservation = self.reserve_pair(basis, dependency)?;
        self.resolve_pair(reservation)
    }

    pub(super) fn issue_pair_with_reserved_capacity(
        &self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
        dependency: ComponentClaimKind,
    ) -> Result<IssuedComponentPinPair, RetentionObligationDenial> {
        let reservation = self.reserve_pair_with_capacity(basis, dependency)?;
        self.resolve_pair(reservation)
    }
}
