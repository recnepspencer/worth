use super::*;

pub(crate) enum RelationalPartitionVisit {
    RadixNode,
    Partition(PartitionId),
}

impl RelationalPersistentRegionSet {
    pub(crate) fn partition_ids_iter(&self) -> impl Iterator<Item = PartitionId> + '_ {
        let mut stack: [Option<(&RelationalPersistentRegionNode, u32, u32)>; 33] = [None; 33];
        let mut length = 0;
        if let Some(root) = self.index_root.as_deref() {
            stack[0] = Some((root, 0, 0));
            length = 1;
        }
        std::iter::from_fn(move || loop {
            if length == 0 {
                return None;
            }
            length -= 1;
            let (node, depth, prefix) = stack[length].take().expect("occupied radix stack entry");
            if depth == PARTITION_KEY_BITS {
                if matches!(node.leaf, Some(RelationalPersistentRegionLeaf::Present(_))) {
                    return Some(PartitionId(prefix));
                }
                continue;
            }
            if let Some(one) = node.one.as_deref() {
                stack[length] = Some((one, depth + 1, (prefix << 1) | 1));
                length += 1;
            }
            if let Some(zero) = node.zero.as_deref() {
                stack[length] = Some((zero, depth + 1, prefix << 1));
                length += 1;
            }
        })
    }

    pub(crate) fn try_for_each_partition_id<E>(
        &self,
        mut visit: impl FnMut(PartitionId) -> Result<(), E>,
    ) -> Result<(), E> {
        self.try_visit_partitions(|event| match event {
            RelationalPartitionVisit::RadixNode => Ok(()),
            RelationalPartitionVisit::Partition(id) => visit(id),
        })
    }

    pub(crate) fn try_visit_partitions<E>(
        &self,
        mut visit: impl FnMut(RelationalPartitionVisit) -> Result<(), E>,
    ) -> Result<(), E> {
        visit_node(self.index_root.as_ref(), 0, 0, &mut visit)
    }
}

fn visit_node<E>(
    current: Option<&Arc<RelationalPersistentRegionNode>>,
    depth: u32,
    key_prefix: u32,
    visit: &mut impl FnMut(RelationalPartitionVisit) -> Result<(), E>,
) -> Result<(), E> {
    let Some(node) = current else { return Ok(()) };
    visit(RelationalPartitionVisit::RadixNode)?;
    if depth == PARTITION_KEY_BITS {
        if matches!(node.leaf, Some(RelationalPersistentRegionLeaf::Present(_))) {
            visit(RelationalPartitionVisit::Partition(PartitionId(key_prefix)))?;
        }
        return Ok(());
    }
    visit_node(node.zero.as_ref(), depth + 1, key_prefix << 1, visit)?;
    visit_node(node.one.as_ref(), depth + 1, (key_prefix << 1) | 1, visit)
}
