//! Actual owner release facts. None means that route owned no such resource.
use crate::domain_computation::{
    WorthQueryManagedProviderWorkEvidence, WorthQueryProviderCheckpointReleaseDisposition,
    WorthQueryProviderExecutionDestructorDisposition as Destructor,
    WorthQueryProviderExecutionDisposalDisposition as Disposal,
    WorthQueryProviderExecutionReleaseEvidence,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryManagedExecutionRelease {
    pub disposal: Disposal,
    pub destructor: Destructor,
}
impl QueryManagedExecutionRelease {
    pub(in crate::domain_computation) fn observe(
        release: &WorthQueryProviderExecutionReleaseEvidence,
    ) -> Self {
        Self {
            disposal: release.disposal(),
            destructor: release.destructor(),
        }
    }
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
    pub(in crate::domain_computation) fn observe(
        native_released: bool,
        work: &WorthQueryManagedProviderWorkEvidence,
    ) -> Self {
        let releases = work.provider_execution_release();
        let provider_execution = releases
            .recovery_evidence()
            .map(QueryManagedExecutionRelease::observe)
            .or_else(|| {
                (releases.release_count() != 0).then_some(QueryManagedExecutionRelease {
                    disposal: Disposal::Completed,
                    destructor: Destructor::Completed,
                })
            });
        Self {
            native_released: native_released && work.provider_retained_bytes() == 0,
            checkpoint: None,
            provider_execution,
            restored_execution: None,
        }
    }
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
