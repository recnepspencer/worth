use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit, ApplicationScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue,
};
use worth_relational::facade::{
    branch::{
        RelationalEntityMaterialization, RelationalMaterializationRecord,
        RelationalRelationMaterialization,
    },
    identity::{EntityId, KindId, RelationId},
    transactions::AspectFieldPatch,
};

use super::WorthQuerySuspendedGeneratedOutput;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputRole, WorthQueryApplicationProducerBinding, WorthQueryCreateOutput,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphLayout,
};

mod relations;

pub struct WorthQueryGeneratedEntity<Schema, Entity> {
    identity: EntityId,
    session: Arc<()>,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

pub struct WorthQueryRetainedGeneratedOutputEntity<Schema, Entity> {
    identity: EntityId,
    session: Arc<()>,
    _marker: PhantomData<fn() -> (Schema, Entity)>,
}

impl<Schema, Entity> Clone for WorthQueryGeneratedEntity<Schema, Entity> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity,
            session: Arc::clone(&self.session),
            _marker: PhantomData,
        }
    }
}

pub struct WorthQueryGeneratedOutputReconstruction<'runtime, Schema, Producer> {
    layout: &'runtime WorthQueryPrimaryGraphLayout,
    suspended: WorthQuerySuspendedGeneratedOutput,
    session: Arc<()>,
    entities: BTreeMap<EntityId, ReconstructionEntity>,
    relations: Vec<ReconstructionRelation>,
    _marker: PhantomData<fn() -> (Schema, Producer)>,
}

struct ReconstructionEntity {
    kind: KindId,
    claimed: bool,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
}

struct ReconstructionRelation {
    identity: RelationId,
    kind: KindId,
    source: EntityId,
    target: EntityId,
    claimed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputReconstructionDenial {
    ForeignRuntime,
    ForeignProducer,
    StaleProducerVersion,
    StaleOutputLineage,
    MissingOutputRole,
    MissingEntity,
    EntityKindMismatch,
    DuplicateEntityClaim,
    ForeignEntityHandle,
    RetainedEntityKindMismatch,
    ForeignRetainedEntityHandle,
    UnknownField,
    DuplicateField,
    InvalidFieldValue,
    MissingRelation,
    DuplicateRelationClaim,
    AmbiguousRelation,
    WrongRelationEndpoint,
    IncompleteManifest,
}

pub struct WorthQueryGeneratedOutputReconstructionFailure {
    denial: WorthQueryGeneratedOutputReconstructionDenial,
    suspended: WorthQuerySuspendedGeneratedOutput,
}

impl WorthQueryGeneratedOutputReconstructionFailure {
    pub const fn denial(&self) -> WorthQueryGeneratedOutputReconstructionDenial {
        self.denial
    }

    pub fn into_suspended(self) -> WorthQuerySuspendedGeneratedOutput {
        self.suspended
    }
}

#[must_use = "completed reconstruction must be restored through Query Host"]
pub struct WorthQueryCompletedGeneratedOutputReconstruction<Schema, Producer> {
    pub(super) suspended: WorthQuerySuspendedGeneratedOutput,
    pub(super) entities: Vec<RelationalEntityMaterialization>,
    pub(super) relations: Vec<RelationalRelationMaterialization>,
    pub(super) marker: PhantomData<fn() -> (Schema, Producer)>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn reconstruct_generated_output<Producer>(
        &self,
        suspended: WorthQuerySuspendedGeneratedOutput,
    ) -> Result<
        WorthQueryGeneratedOutputReconstruction<'_, Schema, Producer>,
        WorthQueryGeneratedOutputReconstructionFailure,
    >
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        if !Arc::ptr_eq(
            &suspended.publication.root_identity(),
            &self.product_runtime.root_identity(),
        ) || !suspended.matches_runtime(self)
        {
            return Err(reconstruction_failure(
                suspended,
                WorthQueryGeneratedOutputReconstructionDenial::ForeignRuntime,
            ));
        }
        if !suspended.matches_producer_binding::<Schema, Producer>() {
            return Err(reconstruction_failure(
                suspended,
                WorthQueryGeneratedOutputReconstructionDenial::ForeignProducer,
            ));
        }
        if !suspended.matches_provider_version::<Schema, Producer>()
            || self.installed_producers.provider::<Producer>().is_none()
        {
            return Err(reconstruction_failure(
                suspended,
                WorthQueryGeneratedOutputReconstructionDenial::StaleProducerVersion,
            ));
        }
        if !suspended.matches_retained_lineage::<Schema, Producer>(self) {
            return Err(reconstruction_failure(
                suspended,
                WorthQueryGeneratedOutputReconstructionDenial::StaleOutputLineage,
            ));
        }
        Ok(WorthQueryGeneratedOutputReconstruction::new(
            &self.primary_provider.graph.layout,
            suspended,
        ))
    }
}

