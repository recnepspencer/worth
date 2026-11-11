//! A declared native Home program whose one action emits an external effect.
//!
//! It shares the account and principal records of `application_home` but is a
//! separate schema and program, so dispatching courts never change the
//! catalog the ordinary Home courts certify.
use super::application_runtime_surface as identity;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
use worth_query_host::facade::{
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    primary_graph::SignalConditionalEvaluationBudget,
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub mod notify;
pub mod recoverable;
pub mod schema;
use schema::{DispatchHomeSchema, DispatchSurface};

pub struct DispatchHomeProgram;
pub struct NotifyCompletion;
impl ApplicationActionDelivery<DispatchHomeSchema, notify::NotifyBalanceBinding>
    for NotifyCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.host.dispatch-home.notify-completion.v1";
}
pub struct RecoverableNoticeCompletion;
impl ApplicationActionDelivery<DispatchHomeSchema, recoverable::RecoverableNoticeBinding>
    for RecoverableNoticeCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str =
        "worth.query.host.dispatch-home.recoverable-notice-completion.v1";
}
struct DispatchFeature;
impl ApplicationFeature<DispatchHomeSchema> for DispatchFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.host.dispatch-home.feature.v1";
}
impl ApplicationProgramDefinition<DispatchHomeSchema> for DispatchHomeProgram {
    type Contributions = (DispatchSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.dispatch-home.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DispatchHomeSchema, DispatchFeature>()
                .action::<notify::NotifyBalanceBinding, NotifyCompletion>()
                .finish_action()
                .action::<recoverable::RecoverableNoticeBinding, RecoverableNoticeCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}

pub fn location() -> std::path::PathBuf {
    worth_harness::tooling::fixture_path::unique_path("query-dispatch-home")
}

/// A request for the Home at `location`. `seed` decides whether this opening
/// may initialize: a reopening passes `false` and panics if asked to.
pub fn request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    handler_contacts: Arc<AtomicUsize>,
    seed: bool,
) -> WorthQueryApplicationHomeRequest<'static, DispatchHomeSchema, DispatchHomeProgram> {
    let declaration = DispatchHomeSchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        ApplicationProgramAuthoring::<DispatchHomeSchema, DispatchHomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (handler_contacts,),
        build,
        WorthQueryApplicationLimits::new(
            identity::world_resources(),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        ),
        move |_, graph, installed| {
            assert!(seed, "an existing dispatch Home must never initialize");
            schema::seed(graph, installed)
        },
    )
}
