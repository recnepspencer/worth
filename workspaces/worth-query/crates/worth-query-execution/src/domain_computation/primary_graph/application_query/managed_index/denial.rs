//! Owned refusal and last-owner custody for the new material receiver.
use super::execution::ProviderExecutionDenial;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledQueryBindingAdmissionStop;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as ExecutionCause;

mod causes;
pub use causes::{WorthQueryManagedIndexDenialCause, WorthQueryManagedIndexRefusal};

pub(super) enum ReceiverCause {
    Provider(ProviderExecutionDenial),
    Execution(ExecutionCause),
    Read(super::super::WorthQueryApplicationOneShotDenial),
    Installation(WorthQueryInstalledQueryBindingAdmissionStop<ExecutionCause>),
    Product(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Refused(WorthQueryManagedIndexRefusal),
}
impl From<ExecutionCause> for ReceiverCause {
    fn from(cause: ExecutionCause) -> Self {
        Self::Execution(cause)
    }
}
impl From<crate::basis::WorthQueryProductBranchAdmissionDenial> for ReceiverCause {
    fn from(cause: crate::basis::WorthQueryProductBranchAdmissionDenial) -> Self {
        Self::Product(cause)
    }
}

/// A failed managed-index operation with its exact owned cause and material custody.
pub struct WorthQueryManagedIndexDenial {
    cause: ReceiverCause,
    // Owned diagnostic backing drops before the actual original grant.
    _allocation: Option<Arc<SourceFactsAllocation>>,
}
impl WorthQueryManagedIndexDenial {
    pub(super) fn new(
        cause: ReceiverCause,
        allocation: Option<Arc<SourceFactsAllocation>>,
    ) -> Self {
        Self {
            cause,
            _allocation: allocation,
        }
    }
    /// Borrows the original typed cause; no session or root authority is exposed.
    pub fn cause(&self) -> WorthQueryManagedIndexDenialCause<'_> {
        use WorthQueryManagedIndexDenialCause as Cause;
        match &self.cause {
            ReceiverCause::Provider(cause) => causes::provider(cause),
            ReceiverCause::Execution(cause) => Cause::Execution(cause),
            ReceiverCause::Read(cause) => Cause::QueryRead(cause),
            ReceiverCause::Product(cause) => Cause::Product(cause),
            ReceiverCause::Refused(cause) => Cause::Refused(*cause),
            ReceiverCause::Installation(cause) => match cause {
                WorthQueryInstalledQueryBindingAdmissionStop::Installation(cause) => {
                    Cause::Installation(cause)
                }
                WorthQueryInstalledQueryBindingAdmissionStop::Admission(cause) => {
                    Cause::Execution(cause)
                }
                WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow => {
                    Cause::Refused(WorthQueryManagedIndexRefusal::InstallationAccountingOverflow)
                }
            },
        }
    }
}
impl std::fmt::Debug for WorthQueryManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ManagedIndexDenial")
            .field(&self.cause())
            .finish()
    }
}
impl std::fmt::Display for WorthQueryManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "managed index refused: {:?}", self.cause())
    }
}
impl std::error::Error for WorthQueryManagedIndexDenial {}
