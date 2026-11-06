use super::WorthQueryApplicationEffectEntity;
use super::WorthQueryApplicationEffectProgramBuilder;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    #[inline(always)]
    pub fn write_field<Entity, Field>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: Field,
        value: Field::Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
        >,
    {
        self.native_effects().write_field(
            target.borrowed(),
            field.identity(),
            &crate::domain_computation::primary_graph::value_decode::ScalarFieldEncoder::<
                Field::Binding,
            >::new(&value),
        )
    }
}
