//! Installed opener refusals are synchronous and leave the worker usable.
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::computation::prepared_request::PreparedCompute;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::computation::wave_dispatch;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::PreparedMember;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::computation::worker_custody::active;
pub use crate::domain_computation::primary_graph::application_contribution::computation::worker_custody::tests;
pub use worth_query_execution::facade::domain_computation::query_test_execution_authority_for_test as test_authority;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy;
pub use worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture;
pub use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
pub use std::num::NonZeroUsize;
pub use std::sync::Arc;
pub use std::time::Duration;
pub use std::time::Instant;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
pub use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn author_held_application_and_product_handles_receive_nested_opening() {
    let application = Arc::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(false).application);
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    for workers in [0, 1, 2, 4] {
        let root = QueryRequestExecution::open(
            if workers == 0 {
                worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
                    NonZeroUsize::MIN,
                    None,
                ))
            } else {
                worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy: test_policy(NonZeroUsize::new(workers).unwrap(), None),
                }
            },
            &scope,
        );
        let members = (0..4)
            .map(|_| {
                let application = application.clone();
                PreparedMember {
                    requests: vec![PreparedCompute::new(move |_| {
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
                    })],
                }
            })
            .collect();
        let (settled, _) = root
            .execution_request()
            .unwrap()
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                worth_query_execution::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::dispatch(&root, members)
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

#[path = "nested_worker/mod.rs"]
pub mod nested_worker;
