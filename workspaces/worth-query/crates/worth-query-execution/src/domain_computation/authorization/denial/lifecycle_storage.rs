//! Native allocation/control cause for retained lifecycle authority storage.
use super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind as Kind,
};
use worth_execution::{ExecutionAllocationDenial, ExecutionAllocationDenialClass as Class};
/// Exact original storage stop; shared only when the denial itself is cloned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum LifecycleStorageCause {
    Allocation(ExecutionAllocationDenial),
    Request(worth_execution::WorkCeilingDenial),
}
impl std::fmt::Display for LifecycleStorageCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lifecycle binding storage refused: {self:?}")
    }
}
impl std::error::Error for LifecycleStorageCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(cause) => Some(cause),
            Self::Request(_) => None,
        }
    }
}
impl WorthQueryOperationAuthorizationDenial {
    pub(in crate::domain_computation::authorization) fn lifecycle_storage_allocation(
        cause: ExecutionAllocationDenial,
        subject: &str,
    ) -> Self {
        let mut denial = match cause.classify() {
            Class::Request(stop) => super::relational_read::denial(stop.into(), subject),
            Class::Physical(_) => Self::new(Kind::Stopped(Ok(
                crate::domain_computation::WorthQueryProviderSessionDenialKind::AllocationDenied,
            )), subject),
        };
        denial.lifecycle_storage_cause = Some(std::sync::Arc::new(
            LifecycleStorageCause::Allocation(cause),
        ));
        denial
    }
    pub(in crate::domain_computation::authorization) fn lifecycle_storage_request(
        cause: worth_execution::WorkCeilingDenial,
        subject: &str,
    ) -> Self {
        let mut denial = super::relational_read::denial(cause.into(), subject);
        denial.lifecycle_storage_cause =
            Some(std::sync::Arc::new(LifecycleStorageCause::Request(cause)));
        denial
    }
    /// Original request/control cause when lifecycle storage admission stopped.
    pub fn storage_request_denial(&self) -> Option<worth_execution::WorkCeilingDenial> {
        match self.lifecycle_storage_cause.as_deref() {
            Some(LifecycleStorageCause::Request(cause)) => Some(*cause),
            _ => None,
        }
    }
    /// Original allocation refusal when lifecycle authority could not be retained.
    pub fn allocation_denial(&self) -> Option<&ExecutionAllocationDenial> {
        match self.lifecycle_storage_cause.as_deref() {
            Some(LifecycleStorageCause::Allocation(cause)) => Some(cause),
            _ => None,
        }
    }
}
