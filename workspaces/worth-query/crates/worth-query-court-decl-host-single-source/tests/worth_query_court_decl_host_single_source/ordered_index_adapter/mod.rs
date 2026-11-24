//! Public Decl+Host courts & 64-seed DST for `WorthQueryOrderedIndexAdapter`.
mod courts;
mod courts_edges;
mod dst;
mod focus_interval;
mod harness;
mod operations;
mod queries;
mod schema;
mod title_prefix;

use focus_interval::FocusIntervalDomain;
use harness::*;
use operations::*;
use queries::*;
use schema::*;
use std::num::NonZeroUsize;
use title_prefix::TitlePrefixDomain;
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_query::ApplicationManagedIndexDefinition;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::ordered_index::WorthQueryOrderedIndexAdapter;
use worth_query_host::facade::runtime::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    WorthQueryApplicationQueryResourceProfile, WorthQueryInvalidationResources,
    WorthQueryProductWorldClock, WorthQueryProductWorldResources,
};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
    WorthQueryApplicationHome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::operator_identity;

pub struct DaymarkFeature;
impl ApplicationFeature<DaymarkSchema> for DaymarkFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "ordered-index-adapter.feature.v1";
}

pub struct Contracts;
impl Registration<DaymarkSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<DaymarkSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DaymarkSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(PrincipalBinding::reference())
            .application_query(source_query())
            .application_query_binding::<SourceBinding>()
            .application_query(find_title_prefix_query())
            .application_query_binding::<FindTitlePrefixBinding>()
            .application_query(find_focus_overlap_query())
            .application_query_binding::<FindFocusOverlapBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<
                DaymarkSchema,
                TitlePrefixIndex,
            >::new())
            .managed_index(ApplicationManagedIndexDefinition::<
                DaymarkSchema,
                FocusIntervalIndex,
            >::new())
            .declared_operation::<CreateTaskOp>(
                CreateTaskOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<CreateTaskBinding>()
            .operation(rename_task_definition())
            .application_mutation_binding::<RenameTaskBinding>()
            .operation(reschedule_task_definition())
            .application_mutation_binding::<RescheduleTaskBinding>()
            .declared_operation::<DeleteTaskOp>(
                DeleteTaskOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<DeleteTaskBinding>())
    }
}

pub struct Contribution;
impl ApplicationSchemaContribution<DaymarkSchema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("ordered-index-adapter.contribution.v1");
    type Members = (Daymark, Contracts);
}
impl WorthQueryApplicationContribution<DaymarkSchema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, DaymarkSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup
            .managed_index_provider::<TitlePrefixIndex, _>(WorthQueryOrderedIndexAdapter::new(
                TitlePrefixDomain,
            ))
            .with_predicate_routing()
            .install()?;
        setup
            .managed_index_provider::<FocusIntervalIndex, _>(WorthQueryOrderedIndexAdapter::new(
                FocusIntervalDomain,
            ))
            .with_predicate_routing()
            .install()?;
        setup.handler::<CreateTaskBinding, _>(CreateTaskHandler)?;
        setup.operation_body::<RenameTaskBinding>()?;
        setup.operation_body::<RescheduleTaskBinding>()?;
        setup.handler::<DeleteTaskBinding, _>(DeleteTaskHandler)
    }
}

pub struct Program;
impl ApplicationProgramDefinition<DaymarkSchema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ordered-index-adapter.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DaymarkSchema, DaymarkFeature>()
                .native_action::<CreateTaskBinding>()
                .finish_action()
                .native_action::<RenameTaskBinding>()
                .finish_action()
                .native_action::<RescheduleTaskBinding>()
                .finish_action()
                .native_action::<DeleteTaskBinding>()
                .finish_action()
                .finish(),
        ]
    }
}

pub(super) type Host = WorthQueryProgramApplicationRuntime<DaymarkSchema, Program>;
pub(super) type Home = WorthQueryApplicationHome<DaymarkSchema, Program>;

pub(super) fn limits(
    memory_bytes: Option<u64>,
    work_ceiling: Option<u64>,
) -> WorthQueryApplicationLimits {
    let clock = WorthQueryAdmissionClock::system();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, memory_bytes, work_ceiling),
    );
    let world = WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 32,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 128,
                history_metadata_bytes: 64 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 128,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 32,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 32,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 256,
                in_flight_pin_acquisition_reservations: 64,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 64,
            },
        },
        WorthQueryProductWorldClock::start(clock),
        WorthQueryInvalidationResources::new(),
        policy,
    )
    .unwrap();
    WorthQueryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(8).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}

pub(super) fn seed_workspace(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DaymarkSchema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        DaymarkSchema,
    >,
    initial_tasks: &[(u64, &str, i64, i64)],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed
            .principal_binding(PrincipalBinding::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
        1_u64,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Workspace::reference(),
            WorthQueryApplicationEntityKey::new("main").unwrap(),
        )
        .field(WorkspaceKeyField::at("WorkspaceKey"), "main".to_owned()),
    )?;
    for &(key, title, start_min, end_min) in initial_tasks {
        let id = format!("task-{key}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                TaskItem::reference(),
                WorthQueryApplicationEntityKey::new(id.clone()).unwrap(),
            )
            .field(TaskKeyField::at("TaskKey"), key)
            .field(TaskTitleField::at("TaskTitle"), title.to_owned())
            .field(StartMinField::at("StartMin"), start_min)
            .field(EndMinField::at("EndMin"), end_min),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ContainsTask::reference(),
            format!("contains-{key}"),
            WorthQueryApplicationEntityKey::new("main").unwrap(),
            WorthQueryApplicationEntityKey::new(id).unwrap(),
        ))?;
    }
    Ok(())
}

pub(super) fn install(
    initial_tasks: &[(u64, &str, i64, i64)],
    memory_bytes: Option<u64>,
    work_ceiling: Option<u64>,
) -> Host {
    let declaration = DaymarkSchema::declaration().unwrap();
    let tasks: Vec<(u64, String, i64, i64)> = initial_tasks
        .iter()
        .map(|&(k, t, s, e)| (k, t.to_owned(), s, e))
        .collect();
    in_memory_program(
        ApplicationProgramAuthoring::<DaymarkSchema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("7b7b7b7b-7b7b-7b7b-7b7b-7b7b7b7b7b7b").unwrap(),
        limits(memory_bytes, work_ceiling),
        move |_, graph, installed| {
            let borrowed: Vec<(u64, &str, i64, i64)> = tasks
                .iter()
                .map(|(k, t, s, e)| (*k, t.as_str(), *s, *e))
                .collect();
            seed_workspace(graph, installed, &borrowed)
        },
    )
    .unwrap()
}
