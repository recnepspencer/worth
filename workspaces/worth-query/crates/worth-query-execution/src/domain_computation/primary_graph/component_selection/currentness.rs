//! Reuse delegates to the existing physical fact and Native comparison owners.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationObservedFact as Fact,
        WorthQueryApplicationSourceAdjacencyEndpoints as Endpoints,
        WorthQuerySourceCurrentnessFailure as Failure,
    },
    source_read_verification::SourceReadVerification,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};
use worth_relational::facade::{
    runtime::RelationalRuntime, transactions::RelationalExecutionDenialCause as Cause,
};
impl ComponentSelection {
    pub(in crate::domain_computation::primary_graph) fn visit_physical<E>(
        &self,
        mut visit: impl FnMut(Fact) -> Result<(), E>,
    ) -> Result<(), E> {
        for source in &self.facts {
            for entity in &source.entities {
                visit(Fact::SourceEntity { entity_id: *entity })?;
            }
            for field in &source.aspects {
                visit(Fact::SourceFieldRevision {
                    entity_id: field.entity,
                    locator: AspectFieldLocator::new(
                        LocatorAuthority::Authoritative,
                        field.aspect.clone(),
                        CanonicalFieldPath::single(field.field.clone()),
                    ),
                    native_revision: field.native_revision,
                })?;
            }
            for adjacency in &source.adjacencies {
                visit(Fact::SourceAdjacencyRevision {
                    relation_kind: adjacency.relation_kind,
                    anchor: adjacency.anchor,
                    direction: adjacency.direction,
                    native_revision: adjacency.native_revision,
                    comparison_work_limit: adjacency.comparison_work_limit,
                    endpoints: Endpoints::empty(),
                })?;
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn touches_entity(
        &self,
        entity: EntityId,
    ) -> bool {
        self.key.root() == entity
            || self
                .facts
                .iter()
                .any(|facts| facts.entities.binary_search(&entity).is_ok())
    }
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        SourceReadVerification::open(runtime, execution, snapshot)?
            .check_component(self, execution, control)
    }
}
