//! A declared test policy owns bare provider fixtures that have no product World.
use super::*;

pub(crate) fn with_standalone_provider_test_advancement<R>(
    placement: worth_runtime_world::facade::RuntimeWorldExecutionPlacement<'_>,
    body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
) -> Result<R, WorthQueryAdvancementDenial> {
    // These internal provider fixtures have no installed product World.
    let _custody = OpeningCustody::enter()?;
    QueryRequestExecution::open_control(placement, worth_execution::CancellationToken::new(), None)
        .run_advancement(None, None, body)
}

impl crate::domain_computation::primary_graph::tests::fixture::AdvancementHost {
    pub(crate) fn run_provider_fixture<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        with_standalone_provider_test_advancement(self.placement(), body)
    }
}
