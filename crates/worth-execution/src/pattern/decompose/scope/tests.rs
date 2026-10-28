//! A decomposition handoff retains the request ownership of its refusal.
use super::*;
use crate::backend::{run_scope, KernelFailure, KernelStop, ScopeStop};
use crate::tests::TEST_LOCK;

#[test]
fn a_refused_decomposition_handoff_cannot_be_taken() {
    if crate::tests::fresh_process::ran_in_fresh_child(
        module_path!(),
        "a_refused_decomposition_handoff_cannot_be_taken",
    ) {
        return;
    }
    let _serial = TEST_LOCK.lock().unwrap();
    // This fixture tests a refused handoff, not incidental framework size. Its
    // physical authority lives in a fresh test process, like the allocation
    // owner fixtures; the shared process-ceiling fixtures remain independent.
    let physical = crate::ExecutionAuthority::try_construct(crate::ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(4).unwrap(),
        charged_memory_bytes: None,
    })
    .unwrap();
    let lifecycle_request = |workers| crate::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(workers).unwrap(),
                None,
                Some(100),
            ),
        ),
        deadline: None,
        cancellation: crate::CancellationToken::new(),
    };
    for width in [0_usize, 1, 2, 4] {
        let parent = physical
            .request_lease(lifecycle_request(width.max(1)))
            .unwrap();
        let child = parent.child(lifecycle_request(1)).unwrap();
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

#[test]
fn lease_free_decomposition_entry_fits_its_serial_tree_quote_exactly() {
    use crate::backend::{run_scope_within, RunLimits};
    use crate::{CancellationToken, KeylessPartition, SerialMemoryBudget, SerialRequest};
    use std::collections::BTreeMap;
    type Combine = fn(&(), &());
    fn combine(_: &(), _: &()) {}
    for short in [0, 1] {
        let request = SerialRequest::from_memory(
            SerialMemoryBudget::new(Some(1 << 20)),
            CancellationToken::new(),
            None,
        );
        let mut entry_admitted = None;
        let outer = run_scope_within(None, 0, 0, true, Some(100), Some(&request), |_| {
            let tree =
                ReductionTree::<(), Combine>::checked_serial_build_memory_bound(1, 0).unwrap();
            // Scope custody: two tree bounds, two identities, the reducer,
            // its staged result and denial, and the active nested context.
            let quote = 2 * tree
                + 2 * size_of::<PartitionIdentity>() as u64
                + size_of::<Combine>() as u64
                + size_of::<Staged<(), (), (), (), (), Combine>>() as u64
                + size_of::<DecomposeFailure<()>>() as u64
                + RunLimits::framework_context_bytes_for_serial_scope(1).unwrap();
            let budget = request.memory().expect("explicit accounted request");
            let _other = budget
                .reserve(
                    budget.limit().expect("explicit finite serial tree court")
                        - budget.charged_memory_bytes()
                        - quote
                        + short,
                )
                .unwrap();
            let mut owner = ExecutionDecompose::<(), (), (), (), (), Combine>::try_new(
                vec![PartitionIdentity::new(1)],
                (),
                combine,
                0,
                0,
                0,
                0,
                0,
            )
            .unwrap();
            let map = ExecutionMap::<(), u64>::from_keyless_partitions(BTreeMap::from([(
                PartitionIdentity::new(1),
                KeylessPartition {
                    value: (),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )]))
            .unwrap();
            let stopped = owner.run(
                None,
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
                        slices: vec![()],
                    })
                },
                |_, _| Ok(()),
            );
            entry_admitted = Some(match stopped {
                Err(DecomposeRunFailure {
                    cause: DecomposeFailure::Interior { .. },
                    ..
                }) => true,
                Err(DecomposeRunFailure {
                    cause: DecomposeFailure::ScopeAdmission(LeaseDenial::MemoryExhausted(_)),
                    ..
                }) => false,
                other => panic!(
                    "serial tree entry boundary returned {:?}",
                    other.err().map(|e| e.cause)
                ),
            });
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(
            outer.result.is_err(),
            "the nested refusal remains request-owned"
        );
        assert_eq!(
            entry_admitted,
            Some(short == 0),
            "serial first-build bound, short by {short}"
        );
        assert_eq!(
            request
                .memory()
                .expect("explicit accounted request")
                .charged_memory_bytes(),
            0
        );
    }
}
