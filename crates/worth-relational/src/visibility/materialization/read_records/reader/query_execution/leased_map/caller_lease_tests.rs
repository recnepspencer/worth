use std::num::NonZeroUsize;

use super::*;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn relational_packet_dispatch_obeys_its_callers_lease() {
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), None, Some(0)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let outcome = execute_leased_query_packets(
        vec![PacketizedQueryWork::ExplicitTargets(vec![])],
        &worth_execution::ExecutionRequest::leased(&lease),
        Some(&lease),
        |_, _, _, context| {
            context.checkpoint(1)?;
            Err(MapKernelFailure::Domain(
                QueryReadPacketDenial::FragmentUnavailable,
            ))
        },
    );
    assert!(
        matches!(
            outcome,
            Err(QueryReadExecutionStop::PacketStopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            })
        ),
        "Relational packet dispatch must obey its caller's lease: {outcome:?}"
    );
}

#[test]
fn uncapped_query_packet_keeps_actual_claims_and_overflow_as_native_cause() {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    let lease = crate::tests::support::test_execution_lease();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    assert_eq!(execution.memory_limit(), None);
    let outcome = execute_leased_query_packets(
        vec![PacketizedQueryWork::ExplicitTargets(vec![])],
        &execution,
        Some(&lease),
        |_, _, memory, context| {
            let mut budget = QueryPacketBudget::new(context, memory)?;
            budget.claim_scratch(1024)?;
            assert!(lease.charged_memory_bytes() >= 1024);
            budget.claim_scratch(u64::MAX)?;
            unreachable!("the real claim counter cannot wrap")
        },
    )
    .unwrap_err();
    assert!(matches!(
        outcome,
        QueryReadExecutionStop::PacketStopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Domain(QueryReadPacketDenial::ReadRequest(
                    Cause::ChargedBytesOverflow
                )),
                ..
            },
            ..
        }
    ));
    assert_eq!(lease.charged_memory_bytes(), 0);
}
