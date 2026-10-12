//! Wrong changed-input shapes have no scope, report, or request effects.
use super::super::*;
use crate::{
    backend::run_scope,
    tests::{authority, request, TEST_LOCK},
    KeylessPartition,
};

#[test]
fn changed_input_denials_mark_nothing_and_precede_overflow_and_admission() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let _held = lease.reserve_retained_memory(17).unwrap();
    let before = lease.charged_memory_bytes();
    let cases = [
        (
            DecomposeInputDenial::ChangedIdentitiesNotCanonical,
            &[1][..],
            &[2, 1][..],
            false,
            0,
            0,
        ),
        // The empty initial tree's scope would fit if shape validation came later.
        (
            DecomposeInputDenial::InitialCoverageMismatch,
            &[][..],
            &[1][..],
            false,
            0,
            0,
        ),
        (
            DecomposeInputDenial::InitialCoverageMismatch,
            &[1][..],
            &[][..],
            false,
            u64::MAX,
            0,
        ),
        (
            DecomposeInputDenial::InitialCoverageMismatch,
            &[1][..],
            &[][..],
            false,
            0,
            10_000,
        ),
        // An empty change over a retained leaf also fits until the edition check.
        (
            DecomposeInputDenial::InteriorKernelCoverageMismatch,
            &[1][..],
            &[][..],
            true,
            0,
            0,
        ),
        (
            DecomposeInputDenial::InteriorKernelCoverageMismatch,
            &[1][..],
            &[][..],
            true,
            0,
            10_000,
        ),
    ];
    for (expected, all, changed_ids, retained, value_cap, staged_cap) in cases {
        let all = identities(all);
        let changed_ids = identities(changed_ids);
        let mut decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
            all.clone(),
            (),
            |_, _| (),
            0,
            0,
            value_cap,
            staged_cap,
            0,
        )
        .unwrap();
        let mut editions = DecomposeKernelEditions {
            interior: 1,
            interface: 1,
            back: 1,
        };
        if retained {
            decomposition
                .run(
                    None,
                    &map(&all),
                    editions,
                    |_, _| {
                        Ok::<_, MapKernelFailure<()>>(InteriorResult {
                            interior: (),
                            contribution: (),
                        })
                    },
                    |_, _| {
                        Ok(InterfaceSolution {
                            solution: (),
                            slices: vec![(); all.len()],
                        })
                    },
                    |_, _| Ok(()),
                )
                .unwrap();
            editions.interior = 2;
        }
        // An admitted map cannot hold a noncanonical identity list.
        let changed = map(
            if expected == DecomposeInputDenial::ChangedIdentitiesNotCanonical {
                &[]
            } else {
                &changed_ids
            },
        );
        let outer = run_scope(Some(&lease), 0, 0, |parent| {
            let scope_before = lease.charged_memory_bytes();
            let failure = if expected == DecomposeInputDenial::ChangedIdentitiesNotCanonical {
                decomposition
                    .check_changed_identities::<()>(&changed_ids, editions)
                    .err()
                    .unwrap()
            } else {
                decomposition
                    .run(
                        Some(&lease),
                        &changed,
                        editions,
                        |_, _| -> Result<InteriorResult<(), ()>, MapKernelFailure<()>> {
                            panic!("shape must precede interior")
                        },
                        |_, _| -> Result<InterfaceSolution<(), ()>, MapKernelFailure<()>> {
                            panic!("shape must precede interface")
                        },
                        |_, _| -> Result<(), MapKernelFailure<()>> {
                            panic!("shape must precede back map")
                        },
                    )
                    .err()
                    .unwrap()
            };
            let DecomposeFailure::Input(denial) = failure.cause else {
                panic!("input denial required")
            };
            assert_eq!(denial, expected);
            assert_eq!(failure.total_report, scope::empty_report());
            assert_eq!(parent.take_nested_computation_stop(), None);
            assert_eq!(parent.checkpoint(0), Ok(()));
            assert_eq!(lease.charged_memory_bytes(), scope_before);
            Ok::<_, MapKernelFailure<()>>(())
        });
        assert!(outer.result.is_ok(), "{expected:?}");
        assert_eq!(lease.charged_memory_bytes(), before);
    }
}

fn identities(values: &[u64]) -> Vec<PartitionIdentity> {
    values.iter().copied().map(PartitionIdentity::new).collect()
}

fn map(identities: &[PartitionIdentity]) -> ExecutionMap<(), u64> {
    ExecutionMap::from_keyless_partitions(
        identities
            .iter()
            .map(|identity| {
                (
                    *identity,
                    KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )
            })
            .collect(),
    )
    .unwrap()
}
