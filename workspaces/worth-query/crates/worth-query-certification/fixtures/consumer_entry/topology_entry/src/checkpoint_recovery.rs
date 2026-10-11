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
        WorthQueryOutputDemandControls, WorthQueryProgramOutputCurrentnessDenial,
    },
    application_installation::{self, WorthQueryInMemoryApplicationLimits},
    primary_graph::{self, WorthQueryOutputDemandDenialKind},
};

use super::*;
#[cfg(feature = "test-query-execution-observer")]
mod advancement_currentness;
mod cached_locator_checkpoint;
#[cfg(feature = "test-query-execution-observer")]
mod clean_reuse;
pub(crate) mod computation_partition;
mod current_family_output;
mod current_output;
mod demand_contact;
mod demand_policy;
mod generated_restoration;
mod input_cutoff;
#[cfg(all(feature = "test-world-operation-control", not(target_arch = "wasm32")))]
mod late_cancellation;
pub(crate) mod mixed_retirement;
#[cfg(feature = "test-query-execution-observer")]
mod parallel_history;
mod producer_domain_denial;
pub(crate) mod required_chain;
mod reuse_opt_out;
mod stable_refresh;
mod support;
use support::{authenticate, install, length};

fn checkpoint_recovery_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static EXCLUSIVE_PROVIDER_COUNTER: std::sync::Mutex<()> = std::sync::Mutex::new(());
    EXCLUSIVE_PROVIDER_COUNTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

worth_query_application! {
    CheckpointSchema {
        owner: "worth.query.certification.checkpoint",
        version: (1, 0),
        contributions: [TopologyContribution],
    }
}

impl TopologySchemaBinding for CheckpointSchema {}

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
    let application = install(None);
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

    let restored = install(Some(checkpoint));
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

    super::producer::reset_provider_contacts();
    let redemanded = settle(&request, &restored);
    assert_eq!(redemanded.work().producer_contact_count(), 0);
    assert_eq!(super::producer::provider_contacts(), 0);
}

#[test]
fn unadopted_recovered_output_survives_an_unrelated_edit_before_first_demand() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
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

    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let unrelated = request
        .query(PlanarRead {
            body_key: "anchor-isolated".to_owned(),
        })
        .execute()
        .expect("the unrelated source is readable");
    request
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

    super::producer::reset_provider_contacts();
    let first_demand = settle(&request, &restored);
    assert_eq!(first_demand.work().producer_contact_count(), 0);
    assert_eq!(super::producer::provider_contacts(), 0);
}

#[test]
fn checkpoint_reopens_sibling_parameter_partitions_without_contact_or_panic() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
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

    super::producer::reset_provider_contacts();
    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let first = settle_for(&request, &restored, "anchor-a");
    assert_eq!(first.work().producer_contact_count(), 0);
    drop(first);
    let sibling = settle_for(&request, &restored, "anchor-b");
    assert_eq!(sibling.work().producer_contact_count(), 0);
    assert_eq!(super::producer::provider_contacts(), 0);
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
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
    // These recovery tests constrain the producer's persisted estimate. Source
    // verification includes fresh authorization and native witness preparation
    // and uses this installation's separate finite currentness allowance.
    .source_currentness_work(
        NonZeroUsize::new(
            application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap(),
    );
    let mut output = request
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
                WorthQueryApplicationProgramOutputProgress::Pending => None,
                WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the program output settles within the bounded advances")
}

#[cfg(feature = "test-query-execution-observer")]
mod advancement_lifetime;

#[cfg(feature = "test-query-execution-observer")]
mod advancement_custody;

#[cfg(feature = "test-query-execution-observer")]
mod advancement_bootstrap;

#[cfg(feature = "test-query-execution-observer")]
mod advancement_discovered;
