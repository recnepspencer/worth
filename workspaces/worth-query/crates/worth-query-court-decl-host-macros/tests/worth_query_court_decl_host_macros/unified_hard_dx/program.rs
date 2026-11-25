//! Unified Daymark program declaration, managed-index installation, seeding, and owner auth.
use std::time::Duration;

use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::{
    application_history::ApplicationHistoryDefinition,
    application_query::ApplicationQueryDisclosureContract, authentication::*,
};
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationContributionSetup, WorthQueryApplicationManagedIndexRegistration,
};
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;
use worth_query_host::facade::ordered_index::WorthQueryTextPrefixIndex;
use worth_query_host::facade::{
    primary_graph::*, worth_query_application_program, WorthQueryApplicationHome,
};

use super::actions::*;
use super::body_links::*;
use super::queries::*;
use super::schema::*;
use super::title_correction::*;

worth_query_decl::facade::worth_query_principal_issuer!(DaymarkIssuer);
worth_query_decl::facade::worth_query_authentication_audience!(HostAudience);
worth_query_decl::facade::worth_query_authentication_method!(LocalProcessMethod);

impl WorthQueryApplicationManagedIndexRegistration<DaymarkSchema> for TaskTitleIndex {
    fn install_managed_index(
        setup: &mut WorthQueryApplicationContributionSetup<'_, DaymarkSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.managed_index_with_predicate_routing::<TaskTitleIndex, _>(
            WorthQueryTextPrefixIndex::ascii_case_insensitive(TaskRow::title()),
        )
    }
}

worth_query_application_program! {
    pub program DaymarkProgram for DaymarkSchema {
        identity: "daymark.unified.program.v1",
        contribution: DaymarkContribution {
            identity: "daymark.unified.contribution.v1",
            members: [DaymarkFragment, DaymarkPrincipal, CorrectTaskState, CorrectTaskTitle],
        },
        managed_indexes: [TaskTitleIndex],
        feature: DaymarkFeature => DaymarkDefinition / DaymarkRuntime {
            identity: "daymark.unified.feature.v1",
            schema: |schema| {
                schema
                    .declared_operation(restore_task_state_definition())
                    .declared_operation(restore_task_title_definition())
                    .application_history(
                        ApplicationHistoryDefinition::<DaymarkSchema, DaymarkHistory>::for_binding(
                            ApplicationQueryDisclosureContract::public(),
                        )
                        .operation::<RenameProjectBinding>("Rename project"),
                    )
            };
            query(ReadyTasksBinding) { definition: ReadyTasksQuery::definition(); };
            query(LiveReadyTasksBinding) { definition: LiveReadyTasksQuery::definition(); };
            query(BlockedTasksBinding) { definition: BlockedTasksQuery::definition(); };
            query(TaskSourceBinding) { definition: TaskSourceQuery::definition(); };
            query(FindTasksByTitleBinding) { definition: FindTasksByTitleQuery::definition(); };
            query(ReadTaskDetailsBinding) { definition: ReadTaskDetailsQuery::definition(); };
            query(ReadProjectTasksBinding) { definition: ReadProjectTasksQuery::definition(); };
            body_action(RescheduleTaskBinding, RescheduleTaskDelivery) as RescheduleTaskAction {
                operation: reschedule_task_definition();
            };
            body_action(RenameProjectBinding, RenameProjectDelivery) as RenameProjectAction {
                operation: rename_project_definition();
            };
            body_action(ProbeOccupiedTaskStateBinding, ProbeOccupiedTaskStateDelivery) as ProbeOccupiedTaskStateAction {
                operation: probe_occupied_task_state_definition();
            };
            body_action(ExhaustedKeyTaskStateBinding, ExhaustedKeyTaskStateDelivery) as ExhaustedKeyTaskStateAction {
                operation: exhausted_key_task_state_definition();
            };
            query(NestedProjectsBinding) { definition: NestedProjectsQuery::definition(); };
            query(WatchedProjectsBinding) { definition: WatchedProjectsQuery::definition(); };
            body_action(NewProjectBinding, NewProjectDelivery) as NewProjectAction {
                operation: new_project_definition();
            };
            body_action(SetDependencyBinding, SetDependencyDelivery) as SetDependencyAction {
                operation: set_dependency_definition();
            };
            body_action_ref(RestoreTaskStateBinding, RestoreTaskStateDelivery) as RestoreTaskStateAction {};
            body_action_ref(SetTaskStateBinding, SetTaskStateDelivery) as SetTaskStateAction {};
            body_action_ref(RenameTaskBinding, RenameTaskDelivery) as RenameTaskAction {};
            body_action_ref(RestoreTaskTitleBinding, RestoreTaskTitleDelivery) as RestoreTaskTitleAction {};
        }
    }
}

