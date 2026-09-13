use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationProgramTarget, OperationCreates,
};
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::{
    candidate_representation_denial, denial, retained_representation, CandidateItemKind,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCreationPartition, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder, WorthQueryApplicationEntityKey,
    WorthQueryApplicationRealizedEffect,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub fn create_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        self.create_entity_at(entity, key, WorthQueryApplicationCreationPartition::Issued)
    }

    pub fn create_entity_in_context<Entity, ContextEntity>(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        if !Arc::ptr_eq(&context.program, &self.program) {
            return Err(foreign_context_denial());
        }
        let EntityReference::Existing(existing) = context.reference else {
            return Err(foreign_context_denial());
        };
        self.create_entity_at(
            entity,
            key,
            WorthQueryApplicationCreationPartition::Context(existing.partition_id),
        )
    }

    fn create_entity_at<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
        partition: WorthQueryApplicationCreationPartition,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        if self
            .creation_partition
            .is_some_and(|selected| selected != partition)
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                "candidate creation partition",
            ));
        }
        self.admit_program_target(&ApplicationOperationProgramTarget::Create {
            entity: entity.name().to_string(),
        })?;
        let kind = self.layout.entity_kind(entity.name()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                entity.name(),
            )
        })?;
        let key = key.into_string();
        if self.keys.contains(&(kind, key.clone())) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
                entity.name(),
            ));
        }
        let retained_bytes = retained_representation::created_entity(&key, entity.name())
            .ok_or_else(candidate_representation_denial)?;
        self.charge_candidate_representation(CandidateItemKind::Create, retained_bytes, 0)?;
        self.keys.insert((kind, key.clone()));
        self.creation_partition = Some(partition);
        let reference = EntityReference::Created(CreatedEntityRef {
            partition_id: partition
                .resolve(worth_relational::facade::identity::PartitionId::main()),
            kind_id: kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
        });
        let created_effect = self.effects.len();
        self.effects
            .push(WorthQueryApplicationRealizedEffect::CreateEntity {
                kind,
                key,
                fields: BTreeMap::new(),
                partition,
            });
        Ok(WorthQueryApplicationEffectEntity {
            reference,
            entity: entity.name().to_string(),
            created_effect: Some(created_effect),
            program: Arc::clone(&self.program),
            _marker: PhantomData,
        })
    }
}

fn foreign_context_denial() -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
        "creation context",
    )
}
