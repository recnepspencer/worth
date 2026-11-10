//! One genuine Home composition of the existing label schema and handler.
use super::*;
use label::{schema_declaration::IdentityExecutionContribution, IdentityExecutionSchema};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    application_installation::{WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster},
    primary_graph::{SignalConditionalEvaluationBudget, WorthQueryPrimaryGraphInstallationDenial},
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
pub(super) struct HomeLabelProgram;
pub(super) struct LabelSurface;
impl ApplicationSchemaContribution<IdentityExecutionSchema> for LabelSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity = <IdentityExecutionContribution as ApplicationSchemaContribution<IdentityExecutionSchema>>::IDENTITY;
    type Members = <IdentityExecutionContribution as ApplicationSchemaContribution<
        IdentityExecutionSchema,
    >>::Members;
}
impl WorthQueryApplicationContribution<IdentityExecutionSchema> for LabelSurface {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, IdentityExecutionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<LabelBinding, _>(LabelHandler)?;
        setup.handler::<label::program_required_binding::ProgramRequiredMutationBinding, _>(
            label::program_required_binding::ProgramRequiredHandler,
        )?;
        setup.handler::<label::capability_touch_binding::CapabilityTouchMutationBinding, _>(
            label::capability_touch_binding::CapabilityTouchHandler,
        )?;
        setup.handler::<label::optional_output_binding::OptionalOutputMutationBinding, _>(
            label::optional_output_binding::OptionalOutputHandler,
        )
    }
}
struct LabelFeature;
impl ApplicationFeature<IdentityExecutionSchema> for LabelFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.home.label.feature.v1";
}
impl ApplicationProgramDefinition<IdentityExecutionSchema> for HomeLabelProgram {
    type Contributions = (LabelSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home.label.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, LabelFeature>()
                .native_action::<LabelBinding>()
                .finish_action()
                .native_action::<label::program_required_binding::ProgramRequiredMutationBinding>()
                .finish_action()
                .finish(),
        ]
    }
}
pub(super) type Home =
    worth_query_host::facade::WorthQueryApplicationHome<IdentityExecutionSchema, HomeLabelProgram>;
pub(super) fn request(
    location: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<'static, IdentityExecutionSchema, HomeLabelProgram> {
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<IdentityExecutionSchema, HomeLabelProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, schema| {
            let binding = schema
                .principal_binding(label::IdentityBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                worth_query_host::facade::primary_graph::WorthQueryApplicationPrincipalKey::new(
                    "principal-0",
                )
                .unwrap(),
                1,
                label::schema_types::external_identity("alice"),
                worth_query_decl::facade::authentication::WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            label::account_seed::bind_account(
                graph,
                label::account_seed::AccountSeedSpec {
                    key: "account-1",
                    status: "open",
                    label: "primary",
                    note: None,
                },
            );
            graph.bind_relation(
                worth_query_host::facade::primary_graph::WorthQueryApplicationRelationSeed::new(
                    label::AccountOwner::reference(),
                    "account-owner",
                    worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey::new(
                        "principal-0",
                    )
                    .unwrap(),
                    worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey::new(
                        "account-1",
                    )
                    .unwrap(),
                ),
            )?;
            Ok(())
        },
    )
}
