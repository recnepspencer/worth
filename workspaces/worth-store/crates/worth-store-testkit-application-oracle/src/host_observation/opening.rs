use super::OracleHome;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_foundational::facade::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    WorthQueryApplicationHomeLocation,
};
use worth_store_application::{
    ApplicationHomeCheckpointLimits, ApplicationHomeOpeningControlForTest,
};

pub fn open(
    location: WorthQueryApplicationHomeLocation,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
    control: Option<ApplicationHomeOpeningControlForTest>,
    limits: ApplicationHomeCheckpointLimits,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> OracleHome {
    let mut request =
        worth_query_testkit_host::facade::host::application_home::request_at_location(
            location,
            build,
            initialized,
            contacts,
        )
        .with_checkpoint_limits(limits);
    if let Some(control) = control {
        request = request.with_opening_control_for_test(control);
    }
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .expect("the actual bootstrap phase is admitted")
        .unwrap_or_else(|refused| panic!("production Host opening refused: {refused:?}"))
}
