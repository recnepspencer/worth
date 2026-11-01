//! Current-loan direct occurrences retain exact nested parent issuance.
use super::*;
use crate::derived_relation::{DerivedRelationData, DerivedRelationReadShape};
use crate::validation::data::StructuralRelationRecord;

/// Flat actual ancestor records avoid recursive destruction. Backing drops
/// before its original payer, which survives every shared read/occurrence.
struct ProjectionLineage {
    records: Vec<StructuralRelationRecord>,
    _retention: super::super::projection::SourceProjectionRetention,
}

/// Actual endpoint preparation tied to its Source row, route and parent path.
pub struct PreparedManagedIndexProjectionRead {
    root: ManagedIndexSourceOccurrence,
    route: usize,
    anchor: EntityId,
    lineage: Option<Arc<ProjectionLineage>>,
    prepared: super::super::projection::PreparedManagedIndexRelationRead,
}

/// One admitted physical relation occurrence; coordinates cannot issue it.
pub struct ManagedIndexSourceProjectionOccurrence {
    root: ManagedIndexSourceOccurrence,
    route: usize,
    lineage: Option<Arc<ProjectionLineage>>,
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
        let descriptor = self
            .projection
            .program
            .projection_routes()
            .get(route)
            .ok_or(Definition::InvalidExpression)?;
        if descriptor.parent_route().is_some() {
            return Err(Definition::InvalidExpression.into());
        }
        let prepared = self.projection.prepare(
            &descriptor.path,
            root.row,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )?;
        Ok(PreparedManagedIndexProjectionRead {
            root: root.clone(),
            route,
            anchor: root.row,
            lineage: None,
            prepared,
        })
    }

    /// A descendant is issued only from an actual occurrence of its declared
    /// parent route in this same immutable Native loan. It grants no READ.
    pub fn prepare_nested_projection(
        &self,
        parent: &PreparedManagedIndexProjectionRead,
        occurrence: &ManagedIndexSourceProjectionOccurrence,
        route: usize,
    ) -> Result<PreparedManagedIndexProjectionRead, ReadDenial> {
        self.admit_projection_occurrence(parent, occurrence)?;
        let descriptor = self
            .projection
            .program
            .projection_routes()
            .get(route)
            .ok_or(Definition::InvalidExpression)?;
        if descriptor.parent_route() != Some(parent.route) {
            return Err(Definition::InvalidExpression.into());
        }
        let previous = parent
            .lineage
            .as_ref()
            .map_or(&[][..], |lineage| &lineage.records);
        let count = previous
            .len()
            .checked_add(1)
            .ok_or(Definition::WorkBudgetExceeded)?;
        let bytes = count
            .checked_mul(size_of::<StructuralRelationRecord>())
            .and_then(|n| n.checked_add(size_of::<(usize, usize, ProjectionLineage)>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Definition::WorkBudgetExceeded)?;
        let retention = self.projection.owned_retention(bytes)?;
        let mut records = Vec::new();
        records
            .try_reserve_exact(count)
            .map_err(|_| allocation_failed())?;
        for record in previous {
            self.projection.checkpoint(1)?;
            records.push(*record);
        }
        self.projection.checkpoint(1)?;
        records.push(occurrence.relation);
        let lineage = Arc::new(ProjectionLineage {
            records,
            _retention: retention,
        });
        let prepared = self.projection.prepare(
            &descriptor.path,
            occurrence.entity,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )?;
        Ok(PreparedManagedIndexProjectionRead {
            root: parent.root.clone(),
            route,
            anchor: occurrence.entity,
            lineage: Some(lineage),
            prepared,
        })
    }

    /// Support includes empty adjacency and every actual physical relation.
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
        self.projection.read(
            &read.prepared,
            &route.path,
            read.anchor,
            DerivedRelationReadShape::Adjacency { reverse: false },
        )
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
            .map_err(|_| allocation_failed())?;
        for &relation in support.physical_relations() {
            self.projection.checkpoint(1)?;
            let (parent, entity) = if outgoing {
                (relation.source, relation.target)
            } else {
                (relation.target, relation.source)
            };
            if relation.kind_id != kind || parent != read.anchor {
                return Err(Definition::ForeignObservation.into());
            }
            // Dangling audit records are support, never live child rows.
            if support.endpoints().binary_search(&entity.into()).is_err() {
                continue;
            }
            occurrences.push(ManagedIndexSourceProjectionOccurrence {
                root: read.root.clone(),
                route: read.route,
                lineage: read.lineage.clone(),
                relation,
                entity,
            });
        }
        Ok(occurrences)
    }

    fn admit_projection_occurrence(
        &self,
        read: &PreparedManagedIndexProjectionRead,
        occurrence: &ManagedIndexSourceProjectionOccurrence,
    ) -> Result<(), ReadDenial> {
        self.projection.checkpoint(0)?;
        let same_origin = match (&read.lineage, &occurrence.lineage) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        };
        if occurrence.root != read.root || occurrence.route != read.route || !same_origin {
            return Err(Definition::ForeignObservation.into());
        }
        let data = self.projection_support(read)?;
        if data
            .physical_relations()
            .binary_search_by_key(&occurrence.relation.relation_id, |record| {
                record.relation_id
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
        Ok(())
    }

    pub fn with_projected_field<T>(
        &self,
        read: &PreparedManagedIndexProjectionRead,
        occurrence: &ManagedIndexSourceProjectionOccurrence,
        column: usize,
        project: impl FnOnce(Option<&worth_foundational::facade::AspectValue>) -> T,
    ) -> Result<T, ReadDenial> {
        self.admit_projection_occurrence(read, occurrence)?;
        let route = &self.projection.program.projection_routes()[read.route];
        let field = route
            .fields()
            .get(column)
            .ok_or(Definition::InvalidExpression)?;
        self.projection
            .with_field_at(occurrence.entity, route.endpoint_kind(), field, project)
    }
}

fn allocation_failed() -> ReadDenial {
    ReadDenial::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}
