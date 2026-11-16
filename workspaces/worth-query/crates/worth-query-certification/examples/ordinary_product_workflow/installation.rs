//! The ordinary temporal operation owns a genuine Directory Home.
use super::product_workflow_support::{
    adapters::ClockSource,
    application::{example_limits, seed_graph},
    conditional_contribution::{TemporalConditional, TemporalContributionConfiguration},
    program::{self, TemporalExampleProgram},
    schema::{TemporalHostSchema, TemporalPrincipalBinding},
    ExampleApplication,
};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{ApplicationBuildIdentity, WorthQueryApplicationProgramRoster},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
pub type Home = WorthQueryApplicationHome<TemporalHostSchema, TemporalExampleProgram>;

pub fn open(directory: &std::path::Path) -> (Home, ExampleApplication) {
    let (clock_source, clock_control) = ClockSource::due();
    let lease = funding();
    let home = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(directory.to_owned()),
                program::validated_program(),
                WorthQueryApplicationProgramRoster::new(),
                TemporalHostSchema::declaration().unwrap(),
                (TemporalContributionConfiguration::new(clock_source),),
                ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
                example_limits(None),
                |_, graph, installed| {
                    let principal = installed
                        .principal_binding(TemporalPrincipalBinding::reference())
                        .unwrap();
                    seed_graph(graph, &principal, "blocked");
                    Ok(())
                },
            )
            .open_in_advancement(&phase)
        },
    )
    .expect("bootstrap admission")
    .unwrap_or_else(|cause| panic!("ordinary Home: {cause:?}"));
    let runtime = home.runtime().clone();
    let conditional = runtime.conditional::<TemporalConditional>().unwrap();
    (
        home,
        ExampleApplication {
            runtime,
            conditional,
            clock_control,
        },
    )
}

pub fn close(home: Home) {
    let lease = funding();
    home.close(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap_or_else(|cause| panic!("ordinary Home close: {cause:?}"))
        .reclaim();
}

fn funding() -> worth_execution::ExecutionResourceLease<'static> {
    use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 << 20), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .expect("bounded Home lifecycle request")
}
