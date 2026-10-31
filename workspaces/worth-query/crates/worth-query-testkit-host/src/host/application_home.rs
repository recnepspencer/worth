//! Public Host inputs reuse the existing genuine identity/read fixture.
use super::application_runtime_surface::{
    self as identity, HostIdentitySchema, HostIdentitySurface,
};
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

pub struct HomeProgram;
struct IdentityFeature;
impl ApplicationFeature<HostIdentitySchema> for IdentityFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.host.home-identity.feature.v1";
}
impl ApplicationProgramDefinition<HostIdentitySchema> for HomeProgram {
    type Contributions = (HostIdentitySurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.home-identity.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<HostIdentitySchema, IdentityFeature>().finish()]
    }
}
pub fn location() -> std::path::PathBuf {
    worth_harness::tooling::fixture_path::unique_path("query-directory-home")
}
pub fn request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
) -> WorthQueryApplicationHomeRequest<'static, HostIdentitySchema, HomeProgram> {
    let declaration = HostIdentitySchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<HostIdentitySchema, HomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        build,
        WorthQueryInMemoryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        move |_, graph, schema| {
            initialized.fetch_add(1, Ordering::SeqCst);
            identity::seed_identity_graph(graph, schema)
        },
    )
}
