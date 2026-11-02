use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryDelegationActivationBinding,
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
};

// Lifecycle predecessors retain actual publication custody. Their charged backing
// belongs only to the admitted lifecycle; ordinary admissions carry no allocation
// and do not embed every possible predecessor in read sets and effect programs.
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
        binding: Box<WorthQueryElevationApprovalBinding>,
        // Release physical backing before its original-request reservation.
        _binding_memory: Option<worth_execution::ExecutionMemoryReservation>,
    },
    ElevationClose {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: Box<WorthQueryElevationCloseBinding>,
        // Release physical backing before its original-request reservation.
        _binding_memory: Option<worth_execution::ExecutionMemoryReservation>,
    },
    MandatoryReview {
        input:
            worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
        binding: Box<WorthQueryMandatoryReviewBinding>,
        // Release physical backing before its original-request reservation.
        _binding_memory: Option<worth_execution::ExecutionMemoryReservation>,
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
