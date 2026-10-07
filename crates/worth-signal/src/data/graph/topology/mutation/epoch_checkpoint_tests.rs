use super::*;
use crate::facade::SignalCheckpointDenial;
use crate::tests::leased_execution::support::{authority, cancellable};
use worth_execution::{ExecutionScan, MapKernelFailure};
use worth_foundational::PartitionIdentity;

#[test]
fn topology_segment_checkpoint_preserves_cancellation() {
    let (request, cancellation) = cancellable(1, 100_000);
    let lease = authority().request_lease(request).unwrap();
    let key = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![key], vec![(key, ())]).unwrap();
    let mut observed = None;
    let _outcome = scan.run(Some(&lease), (), 0, 0, 0, 0, |_, _, request| {
        cancellation.cancel();
        observed = Some(
            with_segment_work(Some(request), |work| {
                work.visit().map_err(SignalError::retained_storage_denied)
            })
            .unwrap_err(),
        );
        Ok::<_, MapKernelFailure<SignalError>>(((), ()))
    });
    assert_eq!(
        observed,
        Some(SignalError::ExecutionCheckpointStopped(
            SignalCheckpointDenial::Cancelled
        ))
    );
}
