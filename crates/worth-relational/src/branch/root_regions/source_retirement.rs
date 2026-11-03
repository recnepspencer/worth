//! Fixed radix-path storage advances through the original immutable region set.
use super::*;

struct Frame {
    node: Arc<RelationalPersistentRegionNode>,
    depth: u32,
    prefix: u32,
}
pub(crate) struct RelationalRegionSourceWalk {
    stack: [Option<Frame>; 33],
    length: usize,
    _regions: Arc<RelationalPersistentRegionSet>,
}
pub(crate) enum RelationalRegionSourceWalkProgress {
    Advanced,
    Region(
        PartitionId,
        Arc<crate::storage::residency::RelationalRegionResidency>,
    ),
    Complete,
}
impl RelationalRegionSourceWalk {
    pub(in crate::branch) fn new(regions: Arc<RelationalPersistentRegionSet>) -> Self {
        let mut stack = std::array::from_fn(|_| None);
        let mut length = 0;
        if let Some(node) = &regions.index_root {
            stack[0] = Some(Frame {
                node: node.clone(),
                depth: 0,
                prefix: 0,
            });
            length = 1;
        }
        Self {
            stack,
            length,
            _regions: regions,
        }
    }
    /// Exactly one actual radix node; original set ownership prevents any
    /// popped Arc from destroying a final node or region payload here.
    pub(crate) fn advance(
        &mut self,
        retire_native_leases: bool,
    ) -> RelationalRegionSourceWalkProgress {
        use RelationalRegionSourceWalkProgress as Progress;
        if self.length == 0 {
            return Progress::Complete;
        }
        self.length -= 1;
        let Frame {
            node,
            depth,
            prefix,
        } = self.stack[self.length]
            .take()
            .expect("original occupied radix frame");
        if depth == PARTITION_KEY_BITS {
            return match &node.leaf {
                Some(RelationalPersistentRegionLeaf::Present(original)) => {
                    if retire_native_leases {
                        original.region.partition.retire_native_leases();
                    }
                    Progress::Region(
                        PartitionId(prefix),
                        original.region.partition.cell().clone(),
                    )
                }
                Some(RelationalPersistentRegionLeaf::Removed(_)) | None => Progress::Advanced,
            };
        }
        for (child, bit) in [(&node.one, 1), (&node.zero, 0)] {
            if let Some(child) = child {
                self.stack[self.length] = Some(Frame {
                    node: child.clone(),
                    depth: depth + 1,
                    prefix: (prefix << 1) | bit,
                });
                self.length += 1;
            }
        }
        Progress::Advanced
    }
}
