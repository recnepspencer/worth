use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, EqualityPredicate, OperationReads,
    WritePosture,
};

use super::super::{HandlerExecutionDenial, WorthQueryInvariantEntityIdentity};
use super::invariant::DecisionReader;

impl<Schema, Binding> DecisionReader<'_, '_, '_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Resolve through the admitted snapshot and retain the identity-field fact
    /// that later candidate authoring and stale-source comparison require.
    pub fn resolve_entity<Entity, Aspect, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let identity = self
            .reader()
            .resolve_entity(field, value)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader()
            .require_decision_field(&identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(identity)
    }

    /// Read a value while retaining its exact field dependency for the attempt.
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Option<Value>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader()
            .decision_field(identity, field)
            .map_err(HandlerExecutionDenial::new)
    }
}
