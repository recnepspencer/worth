//! Borrow typed request controls without consuming suspension failure custody.
use super::WorthQueryGeneratedOutputSuspensionFailure;
impl WorthQueryGeneratedOutputSuspensionFailure {
    /// The original request control stop, if this failure carries one.
    /// Preparation failures retain their complete Native denial and unpublished
    /// failures retain their recovery authority; this projection consumes neither.
    pub fn execution_control_stop(
        &self,
    ) -> Option<crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind> {
        match self {
            Self::ExecutionDenied(cause) => {
                crate::domain_computation::provider_session::protocol::advancement_denial_cause(
                    *cause,
                )
                .err()
            }
            Self::Preparation(cause) => {
                super::preparation_execution::project(cause).and_then(Result::err)
            }
            _ => None,
        }
    }
}
