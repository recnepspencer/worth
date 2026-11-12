//! Actual request controls stop before any existing Home service can escape.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_host::facade::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
    WorthQueryProviderSessionControlStopKind as Stop,
    WorthQueryProviderSessionDenialKind as Provider,
};

#[test]
fn original_cancellation_and_memory_exhaustion_retain_the_same_pre_service_home() {
    for cancel in [true, false] {
        let path = tempfile::tempdir().unwrap();
        let cancellation = worth_execution::CancellationSource::new();
        let lease = checkpoint_capture::bounded_lease(128 * 1024 * 1024, cancellation.token());
        let execution = ExecutionRequest::leased(&lease);
        fresh_closed(path.path(), execution);
        if !cancel {
            with_bootstrap_advancement_from_request(execution, |phase| {
                with_native(path.path(), &phase, |native| {
                    idempotency::row(
                        native,
                        &execution,
                        "actual-memory-stop-row",
                        &"03".repeat(32),
                        &idempotency::intent(),
                        31,
                    );
                });
            })
            .unwrap();
        }
        let cleanup_lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let cleanup = ExecutionRequest::leased(&cleanup_lease);
        let (home, identity) = with_bootstrap_advancement_from_request(execution, |phase| {
            let (query, mut home) = unshared(path.path(), &phase);
            let identity = home.home_identity();
            let held = if cancel {
                cancellation.cancel();
                None
            } else {
                Some(
                    lease
                        .reserve_memory(lease.available_memory_bytes().unwrap())
                        .unwrap(),
                )
            };
            let refused = finalize_native(query, &mut home, &phase)
                .err()
                .expect("the original request cannot admit service");
            if cancel {
                assert!(matches!(
                    refused.reason().kind(),
                    Kind::ExecutionControlStopped {
                        kind: Stop::Cancelled
                    }
                ));
            } else {
                assert!(
                    matches!(
                        refused.reason().kind(),
                        Kind::ExecutionDenied {
                            kind: Provider::ExecutionResource {
                                denial: Resource::MemoryLimit { .. },
                                ..
                            }
                        }
                    ),
                    "{:?}",
                    refused.reason()
                );
            }
            assert_eq!(home.home_identity(), identity);
            drop(refused);
            drop(held);
            (home, identity)
        })
        .unwrap();
        assert_eq!(home.home_identity(), identity);
        let competing = reopen_refusal(path.path(), cleanup);
        assert!(matches!(
            &competing,
            WorthQueryApplicationHomeOpenRefused::Physical { .. }
        ));
        assert!(competing.detail().contains("MutationOwnerContended"));
        competing
            .close(&cleanup)
            .unwrap_or_else(|refused| panic!("competing opening disposal: {refused:?}"));
        assert!(matches!(
            home.close(&cleanup).unwrap(),
            ApplicationHomeCloseOutcome::Closed { .. }
        ));
    }
}
