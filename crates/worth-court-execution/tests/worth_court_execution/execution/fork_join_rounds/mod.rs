#[allow(unused_imports)]
use crate::fork_join_rounds::ordinary_context::*;

pub(crate) use worth_execution::ForkChild;

#[test]
fn fork_children_must_prove_disjoint_access_before_dispatch() {
    let ids = [
        worth_foundational::PartitionIdentity::new(1),
        worth_foundational::PartitionIdentity::new(2),
    ];
    let child = |identity, read_keys, write_keys| ForkChild {
        identity,
        input: identity.value(),
        read_keys,
        write_keys,
        kernel_scratch_bytes: 0,
        max_result_bytes: 0,
    };
    let overlapping = worth_execution::ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![], vec![7]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        overlapping,
        Err(worth_execution::MapDenial::WriteSetOverlap { .. })
    ));

    let read_after_write = worth_execution::ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![7], vec![]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        read_after_write,
        Err(worth_execution::MapDenial::ReadWriteConflict { .. })
    ));
}
