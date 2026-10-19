//! Reuse validated initializers and the retained predecessor, never host semantic tuples.
use super::{
    ApplicationMutationBinding, ApplicationSchema, DecisionReader, HandlerExecutionDenial,
};
use crate::domain_computation::primary_graph::operation_body::{
    CreationDisposition, PreparedBodyInputs, PreparedBodySelections,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyProgram,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_creation_disposition(
        &mut self,
        body: &ApplicationOperationBodyProgram,
        inputs: Option<&PreparedBodyInputs>,
        selections: &PreparedBodySelections<S>,
    ) -> Result<CreationDisposition, HandlerExecutionDenial> {
        let Some(creation) = body.creation().filter(|c| c.reuses_current_if_unchanged()) else {
            return Ok(CreationDisposition::Create);
        };
        let link = &creation.links()[0];
        let Some(selected) = selections.selected(&link.relation, &link.from, &link.to) else {
            return Ok(CreationDisposition::Create);
        };
        let mut input_index = body
            .scope_fields()
            .iter()
            .filter(|f| f.input_member().is_some())
            .count();
        let mut unchanged = true;
        self.context_use.set(self.context_use.get().scope());
        for field in creation.fields() {
            self.checkpoint().map_err(HandlerExecutionDenial::new)?;
            let expected = match field {
                Field::Literal { value, .. } => Some(value),
                Field::Input { .. } | Field::OptionalInput { .. } => {
                    let value =
                        inputs.expect("installed input initializers").values[input_index].as_ref();
                    input_index += 1;
                    value
                }
                _ => unreachable!("installed complete-content reuse profile"),
            };
            unchanged &= self.reader.body_field_equivalent(
                &selected.identity,
                field.field(),
                expected,
                self.execution,
            )?;
        }
        Ok(if unchanged {
            CreationDisposition::Reuse
        } else {
            CreationDisposition::Create
        })
    }
}
