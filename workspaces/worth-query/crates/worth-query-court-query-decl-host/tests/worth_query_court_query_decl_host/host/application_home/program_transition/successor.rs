//! The real closed successor application, including its typed B bindings.
use super::super::*;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_installation::*, WorthQueryApplicationHomeLocation,
    WorthQueryApplicationHomeRequest,
};
pub(super) mod balance_query;
mod migration;
mod native_contribution;
pub(super) mod native_mutation;
pub(super) mod schema;
pub(super) use migration::{body, schema_change};
use native_contribution::ExtendedSurface;
pub(super) use schema::SuccessorSchema;

pub struct ExtendedProgram;
pub struct BalanceCompletion;
impl ApplicationActionDelivery<SuccessorSchema, native_mutation::SetBalanceBinding>
    for BalanceCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.host.home.balance-completion.v1";
}
struct IdentityFeature;
impl ApplicationFeature<SuccessorSchema> for IdentityFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.host.home-identity.feature.v1";
}
impl ApplicationProgramDefinition<SuccessorSchema> for ExtendedProgram {
    type Contributions = (ExtendedSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home-additive-successor.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<SuccessorSchema, IdentityFeature>()
                .action::<native_mutation::SetBalanceBinding, BalanceCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}
pub(super) fn request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
) -> WorthQueryApplicationHomeRequest<'static, SuccessorSchema, ExtendedProgram> {
    let declaration = SuccessorSchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<SuccessorSchema, ExtendedProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (contacts,),
        build,
        WorthQueryInMemoryApplicationLimits::new(
            identity::world_resources(),
            worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::default(),
            worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(
            ),
        ),
        |_, graph, schema| schema::seed(graph, schema),
    )
}
