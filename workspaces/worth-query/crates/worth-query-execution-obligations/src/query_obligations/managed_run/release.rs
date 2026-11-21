//! Actual owner release facts. None means that route owned no such resource.
use worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::{
    WorthQueryProviderCheckpointReleaseDisposition,
    WorthQueryProviderExecutionDestructorDisposition as Destructor,
    WorthQueryProviderExecutionDisposalDisposition as Disposal,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryManagedExecutionRelease {
    pub disposal: Disposal,
    pub destructor: Destructor,
}
impl QueryManagedExecutionRelease {
    pub const fn released(self) -> bool {
        matches!(self.disposal, Disposal::Completed)
            && matches!(self.destructor, Destructor::Completed)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryManagedRunResources {
    pub native_released: bool,
    pub checkpoint: Option<WorthQueryProviderCheckpointReleaseDisposition>,
    pub provider_execution: Option<QueryManagedExecutionRelease>,
    pub restored_execution: Option<QueryManagedExecutionRelease>,
}
impl QueryManagedRunResources {
    pub const fn released(self) -> bool {
        self.native_released
            && !matches!(
                self.checkpoint,
                Some(WorthQueryProviderCheckpointReleaseDisposition::Panicked)
            )
            && match self.provider_execution {
                Some(release) => release.released(),
                None => true,
            }
            && match self.restored_execution {
                Some(release) => release.released(),
                None => true,
            }
    }
}
