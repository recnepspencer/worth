//! One closed source-shape acquisition for invariant and pinned views.
use super::evaluation::SourceRead;
use crate::derived_relation::definition::{
    DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle,
};
use crate::derived_relation::proof::DerivedRelationWalkStep;
use crate::derived_relation::read::DerivedRelationData;
use crate::derived_relation::read_denial::DerivedRelationReadDenial;
use crate::derived_relation::read_shape::DerivedRelationReadShape as Shape;
use crate::{graph_member::NativeMemberKey, schema::data::RelationalSchemaRegistry};

pub(super) fn acquire(
    registry: &RelationalSchemaRegistry,
    definition: &DerivedRelationHandle,
    root: NativeMemberKey,
    shape: Shape,
    read: &mut impl SourceRead,
) -> Result<(DerivedRelationData, Vec<DerivedRelationWalkStep>), DerivedRelationReadDenial> {
    if !matches!(shape, Shape::Adjacency { .. }) {
        let names = definition
            .parts()
            .from
            .name_bytes()
            .and_then(|n| n.checked_add(definition.parts().to.name_bytes()?))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        read.checkpoint(names)?;
        if definition.parts().from != definition.parts().to {
            return Err(DefinitionDenial::DiscontinuousEndpoints.into());
        }
    }
    match shape {
        Shape::Adjacency { reverse } => Ok((
            super::evaluation::evaluate_position(
                registry,
                definition,
                definition.parts().nodes.len() - 1,
                reverse,
                root,
                read,
            )?,
            Vec::new(),
        )),
        Shape::Reachable { reverse } => {
            super::traversal::acquire(registry, definition, root, reverse, false, read)
        }
        Shape::Component => {
            super::traversal::acquire(registry, definition, root, false, true, read)
        }
    }
}