pub type DaymarkHome = WorthQueryApplicationHome<DaymarkSchema, DaymarkProgram>;

pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DaymarkSchema>,
    installed: &WorthQueryInstalledApplicationSchema<DaymarkSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let base = CanonicalDate::parse_iso("2026-11-24").unwrap();
    graph.bind_principal(
        &installed
            .principal_binding(DaymarkPrincipal::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("daymark-owner").unwrap(),
        1_u64,
        owner_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    for (project_id, label) in [(1_u64, "Core"), (2_u64, "Backlog")] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Project::reference(), key("project", project_id))
                .field(ProjectKey::at("ProjectKey"), project_id)
                .field(Label::at("Label"), label.to_owned()),
        )?;
    }
    let tasks: [(u64, u64, &str, &str, u64, i64); 4] = [
        (1, 1, "Draft spec", "OPEN", 1, 1),  // 2026-11-25, OPEN, unblocked
        (2, 1, "Draft tests", "OPEN", 1, 2), // 2026-11-26, OPEN, blocked by 1
        (3, 1, "Review docs", "OPEN", 2, 5), // 2026-11-29, OPEN, unblocked
        (4, 1, "Ship kickoff", "DONE", 1, 0), // 2026-11-24, DONE
    ];
    for (task_id, project_id, title, state, priority, offset) in tasks {
        let due = base.add_days(offset).unwrap();
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Task::reference(), key("task", task_id))
                .field(TaskKey::at("TaskKey"), task_id)
                .field(Title::at("Title"), title.to_owned())
                .field(State::at("State"), state.to_owned())
                .field(Priority::at("Priority"), priority)
                .field(DueDate::at("DueDate"), due),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Contains::reference(),
            format!("contains-{task_id}"),
            key("project", project_id),
            key("task", task_id),
        ))?;
    }
    // Task 2 is blocked by Task 1.
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        BlockedBy::reference(),
        "blocked-2-by-1".to_owned(),
        key("task", 2),
        key("task", 1),
    ))?;
    Ok(())
}

fn key<E>(kind: &str, id: u64) -> WorthQueryApplicationEntityKey<DaymarkSchema, E> {
    WorthQueryApplicationEntityKey::new(format!("{kind}-{id}")).unwrap()
}

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(DaymarkIssuer, "owner").unwrap()
}

pub fn owner(
    home: &DaymarkHome,
    clock: &WorthQueryAdmissionClock,
) -> WorthQueryAuthenticatedExternalPrincipal<DaymarkSchema> {
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    let audience = || WorthQueryAuthenticationAudience::of(HostAudience);
    let method = || WorthQueryAuthenticationMethod::of(LocalProcessMethod);
    let local = WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        HostAudience,
        LocalProcessMethod,
        Duration::from_secs(600),
    )
    .expect("finite local lifetime admits");
    admit_authentication_adapter(
        home.primary_graph().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(audience(), method(), clock.clone()),
        local,
    )
    .expect("local adapter admits")
    .authenticate_local(&scope)
    .expect("local process authenticates")
}
