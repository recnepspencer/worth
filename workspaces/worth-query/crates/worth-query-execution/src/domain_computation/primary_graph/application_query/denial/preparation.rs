//! Inline diagnostic context for a refused preparation quote, never admission.
use worth_query_admission::integration::WorthQueryCanonicalIdentityPreparationContext;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum QueryPreparationOperation {
    FinishPermissionVisit,
    GraphNameBacking,
    Requirements,
    SupportInventory,
    ObligationRetention,
    ObligationIdentity,
    ObligationSelection,
    ResourceBudget,
    GraphReview,
    WorkLimitReview,
    Capacity,
    SessionHeader,
    SessionIdentityBacking,
    NativeSessionHeader,
    NativeSessionCopies,
    ParameterCanonicalPreparation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct QueryPreparationContext {
    pub operation: QueryPreparationOperation,
    pub requested_work: u64,
    pub requested_bytes: u64,
    pub canonical: Option<WorthQueryCanonicalIdentityPreparationContext>,
}

impl QueryPreparationContext {
    pub(in crate::domain_computation::primary_graph) const fn new(
        operation: QueryPreparationOperation,
        requested_work: u64,
        requested_bytes: u64,
    ) -> Self {
        Self {
            operation,
            requested_work,
            requested_bytes,
            canonical: None,
        }
    }
}
