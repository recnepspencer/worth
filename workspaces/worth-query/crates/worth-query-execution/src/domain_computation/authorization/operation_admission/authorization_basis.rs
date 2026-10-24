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
        binding: WorthQueryDelegationActivationBinding,
    },
    CapabilityRevocation {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: WorthQueryCapabilityRevocationBinding,
    },
    ElevationRequest {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: WorthQueryElevationRequestBinding,
    },
    ElevationApproval {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: WorthQueryElevationApprovalBinding,
    },
    ElevationClose {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: WorthQueryElevationCloseBinding,
    },
    MandatoryReview {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: WorthQueryMandatoryReviewBinding,
    },
}

impl<Input> WorthQueryOperationAuthorizationBasis<Input> {
    pub(super) fn capability_input(&self) -> Option<&Input> {
        self.shared_capability_input().map(AsRef::as_ref)
    }

    pub(super) fn shared_capability_input(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
    > {
        match self {
            Self::Conventional => None,
            Self::Capability { input }
            | Self::DelegationActivation { input, .. }
            | Self::CapabilityRevocation { input, .. }
            | Self::ElevationRequest { input, .. }
            | Self::ElevationApproval { input, .. }
            | Self::ElevationClose { input, .. }
            | Self::MandatoryReview { input, .. } => Some(input),
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
