//! Daymark v3 schema, contribution, program, Home request, and declarative
//! `worth_query_program_transition!` declarations.

use super::schema::*;
use super::v2_program::{DaymarkV2Program, DaymarkV2Schema};
use super::v3_mutations::{
    CreateTaskV3Binding, CreateTaskV3Handler, CreateTaskV3Op, UpdateTaskV3Binding,
    UpdateTaskV3Handler, UpdateTaskV3Op,
};
use super::v3_queries::{
    AllTasksV3Query, AllTasksV3ReadBinding, TasksByPriorityQuery, TasksByPriorityReadBinding,
};
use worth_query_decl::facade::{
    application_program::*,
    application_schema::{
        fragment::{self, *},
        *,
    },
    worth_query_program_transition,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    application_installation::*,
    primary_graph::{SignalConditionalEvaluationBudget, WorthQueryPrimaryGraphInstallationDenial},
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::application_runtime_surface as identity;

pub struct DaymarkV3Schema;
impl Fragment for DaymarkV3Schema {
    const NAME: &'static str = "DaymarkSchema";
    type Bindings = NoBindings;
    type Members = (DaymarkV3Surface,);
}
impl ApplicationSchema for DaymarkV3Schema {
    const OWNER: &'static str = "daymark_transition_court";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for DaymarkV3Schema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct DaymarkV3QueryMembers;
impl Fragment for DaymarkV3QueryMembers {
    const NAME: &'static str = "daymark_v3_query_members";
    type Bindings = NoBindings;
    type Members = (
        AllTasksV3Query,
        AllTasksV3ReadBinding,
        TasksByPriorityQuery,
        TasksByPriorityReadBinding,
    );
}

pub struct DaymarkV3Surface;
impl ApplicationSchemaContribution<DaymarkV3Schema> for DaymarkV3Surface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("daymark.transition.surface.v3");
    type Members = (
        DaymarkCore,
        DaymarkV3Extensions,
        DaymarkPrincipalBinding<DaymarkV3Schema>,
        DaymarkV3QueryMembers,
        CreateTaskV3Op,
        CreateTaskV3Binding,
        UpdateTaskV3Op,
        UpdateTaskV3Binding,
    );
}
impl WorthQueryApplicationContribution<DaymarkV3Schema> for DaymarkV3Surface {
    type Configuration = ();
    fn configure(
        _: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DaymarkV3Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<CreateTaskV3Binding, _>(CreateTaskV3Handler)?;
        setup.handler::<UpdateTaskV3Binding, _>(UpdateTaskV3Handler)?;
        Ok(())
    }
}

pub struct DaymarkV3Feature;
impl ApplicationFeature<DaymarkV3Schema> for DaymarkV3Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "daymark.v3.feature.v1";
}
pub struct CreateTaskV3Delivery;
impl ApplicationActionDelivery<DaymarkV3Schema, CreateTaskV3Binding> for CreateTaskV3Delivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "daymark.v3.create-task.delivery.v1";
}
pub struct UpdateTaskV3Delivery;
impl ApplicationActionDelivery<DaymarkV3Schema, UpdateTaskV3Binding> for UpdateTaskV3Delivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "daymark.v3.update-task.delivery.v1";
}

pub struct DaymarkV3Program;
impl ApplicationProgramDefinition<DaymarkV3Schema> for DaymarkV3Program {
    type Contributions = (DaymarkV3Surface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("daymark.program.v3");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DaymarkV3Schema, DaymarkV3Feature>()
                .action::<CreateTaskV3Binding, CreateTaskV3Delivery>()
                .finish_action()
                .action::<UpdateTaskV3Binding, UpdateTaskV3Delivery>()
                .finish_action()
                .finish(),
        ]
    }
}

pub fn v3_request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
) -> WorthQueryApplicationHomeRequest<'static, DaymarkV3Schema, DaymarkV3Program> {
    let declaration = DaymarkV3Schema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<DaymarkV3Schema, DaymarkV3Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        build,
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, schema| seed_base_project(graph, schema),
    )
}

worth_query_program_transition! {
    /// Declarative in-place upgrade from Daymark v2 to Daymark v3.
    pub struct DaymarkV2ToV3;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    added_defaults = [
        task_priority_field()
            => default = "Normal".to_owned(), required = false, label = "Priority::Normal",
    ],
    rebuild_indexes = [task_priority_field()],
    description = "Add TaskPriority::Priority defaulting to Priority::Normal",
}

