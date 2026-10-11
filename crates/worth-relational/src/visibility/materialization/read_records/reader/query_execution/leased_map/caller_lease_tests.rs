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
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 1 << 20, 0),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let outcome = execute_leased_query_packets(
        vec![PacketizedQueryWork::ExplicitTargets(vec![])],
        &lease,
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
