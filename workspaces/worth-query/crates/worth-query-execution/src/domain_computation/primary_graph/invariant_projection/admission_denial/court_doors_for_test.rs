//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInvariantProjectionDenial {
    #[doc(hidden)]
    pub fn execution_request_stopped_for_test(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
        work: Option<super::super::WorthQueryInvariantProjectionWork>,
    ) -> Self {
        Self::execution_request_stopped(cause, work)
    }
}
