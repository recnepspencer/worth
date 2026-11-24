//! In-memory and Directory Home worlds for whole-result live subscription courts
//! and the 64-seed Differential Live-Subscription DST.
use super::mutations::*;
use super::queries::date;
use super::schema::*;
use std::path::PathBuf;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, authentication::*};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::{
    document_retention_model::operator_identity::{
        authenticate_reviewer, operator_identity, request_scope, reviewer_identity,
    },
    document_world_resources::world_resources,
};

pub const BUILD_ID: &str = "7a7a7a7a-7a7a-7a7a-7a7a-7a7a7a7a7a7a";
pub const TODAY_CUTOFF_DAYS: i64 = 100;
pub const FUTURE_DUE_DAYS: i64 = 200;

macro_rules! entity_key {
    ($kind:literal, $key:expr) => {
        WorthQueryApplicationEntityKey::new(format!("{}-{}", $kind, $key)).unwrap()
    };
}

pub struct Program;
pub struct BoardFeature;
impl ApplicationFeature<Schema> for BoardFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "live-whole-result.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("live-whole-result.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, BoardFeature>()
            .native_action::<MutateBinding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<MutateBinding, _>(MutateHandler)
    }
}

pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
pub type Home = WorthQueryApplicationHome<Schema, Program>;

#[derive(Clone, Debug)]
pub struct TaskSeed {
    pub key: u64,
    pub project: String,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub due_days: Option<i64>,
}

pub fn live_limits() -> WorthQueryApplicationLiveLimits {
    WorthQueryApplicationLiveLimits::bounded(64, 64)
}

fn app_limits() -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::new(
        world_resources(
            128,
            64 * 1024 * 1024,
            256,
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(64).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}

fn seed_board(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Schema>,
    tasks: &[TaskSeed],
    blocked_by: &[(u64, u64)],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let pb = installed
        .principal_binding(PrincipalBinding::reference())
        .unwrap();
    // Principal 1: subscriber (subject to grant revocation & principal disablement)
    graph.bind_principal(
        &pb,
        WorthQueryApplicationPrincipalKey::new("subscriber-1").unwrap(),
        1_u64,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    // Principal 2: admin writer (can commit mutations even when subscriber-1 is revoked/disabled)
    graph.bind_principal(
        &pb,
        WorthQueryApplicationPrincipalKey::new("admin-2").unwrap(),
        2_u64,
        reviewer_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    for proj in ["p1", "p2"] {
        let pstr = format!("project-{proj}");
        let gstr = format!("grant-{proj}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Project::reference(), entity_key!("p", proj))
                .field(ProjectKey::at("ProjectKey"), proj.to_owned()),
        )?;
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(BoardGrant::reference(), entity_key!("g", proj))
                .field(GrantIdentity::at("GrantIdentity"), gstr.clone())
                .field(GrantAction::at("GrantAction"), 1_u64)
                .field(GrantPurpose::at("GrantPurpose"), 1_u64)
                .field(GrantStatus::at("GrantStatus"), 1_u64)
                .field(GrantScope::at("GrantScope"), proj.to_owned())
                .field(GrantNotBefore::at("GrantNotBefore"), 0_u64)
                .field(GrantNotAfter::at("GrantNotAfter"), u64::MAX / 2)
                .field(GrantDepth::at("GrantDepth"), 0_u64)
                .field(GrantDisclosure::at("GrantDisclosure"), 1_u64),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantee::reference(),
            format!("grantee-{proj}"),
            WorthQueryApplicationEntityKey::new("subscriber-1".to_owned()).unwrap(),
            entity_key!("g", proj),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantGrantor::reference(),
            format!("grantor-{proj}"),
            WorthQueryApplicationEntityKey::new("admin-2".to_owned()).unwrap(),
            entity_key!("g", proj),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            GrantResource::reference(),
            format!("resource-{proj}"),
            entity_key!("g", proj),
            entity_key!("p", proj),
        ))?;
        let _ = pstr;
    }
    for task in tasks {
        let mut seed =
            WorthQueryApplicationEntitySeed::new(Task::reference(), entity_key!("task", task.key))
                .field(TaskKey::at("TaskKey"), task.key)
                .field(Title::at("Title"), task.title.clone())
                .field(Status::at("Status"), task.status.clone());
        if let Some(notes) = &task.notes {
            seed = seed.field(Notes::at("Notes"), notes.clone());
        }
        if let Some(days) = task.due_days {
            seed = seed.field(Due::at("Due"), date(days));
        }
        graph.bind_entity(seed)?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Contains::reference(),
            format!("contains-{}", task.key),
            entity_key!("p", &task.project),
            entity_key!("task", task.key),
        ))?;
    }
    for (idx, &(from, to)) in blocked_by.iter().enumerate() {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            BlockedBy::reference(),
            format!("blocked-{idx}-{from}-{to}"),
            entity_key!("task", from),
            entity_key!("task", to),
        ))?;
    }
    Ok(())
}

pub fn install(tasks: &[TaskSeed], blocked_by: &[(u64, u64)]) -> Host {
    let tasks = tasks.to_vec();
    let blocked_by = blocked_by.to_vec();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(BUILD_ID).unwrap(),
        app_limits(),
        move |_, graph, installed| seed_board(graph, installed, &tasks, &blocked_by),
    )
    .expect("live-whole-result board installs")
}

fn home_lease<'a>(
    scope: &'a worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> ExecutionResourceLease<'a> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("home lease admits")
}

pub fn open_directory_home(dir: PathBuf, tasks: &[TaskSeed], blocked_by: &[(u64, u64)]) -> Home {
    let tasks = tasks.to_vec();
    let blocked_by = blocked_by.to_vec();
    let declaration = Schema::declaration().unwrap();
    let validated = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let req = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(dir),
        validated,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(BUILD_ID).unwrap(),
        app_limits(),
        move |_, graph, installed| seed_board(graph, installed, &tasks, &blocked_by),
    );
    let scope = request_scope();
    let lease = home_lease(&scope);
    let execution = ExecutionRequest::leased(&lease);
    with_bootstrap_advancement_from_request(execution, |phase| req.open_in_advancement(&phase))
        .expect("bootstrap advancement opens")
        .expect("directory home opens")
}

pub fn close_directory_home(home: Home) {
    let scope = request_scope();
    let lease = home_lease(&scope);
    let execution = ExecutionRequest::leased(&lease);
    let _closed = home
        .close(&execution)
        .expect("directory home closes cleanly");
}

pub fn run_mutate(host: &Host, input: MutateLiveBoard, idem: u64) {
    let scope = request_scope();
    let admin = authenticate_reviewer(host.installed_schema(), &scope);
    run_mutate_as(host, &admin, &scope, input, idem);
}

pub fn run_mutate_as(
    host: &Host,
    admin: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    input: MutateLiveBoard,
    idem: u64,
) {
    let outcome = host
        .runtime()
        .request(admin, scope)
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .expect("board mutation executes");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::<_, Mutated>::Committed { .. }
    ));
}
