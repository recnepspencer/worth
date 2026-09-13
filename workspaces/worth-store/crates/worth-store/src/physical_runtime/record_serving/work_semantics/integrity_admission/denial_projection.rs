use super::{CleanExtentAdmissionDenial, CleanInlineAdmissionDenial};
use crate::physical_runtime::record_serving::{
    RecordReadDenial, RecordReadWorkDenial, RecordStreamFailureKind, StalePhysicalRecordPlacement,
};

impl CleanInlineAdmissionDenial {
    pub(in crate::physical_runtime::record_serving) const fn preserves_resident_bytes(
        self,
    ) -> bool {
        matches!(
            self,
            Self::Unavailable | Self::RuntimeReleased | Self::Residency(_)
        )
    }

    pub(in crate::physical_runtime::record_serving) fn read_denial(self) -> RecordReadDenial {
        match self {
            Self::PageIdentity => {
                RecordReadDenial::StalePlacement(StalePhysicalRecordPlacement::PageIdentity)
            }
            Self::SlotGeneration => {
                RecordReadDenial::StalePlacement(StalePhysicalRecordPlacement::SlotGeneration)
            }
            Self::Format => RecordReadDenial::FormatMismatch,
            Self::Unavailable => RecordReadDenial::ArtifactUnavailable,
            Self::RuntimeReleased => {
                RecordReadDenial::PhysicalWork(RecordReadWorkDenial::RuntimeReleased)
            }
            Self::Residency(reason) => RecordReadDenial::from_residency(reason),
            Self::Damaged => RecordReadDenial::ArtifactDamaged,
        }
    }
}

impl CleanExtentAdmissionDenial {
    pub(in crate::physical_runtime::record_serving) const fn preserves_resident_bytes(
        self,
    ) -> bool {
        matches!(
            self,
            Self::Unavailable | Self::RuntimeReleased | Self::Residency(_)
        )
    }

    pub(in crate::physical_runtime::record_serving) fn read_denial(self) -> RecordReadDenial {
        match self {
            Self::ExtentMembership => {
                RecordReadDenial::StalePlacement(StalePhysicalRecordPlacement::ExtentMembership)
            }
            Self::Format => RecordReadDenial::FormatMismatch,
            Self::Unavailable => RecordReadDenial::ArtifactUnavailable,
            Self::RuntimeReleased => {
                RecordReadDenial::PhysicalWork(RecordReadWorkDenial::RuntimeReleased)
            }
            Self::Residency(reason) => RecordReadDenial::from_residency(reason),
            Self::Damaged => RecordReadDenial::ArtifactDamaged,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn stream_failure_kind(
        self,
    ) -> RecordStreamFailureKind {
        match self {
            Self::ExtentMembership => RecordStreamFailureKind::StalePlacement,
            Self::Format => RecordStreamFailureKind::FormatMismatch,
            Self::Unavailable => RecordStreamFailureKind::ArtifactUnavailable,
            Self::RuntimeReleased => RecordStreamFailureKind::RuntimeReleased,
            Self::Residency(reason) => RecordStreamFailureKind::ResidencyUnavailable(reason.into()),
            Self::Damaged => RecordStreamFailureKind::ArtifactDamaged,
        }
    }
}
