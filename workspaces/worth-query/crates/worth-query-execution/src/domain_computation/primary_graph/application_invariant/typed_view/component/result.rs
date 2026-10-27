//! Native-backed values; typed iteration copies metadata, never membership truth.
use super::super::{
    ViewPosture, WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantReadView,
};
use crate::domain_computation::primary_graph::aggregate_identity::WorthQueryAggregateIdentity;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{AggregateRead, AggregateReadCollection},
};

pub(super) struct ComponentContext<Schema, Entity> {
    binding: ApplicationSchemaBindingIdentity,
    proposal: Option<(u64, u64)>,
    posture: ViewPosture,
    kind: KindId,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}
impl<Schema, Entity> ComponentContext<Schema, Entity> {
    pub(super) fn new(
        view: &WorthQueryApplicationInvariantReadView<'_, Schema>,
        kind: KindId,
    ) -> Self {
        Self {
            binding: view.binding_identity.clone(),
            proposal: view.proposal_affinity,
            posture: view.posture,
            kind,
            _marker: PhantomData,
        }
    }
    fn entity(&self, id: EntityId) -> WorthQueryApplicationInvariantEntity<Schema, Entity> {
        WorthQueryApplicationInvariantEntity {
            entity_id: id,
            kind: self.kind,
            binding_identity: self.binding.clone(),
            proposal_affinity: self.proposal,
            posture: self.posture,
            _marker: PhantomData,
        }
    }
}
impl<Schema, Entity> Clone for ComponentContext<Schema, Entity> {
    fn clone(&self) -> Self {
        Self {
            binding: self.binding.clone(),
            proposal: self.proposal,
            posture: self.posture,
            kind: self.kind,
            _marker: PhantomData,
        }
    }
}
/// One genuine cohort, including persistent identity and complete members.
pub struct WorthQueryApplicationInvariantComponent<Schema, Set, Entity> {
    native: AggregateRead,
    identity: WorthQueryAggregateIdentity<Schema, Set>,
    context: ComponentContext<Schema, Entity>,
}
impl<Schema, Set, Entity> WorthQueryApplicationInvariantComponent<Schema, Set, Entity> {
    pub(super) fn new(native: AggregateRead, context: ComponentContext<Schema, Entity>) -> Self {
        let identity = WorthQueryAggregateIdentity::from_invariant_component_read(
            &native,
            context.binding.clone(),
        );
        Self {
            native,
            identity,
            context,
        }
    }
    pub fn identity(&self) -> &WorthQueryAggregateIdentity<Schema, Set> {
        &self.identity
    }
    pub fn len(&self) -> usize {
        self.native.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    /// Canonical Native entity order from this exact proposed or committed view.
    pub fn members(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationInvariantEntity<Schema, Entity>> + '_
    {
        self.native.members().map(|id| self.context.entity(id))
    }
}
/// Complete set in Native aggregate identity order; no copied membership cache.
pub struct WorthQueryApplicationInvariantComponents<Schema, Set, Entity> {
    native: AggregateReadCollection,
    context: ComponentContext<Schema, Entity>,
    _set: PhantomData<fn() -> Set>,
}
impl<Schema, Set, Entity> WorthQueryApplicationInvariantComponents<Schema, Set, Entity> {
    pub(super) fn new(
        native: AggregateReadCollection,
        context: ComponentContext<Schema, Entity>,
    ) -> Self {
        Self {
            native,
            context,
            _set: PhantomData,
        }
    }
    pub fn len(&self) -> usize {
        self.native.len()
    }
    pub fn is_empty(&self) -> bool {
        self.native.is_empty()
    }
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationInvariantComponent<Schema, Set, Entity>> + '_
    {
        self.native.iter().map(|native| {
            WorthQueryApplicationInvariantComponent::new(native.clone(), self.context.clone())
        })
    }
}
