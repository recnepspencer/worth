//! One Native occurrence/support/cardinality owner for both traversal strategies.
use super::*;
use worth_query_installation::facade::WorthQueryInstalledGraphRelation;
pub(super) struct AcquiredRelation<'a> {
    pub relation: &'a WorthQueryInstalledGraphRelation,
    pub read: PreparedManagedIndexProjectionRead,
    pub occurrences: std::vec::IntoIter<ManagedIndexSourceProjectionOccurrence>,
}
impl<'borrow, 'scope, 'authority> Projector<'borrow, 'scope, 'authority> {
    pub(super) fn acquire(
        &mut self,
        program: &ManagedIndexSourceProgram,
        root: &ManagedIndexSourceOccurrence,
        index: usize,
        parent_path: &str,
        parent: Option<(
            &PreparedManagedIndexProjectionRead,
            &ManagedIndexSourceProjectionOccurrence,
        )>,
    ) -> Result<AcquiredRelation<'borrow>, MaterialDenial> {
        let route = &program.projection_routes()[index];
        let graph = self.graph;
        let relation = graph
            .relations()
            .iter()
            .find(|relation| {
                relation.slot_type() == route.name() && relation.parent_path() == parent_path
            })
            .ok_or(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::InvalidExpression,
            ))?;
        let read = match parent {
            None => self.rows.prepare_projection(root, index)?,
            Some((read, occurrence)) => self
                .rows
                .prepare_nested_projection(read, occurrence, index)?,
        };
        let support = self.rows.projection_support(&read)?;
        for &(entity, _) in support.entity_dependencies() {
            self.dependency(ViewDependency::Entity(entity))?;
        }
        for adjacency in support.adjacency_dependencies() {
            self.dependency(ViewDependency::Adjacency(
                adjacency.anchor,
                adjacency.relation,
                u8::from(!adjacency.outgoing),
            ))?;
        }
        let occurrences = self.rows.projection_occurrences(&read)?;
        validate_relation_cardinality(relation, occurrences.len()).map_err(|cause| {
            let cause = crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenial::new(
                crate::domain_computation::vocabulary::primary_graph::projection_denial::WorthQueryApplicationProjectionDenialKind::RelationCardinalityMismatch,
                cause.into_subject(),
            );
            MaterialDenial::Native(super::super::failure::retain_failure(self.execution, cause))
        })?;
        Ok(AcquiredRelation {
            relation,
            read,
            occurrences: occurrences.into_iter(),
        })
    }
}
