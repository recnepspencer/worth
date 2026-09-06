use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::RuntimeWorldOwnerIdentity;
use crate::retention::{
    ComponentBasisDependencyClass, ComponentBasisDependencyCounts, ExactComponentBasisKey,
    ExactComponentPinRequest,
};
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
            key: ExactComponentPinRequest::relational(
                basis,
                ComponentBasisDependencyClass::HistoricalInspection,
            )
            .key(),
        }
    }
    pub fn signal(basis: &AdmittedCompositeRuntimeWorldBasis) -> Self {
        Self {
            owner: basis.owner_identity(),
            key: ExactComponentPinRequest::signal(
                basis,
                ComponentBasisDependencyClass::HistoricalInspection,
            )
            .key(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldRetentionInspectionDenial {
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    ForeignOwner,
}
#[derive(Debug, Clone, Copy)]
pub struct RuntimeWorldRetentionEntry {
    pub(crate) dependencies: ComponentBasisDependencyCounts,
    pub(crate) owner_lease_present: bool,
    pub(crate) flight_present: bool,
}
impl RuntimeWorldRetentionEntry {
    pub fn dependencies(&self) -> ComponentBasisDependencyCounts {
        self.dependencies
    }
    pub fn owner_lease_present(&self) -> bool {
        self.owner_lease_present
    }
    pub fn flight_present(&self) -> bool {
        self.flight_present
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
}
