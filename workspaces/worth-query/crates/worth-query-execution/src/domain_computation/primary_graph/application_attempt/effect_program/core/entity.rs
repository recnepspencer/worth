//! Entity creation validates context, authority and key uniqueness before retention.
use super::*;
use std::marker::PhantomData;
use worth_relational::facade::transactions::CreatedEntityRef;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn create_entity_in_context(
        &mut self,
        context: Target<'_>,
        entity: &str,
        key: String,
    ) -> Result<WorthQueryApplicationEffectEntity<(), ()>, WorthQueryApplicationAttemptDenial> {
        let partition = context_partition(self.program, context)?;
        self.create_entity_named_at(entity, key, partition)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn create_entity_named_at(
        &mut self,
        entity_name: &str,
        key: String,
        partition: WorthQueryApplicationCreationPartition,
    ) -> Result<WorthQueryApplicationEffectEntity<(), ()>, WorthQueryApplicationAttemptDenial> {
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
            entity: entity_name.to_string(),
        })?;
        let kind = self.layout.entity_kind(entity_name).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                entity_name,
            )
        })?;
        if self.keys.contains(&(kind, key.clone())) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
                entity_name,
            ));
        }
        self.reserve_effects(1)?;
        self.keys.insert((kind, key.clone()));
        *self.creation_partition = Some(partition);
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
                fields: std::collections::BTreeMap::new(),
                partition,
            });
        Ok(WorthQueryApplicationEffectEntity {
            reference,
            entity: entity_name.to_string(),
            created_effect: Some(created_effect),
            program: Arc::clone(self.program),
            _marker: PhantomData,
        })
    }
}
pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn context_partition(
    program: &Arc<()>,
    context: Target<'_>,
) -> Result<WorthQueryApplicationCreationPartition, WorthQueryApplicationAttemptDenial> {
    if !Arc::ptr_eq(context.program, program) {
        return Err(foreign_context_denial());
    }
    let EntityReference::Existing(existing) = context.reference else {
        return Err(foreign_context_denial());
    };
    Ok(WorthQueryApplicationCreationPartition::Context(
        existing.partition_id,
    ))
}
fn foreign_context_denial() -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
        "creation context",
    )
}
