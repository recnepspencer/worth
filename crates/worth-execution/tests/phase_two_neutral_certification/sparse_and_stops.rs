use super::*;

#[derive(Clone, Copy)]
struct Subdomain {
    diagonal: f64,
    coupling: f64,
    rhs: f64,
    interface_diagonal: f64,
    interface_rhs: f64,
}
impl ChargedBytes for Subdomain {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Clone, Copy)]
struct Equation {
    diagonal: f64,
    rhs: f64,
}
impl ChargedBytes for Equation {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl CanonicalBits for Equation {
    fn canonical_len(&self) -> Option<usize> {
        Some(16)
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(&self.diagonal.to_bits().to_le_bytes()) && visit(&self.rhs.to_bits().to_le_bytes())
    }
}

#[derive(Clone, Copy)]
struct Interior {
    diagonal: f64,
    coupling: f64,
    rhs: f64,
}
impl ChargedBytes for Interior {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl CanonicalBits for Interior {
    fn canonical_len(&self) -> Option<usize> {
        Some(24)
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(&self.diagonal.to_bits().to_le_bytes())
            && visit(&self.coupling.to_bits().to_le_bytes())
            && visit(&self.rhs.to_bits().to_le_bytes())
    }
}

fn domains() -> [Subdomain; 2] {
    [
        Subdomain {
            diagonal: 2.0,
            coupling: 1.0,
            rhs: 5.0,
            interface_diagonal: 4.0,
            interface_rhs: 8.0,
        },
        Subdomain {
            diagonal: 3.0,
            coupling: 1.0,
            rhs: 7.0,
            interface_diagonal: 0.0,
            interface_rhs: 0.0,
        },
    ]
}

fn domain_map(values: &[Subdomain; 2]) -> ExecutionMap<Subdomain, u64> {
    let ids = vec![PartitionIdentity::new(1), PartitionIdentity::new(2)];
    ExecutionMap::try_from_declared_partitions(
        ids.clone(),
        ids.into_iter()
            .zip(values.iter().copied())
            .map(|(identity, value)| MapPartition {
                identity,
                value,
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            })
            .collect(),
    )
    .unwrap()
}

fn changed_domain_map(value: Subdomain) -> ExecutionMap<Subdomain, u64> {
    let identity = PartitionIdentity::new(1);
    ExecutionMap::try_from_declared_partitions(
        vec![identity],
        vec![MapPartition {
            identity,
            value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .unwrap()
}

fn decomposition() -> ExecutionDecompose<
    Interior,
    Equation,
    f64,
    f64,
    f64,
    impl Fn(&Equation, &Equation) -> Equation + Clone,
> {
    ExecutionDecompose::try_new(
        vec![PartitionIdentity::new(1), PartitionIdentity::new(2)],
        Equation {
            diagonal: 0.0,
            rhs: 0.0,
        },
        |a: &Equation, b: &Equation| {
            std::thread::yield_now();
            Equation {
                diagonal: a.diagonal + b.diagonal,
                rhs: a.rhs + b.rhs,
            }
        },
        64,
        0,
        64,
        1024,
        0,
    )
    .unwrap()
}

fn interior(
    value: &Subdomain,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<Interior, Equation>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    for _ in 0..(value.diagonal as usize) {
        std::thread::yield_now();
    }
    Ok(InteriorResult {
        interior: Interior {
            diagonal: value.diagonal,
            coupling: value.coupling,
            rhs: value.rhs,
        },
        contribution: Equation {
            diagonal: value.interface_diagonal - value.coupling * value.coupling / value.diagonal,
            rhs: value.interface_rhs - value.coupling * value.rhs / value.diagonal,
        },
    })
}

fn interface(
    value: &Equation,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<f64, f64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    std::thread::yield_now();
    let solved = value.rhs / value.diagonal;
    Ok(InterfaceSolution {
        solution: solved,
        slices: vec![solved; 2],
    })
}

fn back(
    value: &BackInput<Interior, f64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<f64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    for _ in 0..(value.interior.diagonal as usize) {
        std::thread::yield_now();
    }
    Ok(
        (value.interior.rhs - value.interior.coupling * value.interface_slice)
            / value.interior.diagonal,
    )
}

// Gaussian elimination on the original sparse three-variable system, without
// forming Schur complements or using the decomposition's callbacks.
fn undecomposed_solve(values: &[Subdomain; 2]) -> [f64; 3] {
    let mut rows = [
        [values[0].diagonal, values[0].coupling, 0.0, values[0].rhs],
        [values[0].coupling, 4.0, values[1].coupling, 8.0],
        [0.0, values[1].coupling, values[1].diagonal, values[1].rhs],
    ];
    for pivot in 0..3 {
        let divisor = rows[pivot][pivot];
        for cell in &mut rows[pivot][pivot..] {
            *cell /= divisor;
        }
        let pivot_row = rows[pivot];
        for (index, row) in rows.iter_mut().enumerate() {
            if index != pivot {
                let multiplier = row[pivot];
                for (cell, source) in row[pivot..].iter_mut().zip(&pivot_row[pivot..]) {
                    *cell -= multiplier * source;
                }
            }
        }
    }
    [rows[0][3], rows[1][3], rows[2][3]]
}

#[test]
fn sparse_decomposition_matches_independent_undecomposed_solver_across_widths() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let values = domains();
    let expected = undecomposed_solve(&values);
    assert_eq!(expected, [2.0, 1.0, 2.0]);
    let mut serial_bits = None;
    let mut serial_work = None;
    for (posture, width) in execution_matrix() {
        let lease = lease(
            posture,
            width,
            20,
            CancellationToken::new(),
            None,
            DeterminismContract::ContractEquivalent(EQUIVALENCE_ID),
        );
        assert_eq!(
            lease.policy().determinism(),
            DeterminismContract::ContractEquivalent(EQUIVALENCE_ID)
        );
        let mut decompose = decomposition();
        let result = decompose
            .certify(
                &lease,
                0x9e37_79b9 ^ width as u64,
                &domain_map(&values),
                EDITIONS,
                interior,
                interface,
                back,
            )
            .unwrap();
        let bits = result
            .values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        if let Some(serial) = &serial_bits {
            assert_eq!(&bits, serial);
        } else {
            serial_bits = Some(bits);
        }
        assert!(compare_canonical_values(&lease, &expected[0], &result.values[0]).unwrap());
        assert!(compare_canonical_values(&lease, &expected[2], &result.values[1]).unwrap());
        let work = result.total_report.charged_work();
        assert_eq!(
            work,
            result.interior_report.charged_work()
                + result.reduction_metrics.charged_work
                + result.interface_report.unwrap().charged_work()
                + result.back_report.unwrap().charged_work()
        );
        if let Some(serial) = serial_work {
            assert_eq!(work, serial);
        } else {
            serial_work = Some(work);
        }
        assert!(result.total_report.charged_span() < result.total_report.charged_work());

        let changed = Subdomain {
            rhs: values[0].rhs + 1.0,
            ..values[0]
        };
        let updated = decompose
            .certify(
                &lease,
                0xdead_beef ^ width as u64,
                &changed_domain_map(changed),
                EDITIONS,
                interior,
                interface,
                back,
            )
            .unwrap();
        let direct = undecomposed_solve(&[changed, values[1]]);
        assert!(compare_canonical_values(&lease, &direct[0], &updated.values[0]).unwrap());
        assert!(compare_canonical_values(&lease, &direct[2], &updated.values[1]).unwrap());
    }
    let bitwise = lease(
        ExecutionPosture::Serial,
        1,
        1,
        CancellationToken::new(),
        None,
        DeterminismContract::CanonicalBitwise,
    );
    let adjacent = f64::from_bits(expected[0].to_bits() + 1);
    assert!(!compare_canonical_values(&bitwise, &expected[0], &adjacent).unwrap());
    let equivalent = lease(
        ExecutionPosture::Serial,
        1,
        1,
        CancellationToken::new(),
        None,
        DeterminismContract::ContractEquivalent(EQUIVALENCE_ID),
    );
    assert!(compare_canonical_values(&equivalent, &expected[0], &adjacent).unwrap());
    assert!(!compare_canonical_values(&equivalent, &expected[0], &(expected[0] + 1e-3)).unwrap());

    let ceiling = serial_work.unwrap() - 1;
    let limited = lease(
        ExecutionPosture::Serial,
        1,
        ceiling,
        CancellationToken::new(),
        None,
        DeterminismContract::CanonicalBitwise,
    );
    let exhausted = decomposition().run(
        Some(&limited),
        &domain_map(&values),
        EDITIONS,
        interior,
        interface,
        back,
    );
    assert!(matches!(exhausted, Err(error)
        if matches!(error.cause, DecomposeFailure::Back { .. })
            && error.total_report.charged_work() == ceiling));
}

mod stage_stops;
