//! The same fixed-depth radix insertion with explicit ordinary or request funding.
use super::*;

pub(super) fn insert_leaf(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    partition_id: PartitionId,
    leaf: RelationalPersistentRegionLeaf,
    depth: u32,
    issuer: &RelationalBranchRootIdentityIssuer,
    request: Option<&worth_execution::ExecutionRequest<'_, '_>>,
) -> Result<Arc<RelationalPersistentRegionNode>, RelationalBranchRootCaptureDenial> {
    let (mut zero, mut one) =
        current.map_or((None, None), |node| (node.zero.clone(), node.one.clone()));
    let current_leaf = if depth == PARTITION_KEY_BITS {
        Some(leaf)
    } else {
        debug_assert!(current.is_none_or(|node| node.leaf.is_none()));
        if key_bit(partition_id, depth) == 0 {
            zero = Some(insert_leaf(
                zero.as_ref(),
                partition_id,
                leaf,
                depth + 1,
                issuer,
                request,
            )?);
        } else {
            one = Some(insert_leaf(
                one.as_ref(),
                partition_id,
                leaf,
                depth + 1,
                issuer,
                request,
            )?);
        }
        None
    };
    let commitment = node_commitment(depth, zero.as_ref(), one.as_ref(), current_leaf.as_ref());
    let memory = super::super::root::reserve_arc_memory::<RelationalPersistentRegionNode>(request)?;
    Ok(Arc::new(RelationalPersistentRegionNode {
        node_id: issuer.issue_reachability_id()?,
        zero,
        one,
        leaf: current_leaf,
        commitment,
        _memory: memory,
    }))
}
