//! Two contributions own the declared features; one program and one Host
//! install them. Court switches reach the real refusal boundaries.
use super::{flags::*, schema::*, tasks::*};
use std::cell::Cell;
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};
use worth_query_testkit_document_retention::facade::{
    document_retention_model::operator_identity::operator_identity,
    document_world_resources::world_resources,
};

/// Features the board contribution owns; `FlagFamily<Schema, Pin>` belongs to `Pinning`.
pub type BoardFeatures = (TasksDefinition, FlagFamily<Schema, Archive>);
type AllFeatures = (BoardFeatures, FlagFamily<Schema, Pin>);

thread_local! {
    /// When set, the board contribution also configures the pinning feature,
    /// whose members it does not own.
    pub static MISROUTE_PIN: Cell<bool> = const { Cell::new(false) };
}

pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("single-source.board-contribution.v1");
    type Members = (Board, PrincipalBinding, Members<BoardFeatures>);
}
pub struct Pinning;
impl ApplicationSchemaContribution<Schema> for Pinning {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("single-source.pinning.v1");
    type Members = (Members<FlagFamily<Schema, Pin>>,);
}

/// Registers a feature definition's declared members.
pub struct Members<Features>(std::marker::PhantomData<fn() -> Features>);
impl<Features: ApplicationFeatureDefinition<Schema> + 'static> Registration<Schema>
    for Members<Features>
{
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(Features::declare_members(schema))
    }
}

impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = <BoardFeatures as WorthQueryFeatureDefinition<Schema>>::Configuration;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        BoardFeatures::configure(configuration, setup)?;
        if MISROUTE_PIN.with(Cell::get) {
            FlagFamily::<Schema, Pin>::configure(FlagRuntime::new(), setup)?;
        }
        Ok(())
    }
}
impl WorthQueryApplicationContribution<Schema> for Pinning {
    type Configuration = FlagRuntime<Schema, Pin>;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        FlagFamily::<Schema, Pin>::configure(configuration, setup)
    }
}

pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution, Pinning);
    type Outputs =
        ApplicationProgramOutputs<<AllFeatures as ApplicationFeatureDefinition<Schema>>::Outputs>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("single-source.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        AllFeatures::feature_specs()
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

pub const TASKS: [(u64, &str); 2] = [(1, "Draft plan"), (2, "Review plan")];

pub fn install() -> Result<Host, WorthQueryApplicationPreparationDenial> {
    let declaration = Schema::declaration().expect("the board schema declares");
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .expect("the board program validates"),
        declaration,
        (
            (TasksRuntime::new(), FlagRuntime::new()),
            FlagRuntime::new(),
        ),
        ApplicationBuildIdentity::parse("5e5e5e5e-5e5e-5e5e-5e5e-5e5e5e5e5e5e").unwrap(),
        WorthQueryApplicationLimits::new(
            world_resources(128, 64 * 1024 * 1024, 256, WorthQueryAdmissionClock::system()),
            WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(8).unwrap()),
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
            for (key, title) in TASKS {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Task::reference(),
                        WorthQueryApplicationEntityKey::new(format!("task-{key}")).unwrap(),
                    )
                    .field(TaskKey::at("TaskKey"), key)
                    .field(Title::at("Title"), title.to_owned())
                    .field(Archived::at("Archived"), false)
                    .field(Pinned::at("Pinned"), false)
                    .field(Due::at("Due"), CanonicalDate { days_from_unix_epoch: 20_000 + key as i32 }),
                )?;
            }
            Ok(())
        },
    )
}
