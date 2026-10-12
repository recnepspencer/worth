//! Certification reservation refusals latch before any certification run.
use super::*;
use crate::backend::{run_scope, ScopeStop};
use crate::{
    DecomposeKernelEditions, ExecutionDecompose, ExecutionMap, InterfaceSolution, InteriorResult,
};

#[test]
fn nested_certification_reservation_cannot_be_taken() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 1, 100)).unwrap();
    let outer = run_scope(None, 0, 0, |parent| {
        let inner = run_scope(None, 0, 0, |_| {
            let mut decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
                vec![],
                (),
                |_, _| (),
                0,
                0,
                0,
                0,
                0,
            )
            .unwrap();
            let input =
                ExecutionMap::<(), u64>::from_keyless_partitions(Default::default()).unwrap();
            let result = decomposition.certify(
                &lease,
                1,
                &input,
                DecomposeKernelEditions {
                    interior: 1,
                    interface: 1,
                    back: 1,
                },
                |_, _| -> Result<InteriorResult<(), ()>, KernelFailure<()>> {
                    panic!("reservation must precede kernels")
                },
                |_, _| -> Result<InterfaceSolution<(), ()>, KernelFailure<()>> {
                    panic!("reservation must precede kernels")
                },
                |_, _| -> Result<(), KernelFailure<()>> {
                    panic!("reservation must precede kernels")
                },
            );
            assert!(matches!(
                result,
                Err(crate::DecomposeCertificationFailure::Mismatch(
                    crate::oracle::OracleMismatch::Stop
                ))
            ));
            Err::<(), _>(KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        outer.result,
        Err(ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
}
