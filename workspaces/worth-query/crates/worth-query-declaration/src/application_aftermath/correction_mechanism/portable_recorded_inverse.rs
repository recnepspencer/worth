//! Public-read recorded-inverse meaning after schema association.

use super::super::postcondition::DeclaredAftermathPostcondition;
use super::recorded_inverse::DeclaredLoweringCorrespondenceRef;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PortablePreImageLocus {
    pub(super) entity: String,
    pub(super) aspect: String,
    pub(super) field: String,
}

impl PortablePreImageLocus {
    pub fn from_untrusted_fields(entity: String, aspect: String, field: String) -> Self {
        Self {
            entity,
            aspect,
            field,
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortablePreImageDemand {
    pub(super) loci: Vec<PortablePreImageLocus>,
    pub(super) relations: Vec<super::PortablePreImageRelationLocus>,
    pub(super) maximum_encoded_bytes: usize,
}

impl PortablePreImageDemand {
    pub fn from_untrusted_fields(
        loci: Vec<PortablePreImageLocus>,
        maximum_encoded_bytes: usize,
    ) -> Self {
        Self {
            loci,
            relations: Vec::new(),
            maximum_encoded_bytes,
        }
    }

    pub fn with_untrusted_relations(
        mut self,
        relations: Vec<super::PortablePreImageRelationLocus>,
    ) -> Self {
        self.relations = relations;
        self
    }
    pub fn relations(&self) -> &[super::PortablePreImageRelationLocus] {
        &self.relations
    }

    pub fn loci(&self) -> &[PortablePreImageLocus] {
        &self.loci
    }

    pub const fn maximum_encoded_bytes(&self) -> usize {
        self.maximum_encoded_bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortableRecordedInverse {
    pub(super) inverse_operation_slot: String,
    pub(super) lowering_correspondence: DeclaredLoweringCorrespondenceRef,
    pub(super) postcondition: DeclaredAftermathPostcondition,
    pub(super) preimage_demand: PortablePreImageDemand,
}

impl PortableRecordedInverse {
    pub fn from_untrusted_fields(
        inverse_operation_slot: String,
        lowering_correspondence: DeclaredLoweringCorrespondenceRef,
        postcondition: DeclaredAftermathPostcondition,
        preimage_demand: PortablePreImageDemand,
    ) -> Self {
        Self {
            inverse_operation_slot,
            lowering_correspondence,
            postcondition,
            preimage_demand,
        }
    }

    pub fn inverse_operation_slot(&self) -> &str {
        &self.inverse_operation_slot
    }

    pub const fn lowering_correspondence(&self) -> &DeclaredLoweringCorrespondenceRef {
        &self.lowering_correspondence
    }

    pub const fn postcondition(&self) -> &DeclaredAftermathPostcondition {
        &self.postcondition
    }

    pub const fn preimage_demand(&self) -> &PortablePreImageDemand {
        &self.preimage_demand
    }
}
