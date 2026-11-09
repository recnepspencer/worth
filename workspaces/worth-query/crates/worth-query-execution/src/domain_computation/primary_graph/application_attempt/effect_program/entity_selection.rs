use super::{WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEntityIdentity,
    WorthQueryInvariantMutationTarget,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn resolve_observed_entity<
        Entity,
        Aspect,
        Field,
        Value,
    >(
        &self,
        field: Field,
        value: Value,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        Ok(self
            .native_selection()
            .resolve_observed_entity(
                field.identity(),
                &crate::domain_computation::primary_graph::value_decode::ScalarFieldEncoder::<
                    Field::Binding,
                >::new(&value),
            )?
            .into_typed())
    }

    pub fn projected_entity<Entity>(
        &self,
        target: &WorthQueryInvariantMutationTarget<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    {
        Ok(self
            .native_selection()
            .select(
                target.entity_id,
                target.entity.as_ref(),
                target.runtime_authority,
                &target.binding_identity,
                Some((target.admission_identity, target.phase)),
            )?
            .into_typed())
    }

    pub fn existing_entity<Entity>(
        &self,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    {
        Ok(self
            .native_selection()
            .select(
                identity.entity_id(),
                identity.entity_name(),
                identity.runtime_authority(),
                identity.binding_identity(),
                None,
            )?
            .into_typed())
    }
}
