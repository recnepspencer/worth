use super::*;

fn map(entries: &[(u64, u64)]) -> ExecutionMap<u64, u64> {
    let identities = entries
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

fn decomposer() -> ExecutionDecompose<u64, u64, u64, u64, u64, impl Fn(&u64, &u64) -> u64 + Clone> {
    ExecutionDecompose::try_new(
        (1..=64).map(PartitionIdentity::new).collect(),
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

fn interior(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InteriorResult {
        interior: *value,
        contribution: *value,
    })
}

fn interface(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InterfaceSolution {
        solution: *value,
        slices: vec![*value; 64],
    })
}

fn back(
    input: &BackInput<u64, u64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(input.interior + input.interface_slice)
}

#[test]
fn one_changed_contribution_recombines_only_path_and_matches_fresh_tree() {
    let original: Vec<_> = (1..=64).map(|id| (id, id)).collect();
    let mut retained = decomposer();
    let first = retained
        .run(None, &map(&original), EDITIONS, interior, interface, back)
        .unwrap();
    assert_eq!(first.reduction_metrics.recombined_nodes, 64);

    let updated = retained
        .run(
            None,
            &map(&[(31, 100)]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert!(updated.reduction_metrics.recombined_nodes > 0);
    assert!(updated.reduction_metrics.recombined_nodes < 64);
    assert_eq!(updated.reuse.unchanged_contributions, 63);

    let mut fresh_entries = original;
    fresh_entries[30].1 = 100;
    let fresh = decomposer()
        .run(
            None,
            &map(&fresh_entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(updated.values, fresh.values);
}
