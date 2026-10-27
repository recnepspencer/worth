//! A scoped answer retains original Native bodies and the physical visibility facts.
use super::{
    aggregate_observation::AggregateObservation, schema_layout::ComponentSetLayout,
    source_facts::SourceFacts,
};
use std::sync::Arc;
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_query_installation::facade::WorthQueryOperationComponentReadScope;
use worth_relational::facade::{identity::EntityId, snapshots::SnapshotHandle};
pub(in crate::domain_computation::primary_graph) mod acquisition;
mod commitment;
mod currentness;
mod key;
mod origin;
pub(in crate::domain_computation::primary_graph) use key::{
    ComponentSelectionKey, ComponentSelectionSubject,
};
use origin::ComponentSelectionOrigin;

#[derive(Debug)]
pub(in crate::domain_computation) struct ComponentSelection {
    key: ComponentSelectionKey,
    origin: ComponentSelectionOrigin,
    layout: ComponentSetLayout,
    source: SnapshotHandle,
    aggregates: Vec<Arc<AggregateObservation>>,
    facts: Vec<SourceFacts>,
    commitment: [u8; 32],
}
impl ComponentSelection {
    pub(in crate::domain_computation::primary_graph) fn new(
        scope: WorthQueryOperationComponentReadScope,
        layout: ComponentSetLayout,
        root: EntityId,
        subject: ComponentSelectionSubject,
        source: SnapshotHandle,
        aggregates: Vec<Arc<AggregateObservation>>,
        facts: Vec<SourceFacts>,
        poll: impl FnMut() -> Result<
            (),
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        >,
    ) -> Result<Option<Self>, worth_relational::facade::transactions::RelationalExecutionDenialCause>
    {
        let key =
            ComponentSelectionKey::new(scope.meaning_identity(), layout.definition, root, subject);
        Self::from_parts(
            ComponentSelectionOrigin::Operation(scope),
            key,
            layout,
            source,
            aggregates,
            facts,
            poll,
        )
    }
    fn from_parts(
        origin: ComponentSelectionOrigin,
        key: ComponentSelectionKey,
        layout: ComponentSetLayout,
        source: SnapshotHandle,
        aggregates: Vec<Arc<AggregateObservation>>,
        facts: Vec<SourceFacts>,
        poll: impl FnMut() -> Result<
            (),
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        >,
    ) -> Result<Option<Self>, worth_relational::facade::transactions::RelationalExecutionDenialCause>
    {
        if aggregates
            .iter()
            .any(|body| body.source() != &source || body.native().definition() != layout.definition)
        {
            return Ok(None);
        }
        let commitment = commitment::derive(&key, &aggregates, &facts, poll)?;
        Ok(Some(Self {
            key,
            origin,
            layout,
            source,
            aggregates,
            facts,
            commitment,
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn commitment(&self) -> [u8; 32] {
        self.commitment
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &ComponentSelectionKey {
        &self.key
    }
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &SnapshotHandle {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn operation_scope(
        &self,
    ) -> Option<&WorthQueryOperationComponentReadScope> {
        match &self.origin {
            ComponentSelectionOrigin::Operation(scope) => Some(scope),
            ComponentSelectionOrigin::Query { .. } => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn layout(&self) -> &ComponentSetLayout {
        &self.layout
    }
    pub(in crate::domain_computation::primary_graph) fn aggregates(
        &self,
    ) -> &[Arc<AggregateObservation>] {
        &self.aggregates
    }
    pub(in crate::domain_computation::primary_graph) fn facts(&self) -> &[SourceFacts] {
        &self.facts
    }
    pub(in crate::domain_computation::primary_graph) fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        if self.key != other.key
            || self.origin != other.origin
            || self.aggregates.len() != other.aggregates.len()
            || self.facts.len() != other.facts.len()
        {
            return Ok(false);
        }
        for (a, b) in self.aggregates.iter().zip(&other.aggregates) {
            if !a.same_truth_with_poll(b, &mut poll)? {
                return Ok(false);
            }
        }
        for (a, b) in self.facts.iter().zip(&other.facts) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
}
impl PartialEq for ComponentSelection {
    fn eq(&self, other: &Self) -> bool {
        self.same_truth_with_poll(other, || Ok::<(), std::convert::Infallible>(()))
            .unwrap()
    }
}
impl Eq for ComponentSelection {}
