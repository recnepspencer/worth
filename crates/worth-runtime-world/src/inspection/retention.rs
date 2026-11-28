use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::RuntimeWorldOwnerIdentity;
use crate::retention::ExactComponentBasisKey;
/// A descriptive exact key derived from an admitted basis. It issues no lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorldRetentionKey {
    pub(crate) owner: RuntimeWorldOwnerIdentity,
    pub(crate) key: ExactComponentBasisKey,
}
impl RuntimeWorldRetentionKey {
    pub fn relational(basis: &AdmittedCompositeRuntimeWorldBasis) -> Self {
        Self {
            owner: basis.owner_identity(),
            key: ExactComponentBasisKey::Relational(
                basis.relational_basis().admission_identity().clone(),
            ),
        }
    }
    pub fn signal(basis: &AdmittedCompositeRuntimeWorldBasis) -> Self {
        Self {
            owner: basis.owner_identity(),
            key: ExactComponentBasisKey::Signal(basis.signal_basis().admission_identity().clone()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldRetentionInspectionDenial {
    ForeignOwner,
}
/// Live claims sharing one exact component owner's lease.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeWorldRetentionEntry {
    pub(crate) live_claims: usize,
}
impl RuntimeWorldRetentionEntry {
    pub fn live_claims(&self) -> usize {
        self.live_claims
    }
}

/// Bounded registry counters. Observation usage is sampled independently;
/// this image does not claim a globally atomic World snapshot.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeWorldRetentionSnapshot {
    pub(crate) unique_pins: usize,
    pub(crate) component_obligations: usize,
    pub(crate) in_flight_acquisitions: usize,
    pub(crate) reserved_unique_pins: usize,
    pub(crate) reserved_acquisitions: usize,
    pub(crate) observations: usize,
    pub(crate) active_publication_attempts: usize,
}
impl RuntimeWorldRetentionSnapshot {
    pub fn unique_pins(&self) -> usize {
        self.unique_pins
    }
    pub fn component_obligations(&self) -> usize {
        self.component_obligations
    }
    pub fn in_flight_acquisitions(&self) -> usize {
        self.in_flight_acquisitions
    }
    pub fn reserved_unique_pins(&self) -> usize {
        self.reserved_unique_pins
    }
    pub fn reserved_acquisitions(&self) -> usize {
        self.reserved_acquisitions
    }
    pub fn observations(&self) -> usize {
        self.observations
    }
    pub fn active_publication_attempts(&self) -> usize {
        self.active_publication_attempts
    }
}
