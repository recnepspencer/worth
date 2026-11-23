//! A Directory preview keeps its expected source through ordinary publication.
use super::*;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement,
    application_entry::WorthQueryApplicationMutationOutcome,
    application_installation::WorthQueryProgramApplicationRuntime,
    product::{
        WorthQueryExpectedProductBranch, WorthQueryExpectedProductBranchCreationContinuation,
        WorthQueryProductBranch, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchRecoveryDenial,
    },
    runtime::{
        DeterminismContract, ExecutionAllocationPolicy::SystemAllocation, ExecutionBudget,
        ExecutionPosture, ExecutionRequestPolicy,
    },
    WorthQueryApplicationHome,
};

type Schema = fixture::schema::HomeIdentitySchema;
type Runtime = WorthQueryProgramApplicationRuntime<Schema, fixture::HomeProgram>;

#[test]
fn directory_preview_refuses_a_stale_source_and_applies_a_fresh_candidate() {
    let home = open_home();
    let runtime = home.runtime();
    let graph = home.primary_graph();
    let root = graph.current_world();
    let mut selected = root;

    write(runtime, root, 1, 8);
    let preview = fork(runtime, root);
    write(runtime, preview.branch(), 2, 12);
    assert_eq!(read(runtime, root), 8);
    assert_eq!(read(runtime, preview.branch()), 12);

    write(runtime, root, 3, 9);
    let (_, scope) = super::pending_support::scope();
    let failure = graph
        .with_funded_application_completion(&scope, SystemAllocation, |phase| {
            preview.adopt(&mut selected, &phase)
        })
        .expect("admit stale adoption")
        .expect_err("the original plan moved");
    let (stale, cause) = failure.into_parts();
    assert!(cause.is_stale_head(), "{cause:?}");
    assert_eq!(selected, root);
    assert_eq!(read(runtime, selected), 9);
    assert_eq!(read(runtime, stale.branch()), 12);

    let fresh = fork(runtime, root);
    assert_eq!(read(runtime, fresh.branch()), 9);
    write(runtime, fresh.branch(), 4, 17);
    let candidate = fresh.branch();
    let (_, scope) = super::pending_support::scope();
    let previous = graph
        .with_funded_application_completion(&scope, SystemAllocation, |phase| {
            fresh.adopt(&mut selected, &phase)
        })
        .expect("admit fresh adoption")
        .expect("the original plan is current");
    assert_eq!(previous, root);
    assert_eq!(selected, candidate);
    assert_eq!(read(runtime, selected), 17);
    assert_eq!(read(runtime, previous), 9);
    drop(stale);
    close_home(home);
}

#[test]
#[ignore = "Store M3 C5 must thread the caller's execution into World branch retirement"]
fn discard_directory_preview_retires_its_component() {
    let home = open_home();
    let graph = home.primary_graph();
    let root = graph.current_world();
    let preview = fork(home.runtime(), root);
    let candidate = preview.branch();
    let (_, scope) = super::pending_support::scope();
    let receipt = graph
        .with_funded_application_completion(&scope, SystemAllocation, |phase| preview.close(&phase))
        .expect("admit preview close")
        .unwrap_or_else(|failure| panic!("close preview: {:?}", failure.into_parts().1));
    assert_eq!(receipt.product_branch(), candidate);
    assert_eq!(receipt.retired_component_count(), 1);
    assert_eq!(read(home.runtime(), root), 7);
    close_home(home);
}

fn policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 << 20), None),
    )
}

fn open_home() -> WorthQueryApplicationHome<Schema, fixture::HomeProgram> {
    let request = fixture::request(
        fixture::location(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    with_bootstrap_advancement(policy(), |phase| request.open_in_advancement(&phase))
        .expect("bootstrap admission")
        .expect("healthy Directory Home")
}

fn close_home(home: WorthQueryApplicationHome<Schema, fixture::HomeProgram>) {
    let closed = home
        .close_with_policy(policy())
        .expect("close the Directory Home");
    closed.reclaim();
}

fn fork(
    runtime: &Runtime,
    branch: WorthQueryProductBranch,
) -> WorthQueryExpectedProductBranch<'_, Schema> {
    use WorthQueryExpectedProductBranchCreationContinuation as Continued;
    let graph = runtime.runtime();
    let (_, scope) = super::pending_support::scope();
    let initial = graph
        .with_funded_application_completion(&scope, SystemAllocation, |phase| {
            let selected = graph
                .on_branch(branch)
                .select(&phase)
                .expect("select current root");
            graph
                .branches()
                .fork_expected(selected)
                .expect("select original expected source")
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .expect("admit fork");
    let mut recovery = match initial {
        Ok(candidate) => return candidate,
        Err(WorthQueryProductBranchCreateError::ProductUnpublished(original)) => original,
        Err(other) => panic!("start Directory preview: {other:?}"),
    };
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        let (_, scope) = super::pending_support::scope();
        let step = graph
            .with_funded_application_completion(&scope, SystemAllocation, |phase| {
                graph.branches().continue_expected(recovery, &phase)
            })
            .expect("admit continuation");
        match step {
            Ok(Continued::Performed(candidate)) => return candidate,
            Ok(_) => panic!("a healthy preview must publish its candidate"),
            Err(failure) if failure.denial() == WorthQueryProductBranchRecoveryDenial::Busy => {
                recovery = failure.into_recovery();
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(failure) => {
                let cause = failure.denial();
                panic!(
                    "continue Directory preview: {cause:?}, {:?}",
                    failure.into_recovery().inspect()
                );
            }
        }
    }
    panic!("Directory preview stayed pending: {:?}", recovery.inspect());
}

fn write(runtime: &Runtime, branch: WorthQueryProductBranch, key: u64, balance: u64) {
    let (_, scope) = super::pending_support::scope();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".into(),
            balance,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(runtime, SystemAllocation)
        .expect("execute ordinary branch edit");
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { result, .. } if result == balance),
        "{outcome:?}"
    );
}

fn read(runtime: &Runtime, branch: WorthQueryProductBranch) -> u64 {
    let (_, scope) = super::pending_support::scope();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let rows = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .query(fixture::balance_query::AccountRead("account-001".into()))
        .execute()
        .expect("read branch balance");
    let [balance] = rows.rows() else {
        panic!("one balance row for account-001");
    };
    *balance
}
