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
mod support;
mod demand_contact;
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
        vec![
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .conditional_operation::<MutatePlanar>()
                .finish(),
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .conditional_operation::<PublishFinalPlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<CheckpointSchema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
        ]
    }
}

#[test]
fn checkpoint_reopens_ready_output_without_producer_contact_and_recomputes_after_source_change() {
    let _guard = checkpoint_recovery_test_guard();
    super::producer::reset_provider_contacts();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let settled = settle(&request, &application);
    assert!(settled.work().producer_contact_count() > 0);
    assert!(super::producer::provider_contacts() > 0);
    drop(settled);
    drop(request);
    drop(principal);
    drop(scope);

    let checkpoint = application
        .capture_application_checkpoint()
        .expect("the ready output is checkpointable");
    drop(application);

    super::producer::reset_provider_contacts();
    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let restored_settlement = settle(&request, &restored);
    assert_eq!(restored_settlement.work().producer_contact_count(), 0);
    assert_eq!(restored_settlement.work().delivery_contact_count(), 0);
    assert_eq!(restored_settlement.work().derived_publication_count(), 0);
    assert_eq!(super::producer::provider_contacts(), 0);
    let retained = request
        .retain_read()
        .expect("the recovered root is observable");
    request
        .at(&retained)
        .require_current_program_output(&restored_settlement, NonZeroUsize::new(4_096).unwrap())
        .expect("the restored settlement is current at its recovered source");

    let repeated_restored_settlement = settle(&request, &restored);
    assert_eq!(
        repeated_restored_settlement.work().producer_contact_count(),
        0
    );
    assert_eq!(super::producer::provider_contacts(), 0);
    drop(repeated_restored_settlement);

    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the recovered source is readable");
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&77_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(&restored)
        .expect("the recovered source accepts a fresh edit");
    let current = request
        .retain_read()
        .expect("the edited root is observable");
    assert!(matches!(
        request.at(&current).require_current_program_output(
            &restored_settlement,
            NonZeroUsize::new(4_096).unwrap(),
        ),
        Err(WorthQueryProgramOutputCurrentnessDenial::Output(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    drop(restored_settlement);

    super::producer::reset_provider_contacts();
    let recomputed = settle(&request, &restored);
    assert!(recomputed.work().producer_contact_count() > 0);
    assert!(super::producer::provider_contacts() > 0);
}

#[test]
fn recovered_output_survives_an_unrelated_settled_edit_without_producer_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(request);
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint()
        .expect("the ready output is checkpointable");
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
        .execute_performed::<CheckpointProgram, CheckpointRoot>(&restored)
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
    drop(request);
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint()
        .expect("the ready output is checkpointable");
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
        .execute_performed::<CheckpointProgram, CheckpointRoot>(&restored)
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
    drop(request);
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint()
        .expect("both parameter partitions are checkpointable");
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
    );
    let mut output = request
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            application,
            PlanarOutputDemand::new(root),
            controls,
        )
        .expect("the program output starts");
    loop {
        match output
            .advance(request)
            .expect("the program output advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => return settled,
        }
    }
}
