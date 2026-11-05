//! Moved whole from `domain_computation::primary_graph::application_query::projection::projected_tree::content_comparison`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionContentComparisonFault {
    CapacityOverflow,
    AllocationFailed,
    ContentInconsistency,
}

impl ProjectionContentComparisonFault {
    pub(in crate::domain_computation::primary_graph) fn into_native(
        self,
    ) -> worth_relational::facade::transactions::RelationalExecutionDenialCause {
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
        match self {
            Self::CapacityOverflow => Native::ChargedBytesOverflow,
            Self::AllocationFailed => Native::AllocationFailed,
            Self::ContentInconsistency => Native::ContentInconsistency,
        }
    }
}
