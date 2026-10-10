//! Installed opener refusals are synchronous and leave the worker usable.
use super::super::prepared_request::PreparedCompute;
use super::super::wave_dispatch::{self, PreparedMember};
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    test_execution_authority as test_authority, test_policy,
};
use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
use crate::domain_computation::primary_graph::{tests::fixture, WorthQueryAdvancementDenial};
use std::{
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant},
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

#[test]
fn author_held_application_and_product_handles_receive_nested_opening() {
    let application = Arc::new(fixture::installed_authorization_world(false).application);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    for workers in [1, 2, 4] {
        let root = QueryRequestExecution::open(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                authority: test_authority(),
                policy: test_policy(NonZeroUsize::new(workers).unwrap(), 1 << 24),
            },
            &scope,
        );
        let members = (0..4)
            .map(|_| {
                let application = application.clone();
                PreparedMember {
                    requests: vec![PreparedCompute::new(
                        0,
                        0,
                        move |_| {
                            assert!(active(), "the dispatch owns the worker marker");
                            // A bounded number of calls is the watchdog: every call must
                            // return its refusal, and no recursive advance can start.
                            for _ in 0..2 {
                                assert_eq!(
                                    application.with_host_advancement(|_| ()),
                                    Err(WorthQueryAdvancementDenial::NestedOpening)
                                );
                                assert_eq!(
                                    application.product_runtime().with_advancement(|_| ()),
                                    Err(WorthQueryAdvancementDenial::NestedOpening)
                                );
                            }
                        },
                        |_| 0,
                    )],
                }
            })
            .collect();
        let (settled, _) = root
            .execution_request()
            .unwrap()
            .run(worth_execution::ExecutionWorkCeiling::new(u64::MAX), |_| {
                wave_dispatch::dispatch(&root, members)
            })
            .unwrap();
        assert_eq!(settled.len(), 4);
        assert!(
            settled.iter().all(Result::is_ok),
            "every wave member completes its refused host calls"
        );
        assert!(!active());
    }
}
