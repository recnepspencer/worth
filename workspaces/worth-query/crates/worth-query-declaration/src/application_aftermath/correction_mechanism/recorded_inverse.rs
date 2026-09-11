//! Schema-affine authoring and portable meaning for recorded inverses.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use super::super::postcondition::DeclaredAftermathPostcondition;
use super::portable_recorded_inverse::{
    PortablePreImageDemand, PortablePreImageLocus, PortableRecordedInverse,
};
use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue,
};

/// A schema-affine application-field identity required by a recorded inverse.
///
/// The schema witness is retained until the owning operation-definition
/// builder associates the complete aftermath contract with that same schema.
pub struct DeclaredPreImageLocus<Schema> {
    portable: PortablePreImageLocus,
    schema: PhantomData<fn() -> Schema>,
}

impl<Schema> DeclaredPreImageLocus<Schema> {
    pub fn from_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        Self {
            portable: PortablePreImageLocus {
                entity: field.entity().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
            },
            schema: PhantomData,
        }
    }

    pub fn entity(&self) -> &str {
        self.portable.entity()
    }

    pub fn aspect(&self) -> &str {
        self.portable.aspect()
    }

    pub fn field(&self) -> &str {
        self.portable.field()
    }

    fn into_portable(self) -> PortablePreImageLocus {
        self.portable
    }
}

impl<Schema> Clone for DeclaredPreImageLocus<Schema> {
    fn clone(&self) -> Self {
        Self {
            portable: self.portable.clone(),
            schema: PhantomData,
        }
    }
}

impl<Schema> fmt::Debug for DeclaredPreImageLocus<Schema> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.portable.fmt(formatter)
    }
}

impl<Schema> PartialEq for DeclaredPreImageLocus<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.portable == other.portable
    }
}

impl<Schema> Eq for DeclaredPreImageLocus<Schema> {}

impl<Schema> PartialOrd for DeclaredPreImageLocus<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredPreImageLocus<Schema> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.portable.cmp(&other.portable)
    }
}

impl<Schema> Hash for DeclaredPreImageLocus<Schema> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.portable.hash(state);
    }
}

/// Schema-affine retained pre-image demand required by an inverse.
pub struct DeclaredPreImageDemand<Schema> {
    loci: Vec<DeclaredPreImageLocus<Schema>>,
    maximum_encoded_bytes: usize,
}

impl<Schema> DeclaredPreImageDemand<Schema> {
    pub fn new(
        loci: impl IntoIterator<Item = DeclaredPreImageLocus<Schema>>,
        maximum_encoded_bytes: usize,
    ) -> Result<Self, DeclaredPreImageDemandDenial> {
        let loci = loci.into_iter().collect::<Vec<_>>();
        validate_preimage_demand(&loci, maximum_encoded_bytes)?;
        Ok(Self {
            loci,
            maximum_encoded_bytes,
        })
    }

    pub fn loci(&self) -> &[DeclaredPreImageLocus<Schema>] {
        &self.loci
    }

    pub const fn maximum_encoded_bytes(&self) -> usize {
        self.maximum_encoded_bytes
    }

    fn into_portable(self) -> PortablePreImageDemand {
        PortablePreImageDemand {
            loci: self
                .loci
                .into_iter()
                .map(DeclaredPreImageLocus::into_portable)
                .collect(),
            maximum_encoded_bytes: self.maximum_encoded_bytes,
        }
    }
}

fn validate_preimage_demand<Schema>(
    loci: &[DeclaredPreImageLocus<Schema>],
    maximum_encoded_bytes: usize,
) -> Result<(), DeclaredPreImageDemandDenial> {
    if loci.is_empty() {
        return Err(DeclaredPreImageDemandDenial::EmptyDemand);
    }
    if maximum_encoded_bytes == 0 {
        return Err(DeclaredPreImageDemandDenial::ZeroByteBound);
    }
    if loci.iter().any(|locus| {
        locus.entity().trim().is_empty()
            || locus.aspect().trim().is_empty()
            || locus.field().trim().is_empty()
    }) {
        return Err(DeclaredPreImageDemandDenial::EmptyLocusAxis);
    }
    if loci
        .iter()
        .enumerate()
        .any(|(index, locus)| loci[..index].contains(locus))
    {
        return Err(DeclaredPreImageDemandDenial::DuplicateLocus);
    }
    if loci
        .iter()
        .skip(1)
        .any(|locus| locus.entity() != loci[0].entity())
    {
        return Err(DeclaredPreImageDemandDenial::MultipleEntityRoles);
    }
    Ok(())
}

impl<Schema> Clone for DeclaredPreImageDemand<Schema> {
    fn clone(&self) -> Self {
        Self {
            loci: self.loci.clone(),
            maximum_encoded_bytes: self.maximum_encoded_bytes,
        }
    }
}

