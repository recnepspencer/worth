//! Moved whole from `domain_computation::primary_graph::application_attempt::effect_program::model`: the items below and every impl they own.
use std::marker::PhantomData;
use std::sync::Arc;
use worth_relational::facade::transactions::EntityReference;

/// An entity handle usable only inside the effect program that produced it: an
/// existing entity from the read set, or one the program creates.
///
/// Get one from the builder's `existing_entity`, `projected_entity`, or
/// `create_entity`, then write fields or link relations through it. A handle
/// from another effect program is refused.
pub struct WorthQueryApplicationEffectEntity<Schema, Entity> {
    pub(in crate::domain_computation::primary_graph) reference: EntityReference,
    pub(in crate::domain_computation::primary_graph) entity: String,
    pub(in crate::domain_computation::primary_graph) created_effect: Option<usize>,
    pub(in crate::domain_computation::primary_graph) program: Arc<()>,
    pub(in crate::domain_computation::primary_graph) _marker: PhantomData<fn() -> (Schema, Entity)>,
}

/// Only the creating effect program can initialize this new relation's payload.
pub struct WorthQueryApplicationCreatedRelation<Schema, Relation, From, To> {
    pub(in crate::domain_computation::primary_graph) relation:
        worth_query_installation::facade::ApplicationRelationRef<Schema, Relation, From, To>,
    pub(in crate::domain_computation::primary_graph) created_effect: usize,
    pub(in crate::domain_computation::primary_graph) program: Arc<()>,
}

impl<Schema, Entity> WorthQueryApplicationEffectEntity<Schema, Entity> {
    #[doc(hidden)]
    pub fn new_for_test(
        reference: EntityReference,
        entity: String,
        created_effect: Option<usize>,
        program: Arc<()>,
        _marker: PhantomData<fn() -> (Schema, Entity)>,
    ) -> Self {
        Self {
            reference,
            entity,
            created_effect,
            program,
            _marker,
        }
    }
}
