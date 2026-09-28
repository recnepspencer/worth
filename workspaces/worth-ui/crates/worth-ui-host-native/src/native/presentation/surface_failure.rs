#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeSurfaceFailureDisposition {
    RetryAfterTimeout,
    WaitForVisibility,
    ValidationRejected,
    ReconstructionRequired(UiNativeSurfaceRecovery),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeSurfaceRecovery {
    Outdated,
    Lost,
    DeviceLost,
}

impl UiNativeSurfaceRecovery {
    pub(crate) const fn cause(self) -> crate::native::UiNativeRecoveryCause {
        match self {
            Self::Outdated => crate::native::UiNativeRecoveryCause::SurfaceOutdated,
            Self::Lost => crate::native::UiNativeRecoveryCause::SurfaceLost,
            Self::DeviceLost => crate::native::UiNativeRecoveryCause::DeviceLost,
        }
    }

    #[cfg(feature = "certification-support")]
    const fn public_class(self) -> UiNativePresentationRecoveryClass {
        match self {
            Self::Outdated => UiNativePresentationRecoveryClass::SurfaceOutdated,
            Self::Lost => UiNativePresentationRecoveryClass::SurfaceLost,
            Self::DeviceLost => UiNativePresentationRecoveryClass::DeviceLost,
        }
    }
}

pub(crate) const fn classify_surface_failure(
    failure: super::UiNativeSurfaceAcquireFailure,
) -> UiNativeSurfaceFailureDisposition {
    match failure {
        super::UiNativeSurfaceAcquireFailure::Outdated => {
            UiNativeSurfaceFailureDisposition::ReconstructionRequired(
                UiNativeSurfaceRecovery::Outdated,
            )
        }
        super::UiNativeSurfaceAcquireFailure::Lost => {
            UiNativeSurfaceFailureDisposition::ReconstructionRequired(UiNativeSurfaceRecovery::Lost)
        }
        super::UiNativeSurfaceAcquireFailure::DeviceLost => {
            UiNativeSurfaceFailureDisposition::ReconstructionRequired(
                UiNativeSurfaceRecovery::DeviceLost,
            )
        }
        super::UiNativeSurfaceAcquireFailure::Timeout => {
            UiNativeSurfaceFailureDisposition::RetryAfterTimeout
        }
        super::UiNativeSurfaceAcquireFailure::Occluded => {
            UiNativeSurfaceFailureDisposition::WaitForVisibility
        }
        super::UiNativeSurfaceAcquireFailure::Validation => {
            UiNativeSurfaceFailureDisposition::ValidationRejected
        }
    }
}

#[cfg(feature = "certification-support")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePresentationFault {
    Timeout,
    Occluded,
    Outdated,
    SurfaceLost,
    Validation,
    DeviceLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePresentationRecoveryClass {
    #[cfg(feature = "certification-support")]
    Resize,
    Dpi,
    #[cfg(feature = "certification-support")]
    SurfaceOutdated,
    #[cfg(feature = "certification-support")]
    SurfaceLost,
    #[cfg(feature = "certification-support")]
    DeviceLost,
    #[cfg(feature = "certification-support")]
    PresentationIndeterminate,
}

#[must_use = "fault disposition determines the required retry, wait, rejection, or recovery action"]
#[cfg(feature = "certification-support")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePresentationFaultDisposition {
    RetryAfterTimeout,
    WaitForVisibility,
    ValidationRejected,
    ReconstructionRequired(UiNativePresentationRecoveryClass),
}

#[cfg(feature = "certification-support")]
pub fn classify_presentation_fault(
    fault: UiNativePresentationFault,
) -> UiNativePresentationFaultDisposition {
    let internal = match fault {
        UiNativePresentationFault::Timeout => super::UiNativeSurfaceAcquireFailure::Timeout,
        UiNativePresentationFault::Occluded => super::UiNativeSurfaceAcquireFailure::Occluded,
        UiNativePresentationFault::Outdated => super::UiNativeSurfaceAcquireFailure::Outdated,
        UiNativePresentationFault::SurfaceLost => super::UiNativeSurfaceAcquireFailure::Lost,
        UiNativePresentationFault::Validation => super::UiNativeSurfaceAcquireFailure::Validation,
        UiNativePresentationFault::DeviceLost => super::UiNativeSurfaceAcquireFailure::DeviceLost,
    };
    match classify_surface_failure(internal) {
        UiNativeSurfaceFailureDisposition::RetryAfterTimeout => {
            UiNativePresentationFaultDisposition::RetryAfterTimeout
        }
        UiNativeSurfaceFailureDisposition::WaitForVisibility => {
            UiNativePresentationFaultDisposition::WaitForVisibility
        }
        UiNativeSurfaceFailureDisposition::ValidationRejected => {
            UiNativePresentationFaultDisposition::ValidationRejected
        }
        UiNativeSurfaceFailureDisposition::ReconstructionRequired(recovery) => {
            UiNativePresentationFaultDisposition::ReconstructionRequired(recovery.public_class())
        }
    }
}
