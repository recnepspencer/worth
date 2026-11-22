//! One feature, contribution and program; an in-memory Host under bounded
//! serial limits; and the local-process principal that uses it.
use super::{actions::*, reads, relations, schema::*};
use std::time::Duration;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
    worth_query_application_contribution,
};

pub struct BoardFeature;
impl ApplicationFeature<Schema> for BoardFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "macros.board.feature.v1";
}
worth_query_application_contribution! {
    pub feature BoardDefinition<Schema> for BoardFeature => BoardRuntime {
        schema: |schema| {
            schema
                .application_query(reads::search())
                .application_query_binding::<reads::SearchBinding>()
                .application_query(reads::due())
                .application_query_binding::<reads::DueBinding>()
                .application_query(relations::outline())
                .application_query_binding::<relations::OutlineQueryBinding>()
        };
        body_action(RenameBinding, Renamed) { operation: rename_definition(); };
        body_action(RescheduleBinding, Rescheduled) { operation: reschedule_definition(); };
        body_action(CompleteBinding, Completed) { operation: complete_definition(); };
    }
}

pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("macros.board-contribution.v1");
    type Members = (Board, PrincipalBinding, Features);
}
/// Registers the feature definition's declared members.
pub struct Features;
impl Registration<Schema> for Features {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(BoardDefinition::declare_members(schema))
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = <BoardDefinition as WorthQueryFeatureDefinition<Schema>>::Configuration;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        BoardDefinition::configure(configuration, setup)
    }
}

pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<
        <BoardDefinition as ApplicationFeatureDefinition<Schema>>::Outputs,
    >;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("macros.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        BoardDefinition::feature_specs()
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
        |_, graph, installed| {
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
                    WorthQueryApplicationEntitySeed::new(
                        Project::reference(),
                        key("project", project),
                    )
                    .field(ProjectKey::at("ProjectKey"), project),
                )?;
            }
            for (task, project, title, notes, due) in TASKS {
                let mut seed =
                    WorthQueryApplicationEntitySeed::new(Task::reference(), key("task", task))
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
        },
    )
    .expect("the derived board installs")
}

fn key<E>(kind: &str, key: u64) -> WorthQueryApplicationEntityKey<Schema, E> {
    WorthQueryApplicationEntityKey::new(format!("{kind}-{key}")).unwrap()
}

fn owner_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("local://board", "owner").unwrap()
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
    let audience = || WorthQueryAuthenticationAudience::new("host").unwrap();
    let method = || WorthQueryAuthenticationMethod::new("local-process").unwrap();
    let local = WorthQueryLocalProcessPrincipal::new(
        "macros.local-owner.v1",
        owner_identity(),
        audience(),
        method(),
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
