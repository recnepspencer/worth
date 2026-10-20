//! A prepare holds its routing, and each gather's declared bytes, on the
//! request's memory before it holds them. A budget one byte short refuses
//! before the owner gathers, and a leased request and a serial one with the
//! same budget refuse at the same boundary with the same denial.

use std::num::NonZeroUsize;

use worth_execution::KeyedPartitioner;
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

/// What the routing of items 1 to 4 under two parities retains.
fn routing_bytes() -> u64 {
    KeyedPartitioner::<[u8; 32]>::retained_bytes(4, 2, 0).unwrap()
}

/// What each gather holds before it runs: the computation's declared bytes.
fn declared_bytes() -> u64 {
    u64::try_from(Computation::RESOURCES.maximum_retained_bytes()).unwrap()
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

fn refused(requested: u64, admitted: u64) -> Outcome {
    Err(WorthQueryPartitionedComputationDenial::Resource(
        Resource::MemoryLimit {
            requested,
            admitted,
            level: OWN_POLICY,
        },
    ))
}

/// The last item's route would retain one byte more than the budget, so the
/// routing is refused and nothing is gathered.
#[test]
fn one_byte_short_of_the_routing_refuses_before_any_gather() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let routing = routing_bytes();
    for run in under(routing - 1) {
        assert_eq!(run.outcome, refused(routing, routing - 1));
        assert!(run.gathered.is_empty());
    }
}

/// With the routing held, one byte short of a gather's declared bytes is
/// refused before that gather. Exactly enough admits the first gather, and
/// the second's declared bytes, on top of the first's own, are refused
/// before the second, the same way leased and serial.
#[test]
fn one_byte_short_of_a_gathers_declared_bytes_refuses_before_that_gather() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let routing = routing_bytes();
    let declared = declared_bytes();
    for run in under(routing + declared - 1) {
        assert_eq!(run.outcome, refused(declared, declared - 1));
        assert!(run.gathered.is_empty());
    }

    let [leased, serial] = under(routing + declared);
    assert_eq!(leased.gathered, serial.gathered);
    assert_eq!(leased.gathered.len(), 1, "only the first gather fits");
    assert_eq!(leased.outcome, serial.outcome);
    assert!(matches!(
        leased.outcome,
        Err(WorthQueryPartitionedComputationDenial::Resource(
            Resource::MemoryLimit { requested, admitted, level: OWN_POLICY }
        )) if admitted == declared && requested > declared
    ));
}
