//! A typed name issued from actual Native aggregate truth, never a read grant.
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{
    identity::KindId,
    runtime::{AggregateRead, PinnedAggregateLineageScope, PinnedAggregateRead},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum AggregateSource {
    Orbit(KindId),
    Component,
    DerivedOrbit,
}

/// Persistent aggregate name within an installed definition binding. This value
/// grants no observation, write authority, or permission to disclose history.
pub struct WorthQueryAggregateIdentity<Schema, Relation> {
    definition: AggregateDefinitionIdentity,
    identity: AggregateIdentity,
    binding: ApplicationSchemaBindingIdentity,
    source: AggregateSource,
    _marker: PhantomData<fn() -> (Schema, Relation)>,
}
impl<Schema, Relation> WorthQueryAggregateIdentity<Schema, Relation> {
    pub(in crate::domain_computation::primary_graph) fn from_invariant_read(
        native: &AggregateRead,
        binding: ApplicationSchemaBindingIdentity,
        relation: KindId,
    ) -> Self {
        Self::new(
            native.definition(),
            native.identity(),
            binding,
            AggregateSource::Orbit(relation),
        )
    }
    pub(in crate::domain_computation::primary_graph) fn from_pinned_read(
        native: &PinnedAggregateRead,
        binding: ApplicationSchemaBindingIdentity,
        relation: KindId,
    ) -> Self {
        Self::new(
            native.definition(),
            native.identity(),
            binding,
            AggregateSource::Orbit(relation),
        )
    }
    pub(in crate::domain_computation::primary_graph) fn from_component_read<Member>(
        native: &PinnedAggregateRead<Member>,
        binding: ApplicationSchemaBindingIdentity,
    ) -> Self {
        Self::new(
            native.definition(),
            native.identity(),
            binding,
            AggregateSource::Component,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn from_invariant_component_read<Member>(
        native: &AggregateRead<Member>,
        binding: ApplicationSchemaBindingIdentity,
    ) -> Self {
        Self::new(
            native.definition(),
            native.identity(),
            binding,
            AggregateSource::Component,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn from_invariant_derived_orbit_read<
        Member,
    >(
        native: &AggregateRead<Member>,
        binding: ApplicationSchemaBindingIdentity,
    ) -> Self {
        Self::new(
            native.definition(),
            native.identity(),
            binding,
            AggregateSource::DerivedOrbit,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn component_lineage_selection(
        &self,
        binding: &ApplicationSchemaBindingIdentity,
        definition: AggregateDefinitionIdentity,
    ) -> Option<PinnedAggregateLineageScope<'static>> {
        (self.binding == *binding
            && self.source == AggregateSource::Component
            && self.definition == definition)
            .then(|| PinnedAggregateLineageScope::for_definition(self.definition, self.identity))
    }

    fn new(
        definition: AggregateDefinitionIdentity,
        identity: AggregateIdentity,
        binding: ApplicationSchemaBindingIdentity,
        source: AggregateSource,
    ) -> Self {
        Self {
            definition,
            identity,
            binding,
            source,
            _marker: PhantomData,
        }
    }
    /// Portable name bytes for application naming and comparison, not authority.
    pub fn bytes(&self) -> &[u8; 32] {
        self.identity.bytes()
    }

    pub(in crate::domain_computation::primary_graph) fn lineage_selection(
        &self,
        binding: &ApplicationSchemaBindingIdentity,
        relation: KindId,
    ) -> Option<PinnedAggregateLineageScope<'static>> {
        (self.binding == *binding && self.source == AggregateSource::Orbit(relation))
            .then(|| PinnedAggregateLineageScope::new(relation, self.definition, self.identity))
    }

    // Only the lineage body owner qualifies endpoints returned by Native.
    pub(super) fn related(&self, identity: AggregateIdentity) -> Self {
        Self::new(self.definition, identity, self.binding.clone(), self.source)
    }
}
impl<Schema, Relation> Clone for WorthQueryAggregateIdentity<Schema, Relation> {
    fn clone(&self) -> Self {
        Self {
            definition: self.definition,
            identity: self.identity,
            binding: self.binding.clone(),
            source: self.source,
            _marker: PhantomData,
        }
    }
}
impl<Schema, Relation> PartialEq for WorthQueryAggregateIdentity<Schema, Relation> {
    fn eq(&self, other: &Self) -> bool {
        self.definition == other.definition
            && self.identity == other.identity
            && self.binding == other.binding
            && self.source == other.source
    }
}
impl<Schema, Relation> Eq for WorthQueryAggregateIdentity<Schema, Relation> {}
impl<Schema, Relation> Hash for WorthQueryAggregateIdentity<Schema, Relation> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.definition.hash(state);
        self.identity.hash(state);
        self.binding.hash(state);
        self.source.hash(state);
    }
}
impl<Schema, Relation> std::fmt::Debug for WorthQueryAggregateIdentity<Schema, Relation> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryAggregateIdentity")
            .field("definition", &self.definition)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
