//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn field_with_predecode_admission_for_test<Entity, Aspect, Field, Value, Denied>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
        admit: impl for<'raw, 'checkpoint> FnOnce(
            &'raw ApplicationValue,
            &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
        ) -> Result<(), Denied>,
        checkpoint: &dyn Fn() -> Result<(), HandlerInterruption>,
    ) -> Result<Result<Option<Value>, Denied>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        Self::field_with_predecode_admission(self, identity, field, admit, checkpoint)
    }
}
