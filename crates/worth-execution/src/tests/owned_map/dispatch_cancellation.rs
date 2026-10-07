use super::{
    authority, map, observe, request, CancellationSource, MapKernelFailure, MapKernelStop,
    MapOutcome, MapStop, MeasuredValue, Overlap, TEST_LOCK,
};
use crate::backend::BackendKind;
use worth_foundational::PartitionIdentity;

#[test]
fn cancellation_during_dispatch_matches_modes_and_taking_forms() {
    let _lock = TEST_LOCK.lock().unwrap();
    for workers in [1, 4] {
        for taking in [false, true] {
            let mut oracle = None;
            for owned in [false, true] {
                let source = CancellationSource::new();
                let mut req = request(workers, 2_000, 100);
                req.cancellation = source.token();
                let lease = authority().request_lease(req).unwrap();
                let overlap = Overlap::new(workers);
                let hold = taking.then(|| lease.reserve_memory(32).unwrap());
                let kernel = |value: u64, ctx: &mut crate::MapKernelContext<'_, '_>| {
                    assert!(!source.is_cancelled());
                    overlap.wait();
                    source.cancel();
                    overlap.wait();
                    ctx.checkpoint(1)?;
                    Ok::<_, MapKernelFailure<u64>>(MeasuredValue {
                        value,
                        payload: Vec::new(),
                    })
                };
                let result: MapOutcome<MeasuredValue, u64> = if owned {
                    map(0..4).run_owned_with_backend_taking(
                        Some(&lease),
                        BackendKind::Native,
                        hold,
                        kernel,
                    )
                } else {
                    map(0..4).run_with_backend_taking(
                        Some(&lease),
                        BackendKind::Native,
                        hold,
                        |v, ctx| kernel(*v, ctx),
                    )
                };
                let observed = observe(result);
                assert_eq!(observed.1, Some(PartitionIdentity::new(1)));
                assert_eq!(
                    observed.2,
                    Some(MapStop::Failure {
                        identity: PartitionIdentity::new(1),
                        cause: MapKernelFailure::Stop(MapKernelStop::Cancelled),
                    })
                );
                assert_eq!(observed.3.charged_work(), 0);
                if let Some(ref expected) = oracle {
                    assert_eq!(expected, &observed);
                } else {
                    oracle = Some(observed);
                }
            }
        }
    }
}
