//! The admitted decision target and fact are retained before raw-value admission.
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, HandlerInterruption, WorthQueryInvariantEntityIdentity,
};
use worth_foundational::facade::AspectValue as ApplicationValue;
use worth_query_installation::facade::{ApplicationReadableScalarValueBinding, ApplicationSchema};

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
        Denied,
    >(
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
        self.require_decision_field(identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader
            .field_with_predecode_admission(identity, field, admit, checkpoint)
    }
}

mod court_doors_for_test;
