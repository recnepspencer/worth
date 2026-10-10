//! An absent rerun identity is a structural denial, never a request overflow.
use super::*;
use crate::{
    DecomposeCertificationFailure, DecomposeFailure, DecomposeInputDenial, DecomposeKernelEditions,
    ExecutionDecompose, ExecutionMap, InterfaceSolution, InteriorResult, MapPartition,
    NestedComputationStop,
};

#[test]
fn a_swallowed_unknown_identity_in_decompose_run_does_not_stop_the_request() {
    unknown_rerun(false);
}

#[test]
fn a_swallowed_unknown_identity_in_decompose_certify_does_not_stop_the_request() {
    unknown_rerun(true);
}

fn unknown_rerun(certify: bool) {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let editions = DecomposeKernelEditions {
        interior: 1,
        interface: 1,
        back: 1,
    };
    let mut decomposition =
        ExecutionDecompose::<(), (), (), (), (), _>::try_new(vec![], (), |_, _| (), 0, 0, 0, 0, 0)
            .unwrap();
    let empty = ExecutionMap::<(), u64>::from_keyless_partitions(Default::default()).unwrap();
    decomposition
        .run(
            None,
            &empty,
            editions,
            |_, _| -> Result<InteriorResult<(), ()>, KernelFailure<()>> {
                panic!("empty interior")
            },
            |_, _| {
                Ok::<_, KernelFailure<()>>(InterfaceSolution {
                    solution: (),
                    slices: vec![],
                })
            },
            |_, _| -> Result<(), KernelFailure<()>> { panic!("empty back map") },
        )
        .unwrap();
    let unknown = PartitionIdentity::new(1);
    let changed = ExecutionMap::<(), u64>::try_from_declared_partitions(
        vec![unknown],
        vec![MapPartition {
            identity: unknown,
            value: (),
            read_keys: vec![],
            write_keys: vec![],
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .unwrap();
    let interior = |_: &(),
                    _: &mut crate::MapKernelContext<'_, '_>|
     -> Result<InteriorResult<(), ()>, KernelFailure<()>> {
        panic!("unknown identity must precede kernels")
    };
    let interface = |_: &(),
                     _: &mut crate::MapKernelContext<'_, '_>|
     -> Result<InterfaceSolution<(), ()>, KernelFailure<()>> {
        panic!("unknown identity must precede kernels")
    };
    let back = |_: &crate::BackInput<(), ()>,
                _: &mut crate::MapKernelContext<'_, '_>|
     -> Result<(), KernelFailure<()>> {
        panic!("unknown identity must precede kernels")
    };
    let mut denial = None;
    let mut checkpoint = Ok(());
    let mut taken = None;
    let mut inner_succeeded = false;
    let outer = crate::backend::run_scope(None, 0, 0, |grandparent| {
        let inner = crate::backend::run_scope(None, 0, 0, |kernel| {
            let result = if certify {
                match decomposition
                    .certify(&lease, 1, &changed, editions, interior, interface, back)
                {
                    Err(DecomposeCertificationFailure::Run(failure)) => Some(failure),
                    _ => None,
                }
            } else {
                decomposition
                    .run(Some(&lease), &changed, editions, interior, interface, back)
                    .err()
            };
            denial = result.and_then(|failure| match failure.cause {
                DecomposeFailure::Input(cause) => Some(cause),
                _ => None,
            });
            taken = kernel.take_nested_computation_stop();
            checkpoint = kernel.checkpoint(0);
            Ok::<_, KernelFailure<()>>(())
        });
        inner_succeeded = inner.result.is_ok();
        assert_eq!(grandparent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        taken,
        None | Some(NestedComputationStop::DomainFailure)
    ));
    assert_eq!(
        (denial, checkpoint, inner_succeeded, outer.result.is_ok()),
        (
            Some(DecomposeInputDenial::UnknownIdentity(unknown)),
            Ok(()),
            true,
            true
        ),
    );
}
