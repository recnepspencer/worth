//! Borrowed complete physical dependencies, including a managed Source's empty set.
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::AggregateObservation, component_selection::ComponentSelection,
    structural_walk::StructuralWalkDependencies,
};
use std::sync::Arc;

pub(super) struct FactSource<'a> {
    pub entities: &'a [EntityId],
    pub aspects: &'a [WorthQueryObservedFieldRevision],
    pub adjacencies: &'a [WorthQueryObservedAdjacencyRevision],
    pub walks: Option<&'a Arc<StructuralWalkDependencies>>,
    pub aggregate: Option<&'a Arc<AggregateObservation>>,
    pub components: Option<&'a Arc<ComponentSelection>>,
}
impl<'a> FactSource<'a> {
    fn row(value: &'a WorthQueryObservedSourceFootprint) -> Self {
        Self {
            entities: &value.entities,
            aspects: &value.aspects,
            adjacencies: &value.adjacencies,
            walks: None,
            aggregate: None,
            components: None,
        }
    }
    fn selection(value: &'a WorthQueryObservedRootSelection) -> Self {
        Self {
            entities: &value.entities,
            aspects: &value.aspects,
            adjacencies: &value.adjacencies,
            walks: value.walks.as_ref(),
            aggregate: value.orbit.as_ref().map(|x| &x.aggregate),
            components: value.components.as_ref().map(|x| &x.body),
        }
    }
    pub fn count(&self) -> Option<usize> {
        self.entities
            .len()
            .checked_add(self.aspects.len())?
            .checked_add(self.adjacencies.len())?
            .checked_add(self.walks.map_or(0, |x| x.values().len()))?
            .checked_add(usize::from(self.aggregate.is_some()))?
            .checked_add(usize::from(self.components.is_some()))
    }
}
impl WorthQueryObservedSourceFootprint {
    pub(in crate::domain_computation::primary_graph::application_query) fn fact_count(
        &self,
    ) -> Option<usize> {
        let base = FactSource::row(self).count()?;
        let selected = self.root_selection.as_ref().map_or(Some(0), |selection| {
            FactSource::selection(selection).count()
        })?;
        base.checked_add(selected)?
            .checked_add(self.managed_counts().0)
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn field_count(
        &self,
    ) -> Option<usize> {
        self.aspects
            .len()
            .checked_add(
                self.root_selection
                    .as_ref()
                    .map_or(0, |selection| selection.aspects.len()),
            )?
            .checked_add(self.managed_counts().1)
    }
    fn managed_counts(&self) -> (usize, usize) {
        self.root_selection
            .as_ref()
            .and_then(|selection| selection.managed_index.as_ref())
            .map_or((0, 0), |managed| {
                (
                    managed.dependencies.fact_count,
                    managed.dependencies.field_count,
                )
            })
    }

    pub(super) fn fact_sources(&self) -> impl Iterator<Item = FactSource<'_>> {
        std::iter::once(FactSource::row(self))
            .chain(self.root_selection.iter().map(|x| FactSource::selection(x)))
            .chain(self.managed_meanings().flat_map(|meaning| {
                let source = meaning.footprint();
                std::iter::once(FactSource::row(source)).chain(
                    source
                        .root_selection
                        .iter()
                        .map(|x| FactSource::selection(x)),
                )
            }))
    }
    pub(super) fn managed_meanings(
        &self,
    ) -> impl Iterator<Item = &Arc<source_identity::WorthQueryObservedSourceMeaning>> {
        self.root_selection
            .iter()
            .flat_map(|selection| selection.managed_index.iter())
            .flat_map(|managed| managed.dependencies.meanings())
    }
    pub(super) fn validate_selections(
        &self,
        layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        poll: &impl Fn(&str) -> Result<(), WorthQuerySourceExpectationDenial>,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        for selection in self.root_selection.iter().chain(
            self.managed_meanings()
                .flat_map(|m| m.footprint().root_selection.iter()),
        ) {
            poll("source selection")?;
            selection.validate_walks(layout, poll)?;
        }
        Ok(())
    }
}
