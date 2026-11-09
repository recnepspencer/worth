//! Public Host transition over the original nonempty directory and ordinary B reopen.
use super::*;
mod activation_siblings;
mod successor;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome, home_program_transition::*,
    WorthQueryHomeProgramTransitionDenial as Denial,
    WorthQueryHomeProgramTransitionOpenPhase as Refused,
    WorthQueryHomeProgramTransitionPlan as Plan, WorthQueryHomeProgramTransitionStep as Step,
};
type B = worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
    successor::SuccessorSchema,
    successor::ExtendedProgram,
>;
fn resources() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    }
}

#[test]
fn host_transition_requires_finite_resources_before_creating_a_home() {
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    for maximum_work_units in [None, Some(0)] {
        let location = fixture::location();
        assert!(!location.exists());
        let contacts = Arc::new(AtomicUsize::new(0));
        let before = fixture::request_with_initializer(
            location.clone(),
            build,
            contacts.clone(),
            |_, _, _| panic!("A must not initialize"),
        );
        let mut budget = resources();
        budget.maximum_work_units = maximum_work_units;
        let request = successor(location.clone(), build, contacts.clone()).with_program_transition(
            before,
            budget,
            |_, _| panic!("invalid resources cannot invoke the author"),
        );
        let refused = with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .err()
        .expect("finite positive migration resources are required");
        assert!(matches!(
            refused.phase(),
            Refused::Configuration(Denial::FiniteResourcesRequired)
        ));
        assert!(
            !location.exists(),
            "resource refusal precedes the first physical owner effect"
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 0);
    }
}
fn successor(
    path: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
) -> worth_query_host::facade::WorthQueryApplicationHomeRequest<
    'static,
    successor::SuccessorSchema,
    successor::ExtendedProgram,
> {
    successor::request(path, build, contacts)
}
fn body(id: EntityId, field: AspectFieldLocator, value: u64) -> Step {
    Step {
        batches: vec![
            WorkerIntentBatch::new("authored-balance").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: id,
                    fields: AspectFieldPatch::from_locator(field, AspectValue::UInt64(value)),
                }),
            )),
        ],
    }
}
fn balance(runtime: &B, scope: &WorthQueryRequestScope) -> u64 {
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    let result = runtime
        .request(&principal, scope)
        .query(successor::balance_query::AccountRead("account-001".into()))
        .execute()
        .unwrap();
    result.rows()[0]
}
fn set(runtime: &B, scope: &WorthQueryRequestScope, key: u64, value: u64) {
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    let result = runtime
        .request(&principal, scope)
        .advanced()
        .mutate(successor::native_mutation::SetAccountBalance {
            account: "account-001".into(),
            balance: value,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(result, WorthQueryApplicationMutationOutcome::Committed { result, .. } if result == value)
    );
}
#[test]
fn host_transition_plans_on_nonempty_a_then_serves_writes_closes_and_reopens_b() {
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let planned = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("A initializes: {cause:?}")),
        )
    })
    .unwrap();
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("A closes: {cause:?}"));
    let predecessor =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("A cannot initialize during transition")
        });
    let author_calls = planned.clone();
    let request = successor(location.clone(), build, contacts.clone()).with_program_transition(
        predecessor,
        resources(),
        move |source, target| {
            author_calls.fetch_add(1, Ordering::SeqCst);
            let rows = source.entities("Account", 10_000)?;
            let [row] = rows.records() else {
                panic!("the author reads the original seeded A row");
            };
            let before = source
                .planned_field("Account", "AccountIdentity", "Balance")
                .unwrap();
            let after = target
                .planned_field("Account", "AccountIdentity", "Balance")
                .unwrap();
            let mut target_body = body(row.entity_id, after.clone(), 9);
            target_body.batches.push(successor::body(target));
            Ok(Plan {
                target: 1,
                schema_transition: Some((successor::schema_change(source, target), None)),
                steps: vec![
                    body(row.entity_id, before, 8),
                    target_body,
                    body(row.entity_id, after, 10),
                ],
            })
        },
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|cause| panic!("actual Host transition: {cause:?}")),
        )
    })
    .unwrap();
    let (_, scope) = super::pending_support::scope();
    assert_eq!(balance(home.runtime(), &scope), 10);
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        0,
        "transition bodies do not invoke the installed supplier"
    );
    set(home.runtime(), &scope, 0xd1601, 11);
    assert_eq!(balance(home.runtime(), &scope), 11);
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("promoted B closes: {cause:?}"));
    let request = successor(location, build, contacts.clone());
    let home =
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(request.open_in_advancement(&phase).unwrap_or_else(|cause| {
                panic!("ordinary B reopens without A or author: {cause:?}")
            }))
        })
        .unwrap();
    assert_eq!(balance(home.runtime(), &scope), 11);
    set(home.runtime(), &scope, 0xd1602, 12);
    assert_eq!(balance(home.runtime(), &scope), 12);
    assert_eq!(planned.load(Ordering::SeqCst), 1);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    (*home)
        .close(&execution)
        .unwrap_or_else(|cause| panic!("ordinary B closes: {cause:?}"));
}

