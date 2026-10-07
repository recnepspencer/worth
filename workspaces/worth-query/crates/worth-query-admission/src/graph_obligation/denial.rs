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
    CapacityUnavailable,
    IdentityExhausted,
}
