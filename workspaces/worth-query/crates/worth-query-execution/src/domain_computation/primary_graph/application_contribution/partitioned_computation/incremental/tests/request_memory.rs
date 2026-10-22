//! Real request policy can refuse routing storage before gathering.

use std::num::NonZeroUsize;

use worth_foundational::ExecutionRequestPolicy;

use super::super::super::super::request_execution::{
    query_test_execution_authority_for_test, test_policy,
};
use super::super::super::super::WorthQueryManagedComputationResourceDenial as Resource;
use super::super::super::super::WorthQueryMemoryLimitLevel;
use super::*;

/// A two-worker policy of `memory`. Its lease is drawn from the test
/// process's one authority.
fn policy(memory: u64) -> ExecutionRequestPolicy {
    test_policy(NonZeroUsize::new(2).unwrap(), Some(memory))
}

/// A full run under `memory`, leased, then serial.
fn under(memory: u64) -> [Attempt; 2] {
    let world = installed_authorization_world(true);
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
        let baseline = projected_attempt::attempt_admitted_placed_observing(
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
        projected_attempt::attempt_admitted_placed_observing(
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
const OWN_POLICY: WorthQueryMemoryLimitLevel = WorthQueryMemoryLimitLevel::Policy;

/// Exhaust the real request's available room before native preparation.
#[test]
fn exhausted_request_refuses_native_preparation_before_any_gather() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for run in under(0) {
        assert!(matches!(run.outcome,
            Err(WorthQueryPartitionedComputationDenial::Resource(Resource::MemoryLimit {
                requested, admitted: 0, level: OWN_POLICY,
            })) if requested > 0
        ));
        assert!(run.gathered.is_empty());
    }
}
