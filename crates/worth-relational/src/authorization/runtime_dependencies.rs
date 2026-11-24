//! Bind durable dependency capture to the evidence's exact snapshot.
use super::durable_dependencies::currentness::RelationalAuthorizationDependencySnapshots;
use super::{
    durable_dependencies::capture::capture, RelationalAuthorizationDependencyDenial as Denial,
    RelationalAuthorizationDurableDependencies, RelationalAuthorizationObservationEvidence,
};
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Capture exact source revisions from the evidence's own immutable root.
    /// Cost is O(tracked reads); no graph reconstruction is permitted here.
    pub fn capture_authorization_durable_dependencies(
        &self,
        evidence: &RelationalAuthorizationObservationEvidence,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthorizationDurableDependencies, Denial> {
        let view =
            <RelationalAuthorizationDependencySnapshots as From<&RelationalRuntime>>::from(self)
                .project(evidence.snapshot())?;
        capture(&view, execution, evidence)
    }
}

impl<'a> From<&'a RelationalRuntime> for RelationalAuthorizationDependencySnapshots<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            projection: runtime.into(),
            directory: runtime.into(),
        }
    }
}
impl<'a> From<&'a mut RelationalRuntime> for RelationalAuthorizationDependencySnapshots<'a> {
    fn from(runtime: &'a mut RelationalRuntime) -> Self {
        (&*runtime).into()
    }
}
impl<'a> From<&'a Box<RelationalRuntime>> for RelationalAuthorizationDependencySnapshots<'a> {
    fn from(runtime: &'a Box<RelationalRuntime>) -> Self {
        runtime.as_ref().into()
    }
}
