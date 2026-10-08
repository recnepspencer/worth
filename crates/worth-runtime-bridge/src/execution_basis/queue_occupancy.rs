use super::{
    BridgeBoundExecutionBasis, BridgeExecutionBasisIdentity, BridgeManagedQueueFailure,
    BridgeManagedQueueFailureKind, BridgeManagedQueueMutation,
};

/// Exact, move-only authority for one successful Signal queue admission.
#[must_use = "managed queue occupancy must be released before the execution basis can finalize"]
pub struct BridgeManagedQueueOccupancy {
    basis_identity: BridgeExecutionBasisIdentity,
    width: u64,
}

#[derive(Debug)]
pub struct BridgeManagedQueueAdmission {
    mutation: BridgeManagedQueueMutation,
    occupancy: BridgeManagedQueueOccupancy,
}

pub struct BridgeManagedQueueReleaseFailure {
    failure: BridgeManagedQueueFailure,
    occupancy: BridgeManagedQueueOccupancy,
}

impl BridgeManagedQueueOccupancy {
    pub(super) fn new(basis: &BridgeBoundExecutionBasis, width: u64) -> Self {
        Self {
            basis_identity: basis.identity().clone(),
            width,
        }
    }

    pub const fn width(&self) -> u64 {
        self.width
    }
}

impl std::fmt::Debug for BridgeManagedQueueOccupancy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeManagedQueueOccupancy")
            .field("basis_identity", &self.basis_identity)
            .field("width", &self.width)
            .finish()
    }
}

impl BridgeManagedQueueAdmission {
    pub(super) fn new(
        mutation: BridgeManagedQueueMutation,
        occupancy: BridgeManagedQueueOccupancy,
    ) -> Self {
        Self {
            mutation,
            occupancy,
        }
    }

    pub fn mutation(&self) -> &BridgeManagedQueueMutation {
        &self.mutation
    }

    pub fn into_parts(self) -> (BridgeManagedQueueMutation, BridgeManagedQueueOccupancy) {
        (self.mutation, self.occupancy)
    }
}

impl BridgeBoundExecutionBasis {
    pub fn release_managed_queue_occupancy(
        &mut self,
        occupancy: BridgeManagedQueueOccupancy,
    ) -> Result<BridgeManagedQueueMutation, BridgeManagedQueueReleaseFailure> {
        if occupancy.basis_identity != self.identity {
            return Err(BridgeManagedQueueReleaseFailure::new(
                BridgeManagedQueueFailure::new(
                    BridgeManagedQueueFailureKind::SignalRequestMismatch,
                    "managed queue occupancy belongs to another execution basis",
                ),
                occupancy,
            ));
        }
        let mutation = match self.dequeue_managed_queue_width(occupancy.width) {
            Ok(mutation) => mutation,
            Err(failure) => {
                return Err(BridgeManagedQueueReleaseFailure::new(failure, occupancy));
            }
        };
        let managed = self
            .posture
            .managed_mut()
            .expect("released queue occupancy proves managed posture");
        managed.occupancy_width = managed
            .occupancy_width
            .checked_sub(occupancy.width)
            .expect("queue occupancy can only be released by its exact move-only authority");
        Ok(mutation)
    }
}

impl BridgeManagedQueueReleaseFailure {
    fn new(failure: BridgeManagedQueueFailure, occupancy: BridgeManagedQueueOccupancy) -> Self {
        Self { failure, occupancy }
    }

    pub fn failure(&self) -> &BridgeManagedQueueFailure {
        &self.failure
    }

    pub fn into_occupancy(self) -> BridgeManagedQueueOccupancy {
        self.occupancy
    }
}

impl std::fmt::Debug for BridgeManagedQueueReleaseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeManagedQueueReleaseFailure")
            .field("failure", &self.failure)
            .field("occupancy", &self.occupancy)
            .finish()
    }
}
