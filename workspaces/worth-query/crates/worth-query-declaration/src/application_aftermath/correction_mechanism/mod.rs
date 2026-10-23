//! Declared correction-mechanism axis.
//!
//! Populated with exactly `RecordedInverse` and `Compensation` for Milestone
//! 9.16. Deterministic re-derivation is a committed future sibling under this
//! parent axis and must not appear as an empty placeholder file.

mod compensation;
mod portable_recorded_inverse;
mod recorded_inverse;

pub use compensation::DeclaredCompensation;
pub use portable_recorded_inverse::{
    PortablePreImageDemand, PortablePreImageLocus, PortableRecordedInverse,
};
pub use recorded_inverse::{
    DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand, DeclaredPreImageDemandDenial,
    DeclaredPreImageLocus, DeclaredRecordedInverse,
};

/// How the corrected state is produced when correction authority permits it.
pub enum DeclaredCorrectionMechanism<Schema> {
    RecordedInverse(DeclaredRecordedInverse<Schema>),
    Compensation(DeclaredCompensation),
}

impl<Schema: crate::application_schema::ApplicationSchema> DeclaredCorrectionMechanism<Schema> {
    /// Derives the inverse operation and exact capture from one typed binding.
    pub fn recorded_inverse_binding<C: super::ApplicationCorrectionBinding<Schema>>(
        demand: DeclaredPreImageDemand<Schema>,
    ) -> Self {
        Self::RecordedInverse(DeclaredRecordedInverse::from_binding::<C>(demand))
    }

    /// Derives the actual compensating operation from its concrete mutation binding.
    pub fn compensation_binding<C: super::ApplicationCorrectionBinding<Schema>>(
        postcondition: super::DeclaredAftermathPostcondition,
    ) -> Result<Self, &'static str> {
        use crate::application_operation::ApplicationMutationBinding;
        let action = super::ApplicationInverseBinding::<Schema, C>::descriptor();
        DeclaredCompensation::new(action.operation_name(), postcondition).map(Self::Compensation)
    }
}

impl<Schema> Clone for DeclaredCorrectionMechanism<Schema> {
    fn clone(&self) -> Self {
        match self {
            Self::RecordedInverse(inverse) => Self::RecordedInverse(inverse.clone()),
            Self::Compensation(compensation) => Self::Compensation(compensation.clone()),
        }
    }
}

impl<Schema> std::fmt::Debug for DeclaredCorrectionMechanism<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecordedInverse(inverse) => formatter
                .debug_tuple("RecordedInverse")
                .field(inverse)
                .finish(),
            Self::Compensation(compensation) => formatter
                .debug_tuple("Compensation")
                .field(compensation)
                .finish(),
        }
    }
}

impl<Schema> PartialEq for DeclaredCorrectionMechanism<Schema> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::RecordedInverse(left), Self::RecordedInverse(right)) => left == right,
            (Self::Compensation(left), Self::Compensation(right)) => left == right,
            _ => false,
        }
    }
}

impl<Schema> Eq for DeclaredCorrectionMechanism<Schema> {}

impl<Schema> PartialOrd for DeclaredCorrectionMechanism<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredCorrectionMechanism<Schema> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::RecordedInverse(left), Self::RecordedInverse(right)) => left.cmp(right),
            (Self::Compensation(left), Self::Compensation(right)) => left.cmp(right),
            (Self::RecordedInverse(_), Self::Compensation(_)) => std::cmp::Ordering::Less,
            (Self::Compensation(_), Self::RecordedInverse(_)) => std::cmp::Ordering::Greater,
        }
    }
}

impl<Schema> DeclaredCorrectionMechanism<Schema> {
    pub(super) fn into_portable(self) -> PortableCorrectionMechanism {
        match self {
            Self::RecordedInverse(inverse) => {
                PortableCorrectionMechanism::RecordedInverse(inverse.into_portable())
            }
            Self::Compensation(compensation) => {
                PortableCorrectionMechanism::Compensation(compensation)
            }
        }
    }
}

/// Public-read correction meaning available only after schema association.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PortableCorrectionMechanism {
    RecordedInverse(PortableRecordedInverse),
    Compensation(DeclaredCompensation),
}
