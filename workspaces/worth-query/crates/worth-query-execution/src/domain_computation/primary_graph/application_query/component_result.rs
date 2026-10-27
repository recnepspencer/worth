//! Inspect complete disclosed component groups without any graph read or member cache.
use super::{observed_source::DeclaredComponentSelection, WorthQueryObservedResultSet};
use crate::domain_computation::primary_graph::{
    aggregate_identity::WorthQueryAggregateIdentity, aggregate_observation::AggregateObservation,
};
use std::{marker::PhantomData, sync::Arc};
use worth_query_declaration::facade::application_query::ApplicationQueryComponentRef;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

/// A complete Native component and its completed member rows in Native order.
pub struct WorthQueryApplicationQueryComponent<'a, Schema, Set, Row> {
    identity: WorthQueryAggregateIdentity<Schema, Set>,
    members: &'a [Row],
}
impl<Schema, Set, Row> WorthQueryApplicationQueryComponent<'_, Schema, Set, Row> {
    pub fn identity(&self) -> &WorthQueryAggregateIdentity<Schema, Set> {
        &self.identity
    }
    pub const fn members(&self) -> &[Row] {
        self.members
    }
    pub const fn len(&self) -> usize {
        self.members.len()
    }
    pub const fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}
/// One complete Of or Census slot, backed by its original Native observations.
pub struct WorthQueryApplicationQueryComponents<'a, Schema, Set, Row> {
    selected: &'a DeclaredComponentSelection,
    rows: &'a [Row],
    cohorts: &'a [Arc<AggregateObservation>],
    _marker: PhantomData<fn() -> (Schema, Set)>,
}
impl<Schema, Set, Row> WorthQueryApplicationQueryComponents<'_, Schema, Set, Row> {
    pub fn len(&self) -> usize {
        self.cohorts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationQueryComponent<'_, Schema, Set, Row>>
    {
        ComponentIter {
            cohorts: self.cohorts.iter(),
            remaining: self.rows,
            binding: &self.selected.binding,
            _marker: PhantomData,
        }
    }
}
struct ComponentIter<'a, Schema, Set, Row> {
    cohorts: std::slice::Iter<'a, Arc<AggregateObservation>>,
    remaining: &'a [Row],
    binding: &'a ApplicationSchemaBindingIdentity,
    _marker: PhantomData<fn() -> (Schema, Set)>,
}
impl<'a, Schema, Set, Row> Iterator for ComponentIter<'a, Schema, Set, Row> {
    type Item = WorthQueryApplicationQueryComponent<'a, Schema, Set, Row>;
    fn next(&mut self) -> Option<Self::Item> {
        let body = self.cohorts.next()?;
        let (members, remaining) = self.remaining.split_at(body.native().len());
        self.remaining = remaining;
        Some(WorthQueryApplicationQueryComponent {
            identity: WorthQueryAggregateIdentity::from_component_read(
                body.native(),
                self.binding.clone(),
            ),
            members,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.cohorts.size_hint()
    }
}
impl<Schema, Set, Row> ExactSizeIterator for ComponentIter<'_, Schema, Set, Row> {}

pub(super) fn inspect<'a, Schema, Query, Slot, Set, Entity, Row>(
    rows: &'a [Row],
    set: &'a WorthQueryObservedResultSet<Query>,
    reference: ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity>,
) -> Option<WorthQueryApplicationQueryComponents<'a, Schema, Set, Row>> {
    let selected = set.declared_components()?;
    if selected.meaning.as_ref() != reference.meaning() {
        return None;
    }
    let cohorts = selected.body.physical_aggregates()?;
    let count = cohorts
        .iter()
        .try_fold(0usize, |count, body| count.checked_add(body.native().len()))?;
    if count != rows.len() {
        return None;
    }
    Some(WorthQueryApplicationQueryComponents {
        selected,
        rows,
        cohorts,
        _marker: PhantomData,
    })
}
