pub use std::sync::atomic::AtomicUsize;
pub use std::sync::atomic::Ordering;
pub use std::sync::Arc;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_mutation_free_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;

pub struct ZeroInvariantCompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for ZeroInvariantCompletingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

pub fn install_zero_invariant_transport(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> Arc<ZeroInvariantCompletingTransport> {
    let transport = Arc::new(ZeroInvariantCompletingTransport(AtomicUsize::new(0)));
    world
        .application
        .install_external_effect_transport(transport.clone())
        .expect("zero-invariant fixture installs its external transport");
    transport
}

#[test]
fn missing_owner_candidate_is_denied_before_semantic_invariant_execution() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-bypass-relational",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.skip_next_invariant_owner_execution();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(22, 22),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("missing owner admission must deny before semantic execution");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        (*world.application.primary_provider_for_test())
            .published_application_commit_count_for_test(),
        0,
        "owner validation denial must precede commit and outbox publication"
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn relational_invariant_violation_denies_before_provider_commit() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-pass-relational-invariant",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.violate_next_relational_invariant();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(23, 23),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the installed Relational invariant violation must deny");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn zero_semantic_invariants_still_require_relational_candidate_validation() {
    let world = installed_authorization_world(true);
    let transport = install_zero_invariant_transport(&world);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_mutation_free_program(&phase, &world, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");

    world.faults.violate_next_relational_invariant();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(25, 25),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("zero semantic requirements must not bypass Relational validation");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        (*world.application.primary_provider_for_test())
            .published_application_commit_count_for_test(),
        0,
        "zero-requirement owner denial must precede commit and outbox publication"
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 0);
}

#[test]
fn zero_semantic_invariants_commit_after_owner_candidate_admission() {
    let world = installed_authorization_world(true);
    let transport = install_zero_invariant_transport(&world);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_mutation_free_program(&phase, &world, &principal, &account, &request)
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(26, 26),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("owner-admitted candidate with no semantic requirements must commit");
    };
    assert_eq!(
        receipt.changed_record_count(),
        2,
        "only provider idempotency and outbox records may change"
    );
    assert_eq!(receipt.emitted_effect_count(), 1);
    assert!(
        receipt.dispatch_outbox().is_some(),
        "the mutation-free external effect must co-commit its outbox"
    );
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
}

#[test]
fn owner_validated_application_touch_outside_installed_ceiling_is_denied() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let rejected = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "must-not-pass-installed-touch-ceiling",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.add_next_undeclared_application_touch();
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            rejected,
            idempotency(24, 24),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("an undeclared performed application touch must deny");
    };
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    let _still_open = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &live_scope(),
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[path = "aftermath_sealing_interruption/mod.rs"]
pub mod aftermath_sealing_interruption;
#[path = "causal_fact_survives_post_commit_snapshot_failure_via_relational/mod.rs"]
pub mod causal_fact_survives_post_commit_snapshot_failure_via_relational;
#[path = "idempotency_without_the_claimed_causal_fact_is_not_equivalent/mod.rs"]
pub mod idempotency_without_the_claimed_causal_fact_is_not_equivalent;
#[path = "preparation_rejection_is_denied_without_effect_or_idempotency_re/mod.rs"]
pub mod preparation_rejection_is_denied_without_effect_or_idempotency_re;
#[path = "pretransaction_commit_failure_is_proved_aborted_and_applies_noth/mod.rs"]
pub mod pretransaction_commit_failure_is_proved_aborted_and_applies_noth;