impl<Schema> fmt::Debug for DeclaredPreImageDemand<Schema> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeclaredPreImageDemand")
            .field("loci", &self.loci)
            .field("maximum_encoded_bytes", &self.maximum_encoded_bytes)
            .finish()
    }
}

impl<Schema> PartialEq for DeclaredPreImageDemand<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.loci == other.loci && self.maximum_encoded_bytes == other.maximum_encoded_bytes
    }
}

impl<Schema> Eq for DeclaredPreImageDemand<Schema> {}

impl<Schema> PartialOrd for DeclaredPreImageDemand<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredPreImageDemand<Schema> {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.loci, self.maximum_encoded_bytes).cmp(&(&other.loci, other.maximum_encoded_bytes))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeclaredPreImageDemandDenial {
    EmptyDemand,
    EmptyLocusAxis,
    ZeroByteBound,
    DuplicateLocus,
    MultipleEntityRoles,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DeclaredLoweringCorrespondenceRef {
    correspondence_slot: String,
}

impl DeclaredLoweringCorrespondenceRef {
    pub fn new(correspondence_slot: impl Into<String>) -> Result<Self, &'static str> {
        let correspondence_slot = correspondence_slot.into();
        if correspondence_slot.trim().is_empty() {
            return Err("empty-lowering-correspondence");
        }
        Ok(Self {
            correspondence_slot,
        })
    }

    pub fn correspondence_slot(&self) -> &str {
        &self.correspondence_slot
    }
}

/// Schema-affine recorded inverse authoring.
pub struct DeclaredRecordedInverse<Schema> {
    inverse_operation_slot: String,
    lowering_correspondence: DeclaredLoweringCorrespondenceRef,
    postcondition: DeclaredAftermathPostcondition,
    preimage_demand: DeclaredPreImageDemand<Schema>,
}

impl<Schema> DeclaredRecordedInverse<Schema> {
    pub fn new(
        inverse_operation_slot: impl Into<String>,
        lowering_correspondence: DeclaredLoweringCorrespondenceRef,
        postcondition: DeclaredAftermathPostcondition,
        preimage_demand: DeclaredPreImageDemand<Schema>,
    ) -> Result<Self, &'static str> {
        let inverse_operation_slot = inverse_operation_slot.into();
        if inverse_operation_slot.trim().is_empty() {
            return Err("empty-inverse-operation");
        }
        if !matches!(
            postcondition,
            DeclaredAftermathPostcondition::ExactPriorTruth
        ) {
            return Err("recorded-inverse-requires-exact-prior-truth");
        }
        Ok(Self {
            inverse_operation_slot,
            lowering_correspondence,
            postcondition,
            preimage_demand,
        })
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

    pub const fn preimage_demand(&self) -> &DeclaredPreImageDemand<Schema> {
        &self.preimage_demand
    }

    pub(super) fn into_portable(self) -> PortableRecordedInverse {
        PortableRecordedInverse {
            inverse_operation_slot: self.inverse_operation_slot,
            lowering_correspondence: self.lowering_correspondence,
            postcondition: self.postcondition,
            preimage_demand: self.preimage_demand.into_portable(),
        }
    }
}

impl<Schema> fmt::Debug for DeclaredRecordedInverse<Schema> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DeclaredRecordedInverse")
            .field("inverse_operation_slot", &self.inverse_operation_slot)
            .field("lowering_correspondence", &self.lowering_correspondence)
            .field("postcondition", &self.postcondition)
            .field("preimage_demand", &self.preimage_demand)
            .finish()
    }
}

impl<Schema> Clone for DeclaredRecordedInverse<Schema> {
    fn clone(&self) -> Self {
        Self {
            inverse_operation_slot: self.inverse_operation_slot.clone(),
            lowering_correspondence: self.lowering_correspondence.clone(),
            postcondition: self.postcondition.clone(),
            preimage_demand: self.preimage_demand.clone(),
        }
    }
}

impl<Schema> PartialEq for DeclaredRecordedInverse<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.inverse_operation_slot == other.inverse_operation_slot
            && self.lowering_correspondence == other.lowering_correspondence
            && self.postcondition == other.postcondition
            && self.preimage_demand == other.preimage_demand
    }
}

impl<Schema> Eq for DeclaredRecordedInverse<Schema> {}

impl<Schema> PartialOrd for DeclaredRecordedInverse<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredRecordedInverse<Schema> {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            &self.inverse_operation_slot,
            &self.lowering_correspondence,
            &self.postcondition,
            &self.preimage_demand,
        )
            .cmp(&(
                &other.inverse_operation_slot,
                &other.lowering_correspondence,
                &other.postcondition,
                &other.preimage_demand,
            ))
    }
}
