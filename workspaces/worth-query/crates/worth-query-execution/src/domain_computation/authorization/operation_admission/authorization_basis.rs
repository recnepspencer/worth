use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryDelegationActivationBinding,
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
};

pub(in crate::domain_computation) enum WorthQueryOperationAuthorizationBasis<Input> {
    Conventional,
    Capability {
        input: std::sync::Arc<Input>,
    },
    DelegationActivation {
        input: std::sync::Arc<Input>,
        binding: WorthQueryDelegationActivationBinding,
    },
    CapabilityRevocation {
        input: std::sync::Arc<Input>,
        binding: WorthQueryCapabilityRevocationBinding,
    },
    ElevationRequest {
        input: std::sync::Arc<Input>,
        binding: WorthQueryElevationRequestBinding,
    },
    ElevationApproval {
        input: std::sync::Arc<Input>,
        binding: WorthQueryElevationApprovalBinding,
    },
    ElevationClose {
        input: std::sync::Arc<Input>,
        binding: WorthQueryElevationCloseBinding,
    },
    MandatoryReview {
        input: std::sync::Arc<Input>,
        binding: WorthQueryMandatoryReviewBinding,
    },
}

impl<Input> WorthQueryOperationAuthorizationBasis<Input> {
    pub(super) fn capability_input(&self) -> Option<&Input> {
        match self {
            Self::Conventional => None,
            Self::Capability { input }
            | Self::DelegationActivation { input, .. }
            | Self::CapabilityRevocation { input, .. }
            | Self::ElevationRequest { input, .. }
            | Self::ElevationApproval { input, .. }
            | Self::ElevationClose { input, .. }
            | Self::MandatoryReview { input, .. } => Some(input.as_ref()),
        }
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
