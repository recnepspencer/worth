//! Current-loan child reads preserve actual full-generation relation occurrences.
use super::*;
use crate::derived_relation::{DerivedRelationData, DerivedRelationReadShape};
use crate::validation::data::StructuralRelationRecord;

/// Actual endpoint preparation tied to its originating Source row and route.
pub struct PreparedManagedIndexProjectionRead {
    root: ManagedIndexSourceOccurrence,
    route: usize,
    prepared: super::super::projection::PreparedManagedIndexRelationRead,
}

/// One admitted direct relation occurrence; an entity ID alone cannot issue it.
pub struct ManagedIndexSourceProjectionOccurrence {
    root: ManagedIndexSourceOccurrence,
    route: usize,
    relation: StructuralRelationRecord,
    entity: EntityId,
}
impl ManagedIndexSourceProjectionOccurrence {
    pub const fn entity(&self) -> EntityId {
        self.entity
    }
    pub const fn relation(&self) -> StructuralRelationRecord {
        self.relation
    }
}

impl ManagedIndexSourceRows<'_> {
    pub fn prepare_projection(
        &self,
        root: &ManagedIndexSourceOccurrence,
        route: usize,
    ) -> Result<PreparedManagedIndexProjectionRead, ReadDenial> {
        self.admit_occurrence(root)?;
        let path = &self
            .projection
            .program
            .projection_routes()
            .get(route)
            .ok_or(Definition::InvalidExpression)?
            .path;
        let prepared = self.projection.prepare(
            path,
            root.row,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )?;
        Ok(PreparedManagedIndexProjectionRead {
            root: root.clone(),
            route,
            prepared,
        })
    }

    /// Support includes empty adjacency and every physical relation, even when
    /// several genuine relations select the same endpoint entity.
    pub fn projection_support<'borrow>(
        &'borrow self,
        read: &'borrow PreparedManagedIndexProjectionRead,
    ) -> Result<&'borrow DerivedRelationData, ReadDenial> {
        self.admit_occurrence(&read.root)?;
        let route = self
            .projection
            .program
            .projection_routes()
            .get(read.route)
            .ok_or(Definition::InvalidExpression)?;
        let shape = DerivedRelationReadShape::Adjacency { reverse: false };
        self.projection
            .read(&read.prepared, &route.path, read.root.row, shape)
    }

    pub fn projection_occurrences(
        &self,
        read: &PreparedManagedIndexProjectionRead,
    ) -> Result<Vec<ManagedIndexSourceProjectionOccurrence>, ReadDenial> {
        let support = self.projection_support(read)?;
        let route = &self.projection.program.projection_routes()[read.route];
        let (kind, outgoing) = route.physical_relation();

        let bytes = support
            .physical_relations()
            .len()
            .checked_mul(size_of::<ManagedIndexSourceProjectionOccurrence>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Definition::WorkBudgetExceeded)?;
        self.projection.claim_retained(bytes)?;
        let mut occurrences = Vec::new();
        occurrences
            .try_reserve_exact(support.physical_relations().len())
            .map_err(|_| {
                ReadDenial::Request(
                    crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                )
            })?;
        for &relation in support.physical_relations() {
            self.projection.checkpoint(1)?;
            let (parent, entity) = if outgoing {
                (relation.source, relation.target)
            } else {
                (relation.target, relation.source)
            };
            if relation.kind_id != kind || parent != read.root.row {
                return Err(Definition::ForeignObservation.into());
            }
            // Retained dangling audit edges are physical support, never live rows.
            if support.endpoints().binary_search(&entity.into()).is_err() {
                continue;
            }
            occurrences.push(ManagedIndexSourceProjectionOccurrence {
                root: read.root.clone(),
                route: read.route,
                relation,
                entity,
            });
        }
        Ok(occurrences)
    }

    pub fn with_projected_field<T>(
        &self,
        read: &PreparedManagedIndexProjectionRead,
        occurrence: &ManagedIndexSourceProjectionOccurrence,
        column: usize,
        project: impl FnOnce(Option<&worth_foundational::facade::AspectValue>) -> T,
    ) -> Result<T, ReadDenial> {
        if occurrence.root != read.root || occurrence.route != read.route {
            return Err(Definition::ForeignObservation.into());
        }
        let data = self.projection_support(read)?;
        if data
            .physical_relations()
            .binary_search_by_key(&occurrence.relation.relation_id, |relation| {
                relation.relation_id
            })
            .ok()
            .and_then(|index| data.physical_relations().get(index))
            != Some(&occurrence.relation)
            || data
                .endpoints()
                .binary_search(&occurrence.entity.into())
                .is_err()
        {
            return Err(Definition::ForeignObservation.into());
        }
        let route = &self.projection.program.projection_routes()[read.route];
        let field = route
            .fields()
            .get(column)
            .ok_or(Definition::InvalidExpression)?;
        self.projection
            .with_field_at(occurrence.entity, route.endpoint_kind(), field, project)
    }
}
