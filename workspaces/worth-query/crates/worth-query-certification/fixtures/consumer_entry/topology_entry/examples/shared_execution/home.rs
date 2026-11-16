//! Actual Directory Homes use the one process execution service.
use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
pub type Home = WorthQueryApplicationHome<ExampleSchema, ExampleProgram<false>>;

pub fn open(
    directory: &std::path::Path,
    build: installation::ApplicationBuildIdentity,
    limits: installation::WorthQueryApplicationLimits,
) -> Home {
    let declaration = ExampleSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<ExampleSchema, ExampleProgram<false>>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let lease = funding();
    with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(directory.to_owned()),
                program,
                installation::WorthQueryApplicationProgramRoster::new(),
                declaration,
                (setup::configuration(),),
                build,
                limits,
                |_, graph, installed| setup::seed_graph(graph, installed, 0),
            )
            .open_in_advancement(&phase)
        },
    )
    .expect("bootstrap admission")
    .unwrap_or_else(|cause| panic!("shared-service Home: {cause:?}"))
}

pub fn close(home: Home) {
    let lease = funding();
    home.close(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap_or_else(|cause| panic!("shared-service Home close: {cause:?}"))
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
