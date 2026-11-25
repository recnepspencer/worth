use super::OracleHome;
use crate::host_interruption::opening::{into_control, AdversarialPlacement, PhysicalPauseHandle};
use std::sync::{atomic::AtomicUsize, Arc};
use worth_foundational::facade::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryApplicationContributionTuple,
    },
    declaration::{
        application_program::ApplicationProgramDefinition,
        application_schema::ApplicationSchemaComposition,
    },
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::application_home::{
    schema::HomeIdentitySchema, HomeProgram,
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

pub fn open_request<Schema, Program>(
    mut request: worth_query_host::facade::WorthQueryApplicationHomeRequest<'_, Schema, Program>,
    control: Option<ApplicationHomeOpeningControlForTest>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_host::facade::WorthQueryApplicationHome<Schema, Program>
where
    Schema: worth_query_host::facade::declaration::application_schema::ApplicationSchemaComposition,
    Program: worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition<
            Schema,
        > + 'static,
    Program::Outputs: worth_query_host::facade::declaration::application_program::ApplicationProgramOutputsShape<Schema>
        + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: worth_query_host::facade::application_contribution::WorthQueryApplicationContributionTuple<Schema>,
{
    if let Some(control) = control {
        request = request.with_opening_control_for_test(control);
    }
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .expect("the actual bootstrap phase is admitted")
        .unwrap_or_else(|refused| panic!("production Host opening refused: {refused:?}"))
}

/// Attach certification placement to any Host opening request and return its
/// physical mutation pause handle.
pub fn attach_placement<'a, Schema, Program, Transition>(
    request: WorthQueryApplicationHomeRequest<'a, Schema, Program, Transition>,
    placement: AdversarialPlacement,
) -> (
    WorthQueryApplicationHomeRequest<'a, Schema, Program, Transition>,
    PhysicalPauseHandle,
)
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let (control, pause) = into_control(placement);
    (request.with_opening_control_for_test(control), pause)
}

/// Parameters for opening the oracle Home with optional adversarial placement.
pub struct PlacementOpenConfig {
    pub location: WorthQueryApplicationHomeLocation,
    pub build: ApplicationBuildIdentity,
    pub initialized: Arc<AtomicUsize>,
    pub contacts: Arc<AtomicUsize>,
    pub placement: Option<AdversarialPlacement>,
    pub limits: ApplicationHomeCheckpointLimits,
}

/// Open the oracle Home with optional adversarial placement, returning any
/// pre-service opening refusal directly for typed refusal verification.
pub fn try_open_with_placement(
    config: PlacementOpenConfig,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    (OracleHome, Option<PhysicalPauseHandle>),
    WorthQueryApplicationHomeOpenRefused<HomeIdentitySchema, HomeProgram>,
> {
    let request = worth_query_testkit_host::facade::host::application_home::request_at_location(
        config.location,
        config.build,
        config.initialized,
        config.contacts,
    )
    .with_checkpoint_limits(config.limits);
    let (request, pause) = match config.placement {
        Some(placement) => {
            let (request, pause) = attach_placement(request, placement);
            (request, Some(pause))
        }
        None => (request, None),
    };
    let home = with_bootstrap_advancement_from_request(*execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .expect("the actual bootstrap phase is admitted")?;
    Ok((home, pause))
}

/// Open the oracle Home with optional adversarial placement, panicking on refusal.
pub fn open_with_placement(
    config: PlacementOpenConfig,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (OracleHome, Option<PhysicalPauseHandle>) {
    try_open_with_placement(config, execution)
        .unwrap_or_else(|refused| panic!("production Host opening refused: {refused:?}"))
}