worth_query_program_transition! {
    /// Deliberate downgrade declaration used to certify fail-closed downgrade refusal.
    pub struct DaymarkV3ToV2Downgrade;
    from = (DaymarkV3Schema, DaymarkV3Program, version = 3)
        => to = (DaymarkV2Schema, DaymarkV2Program, version = 2),
    retained_records = [Project, Task],
    description = "Illegal v3 -> v2 downgrade",
}

worth_query_program_transition! {
    /// Deliberate duplicate retained record declaration.
    pub struct DuplicateRetainedTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task, Task],
    description = "Duplicate retained record",
}

worth_query_program_transition! {
    /// Deliberate duplicate added record declaration.
    pub struct DuplicateAddedTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    added_records = [Task, Task],
    description = "Duplicate added record",
}

worth_query_program_transition! {
    /// Deliberate record present in both retained and added lists.
    pub struct OverlapRetainedAddedTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    added_records = [Task],
    description = "Overlap between retained and added records",
}

worth_query_program_transition! {
    /// Deliberate duplicate index rebuild declaration.
    pub struct DuplicateIndexRebuildTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    rebuild_indexes = [task_priority_field(), task_priority_field()],
    description = "Duplicate index rebuild",
}

worth_query_program_transition! {
    /// Deliberate default targeting a field that ALREADY exists in predecessor v2.
    pub struct OverwritePredecessorFieldTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    added_defaults = [
        task_v2_value_field()
            => default = "clobber".to_owned(), required = false, label = "clobber",
    ],
    description = "Illegal overwrite of predecessor field",
}

worth_query_program_transition! {
    /// Deliberate index rebuild targeting a non-indexed field (`V2Value` is not equality_queryable).
    pub struct NonIndexedRebuildTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    added_defaults = [
        task_priority_field()
            => default = "Normal".to_owned(), required = false, label = "Priority::Normal",
    ],
    rebuild_indexes = [task_v2_value_field()],
    description = "Illegal rebuild of non-indexed field",
}

worth_query_program_transition! {
    /// Transition with an oversized raw string default used to certify exact
    /// pre-allocation retained-memory admission in backfill planning.
    pub struct OversizedDefaultTransition;
    from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
        => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
    retained_records = [Project, Task],
    added_defaults = [
        task_priority_field()
            => default = "X".repeat(24 * 1024 * 1024), required = false, label = "OversizedDefault",
    ],
    rebuild_indexes = [task_priority_field()],
    description = "Oversized default payload for retained-memory admission test",
}

/// Adversarial manual `WorthQueryProgramTransitionSpec` whose `descriptor()`
/// tries to spoof a valid `2 -> 3` descriptor for `3 -> 2` endpoints.
pub struct SpoofedEndpointsTransition;
impl worth_query_decl::facade::program_transition::WorthQueryProgramTransitionSpec
    for SpoofedEndpointsTransition
{
    type FromSchema = DaymarkV3Schema;
    type FromProgram = DaymarkV3Program;
    type ToSchema = DaymarkV2Schema;
    type ToProgram = DaymarkV2Program;

    const NAME: &'static str = "SpoofedEndpointsTransition";
    const FROM_VERSION: u32 = 3;
    const TO_VERSION: u32 = 2;

    fn descriptor() -> Result<
        worth_query_decl::facade::program_transition::WorthQueryProgramTransitionDescriptor,
        worth_query_decl::facade::program_transition::WorthQueryProgramTransitionDeclarationDenial,
    > {
        DaymarkV2ToV3::descriptor()
    }
}

/// Hostile caller-scope macro hygiene check: caller defines constants matching
/// every descriptor field name in the exact scope of `worth_query_program_transition!`.
#[allow(non_upper_case_globals, dead_code)]
pub mod hostile_macro_scope {
    use super::*;

    pub const retained_records: usize = 0;
    pub const added_records: usize = 0;
    pub const added_defaults: usize = 0;
    pub const index_rebuilds: usize = 0;
    pub const description: usize = 0;

    worth_query_program_transition! {
        pub struct HygienicV2ToV3;
        from = (DaymarkV2Schema, DaymarkV2Program, version = 2)
            => to = (DaymarkV3Schema, DaymarkV3Program, version = 3),
        retained_records = [Project, Task],
        added_defaults = [
            task_priority_field()
                => default = "Normal".to_owned(), required = false, label = "Priority::Normal",
        ],
        rebuild_indexes = [task_priority_field()],
        description = "Hygienic macro invocation under hostile constants",
    }
}
