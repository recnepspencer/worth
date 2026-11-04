//! Schema-affine authoring and portable meaning for recorded inverses.

use std::cmp::Ordering;
use std::fmt;

use super::super::postcondition::DeclaredAftermathPostcondition;
use super::portable_recorded_inverse::PortableRecordedInverse;
use super::DeclaredPreImageDemand;

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
    pub(crate) fn from_binding<Correction>(preimage_demand: DeclaredPreImageDemand<Schema>) -> Self
    where
        Schema: crate::application_schema::ApplicationSchema,
        Correction: super::super::ApplicationCorrectionBinding<Schema>,
    {
        use crate::application_operation::ApplicationMutationBinding;
        let inverse = super::super::ApplicationInverseBinding::<Schema, Correction>::descriptor();
        Self {
            inverse_operation_slot: inverse.operation_name().to_owned(),
            lowering_correspondence: DeclaredLoweringCorrespondenceRef {
                correspondence_slot: Correction::IDENTITY.to_owned(),
            },
            postcondition: DeclaredAftermathPostcondition::ExactPriorTruth,
            preimage_demand,
        }
    }

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
