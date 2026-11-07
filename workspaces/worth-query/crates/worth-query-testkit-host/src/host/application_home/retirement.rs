//! An authored program with the same schema and an explicitly refusing handler.
use super::*;
use std::sync::atomic::AtomicBool;
use worth_query_decl::facade::application_schema::{
    ApplicationSchemaContribution, ApplicationSchemaContributionIdentity,
    ApplicationSchemaDeclarationBuilder,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub struct RefusingHomeProgram;
pub struct RefusingNativeSurface;
pub struct RefusingHandlerConfiguration {
    pub contacts: Arc<AtomicUsize>,
    pub refusal: Arc<AtomicBool>,
}
impl ApplicationSchemaContribution<HomeIdentitySchema> for RefusingNativeSurface {
    // This program configures the SAME declared schema contribution. Its
    // distinct program identity selects the explicit supplier implementation.
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <HomeNativeSurface as ApplicationSchemaContribution<HomeIdentitySchema>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
    ) -> ApplicationSchemaDeclarationBuilder<HomeIdentitySchema> {
        <HomeNativeSurface as ApplicationSchemaContribution<HomeIdentitySchema>>::register_members(
            builder,
        )
    }
}
impl WorthQueryApplicationContribution<HomeIdentitySchema> for RefusingNativeSurface {
    type Configuration = RefusingHandlerConfiguration;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, HomeIdentitySchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<native_mutation::SetBalanceBinding, _>(
            native_mutation::RefusingBalanceHandler {
                original: native_mutation::BalanceHandler(configuration.contacts),
                refusal: configuration.refusal,
            },
        )?;
        Ok(())
    }
}
impl ApplicationProgramDefinition<HomeIdentitySchema> for RefusingHomeProgram {
    type Contributions = (RefusingNativeSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.home-retirement-refusal.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <HomeProgram as ApplicationProgramDefinition<HomeIdentitySchema>>::feature_specs()
    }
}
pub fn request_with_handler_retirement_refusal(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    refusal: Arc<AtomicBool>,
) -> WorthQueryApplicationHomeRequest<'static, HomeIdentitySchema, RefusingHomeProgram> {
    let declaration = HomeIdentitySchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<HomeIdentitySchema, RefusingHomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (RefusingHandlerConfiguration { contacts, refusal },),
        build,
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        |_, graph, schema| schema::seed(graph, schema),
    )
}
