pub(crate) use {
    crate::decompose_pattern::ordinary_context::EDITIONS, worth_execution::BackInput,
    worth_execution::ExecutionDecompose, worth_execution::ExecutionMap,
    worth_execution::InterfaceSolution, worth_execution::InteriorResult,
    worth_execution::MapKernelFailure, worth_execution::MapPartition,
    worth_foundational::PartitionIdentity,
};

pub(crate) fn map(entries: &[(u64, u64)]) -> ExecutionMap<u64, u64> {
    let identities = entries
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

pub(crate) fn decomposer(
) -> ExecutionDecompose<u64, u64, u64, u64, u64, impl Fn(&u64, &u64) -> u64 + Clone> {
    worth_execution::ExecutionDecompose::try_new(
        (1..=64)
            .map(worth_foundational::PartitionIdentity::new)
            .collect(),
        0,
        |left: &u64, right: &u64| left.wrapping_add(*right),
        1024,
        0,
        64,
        3_000,
        0,
    )
    .unwrap()
}

pub(crate) fn interior(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InteriorResult {
        interior: *value,
        contribution: *value,
    })
}

pub(crate) fn interface(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InterfaceSolution {
        solution: *value,
        slices: vec![*value; 64],
    })
}

pub(crate) fn back(
    input: &BackInput<u64, u64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(input.interior + input.interface_slice)
}

pub(crate) fn recombined(metrics: &worth_execution::ReductionMetrics) -> u64 {
    metrics
        .accounted_metrics()
        .expect("a retained decomposition carries reduction accounting")
        .recombined_nodes
}
