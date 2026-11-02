//! The authorization mode an admission carries, without its typed input.
use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryDelegationActivationBinding,
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
};

/// Every mode but `Conventional` was admitted from a capability request, so
/// its brand carries that request's typed input beside the core.
pub(super) enum ErasedAuthorizationBasis {
    Conventional,
    Capability,
    DelegationActivation {
        binding: WorthQueryDelegationActivationBinding,
    },
    CapabilityRevocation {
        binding: WorthQueryCapabilityRevocationBinding,
    },
    ElevationRequest {
        binding: WorthQueryElevationRequestBinding,
    },
    ElevationApproval {
        binding: WorthQueryElevationApprovalBinding,
    },
    ElevationClose {
        binding: WorthQueryElevationCloseBinding,
    },
    MandatoryReview {
        binding: WorthQueryMandatoryReviewBinding,
    },
}

impl ErasedAuthorizationBasis {
    pub(super) const fn carries_capability_input(&self) -> bool {
        !matches!(self, Self::Conventional)
    }

    pub(super) const fn is_elevation_lifecycle(&self) -> bool {
        matches!(
            self,
            Self::ElevationRequest { .. }
                | Self::ElevationApproval { .. }
                | Self::ElevationClose { .. }
                | Self::MandatoryReview { .. }
        )
    }
}
