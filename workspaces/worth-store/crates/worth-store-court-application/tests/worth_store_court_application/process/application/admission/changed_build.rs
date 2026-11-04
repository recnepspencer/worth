//! A changed Host build must readmit its real filesystem Home without initialization.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryApplicationMutationOutcome as Outcome},
    runtime::ExecutionAllocationPolicy,
};
use worth_query_testkit_host::facade::host::application_runtime_surface::inspection_authentication::authenticate;

#[test]
fn changed_host_build_keeps_initializer_closed_and_reaches_native_readmission() {
    let directory = tempfile::tempdir().unwrap();
    let initialized = Arc::new(AtomicUsize::new(0));
    let original_build = build();
    let next_build =
        ApplicationBuildIdentity::parse("22222222-2222-4222-8222-222222222222").unwrap();
    assert_ne!(original_build, next_build);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
    );
    let set = |home: &Home| {
        let runtime = home.runtime();
        let principal = authenticate(runtime.runtime().installed_schema(), &scope);
        runtime
            .request(&principal, &scope)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 9,
            })
            .without_source()
            .idempotency(&0x6275696c64_u64)
            .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    let original = with_open(
        directory.path(),
        original_build,
        initialized.clone(),
        |opened, contacts| {
            let home = opened
                .as_ref()
                .unwrap_or_else(|cause| panic!("original build Host: {cause:?}"));
            assert!(home.is_directory());
            let original = match set(home) {
                Outcome::Committed { receipt, result: 9 } => receipt.commit_reference().clone(),
                other => panic!("original build must commit: {other:?}"),
            };
            assert_eq!(contacts.load(Ordering::SeqCst), 1);
            original
        },
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    // This is process-abandonment custody release, never a claim of Closed.
    let initialized = Arc::new(AtomicUsize::new(0));
    with_open(
        directory.path(),
        next_build,
        initialized.clone(),
        |opened, contacts| match opened {
            Ok(home) => {
                assert!(home.is_directory());
                assert_eq!(
                    contacts.load(Ordering::SeqCst),
                    0,
                    "build readmission cannot execute a supplier"
                );
                let runtime = home.runtime();
                let principal = authenticate(runtime.runtime().installed_schema(), &scope);
                let observed = runtime
                    .request(&principal, &scope)
                    .query(fixture::balance_query::AccountRead(
                        "account-001".to_owned(),
                    ))
                    .execute()
                    .unwrap();
                assert_eq!(observed.rows(), &[9]);
                match set(home) {
                    Outcome::RecoveredCommitted(receipt) => {
                        assert_eq!(receipt.commit_reference(), &original)
                    }
                    other => {
                        panic!("changed build resolves the original durable history: {other:?}")
                    }
                }
                assert_eq!(
                    contacts.load(Ordering::SeqCst),
                    0,
                    "durable retry cannot regenerate the original supplier"
                );
                println!("HOST_CHANGED_BUILD=Serving");
            }
            Err(cause) => panic!("build changes must pass descriptor admission: {cause:?}"),
        },
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 0);
}
