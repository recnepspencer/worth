//! Real Home support statuses survive the same ordinary close/reopen route.
use super::*;
mod separate_process;
use fixture::schema::HomeIdentitySchema;
use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
use worth_query_host::facade::{
    application_installation::{
        WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
        WorthQueryProgramSupportRetirementFailure,
    },
    domain::WorthQueryProgramSupportRetirementDenial,
    primary_graph::SignalConditionalEvaluationBudget,
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
struct UnusedProgram;
impl ApplicationProgramDefinition<HomeIdentitySchema> for UnusedProgram {
    type Contributions =
        <fixture::HomeProgram as ApplicationProgramDefinition<HomeIdentitySchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home-unused-support.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        fixture::HomeProgram::feature_specs()
    }
}
fn request(
    location: std::path::PathBuf,
    initialized: Arc<AtomicUsize>,
) -> WorthQueryApplicationHomeRequest<'static, HomeIdentitySchema, fixture::HomeProgram> {
    let declaration = HomeIdentitySchema::declaration().unwrap();
    let extra = ApplicationProgramAuthoring::<HomeIdentitySchema, UnusedProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<HomeIdentitySchema, fixture::HomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new().support(extra),
        declaration,
        (Arc::new(AtomicUsize::new(0)),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        move |phase, graph, schema| {
            initialized.fetch_add(1, Ordering::SeqCst);
            fixture::schema::seed_batches(phase, graph, schema, true)
        },
    )
}
#[test]
fn directory_program_support_retirement_stays_retired_after_graceful_reopen() {
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(location.clone(), initialized.clone()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("{refused:?}"));
    let revision = *home
        .runtime()
        .supported_program::<UnusedProgram>()
        .expect("extra support is genuinely installed")
        .installed_program()
        .revision();
    let receipt = home
        .runtime()
        .retire_program_support(&execution, &revision)
        .expect("unused installed support retires");
    assert!(receipt.inventory().can_retire());
    assert!(home
        .runtime()
        .supported_program::<UnusedProgram>()
        .is_none());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("retired close: {refused:?}"))
        .reclaim();
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(location, initialized.clone()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("retired reopen: {refused:?}"));
    assert!(home
        .runtime()
        .supported_program::<UnusedProgram>()
        .is_none());
    assert!(
        matches!(home.runtime().retire_program_support(&execution, &revision), Err(WorthQueryProgramSupportRetirementFailure::Policy(WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision: refused })) if refused == revision)
    );
    assert!(
        home.runtime().presented_program().is_some(),
        "the active main program is preserved independently"
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("second close: {refused:?}"))
        .reclaim();
}
#[test]
fn refused_directory_support_retirement_records_active_abort_before_reopen() {
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(location.clone(), initialized.clone()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("{refused:?}"));
    let revision = *home.runtime().installed_program().revision();
    let Err(WorthQueryProgramSupportRetirementFailure::Policy(
        WorthQueryProgramSupportRetirementDenial::CurrentBranches(inventory),
    )) = home.runtime().retire_program_support(&execution, &revision)
    else {
        panic!("main is an actual retirement blocker");
    };
    assert_eq!(inventory.current_branches(), 1);
    assert!(home.runtime().presented_program().is_some());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("abort close: {refused:?}"))
        .reclaim();
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(location, initialized.clone()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("active abort reopen: {refused:?}"));
    assert!(home.runtime().presented_program().is_some());
    assert!(home
        .runtime()
        .supported_program::<UnusedProgram>()
        .is_some());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("second close: {refused:?}"))
        .reclaim();
}