#[test]
fn fresh_and_current_home_skip_the_optional_transition_author() {
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    for _ in 0..2 {
        let before = fixture::request_with_initializer(
            location.clone(),
            build,
            contacts.clone(),
            |_, _, _| panic!("unused A initializer"),
        );
        let request = successor(location.clone(), build, contacts.clone()).with_program_transition(
            before,
            resources(),
            |_, _| panic!("no transition is required"),
        );
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(
                request
                    .open_in_advancement(&phase)
                    .unwrap_or_else(|cause| panic!("current/fresh B: {cause:?}")),
            )
        })
        .unwrap();
        (*home)
            .close(&execution)
            .unwrap_or_else(|cause| panic!("current/fresh B close: {cause:?}"));
    }
}

#[test]
fn host_transition_refuses_private_record_bodies_before_intent_and_preserves_a() {
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let before = fixture::request(
        location.clone(),
        build,
        Arc::new(AtomicUsize::new(0)),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(before.open_in_advancement(&phase).unwrap())
    })
    .unwrap();
    (*home).close(&execution).unwrap();
    let before =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("existing A")
        });
    let request = successor(location.clone(), build, contacts.clone()).with_program_transition(
        before,
        resources(),
        |source, _| {
            // The schema observer describes platform kinds but grants no private-row mutation.
            let snapshot = source.schema_registry().authority_snapshot();
            let kind = snapshot
                .entity_kinds
                .iter()
                .find(|registration| registration.kind_name == "worth-query-program-activation")
                .expect("actual A owns its private activation kind")
                .kind_id;
            Ok(Plan {
                target: 0,
                schema_transition: None,
                steps: vec![Step {
                    batches: vec![WorkerIntentBatch::new("private-activation-create").push(
                        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                            partition_id: PartitionId::main(),
                            kind_id: kind,
                            client_key: ClientKey::raw("unauthored-activation"),
                            fields: AspectFieldPatch::default(),
                        })),
                    )],
                }],
            })
        },
    );
    let refused = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .err()
    .expect("an unauthored identity must refuse");
    match refused.phase() {
        Refused::Transition {
            home,
            cause: Denial::AuthoredBody(_),
            ..
        } => {
            assert!(home.home_program_transition_intent().is_none());
            // Read-only inspection leaves the complete owning refusal intact.
        }
        other => panic!("typed authored body refusal: {other:?}"),
    }
    drop(refused); // Abandon the whole original opening; no closed receipt is claimed.
    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("A is still existing")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(request.open_in_advancement(&phase).unwrap())
    })
    .unwrap();
    let (_, scope) = super::pending_support::scope();
    assert_eq!(super::pending_support::balance(home.runtime(), &scope), 7);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    (*home).close(&execution).unwrap();
}

mod workflow_dispositions;
