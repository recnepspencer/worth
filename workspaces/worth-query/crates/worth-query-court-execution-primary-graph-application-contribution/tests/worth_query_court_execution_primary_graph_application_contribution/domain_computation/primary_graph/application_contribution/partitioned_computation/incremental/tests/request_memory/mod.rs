//! Real request policy can refuse routing storage before gathering.

use std::num::NonZeroUsize;

use worth_foundational::ExecutionRequestPolicy;

use worth_query_execution::facade::domain_computation::query_test_execution_authority_for_test;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;

/// A two-worker policy of `memory`. Its lease is drawn from the test
/// process's one authority.
pub fn policy(memory: u64) -> ExecutionRequestPolicy {
    test_policy(NonZeroUsize::new(2).unwrap(), Some(memory))
}

/// A full run under `memory`, leased, then serial.
pub fn under(memory: u64) -> [Attempt; 2] {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), sum);
    [false, true].map(|serial| {
        let placement = |memory| {
            if serial {
                RuntimeWorldExecutionPlacement::Serial(policy(memory))
            } else {
                RuntimeWorldExecutionPlacement::Leased {
                    authority: query_test_execution_authority_for_test(),
                    policy: policy(memory),
                }
            }
        };
        // The existing reservation denial observes remaining room without holding
        // bytes, as in tree_admission. Run the same fixture with ample room first.
        let unconstrained = 1 << 30;
        let mut held = None;
        let baseline = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::projected_attempt::attempt_admitted_placed_observing(
            &world,
            &installed,
            None,
            &live_scope(),
            placement(unconstrained),
            |execution| {
                let Err(Resource::MemoryLimit {
                    admitted,
                    level: OWN_POLICY,
                    ..
                }) = execution.reserve(u64::MAX)
                else {
                    panic!("the finite request policy reports its remaining room");
                };
                held = Some(unconstrained - admitted);
            },
        );
        assert!(baseline.outcome.is_ok(), "unconstrained fixture completes");
        let held = held.expect("observed before the first computation charge");
        eprintln!("projection holds {held} request bytes before computation (serial={serial})");
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::projected_attempt::attempt_admitted_placed_observing(
            &world,
            &installed,
            None,
            &live_scope(),
            placement(held + memory),
            |_| {},
        )
    })
}

/// Every refusal here is the request's policy, leased or serial.
pub const OWN_POLICY: WorthQueryMemoryLimitLevel = WorthQueryMemoryLimitLevel::Policy;

/// Exhaust the real request's available room before native preparation.
#[test]
fn exhausted_request_refuses_native_preparation_before_any_gather() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    for run in under(0) {
        assert!(matches!(run.outcome,
            Err(WorthQueryPartitionedComputationDenial::Resource(Resource::MemoryLimit {
                requested, admitted: 0, level: OWN_POLICY,
            })) if requested > 0
        ));
        assert!(run.gathered.is_empty());
    }
}
