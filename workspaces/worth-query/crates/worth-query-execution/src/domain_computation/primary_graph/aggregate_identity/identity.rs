//! A typed name issued from actual Native aggregate truth, never a read grant.
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{identity::KindId, runtime::AggregateRead};

/// Persistent aggregate name within an installed relation binding. This value
/// grants no observation, write authority, or permission to disclose history.
pub struct WorthQueryAggregateIdentity<Schema, Relation> {
    definition: AggregateDefinitionIdentity,
    identity: AggregateIdentity,
    binding: ApplicationSchemaBindingIdentity,
    relation: KindId,
    _marker: PhantomData<fn() -> (Schema, Relation)>,
}
impl<Schema, Relation> WorthQueryAggregateIdentity<Schema, Relation> {
    pub(in crate::domain_computation::primary_graph) fn from_invariant_read(
        native: &AggregateRead,
        binding: ApplicationSchemaBindingIdentity,
        relation: KindId,
    ) -> Self {
        Self {
            definition: native.definition(),
            identity: native.identity(),
            binding,
            relation,
            _marker: PhantomData,
        }
    }
    /// Portable name bytes for application naming and comparison, not authority.
    pub fn bytes(&self) -> &[u8; 32] {
        self.identity.bytes()
    }
}
impl<Schema, Relation> Clone for WorthQueryAggregateIdentity<Schema, Relation> {
    fn clone(&self) -> Self {
        Self {
            definition: self.definition,
            identity: self.identity,
            binding: self.binding.clone(),
            relation: self.relation,
            _marker: PhantomData,
        }
    }
}
impl<Schema, Relation> PartialEq for WorthQueryAggregateIdentity<Schema, Relation> {
    fn eq(&self, other: &Self) -> bool {
        self.definition == other.definition
            && self.identity == other.identity
            && self.binding == other.binding
            && self.relation == other.relation
    }
}
impl<Schema, Relation> Eq for WorthQueryAggregateIdentity<Schema, Relation> {}
impl<Schema, Relation> Hash for WorthQueryAggregateIdentity<Schema, Relation> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.definition.hash(state);
        self.identity.hash(state);
        self.binding.hash(state);
        self.relation.hash(state);
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
