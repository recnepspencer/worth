//! Compute once from this exact admitted decision reader, before candidate effects.
use super::*;
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    pub(in crate::domain_computation::primary_graph) fn body_values(
        &mut self,
        values: &[crate::domain_computation::primary_graph::operation_body::value::InstalledBodyValue],
    ) -> Result<
        Option<crate::domain_computation::primary_graph::operation_body::value::PreparedBodyValues>,
        HandlerExecutionDenial,
    > {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().scope());
        if values.iter().flat_map(|value| value.program.operands()).any(|operand| matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::PrincipalIdentity { .. })) { let _ = self.principal_identity(); }
        crate::domain_computation::primary_graph::operation_body::value::prepare(
            self.reader,
            self.scope,
            values,
            self.execution,
        )
    }
}
