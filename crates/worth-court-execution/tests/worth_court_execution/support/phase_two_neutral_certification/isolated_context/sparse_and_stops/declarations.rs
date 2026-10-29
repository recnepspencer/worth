pub(crate) use {worth_execution::BackInput, worth_execution::CancellationToken, worth_execution::CanonicalBits, worth_execution::ChargedBytes, worth_execution::DecomposeFailure, worth_foundational::DeterminismContract, crate::phase_two_neutral_certification::isolated_context::EDITIONS, crate::phase_two_neutral_certification::isolated_context::EQUIVALENCE_ID, worth_execution::ExecutionDecompose, worth_execution::ExecutionMap, worth_foundational::ExecutionPosture, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::MapKernelFailure, worth_execution::MapPartition, worth_foundational::PartitionIdentity, crate::phase_two_neutral_certification::isolated_context::TEST_LOCK, worth_execution::compare_canonical_values, crate::phase_two_neutral_certification::isolated_context::execution_matrix, crate::phase_two_neutral_certification::isolated_context::lease};

#[derive(Clone, Copy)]
pub(crate) struct Subdomain {
    pub(crate) diagonal: f64,
    pub(crate) coupling: f64,
    pub(crate) rhs: f64,
    pub(crate) interface_diagonal: f64,
    pub(crate) interface_rhs: f64,
}
impl ChargedBytes for Subdomain {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Equation {
    pub(crate) diagonal: f64,
    pub(crate) rhs: f64,
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
pub(crate) struct Interior {
    pub(crate) diagonal: f64,
    pub(crate) coupling: f64,
    pub(crate) rhs: f64,
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

pub(crate) fn domains() -> [Subdomain; 2] {
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

pub(crate) fn domain_map(values: &[Subdomain; 2]) -> ExecutionMap<Subdomain, u64> {
    let ids = vec![worth_foundational::PartitionIdentity::new(1), worth_foundational::PartitionIdentity::new(2)];
    worth_execution::ExecutionMap::try_from_declared_partitions(
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

pub(crate) fn changed_domain_map(value: Subdomain) -> ExecutionMap<Subdomain, u64> {
    let identity = worth_foundational::PartitionIdentity::new(1);
    worth_execution::ExecutionMap::try_from_declared_partitions(
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

pub(crate) fn decomposition() -> ExecutionDecompose<
    Interior,
    Equation,
    f64,
    f64,
    f64,
    impl Fn(&Equation, &Equation) -> Equation + Clone,
> {
    worth_execution::ExecutionDecompose::try_new(
        vec![worth_foundational::PartitionIdentity::new(1), worth_foundational::PartitionIdentity::new(2)],
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

pub(crate) fn interior(
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

pub(crate) fn interface(
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

pub(crate) fn back(
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
pub(crate) fn undecomposed_solve(values: &[Subdomain; 2]) -> [f64; 3] {
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