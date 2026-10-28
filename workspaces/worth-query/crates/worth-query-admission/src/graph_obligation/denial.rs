use crate::domain_computation::execution_resource_admission::WorthQueryExecutionResourceAdmissionDenial;
use crate::graph_read_access::WorthQueryGraphReadPlanReviewDenialKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphObligationSelectionDenialKind {
    SubjectKindMismatch,
    MutationAuthorityRequired,
    ReadOnlyIntentCannotSelectMutation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphObligationSelectionDenial {
    kind: WorthQueryGraphObligationSelectionDenialKind,
    subject: String,
}

impl WorthQueryGraphObligationSelectionDenial {
    pub(super) fn new(
        kind: WorthQueryGraphObligationSelectionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryGraphObligationSelectionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

#[derive(Debug)]
pub enum WorthQueryGraphWorkAdmissionDenial {
    Selection(WorthQueryGraphObligationSelectionDenial),
    IntentMismatch,
    UnsupportedOwner,
    GraphReadRequirementMismatch,
    GraphReadPlan(WorthQueryGraphReadPlanReviewDenialKind),
    ExecutionResource(WorthQueryExecutionResourceAdmissionDenial),
    ProviderSupportUnavailable,
    OperationAuthorityMismatch,
    ResourceAuthorityMismatch,
    InspectionAuthorityMismatch,
    CapacityUnavailable,
    IdentityExhausted,
}

impl worth_execution::ChargedBytes for WorthQueryGraphObligationSelectionDenial {
    fn additional_charged_bytes(&self) -> u64 {
        u64::try_from(self.subject.capacity()).unwrap_or(u64::MAX)
    }
}

impl worth_execution::ChargedBytes for WorthQueryGraphWorkAdmissionDenial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Selection(cause) => {
                worth_execution::ChargedBytes::additional_charged_bytes(cause)
            }
            Self::ExecutionResource(cause) => {
                worth_execution::ChargedBytes::additional_charged_bytes(cause)
            }
            Self::IntentMismatch
            | Self::UnsupportedOwner
            | Self::GraphReadRequirementMismatch
            | Self::GraphReadPlan(_)
            | Self::ProviderSupportUnavailable
            | Self::OperationAuthorityMismatch
            | Self::ResourceAuthorityMismatch
            | Self::InspectionAuthorityMismatch
            | Self::CapacityUnavailable
            | Self::IdentityExhausted => 0,
        }
    }
}
