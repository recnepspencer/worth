//! Fresh Query completion through actual Directory Home owners and sealed ports.
use fixture::{schema::*, HomeProgram};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::application_home_integration::*;
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::*,
    declaration::application_program::*,
    primary_graph::*,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};
use worth_store_application::*;
mod inspection;
pub(super) type DirectoryHome = WorldReadmittedApplicationHome<(), (), (), (), ()>;
pub(super) type Query = PreparedHomeQueryRuntime<HomeIdentitySchema, HomeProgram>;

pub(super) fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap()
}
fn limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        identity::world_resources(),
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::default(),
        SignalConditionalEvaluationBudget::development(),
    )
}

fn seed_batches(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<HomeIdentitySchema>,
    schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        HomeIdentitySchema,
    >,
    flush: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    for ordinal in 1..=2 {
        graph.bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new(format!("host-principal-{ordinal}")).unwrap(),
            ordinal,
            worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                if ordinal == 1 { "subject" } else { "subject-2" },
            )
            .unwrap(),
            worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )?;
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Account::reference(),
                WorthQueryApplicationEntityKey::new(format!("account-row-{ordinal}")).unwrap(),
            )
            .field(AccountNumber::reference(), format!("account-00{ordinal}"))
            .field(fixture::native_mutation::Balance::reference(), 7),
        )?;
        if ordinal == 1 || flush {
            graph.commit_seed_batch(phase)?;
        }
    }
    Ok(())
}

pub(super) fn prepared(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    flush: bool,
    callbacks: &AtomicUsize,
) -> (
    Query,
    DirectoryHome,
    worth_foundational::facade::ApplicationHomeBinding,
) {
    let declaration = HomeIdentitySchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<HomeIdentitySchema, HomeProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let query = prepare_home_program(
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (Arc::new(AtomicUsize::new(0)),),
        build(),
        limits(),
        None,
        |_, _| panic!("the Relational-only fixture has no authored component seed"),
    )
    .unwrap();
    let expected = query
        .admitted_home_binding()
        .unwrap()
        .consume(|owner| owner.into_description());
    let request = phase.execution_request();
    let physical = ApplicationHomeRequest::new(
        path.to_path_buf(),
        query.home_relational_definition(),
        query.admitted_home_binding().unwrap(),
        build(),
    );
    let home = admit_home(recover_home(physical, &request).unwrap(), &request).unwrap();
    let mut home =
        attach_relational(home, &request).unwrap_or_else(|_| panic!("actual Native attaches"));
    let preparation = home.prepare_native_schema_for_home(&request).unwrap();
    let query = match preparation {
        ApplicationHomeNativeSchemaPreparation::Fresh {
            native,
            initialization,
        } => query.initialize_home_native_schema(phase, native, initialization),
        ApplicationHomeNativeSchemaPreparation::Existing { .. } => {
            panic!("fresh Directory fixture selected an existing source")
        }
    }
    .unwrap_or_else(|refused| panic!("actual schema: {:?}", refused.reason()));
    let home = home
        .readmit(query.home_initial_schema_receipt(), &request)
        .unwrap_or_else(|_| panic!("actual Native readmission"))
        .consume(|home| home);
    let (query, home) = home
        .initialize_new_home(&request, |mut home| {
            let (native, admission) = home.fresh_query_finalization_parts().unwrap();
            let query = query
                .finish_fresh_home_native_readmission(phase, native, admission)
                .unwrap_or_else(|refused| panic!("fresh finalization: {:?}", refused.reason()));
            let bridge = query.home_bridge_definition().unwrap();
            let (budgets, clock, policy, execution, conditionals) = query.home_world_resources();
            let definition = ApplicationHomeWorldDefinition::new(
                bridge,
                "primary-application-source",
                budgets,
                clock,
                policy,
                execution,
                conditionals,
            );
            let home = readmit_query_world(home.share(), definition, &request)
                .unwrap_or_else(|refused| panic!("actual World: {:?}", refused.reason()))
                .consume(|home| home);
            let (source, bridge, conditional) = home.query_composition_parts().unwrap();
            let mut query = query
                .attach_home_registered_source(phase, source, bridge, conditional)
                .unwrap_or_else(|refused| panic!("actual source: {:?}", refused.reason()));
            query
                .initialize_home_query_state(phase, |phase, graph, schema| {
                    callbacks.fetch_add(1, Ordering::SeqCst);
                    seed_batches(phase, graph, schema, flush)
                })
                .unwrap();
            (query, home)
        })
        .unwrap();
    (query, home, expected)
}

