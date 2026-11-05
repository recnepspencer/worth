//! Moved whole from `domain_computation::primary_graph::invariant_projection::consumed_output::verification`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::capacity_stop::VerificationCapacityStop;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::mvcc::RelationalInterruptionEvent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumedOutputVerification {
    Current,
    ChangedDirectFact(usize),
    ChangedUpstream,
    /// Native pending state, prior to ordinary driving-policy translation.
    PendingUpstream,
    /// A recorded output witness moved, not merely its input source facts.
    ChangedOutput,
    /// Evidence is from a different or later native source position.
    Unaligned,
}

impl ConsumedOutputVerification {
    pub(in crate::domain_computation::primary_graph) fn driving(
        self,
    ) -> Result<Self, ConsumedOutputVerificationStop> {
        match self {
            Self::PendingUpstream => Err(ConsumedOutputVerificationStop::PendingUpstream),
            Self::ChangedOutput | Self::Unaligned => Ok(Self::ChangedUpstream),
            other => Ok(other),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsumedOutputVerificationStop {
    ManagedIndex(crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    SourceQueryAuthorization(
        crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
    ),
    Unavailable,
    CapacityExhausted(VerificationCapacityStop),
    PendingUpstream,
    RetryCurrentness(CompanionCellEditStop),
    WorkExhausted,
    /// The request was cancelled or ran out of time while verifying. The
    /// evidence was neither found current nor found missing.
    Interrupted(RelationalInterruptionEvent),
}

impl std::fmt::Display for ConsumedOutputVerificationStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native output verification: {self:?}")
    }
}

impl std::error::Error for ConsumedOutputVerificationStop {}
