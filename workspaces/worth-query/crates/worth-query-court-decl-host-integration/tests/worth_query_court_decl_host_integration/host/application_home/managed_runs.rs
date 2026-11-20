//! Actual pre-World managed work abandons before the handoff and remains Failed cold.
use super::*;
use fixture::{native_mutation::SetBalance, schema::*};
use worth_query_decl::facade::application_schema::{fragment::FieldSelection, TypedMutationPreconditions};
use worth_query_execution::facade::{domain_computation::{WorthQueryApplicationIdempotencyBinding, WorthQueryManagedRunTerminalKind as Terminal, WorthQueryManagedRunCleanupDisposition as Cleanup}, primary_graph::WorthQueryPrincipalResolutionMode};
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::{
    entry_custody::preparation::prepare_application_commit,
    running::start_managed_application_commit,
    WorthQueryApplicationCommitPreparation, WorthQueryApplicationCommitPreparationRequest,
};
use worth_query_host::facade::primary_graph::{QueryManagedRunPosture, WorthQueryHomeRetirementDenial};

#[test]
fn abandoned_managed_direct_work_preserves_failed_terminal_after_directory_reopen() {
    let location = fixture::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let initialized = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        fixture::request(
            location.clone(),
            build,
            initialized.clone(),
            contacts.clone(),
        )
        .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("managed Home: {cause:?}"));
    let (_, scope) = pending_support::scope();
    // Use this Home's actual installed operation, principal, selected product,
    // invariant projection, resource admission and lower managed owners. Stop
    // before World admits an attempt: no future original may be fabricated.
    let running = {
        let app = home.runtime().runtime();
        let external =
            identity::inspection_authentication::authenticate(app.installed_schema(), &scope);
        app.with_host_advancement(|phase| {
            let selected = app.on_branch(app.current_world()).select(&phase).unwrap();
            let binding = app
                .installed_schema()
                .principal_binding(IdentityBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    FieldSelection::<HomeRecords, Account, AccountIdentity, String>::at(
                        "AccountNumber",
                    ),
                    "account-001".to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = app
                .installed_schema()
                .installed_operation(SetBalance::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &scope,
                )
                .unwrap();
            let idempotency = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
                HomeIdentitySchema,
                SetBalance,
                _,
                _,
            >(&"abandoned-managed-work", &29_u64)
            .unwrap()
            .bind_admission_for_test(&admission);
            let (_, projection, _) = app
                .mutation_projection_for_test()
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, projected| {
                        reader
                            .require_decision_field(
                                projected,
                                FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at(
                                    "Balance",
                                ),
                            )
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let mut effects = app
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let target = effects.existing_entity(&account).unwrap();
            effects
                .write_field(
                    &target,
                    FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance"),
                    29,
                )
                .unwrap();
            let program = effects.finish().unwrap();
            let entry = app
                .validate_commit_program_in_advancement_for_test(&phase, program)
                .unwrap_or_else(|cause| panic!("real program: {cause:?}"));
            let prepared = prepare_application_commit(
                &phase.borrowed_request_for_test(),
                WorthQueryApplicationCommitPreparationRequest::new_for_test(
                    entry,
                    idempotency,
                    None,
                    None,
                ),
            );
            let WorthQueryApplicationCommitPreparation::Ready(prepared) = prepared else {
                panic!("real managed preparation");
            };
            start_managed_application_commit(&phase, app, prepared)
                .unwrap_or_else(|cause| panic!("real managed start: {cause:?}"))
                .running_into_for_test()
        })
        .unwrap()
    };
    let attempt = running.identity().to_owned();
    let logical = running.logical_run_identity().to_owned();
    let retained = home.runtime().runtime().managed_run_obligations();
    let [issued] = retained.as_slice() else {
        panic!("one original managed work");
    };
    let identity = issued.identity;
    assert_eq!(issued.posture, QueryManagedRunPosture::Active);
    assert!(issued.original.is_none());
    let refused = home
        .close(&execution)
        .err()
        .expect("active work cannot disappear at close");
    assert!(matches!(
        refused.query_denial(),
        Some(WorthQueryHomeRetirementDenial::ManagedCleanup(worth_query_execution::facade::query_obligations::QueryObligationDenial::ManagedRunPending { identity: refused_identity })) if *refused_identity == identity
    ));
    let home = refused
        .resume()
        .unwrap_or_else(|cause| panic!("same live managed owner: {cause:?}"));
    let receipt = running
        .abandon()
        .cleanup()
        .expect("actual owner-thread abandonment cleanup");
    assert_eq!(receipt.inspection().terminal(), Terminal::Failed);
    assert!(receipt.inspection().resources_released());
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("released managed work: {cause:?}"))
        .reclaim();

    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("initializer replayed")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("fresh managed owner: {cause:?}"));
    let rows = home.runtime().runtime().managed_run_obligations();
    let [cold] = rows.as_slice() else {
        panic!("same original managed work");
    };
    assert_eq!(cold.identity, identity);
    assert_eq!(cold.logical_run, logical);
    assert_eq!(cold.original_attempt, attempt);
    assert_eq!(cold.attempt, attempt);
    assert!(cold.original.is_none());
    assert!(
        matches!(cold.posture, QueryManagedRunPosture::DirectCleanup {
        terminal: Terminal::Failed, disposition: Cleanup::CleanupComplete, resources,
    } if resources.released())
    );
    assert_eq!(pending_support::balance(home.runtime(), &scope), 7);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| {
            home.runtime()
                .runtime()
                .discharge_managed_run(&phase, identity)
        })
        .unwrap()
        .unwrap();
    assert!(home
        .runtime()
        .runtime()
        .managed_run_obligations()
        .is_empty());
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("discharged managed work: {cause:?}"))
        .reclaim();
}
