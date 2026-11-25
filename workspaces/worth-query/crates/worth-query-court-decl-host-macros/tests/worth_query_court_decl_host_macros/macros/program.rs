//! One feature, contribution and program collapsed through
//! `worth_query_application_program!`; an in-memory Host under bounded
//! serial limits; and the local-process principal that uses it.
use super::completion_correction::{self, CorrectCompletion, RestoreDoneBinding, RestoredDone};
use super::{actions::*, reads, relations, schema::*};
use std::time::Duration;
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;
use worth_query_host::facade::{
    application_installation::*, primary_graph::*, worth_query_application_program,
};

worth_query_decl::facade::worth_query_principal_issuer!(BoardIssuer);
worth_query_decl::facade::worth_query_authentication_audience!(HostAudience);
worth_query_decl::facade::worth_query_authentication_method!(LocalProcessMethod);

worth_query_application_program! {
    pub program Program for Schema {
        identity: "macros.program.v1",
        contribution: Contribution {
            identity: "macros.board-contribution.v1",
            members: [Board, PrincipalBinding, CorrectCompletion],
        },
        feature: BoardFeature => BoardDefinition / BoardRuntime {
            identity: "macros.board.feature.v1",
            schema: |schema| {
                schema.declared_operation(completion_correction::restore_done())
            };
            query(completion_correction::DetailsBinding) {
                definition: completion_correction::task_details();
            };
            query(reads::SearchBinding) { definition: reads::search(); };
            query(reads::DueBinding) { definition: reads::due(); };
            query(reads::TitleSearchBinding) { definition: reads::title_search(); };
            query(relations::OutlineQueryBinding) { definition: relations::outline(); };
            body_action(RenameBinding, Renamed) as RenameAction { operation: rename_definition(); };
            body_action(RescheduleBinding, Rescheduled) as RescheduleAction { operation: reschedule_definition(); };
            body_action(RetitleFirstBinding, RetitledFirst) as RetitleFirstAction { operation: retitle_first_definition(); };
            body_action_ref(CompleteBinding, Completed) as CompleteAction {};
            body_action_ref(RestoreDoneBinding, RestoredDone) as RestoreAction {};
        }
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

/// One seeded task: `(key, project, title, notes, due day)`.
pub type Seed = (u64, u64, &'static str, Option<&'static str>, Option<i32>);
pub const TASKS: [Seed; 5] = [
    (
        1,
        1,
        "Mill housing",
        Some("check FLANGE tolerance"),
        Some(100),
    ),
    (2, 1, "Drill flange holes", None, None),
    (3, 1, "Paint", Some("matte black"), Some(50)),
    (4, 1, "Inspect", None, Some(200)),
    (5, 2, "Flange spare", None, Some(10)),
];

pub fn day(days: i32) -> CanonicalDate {
    CanonicalDate {
        days_from_unix_epoch: days,
    }
}

pub fn install(clock: &WorthQueryAdmissionClock) -> Host {
    let declaration = Schema::declaration().expect("the derived board declares");
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .install()
        .expect("the bounded serial preset installs");
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .expect("the board program validates"),
        declaration,
        (BoardRuntime::new(),),
        ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap(),
        limits,
        |_, graph, installed| seed(graph, installed),
    )
    .expect("the derived board installs")
}

pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed
            .principal_binding(PrincipalBinding::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("board-owner").unwrap(),
        1_u64,
        owner_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    for project in [1_u64, 2] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(Project::reference(), key("project", project))
                .field(ProjectKey::at("ProjectKey"), project),
        )?;
    }
    for (task, project, title, notes, due) in TASKS {
        let mut seed = WorthQueryApplicationEntitySeed::new(Task::reference(), key("task", task))
            .field(TaskKey::at("TaskKey"), task)
            .field(Title::at("Title"), title.to_owned())
            .field(Done::at("Done"), false);
        if let Some(notes) = notes {
            seed = seed.field(Notes::at("Notes"), notes.to_owned());
        }
        if let Some(due) = due {
            seed = seed.field(Due::at("Due"), day(due));
        }
        graph.bind_entity(seed)?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            Contains::reference(),
            format!("contains-{task}"),
            key("project", project),
            key("task", task),
        ))?;
    }
    Ok(())
}

fn key<E>(kind: &str, key: u64) -> WorthQueryApplicationEntityKey<Schema, E> {
    WorthQueryApplicationEntityKey::new(format!("{kind}-{key}")).unwrap()
}

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(BoardIssuer, "owner").unwrap()
}

pub fn scope(clock: &WorthQueryAdmissionClock) -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    )
}

/// The process's own owner, authenticated on the calling thread.
pub fn owner(
    host: &Host,
    clock: &WorthQueryAdmissionClock,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    let audience = || WorthQueryAuthenticationAudience::of(HostAudience);
    let method = || WorthQueryAuthenticationMethod::of(LocalProcessMethod);
    let local = WorthQueryLocalProcessPrincipal::new(
        owner_identity(),
        HostAudience,
        LocalProcessMethod,
        Duration::from_secs(600),
    )
    .expect("a finite local lifetime is admitted");
    admit_authentication_adapter(
        host.runtime().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(audience(), method(), clock.clone()),
        local,
    )
    .expect("the local adapter is admitted")
    .authenticate_local(scope)
    .expect("the local process authenticates")
}
