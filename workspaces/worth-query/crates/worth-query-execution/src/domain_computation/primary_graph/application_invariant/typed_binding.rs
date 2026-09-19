use std::marker::PhantomData;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::KindId;

/// Installed entity-kind access for one exact application schema binding.
pub struct WorthQueryApplicationInvariantEntityBinding<Schema, Entity> {
    pub(in crate::domain_computation::primary_graph) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// Installed field access for one exact application schema binding.
pub struct WorthQueryApplicationInvariantFieldBinding<Schema, Entity, Value> {
    pub(in crate::domain_computation::primary_graph) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) presence:
        worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
    pub(in crate::domain_computation::primary_graph) decode:
        fn(&worth_foundational::facade::AspectValue) -> Result<Value, String>,
    pub(in crate::domain_computation::primary_graph) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// Installed relation access for one exact application schema binding.
pub struct WorthQueryApplicationInvariantRelationBinding<Schema, Relation, From, To> {
    pub(in crate::domain_computation::primary_graph) binding_identity:
        ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) relation_kind: KindId,
    pub(in crate::domain_computation::primary_graph) from_kind: KindId,
    pub(in crate::domain_computation::primary_graph) to_kind: KindId,
    pub(in crate::domain_computation::primary_graph) _marker:
        PhantomData<fn() -> (Schema, Relation, From, To)>,
}