impl<'runtime, Schema, Producer> WorthQueryGeneratedOutputReconstruction<'runtime, Schema, Producer>
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    pub fn abort(self) -> WorthQuerySuspendedGeneratedOutput {
        self.suspended
    }

    fn new(
        layout: &'runtime WorthQueryPrimaryGraphLayout,
        suspended: WorthQuerySuspendedGeneratedOutput,
    ) -> Self {
        let mut entities = BTreeMap::new();
        let mut relations = Vec::new();
        for record in suspended.custody_records() {
            match *record {
                RelationalMaterializationRecord::Entity { entity_id, kind_id } => {
                    assert!(
                        entities
                            .insert(
                                entity_id,
                                ReconstructionEntity {
                                    kind: kind_id,
                                    claimed: false,
                                    fields: BTreeMap::new(),
                                },
                            )
                            .is_none(),
                        "materialization manifest is canonical"
                    );
                }
                RelationalMaterializationRecord::Relation {
                    relation_id,
                    kind_id,
                    source,
                    target,
                } => relations.push(ReconstructionRelation {
                    identity: relation_id,
                    kind: kind_id,
                    source,
                    target,
                    claimed: false,
                }),
            }
        }
        Self {
            layout,
            suspended,
            session: Arc::new(()),
            entities,
            relations,
            _marker: PhantomData,
        }
    }

    pub fn entity<Entity>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Producer::Operation, Entity, WorthQueryCreateOutput>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Result<
        WorthQueryGeneratedEntity<Schema, Entity>,
        WorthQueryGeneratedOutputReconstructionDenial,
    >
    where
        Entity: 'static,
    {
        let identity = self
            .suspended
            .correspondence()
            .entity(role)
            .map_err(|_| WorthQueryGeneratedOutputReconstructionDenial::MissingOutputRole)?
            .entity_id();
        let expected = self
            .layout
            .entity_kind(entity.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::EntityKindMismatch)?;
        let reconstructed = self
            .entities
            .get_mut(&identity)
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingEntity)?;
        if reconstructed.kind != expected {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::EntityKindMismatch);
        }
        if std::mem::replace(&mut reconstructed.claimed, true) {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::DuplicateEntityClaim);
        }
        Ok(WorthQueryGeneratedEntity {
            identity,
            session: Arc::clone(&self.session),
            _marker: PhantomData,
        })
    }

    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        entity: &WorthQueryGeneratedEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.validate_handle(entity)?;
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::UnknownField)?;
        let encoded = Field::Binding::encode(&value)
            .map_err(|_| WorthQueryGeneratedOutputReconstructionDenial::InvalidFieldValue)?;
        let reconstructed = self
            .entities
            .get_mut(&entity.identity)
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingEntity)?;
        if reconstructed.fields.contains_key(&locator) {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::DuplicateField);
        }
        reconstructed.fields.insert(locator, encoded);
        Ok(())
    }

    fn validate_handle<Entity>(
        &self,
        entity: &WorthQueryGeneratedEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        if !Arc::ptr_eq(&self.session, &entity.session) {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::ForeignEntityHandle);
        }
        Ok(())
    }

    fn validate_retained_handle<Entity>(
        &self,
        entity: &WorthQueryRetainedGeneratedOutputEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        if !Arc::ptr_eq(&self.session, &entity.session) {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::ForeignRetainedEntityHandle);
        }
        Ok(())
    }

    pub fn finish(
        self,
    ) -> Result<
        WorthQueryCompletedGeneratedOutputReconstruction<Schema, Producer>,
        WorthQueryGeneratedOutputReconstructionFailure,
    > {
        if self.entities.values().any(|entity| !entity.claimed)
            || self.relations.iter().any(|relation| !relation.claimed)
        {
            return Err(reconstruction_failure(
                self.suspended,
                WorthQueryGeneratedOutputReconstructionDenial::IncompleteManifest,
            ));
        }
        let entities = self
            .entities
            .into_iter()
            .map(|(entity_id, entity)| RelationalEntityMaterialization {
                entity_id,
                kind_id: entity.kind,
                fields: AspectFieldPatch::new(entity.fields),
            })
            .collect();
        let relations = self
            .relations
            .into_iter()
            .map(|relation| RelationalRelationMaterialization {
                relation_id: relation.identity,
                kind_id: relation.kind,
                source: relation.source,
                target: relation.target,
                fields: AspectFieldPatch::default(),
            })
            .collect();
        Ok(WorthQueryCompletedGeneratedOutputReconstruction {
            suspended: self.suspended,
            entities,
            relations,
            marker: PhantomData,
        })
    }
}

fn reconstruction_failure(
    suspended: WorthQuerySuspendedGeneratedOutput,
    denial: WorthQueryGeneratedOutputReconstructionDenial,
) -> WorthQueryGeneratedOutputReconstructionFailure {
    WorthQueryGeneratedOutputReconstructionFailure { denial, suspended }
}
