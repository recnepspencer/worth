//! Sealed test doors over this module's private members.
use super::*;

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn open_control_for_test( placement: RuntimeWorldExecutionPlacement<'a>, cancellation: worth_execution::CancellationToken, deadline: Option<std::time::Instant>) -> Self {
        Self::open_control(placement, cancellation, deadline)
    }
}
