//! A prepared second scope must occur as an actual declared identity in the first tree.

use super::super::super::projection::WorthQueryApplicationProjectionNode;
use super::Denial;
use worth_execution::{MapKernelContext, MapKernelFailure};
use worth_foundational::facade::{AspectFieldLocator, AspectKey, AspectValue, FieldKey};
use worth_query_installation::facade::WorthQueryInstalledGraphProjection;
use worth_relational::facade::identity::EntityId;

/// Entirely borrowed installed/source custody; there is no new retained allocation.
pub(in crate::domain_computation::primary_graph::application_query) struct PreparedPairBinding<'a> {
    entity: EntityId,
    projections: &'a [WorthQueryInstalledGraphProjection],
    aspect: &'a AspectKey,
    field: &'a FieldKey,
    value: &'a AspectValue,
}

impl<'a> PreparedPairBinding<'a> {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        entity: EntityId,
        projections: &'a [WorthQueryInstalledGraphProjection],
        locator: &'a AspectFieldLocator,
        value: &'a AspectValue,
    ) -> Result<Self, Denial> {
        let [field] = locator.field_path().fields() else {
            return Err(Denial::IncompleteDependencies);
        };
        Ok(Self {
            entity,
            projections,
            aspect: locator.aspect().aspect_key(),
            field,
            value,
        })
    }

    pub(super) fn matches(
        &self,
        node: &WorthQueryApplicationProjectionNode,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<bool, MapKernelFailure<Denial>> {
        context.checkpoint(1)?;
        if node.entity_id() == self.entity {
            for projection in self.projections {
                context.checkpoint(1)?;
                if projection.parent_path() == node.result_path()
                    && projection.aspect_key() == self.aspect
                    && projection.field_key() == self.field
                    && node
                        .field(projection.slot_type())
                        .map(|field| field.value())
                        == Some(self.value)
                {
                    return Ok(true);
                }
            }
        }
        for relation in node.relations() {
            for child in relation.rows() {
                if self.matches(child, context)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
