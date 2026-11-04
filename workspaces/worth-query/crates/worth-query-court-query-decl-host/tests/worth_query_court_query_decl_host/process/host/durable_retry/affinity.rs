//! A copied Native root does not transfer the original product's retry affinity.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial as Denial;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind;

#[test]
fn inherited_directory_key_refuses_a_sibling_product_affinity() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = pending_support::open(execution, contacts.clone());
    let (_, live) = scope();
    let original = set(home.runtime(), &live, 0x7375626c696e67, 8).unwrap();
    assert!(matches!(original, Outcome::Committed { result: 8, .. }));
    let application = home.runtime().runtime();
    let sibling = application
        .with_application_advancement(&live, |phase| {
            application
                .branches()
                .fork(application.current_world())
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
        })
        .unwrap()
        .expect("the actual directory World creates the exact sibling");
    let principal =
        identity::inspection_authentication::authenticate(application.installed_schema(), &live);
    let retry = home
        .runtime()
        .request(&principal, &live)
        .on_branch(sibling)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".to_owned(),
            balance: 8,
        })
        .without_source()
        .idempotency(&0x7375626c696e67_u64)
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    match retry {
        Err(Denial::Idempotency(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationIdempotencyResolutionDenialKind::ForeignAdmission
        ),
        other => panic!("the inherited key belongs to its original product: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    drop(original);
    drop(home);
}

#[test]
fn reopened_home_refuses_the_prior_open_branch_selector_before_effect() {
    use worth_query_host::facade::primary_graph::{
        WorthQueryProductBranchAdmissionDenial as BranchDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    };
    let contacts = Arc::new(AtomicUsize::new(0));
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        Arc::new(AtomicUsize::new(0)),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("stale-selector writer Home", &refused)),
        )
    })
    .unwrap();
    let stale = home.runtime().runtime().current_world();
    let (_, live) = scope();
    assert!(matches!(
        set(home.runtime(), &live, 0x7374616c65, 8).unwrap(),
        Outcome::Committed { .. }
    ));
    // Ordinary abandonment makes no D11 close claim. The opaque supplier's
    // retirement belongs to its owner; only the old selector survives here.
    drop(home);
    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("existing Home cannot initialize")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            request
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("stale-selector reader Home", &refused)),
        )
    })
    .unwrap();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &live,
    );
    let answer = home
        .runtime()
        .request(&principal, &live)
        .on_branch(stale)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".to_owned(),
            balance: 8,
        })
        .without_source()
        .idempotency(&0x7374616c65_u64)
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    match answer {
        Err(Denial::ProductSelection(denial)) => assert_eq!(
            denial,
            BranchDenial::Refused(WorthQueryProductBranchAdmissionRefusalKind::StaleIncarnation)
        ),
        other => panic!("the previous open's selector grants no current admission: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    assert!(
        matches!(
            set(home.runtime(), &live, 0x7374616c65, 8).unwrap(),
            Outcome::RecoveredCommitted(_)
        ),
        "fresh admission still resolves the original durable occurrence"
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
}

mod capability_home;
mod session_grant;
