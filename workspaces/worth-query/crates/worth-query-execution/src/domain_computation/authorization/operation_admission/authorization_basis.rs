mod lifecycle_binding;
pub(super) use lifecycle_binding::LifecycleBinding;

use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryDelegationActivationBinding,
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
};

pub(in crate::domain_computation) enum WorthQueryOperationAuthorizationBasis<Input> {
    Conventional,
    Capability {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
    },
    DelegationActivation {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: LifecycleBinding<WorthQueryDelegationActivationBinding>,
    },
    CapabilityRevocation {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: LifecycleBinding<WorthQueryCapabilityRevocationBinding>,
    },
    ElevationRequest {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: LifecycleBinding<WorthQueryElevationRequestBinding>,
    },
    ElevationApproval {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: LifecycleBinding<WorthQueryElevationApprovalBinding>,
    },
    ElevationClose {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: LifecycleBinding<WorthQueryElevationCloseBinding>,
    },
    MandatoryReview {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: LifecycleBinding<WorthQueryMandatoryReviewBinding>,
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
