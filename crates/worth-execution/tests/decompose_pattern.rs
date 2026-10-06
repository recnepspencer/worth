use worth_execution::{
    BackInput, DecomposeFailure, DecomposeInputDenial, DecomposeKernelEditions, ExecutionDecompose,
    ExecutionMap, InterfaceSolution, InteriorResult, MapKernelFailure, MapPartition,
};
use worth_foundational::PartitionIdentity;

const EDITIONS: DecomposeKernelEditions = DecomposeKernelEditions {
    interior: 1,
    interface: 1,
    back: 1,
};

#[derive(Clone, Copy)]
struct Subdomain {
    diagonal: f64,
    interface_coupling: f64,
    rhs: f64,
    base_diagonal: f64,
    base_rhs: f64,
}

impl worth_execution::ChargedBytes for Subdomain {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Clone, Copy)]
struct InterfaceEquation {
    diagonal: f64,
    rhs: f64,
}

impl worth_execution::ChargedBytes for InterfaceEquation {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl worth_execution::CanonicalBits for InterfaceEquation {
    fn canonical_len(&self) -> Option<usize> {
        Some(16)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.diagonal.to_bits().to_le_bytes())
            && visitor(&self.rhs.to_bits().to_le_bytes())
    }
}

#[derive(Clone, Copy)]
struct Interior {
    diagonal: f64,
    interface_coupling: f64,
    rhs: f64,
}

