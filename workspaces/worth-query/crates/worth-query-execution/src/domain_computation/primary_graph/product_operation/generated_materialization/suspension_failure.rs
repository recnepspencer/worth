//! Borrow typed request controls without consuming suspension failure custody.
use super::WorthQueryGeneratedOutputSuspensionFailure;
impl WorthQueryGeneratedOutputSuspensionFailure {
    /// The original request control stop, if this failure carries one.
    /// Preparation failures retain their complete Native denial and unpublished
    /// failures retain their recovery authority; this projection consumes neither.
    pub fn execution_control_stop(
        &self,
    ) -> Option<crate::domain_computation::WorthQueryProviderSessionControlStopKind> {
        match self {
            Self::ExecutionDenied(cause) => cause.provider_denial_cause().err(),
            Self::Preparation(cause) => {
                super::preparation_execution::project(cause).and_then(Result::err)
            }
            _ => None,
        }
    }
}
