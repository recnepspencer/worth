use worth_store_buffer_pool::CleanFrameIntegrityValidationDenial;
use worth_store_physical_integrity::PhysicalIntegrityRejection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum ResidentIntegrityAdmissionDenial {
    SourceScopeMismatch,
    SourceIncarnationMismatch,
    LifecycleGenerationChanged,
    FrameGenerationChanged,
    RetainedRecordInvalidated,
    RetainedRecordChanged,
    Validation(PhysicalIntegrityRejection),
    BootstrapScopeMismatch(worth_store_physical_integrity::BootstrapCatalogScopeMismatch),
    BootstrapUnsupportedFormat(worth_store_physical_integrity::BootstrapCatalogUnsupportedFormat),
    Frame(CleanFrameIntegrityValidationDenial),
}

impl ResidentIntegrityAdmissionDenial {
    pub(in crate::physical_runtime) const fn preserves_resident_bytes(self) -> bool {
        matches!(
            self,
            Self::LifecycleGenerationChanged
                | Self::SourceIncarnationMismatch
                | Self::FrameGenerationChanged
                | Self::RetainedRecordInvalidated
                | Self::RetainedRecordChanged
        ) || self.residency_unavailability().is_some()
    }

    pub(in crate::physical_runtime) const fn residency_unavailability(
        self,
    ) -> Option<worth_store_buffer_pool::PhysicalResidencyDenial> {
        use worth_store_buffer_pool::PhysicalResidencyDenial;

        match self {
            Self::FrameGenerationChanged
            | Self::Frame(CleanFrameIntegrityValidationDenial::FrameNotResident)
            | Self::Frame(CleanFrameIntegrityValidationDenial::FrameGenerationChanged) => {
                Some(PhysicalResidencyDenial::FrameNotResident)
            }
            Self::Frame(CleanFrameIntegrityValidationDenial::PoolClosed) => {
                Some(PhysicalResidencyDenial::PoolClosed)
            }
            Self::Frame(CleanFrameIntegrityValidationDenial::FrameBytesChanged)
            | Self::Frame(CleanFrameIntegrityValidationDenial::FrameDirty) => {
                Some(PhysicalResidencyDenial::FrameDirty)
            }
            _ => None,
        }
    }
}