impl worth_execution::ChargedBytes for Interior {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl worth_execution::CanonicalBits for Interior {
    fn canonical_len(&self) -> Option<usize> {
        Some(24)
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visitor(&self.diagonal.to_bits().to_le_bytes())
            && visitor(&self.interface_coupling.to_bits().to_le_bytes())
            && visitor(&self.rhs.to_bits().to_le_bytes())
    }
}

fn subdomain_map(entries: &[(u64, Subdomain)]) -> ExecutionMap<Subdomain, u64> {
    let identities: Vec<_> = entries
        .iter()
        .map(|(id, _)| PartitionIdentity::new(*id))
        .collect();
    let partitions = entries
        .iter()
        .map(|(id, value)| MapPartition {
            identity: PartitionIdentity::new(*id),
            value: *value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(identities, partitions).unwrap()
}

fn interior(
    value: &Subdomain,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<Interior, InterfaceEquation>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    let local = Interior {
        diagonal: value.diagonal,
        interface_coupling: value.interface_coupling,
        rhs: value.rhs,
    };
    let contribution = InterfaceEquation {
        diagonal: value.base_diagonal
            - value.interface_coupling * value.interface_coupling / value.diagonal,
        rhs: value.base_rhs - value.interface_coupling * value.rhs / value.diagonal,
    };
    Ok(InteriorResult {
        interior: local,
        contribution,
    })
}

fn interface(
    equation: &InterfaceEquation,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<f64, f64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    let value = equation.rhs / equation.diagonal;
    Ok(InterfaceSolution {
        solution: value,
        slices: vec![value, value],
    })
}

fn back(
    input: &BackInput<Interior, f64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<f64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(
        (input.interior.rhs - input.interior.interface_coupling * input.interface_slice)
            / input.interior.diagonal,
    )
}

fn decomposer() -> ExecutionDecompose<
    Interior,
    InterfaceEquation,
    f64,
    f64,
    f64,
    impl Fn(&InterfaceEquation, &InterfaceEquation) -> InterfaceEquation + Clone,
> {
    ExecutionDecompose::try_new(
        vec![PartitionIdentity::new(1), PartitionIdentity::new(2)],
        InterfaceEquation {
            diagonal: 0.0,
            rhs: 0.0,
        },
        |left: &InterfaceEquation, right: &InterfaceEquation| InterfaceEquation {
            diagonal: left.diagonal + right.diagonal,
            rhs: left.rhs + right.rhs,
        },
        64,
        0,
        64,
        1024,
        0,
    )
    .unwrap()
}

fn base_subdomains() -> [(u64, Subdomain); 2] {
    [
        (
            1,
            Subdomain {
                diagonal: 2.0,
                interface_coupling: 1.0,
                rhs: 5.0,
                base_diagonal: 4.0,
                base_rhs: 8.0,
            },
        ),
        (
            2,
            Subdomain {
                diagonal: 3.0,
                interface_coupling: 1.0,
                rhs: 7.0,
                base_diagonal: 0.0,
                base_rhs: 0.0,
            },
        ),
    ]
}

fn direct_solve(subdomains: &[Subdomain; 2]) -> [f64; 3] {
    let mut rows = [
        [
            subdomains[0].diagonal,
            subdomains[0].interface_coupling,
            0.0,
            subdomains[0].rhs,
        ],
        [
            subdomains[0].interface_coupling,
            4.0,
            subdomains[1].interface_coupling,
            8.0,
        ],
        [
            0.0,
            subdomains[1].interface_coupling,
            subdomains[1].diagonal,
            subdomains[1].rhs,
        ],
    ];
    for pivot in 0..3 {
        let divisor = rows[pivot][pivot];
        for value in &mut rows[pivot][pivot..] {
            *value /= divisor;
        }
        let pivot_row = rows[pivot];
        for (row, values) in rows.iter_mut().enumerate() {
            if row == pivot {
                continue;
            }
            let factor = values[pivot];
            for (value, pivot_value) in values[pivot..].iter_mut().zip(&pivot_row[pivot..]) {
                *value -= factor * pivot_value;
            }
        }
    }
    [rows[0][3], rows[1][3], rows[2][3]]
}

#[test]
fn sparse_interface_solve_reuses_unchanged_contribution_and_local_slice() {
    let entries = base_subdomains();
    let mut decompose = decomposer();
    let initial = decompose
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(initial.total_report.charged_work(), 20);
    let serial = decomposer()
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in initial.values.iter().zip(&serial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    let direct = direct_solve(&[entries[0].1, entries[1].1]);
    assert!((initial.values[0] - direct[0]).abs() < 1e-12);
    assert!((initial.values[1] - direct[2]).abs() < 1e-12);
    assert!((direct[1] - 1.0).abs() < 1e-12);

    let repeated = decompose
        .run(
            None,
            &subdomain_map(&[]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(repeated.reuse.unchanged_contributions, 2);
    assert!(repeated.reuse.interface_solve_reused);
    assert_eq!(repeated.reuse.back_substitutions_reused, 2);
    assert!(repeated.interface_report.is_none());
    assert!(repeated.back_report.is_none());

    let mut changed = entries[0].1;
    changed.rhs += 2.0;
    changed.base_rhs += 1.0;
    let updated = decompose
        .run(
            None,
            &subdomain_map(&[(1, changed)]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(updated.reuse.unchanged_contributions, 2);
    assert!(updated.reuse.interface_solve_reused);
    assert_eq!(updated.reuse.back_substitutions_reused, 1);
    assert!(updated.back_report.is_some());
    let serial = decomposer()
        .run(
            None,
            &subdomain_map(&[(1, changed), entries[1]]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in updated.values.iter().zip(&serial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }

    changed.rhs += 1.0;
    let affected = decompose
        .run(
            None,
            &subdomain_map(&[(1, changed)]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(affected.reuse.unchanged_contributions, 1);
    assert!(!affected.reuse.interface_solve_reused);
    assert_eq!(affected.reuse.back_substitutions_reused, 0);
    let serial = decomposer()
        .run(
            None,
            &subdomain_map(&[(1, changed), entries[1]]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in affected.values.iter().zip(&serial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

#[test]
fn stopped_stage_does_not_publish_a_partial_snapshot() {
    let entries = base_subdomains();
    let mut decompose = decomposer();
    let initial = decompose
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    let failure = decompose.run(
        None,
        &subdomain_map(&[(1, entries[0].1)]),
        EDITIONS,
        |_value,
         _context|
         -> Result<InteriorResult<Interior, InterfaceEquation>, MapKernelFailure<()>> {
            panic!("test kernel panic")
        },
        interface,
        back,
    );
    assert!(
        matches!(failure, Err(error) if matches!(error.cause, DecomposeFailure::Interior { .. }))
    );
    let after = decompose
        .run(
            None,
            &subdomain_map(&[]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in after.values.iter().zip(&initial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    assert_eq!(after.reuse.back_substitutions_reused, 2);
}

#[path = "decompose_pattern/budget.rs"]
mod budget;
#[path = "decompose_pattern/editions.rs"]
mod editions;
#[path = "decompose_pattern/tree_reuse.rs"]
mod tree_reuse;
