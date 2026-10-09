//! The admitted decision target and fact are retained before raw-value admission.
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, HandlerInterruption, WorthQueryInvariantEntityIdentity,
};
use worth_foundational::facade::AspectValue as ApplicationValue;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, OperationReads, WritePosture,
};

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::type_complexity)]
    pub(in crate::domain_computation::primary_graph) fn decision_field_with_predecode_admission<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
        Denied,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        admit: impl for<'raw, 'checkpoint> FnOnce(
            &'raw ApplicationValue,
            &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
        ) -> Result<(), Denied>,
        checkpoint: &dyn Fn() -> Result<(), HandlerInterruption>,
    ) -> Result<Result<Option<Value>, Denied>, HandlerExecutionDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.require_decision_field(identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader
            .field_with_predecode_admission(identity, field, admit, checkpoint)
    }
}
