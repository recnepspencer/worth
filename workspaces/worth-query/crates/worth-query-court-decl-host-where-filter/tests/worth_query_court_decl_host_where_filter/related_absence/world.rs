//! Installed Host world and funded/pinned/mutation helpers for related-absence courts.
use super::{mutations::*, queries::*, schema::*};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestMutationDenial, WorthQueryApplicationRequestQueryDenial,
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
pub struct BoardFeature;
impl ApplicationFeature<Schema> for BoardFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "related-absence.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("related-absence.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, BoardFeature>()
            .native_action::<MutateBinding>()
            .finish_action()
            .native_action::<ObserveBinding>()
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
        setup.handler::<MutateBinding, _>(MutateHandler)?;
        setup.handler::<ObserveBinding, _>(ObserveHandler)
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

#[derive(Clone, Debug)]
pub struct TaskSeed {
    pub key: u64,
    pub project: u64,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub rank: i64,
    pub due_days: Option<i64>,
}

pub fn install(tasks: &[TaskSeed], blocked_by: &[(u64, u64)]) -> Host {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("68686868-6868-6868-6868-686868686868").unwrap(),
        WorthQueryApplicationLimits::new(
            world_resources(
                128,
                64 * 1024 * 1024,
                256,
                worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
            ),
            WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(64).unwrap()),
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
                    WorthQueryApplicationEntitySeed::new(
                        Project::reference(),
                        entity_key!("project", project),
                    )
                    .field(ProjectKey::at("ProjectKey"), project),
                )?;
            }
            for task in tasks {
                let mut seed = WorthQueryApplicationEntitySeed::new(
                    Task::reference(),
                    entity_key!("task", task.key),
                )
                .field(TaskKey::at("TaskKey"), task.key)
                .field(Title::at("Title"), task.title.clone())
                .field(Status::at("Status"), task.status.clone())
                .field(Rank::at("Rank"), task.rank);
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
                    entity_key!("project", task.project),
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
        },
    )
    .expect("related-absence board installs")
}

pub fn run_mutate(host: &Host, input: MutateBoard, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
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

/// Checks exact source currentness against the Host: returns `Ok(true)` ONLY
/// on `Committed`, `Ok(false)` ONLY on typed `SourceChanged`, and `Err` on any
/// other outcome or denial so unrelated errors can never masquerade as stale.
pub fn observe_ready_source(
    host: &Host,
    project: u64,
    source: &WorthQueryObservedSource<ReadyTasksQuery>,
    key: &str,
) -> Result<bool, String> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    match host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(ObserveReady { project })
        .expect_source(source.clone())
        .idempotency(&key.to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
    {
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. }) => Ok(true),
        Ok(other) => Err(format!(
            "expected Committed currentness check, got {other:?}"
        )),
        Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Attempt(cause),
        )) if cause.kind() == WorthQueryApplicationAttemptDenialKind::SourceChanged => Ok(false),
        Err(other) => Err(format!("expected SourceChanged denial, got {other:?}")),
    }
}

pub fn ready_with_source(
    host: &Host,
    project: u64,
) -> (Vec<u64>, WorthQueryObservedSource<ReadyTasksQuery>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .query(ReadyIntent(ProjectScope { project }))
        .execute()
        .expect("ready_tasks succeeds");
    let keys = result.rows().iter().map(|r| r.key).collect();
    (
        keys,
        result
            .result_set_observation()
            .expect("result set source present")
            .source()
            .clone(),
    )
}

pub fn funded_ready(
    host: &Host,
    scope: &WorthQueryRequestScope,
    memory_bytes: u64,
    work: u64,
    project: u64,
) -> Result<(Vec<u64>, usize), Box<WorthQueryApplicationRequestQueryDenial>> {
    let caller = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory_bytes), Some(work)),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("finite lease admitted");
    let principal = authenticate_operator(host.installed_schema(), scope);
    host.runtime()
        .with_funded_application_advancement(
            scope,
            ExecutionAllocationPolicy::Execution(&caller),
            |phase| {
                host.runtime()
                    .request(&principal, scope)
                    .query(ReadyIntent(ProjectScope { project }))
                    .execute_in_advancement(&phase)
                    .map(|result| {
                        let keys = result.rows().iter().map(|r| r.key).collect();
                        (
                            keys,
                            result.receipt().inspect().read_work().total_work_units(),
                        )
                    })
                    .map_err(Box::new)
            },
        )
        .expect("funded advancement opens")
}

/// Runs unfiltered `UnlinkedTasksQuery` (`where_no_related(BlockedBy)` with zero
/// target field reads) so all fanout work and memory isolates `BlockedBy`
/// discovery, native collector, sort, and endpoint conversion.
pub fn funded_unlinked(
    host: &Host,
    scope: &WorthQueryRequestScope,
    memory_bytes: u64,
    work: u64,
    project: u64,
) -> Result<(Vec<u64>, usize), Box<WorthQueryApplicationRequestQueryDenial>> {
    let caller = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(memory_bytes), Some(work)),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .expect("finite lease admitted");
    let principal = authenticate_operator(host.installed_schema(), scope);
    host.runtime()
        .with_funded_application_advancement(
            scope,
            ExecutionAllocationPolicy::Execution(&caller),
            |phase| {
                host.runtime()
                    .request(&principal, scope)
                    .query(UnlinkedIntent(ProjectScope { project }))
                    .execute_in_advancement(&phase)
                    .map(|result| {
                        let keys = result.rows().iter().map(|r| r.key).collect();
                        (
                            keys,
                            result.receipt().inspect().read_work().total_work_units(),
                        )
                    })
                    .map_err(Box::new)
            },
        )
        .expect("funded advancement opens")
}
