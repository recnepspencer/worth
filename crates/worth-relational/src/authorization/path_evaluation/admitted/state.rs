use crate::authorization::admission::{self, ObservationResult};
use crate::authorization::dependency_collection::RelationalAuthorizationDependencySets;
use crate::authorization::evidence::RelationalAuthorizationPathDependencies;
use crate::authorization::{
    RelationalAuthorizationAdjacencyDependency, RelationalAuthorizationBudgetedObservationStop,
    RelationalAuthorizationObservationAdmission, RelationalAuthorizationObservationCounters,
};
use crate::identity::data::{EntityId, RelationId};
use crate::runtime::RelationalRuntime;
use crate::visibility::materialization::read_records::VisibilityProjectionView;
use worth_foundational::facade::AspectFieldLocator;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Witness {
    pub(super) entities: Vec<EntityId>,
}
impl Witness {
    pub(super) fn current(&self) -> EntityId {
        *self
            .entities
            .last()
            .expect("a witness retains its principal")
    }
    pub(super) fn entity_at(&self, ordinal: usize) -> Option<EntityId> {
        self.entities.get(ordinal).copied()
    }
}
pub(super) struct Context<'r, 'v, 's> {
    pub(super) runtime: &'r RelationalRuntime,
    pub(super) view: &'v VisibilityProjectionView<'s>,
}
pub(super) struct State<'a, A> {
    pub(super) dependencies: RelationalAuthorizationDependencySets,
    pub(super) counters: &'a mut RelationalAuthorizationObservationCounters,
    pub(super) admission: &'a mut A,
}
impl<'a, A: RelationalAuthorizationObservationAdmission> State<'a, A> {
    pub(super) fn new(
        principal: EntityId,
        counters: &'a mut RelationalAuthorizationObservationCounters,
        admission: &'a mut A,
    ) -> ObservationResult<Self, A> {
        admission::ordered_insert::<A, EntityId>(admission, 0, 1, 0)?;
        Ok(Self {
            dependencies: RelationalAuthorizationDependencySets::new(principal),
            counters,
            admission,
        })
    }
    pub(super) fn entity(&mut self, entity: EntityId) -> ObservationResult<(), A> {
        admission::ordered_insert::<A, EntityId>(
            self.admission,
            self.dependencies.entities.len(),
            1,
            0,
        )?;
        self.dependencies.entities.insert(entity);
        Ok(())
    }
    pub(super) fn relation(&mut self, relation: RelationId) -> ObservationResult<(), A> {
        admission::ordered_insert::<A, RelationId>(
            self.admission,
            self.dependencies.relations.len(),
            1,
            0,
        )?;
        self.dependencies.relations.insert(relation);
        Ok(())
    }
    pub(super) fn adjacency(
        &mut self,
        adjacency: RelationalAuthorizationAdjacencyDependency,
    ) -> ObservationResult<(), A> {
        admission::ordered_insert::<A, RelationalAuthorizationAdjacencyDependency>(
            self.admission,
            self.dependencies.adjacencies.len(),
            1,
            0,
        )?;
        self.dependencies.adjacencies.insert(adjacency);
        Ok(())
    }
    pub(super) fn field(
        &mut self,
        entity: EntityId,
        field: &AspectFieldLocator,
    ) -> ObservationResult<(), A> {
        admission::prepare(
            self.admission,
            field.field_path().fields().len() as u64 + 1,
            0,
        )?;
        let width = field
            .field_path()
            .fields()
            .iter()
            .try_fold(
                field.aspect().aspect_key().as_str().len(),
                |width, field| width.checked_add(field.as_str().len()),
            )
            .and_then(|width| width.checked_add(1))
            .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
        admission::ordered_insert::<A, (EntityId, AspectFieldLocator)>(
            self.admission,
            self.dependencies.fields.len(),
            width,
            field.owned_allocation_capacity_bytes(),
        )?;
        admission::prepare(self.admission, width as u64, 0)?;
        self.dependencies.fields.insert((entity, field.clone()));
        Ok(())
    }
    pub(super) fn finish(self) -> ObservationResult<RelationalAuthorizationPathDependencies, A> {
        admission::array::<A, EntityId>(self.admission, self.dependencies.entities.len())?;
        admission::array::<A, RelationId>(self.admission, self.dependencies.relations.len())?;
        admission::array::<A, RelationalAuthorizationAdjacencyDependency>(
            self.admission,
            self.dependencies.adjacencies.len(),
        )?;
        admission::array::<A, (EntityId, AspectFieldLocator)>(
            self.admission,
            self.dependencies.fields.len(),
        )?;
        Ok(self.dependencies.finish())
    }
}
