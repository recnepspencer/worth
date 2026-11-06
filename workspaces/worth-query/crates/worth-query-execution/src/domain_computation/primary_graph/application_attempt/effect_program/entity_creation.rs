use worth_query_installation::facade::ApplicationEntityRef;
use worth_query_installation::facade::OperationCreates;

use super::denial;
use super::WorthQueryApplicationAttemptDenial;
use super::WorthQueryApplicationAttemptDenialKind;
use super::WorthQueryApplicationCreationPartition;
use super::WorthQueryApplicationEffectEntity;
use super::WorthQueryApplicationEffectProgramBuilder;
use super::WorthQueryApplicationEntityKey;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    #[inline(always)]
    pub fn create_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        Ok(self
            .native_effects()
            .create_entity_named_at(
                entity.name(),
                key.into_string(),
                WorthQueryApplicationCreationPartition::Issued,
            )?
            .into_typed())
    }

    #[inline(always)]
    pub fn create_entity_in_context<Entity, ContextEntity>(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        Ok(self
            .native_effects()
            .create_entity_in_context(context.borrowed(), entity.name(), key.into_string())?
            .into_typed())
    }

    /// Only the installed native body may enter this dynamic bridge. The
    /// complete body and its Create membership were readmitted at installation.
    pub(in crate::domain_computation::primary_graph) fn create_body_entity_in_context<
        ContextEntity,
    >(
        &mut self,
        context: &WorthQueryApplicationEffectEntity<Schema, ContextEntity>,
        entity: &str,
        key: &str,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, ()>, WorthQueryApplicationAttemptDenial>
    {
        let partition = super::core::context_partition(&self.program, context.borrowed())?;
        let key = WorthQueryApplicationEntityKey::<Schema, ()>::new(key).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                entity,
            )
        })?;
        self.create_entity_named_at(entity, key.into_string(), partition)
    }

    pub(super) fn create_entity_named_at<Entity>(
        &mut self,
        entity_name: &str,
        key: String,
        partition: WorthQueryApplicationCreationPartition,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    {
        Ok(self
            .native_effects()
            .create_entity_named_at(entity_name, key, partition)?
            .into_typed())
    }
}
