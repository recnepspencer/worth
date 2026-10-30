//! Visit each immutable radix allocation before deciding whether to descend.
use super::*;
use crate::branch::RelationalRootAllocationVisitor;

impl RelationalPersistentRegionSet {
    pub(crate) fn try_visit_allocation_union<V: RelationalRootAllocationVisitor>(
        &self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        if visitor.allocation(
            RelationalPersistentRegionNodeObservation {
                node_id: self.set_id,
                allocation_kind: RelationalPersistentRegionAllocationKind::SetObject,
                authoritative_bytes: size_of::<Self>() as u64,
            }
            .into(),
        ) {
            visit_node(self.index_root.as_deref(), visitor)?;
        }
        Ok(())
    }
}

fn visit_node<V: RelationalRootAllocationVisitor>(
    node: Option<&RelationalPersistentRegionNode>,
    visitor: &mut V,
) -> Result<(), V::Error> {
    let Some(node) = node else { return Ok(()) };
    if !visitor.allocation(
        RelationalPersistentRegionNodeObservation {
            node_id: node.node_id,
            allocation_kind: RelationalPersistentRegionAllocationKind::MapNodeObject,
            authoritative_bytes: size_of::<RelationalPersistentRegionNode>() as u64,
        }
        .into(),
    ) {
        return Ok(());
    }
    match &node.leaf {
        Some(RelationalPersistentRegionLeaf::Present(value)) => {
            visitor.allocation(
                RelationalPersistentRegionNodeObservation {
                    node_id: node.node_id,
                    allocation_kind: RelationalPersistentRegionAllocationKind::ReplacementStorage,
                    authoritative_bytes: size_of_val(value.as_ref()) as u64,
                }
                .into(),
            );
            visitor.region(
                value.0.observation(),
                value.0.partition.try_admitted_partition(),
            )?;
        }
        Some(RelationalPersistentRegionLeaf::Removed(value)) => {
            visitor.allocation(
                RelationalPersistentRegionNodeObservation {
                    node_id: node.node_id,
                    allocation_kind: RelationalPersistentRegionAllocationKind::RemovalStorage,
                    authoritative_bytes: size_of_val(value.as_ref()) as u64,
                }
                .into(),
            );
        }
        None => {}
    }
    visit_node(node.zero.as_deref(), visitor)?;
    visit_node(node.one.as_deref(), visitor)
}
