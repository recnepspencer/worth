//! Daymark v1 and v2 schemas, contributions, programs, and Home requests.

use super::schema::*;
use super::v2_mutations::{
    CreateTaskV2Binding, CreateTaskV2Handler, CreateTaskV2Op, UpdateTaskV2Binding,
    UpdateTaskV2Handler, UpdateTaskV2Op,
};
use super::v2_queries::{AllTasksV2Query, AllTasksV2ReadBinding};
use worth_query_decl::facade::{
    application_program::*,
    application_schema::{
        fragment::{self, *},
        *,
    },
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

// ---- V1 Program (for wrong-predecessor refusal tests) ----

pub struct DaymarkV1Feature;
impl ApplicationFeature<DaymarkV2Schema> for DaymarkV1Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "daymark.v1.feature.v1";
}

pub struct DaymarkV1Program;
impl ApplicationProgramDefinition<DaymarkV2Schema> for DaymarkV1Program {
    type Contributions = (DaymarkV2Surface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("daymark.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DaymarkV2Schema, DaymarkV1Feature>()
                .action::<CreateTaskV2Binding, CreateTaskV2Delivery>()
                .finish_action()
                .finish(),
        ]
    }
}

pub fn v1_request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
) -> WorthQueryApplicationHomeRequest<'static, DaymarkV2Schema, DaymarkV1Program> {
    let declaration = DaymarkV2Schema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<DaymarkV2Schema, DaymarkV1Program>::begin()
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

// ---- V2 Schema, Contribution & Program ----

pub struct DaymarkV2Schema;
impl Fragment for DaymarkV2Schema {
    const NAME: &'static str = "DaymarkSchema";
    type Bindings = NoBindings;
    type Members = (DaymarkV2Surface,);
}
impl ApplicationSchema for DaymarkV2Schema {
    const OWNER: &'static str = "daymark_transition_court";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for DaymarkV2Schema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct DaymarkV2Surface;
impl ApplicationSchemaContribution<DaymarkV2Schema> for DaymarkV2Surface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("daymark.transition.surface.v2");
    type Members = (
        DaymarkCore,
        DaymarkPrincipalBinding<DaymarkV2Schema>,
        AllTasksV2Query,
        AllTasksV2ReadBinding,
        CreateTaskV2Op,
        CreateTaskV2Binding,
        UpdateTaskV2Op,
        UpdateTaskV2Binding,
    );
}
impl WorthQueryApplicationContribution<DaymarkV2Schema> for DaymarkV2Surface {
    type Configuration = ();
    fn configure(
        _: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DaymarkV2Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<CreateTaskV2Binding, _>(CreateTaskV2Handler)?;
        setup.handler::<UpdateTaskV2Binding, _>(UpdateTaskV2Handler)?;
        Ok(())
    }
}

pub struct DaymarkV2Feature;
impl ApplicationFeature<DaymarkV2Schema> for DaymarkV2Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "daymark.v2.feature.v1";
}
pub struct CreateTaskV2Delivery;
impl ApplicationActionDelivery<DaymarkV2Schema, CreateTaskV2Binding> for CreateTaskV2Delivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "daymark.v2.create-task.delivery.v1";
}
pub struct UpdateTaskV2Delivery;
impl ApplicationActionDelivery<DaymarkV2Schema, UpdateTaskV2Binding> for UpdateTaskV2Delivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "daymark.v2.update-task.delivery.v1";
}

pub struct DaymarkV2Program;
impl ApplicationProgramDefinition<DaymarkV2Schema> for DaymarkV2Program {
    type Contributions = (DaymarkV2Surface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("daymark.program.v2");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DaymarkV2Schema, DaymarkV2Feature>()
                .action::<CreateTaskV2Binding, CreateTaskV2Delivery>()
                .finish_action()
                .action::<UpdateTaskV2Binding, UpdateTaskV2Delivery>()
                .finish_action()
                .finish(),
        ]
    }
}

pub fn v2_request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
) -> WorthQueryApplicationHomeRequest<'static, DaymarkV2Schema, DaymarkV2Program> {
    let declaration = DaymarkV2Schema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<DaymarkV2Schema, DaymarkV2Program>::begin()
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
