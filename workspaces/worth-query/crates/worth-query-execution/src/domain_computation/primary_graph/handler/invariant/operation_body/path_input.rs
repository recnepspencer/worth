//! Path selection and the terminal comparison share the native decision observation.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyPathFieldSource;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_path_input_equals(
        &mut self,
        source: &ApplicationOperationBodyPathFieldSource,
        expected: &AspectValue,
    ) -> Result<bool, HandlerExecutionDenial> {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().scope());
        self.execution
            .decision_stage(|context| {
                let selected = self
                    .reader
                    .body_path_selection(self.scope, &source.path, self.execution, context)?
                    .ok_or_else(|| {
                        HandlerExecutionDenial::new(Denial::new(
                            Kind::MissingAuthoritativeFact,
                            source.path.terminal_entity(),
                        ))
                    })?;
                self.reader.body_field(
                    &selected.identity,
                    &source.field,
                    BodyFieldExpectation::Equals(expected),
                )
            })
            .map_err(HandlerExecutionDenial::new)?
    }
}
