//! Directory ownership for the authored document workflows.
use super::document_retention_model::{host, programs, schema::DocumentRetentionSchema, workflow};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{ApplicationBuildIdentity, WorthQueryApplicationProgramRoster},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub type Home = WorthQueryApplicationHome<DocumentRetentionSchema, programs::RetentionProgramP0>;

pub fn location(journey: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "worth-authored-{journey}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

pub fn open(directory: &std::path::Path) -> (Home, host::DocumentWorkflowRuntime) {
    let program = programs::validated_first_program();
    let roster = host::with_lifecycle_programs(
        WorthQueryApplicationProgramRoster::new().support(programs::validated_second_program()),
        program.identity(),
        program.revision(),
    );
    let lease = funding();
    let home = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.to_owned()), program, roster,
            DocumentRetentionSchema::declaration().unwrap(), ((),),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
            host::seed_host).open_in_advancement(&phase))
        .expect("bootstrap admission").unwrap_or_else(|cause| panic!("authored Home: {cause:?}"));
    let application = workflow::retain_workflow(home.runtime().clone());
    (home, application)
}

pub fn close(home: Home) {
    let lease = funding();
    home.close(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap_or_else(|cause| panic!("authored Home close: {cause:?}"))
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
