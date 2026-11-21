use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, PortableAspectContractBasis};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::transactions::EntityReference;

pub mod emission;
pub use emission::{WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission};

/// An entity handle usable only inside the effect program that produced it: an
/// existing entity from the read set, or one the program creates.
///
/// Get one from the builder's `existing_entity`, `projected_entity`, or
/// `create_entity`, then write fields or link relations through it. A handle
/// from another effect program is refused.
pub struct WorthQueryApplicationEffectEntity<Schema, Entity> {
    pub reference: EntityReference,
    pub entity: String,
    pub created_effect: Option<usize>,
    pub program: Arc<()>,
    pub _marker: PhantomData<fn() -> (Schema, Entity)>,
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

    pub fn for_fragment(
        &self,
    ) -> crate::domain_computation::primary_graph::fragment_view::CandidateEntity<Entity> {
        crate::domain_computation::primary_graph::fragment_view::CandidateEntity {
            erased: crate::domain_computation::primary_graph::fragment_view::ErasedCandidate {
                reference: self.reference.clone(),
                entity: self.entity.clone(),
                created_effect: self.created_effect,
                program: Arc::clone(&self.program),
            },
            marker: PhantomData,
        }
    }
}

impl WorthQueryApplicationEffectEntity<(), ()> {
    #[inline(always)]
    pub fn into_typed<S, E>(self) -> WorthQueryApplicationEffectEntity<S, E> {
        WorthQueryApplicationEffectEntity {
            reference: self.reference,
            entity: self.entity,
            created_effect: self.created_effect,
            program: self.program,
            _marker: PhantomData,
        }
    }
}

/// Only the creating effect program can initialize this new relation's payload.
pub struct WorthQueryApplicationCreatedRelation<Schema, Relation, From, To> {
    pub relation:
        worth_query_installation::facade::ApplicationRelationRef<Schema, Relation, From, To>,
    pub created_effect: usize,
    pub program: Arc<()>,
}

pub struct WorthQueryApplicationOptionalFieldWrite {
    pub contract: PortableAspectContractBasis,
    pub value: Option<AspectValue>,
}

pub enum WorthQueryApplicationRealizedEffect {
    CreateEntity {
        kind: KindId,
        key: String,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
        partition: WorthQueryApplicationCreationPartition,
    },
    UpdateEntity {
        entity: String,
        entity_id: EntityId,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
    },
    PatchOptionalEntityFields {
        entity: String,
        entity_id: EntityId,
        fields: BTreeMap<AspectFieldLocator, WorthQueryApplicationOptionalFieldWrite>,
    },
    DeleteEntity {
        entity_id: EntityId,
    },
    CreateRelation {
        kind: KindId,
        key: String,
        from: EntityReference,
        to: EntityReference,
        fields: BTreeMap<AspectFieldLocator, AspectValue>,
    },
    DeleteRelation {
        relation_id: RelationId,
    },
    Emit(WorthQueryApplicationEmission),
    RemoveAspect {
        entity: String,
        entity_id: EntityId,
        contract: PortableAspectContractBasis,
        fields: Vec<AspectFieldLocator>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCreationPartition {
    Issued,
    Context(worth_relational::facade::identity::PartitionId),
}

impl WorthQueryApplicationCreationPartition {
    pub fn resolve(
        self,
        issued: worth_relational::facade::identity::PartitionId,
    ) -> worth_relational::facade::identity::PartitionId {
        match self {
            Self::Issued => issued,
            Self::Context(partition) => partition,
        }
    }
}
