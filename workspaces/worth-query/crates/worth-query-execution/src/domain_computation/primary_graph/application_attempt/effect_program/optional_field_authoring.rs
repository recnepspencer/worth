//! Typed optional field shell; concrete patch semantics belong to the effect core.
use super::{WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;
impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// Authors an exact presence change for an installed optional field.
    /// `Some(value)` preserves even empty or zero values; `None` clears the
    /// field. The field must remain an admitted decision read so absence or
    /// value drift participates in compare-and-commit currentness.
    #[inline(always)]
    pub fn write_optional_field<Entity, Field>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: Field,
        value: Option<Field::Value>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
        >,
    {
        match value {
            Some(value) => self.native_effects().write_optional_field(
                target.borrowed(),
                field.identity(),
                Some(
                    &crate::domain_computation::primary_graph::value_decode::ScalarFieldEncoder::<
                        Field::Binding,
                    >::new(&value),
                ),
            ),
            None => self.native_effects().write_optional_field(
                target.borrowed(),
                field.identity(),
                None,
            ),
        }
    }
}
