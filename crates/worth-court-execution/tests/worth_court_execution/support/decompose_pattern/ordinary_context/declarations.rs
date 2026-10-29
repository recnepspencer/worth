pub(crate) use {worth_execution::BackInput, worth_execution::DecomposeFailure, worth_execution::DecomposeInputDenial, worth_execution::DecomposeKernelEditions, worth_execution::ExecutionDecompose, worth_execution::ExecutionMap, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::MapKernelFailure, worth_execution::MapPartition};
pub(crate) use {worth_foundational::PartitionIdentity};

pub(crate) const EDITIONS: DecomposeKernelEditions = DecomposeKernelEditions {
    interior: 1,
    interface: 1,
    back: 1,
};

#[derive(Clone, Copy)]
pub(crate) struct Subdomain {
    pub(crate) diagonal: f64,
    pub(crate) interface_coupling: f64,
    pub(crate) rhs: f64,
    pub(crate) base_diagonal: f64,
    pub(crate) base_rhs: f64,
}

impl worth_execution::ChargedBytes for Subdomain {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InterfaceEquation {
    pub(crate) diagonal: f64,
    pub(crate) rhs: f64,
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
pub(crate) struct Interior {
    pub(crate) diagonal: f64,
    pub(crate) interface_coupling: f64,
    pub(crate) rhs: f64,
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

pub(crate) fn subdomain_map(entries: &[(u64, Subdomain)]) -> ExecutionMap<Subdomain, u64> {
    let identities: Vec<_> = entries
        .iter()
        .map(|(id, _)| worth_foundational::PartitionIdentity::new(*id))
        .collect();
    let partitions = entries
        .iter()
        .map(|(id, value)| MapPartition {
            identity: worth_foundational::PartitionIdentity::new(*id),
            value: *value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    worth_execution::ExecutionMap::try_from_declared_partitions(identities, partitions).unwrap()
}

pub(crate) fn interior(
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

pub(crate) fn interface(
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

pub(crate) fn back(
    input: &BackInput<Interior, f64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<f64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(
        (input.interior.rhs - input.interior.interface_coupling * input.interface_slice)
            / input.interior.diagonal,
    )
}

pub(crate) fn decomposer() -> ExecutionDecompose<
    Interior,
    InterfaceEquation,
    f64,
    f64,
    f64,
    impl Fn(&InterfaceEquation, &InterfaceEquation) -> InterfaceEquation + Clone,
> {
    worth_execution::ExecutionDecompose::try_new(
        vec![worth_foundational::PartitionIdentity::new(1), worth_foundational::PartitionIdentity::new(2)],
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

pub(crate) fn base_subdomains() -> [(u64, Subdomain); 2] {
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

pub(crate) fn direct_solve(subdomains: &[Subdomain; 2]) -> [f64; 3] {
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
#[allow(unused_imports)]
pub(crate) use {};