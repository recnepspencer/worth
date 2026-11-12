//! Descriptive availability of the actual external canonical-frame read port.
use super::DurabilityAccess;
use worth_foundational::facade::FoundationalBoundaryAvailability;

impl DurabilityAccess<'_> {
    /// Report whether an external canonical-frame read port is currently bound.
    /// This describes the port, without validating artifact bytes or granting
    /// recovery authority. Resident canonical frames remain a separate read basis.
    pub fn external_canonical_source_availability(&self) -> FoundationalBoundaryAvailability {
        match self.runtime.durability.cold_source() {
            Some(_) => FoundationalBoundaryAvailability::Present,
            None => FoundationalBoundaryAvailability::Unavailable,
        }
    }
}
