//! Installed recorded-inverse mechanism.

use worth_query_declaration::facade::application_aftermath::{
    PortablePreImageDemand, PortablePreImageLocus, PortableRecordedInverse,
};

use super::super::lowering_correspondence::InstalledLoweringCorrespondence;
use super::super::postcondition::InstalledAftermathPostcondition;

/// Installed Bridge lowering correspondence — resolved typed value (R8.9).
///
/// The diagnostic slot string is retained for explanation only. Binding identity
/// is the resolved correspondence digest, generation, and graph participation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InstalledLoweringCorrespondenceRef {
    resolved: InstalledLoweringCorrespondence,
}

impl InstalledLoweringCorrespondenceRef {
    pub(crate) fn from_resolved(resolved: InstalledLoweringCorrespondence) -> Self {
        Self { resolved }
    }

    pub const fn resolved(&self) -> &InstalledLoweringCorrespondence {
        &self.resolved
    }

    /// Diagnostic label only — never use as binding identity.
    pub fn correspondence_slot(&self) -> &str {
        self.resolved.correspondence_slot()
    }
}

/// Installed pre-image demand bound by the operation's declared reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledPreImageDemand {
    loci: Vec<InstalledPreImageLocus>,
    relations: Vec<super::InstalledPreImageRelationLocus>,
    maximum_encoded_bytes: usize,
}

impl InstalledPreImageDemand {
    pub(crate) fn from_portable(portable: &PortablePreImageDemand) -> Self {
        Self {
            loci: portable
                .loci()
                .iter()
                .map(InstalledPreImageLocus::from_portable)
                .collect(),
            relations: portable
                .relations()
                .iter()
                .map(super::InstalledPreImageRelationLocus::from_portable)
                .collect(),
            maximum_encoded_bytes: portable.maximum_encoded_bytes(),
        }
    }

    pub fn relations(&self) -> &[super::InstalledPreImageRelationLocus] {
        &self.relations
    }

    /// True when the inverse demands no prior truth at all: neither a field
    /// locus nor a relation locus. Only the created-entity inverse is empty;
    /// it derives from the original's outputs instead of a retained pre-image.
    /// A relation-only demand is not empty.
    pub fn is_empty(&self) -> bool {
        self.loci.is_empty() && self.relations.is_empty()
    }

    pub fn loci(&self) -> &[InstalledPreImageLocus] {
        &self.loci
    }

    pub const fn maximum_encoded_bytes(&self) -> usize {
        self.maximum_encoded_bytes
    }
}

/// Exact application field identity sealed into an installed inverse demand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledPreImageLocus {
    entity: String,
    aspect: String,
    field: String,
}

impl InstalledPreImageLocus {
    fn from_portable(portable: &PortablePreImageLocus) -> Self {
        Self {
            entity: portable.entity().to_owned(),
            aspect: portable.aspect().to_owned(),
            field: portable.field().to_owned(),
        }
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

/// Installed recorded-inverse correction mechanism.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledRecordedInverse {
    inverse_operation_slot: String,
    lowering_correspondence: InstalledLoweringCorrespondenceRef,
    postcondition: InstalledAftermathPostcondition,
    preimage_demand: InstalledPreImageDemand,
}

impl InstalledRecordedInverse {
    pub(crate) fn from_portable(
        portable: &PortableRecordedInverse,
        lowering_correspondence: InstalledLoweringCorrespondence,
    ) -> Self {
        Self {
            inverse_operation_slot: portable.inverse_operation_slot().to_owned(),
            lowering_correspondence: InstalledLoweringCorrespondenceRef::from_resolved(
                lowering_correspondence,
            ),
            postcondition: InstalledAftermathPostcondition::from_declared(portable.postcondition()),
            preimage_demand: InstalledPreImageDemand::from_portable(portable.preimage_demand()),
        }
    }

    pub fn inverse_operation_slot(&self) -> &str {
        &self.inverse_operation_slot
    }

    pub const fn lowering_correspondence(&self) -> &InstalledLoweringCorrespondenceRef {
        &self.lowering_correspondence
    }

    pub const fn postcondition(&self) -> &InstalledAftermathPostcondition {
        &self.postcondition
    }

    pub const fn preimage_demand(&self) -> &InstalledPreImageDemand {
        &self.preimage_demand
    }
}
