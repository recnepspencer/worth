use super::{read_execution_denial, RootSelectionWork, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::application_query::{
    observed_source::{
        DeclaredOrbitSelection, WorthQueryObservedFieldRevision, WorthQueryObservedRootSelection,
    },
    read_execution::{relational_read_denial, WorthQueryApplicationReadExecutionDenialKind},
};
use crate::domain_computation::primary_graph::{
    source_facts::{SourceFactReadDenial, SourceFactsBuilder},
    structural_walk::StructuralWalkDependencies,
    WorthQueryPrimaryGraphLayout,
};
use std::sync::Arc;
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::{identity::EntityId, runtime::VisibilityProjectionView};

#[derive(Clone, Default)]
pub(super) struct RootPathSourceBuilder {
    facts: SourceFactsBuilder,
    orbit: Option<DeclaredOrbitSelection>,
}
impl RootPathSourceBuilder {
    pub(super) fn observe_entity(&mut self, entity: EntityId) -> bool {
        self.facts.observe_entity(entity)
    }
    pub(super) fn set_walks(&mut self, walks: Arc<StructuralWalkDependencies>) {
        self.facts.set_walks(walks);
    }
    pub(super) fn set_orbit(&mut self, orbit: DeclaredOrbitSelection) {
        self.orbit = Some(orbit);
    }
    pub(super) fn observe_guard_field(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        projection: &VisibilityProjectionView,
        graph: &WorthQueryPrimaryGraphLayout,
        entity: EntityId,
        entity_name: &str,
        aspect: &AspectKey,
        field: &FieldKey,
        work: &mut RootSelectionWork,
    ) -> Result<WorthQueryObservedFieldRevision, WorthQueryApplicationReadExecutionDenial> {
        if !self.facts.has_field(entity, aspect, field) {
            work.charge_source_observation(entity_name)?;
        }
        self.facts
            .observe_guard_field(
                execution,
                projection,
                graph,
                entity,
                entity_name,
                aspect,
                field,
            )
            .map_err(|denial| source_denial(denial, entity_name))
    }
    pub(super) fn finish(
        self,
    ) -> Result<Arc<WorthQueryObservedRootSelection>, WorthQueryApplicationReadExecutionDenial>
    {
        let facts = self
            .facts
            .finish()
            .map_err(|cause| relational_read_denial(cause, "source witness allocation"))?;
        Ok(Arc::new(WorthQueryObservedRootSelection::new(
            facts, self.orbit,
        )))
    }
}
fn source_denial(
    denial: SourceFactReadDenial,
    subject: &str,
) -> WorthQueryApplicationReadExecutionDenial {
    match denial {
        SourceFactReadDenial::Execution(cause) => relational_read_denial(cause, subject),
        SourceFactReadDenial::Unavailable => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            subject,
        ),
        SourceFactReadDenial::WorkLimitExceeded => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
            subject,
        ),
    }
}