#[test]
fn directory_completion_covers_flushed_and_pending_batches_before_world_genesis() {
    for flush in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = ExecutionRequest::leased(&lease);
        let callbacks = AtomicUsize::new(0);
        with_bootstrap_advancement_from_request(execution, |phase| {
            let (mut query, mut home, expected) =
                prepared(directory.path(), &phase, flush, &callbacks);
            query.publish_home_query_seed(&phase).unwrap();
            let count = home
                .relational()
                .runtime()
                .history()
                .immutable_commit_count();
            query.publish_home_query_seed(&phase).unwrap();
            assert_eq!(
                home.relational()
                    .runtime()
                    .history()
                    .immutable_commit_count(),
                count,
                "cached success cannot publish a second completion"
            );
            let root = home.select_query_root(&execution).unwrap();
            inspection::completed(home.relational().runtime(), &root, &expected, &execution);
            assert_eq!(
                count, 4,
                "one activation, two genuine seed commits, one completion"
            );
            assert_eq!(callbacks.load(Ordering::SeqCst), 1);
            drop(root);
            drop(query);
            let close = home
                .close(&execution)
                .unwrap_or_else(|refused| panic!("actual World closes: {:?}", refused.reason()));
            assert!(matches!(close, ApplicationHomeCloseOutcome::Closed { .. }));
        })
        .unwrap();
    }
}

#[test]
fn failed_completion_retains_seed_history_and_same_home_without_retry_or_genesis() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let callbacks = AtomicUsize::new(0);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (mut query, home, _) = prepared(directory.path(), &phase, true, &callbacks);
        let identity = home.home_identity();
        let physical = home.physical_serving_for_test().runtime_identity();
        home.physical_serving_for_test().certification_reject_next_candidate_retention_before_effect();
        let cause = query.publish_home_query_seed(&phase).unwrap_err();
        assert_eq!(cause.kind(), WorthQueryPrimaryGraphInstallationDenialKind::RelationalDurabilityUnresolved, "{cause:?}");
        assert_eq!(cause.subject(), "Relational rejected the application seed transaction: durable publication of canonical commit 4 at position 4 is unresolved: Indeterminate { stage: DataDispatch }");
        {
            let journal = home.physical_serving_for_test().canonical_commits().unwrap();
            assert_eq!(journal.head().unwrap().unwrap().position(), 3);
            // Beyond the inspected durable head, resolve consults unresolved
            // position custody before any digest comparison.
            assert_eq!(journal.resolve(4, [0; 32]).unwrap(),
                worth_store::physical_runtime::OrderedJournalResolution::Unresolved);
        }
        assert_eq!(home.relational().runtime().history().immutable_commit_count(), 3);
        assert!(home.world().observation_port().observe_root_product_branch().is_err());
        let again = query.publish_home_query_seed(&phase).unwrap_err();
        assert_eq!(again.subject(), "a previous installation seed batch failed");
        assert_eq!(home.relational().runtime().history().immutable_commit_count(), 3);
        assert_eq!(callbacks.load(Ordering::SeqCst), 1);
        assert_eq!(home.home_identity(), identity);
        assert_eq!(home.physical_serving_for_test().runtime_identity(), physical);
        assert!(matches!(recover_home(ApplicationHomeRequest::new(directory.path().to_path_buf(),
            // A genuine independently prepared request must meet the SAME held lease.
            meaning().home_relational_definition(), meaning().admitted_home_binding().unwrap(), build()), &execution),
            Err(ApplicationHomeOpenRefusal::Physical(refused)) if refused.kind() == ApplicationHomePhysicalRefusalKind::MutationOwnerContended));
        drop(query);
        let sealed = home.seal_world(&execution).unwrap_or_else(|refused| panic!("World seal: {:?}", refused.reason()));
        let close = sealed.finish_close(&execution);
        inspection::failed_close(close, identity);
    }).unwrap();
}

pub(super) fn meaning() -> PreparedHomeApplication<HomeIdentitySchema, HomeProgram> {
    let declaration = HomeIdentitySchema::declaration().unwrap();
    prepare_home_program(
        ApplicationProgramAuthoring::<HomeIdentitySchema, HomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (Arc::new(AtomicUsize::new(0)),),
        build(),
        limits(),
        None,
        |_, _| panic!("the Relational-only fixture has no authored component seed"),
    )
    .unwrap()
}

#[test]
fn memory_multibatch_bootstrap_keeps_its_original_three_commit_basis() {
    use worth_query_host::facade::{
        admission::authenticated_principal::{
            WorthQueryCancellationSource, WorthQueryRequestScope,
        },
        application_entry::WorthQueryApplicationRequestExt,
        WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    };
    let declaration = HomeIdentitySchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        ApplicationProgramAuthoring::<HomeIdentitySchema, HomeProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (Arc::new(AtomicUsize::new(0)),),
        build(),
        limits(),
        |phase, graph, schema| seed_batches(phase, graph, schema, true),
    );
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("Memory bootstrap: {refused:?}"));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    {
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let result = runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead("account-001".into()))
            .execute()
            .unwrap();
        assert_eq!(result.rows(), &[7]);
        assert_eq!(result.receipt().inspect().basis().version(), 3);
        assert_eq!(runtime.runtime().publication().principal_binding_count(), 2);
    }
    let refused = home
        .close(&execution)
        .err()
        .expect("Memory Home retains its existing lifecycle refusal");
    assert_eq!(
        refused.detail(),
        "memory Home World lifecycle has not been readmitted"
    );
    let home = refused
        .resume()
        .unwrap_or_else(|_| panic!("same actual Memory owner remains resumable"));
    assert!(!home.is_directory());
    // Explicit unbudgeted disposal proves no Memory Home close completion.
    drop(home);
}
