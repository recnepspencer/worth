use super::{correspondence, WorthUiPresentationAsyncObservation};

type CorrespondenceAuthority = std::sync::Arc<correspondence::PresentationCorrespondenceAuthority>;
type PresentationAttempt = worth_ui_host_contract::UiMountedPresentationAttemptIdentity;
type SurfaceBinding = worth_ui_host_contract::UiSurfaceBindingGeneration;

#[derive(Debug)]
pub struct WorthUiPresentationPendingReceipt {
    pub(super) authority: CorrespondenceAuthority,
    pub(super) attempt: PresentationAttempt,
    pub(super) binding: SurfaceBinding,
    pub(super) observation: WorthUiPresentationAsyncObservation,
    pub(super) nonce: u64,
}

#[derive(Debug)]
pub struct WorthUiPresentationIncompleteAdmission {
    pub(super) authority: CorrespondenceAuthority,
    pub(super) attempt: PresentationAttempt,
    pub(super) binding: SurfaceBinding,
    pub(super) nonce: u64,
}

#[derive(Debug)]
pub struct WorthUiPresentationCleanupRecovery {
    pub(super) authority: CorrespondenceAuthority,
    pub(super) attempt: PresentationAttempt,
    pub(super) binding: SurfaceBinding,
    pub(super) nonce: u64,
}

#[derive(Debug)]
pub enum WorthUiPresentationAdmissionRecovery {
    Incomplete(WorthUiPresentationIncompleteAdmission),
    Cleanup(WorthUiPresentationCleanupRecovery),
}

#[derive(Debug)]
pub enum WorthUiPresentationRecoveryReceipt {
    Pending(WorthUiPresentationPendingReceipt),
    Admission(WorthUiPresentationAdmissionRecovery),
}

pub struct WorthUiPresentationPresentedReceipt {
    pub(super) observation: WorthUiPresentationAsyncObservation,
    pub(super) predecessor_observation: Option<WorthUiPresentationAsyncObservation>,
}

pub struct WorthUiPresentationUnresolvedReceipt {
    pub(super) authority: CorrespondenceAuthority,
    pub(super) attempt: PresentationAttempt,
    pub(super) binding: SurfaceBinding,
    pub(super) nonce: u64,
    pub(super) observation: WorthUiPresentationAsyncObservation,
}

pub struct WorthUiPresentationRecoveryRequiredReceipt {
    pub(super) attempt: PresentationAttempt,
    pub(super) binding: SurfaceBinding,
    pub(super) observation: WorthUiPresentationAsyncObservation,
}

impl WorthUiPresentationPendingReceipt {
    pub const fn attempt(&self) -> PresentationAttempt {
        self.attempt
    }

    pub const fn binding(&self) -> SurfaceBinding {
        self.binding
    }

    pub const fn observation(&self) -> WorthUiPresentationAsyncObservation {
        self.observation
    }
}

impl WorthUiPresentationIncompleteAdmission {
    pub const fn attempt(&self) -> PresentationAttempt {
        self.attempt
    }

    pub const fn binding(&self) -> SurfaceBinding {
        self.binding
    }
}

impl WorthUiPresentationCleanupRecovery {
    pub const fn attempt(&self) -> PresentationAttempt {
        self.attempt
    }

    pub const fn binding(&self) -> SurfaceBinding {
        self.binding
    }
}

impl WorthUiPresentationAdmissionRecovery {
    pub const fn incomplete(&self) -> Option<&WorthUiPresentationIncompleteAdmission> {
        match self {
            Self::Incomplete(receipt) => Some(receipt),
            Self::Cleanup(_) => None,
        }
    }

    pub const fn attempt(&self) -> PresentationAttempt {
        match self {
            Self::Incomplete(receipt) => receipt.attempt(),
            Self::Cleanup(receipt) => receipt.attempt(),
        }
    }

    pub const fn binding(&self) -> SurfaceBinding {
        match self {
            Self::Incomplete(receipt) => receipt.binding(),
            Self::Cleanup(receipt) => receipt.binding(),
        }
    }
}

impl WorthUiPresentationRecoveryReceipt {
    pub const fn pending(&self) -> Option<&WorthUiPresentationPendingReceipt> {
        match self {
            Self::Pending(receipt) => Some(receipt),
            Self::Admission(_) => None,
        }
    }

    pub const fn attempt(&self) -> PresentationAttempt {
        match self {
            Self::Pending(receipt) => receipt.attempt(),
            Self::Admission(receipt) => receipt.attempt(),
        }
    }

    pub const fn binding(&self) -> SurfaceBinding {
        match self {
            Self::Pending(receipt) => receipt.binding(),
            Self::Admission(receipt) => receipt.binding(),
        }
    }
}

impl From<WorthUiPresentationPendingReceipt> for WorthUiPresentationRecoveryReceipt {
    fn from(receipt: WorthUiPresentationPendingReceipt) -> Self {
        Self::Pending(receipt)
    }
}

impl From<WorthUiPresentationAdmissionRecovery> for WorthUiPresentationRecoveryReceipt {
    fn from(receipt: WorthUiPresentationAdmissionRecovery) -> Self {
        Self::Admission(receipt)
    }
}

impl WorthUiPresentationPresentedReceipt {
    pub const fn observation(&self) -> WorthUiPresentationAsyncObservation {
        self.observation
    }

    pub const fn predecessor_observation(&self) -> Option<WorthUiPresentationAsyncObservation> {
        self.predecessor_observation
    }
}

impl WorthUiPresentationUnresolvedReceipt {
    pub const fn attempt(&self) -> PresentationAttempt {
        self.attempt
    }

    pub const fn binding(&self) -> SurfaceBinding {
        self.binding
    }

    pub const fn observation(&self) -> WorthUiPresentationAsyncObservation {
        self.observation
    }
}

impl WorthUiPresentationRecoveryRequiredReceipt {
    pub const fn attempt(&self) -> PresentationAttempt {
        self.attempt
    }

    pub const fn binding(&self) -> SurfaceBinding {
        self.binding
    }

    pub const fn observation(&self) -> WorthUiPresentationAsyncObservation {
        self.observation
    }
}
