//! A concrete declared native Home program with genuine principal and account seed.
use super::application_runtime_surface as identity;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationProgramRoster,
        WorthQueryInMemoryApplicationLimits,
    },
    primary_graph::SignalConditionalEvaluationBudget,
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub mod balance_query;
mod native_contribution;
pub mod schema;
use schema::HomeIdentitySchema;
pub mod native_mutation;
use native_contribution::HomeNativeSurface;
pub struct HomeProgram;
pub struct BalanceCompletion;
impl ApplicationActionDelivery<HomeIdentitySchema, native_mutation::SetBalanceBinding>
    for BalanceCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.host.home.balance-completion.v1";
}
struct IdentityFeature;
impl ApplicationFeature<HomeIdentitySchema> for IdentityFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.host.home-identity.feature.v1";
}
impl ApplicationProgramDefinition<HomeIdentitySchema> for HomeProgram {
    type Contributions = (HomeNativeSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.home-identity.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<HomeIdentitySchema, IdentityFeature>()
                .action::<native_mutation::SetBalanceBinding, BalanceCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}
pub fn location() -> std::path::PathBuf {
    worth_harness::tooling::fixture_path::unique_path("query-directory-home")
}
pub fn request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
) -> WorthQueryApplicationHomeRequest<'static, HomeIdentitySchema, HomeProgram> {
    request_with_initializer(location, build, contacts, move |_, graph, schema| {
        initialized.fetch_add(1, Ordering::SeqCst);
        schema::seed(graph, schema)
    })
}

pub fn request_with_initializer(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    initial_state: impl FnOnce(
            &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
                '_,
            >,
            &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<
                HomeIdentitySchema,
            >,
            &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
                HomeIdentitySchema,
            >,
        ) -> Result<
            (),
            worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
        > + 'static,
) -> WorthQueryApplicationHomeRequest<'static, HomeIdentitySchema, HomeProgram> {
    let declaration = HomeIdentitySchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<HomeIdentitySchema, HomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (contacts,),
        build,
        WorthQueryInMemoryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        initial_state,
    )
}
