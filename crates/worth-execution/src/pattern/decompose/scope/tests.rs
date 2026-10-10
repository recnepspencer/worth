//! A decomposition handoff retains the request ownership of its refusal.
use super::*;
use crate::backend::{run_scope, KernelFailure, KernelStop, ScopeStop};
use crate::tests::{authority, request, TEST_LOCK};

#[test]
fn a_refused_decomposition_handoff_cannot_be_taken() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let child = parent.child(request(1, 2_000, 100)).unwrap();
        let backend = if width == 0 {
            BackendKind::Serial
        } else {
            BackendKind::Native
        };
        let reached = std::sync::atomic::AtomicBool::new(false);
        let outer = run_scope(Some(&parent), 0, 0, |context| {
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
            let map = ExecutionMap::<(), u64>::from_keyless_partitions(Default::default()).unwrap();
            let mut handoff = |_| {
                reached.store(true, std::sync::atomic::Ordering::SeqCst);
                child.reserve_retained_memory(u64::MAX).map(drop)
            };
            let result = decomposition.run_with_backend(
                Some(&child),
                backend,
                &map,
                DecomposeKernelEditions {
                    interior: 1,
                    interface: 1,
                    back: 1,
                },
                |_, _| {
                    Ok::<_, MapKernelFailure<()>>(InteriorResult {
                        interior: (),
                        contribution: (),
                    })
                },
                |_, _| {
                    Ok(InterfaceSolution {
                        solution: (),
                        slices: vec![],
                    })
                },
                |_, _| Ok(()),
                true,
                Some(&mut handoff),
            );
            assert!(
                reached.load(std::sync::atomic::Ordering::SeqCst),
                "the actual handoff ran: {:?}",
                result.as_ref().err()
            );
            assert!(matches!(
                result,
                Err(DecomposeRunFailure {
                    cause: DecomposeFailure::ScopeAdmission(_),
                    ..
                })
            ));
            assert_eq!(
                context.take_nested_computation_stop(),
                None,
                "the handoff's refused reservation belongs to its request"
            );
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(matches!(
            outer.result,
            Err(ScopeStop::Failure(KernelFailure::Stop(
                KernelStop::NestedStopped
            )))
        ));
    }
}
