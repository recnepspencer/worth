//! The installed board program, its seed, and funded public reads.
use super::{edit, observe, queries::*, schema::*};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial,
};
use worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};
use worth_query_testkit_document_retention::facade::{
    document_retention_model::operator_identity::{
        authenticate_operator, operator_identity, request_scope,
    },
    document_world_resources::world_resources,
};

macro_rules! entity_key {
    ($kind:literal, $key:expr) => {
        WorthQueryApplicationEntityKey::new(format!("{}-{}", $kind, $key)).unwrap()
    };
}

pub struct Program;
pub struct Board;
impl ApplicationFeature<Schema> for Board {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "where-filter.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("where-filter.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Board>()
            .native_action::<edit::Binding>()
            .finish_action()
            .native_action::<observe::Binding>()
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
        setup.handler::<edit::Binding, _>(edit::Handler)?;
        setup.handler::<observe::Binding, _>(observe::Handler)
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

/// One seeded task: `(key, project, title, notes, status, rank, length_micros, due_days)`.
pub type Seed = (
    u64,
    u64,
    &'static str,
    Option<String>,
    &'static str,
    i64,
    u64,
    Option<i64>,
);

pub fn board() -> Vec<Seed> {
    let notes = |text: &str| Some(text.to_owned());
    vec![
        (
            1,
            1,
            "Mill housing",
            notes("check FLANGE tolerance"),
            "open",
            5,
            12_500_000,
            Some(100),
        ),
        (
            2,
            1,
            "Drill flange holes",
            None,
            "open",
            9,
            40_000_000,
            None,
        ),
        (
            3,
            1,
            "Paint",
            notes("matte black"),
            "done",
            7,
            3_000_000,
            Some(50),
        ),
        (
            4,
            1,
            "Inspect",
            notes("awaiting parts"),
            "open",
            2,
            80_000_000,
            Some(200),
        ),
        (
            5,
            1,
            "Weld bracket",
            notes("flange-adjacent"),
            "open",
            9,
            25_000_000,
            Some(150),
        ),
        (
            6,
            2,
            "Flange spare",
            notes("flange"),
            "open",
            9,
            1_000_000,
            Some(10),
        ),
    ]
}

pub fn install(tasks: &[Seed]) -> Host {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("57575757-5757-5757-5757-575757575757").unwrap(),
        WorthQueryApplicationLimits::new(
            world_resources(
                128,
                64 * 1024 * 1024,
                256,
                worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
            ),
            WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(32).unwrap()),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(PrincipalBinding::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("board-author").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for project in [1_u64, 2] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Project::reference(), entity_key!("project", project))
                        .field(ProjectKey::at("ProjectKey"), project),
                )?;
            }
            for (key, project, title, notes, status, rank, micros, due) in tasks {
                let mut seed = WorthQueryApplicationEntitySeed::new(Task::reference(), entity_key!("task", *key))
                    .field(TaskKey::at("TaskKey"), *key)
                    .field(Title::at("Title"), (*title).to_owned())
                    .field(Status::at("Status"), (*status).to_owned())
                    .field(Rank::at("Rank"), *rank)
                    .field(Length::at("Length"), length(*micros));
                if let Some(notes) = notes {
                    seed = seed.field(Notes::at("Notes"), notes.clone());
                }
                if let Some(days) = due {
                    seed = seed.field(Due::at("Due"), date(*days));
                }
                graph.bind_entity(seed)?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Contains::reference(),
                    format!("contains-{key}"),
                    entity_key!("project", *project),
                    entity_key!("task", *key),
                ))?;
            }
            Ok(())
        },
    )
    .expect("the board application installs")
}

pub type Rows = Result<Vec<TaskRow>, WorthQueryApplicationRequestQueryDenial>;

macro_rules! reader {
    ($name:ident, $intent:ty) => {
        /// An ordinary current read through the public request front door,
        /// optionally narrowing the installed result ceiling.
        pub fn $name(host: &Host, intent: $intent, limit: Option<std::num::NonZeroUsize>) -> Rows {
            let scope = request_scope();
            let principal = authenticate_operator(host.installed_schema(), &scope);
            let request = host.runtime().request(&principal, &scope);
            let mut query = request.query(intent);
            if let Some(limit) = limit {
                query = query.limit_results(limit);
            }
            query.execute().map(|result| result.rows().to_vec())
        }
    };
}
reader!(search, Search);
reader!(due_by, DueBy);
reader!(open_inbox, OpenInbox);
reader!(literal_filter, LiteralFilter);
reader!(fit, Fit);
reader!(by_status, ByStatus);

pub fn exact_project(
    host: &Host,
    project: u64,
) -> Result<Vec<ProjectRow>, WorthQueryApplicationRequestQueryDenial> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .query(ExactProject { project })
        .execute()
        .map(|result| result.rows().to_vec())
}

pub fn exact_open_task(host: &Host, task: u64, expected_rank: i64) -> Rows {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .query(ExactOpenTask {
            task,
            expected_rank,
        })
        .execute()
        .map(|result| result.rows().to_vec())
}

pub fn funded_due(
    host: &Host,
    scope: &WorthQueryRequestScope,
    work: u64,
    intent: DueBy,
) -> Result<(Vec<u64>, usize), WorthQueryApplicationRequestQueryDenial> {
    funded_due_with_budget(host, scope, 64 * 1024 * 1024, work, intent)
}

pub fn funded_due_with_budget(
    host: &Host,
    scope: &WorthQueryRequestScope,
    memory_bytes: u64,
    work: u64,
    intent: DueBy,
) -> Result<(Vec<u64>, usize), WorthQueryApplicationRequestQueryDenial> {
    let caller = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(memory_bytes),
                Some(work),
            ),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("the finite caller request is admitted");
    let principal = authenticate_operator(host.installed_schema(), scope);
    host.runtime()
        .with_funded_application_advancement(
            scope,
            ExecutionAllocationPolicy::Execution(&caller),
            |phase| {
                host.runtime()
                    .request(&principal, scope)
                    .query(intent)
                    .execute_in_advancement(&phase)
                    .map(|result| {
                        let keys = result.rows().iter().map(|row| row.key).collect();
                        (
                            keys,
                            result.receipt().inspect().read_work().total_work_units(),
                        )
                    })
            },
        )
        .expect("the installed host opens funded advancement")
}

pub fn keys(rows: Rows) -> Vec<u64> {
    rows.expect("the filtered read succeeds")
        .iter()
        .map(|row| row.key)
        .collect()
}

/// A search funded by a caller lease of exactly `work` units; on success,
/// the rows' keys and the receipt's total read work.
pub fn funded_search(
    host: &Host,
    scope: &WorthQueryRequestScope,
    work: u64,
    search: Search,
) -> Result<(Vec<u64>, usize), WorthQueryApplicationRequestQueryDenial> {
    let caller = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 * 1024 * 1024),
                Some(work),
            ),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("the finite caller request is admitted");
    let principal = authenticate_operator(host.installed_schema(), scope);
    host.runtime()
        .with_funded_application_advancement(
            scope,
            ExecutionAllocationPolicy::Execution(&caller),
            |phase| {
                host.runtime()
                    .request(&principal, scope)
                    .query(search)
                    .execute_in_advancement(&phase)
                    .map(|result| {
                        let keys = result.rows().iter().map(|row| row.key).collect();
                        (
                            keys,
                            result.receipt().inspect().read_work().total_work_units(),
                        )
                    })
            },
        )
        .expect("the installed host opens funded advancement")
}
