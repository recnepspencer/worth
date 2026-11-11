//! Installed correction-mechanism axis.
mod preimage_relation;
pub use preimage_relation::InstalledPreImageRelationLocus;

mod compensation;
mod recorded_inverse;

pub use compensation::InstalledCompensation;
pub use recorded_inverse::{
    InstalledLoweringCorrespondenceRef, InstalledPreImageDemand, InstalledPreImageLocus,
    InstalledRecordedInverse,
};

use worth_query_declaration::facade::application_aftermath::PortableCorrectionMechanism;

use super::lowering_correspondence::InstalledLoweringCorrespondence;

/// Installed correction mechanism for one aftermath contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstalledCorrectionMechanism {
    RecordedInverse(InstalledRecordedInverse),
    Compensation(InstalledCompensation),
}

impl InstalledCorrectionMechanism {
    pub(crate) fn from_declared_compensation(
        declared: &worth_query_declaration::facade::application_aftermath::DeclaredCompensation,
    ) -> Self {
        Self::Compensation(InstalledCompensation::from_declared(declared))
    }

    pub(crate) fn from_portable_recorded_inverse(
        portable: &worth_query_declaration::facade::application_aftermath::PortableRecordedInverse,
        lowering_correspondence: InstalledLoweringCorrespondence,
    ) -> Self {
        Self::RecordedInverse(InstalledRecordedInverse::from_portable(
            portable,
            lowering_correspondence,
        ))
    }

    pub(crate) fn from_portable(
        portable: &PortableCorrectionMechanism,
        lowering_correspondence: Option<InstalledLoweringCorrespondence>,
    ) -> Result<Self, &'static str> {
        match portable {
            PortableCorrectionMechanism::RecordedInverse(inverse) => {
                let correspondence = lowering_correspondence
                    .ok_or("recorded-inverse-requires-resolved-lowering-correspondence")?;
                Ok(Self::from_portable_recorded_inverse(
                    inverse,
                    correspondence,
                ))
            }
            PortableCorrectionMechanism::Compensation(compensation) => {
                Ok(Self::from_declared_compensation(compensation))
            }
        }
    }
}
