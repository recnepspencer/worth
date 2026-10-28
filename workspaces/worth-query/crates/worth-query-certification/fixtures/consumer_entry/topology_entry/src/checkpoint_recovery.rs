use std::num::NonZeroUsize;
use worth_query_decl::facade::application_program::{
    ApplicationCommitBoundary, ApplicationConnectionRef, ApplicationFeatureSpec,
    ApplicationOutputEdge, ApplicationOutputGraph, ApplicationOutputLeaf,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramOutputs, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ApplicationSharedRuleRef,
};
use worth_query_decl::facade::application_schema::ApplicationSchemaComposition;
use worth_query_decl::facade::worth_query_application;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_host::facade::{
    admission::authenticated_principal as authentication,
    application_entry::{
        WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
        WorthQueryOutputCurrentnessDenial, WorthQueryOutputDemandControls,
    },
    application_installation::{self, WorthQueryInMemoryApplicationLimits},
    primary_graph::{self, WorthQueryOutputDemandDenialKind},
};

use super::*;
mod advancement_currentness;
mod clean_reuse;
pub(crate) mod computation_partition;
mod contribution;
mod current_family_output;
mod current_output;
mod demand_contact;
mod demand_only;
mod demand_policy;
mod generated_restoration;
mod input_cutoff;
worth_execution::needs_os_threads! {
    mod late_cancellation;
}
pub(crate) mod mixed_retirement;
mod parallel_history;
mod producer_domain_denial;
pub(crate) mod required_chain;
pub(crate) mod resource_quote;
mod reuse_opt_out;
mod root_delivery;
mod stable_refresh;
mod support;
use support::{authenticate, install, length};

mod observation_scope;
mod output_availability;
mod provider_contact_custody;
use observation_scope::checkpoint_recovery_test_guard;

worth_query_application! {
    CheckpointSchema {
        owner: "worth.query.certification.checkpoint",
        version: (1, 0),
        contributions: [contribution::CheckpointContribution],
    }
}

impl TopologySchemaBinding for CheckpointSchema {
    const PLANAR_EDIT_PROJECTION_WORK: Option<usize> = None;

    // Bootstrap validates every newly authored ring. Its population is not a
    // rule-local resource policy; the original request still meters all work.
    fn planar_invariant_work() -> Option<std::num::NonZeroU64> {
        None
    }
}

type RootConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;
type FinalConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;
type AlternateConnection = ApplicationConnectionRef<
    CheckpointSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateDerivedBodyInput,
    PlanarOutputToAlternateFinalConnection,
>;
type CheckpointRoot = ApplicationOutputGraph<
    RootConnection,
    (
        ApplicationOutputEdge<FinalConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<AlternateConnection, ApplicationOutputLeaf>,
    ),
>;
type CheckpointRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<CheckpointSchema, PositivePlanarTurn>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

struct CheckpointProgram;

impl ApplicationProgramDefinition<CheckpointSchema> for CheckpointProgram {
    type Contributions = <CheckpointSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<CheckpointRoot>;
    type Rules = CheckpointRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.checkpoint-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs::<demand_policy::FinalArtifact>()
    }
}

#[path = "checkpoint_recovery/checkpoint_reopens_ready_output_without_producer_contact_and_rec.rs"]
mod checkpoint_reopens_ready_output_without_producer_contact_and_rec;

#[test]
fn recovered_output_survives_an_unrelated_settled_edit_without_producer_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let application = support::observed::install(None, &provider_observation);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the ready output is checkpointable")
        })
        .expect("installed host call opens");
    drop(application);

    let restored = support::observed::install(Some(checkpoint), &provider_observation);
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    drop(settle(&request, &restored));
    let unrelated = request
        .query(PlanarRead {
            body_key: "anchor-isolated".to_owned(),
        })
        .execute()
        .expect("the unrelated source is readable");
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-isolated".to_owned(),
            replacement_y: length(51),
        })
        .expect_source(unrelated.observed_sources()[0].clone())
        .idempotency(&88_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the unrelated edit settles");

    provider_observation.reset();
    let redemanded = settle(&request, &restored);
    assert_eq!(redemanded.work().producer_contact_count(), 0);
    assert_eq!(provider_observation.count(), 0);
}

#[test]
fn unadopted_recovered_output_survives_an_unrelated_edit_before_first_demand() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let application = support::observed::install(None, &provider_observation);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the ready output is checkpointable")
        })
        .expect("installed host call opens");
    drop(application);

    let restored = support::observed::install(Some(checkpoint), &provider_observation);
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let unrelated = request
        .query(PlanarRead {
            body_key: "anchor-isolated".to_owned(),
        })
        .execute()
        .expect("the unrelated source is readable");
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-isolated".to_owned(),
            replacement_y: length(51),
        })
        .expect_source(unrelated.observed_sources()[0].clone())
        .idempotency(&89_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the unrelated edit settles before adoption");

    provider_observation.reset();
    let first_demand = settle(&request, &restored);
    assert_eq!(first_demand.work().producer_contact_count(), 0);
    assert_eq!(provider_observation.count(), 0);
}

#[test]
fn checkpoint_reopens_sibling_parameter_partitions_without_contact_or_panic() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let application = support::observed::install(None, &provider_observation);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle_for(&request, &application, "anchor-a"));
    drop(settle_for(&request, &application, "anchor-b"));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("both parameter partitions are checkpointable")
        })
        .expect("installed host call opens");
    drop(application);

    provider_observation.reset();
    let restored = support::observed::install(Some(checkpoint), &provider_observation);
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let first = settle_for(&request, &restored, "anchor-a");
    assert_eq!(first.work().producer_contact_count(), 0);
    drop(first);
    let sibling = settle_for(&request, &restored, "anchor-b");
    assert_eq!(sibling.work().producer_contact_count(), 0);
    assert_eq!(provider_observation.count(), 0);
}

fn settle<'application, 'principal, 'scope>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >,
    application: &'application application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        CheckpointProgram,
    >,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputSettlement<
    PlanarQuery,
> {
    settle_for(request, application, "anchor-a")
}

fn settle_for<'application, 'principal, 'scope>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >,
    application: &'application application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        CheckpointProgram,
    >,
    root: &str,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputSettlement<
    PlanarQuery,
> {
    let controls = WorthQueryOutputDemandControls::default();
    let mut output = request
        .advanced()
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            application,
            PlanarOutputDemand::new(root),
            controls,
        )
        .expect("the program output starts");
    // An output that never settles fails the recovery instead of hanging it.
    (0..256)
        .find_map(|_| {
            match output
                .advance(application, request)
                .expect("the program output advances")
            {
                WorthQueryApplicationProgramOutputProgress::Pending
                | WorthQueryApplicationProgramOutputProgress::Progressed => None,
                WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the program output settles within the bounded advances")
}

mod advancement_lifetime;

mod advancement_custody;

mod advancement_bootstrap;

mod advancement_discovered;
